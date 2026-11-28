//! One RAW camera file can be attached to an artwork. The file is copied into the private media
//! folder (streamed, so large files never sit in memory), identified by its SHA-256 and
//! included in backups. Only files that really look like photographic RAW formats are accepted.
use crate::db::Database;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri::{Manager, State};
use uuid::Uuid;

/// Largest RAW file accepted (1 GiB).
pub const MAX_RAW_BYTES: u64 = 1 << 30;
const EXTENSIONS: [&str; 29] = [
    "cr2", "cr3", "crw", "nef", "nrw", "arw", "srf", "sr2", "dng", "orf", "rw2", "raf", "pef", "srw", "3fr",
    "fff", "erf", "kdc", "dcr", "mrw", "x3f", "iiq", "rwl", "raw", "gpr", "mef", "mos", "cap", "rwz",
];

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RawFile {
    pub artwork_id: String,
    pub media_path: String,
    pub original_name: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub created_at: String,
}

/// Most RAW formats are TIFF containers; a few have their own signature.
fn looks_like_raw(header: &[u8]) -> bool {
    let starts = |magic: &[u8]| header.starts_with(magic);
    starts(b"II*\0") || starts(b"MM\0*")                       // TIFF-based: CR2, NEF, ARW, DNG, PEF, SRW, 3FR…
        || starts(b"IIRO") || starts(b"IIRS") || starts(b"MMOR") // Olympus ORF
        || starts(b"IIU\0")                                      // Panasonic RW2
        || starts(b"FUJIFILMCCD-RAW")                            // Fujifilm RAF
        || starts(b"FOVb")                                       // Sigma X3F
        || starts(b"\0MRM")                                      // Minolta MRW
        || (header.len() >= 14 && starts(b"II\x1a\0") && &header[6..14] == b"HEAPCCDR") // Canon CRW
        || (header.len() >= 12 && &header[4..8] == b"ftyp")      // ISO media: Canon CR3
}

fn extension_of(path: &Path) -> Option<String> {
    path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).filter(|e| EXTENSIONS.contains(&e.as_str()))
}

/// Reject a media directory redirected outside of this local archive.
fn regular_media_dir(directory: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(directory.join("media")).map_err(err)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("La cartella media non è una directory regolare".into());
    }
    Ok(())
}

/// Never trust database paths blindly after an import or external file modification.
fn regular_raw_path(directory: &Path, relative: &str) -> Result<PathBuf, String> {
    let name = relative.strip_prefix("media/raw-").ok_or("Percorso RAW non valido")?;
    if name.is_empty() || name.len() > 100 || name.contains("..")
        || name.contains(['/', '\\'])
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || extension_of(Path::new(name)).is_none()
    {
        return Err("Percorso RAW non valido".into());
    }
    regular_media_dir(directory)?;
    let path = directory.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(err)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Il RAW archiviato non è un file regolare".into());
    }
    Ok(path)
}

/// A validated RAW file already copied into the media folder, not yet linked to an artwork.
#[derive(Debug)]
pub struct StagedRaw {
    pub relative: String,
    pub original_name: String,
    pub size_bytes: u64,
    pub sha256: String,
    full: PathBuf,
}

impl StagedRaw {
    /// Remove the copied file (after a failed operation).
    pub fn discard(&self) { let _ = fs::remove_file(&self.full); }
}

