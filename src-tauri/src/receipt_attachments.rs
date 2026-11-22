//! Local evidence for manually recorded sale receipts.
//! No network access; files are copied into the same media folder as artwork images.
use crate::db::Database;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path},
};
use tauri::State;
use uuid::Uuid;

const MAX_BYTES: usize = 12 * 1024 * 1024;
const FIELDS: &str = "receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at";
fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

#[derive(Debug, Clone, Serialize)]
pub struct ReceiptAttachment {
    pub receipt_id: String,
    pub media_path: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub created_at: String,
}
fn row(r: &Row<'_>) -> rusqlite::Result<ReceiptAttachment> {
    Ok(ReceiptAttachment {
        receipt_id: r.get(0)?, media_path: r.get(1)?,
        original_name: r.get(2)?, mime_type: r.get(3)?,
        size_bytes: r.get(4)?, sha256: r.get(5)?, created_at: r.get(6)?,
    })
}
pub(crate) fn mime(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(b"%PDF-") { Some(("pdf", "application/pdf")) }
    else if bytes.starts_with(&[0xff,0xd8,0xff]) { Some(("jpg", "image/jpeg")) }
    else if bytes.starts_with(&[137,80,78,71,13,10,26,10]) { Some(("png", "image/png")) }
    else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(("webp", "image/webp"))
    } else { None }
}
pub(crate) fn safe_media_path(relative: &str) -> bool {
    let mut parts = Path::new(relative).components();
    if !matches!(parts.next(), Some(Component::Normal(dir)) if dir == "media") {
        return false;
    }
    let name = match parts.next() {
        Some(Component::Normal(name)) => name.to_string_lossy(),
        _ => return false,
    };
    parts.next().is_none() && !relative.contains('\\')
        && name.starts_with("receipt-")
        && name.len() <= 100
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && ["pdf", "jpg", "png", "webp"].iter().any(|ext| name.ends_with(format!(".{ext}").as_str()))
}
fn regular_file(path: &Path) -> Result<u64, String> {
    let metadata = fs::symlink_metadata(path).map_err(err)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Il documento non è un file regolare".into());
    }
    Ok(metadata.len())
}
fn media_directory(db: &Database) -> Result<(), String> {
    let directory = db.directory.join("media");
    let kind = fs::symlink_metadata(directory).map_err(err)?.file_type();
    if !kind.is_dir() || kind.is_symlink() {
        return Err("La cartella media non è una directory regolare".into());
    }
    Ok(())
}
fn get_core(db: &Database, receipt_id: &str) -> Result<Option<ReceiptAttachment>, String> {
    let conn = db.conn.lock().map_err(err)?;
    conn.query_row(
        &format!("SELECT {FIELDS} FROM sale_receipt_attachments WHERE receipt_id=?1"),
        params![receipt_id], row,
    ).optional().map_err(err)
}
fn attach_core(db: &Database, receipt_id: &str, source: &Path) -> Result<ReceiptAttachment, String> {
    if !source.is_absolute() { return Err("Seleziona un file tramite il dialogo nativo".into()); }
    let name = source.file_name().and_then(|v| v.to_str()).ok_or("Nome del documento non valido")?;
    if name.is_empty() || name.chars().count() > 240 || name.chars().any(char::is_control) {
        return Err("Nome del documento non valido".into());
    }
    let size = regular_file(source)?;
    if size == 0 || size > MAX_BYTES as u64 {
        return Err("Documento vuoto o troppo grande (massimo 12 MiB)".into());
    }
    let mut bytes = Vec::new();
    File::open(source).map_err(err)?.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes).map_err(err)?;
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return Err("Documento vuoto o troppo grande (massimo 12 MiB)".into());
    }
    let (ext, mime_type) = mime(&bytes).ok_or("Sono consentiti solo PDF, JPEG, PNG e WebP")?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let conn = db.conn.lock().map_err(err)?;
    media_directory(db)?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sale_receipts WHERE id=?1)",
        params![receipt_id], |r| r.get(0),
    ).map_err(err)?;
    if !exists { return Err("Incasso non trovato".into()); }
    let already: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sale_receipt_attachments WHERE receipt_id=?1)",
        params![receipt_id], |r| r.get(0),
    ).map_err(err)?;
    if already { return Err("Questo incasso ha già un allegato".into()); }

    let relative = format!("media/receipt-{}.{}", Uuid::new_v4(), ext);
    let target = db.directory.join(&relative);
    let mut output = OpenOptions::new().write(true).create_new(true).open(&target).map_err(err)?;
    let written = output.write_all(&bytes).and_then(|_| output.sync_all());
    drop(output);
    if let Err(e) = written { let _ = fs::remove_file(&target); return Err(err(e)); }

    let stamp = Utc::now().to_rfc3339();
    let inserted = conn.execute(
        "INSERT INTO sale_receipt_attachments
        (receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at)
        VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![receipt_id,relative,name,mime_type,bytes.len() as i64,digest,stamp],
    );
    if let Err(e) = inserted { let _ = fs::remove_file(&target); return Err(err(e)); }
    conn.query_row(&format!("SELECT {FIELDS} FROM sale_receipt_attachments WHERE receipt_id=?1"),
        params![receipt_id], row).map_err(err)
}
// Used by both viewing/exporting and by destructive unlinking. A manually
// replaced or corrupt file must not be discarded just because the record's
// media_path still looks safe.
fn verified_attachment_bytes(db: &Database, attachment: &ReceiptAttachment) -> Result<Vec<u8>, String> {
    media_directory(db)?;
    if !safe_media_path(&attachment.media_path) {
        return Err("Percorso allegato non sicuro".into());
    }
    let path = db.directory.join(&attachment.media_path);
    let size = regular_file(&path)?;
    if size == 0 || size > MAX_BYTES as u64 || size != attachment.size_bytes as u64 {
        return Err("Dimensione allegato non valida".into());
    }
    let mut bytes = Vec::new();
    File::open(&path).map_err(err)?.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes).map_err(err)?;
    if bytes.len() != size as usize || mime(&bytes).map(|(_, m)| m) != Some(attachment.mime_type.as_str()) {
        return Err("Formato allegato non valido".into());
    }
    if format!("{:x}", Sha256::digest(&bytes)) != attachment.sha256 {
        return Err("Impronta SHA-256 allegato non valida".into());
    }
    Ok(bytes)
}
fn data_core(db: &Database, receipt_id: &str) -> Result<String, String> {
    // Hold the SQLite mutex while reading to serialize with backup and attachment removal.
    let conn = db.conn.lock().map_err(err)?;
    let attachment: ReceiptAttachment = conn.query_row(
        &format!("SELECT {FIELDS} FROM sale_receipt_attachments WHERE receipt_id=?1"),
        params![receipt_id], row,
    ).optional().map_err(err)?.ok_or("Allegato non trovato")?;
    let bytes = verified_attachment_bytes(db, &attachment)?;
    Ok(format!("data:{};base64,{}", attachment.mime_type, STANDARD.encode(bytes)))
}
// Imported or manually edited archives can point an artwork, collector, RAW,
// catalogue revision or historical certificate at a receipt-named file.
// Removing the receipt must never destroy media still used elsewhere. An unreadable
// certificate snapshot is conservatively treated as a possible reference.
fn referenced_elsewhere(
    conn: &rusqlite::Connection, relative: &str, receipt_id: &str,
) -> Result<bool, String> {
    let linked: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sale_receipt_attachments
            WHERE media_path=?1 AND receipt_id<>?2)
          OR EXISTS(SELECT 1 FROM artworks WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM artwork_images WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM collector_images WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM media_revisions WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM artwork_raw_files WHERE media_path=?1)",
        params![relative, receipt_id], |r| r.get(0),
    ).map_err(err)?;
    if linked { return Ok(true); }

    let mut statement = conn.prepare("SELECT snapshot_json FROM certificates").map_err(err)?;
    let snapshots = statement.query_map([], |r| r.get::<_, String>(0)).map_err(err)?;
    for snapshot in snapshots {
        let value: serde_json::Value = match serde_json::from_str(&snapshot.map_err(err)?) {
            Ok(value) => value,
            Err(_) => return Ok(true),
        };
        let Some(fields) = value.as_object() else { return Ok(true); };
        match fields.get("image_path") {
            Some(serde_json::Value::String(path)) if path == relative => return Ok(true),
            Some(serde_json::Value::String(_)) | Some(serde_json::Value::Null) | None => {}
            Some(_) => return Ok(true),
        }
    }
    Ok(false)
}

