//! Portable, local-only backups. A backup is a self-contained directory: snapshot SQLite,
//! copied media and a SHA-256 manifest. No network or third-party account is involved.
use crate::db::{self, Database};
use chrono::Utc;
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use tauri::Manager;
use uuid::Uuid;

const FORMAT: &str = "repertica-studio-backup";
const FORMAT_VERSION: u32 = 1;
const DB_FILE: &str = "repertica.sqlite";
const MANIFEST_FILE: &str = "manifest.json";
const ROLLBACK_FOLDER: &str = ".repertica-studio-restore-rollback";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FileRecord {
    path: String,
    bytes: u64,
    sha256: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    format: String,
    format_version: u32,
    created_at: String,
    schema_version: i64,
    files: Vec<FileRecord>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    directory: String,
    created_at: String,
    files: usize,
    total_bytes: u64,
    artworks: i64,
    collectors: i64,
    sales: i64,
    certificates: i64,
    schema_version: i64,
    verified: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSummary {
    pub restored: BackupSummary,
    pub previous_archive: String,
}

fn error(e: impl std::fmt::Display) -> String { e.to_string() }
fn unique_name(prefix: &str) -> String {
    format!("{}-{}-{}", prefix, Utc::now().format("%Y%m%d-%H%M%S"), Uuid::new_v4().simple())
}
fn app_parent(data_dir: &Path) -> Result<&Path, String> {
    data_dir.parent().ok_or_else(|| "Cartella dati non valida".into())
}
fn rollback_folder(data_dir: &Path) -> Result<PathBuf, String> {
    Ok(app_parent(data_dir)?.join(ROLLBACK_FOLDER))
}
fn check_regular(path: &Path) -> Result<(), String> {
    let typ = fs::symlink_metadata(path).map_err(error)?.file_type();
    if !typ.is_file() || typ.is_symlink() {
        return Err(format!("Il backup contiene un file non regolare: {}", path.display()));
    }
    Ok(())
}
fn safe_filename(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && name != "." && name != ".."
}
fn valid_record_path(path: &str) -> bool {
    if path == DB_FILE { return true; }
    let p = Path::new(path);
    let mut parts = p.components();
    matches!(parts.next(), Some(Component::Normal(root)) if root == "media")
        && matches!(parts.next(), Some(Component::Normal(name)) if safe_filename(&name.to_string_lossy()))
        && parts.next().is_none()
        && !path.contains('\\')
}
fn record(root: &Path, relative: &str) -> Result<FileRecord, String> {
    if !valid_record_path(relative) { return Err("Percorso non sicuro nel backup".into()); }
    let path = root.join(relative);
    check_regular(&path)?;
    let mut file = File::open(&path).map_err(error)?;
    let mut hash = Sha256::new();
    let mut bytes = 0_u64;
    let mut buf = [0_u8; 65_536];
    loop {
        let n = file.read(&mut buf).map_err(error)?;
        if n == 0 { break; }
        hash.update(&buf[..n]);
        bytes += n as u64;
    }
    Ok(FileRecord { path: relative.into(), bytes, sha256: format!("{:x}", hash.finalize()) })
}
fn copy_file(source: &Path, dest: &Path) -> Result<(), String> {
    check_regular(source)?;
    let mut input = File::open(source).map_err(error)?;
    let mut output = File::create_new(dest).map_err(error)?;
    std::io::copy(&mut input, &mut output).map_err(error)?;
    output.sync_all().map_err(error)
}
fn validate_snapshot(path: &Path) -> Result<(i64, [i64; 4], Vec<String>), String> {
    check_regular(path)?;
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(error)?;
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(error)?;
    if integrity != "ok" { return Err(format!("Verifica SQLite fallita: {integrity}")); }
    let foreign_key_problem: i64 = conn.query_row(
        "SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0)
    ).map_err(error)?;
    if foreign_key_problem != 0 { return Err("Il database contiene relazioni non valide".into()); }
    let schema: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0)).map_err(error)?;
    if !(1..=4).contains(&schema) { return Err(format!("Versione schema SQLite non supportata: {schema}")); }
    let mut counts = [0_i64; 4];
    for (i, table) in ["artworks", "collectors", "sales", "certificates"].iter().enumerate() {
        counts[i] = conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).map_err(error)?;
    }
    let mut statement = conn.prepare("SELECT image_path FROM artworks WHERE image_path IS NOT NULL").map_err(error)?;
    let mut references = statement.query_map([], |r| r.get::<_, String>(0)).map_err(error)?
        .collect::<Result<Vec<_>, _>>().map_err(error)?;
    if schema >= 3 {
        let mut gallery = conn.prepare("SELECT image_path FROM artwork_images").map_err(error)?;
        references.extend(gallery.query_map([], |r| r.get::<_, String>(0)).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?);
    }
    Ok((schema, counts, references))
}
fn manifest_from_dir(dir: &Path) -> Result<Manifest, String> {
    let manifest_path = dir.join(MANIFEST_FILE);
    check_regular(&manifest_path)?;
    if fs::metadata(&manifest_path).map_err(error)?.len() > 10 * 1024 * 1024 {
        return Err("Manifest del backup troppo grande".into());
    }
    let value: Manifest = serde_json::from_slice(&fs::read(&manifest_path).map_err(error)?).map_err(error)?;
    if value.format != FORMAT || value.format_version != FORMAT_VERSION || !(1..=4).contains(&value.schema_version) {
        return Err("Questo backup non è compatibile con Repertica Studio".into());
    }
    if value.files.len() > 1_000_000 { return Err("Troppi file nel backup".into()); }
    Ok(value)
}
pub fn inspect_backup_core(folder: &Path) -> Result<BackupSummary, String> {
    let directory = fs::canonicalize(folder).map_err(error)?;
    if !directory.is_dir() { return Err("Seleziona la cartella di un backup Repertica".into()); }
    let manifest = manifest_from_dir(&directory)?;
    let mut seen = HashSet::new();
    let mut total = 0_u64;
    for entry in &manifest.files {
        if !valid_record_path(&entry.path) || !seen.insert(entry.path.clone()) {
            return Err("Il manifest contiene un percorso non valido o duplicato".into());
        }
        let actual = record(&directory, &entry.path)?;
        if actual.bytes != entry.bytes || actual.sha256 != entry.sha256 {
            return Err(format!("File del backup alterato o danneggiato: {}", entry.path));
        }
        total = total.checked_add(entry.bytes).ok_or("Dimensione del backup non valida")?;
    }
    if !seen.contains(DB_FILE) { return Err("Database assente nel backup".into()); }
    let (schema, counts, references) = validate_snapshot(&directory.join(DB_FILE))?;
    if schema != manifest.schema_version { return Err("Inconsistent backup schema version".into()); }
    for ref_path in references {
        if !seen.contains(&ref_path) {
            return Err(format!("Immagine dell'opera non inclusa nel backup: {ref_path}"));
        }
    }
    Ok(BackupSummary {
        directory: directory.to_string_lossy().into_owned(),
        created_at: manifest.created_at,
        files: manifest.files.len(),
        total_bytes: total,
        artworks: counts[0],
        collectors: counts[1],
        sales: counts[2],
        certificates: counts[3],
        schema_version: schema,
        verified: true,
    })
}
pub fn create_backup_core(db: &Database, destination: &Path) -> Result<BackupSummary, String> {
    let root = fs::canonicalize(destination).map_err(error)?;
    if !root.is_dir() { return Err("Seleziona una cartella di destinazione esistente".into()); }
    let live = fs::canonicalize(&db.directory).map_err(error)?;
    if root.starts_with(&live) {
        return Err("Non puoi salvare il backup all'interno della cartella dati attiva".into());
    }

    let name = unique_name("ReperticaStudio-backup");
    let partial = root.join(format!(".{name}.partial"));
    let target = root.join(&name);
    fs::create_dir(&partial).map_err(error)?;
    let result = (|| -> Result<BackupSummary, String> {
        fs::create_dir(partial.join("media")).map_err(error)?;
        // One application mutex covers the SQLite snapshot and all attached media,
        // so an image cannot be changed halfway through the backup.
        let conn = db.conn.lock().map_err(error)?;
        let snapshot = partial.join(DB_FILE);
        let output = snapshot.to_str().ok_or("Il percorso non può essere convertito in UTF-8")?;
        conn.execute("VACUUM INTO ?1", params![output]).map_err(error)?;
        // VACUUM INTO creates a consistent DB, and DELETE journal mode makes the
        // backed up file independently readable without -wal/-shm sidecars.
        let copy = Connection::open(&snapshot).map_err(error)?;
        copy.pragma_update(None, "journal_mode", "DELETE").map_err(error)?;
        drop(copy);

        let schema_version = validate_snapshot(&snapshot)?.0;
        let mut files = vec![record(&partial, DB_FILE)?];
        let media_dir = db.directory.join("media");
        for entry in fs::read_dir(&media_dir).map_err(error)? {
            let entry = entry.map_err(error)?;
            if !entry.file_type().map_err(error)?.is_file() {
                return Err("La cartella media contiene un elemento non regolare".into());
            }
            let name = entry.file_name();
            let name = name.to_str().ok_or("Nome immagine non valido")?;
            if !safe_filename(name) { return Err("Nome immagine non valido".into()); }
            let relative = format!("media/{name}");
            copy_file(&entry.path(), &partial.join(&relative))?;
            files.push(record(&partial, &relative)?);
        }
        drop(conn);
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let manifest = Manifest {
            format: FORMAT.into(), format_version: FORMAT_VERSION,
            created_at: Utc::now().to_rfc3339(), schema_version, files,
        };
        let mut output = File::create_new(partial.join(MANIFEST_FILE)).map_err(error)?;
        output.write_all(&serde_json::to_vec_pretty(&manifest).map_err(error)?).map_err(error)?;
        output.sync_all().map_err(error)?;
        // Validation runs on the finished temporary tree before publication.
        inspect_backup_core(&partial)?;
        fs::rename(&partial, &target).map_err(error)?;
        inspect_backup_core(&target)
    })();
    if result.is_err() && partial.exists() { let _ = fs::remove_dir_all(&partial); }
    result
}
pub fn recover_interrupted_restore(data_dir: &Path) -> Result<(), String> {
    let rollback = rollback_folder(data_dir)?;
    if !rollback.exists() { return Ok(()); }
    if !data_dir.exists() {
        fs::rename(&rollback, data_dir).map_err(error)?;
    } else {
        let parent = app_parent(data_dir)?;
        let previous = parent.join(unique_name("ReperticaStudio-before-restore"));
        fs::rename(&rollback, &previous).map_err(error)?;
    }
    Ok(())
}
pub fn restore_backup_core(db: &Database, folder: &Path) -> Result<RestoreSummary, String> {
    let selected = fs::canonicalize(folder).map_err(error)?;
    let live = fs::canonicalize(&db.directory).map_err(error)?;
    if selected.starts_with(&live) { return Err("Non puoi ripristinare direttamente dall'archivio attivo".into()); }
    let summary = inspect_backup_core(&selected)?;
    let parent = app_parent(&db.directory)?;
    let rollback = rollback_folder(&db.directory)?;
    if rollback.exists() {
        return Err("Ripristino precedente incompleto: riavvia Repertica Studio prima di procedere".into());
    }
    let staging = parent.join(unique_name(".repertica-restore-staging"));
    fs::create_dir(&staging).map_err(error)?;
    let preparation = (|| -> Result<(), String> {
        fs::create_dir(staging.join("media")).map_err(error)?;
        let manifest = manifest_from_dir(&selected)?;
        for entry in manifest.files {
            copy_file(&selected.join(&entry.path), &staging.join(&entry.path))?;
        }
        let mut manifest_target = File::create_new(staging.join(MANIFEST_FILE)).map_err(error)?;
        manifest_target.write_all(&fs::read(selected.join(MANIFEST_FILE)).map_err(error)?).map_err(error)?;
        manifest_target.sync_all().map_err(error)?;
        inspect_backup_core(&staging)?;
        // The manifest lives only in backups; the live folder only needs DB+media.
        fs::remove_file(staging.join(MANIFEST_FILE)).map_err(error)?;
        Ok(())
    })();
    if let Err(e) = preparation {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }

    let mut handle = db.conn.lock().map_err(error)?;
    // No writes can occur while we replace the archive directory.
    let placeholder = Connection::open_in_memory().map_err(error)?;
    let previous_connection = std::mem::replace(&mut *handle, placeholder);
    drop(previous_connection);
    if let Err(e) = fs::rename(&db.directory, &rollback) {
        let reopened = db::open(db.directory.clone()).map_err(error)?;
        *handle = reopened.conn.into_inner().map_err(error)?;
        let _ = fs::remove_dir_all(&staging);
        return Err(format!("Impossibile mettere al sicuro l'archivio attuale: {e}"));
    }
    if let Err(e) = fs::rename(&staging, &db.directory) {
        let recovered = fs::rename(&rollback, &db.directory);
        if recovered.is_ok() {
            let reopened = db::open(db.directory.clone()).map_err(error)?;
            *handle = reopened.conn.into_inner().map_err(error)?;
        }
        return Err(format!("Ripristino annullato: {e}. Ripristino archivio precedente: {}", if recovered.is_ok() {"riuscito"} else {"non riuscito: conserva la cartella di rollback"}));
    }
    let reopened = db::open(db.directory.clone());
    match reopened {
        Ok(opened) => {
            *handle = opened.conn.into_inner().map_err(error)?;
            let previous = parent.join(unique_name("ReperticaStudio-before-restore"));
            // Retain the entire previous archive (including WAL/media) rather than
            // deleting anything. If renaming fails the fixed rollback directory remains.
            let saved = if fs::rename(&rollback, &previous).is_ok() { previous } else { rollback };
            Ok(RestoreSummary { restored: summary, previous_archive: saved.to_string_lossy().into_owned() })
        }
        Err(e) => {
            let rejected = parent.join(unique_name(".repertica-rejected-restore"));
            let removed = fs::rename(&db.directory, &rejected);
            if removed.is_ok() && fs::rename(&rollback, &db.directory).is_ok() {
                let original = db::open(db.directory.clone()).map_err(error)?;
                *handle = original.conn.into_inner().map_err(error)?;
                Err(format!("Ripristino annullato: il nuovo archivio non si apre ({e})"))
            } else {
                Err(format!("Ripristino non riuscito: {e}. Conservare le cartelle di rollback e contattare il supporto."))
            }
        }
    }
}