/// Validate `source` and copy it into `media/` while hashing it. The copy lives under its final
/// name but nothing refers to it until `link` succeeds.
pub fn stage(directory: &Path, source: &Path, max_bytes: u64) -> Result<StagedRaw, String> {
    if !source.is_absolute() { return Err("Seleziona il file RAW tramite il dialogo di sistema".into()); }
    let extension = extension_of(source).ok_or("Formato non riconosciuto: sono accettati i file RAW delle fotocamere (CR2, CR3, NEF, ARW, DNG, ORF, RAF, RW2…)")?;
    let name = source.file_name().and_then(|n| n.to_str()).ok_or("Nome del file non valido")?.to_string();
    if name.chars().count() > 240 || name.chars().any(char::is_control) { return Err("Nome del file non valido".into()); }
    let metadata = fs::symlink_metadata(source).map_err(err)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() { return Err("Il file RAW non è un file regolare".into()); }
    if metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(format!("File RAW vuoto o troppo grande (massimo {} MiB)", max_bytes / (1024 * 1024)));
    }
    let mut input = File::open(source).map_err(err)?;
    let mut header = [0_u8; 16];
    let count = input.read(&mut header).map_err(err)?;
    if !looks_like_raw(&header[..count]) { return Err("Il contenuto non corrisponde a un file RAW fotografico".into()); }
    regular_media_dir(directory)?;
    let relative = format!("media/raw-{}.{extension}", Uuid::new_v4());
    let full = directory.join(&relative);
    let mut output = OpenOptions::new().write(true).create_new(true).open(&full).map_err(err)?;
    let copied = (|| -> std::io::Result<(u64, String)> {
        let mut hash = Sha256::new();
        hash.update(&header[..count]);
        output.write_all(&header[..count])?;
        let (mut total, mut buffer) = (count as u64, vec![0_u8; 1 << 20]);
        loop {
            let read = input.read(&mut buffer)?;
            if read == 0 { break; }
            total += read as u64;
            if total > max_bytes { return Err(std::io::Error::other("File RAW troppo grande")); }
            hash.update(&buffer[..read]);
            output.write_all(&buffer[..read])?;
        }
        output.sync_all()?;
        Ok((total, format!("{:x}", hash.finalize())))
    })();
    drop(output);
    match copied {
        Ok((size_bytes, sha256)) => Ok(StagedRaw { relative, original_name: name, size_bytes, sha256, full }),
        Err(e) => { let _ = fs::remove_file(&full); Err(err(e)) }
    }
}

/// Link a staged file to an artwork, replacing a previous RAW. Returns the path of the replaced
/// file, to be deleted after the surrounding transaction commits.
pub fn link(conn: &Connection, artwork_id: &str, staged: &StagedRaw) -> Result<Option<String>, String> {
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let previous: Option<String> = conn.query_row("SELECT media_path FROM artwork_raw_files WHERE artwork_id=?1", params![artwork_id], |r| r.get(0))
        .optional().map_err(err)?;
    let deleted = conn.execute("DELETE FROM artwork_raw_files WHERE artwork_id=?1", params![artwork_id]).map_err(err)?;
    if previous.is_some() && deleted != 1 {
        return Err("Il RAW precedente non è stato scollegato: sostituzione annullata".into());
    }
    let inserted = conn.execute("INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
        params![artwork_id, staged.relative, staged.original_name, staged.size_bytes as i64, staged.sha256, Utc::now().to_rfc3339()]).map_err(err)?;
    if inserted != 1 {
        return Err("Il nuovo RAW non è stato registrato: sostituzione annullata".into());
    }
    Ok(previous)
}

pub fn remove_file(directory: &Path, relative: &str) {
    if let Ok(path) = regular_raw_path(directory, relative) {
        let _ = fs::remove_file(path);
    }
}

/// Check immutable recorded metadata before detaching an imported or altered RAW.
/// Stream the verification to keep RAM bounded even for 1 GiB originals.
fn verify_raw_integrity(directory: &Path, raw: &RawFile) -> Result<(), String> {
    let path = regular_raw_path(directory, &raw.media_path)?;
    if raw.size_bytes <= 0 || raw.size_bytes as u64 > MAX_RAW_BYTES {
        return Err("Dimensione RAW registrata non valida".into());
    }
    let mut input = File::open(&path).map_err(err)?;
    if input.metadata().map_err(err)?.len() != raw.size_bytes as u64 {
        return Err("Dimensione RAW diversa dai metadati: allegato conservato".into());
    }
    let mut digest = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer).map_err(err)?;
        if count == 0 { break; }
        total += count as u64;
        if total > raw.size_bytes as u64 {
            return Err("Il RAW è cambiato durante la verifica: allegato conservato".into());
        }
        digest.update(&buffer[..count]);
    }
    if total != raw.size_bytes as u64 || !format!("{:x}", digest.finalize()).eq_ignore_ascii_case(&raw.sha256) {
        return Err("Hash RAW diverso dai metadati: allegato conservato".into());
    }
    Ok(())
}

/// Retain a detached RAW when a gallery, collector, certificate, catalogue
/// revision or another RAW record still needs the same path. Fail closed.
fn cleanup_unlinked_raw(conn: &Connection, directory: &Path, relative: &str) {
    if regular_raw_path(directory, relative).is_err() { return; }
    let in_use = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM artwork_raw_files WHERE media_path=?1)",
        params![relative], |r| r.get::<_, bool>(0),
    );
    if matches!(in_use, Ok(false)) {
        crate::media::remove_file_unless_catalogued(conn, directory, relative);
    }
}