// An I/O failure after the database commit is reported explicitly: the orphan
// remains in media/ and is retained by full backups, never silently discarded.
fn cleanup_unlinked(db: &Database, relative: &str) -> Result<(), String> {
    if !safe_media_path(relative) { return Err("Percorso allegato non sicuro: file conservato".into()); }
    media_directory(db)?;
    let path = db.directory.join(relative);
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Collegamento rimosso, ma il file è ancora nell'archivio: {e}")),
    }
}
fn detach_core(db: &Database, receipt_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let attachment: ReceiptAttachment = conn.query_row(
        &format!("SELECT {FIELDS} FROM sale_receipt_attachments WHERE receipt_id=?1"),
        params![receipt_id], row,
    ).optional().map_err(err)?.ok_or("Allegato non trovato")?;
    if !safe_media_path(&attachment.media_path) {
        return Err("Percorso allegato non sicuro".into());
    }
    // Check the directory and all other references BEFORE modifying SQLite.
    // If the media directory was replaced or a query fails, retain the link.
    media_directory(db)?;
    let still_used = referenced_elsewhere(&conn, &attachment.media_path, receipt_id)?;
    if !still_used {
        // Imported archives and external edits may contain a replaced file,
        // including a dangling symlink. Never silently delete unverified bytes.
        // A genuinely missing file may still be unlinked to repair a broken
        // attachment record; the cleanup step already tolerates NotFound.
        let target = db.directory.join(&attachment.media_path);
        match fs::symlink_metadata(&target) {
            Ok(_) => { verified_attachment_bytes(db, &attachment)?; }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(err(e)),
        }
    }
    conn.execute("DELETE FROM sale_receipt_attachments WHERE receipt_id=?1", params![receipt_id])
        .map_err(err)?;
    if still_used { return Ok(()); }
    cleanup_unlinked(db, &attachment.media_path)
}
#[tauri::command]
pub fn get_sale_receipt_attachment(db: State<'_, Database>, receipt_id: String)
    -> Result<Option<ReceiptAttachment>, String> { get_core(&db, &receipt_id) }
#[tauri::command]
pub fn attach_sale_receipt_file(db: State<'_, Database>, receipt_id: String, source_path: String)
    -> Result<ReceiptAttachment, String> { attach_core(&db, &receipt_id, Path::new(&source_path)) }
#[tauri::command]
pub fn sale_receipt_attachment_data(db: State<'_, Database>, receipt_id: String)
    -> Result<String, String> { data_core(&db, &receipt_id) }
#[tauri::command]
pub fn remove_sale_receipt_attachment(db: State<'_, Database>, receipt_id: String)
    -> Result<(), String> { detach_core(&db, &receipt_id) }


fn export_core(db: &Database, receipt_id: &str, destination: &Path) -> Result<String, String> {
    // Reuse the read path: it validates the stored file type, length and original
    // SHA-256 *before* any external file is written. No original source path is used.
    let url = data_core(db, receipt_id)?;
    let (prefix, encoded) = url.split_once(";base64,").ok_or("Formato documento interno non valido")?;
    let ext = match prefix {
        "data:application/pdf" => "pdf",
        "data:image/jpeg" => "jpg",
        "data:image/png" => "png",
        "data:image/webp" => "webp",
        _ => return Err("Tipo documento non consentito".into()),
    };
    if !destination.is_absolute()
        || !destination.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case(ext)) {
        return Err(format!("Seleziona un percorso assoluto con estensione .{ext}"));
    }
    let parent = destination.parent().ok_or("Cartella di esportazione non valida")?;
    let parent = fs::canonicalize(parent).map_err(err)?;
    if !parent.is_dir() { return Err("Cartella di esportazione non valida".into()); }
    let live = fs::canonicalize(&db.directory).map_err(err)?;
    if parent.starts_with(live) {
        return Err("Non esportare documenti all'interno dell'archivio attivo".into());
    }
    let name = destination.file_name().ok_or("Nome file non valido")?;
    let target = parent.join(name);
    let bytes = STANDARD.decode(encoded).map_err(err)?;
    let mut output = OpenOptions::new().write(true).create_new(true)
        .open(&target).map_err(err)?;
    let written = output.write_all(&bytes).and_then(|_| output.sync_all());
    drop(output);
    if let Err(e) = written {
        let _ = fs::remove_file(&target);
        return Err(err(e));
    }
    Ok(crate::util::display_path(&target))
}
#[tauri::command]
pub fn export_sale_receipt_attachment(
    db: State<'_, Database>, receipt_id: String, destination: String,
) -> Result<String, String> {
    export_core(&db, &receipt_id, Path::new(&destination))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup(db: &Database) {
        let c = db.conn.lock().unwrap();
        c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
            VALUES('a','A','Opera','t','t')", []).unwrap();
        c.execute("INSERT INTO collectors(id,name,created_at,updated_at)
            VALUES('c','Cliente','t','t')", []).unwrap();
        c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
            VALUES('s','a','c',1000,'2026-11-21','t')", []).unwrap();
        c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('r','s','2026-11-21',400,'cash','t')", []).unwrap();
    }
    #[test]
    fn attachment_is_linked_checked_and_explicitly_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("ricevuta.pdf");
        fs::write(&source, b"%PDF-1.7\nreceipt").unwrap();
        let saved = attach_core(&db, "r", &source).unwrap();
        assert_eq!(saved.original_name, "ricevuta.pdf");
        assert_eq!(saved.mime_type, "application/pdf");
        assert_eq!(saved.size_bytes, 16);
        assert!(data_core(&db, "r").unwrap().starts_with("data:application/pdf;base64,"));
        let exported = tmp.path().join("copy.pdf");
        // The reported path is canonical without the Windows verbatim prefix.
        let reported = export_core(&db, "r", &exported).unwrap();
        assert_eq!(reported, crate::util::display_path(&fs::canonicalize(&exported).unwrap()));
        assert!(!reported.starts_with(r"\\?\"));
        assert_eq!(fs::read(&exported).unwrap(), b"%PDF-1.7\nreceipt");
        assert!(export_core(&db, "r", &exported).is_err()); // never overwrite
        assert!(export_core(&db, "r", &tmp.path().join("wrong.png")).is_err());
        assert!(export_core(&db, "r", &db.directory.join("media/private.pdf")).is_err());
        assert!(attach_core(&db, "r", &source).is_err());
        assert!(attach_core(&db, "unknown", &source).is_err());
        assert!(db.conn.lock().unwrap().execute("DELETE FROM sale_receipts WHERE id='r'", []).is_err());
        detach_core(&db, "r").unwrap();
        assert!(get_core(&db, "r").unwrap().is_none());
        assert!(!db.directory.join(saved.media_path).exists());
        db.conn.lock().unwrap().execute("DELETE FROM sale_receipts WHERE id='r'", []).unwrap();
    }
    #[test]
    fn rejects_bad_type_empty_and_symlink_inputs() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("bad.bin");
        fs::write(&source, b"something else").unwrap();
        assert!(attach_core(&db, "r", &source).is_err());
        fs::write(&source, b"").unwrap();
        assert!(attach_core(&db, "r", &source).is_err());
        #[cfg(unix)] {
            let alias = tmp.path().join("alias.pdf");
            std::os::unix::fs::symlink(&source, &alias).unwrap();
            assert!(attach_core(&db, "r", &alias).is_err());
        }
    }
    #[test]
    fn detaching_a_reused_receipt_file_preserves_artwork_and_collector_media() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("evidence.pdf");
        fs::write(&source, b"%PDF-1.7\nevidence").unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        let stored = db.directory.join(&attachment.media_path);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE artworks SET image_path=?1 WHERE id='a'",
                params![&attachment.media_path]).unwrap();
            conn.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at)
                 VALUES('shared','c','PHOTO',?1,'t')",
                params![&attachment.media_path]).unwrap();
        }
        detach_core(&db, "r").unwrap();
        assert!(get_core(&db, "r").unwrap().is_none());
        assert!(stored.exists(), "a reused file must survive detaching one receipt");
        assert_eq!(fs::read(stored).unwrap(), b"%PDF-1.7\nevidence");
    }

    #[test]
    fn malformed_certificate_history_never_authorizes_receipt_file_deletion() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("proof.pdf");
        fs::write(&source, b"%PDF-1.7\nevidence").unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                 VALUES('cert','a','CERT-1','2026-11-21','Artist','{','t')", []
            ).unwrap();
        }
        detach_core(&db, "r").unwrap();
        assert!(get_core(&db, "r").unwrap().is_none());
        assert!(db.directory.join(attachment.media_path).exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_replaced_media_directory_cannot_unlink_a_receipt_in_sqlite() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("proof.pdf");
        fs::write(&source, b"%PDF-1.7\nevidence").unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        let media = db.directory.join("media");
        let original = db.directory.join("media-saved");
        fs::rename(&media, &original).unwrap();
        let external = tmp.path().join("external");
        fs::create_dir(&external).unwrap();
        std::os::unix::fs::symlink(&external, &media).unwrap();
        assert!(detach_core(&db, "r").is_err());
        assert!(get_core(&db, "r").unwrap().is_some(), "SQL link must survive unsafe media");
        let filename = Path::new(&attachment.media_path).file_name().unwrap();
        assert!(original.join(filename).exists());
        assert_eq!(fs::read_dir(&external).unwrap().count(), 0);
    }

    #[test]
    fn altered_evidence_cannot_be_silently_deleted_or_unlinked() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let original = b"%PDF-1.7\\nproof-A";
        let source = tmp.path().join("proof.pdf");
        fs::write(&source, original).unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        let stored = db.directory.join(&attachment.media_path);

        // Same size and same PDF signature: only the SHA-256 detects this edit.
        fs::write(&stored, b"%PDF-1.7\\nproof-B").unwrap();
        let error = detach_core(&db, "r").unwrap_err();
        assert!(error.contains("SHA-256"), "{error}");
        assert!(get_core(&db, "r").unwrap().is_some(), "retain DB reference on mismatch");
        assert!(stored.exists(), "never remove the altered file");

        fs::write(&stored, original).unwrap();
        detach_core(&db, "r").unwrap();
        assert!(get_core(&db, "r").unwrap().is_none());
        assert!(!stored.exists());
    }

    #[cfg(unix)]
    #[test]
    fn replacing_stored_evidence_with_a_symlink_does_not_unlink_the_record() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("proof.pdf");
        fs::write(&source, b"%PDF-1.7\\nproof").unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        let stored = db.directory.join(&attachment.media_path);
        fs::remove_file(&stored).unwrap();
        let external = tmp.path().join("external.pdf");
        fs::write(&external, b"%PDF-1.7\\nproof").unwrap();
        std::os::unix::fs::symlink(&external, &stored).unwrap();

        assert!(detach_core(&db, "r").is_err());
        assert!(get_core(&db, "r").unwrap().is_some());
        assert!(fs::symlink_metadata(&stored).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&external).unwrap(), b"%PDF-1.7\\nproof");
    }

    #[test]
    fn missing_evidence_can_be_unlinked_without_recreating_the_file() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("proof.pdf");
        fs::write(&source, b"%PDF-1.7\\nproof").unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        let stored = db.directory.join(&attachment.media_path);
        fs::remove_file(&stored).unwrap();
        detach_core(&db, "r").unwrap();
        assert!(get_core(&db, "r").unwrap().is_none());
        assert!(!stored.exists());
    }

    #[test]
    fn backup_rejects_missing_receipt_file_even_with_matching_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("active")).unwrap();
        setup(&db);
        let source = tmp.path().join("proof.pdf");
        fs::write(&source, b"%PDF-1.7\nreceipt").unwrap();
        let attachment = attach_core(&db, "r", &source).unwrap();
        let dest = tmp.path().join("backups");
        fs::create_dir(&dest).unwrap();
        let backup = crate::backup::create_backup_core(&db, &dest).unwrap();
        let summary = serde_json::to_value(&backup).unwrap();
        let directory = summary["directory"].as_str().unwrap();
        let folder = Path::new(directory);
        assert_eq!(summary["schemaVersion"], crate::db::SCHEMA_VERSION);
        assert_eq!(serde_json::to_value(crate::backup::inspect_backup_core(folder).unwrap()).unwrap()["verified"], true);
        let manifest_path = folder.join("manifest.json");
        let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["files"].as_array_mut().unwrap().retain(|v|v["path"] != attachment.media_path);
        fs::remove_file(folder.join(&attachment.media_path)).unwrap();
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
        let error = crate::backup::inspect_backup_core(folder).unwrap_err();
        assert!(error.contains(&attachment.media_path), "{error}");
        assert!(crate::backup::restore_backup_core(&db, folder).is_err());
        assert!(db.directory.join(&attachment.media_path).exists());
    }
}