#[tauri::command]
pub async fn create_backup(app: tauri::AppHandle, destination: String) -> Result<BackupSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        create_backup_core(&db, Path::new(&destination))
    }).await.map_err(error)?
}
#[tauri::command]
pub async fn inspect_backup(folder: String) -> Result<BackupSummary, String> {
    tauri::async_runtime::spawn_blocking(move || inspect_backup_core(Path::new(&folder)))
        .await.map_err(error)?
}
#[tauri::command]
pub async fn restore_backup(app: tauri::AppHandle, folder: String) -> Result<RestoreSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        restore_backup_core(&db, Path::new(&folder))
    }).await.map_err(error)?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seed(db: &Database) {
        let conn = db.conn.lock().unwrap();
        conn.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at,image_path) VALUES('a','R-1','Opera','available','now','now','media/example.png')", []).unwrap();
        conn.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Cliente','now','now')", []).unwrap();
        fs::write(db.directory.join("media/example.png"), [1_u8,2,3,4]).unwrap();
    }
    #[test]
    fn creates_in_arbitrary_external_folder_and_restores_with_safety_copy() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let destination = tmp.path().join("un altro archivio con spazi");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        assert_eq!(backup.artworks, 1);
        assert_eq!(backup.files, 2);
        assert!(backup.verified);
        assert!(Path::new(&backup.directory).starts_with(&destination));
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('d','Dopo il backup','now','now')",[]).unwrap();
        }
        let restored = restore_backup_core(&db, Path::new(&backup.directory)).unwrap();
        assert!(Path::new(&restored.previous_archive).is_dir());
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM collectors",[],|r|r.get(0)).unwrap();
        assert_eq!(count, 1);
        assert_eq!(fs::read(db.directory.join("media/example.png")).unwrap(), vec![1,2,3,4]);
    }
    #[test]
    fn gallery_media_is_backed_up_and_verified_by_reference() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
                VALUES('photo2','a','media/secondary.png','Back side','now')",[]).unwrap();
        }
        fs::write(db.directory.join("media/secondary.png"), b"example secondary image").unwrap();
        let destination = tmp.path().join("disk");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        assert_eq!(backup.schema_version, 4);
        assert_eq!(backup.files, 3);
        assert!(backup.verified);

        // A manifest containing the DB but omitting a secondary photo cannot pass verification.
        let folder = Path::new(&backup.directory);
        let manifest_path = folder.join(MANIFEST_FILE);
        let mut manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.files.retain(|record| record.path != "media/secondary.png");
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
        assert!(inspect_backup_core(folder).is_err());
        assert!(restore_backup_core(&db, folder).is_err());
    }
    #[test]
    fn receipts_are_included_in_verified_backup_and_survive_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',10000,'2026-11-21','t')",[]).unwrap();
            conn.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                VALUES('r','s','2026-11-21',3500,'bank_transfer','t')",[]).unwrap();
        }
        let destination=tmp.path().join("removable");
        fs::create_dir(&destination).unwrap();
        let backup=create_backup_core(&db,&destination).unwrap();
        assert_eq!(backup.schema_version,4);
        assert!(backup.verified);
        {
            let conn=db.conn.lock().unwrap();
            conn.execute("DELETE FROM sale_receipts WHERE id='r'",[]).unwrap();
        }
        restore_backup_core(&db,Path::new(&backup.directory)).unwrap();
        let conn=db.conn.lock().unwrap();
        let received:i64=conn.query_row("SELECT amount_cents FROM sale_receipts WHERE id='r'",[],|r|r.get(0)).unwrap();
        assert_eq!(received,3500);
        let foreign_keys:i64=conn.query_row("SELECT count(*) FROM pragma_foreign_key_check",[],|r|r.get(0)).unwrap();
        assert_eq!(foreign_keys,0);
    }
    #[test]
    fn corrupted_media_is_rejected_before_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let out = tmp.path().join("drive");
        fs::create_dir(&out).unwrap();
        let backup = create_backup_core(&db, &out).unwrap();
        fs::write(Path::new(&backup.directory).join("media/example.png"),b"corrupt").unwrap();
        assert!(inspect_backup_core(Path::new(&backup.directory)).is_err());
        assert!(restore_backup_core(&db, Path::new(&backup.directory)).is_err());
        assert!(db.directory.join(DB_FILE).is_file());
    }
    #[test]
    fn incomplete_or_outside_media_paths_are_rejected() {
        assert!(!valid_record_path("../secret"));
        assert!(!valid_record_path("media/../secret"));
        assert!(!valid_record_path("media/subfolder/file.png"));
        assert!(!valid_record_path("media\\file.png"));
        assert!(valid_record_path("media/image.png"));
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        assert!(create_backup_core(&db, &db.directory).is_err());
    }
    #[test]
    fn interrupted_restore_recovers_old_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().join("active");
        let db = db::open(data_dir.clone()).unwrap();
        drop(db);
        let rollback = rollback_folder(&data_dir).unwrap();
        fs::rename(&data_dir,&rollback).unwrap();
        recover_interrupted_restore(&data_dir).unwrap();
        assert!(data_dir.join(DB_FILE).is_file());
        assert!(!rollback.exists());
    }
}
