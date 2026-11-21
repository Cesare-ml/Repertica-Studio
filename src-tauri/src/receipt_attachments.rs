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
fn err(e: impl std::fmt::Display) -> String { e.to_string() }

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
fn data_core(db: &Database, receipt_id: &str) -> Result<String, String> {
    // Hold the SQLite mutex while reading to serialize with backup and attachment removal.
    let conn = db.conn.lock().map_err(err)?;
    let attachment: ReceiptAttachment = conn.query_row(
        &format!("SELECT {FIELDS} FROM sale_receipt_attachments WHERE receipt_id=?1"),
        params![receipt_id], row,
    ).optional().map_err(err)?.ok_or("Allegato non trovato")?;
    media_directory(db)?;
    if !safe_media_path(&attachment.media_path) { return Err("Percorso allegato non sicuro".into()); }
    let file = db.directory.join(&attachment.media_path);
    let size = regular_file(&file)?;
    if size == 0 || size > MAX_BYTES as u64 || size != attachment.size_bytes as u64 {
        return Err("Dimensione allegato non valida".into());
    }
    let mut bytes = Vec::new();
    File::open(&file).map_err(err)?.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes).map_err(err)?;
    if bytes.len() != size as usize || mime(&bytes).map(|(_,m)| m) != Some(attachment.mime_type.as_str()) {
        return Err("Formato allegato non valido".into());
    }
    if format!("{:x}", Sha256::digest(&bytes)) != attachment.sha256 {
        return Err("Impronta SHA-256 allegato non valida".into());
    }
    Ok(format!("data:{};base64,{}", attachment.mime_type, STANDARD.encode(bytes)))
}
// Removal occurs only for files stored with the generated receipt-UUID path.
// An I/O failure after the database commit is reported explicitly: the orphan
// remains in media/ and is retained by full backups, never silently discarded.
pub(crate) fn cleanup_unlinked(db: &Database, relative: &str) -> Result<(), String> {
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
    let relative: String = conn.query_row(
        "SELECT media_path FROM sale_receipt_attachments WHERE receipt_id=?1",
        params![receipt_id], |r| r.get(0),
    ).optional().map_err(err)?.ok_or("Allegato non trovato")?;
    if !safe_media_path(&relative) { return Err("Percorso allegato non sicuro".into()); }
    conn.execute("DELETE FROM sale_receipt_attachments WHERE receipt_id=?1", params![receipt_id])
        .map_err(err)?;
    cleanup_unlinked(db, &relative)
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
        assert_eq!(summary["schemaVersion"], 6);
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