fn row(conn: &Connection, artwork_id: &str) -> Result<Option<RawFile>, String> {
    conn.query_row("SELECT artwork_id,media_path,original_name,size_bytes,sha256,created_at FROM artwork_raw_files WHERE artwork_id=?1",
        params![artwork_id], |r| Ok(RawFile { artwork_id: r.get(0)?, media_path: r.get(1)?, original_name: r.get(2)?, size_bytes: r.get(3)?,
            sha256: r.get(4)?, created_at: r.get(5)? })).optional().map_err(err)
}

pub fn attach_core(db: &Database, artwork_id: &str, source: &Path, max_bytes: u64) -> Result<RawFile, String> {
    // The long copy happens before the archive lock is taken, so the application stays responsive.
    let staged = stage(&db.directory, source, max_bytes)?;
    let outcome = (|| -> Result<(RawFile, Option<RawFile>), String> {
        let mut conn = db.conn.lock().map_err(err)?;
        let tx = conn.transaction().map_err(err)?;
        let replaced = row(&tx, artwork_id)?;
        // Do not discard the reference to an externally modified original.
        if let Some(ref old) = replaced { verify_raw_integrity(&db.directory, old)?; }
        link(&tx, artwork_id, &staged)?;
        let saved = row(&tx, artwork_id)?.ok_or("Allegato non salvato")?;
        // Imported databases may contain AFTER INSERT triggers that silently alter
        // the new reference. Never commit or discard the old RAW in that case.
        if saved.artwork_id != artwork_id || saved.media_path != staged.relative
            || saved.original_name != staged.original_name || saved.size_bytes != staged.size_bytes as i64
            || !saved.sha256.eq_ignore_ascii_case(&staged.sha256)
        {
            return Err("I metadati del RAW non corrispondono al file copiato: salvataggio annullato".into());
        }
        tx.commit().map_err(err)?;
        Ok((saved, replaced))
    })();
    match outcome {
        Ok((saved, replaced)) => {
            if let Some(old) = replaced {
                // Recheck before unlinking; if bytes changed meanwhile, preserve
                // the file rather than delete evidence that no longer matches.
                if verify_raw_integrity(&db.directory, &old).is_ok() {
                    if let Ok(conn) = db.conn.lock() {
                        cleanup_unlinked_raw(&conn, &db.directory, &old.media_path);
                    }
                }
            }
            Ok(saved)
        }
        Err(e) => { staged.discard(); Err(e) }
    }
}

pub fn remove_core(db: &Database, artwork_id: &str) -> Result<(), String> {
    regular_media_dir(&db.directory)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let file = row(&tx, artwork_id)?.ok_or("Nessun file RAW allegato")?;
    // A damaged import must keep its database reference for recovery.
    verify_raw_integrity(&db.directory, &file)?;
    let deleted = tx.execute("DELETE FROM artwork_raw_files WHERE artwork_id=?1", params![artwork_id]).map_err(err)?;
    // RAISE(IGNORE) can report success with zero changes; an AFTER DELETE
    // trigger can even recreate the row. Both must roll back before file cleanup.
    if deleted != 1 || row(&tx, artwork_id)?.is_some() {
        return Err("Il collegamento RAW non è stato rimosso: allegato conservato".into());
    }
    tx.commit().map_err(err)?;
    cleanup_unlinked_raw(&conn, &db.directory, &file.media_path);
    Ok(())
}

