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
    conn.execute("DELETE FROM artwork_raw_files WHERE artwork_id=?1", params![artwork_id]).map_err(err)?;
    conn.execute("INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
        params![artwork_id, staged.relative, staged.original_name, staged.size_bytes as i64, staged.sha256, Utc::now().to_rfc3339()]).map_err(err)?;
    Ok(previous)
}

pub fn remove_file(directory: &Path, relative: &str) {
    let ok = relative.strip_prefix("media/")
        .is_some_and(|name| name.starts_with("raw-") && !name.contains(['/', '\\']) && !name.contains(".."));
    if ok { let _ = fs::remove_file(directory.join(relative)); }
}

fn row(conn: &Connection, artwork_id: &str) -> Result<Option<RawFile>, String> {
    conn.query_row("SELECT artwork_id,media_path,original_name,size_bytes,sha256,created_at FROM artwork_raw_files WHERE artwork_id=?1",
        params![artwork_id], |r| Ok(RawFile { artwork_id: r.get(0)?, media_path: r.get(1)?, original_name: r.get(2)?, size_bytes: r.get(3)?,
            sha256: r.get(4)?, created_at: r.get(5)? })).optional().map_err(err)
}

pub fn attach_core(db: &Database, artwork_id: &str, source: &Path, max_bytes: u64) -> Result<RawFile, String> {
    // The long copy happens before the archive lock is taken, so the application stays responsive.
    let staged = stage(&db.directory, source, max_bytes)?;
    let outcome = (|| -> Result<(RawFile, Option<String>), String> {
        let mut conn = db.conn.lock().map_err(err)?;
        let tx = conn.transaction().map_err(err)?;
        let replaced = link(&tx, artwork_id, &staged)?;
        let saved = row(&tx, artwork_id)?.ok_or("Allegato non salvato")?;
        tx.commit().map_err(err)?;
        Ok((saved, replaced))
    })();
    match outcome {
        Ok((saved, replaced)) => { if let Some(old) = replaced { remove_file(&db.directory, &old); } Ok(saved) }
        Err(e) => { staged.discard(); Err(e) }
    }
}

pub fn remove_core(db: &Database, artwork_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let file = row(&conn, artwork_id)?.ok_or("Nessun file RAW allegato")?;
    conn.execute("DELETE FROM artwork_raw_files WHERE artwork_id=?1", params![artwork_id]).map_err(err)?;
    remove_file(&db.directory, &file.media_path);
    Ok(())
}

/// Save a copy of the RAW file outside the archive. Never overwrites an existing file.
pub fn export_core(db: &Database, artwork_id: &str, destination: &Path) -> Result<String, String> {
    let source = {
        let conn = db.conn.lock().map_err(err)?;
        let file = row(&conn, artwork_id)?.ok_or("Nessun file RAW allegato")?;
        db.directory.join(file.media_path)
    };
    let live = fs::canonicalize(&db.directory).map_err(err)?;
    let parent = destination.parent().ok_or("Percorso di destinazione non valido")?;
    let parent = fs::canonicalize(parent).map_err(err)?;
    if parent.starts_with(&live) { return Err("Non esportare documenti all'interno dell'archivio attivo".into()); }
    let name = destination.file_name().ok_or("Nome file non valido")?;
    let target = parent.join(name);
    let mut input = File::open(&source).map_err(err)?;
    let mut output = OpenOptions::new().write(true).create_new(true).open(&target)
        .map_err(|e| format!("Impossibile creare il file (scegli un nuovo nome se esiste già): {}", err(e)))?;
    let copied = std::io::copy(&mut input, &mut output).and_then(|_| output.sync_all());
    drop(output);
    if let Err(e) = copied { let _ = fs::remove_file(&target); return Err(err(e)); }
    Ok(crate::util::display_path(&target))
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
pub fn remove_artwork_raw(db: State<'_, Database>, artwork_id: String) -> Result<(), String> {
    remove_core(&db, &artwork_id)
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
    fn removal_only_touches_raw_files_inside_the_media_folder() {
        let (dir, db) = setup();
        let outside = dir.path().join("importante.txt");
        fs::write(&outside, "x").unwrap();
        remove_file(&db.directory, "../importante.txt");
        remove_file(&db.directory, "media/../../importante.txt");
        remove_file(&db.directory, "importante.txt");
        assert!(outside.is_file());
    }
}