/// Save a copy of the RAW file outside the archive. Never overwrites an existing file.
pub fn export_core(db: &Database, artwork_id: &str, destination: &Path) -> Result<String, String> {
    let file = {
        let conn = db.conn.lock().map_err(err)?;
        row(&conn, artwork_id)?.ok_or("Nessun file RAW allegato")?
    };
    let source = regular_raw_path(&db.directory, &file.media_path)?;
    if file.size_bytes <= 0 || file.size_bytes as u64 > MAX_RAW_BYTES {
        return Err("Dimensione RAW registrata non valida".into());
    }
    let metadata = fs::symlink_metadata(&source).map_err(err)?;
    if metadata.len() != file.size_bytes as u64 {
        return Err("Il file RAW è cambiato rispetto ai metadati registrati".into());
    }

    let live = fs::canonicalize(&db.directory).map_err(err)?;
    let parent = destination.parent().ok_or("Percorso di destinazione non valido")?;
    let parent = fs::canonicalize(parent).map_err(err)?;
    if parent.starts_with(&live) { return Err("Non esportare documenti all'interno dell'archivio attivo".into()); }
    let name = destination.file_name().ok_or("Nome file non valido")?;
    let target = parent.join(name);
    let mut input = File::open(&source).map_err(err)?;
    let mut output = OpenOptions::new().write(true).create_new(true).open(&target)
        .map_err(|e| format!("Impossibile creare il file (scegli un nuovo nome se esiste già): {}", err(e)))?;

    // Stream and verify copied bytes against immutable recorded metadata.
    // A damaged RAW must never be presented as a valid portable copy.
    let copied = (|| -> std::io::Result<(u64, String)> {
        let mut digest = Sha256::new();
        let mut total = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 { break; }
            total += count as u64;
            if total > file.size_bytes as u64 {
                return Err(std::io::Error::other("Il file RAW è cambiato durante l'esportazione"));
            }
            output.write_all(&buffer[..count])?;
            digest.update(&buffer[..count]);
        }
        output.sync_all()?;
        Ok((total, format!("{:x}", digest.finalize())))
    })();
    drop(output);
    match copied {
        Ok((size, hash)) if size == file.size_bytes as u64 && hash.eq_ignore_ascii_case(&file.sha256) =>
            Ok(crate::util::display_path(&target)),
        Ok(_) => {
            let _ = fs::remove_file(&target);
            Err("Il file RAW è cambiato rispetto all'hash registrato: esportazione annullata".into())
        }
        Err(e) => {
            let _ = fs::remove_file(&target);
            Err(err(e))
        }
    }
}

#[tauri::command]
pub fn list_artwork_raw_files(db: State<'_, Database>) -> Result<Vec<RawFile>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let mut statement = conn.prepare("SELECT artwork_id,media_path,original_name,size_bytes,sha256,created_at FROM artwork_raw_files").map_err(err)?;
    let files = statement.query_map([], |r| Ok(RawFile { artwork_id: r.get(0)?, media_path: r.get(1)?, original_name: r.get(2)?,
        size_bytes: r.get(3)?, sha256: r.get(4)?, created_at: r.get(5)? })).map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    Ok(files)
}

#[tauri::command]
pub async fn attach_artwork_raw(app: tauri::AppHandle, artwork_id: String, source_path: String) -> Result<RawFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        attach_core(&db, &artwork_id, Path::new(&source_path), MAX_RAW_BYTES)
    }).await.map_err(err)?
}

#[tauri::command]
pub async fn remove_artwork_raw(app: tauri::AppHandle, artwork_id: String) -> Result<(), String> {
    // Reading and hashing a RAW can take time; never block the Tauri UI thread.
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        remove_core(&db, &artwork_id)
    }).await.map_err(err)?
}

#[tauri::command]
pub async fn export_artwork_raw(app: tauri::AppHandle, artwork_id: String, destination: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        export_core(&db, &artwork_id, Path::new(&destination))
    }).await.map_err(err)?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        db.conn.lock().unwrap().execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES('a','A-1','Opera','t','t')", []).unwrap();
        (dir, db)
    }
    fn raw(dir: &Path, name: &str, header: &[u8], extra: usize) -> PathBuf {
        let path = dir.join(name);
        let mut bytes = header.to_vec();
        bytes.extend(std::iter::repeat_n(7_u8, extra));
        fs::write(&path, bytes).unwrap();
        path
    }
    fn media_count(db: &Database) -> usize { fs::read_dir(db.directory.join("media")).unwrap().count() }

    #[test]
    fn real_raw_formats_are_recognised_by_extension_and_signature() {
        let (dir, db) = setup();
        for (name, header) in [("a.NEF", &b"II*\0"[..]), ("b.cr3", &b"\0\0\0\x18ftypcrx "[..]), ("c.orf", &b"IIRO"[..]), ("d.raf", &b"FUJIFILMCCD-RAW 0201"[..]),
            ("e.rw2", &b"IIU\0"[..]), ("f.dng", &b"MM\0*"[..]), ("g.x3f", &b"FOVb"[..]), ("h.crw", &b"II\x1a\0\0\0HEAPCCDR"[..])] {
            let path = raw(dir.path(), name, header, 64);
            let staged = stage(&db.directory, &path, MAX_RAW_BYTES).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(staged.size_bytes, header.len() as u64 + 64);
            staged.discard();
        }
        assert_eq!(media_count(&db), 0);
    }

    #[test]
    fn other_files_are_refused_even_with_a_raw_extension() {
        let (dir, db) = setup();
        for (name, header) in [("x.exe", &b"II*\0"[..]), ("foto.jpg", &b"II*\0"[..]), ("fake.nef", &b"MZ\x90\0"[..]), ("fake.cr2", &b"%PDF-1.7"[..]), ("noext", &b"II*\0"[..])] {
            let path = raw(dir.path(), name, header, 16);
            assert!(stage(&db.directory, &path, MAX_RAW_BYTES).is_err(), "{name}");
        }
        assert!(stage(&db.directory, Path::new("relativo.nef"), MAX_RAW_BYTES).unwrap_err().contains("dialogo"));
        let empty = dir.path().join("vuoto.nef");
        fs::write(&empty, []).unwrap();
        assert!(stage(&db.directory, &empty, MAX_RAW_BYTES).is_err());
        let big = raw(dir.path(), "grande.nef", b"II*\0", 2_000);
        assert!(stage(&db.directory, &big, 1_000).unwrap_err().contains("troppo grande"));
        assert_eq!(media_count(&db), 0, "refused files leave nothing behind");
    }

    #[test]
    fn attaching_stores_hash_size_and_replaces_the_previous_file() {
        let (dir, db) = setup();
        let first = raw(dir.path(), "scatto-001.NEF", b"II*\0", 5_000);
        let saved = attach_core(&db, "a", &first, MAX_RAW_BYTES).unwrap();
        assert_eq!((saved.original_name.as_str(), saved.size_bytes), ("scatto-001.NEF", 5_004));
        assert_eq!(saved.sha256, format!("{:x}", Sha256::digest(fs::read(&first).unwrap())));
        assert!(db.directory.join(&saved.media_path).is_file());
        let second = raw(dir.path(), "scatto-002.dng", b"MM\0*", 100);
        let replaced = attach_core(&db, "a", &second, MAX_RAW_BYTES).unwrap();
        assert!(!db.directory.join(&saved.media_path).exists(), "the replaced RAW file is removed");
        assert!(db.directory.join(&replaced.media_path).is_file());
        assert_eq!(media_count(&db), 1);
        assert!(attach_core(&db, "inesistente", &second, MAX_RAW_BYTES).is_err());
        assert_eq!(media_count(&db), 1, "a failed attach cleans up its copy");
    }

    #[test]
    fn removing_and_exporting_never_endanger_the_archive() {
        let (dir, db) = setup();
        let source = raw(dir.path(), "scatto.arw", b"II*\0", 300);
        let saved = attach_core(&db, "a", &source, MAX_RAW_BYTES).unwrap();
        let copy = dir.path().join("copia.arw");
        let reported = export_core(&db, "a", &copy).unwrap();
        assert_eq!(fs::read(&copy).unwrap(), fs::read(&source).unwrap());
        assert!(!reported.starts_with(r"\\?\"));
        assert!(export_core(&db, "a", &copy).is_err(), "an existing file is never overwritten");
        assert!(export_core(&db, "a", &db.directory.join("media").join("dentro.arw")).is_err(), "no export inside the archive");
        assert!(export_core(&db, "inesistente", &dir.path().join("altro.arw")).is_err());
        remove_core(&db, "a").unwrap();
        assert!(!db.directory.join(&saved.media_path).exists());
        assert!(remove_core(&db, "a").is_err());
        let rows: i64 = db.conn.lock().unwrap().query_row("SELECT count(*) FROM artwork_raw_files", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 0);
    }

    #[test]
    fn altered_raw_cannot_be_removed_or_replaced_and_staged_replacement_is_cleaned() {
        let (dir, db) = setup();
        let first = raw(dir.path(), "original.nef", &[73, 73, 42, 0], 128);
        let saved = attach_core(&db, "a", &first, MAX_RAW_BYTES).unwrap();
        let stored = db.directory.join(&saved.media_path);
        fs::write(&stored, vec![42_u8; saved.size_bytes as usize]).unwrap();
        assert!(remove_core(&db, "a").unwrap_err().contains("Hash"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved.clone()));
        let replacement = raw(dir.path(), "replacement.dng", &[77, 77, 0, 42], 256);
        assert!(attach_core(&db, "a", &replacement, MAX_RAW_BYTES).unwrap_err().contains("Hash"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved));
        assert_eq!(media_count(&db), 1, "rejected replacement must not leak staged media");
        assert!(stored.is_file(), "altered bytes are evidence, not disposable media");
    }

    #[test]
    fn removing_raw_preserves_files_referenced_by_an_artwork_gallery() {
        let (dir, db) = setup();
        let source = raw(dir.path(), "original.nef", &[73, 73, 42, 0], 96);
        let saved = attach_core(&db, "a", &source, MAX_RAW_BYTES).unwrap();
        // An imported archive may reuse a RAW path in another media table.
        db.conn.lock().unwrap().execute(
            "INSERT INTO artwork_images(id,artwork_id,image_path,created_at,sort_order)
             VALUES('photo','a',?1,'t',1)",
            params![saved.media_path],
        ).unwrap();
        remove_core(&db, "a").unwrap();
        assert!(db.directory.join(&saved.media_path).is_file());
        assert!(row(&db.conn.lock().unwrap(), "a").unwrap().is_none());
    }

    #[test]
    fn removal_only_touches_raw_files_inside_the_media_folder() {
        let (dir, db) = setup();
        let outside = dir.path().join("importante.txt");
        fs::write(&outside, "x").unwrap();
        remove_file(&db.directory, "../importante.txt");
        remove_file(&db.directory, "media/../../importante.txt");
        remove_file(&db.directory, "importante.txt");
        assert!(outside.is_file());
    }

    #[test]
    fn changed_raw_cannot_be_exported_as_a_valid_copy() {
        let (dir, db) = setup();
        let source = raw(dir.path(), "foto.nef", b"II*\0", 64);
        let saved = attach_core(&db, "a", &source, MAX_RAW_BYTES).unwrap();
        let stored = db.directory.join(&saved.media_path);
        // Same byte count, different contents: size checks alone would miss this.
        fs::write(&stored, vec![42_u8; saved.size_bytes as usize]).unwrap();
        let target = dir.path().join("invalid.nef");
        assert!(export_core(&db, "a", &target).unwrap_err().contains("hash"));
        assert!(!target.exists(), "a failed export must not leave a misleading copy");
        assert!(row(&db.conn.lock().unwrap(), "a").unwrap().is_some(), "the database link survives");
        // Even a corrupt relative path from SQLite cannot escape the archive.
        db.conn.lock().unwrap().execute("UPDATE artwork_raw_files SET media_path='../outside.nef' WHERE artwork_id='a'", []).unwrap();
        assert!(export_core(&db, "a", &target).unwrap_err().contains("Percorso"));
        assert!(!target.exists());
    }

    #[cfg(unix)]
    #[test]
    fn replaced_media_folder_cannot_touch_external_raws() {
        use std::os::unix::fs::symlink;
        let (dir, db) = setup();
        let raw_source = raw(dir.path(), "foto.nef", b"II*\0", 64);
        let saved = attach_core(&db, "a", &raw_source, MAX_RAW_BYTES).unwrap();
        let external = dir.path().join("external");
        fs::create_dir(&external).unwrap();
        let external_raw = external.join(Path::new(&saved.media_path).file_name().unwrap());
        fs::write(&external_raw, b"private file").unwrap();
        fs::rename(db.directory.join("media"), db.directory.join("media-original")).unwrap();
        symlink(&external, db.directory.join("media")).unwrap();

        assert!(stage(&db.directory, &raw_source, MAX_RAW_BYTES).is_err());
        assert!(remove_core(&db, "a").is_err());
        remove_file(&db.directory, &saved.media_path);
        let target = dir.path().join("export.nef");
        assert!(export_core(&db, "a", &target).is_err());
        assert!(!target.exists());
        assert_eq!(fs::read(&external_raw).unwrap(), b"private file");
        assert!(row(&db.conn.lock().unwrap(), "a").unwrap().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_raw_file_is_not_exported() {
        use std::os::unix::fs::symlink;
        let (dir, db) = setup();
        let original = raw(dir.path(), "foto.nef", b"II*\0", 64);
        let saved = attach_core(&db, "a", &original, MAX_RAW_BYTES).unwrap();
        fs::remove_file(db.directory.join(&saved.media_path)).unwrap();
        symlink(&original, db.directory.join(&saved.media_path)).unwrap();

        let target = dir.path().join("copy.nef");
        assert!(export_core(&db, "a", &target).unwrap_err().contains("regolare"));
        assert!(!target.exists());
        remove_file(&db.directory, &saved.media_path);
        assert!(db.directory.join(&saved.media_path).is_symlink(), "removal must not touch tampered entries");
        assert!(original.exists());
    }

    #[test]
    fn ignored_or_recreated_deletions_keep_raw_and_database_link() {
        let (dir, db) = setup();
        let source = raw(dir.path(), "original.nef", b"II*\0", 96);
        let saved = attach_core(&db, "a", &source, MAX_RAW_BYTES).unwrap();
        let stored = db.directory.join(&saved.media_path);

        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER ignore_raw_delete BEFORE DELETE ON artwork_raw_files
             BEGIN SELECT RAISE(IGNORE); END;"
        ).unwrap();
        assert!(remove_core(&db, "a").unwrap_err().contains("non è stato rimosso"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved.clone()));
        assert!(stored.exists());
        db.conn.lock().unwrap().execute_batch(
            "DROP TRIGGER ignore_raw_delete;
             CREATE TRIGGER restore_raw_delete AFTER DELETE ON artwork_raw_files
             BEGIN
               INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at)
               VALUES(OLD.artwork_id,OLD.media_path,OLD.original_name,OLD.size_bytes,OLD.sha256,OLD.created_at);
             END;"
        ).unwrap();
        assert!(remove_core(&db, "a").unwrap_err().contains("non è stato rimosso"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved));
        assert!(stored.exists());
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER restore_raw_delete;").unwrap();
        remove_core(&db, "a").unwrap();
        assert!(!stored.exists());
        assert!(row(&db.conn.lock().unwrap(), "a").unwrap().is_none());
    }

    #[test]
    fn ignored_or_tampered_insert_does_not_replace_a_raw_or_leak_files() {
        let (dir, db) = setup();
        let source = raw(dir.path(), "first.nef", b"II*\0", 64);
        let saved = attach_core(&db, "a", &source, MAX_RAW_BYTES).unwrap();
        let replacement = raw(dir.path(), "second.dng", b"MM\0*", 128);

        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER ignore_raw_replace BEFORE DELETE ON artwork_raw_files
             BEGIN SELECT RAISE(IGNORE); END;"
        ).unwrap();
        assert!(attach_core(&db, "a", &replacement, MAX_RAW_BYTES).unwrap_err().contains("non è stato scollegato"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved.clone()));
        assert_eq!(media_count(&db), 1);
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER ignore_raw_replace;").unwrap();

        db.conn.lock().unwrap().execute_batch(
            "CREATE TRIGGER ignore_raw_insert BEFORE INSERT ON artwork_raw_files
             BEGIN SELECT RAISE(IGNORE); END;"
        ).unwrap();
        assert!(attach_core(&db, "a", &replacement, MAX_RAW_BYTES).unwrap_err().contains("non è stato registrato"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved.clone()));
        assert_eq!(media_count(&db), 1);

        db.conn.lock().unwrap().execute_batch(
            "DROP TRIGGER ignore_raw_insert;
             CREATE TRIGGER corrupt_raw_insert AFTER INSERT ON artwork_raw_files
             BEGIN UPDATE artwork_raw_files SET sha256='0000000000000000000000000000000000000000000000000000000000000000' WHERE artwork_id=NEW.artwork_id; END;"
        ).unwrap();
        assert!(attach_core(&db, "a", &replacement, MAX_RAW_BYTES).unwrap_err().contains("non corrispondono"));
        assert_eq!(row(&db.conn.lock().unwrap(), "a").unwrap(), Some(saved.clone()));
        assert_eq!(media_count(&db), 1);
        assert!(db.directory.join(&saved.media_path).exists());
        db.conn.lock().unwrap().execute_batch("DROP TRIGGER corrupt_raw_insert;").unwrap();
        let updated = attach_core(&db, "a", &replacement, MAX_RAW_BYTES).unwrap();
        assert_ne!(updated.media_path, saved.media_path);
        assert_eq!(media_count(&db), 1);
    }

}
