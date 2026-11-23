//! Portable, local-only backups. A backup is a self-contained directory: snapshot SQLite,
//! copied media and a SHA-256 manifest. No network or third-party account is involved.
use crate::db::{self, Database};
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
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
    pub(crate) directory: String,
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

fn error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
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
        return Err(format!("Il backup contiene un file non regolare: {}", crate::util::display_path(path)));
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
// Verify a cross-row financial invariant independently of the SQLite write triggers.
// An exported database can have valid pages, hashes and foreign keys even if someone
// disabled its triggers before adding excessive payment records. Never restore such
// an archive into the live collection.
fn validate_receipt_balances(conn: &Connection) -> Result<(), String> {
    let invalid: Option<(String, i64, i64)> = conn.query_row(
        "SELECT s.id, s.amount_cents, COALESCE(SUM(r.amount_cents), 0)
         FROM sales s LEFT JOIN sale_receipts r ON r.sale_id = s.id
         GROUP BY s.id
         HAVING COALESCE(SUM(r.amount_cents), 0) > s.amount_cents
         LIMIT 1",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional().map_err(error)?;
    if let Some((id, sale, received)) = invalid {
        return Err(format!(
            "Il backup contiene incassi superiori al valore della vendita {id} ({received} > {sale} centesimi)"
        ));
    }
    Ok(())
}

// A restored financial archive must not silently move collections between
// months because an external importer stored an impossible or non-canonical
// date. SQLite's integrity_check does not validate date values in TEXT columns.
fn validate_receipt_dates(conn: &Connection) -> Result<(), String> {
    let mut statement = conn.prepare("SELECT id, received_at FROM sale_receipts").map_err(error)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(error)?;
    for row in rows {
        let (id, date) = row.map_err(error)?;
        let canonical = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map(|parsed| parsed.format("%Y-%m-%d").to_string() == date)
            .unwrap_or(false);
        if !canonical {
            return Err(format!("Il backup contiene una data di incasso non valida: {id}"));
        }
    }
    Ok(())
}

// Sale dates determine monthly turnover. A re-hashed SQLite snapshot can pass
// page and foreign-key checks while placing a sale in an impossible month.
// Check all schema versions, since the sales table exists since v1.
fn validate_sale_dates(conn: &Connection) -> Result<(), String> {
    let mut statement = conn.prepare("SELECT id, sold_at FROM sales").map_err(error)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(error)?;
    for row in rows {
        let (id, date) = row.map_err(error)?;
        let canonical = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map(|parsed| parsed.format("%Y-%m-%d").to_string() == date)
            .unwrap_or(false);
        if !canonical {
            return Err(format!("Il backup contiene una data di vendita non valida: {id}"));
        }
    }
    Ok(())
}

// Expense dates determine which month owns a studio cost. SQLite validates
// the table structure but not the YYYY-MM-DD content of its TEXT dates.
fn validate_expense_dates(conn: &Connection) -> Result<(), String> {
    let mut statement = conn.prepare("SELECT id, expense_date FROM expenses").map_err(error)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(error)?;
    for row in rows {
        let (id, date) = row.map_err(error)?;
        let canonical = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map(|parsed| parsed.format("%Y-%m-%d").to_string() == date)
            .unwrap_or(false);
        if !canonical {
            return Err(format!("Il backup contiene una data di spesa non valida: {id}"));
        }
    }
    Ok(())
}

// SQLite page integrity and foreign keys alone cannot detect an imported sale
// without its financial row, or taxes/shipping exceeding the agreed price.
// All schema versions >= 2 populate sale_finances for every historical sale.
fn validate_sale_finances(conn: &Connection) -> Result<(), String> {
    let invalid_sale: Option<String> = conn.query_row(
        "SELECT s.id FROM sales s
         LEFT JOIN sale_finances f ON f.sale_id=s.id
         WHERE f.sale_id IS NULL OR s.amount_cents <= 0
            OR f.tax_cents NOT BETWEEN 0 AND s.amount_cents
            OR f.shipping_cents NOT BETWEEN 0 AND (s.amount_cents - f.tax_cents)
            OR f.personal_bp NOT BETWEEN 0 AND 10000
            OR f.activity_bp NOT BETWEEN 0 AND 10000
            OR f.savings_bp NOT BETWEEN 0 AND 10000
            OR f.personal_bp != 10000 - f.activity_bp - f.savings_bp
         LIMIT 1",
        [], |row| row.get(0)
    ).optional().map_err(error)?;
    if let Some(sale_id) = invalid_sale {
        return Err(format!(
            "Il backup contiene dettagli economici mancanti o incoerenti per la vendita {sale_id}"
        ));
    }
    Ok(())
}

// Imported plans can have well-formed SQLite pages, valid foreign keys and
// the correct total while containing impossible due dates or non-contiguous
// positions. The UI allocates payments by position, so never publish or
// restore such a schedule. Installments appeared in schema v11.
fn validate_installment_schedule(conn: &Connection) -> Result<(), String> {
    let mut statement = conn.prepare(
        "SELECT id,sale_id,position,due_date,amount_cents FROM sale_installments
         ORDER BY sale_id,position,id"
    ).map_err(error)?;
    let rows = statement.query_map([], |row| Ok((
        row.get::<_, String>(0)?, row.get::<_, String>(1)?,
        row.get::<_, i64>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?
    ))).map_err(error)?;
    let mut current_sale = String::new();
    let mut previous_due = String::new();
    let mut next_position = 1_i64;
    let mut has_plan = false;
    for row in rows {
        let (id, sale_id, position, due_date, amount_cents) = row.map_err(error)?;
        if !has_plan || sale_id != current_sale {
            if has_plan && next_position < 3 {
                return Err(format!("Il backup contiene un piano con meno di due rate: {current_sale}"));
            }
            current_sale = sale_id;
            previous_due.clear();
            next_position = 1;
            has_plan = true;
        }
        if !(1..=60).contains(&position) || position != next_position {
            return Err(format!("Il backup contiene posizioni delle rate non consecutive: {current_sale}"));
        }
        if !(1..=100_000_000_000).contains(&amount_cents) {
            return Err(format!("Il backup contiene un importo di rata non valido: {id}"));
        }
        let valid_date = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
            .map(|date| date.format("%Y-%m-%d").to_string() == due_date)
            .unwrap_or(false);
        if !valid_date {
            return Err(format!("Il backup contiene una data di scadenza non valida: {id}"));
        }
        if !previous_due.is_empty() && due_date < previous_due {
            return Err(format!("Il backup contiene rate fuori ordine di scadenza: {current_sale}"));
        }
        previous_due = due_date;
        next_position += 1;
    }
    if has_plan && next_position < 3 {
        return Err(format!("Il backup contiene un piano con meno di due rate: {current_sale}"));
    }
    Ok(())
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
    if !(1..=crate::db::SCHEMA_VERSION).contains(&schema) { return Err(format!("Versione schema SQLite non supportata: {schema}")); }
    // Sale dates are present even in the original v1 schema.
    validate_sale_dates(&conn)?;
    // v1 predates finance rows; migration v2 created them for all existing sales.
    if schema >= 2 { validate_sale_finances(&conn)?; validate_expense_dates(&conn)?; }
    // Legacy schemas 1-3 have no sale_receipts table: never infer payments.
    if schema >= 4 { validate_receipt_balances(&conn)?; validate_receipt_dates(&conn)?; }
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
    if schema >= 13 {
        let mut revisions = conn.prepare("SELECT image_path FROM media_revisions").map_err(error)?;
        references.extend(revisions.query_map([], |r| r.get::<_, String>(0)).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?);
    }
    if schema >= 12 {
        let mut raws = conn.prepare("SELECT media_path FROM artwork_raw_files").map_err(error)?;
        references.extend(raws.query_map([], |r| r.get::<_, String>(0)).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?);
    }
    if schema >= 11 {
        validate_installment_schedule(&conn)?;
        let inconsistent: i64 = conn.query_row(
            "SELECT count(*) FROM (SELECT i.sale_id FROM sale_installments i JOIN sales s ON s.id=i.sale_id
              GROUP BY i.sale_id HAVING SUM(i.amount_cents)<>MAX(s.amount_cents))",
            [], |r| r.get(0)).map_err(error)?;
        if inconsistent != 0 { return Err("Il backup contiene piani rateali con un totale diverso dal prezzo di vendita".into()); }
    }
    // A sale carries either no frozen shares (split switched off) or shares totalling exactly 100%.
    if schema >= 9 {
        let inconsistent: i64 = conn.query_row(
            "SELECT count(*) FROM (SELECT sale_id FROM sale_allocations GROUP BY sale_id HAVING SUM(share_bp)<>10000)",
            [], |r| r.get(0)).map_err(error)?;
        if inconsistent != 0 { return Err("Il backup contiene vendite con una ripartizione diversa dal 100%".into()); }
    }
    if schema >= 8 {
        let mut photos = conn.prepare("SELECT image_path FROM collector_images").map_err(error)?;
        references.extend(photos.query_map([], |r| r.get::<_, String>(0)).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?);
    }
    // Receipts can reference local PDF/image evidence from schema v6 onward.
    if schema >= 6 {
        let mut attachments = conn.prepare("SELECT media_path FROM sale_receipt_attachments").map_err(error)?;
        references.extend(attachments.query_map([], |r| r.get::<_, String>(0)).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?);
    }
    // Certificates freeze the cover at issue time. After replacing the live
    // cover, its historical file can be referenced only by this snapshot.
    // Preserve backward compatibility with malformed legacy snapshots: their
    // media references cannot be determined, so do not invent any.
    let mut certificates = conn.prepare("SELECT snapshot_json FROM certificates").map_err(error)?;
    let snapshots = certificates.query_map([], |r| r.get::<_, String>(0)).map_err(error)?;
    for row in snapshots {
        let json = row.map_err(error)?;
        if let Ok(snapshot) = serde_json::from_str::<serde_json::Value>(&json) {
            if let Some(image_path) = snapshot.get("image_path").and_then(|value| value.as_str()) {
                references.push(image_path.to_string());
            }
        }
    }
    Ok((schema, counts, references))
}
// A valid manifest proves that backup bytes match the manifest, not that an
// attachment still matches the checksum saved when the receipt was attached.
// Reject a self-consistent but corrupted/re-hashed backup before restoring it.
fn validate_receipt_attachment_evidence(
    snapshot: &Path, root: &Path, files: &[FileRecord],
) -> Result<(), String> {
    let conn = Connection::open_with_flags(snapshot, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(error)?;
    let mut statement = conn.prepare(
        "SELECT receipt_id,media_path,mime_type,size_bytes,sha256 FROM sale_receipt_attachments"
    ).map_err(error)?;
    let by_path: HashMap<&str, &FileRecord> =
        files.iter().map(|record| (record.path.as_str(), record)).collect();
    let attachments = statement.query_map([], |r| Ok((
        r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?,
        r.get::<_, i64>(3)?, r.get::<_, String>(4)?,
    ))).map_err(error)?;
    for attachment in attachments {
        let (receipt_id, relative, stored_mime, stored_bytes, stored_sha) = attachment.map_err(error)?;
        if !crate::receipt_attachments::safe_media_path(&relative) {
            return Err(format!("Percorso allegato incasso non sicuro: {receipt_id}"));
        }
        let entry = by_path.get(relative.as_str())
            .ok_or_else(|| format!("Allegato incasso assente nel backup: {relative}"))?;
        if stored_bytes <= 0 || entry.bytes != stored_bytes as u64 || entry.sha256 != stored_sha {
            return Err(format!("Allegato incasso non coerente con i metadati SQLite: {relative}"));
        }
        // A matching checksum should not bless an invalid document header or
        // mismatched extension/MIME after offline modification of an archive.
        let mut header = Vec::with_capacity(12);
        File::open(root.join(&relative)).map_err(error)?
            .take(12).read_to_end(&mut header).map_err(error)?;
        let format_valid = crate::receipt_attachments::mime(&header)
            .is_some_and(|(ext, found_mime)| {
                stored_mime == found_mime && relative.ends_with(&format!(".{ext}"))
            });
        if !format_valid {
            return Err(format!("Formato allegato incasso non valido: {relative}"));
        }
    }
    Ok(())
}
fn manifest_from_dir(dir: &Path) -> Result<Manifest, String> {
    let manifest_path = dir.join(MANIFEST_FILE);
    check_regular(&manifest_path)?;
    if fs::metadata(&manifest_path).map_err(error)?.len() > 10 * 1024 * 1024 {
        return Err("Manifest del backup troppo grande".into());
    }
    let value: Manifest = serde_json::from_slice(&fs::read(&manifest_path).map_err(error)?).map_err(error)?;
    if value.format != FORMAT || value.format_version != FORMAT_VERSION || !(1..=crate::db::SCHEMA_VERSION).contains(&value.schema_version) {
        return Err("Questo backup non è compatibile con Repertica Studio".into());
    }
    if value.files.len() > 1_000_000 { return Err("Troppi file nel backup".into()); }
    Ok(value)
}
// A manifest must describe every media file, including unreferenced historical
// assets. Otherwise a restore would silently discard files present in the
// selected archive. Reject directory links before hashing any media entry:
// checking only the final file would follow a symlinked parent directory.
fn check_media_directory(root: &Path) -> Result<(), String> {
    let media = root.join("media");
    let kind = fs::symlink_metadata(&media).map_err(error)?.file_type();
    if !kind.is_dir() || kind.is_symlink() {
        return Err("La cartella media del backup non è una directory regolare".into());
    }
    Ok(())
}
// SQLite may read a sidecar alongside the database. None of these files is
// part of our manifest or restored by staging; accepting one could make an
// inspection describe different rows from the database actually restored.
// Even a broken symlink must count as an untracked sidecar.
fn reject_sqlite_sidecars(root: &Path) -> Result<(), String> {
    for suffix in ["-wal", "-shm", "-journal"] {
        let path = root.join(format!("{DB_FILE}{suffix}"));
        match fs::symlink_metadata(&path) {
            Ok(_) => return Err(format!(
                "Il backup contiene un file SQLite ausiliario non verificato: {}",
                crate::util::display_path(&path)
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
            Err(e) => return Err(error(e)),
        }
    }
    Ok(())
}
fn check_unlisted_media(root: &Path, recorded: &HashSet<String>) -> Result<(), String> {
    for entry in fs::read_dir(root.join("media")).map_err(error)? {
        let entry = entry.map_err(error)?;
        let file_name = entry.file_name();
        let name = file_name.to_str().ok_or("Nome file media non valido nel backup")?;
        let kind = entry.file_type().map_err(error)?;
        if !safe_filename(name) || !kind.is_file() || kind.is_symlink() {
            return Err(format!("Elemento media non regolare nel backup: {}", crate::util::display_path(&entry.path())));
        }
        let relative = format!("media/{name}");
        if !recorded.contains(&relative) {
            return Err(format!("File media non presente nel manifest del backup: {relative}"));
        }
    }
    Ok(())
}
pub fn inspect_backup_core(folder: &Path) -> Result<BackupSummary, String> {
    let directory = fs::canonicalize(folder).map_err(error)?;
    if !directory.is_dir() { return Err("Seleziona la cartella di un backup Repertica".into()); }
    let manifest = manifest_from_dir(&directory)?;
    // A valid hash of repertica.sqlite is not sufficient if SQLite can also
    // see unmanifested WAL/journal pages from the selected backup directory.
    reject_sqlite_sidecars(&directory)?;
    check_media_directory(&directory)?;
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
    check_unlisted_media(&directory, &seen)?;
    let (schema, counts, references) = validate_snapshot(&directory.join(DB_FILE))?;
    if schema != manifest.schema_version { return Err("Inconsistent backup schema version".into()); }
    if schema >= 6 {
        validate_receipt_attachment_evidence(&directory.join(DB_FILE), &directory, &manifest.files)?;
    }
    for ref_path in references {
        if !seen.contains(&ref_path) {
            return Err(format!("Immagine dell'opera non inclusa nel backup: {ref_path}"));
        }
    }
    Ok(BackupSummary {
        directory: crate::util::display_path(&directory),
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
        // Refuse an externally linked/replaced live media directory: otherwise
        // a backup could silently copy unrelated private files from outside
        // the archive. The restore/inspect path already enforces this check.
        check_media_directory(&db.directory)?;
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
        {
            // The handle must be closed before the folder is renamed: Windows refuses to
            // rename a directory that contains a file with an open handle.
            let mut output = File::create_new(partial.join(MANIFEST_FILE)).map_err(error)?;
            output.write_all(&serde_json::to_vec_pretty(&manifest).map_err(error)?).map_err(error)?;
            output.sync_all().map_err(error)?;
        }
        // Validation runs on the finished temporary tree before publication.
        inspect_backup_core(&partial)?;
        fs::rename(&partial, &target)
            .map_err(|e| format!("Impossibile pubblicare il backup (rinomina cartella): {e}"))?;
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
            Ok(RestoreSummary { restored: summary, previous_archive: crate::util::display_path(&saved) })
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
    fn seed_receipt_evidence(db: &Database) -> String {
        let relative = "media/receipt-00000000-0000-4000-8000-000000000001.pdf".to_string();
        let bytes = b"%PDF-1.7\nCollection proof";
        fs::write(db.directory.join(&relative), bytes).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
            VALUES('sale','a','c',1000,'2026-11-21','now')", []).unwrap();
        conn.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
            VALUES('sale',0,0,3334,3333,3333)", []).unwrap();
        conn.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
            VALUES('receipt','sale','2026-11-21',500,'cash','now')", []).unwrap();
        conn.execute("INSERT INTO sale_receipt_attachments
            (receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at)
            VALUES('receipt',?1,'proof.pdf','application/pdf',?2,?3,'now')",
            params![relative, bytes.len() as i64, format!("{:x}", Sha256::digest(bytes))]).unwrap();
        relative
    }
    fn refresh_manifest_record(folder: &Path, relative: &str) {
        let manifest_path = folder.join(MANIFEST_FILE);
        let mut manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        let record_to_update = manifest.files.iter_mut().find(|item| item.path == relative).unwrap();
        *record_to_update = record(folder, relative).unwrap();
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
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
        // The summary reports a canonical path without the Windows verbatim prefix.
        assert!(Path::new(&backup.directory).starts_with(crate::util::display_path(&fs::canonicalize(&destination).unwrap())));
        assert!(!backup.directory.starts_with(r"\\?\"), "users must not see verbatim prefixes");
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
    fn gallery_order_survives_verified_backup_and_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            for (id,sort) in [("detail",2),("back",1)] {
                conn.execute("INSERT INTO artwork_images
                    (id,artwork_id,image_path,caption,created_at,sort_order)
                    VALUES(?1,'a',?2,?1,'t',?3)",
                    params![id,format!("media/{id}.png"),sort]).unwrap();
                fs::write(db.directory.join(format!("media/{id}.png")),id.as_bytes()).unwrap();
            }
        }
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db,&destination).unwrap();
        assert_eq!(backup.schema_version,crate::db::SCHEMA_VERSION);
        assert!(backup.verified);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE artwork_images SET sort_order=100 WHERE id='back'",[]).unwrap();
        }
        restore_backup_core(&db,Path::new(&backup.directory)).unwrap();
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,sort_order,caption FROM artwork_images
            WHERE artwork_id='a' ORDER BY sort_order,id").unwrap();
        let order:Vec<(String,i64,String)> = stmt.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))
            .unwrap().map(Result::unwrap).collect();
        assert_eq!(order,vec![("back".into(),1,"back".into()),("detail".into(),2,"detail".into())]);
        assert_eq!(fs::read(db.directory.join("media/detail.png")).unwrap(),b"detail".to_vec());
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
        assert_eq!(backup.schema_version, crate::db::SCHEMA_VERSION);
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
    fn certificate_historical_photo_is_required_after_cover_change() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        // A certificate was issued with the old cover. The artwork now has
        // another cover and no gallery row references the original file.
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO certificates
                (id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                VALUES('cert','a','RS-1','2026-11-21','Artist',
                '{\"image_path\":\"media/example.png\"}','t')", []).unwrap();
            conn.execute("UPDATE artworks SET image_path='media/current.png' WHERE id='a'", []).unwrap();
        }
        fs::write(db.directory.join("media/current.png"), b"current cover").unwrap();
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        assert_eq!(backup.files, 3);
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);

        // Deliberately remove the historical image AND its manifest record.
        // Hashes and the live artwork cover are still fine: only the immutable
        // certificate snapshot can reveal the missing dependency.
        let manifest_path = folder.join(MANIFEST_FILE);
        let mut manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.files.retain(|entry| entry.path != "media/example.png");
        fs::remove_file(folder.join("media/example.png")).unwrap();
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();

        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("media/example.png"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        let conn = db.conn.lock().unwrap();
        let cover: String = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(cover, "media/current.png");
        assert!(db.directory.join("media/example.png").is_file());
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
            conn.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
                VALUES('s',0,0,3334,3333,3333)",[]).unwrap();
            conn.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                VALUES('r','s','2026-11-21',3500,'bank_transfer','t')",[]).unwrap();
        }
        let destination=tmp.path().join("removable");
        fs::create_dir(&destination).unwrap();
        let backup=create_backup_core(&db,&destination).unwrap();
        assert_eq!(backup.schema_version,crate::db::SCHEMA_VERSION);
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
    fn rehashed_backup_with_overpaid_sale_is_rejected_without_touching_live_data() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',10000,'2026-11-21','t')", []).unwrap();
            conn.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
                VALUES('s',0,0,3334,3333,3333)", []).unwrap();
            conn.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                VALUES('r1','s','2026-11-21',4000,'cash','t')", []).unwrap();
        }
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);

        // Simulate a manipulated but valid SQLite database whose write trigger
        // was removed. Recalculate its SHA-256 so the failure isn't an hash error.
        {
            let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
            snapshot.execute_batch(
                "DROP TRIGGER sale_receipts_limit_insert;
                 INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                 VALUES('r2','s','2026-11-22',7000,'cash','t');"
            ).unwrap();
            let integrity: String = snapshot.query_row(
                "PRAGMA integrity_check", [], |row| row.get(0)
            ).unwrap();
            assert_eq!(integrity, "ok");
            let foreign_keys: i64 = snapshot.query_row(
                "SELECT count(*) FROM pragma_foreign_key_check", [], |row| row.get(0)
            ).unwrap();
            assert_eq!(foreign_keys, 0);
        }
        let manifest_path = folder.join(MANIFEST_FILE);
        let mut manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        let db_record = manifest.files.iter_mut().find(|file| file.path == DB_FILE).unwrap();
        *db_record = record(folder, DB_FILE).unwrap();
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();

        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("incassi superiori"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM sale_receipts", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);
        assert!(db.directory.join(DB_FILE).exists());
    }

    #[test]
    fn rehashed_backup_with_invalid_receipt_date_is_rejected_before_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        seed_receipt_evidence(&db);
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);

        // SQLite accepts impossible TEXT dates and its integrity/foreign-key
        // checks still pass; the manifest can also be rehashed by an importer.
        for invalid in ["2026-02-30", "2026-2-3"] {
            {
                let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
                snapshot.execute("UPDATE sale_receipts SET received_at=?1 WHERE id='receipt'",
                    params![invalid]).unwrap();
                let integrity: String = snapshot.query_row(
                    "PRAGMA integrity_check", [], |row| row.get(0)
                ).unwrap();
                assert_eq!(integrity, "ok");
            }
            refresh_manifest_record(folder, DB_FILE);
            let message = inspect_backup_core(folder).unwrap_err();
            assert!(message.contains("data di incasso non valida"), "{message}");
            assert!(restore_backup_core(&db, folder).is_err());
        }
        let conn = db.conn.lock().unwrap();
        let date: String = conn.query_row(
            "SELECT received_at FROM sale_receipts WHERE id='receipt'", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(date, "2026-11-21", "the live archive must remain intact");
    }

    #[test]
    fn invalid_expense_dates_cannot_be_backed_up_or_restored() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO expenses(id,expense_date,category,description,amount_cents,created_at,updated_at)
                 VALUES('expense','2026-11-21','materials','Paint',250,'now','now')",
                [],
            ).unwrap();
        }
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);

        // A rehashed import with invalid TEXT dates passes SQLite page and
        // foreign-key checks, but would shift or hide costs in monthly reports.
        for invalid in ["2026-02-30", "2026-2-3", "not-a-date"] {
            {
                let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
                snapshot.execute(
                    "UPDATE expenses SET expense_date=?1 WHERE id='expense'",
                    params![invalid],
                ).unwrap();
                let integrity: String = snapshot.query_row(
                    "PRAGMA integrity_check", [], |row| row.get(0)
                ).unwrap();
                assert_eq!(integrity, "ok");
                let foreign_keys: i64 = snapshot.query_row(
                    "SELECT count(*) FROM pragma_foreign_key_check",
                    [], |row| row.get(0),
                ).unwrap();
                assert_eq!(foreign_keys, 0);
            }
            refresh_manifest_record(folder, DB_FILE);
            let message = inspect_backup_core(folder).unwrap_err();
            assert!(message.contains("data di spesa non valida"), "{message}");
            assert!(restore_backup_core(&db, folder).is_err());
        }
        let date: String = db.conn.lock().unwrap().query_row(
            "SELECT expense_date FROM expenses WHERE id='expense'",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(date, "2026-11-21", "the live archive must not be replaced");

        // Also refuse to publish a new backup if the active archive itself
        // contains invalid dates from an external SQLite import.
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE expenses SET expense_date='2026-02-30' WHERE id='expense'",
                [],
            ).unwrap();
        }
        let message = create_backup_core(&db, &destination).unwrap_err();
        assert!(message.contains("data di spesa non valida"), "{message}");
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 1,
            "a refused backup must not leave a published or partial folder");
    }

    #[test]
    fn rehashed_backup_with_invalid_sale_date_is_rejected_before_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        seed_receipt_evidence(&db);
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);

        // Neither PRAGMA integrity_check nor foreign_key_check validates the
        // contents of a TEXT date. Re-hash the SQLite file to prove the
        // semantic validation, rather than the manifest, blocks the restore.
        for invalid in ["2026-02-30", "2026-2-3", "not-a-date"] {
            {
                let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
                snapshot.execute(
                    "UPDATE sales SET sold_at=?1 WHERE id='sale'",
                    params![invalid],
                ).unwrap();
                let integrity: String = snapshot.query_row(
                    "PRAGMA integrity_check", [], |row| row.get(0)
                ).unwrap();
                assert_eq!(integrity, "ok");
                let foreign_keys: i64 = snapshot.query_row(
                    "SELECT count(*) FROM pragma_foreign_key_check", [], |row| row.get(0)
                ).unwrap();
                assert_eq!(foreign_keys, 0);
            }
            refresh_manifest_record(folder, DB_FILE);
            let message = inspect_backup_core(folder).unwrap_err();
            assert!(message.contains("data di vendita non valida"), "{message}");
            assert!(restore_backup_core(&db, folder).is_err());
        }
        let conn = db.conn.lock().unwrap();
        let date: String = conn.query_row(
            "SELECT sold_at FROM sales WHERE id='sale'", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(date, "2026-11-21", "the live archive must stay intact");
        assert!(db.directory.join(DB_FILE).is_file());
    }

    #[test]
    fn rehashed_backup_with_invalid_installment_schedule_cannot_replace_live_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('sale','a','c',1000,'2026-11-21','now')", []).unwrap();
            conn.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
                VALUES('sale',0,0,3334,3333,3333)", []).unwrap();
            conn.execute_batch("INSERT INTO sale_installments(id,sale_id,position,due_date,amount_cents)
                VALUES('i1','sale',1,'2026-12-01',400);
                INSERT INTO sale_installments(id,sale_id,position,due_date,amount_cents)
                VALUES('i2','sale',2,'2027-01-01',600);").unwrap();
        }
        let external = tmp.path().join("external");
        fs::create_dir(&external).unwrap();
        let backup = create_backup_core(&db, &external).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);
        let original_bytes = fs::read(folder.join(DB_FILE)).unwrap();

        // All cases remain valid SQLite and preserve FK relationships. The
        // manifest is refreshed, so semantic validation must reject them.
        for (sql, expected) in [
            ("UPDATE sale_installments SET due_date='2026-02-30' WHERE id='i1'",
             "data di scadenza non valida"),
            ("UPDATE sale_installments SET due_date='2026-2-3' WHERE id='i1'",
             "data di scadenza non valida"),
            ("UPDATE sale_installments SET position=3 WHERE id='i2'",
             "posizioni delle rate non consecutive"),
            ("UPDATE sale_installments SET due_date='2026-01-01' WHERE id='i2'",
             "fuori ordine di scadenza"),
            ("DELETE FROM sale_installments WHERE id='i2'",
             "meno di due rate"),
        ] {
            fs::write(folder.join(DB_FILE), &original_bytes).unwrap();
            {
                let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
                snapshot.execute_batch(sql).unwrap();
                let integrity: String = snapshot.query_row(
                    "PRAGMA integrity_check", [], |row| row.get(0)
                ).unwrap();
                assert_eq!(integrity, "ok", "{sql}");
                let foreign_keys: i64 = snapshot.query_row(
                    "SELECT count(*) FROM pragma_foreign_key_check", [], |row| row.get(0)
                ).unwrap();
                assert_eq!(foreign_keys, 0, "{sql}");
            }
            refresh_manifest_record(folder, DB_FILE);
            let message = inspect_backup_core(folder).unwrap_err();
            assert!(message.contains(expected), "{sql}: {message}");
            assert!(restore_backup_core(&db, folder).is_err(), "{sql}");
        }
        let conn = db.conn.lock().unwrap();
        let rows: i64 = conn.query_row(
            "SELECT count(*) FROM sale_installments WHERE sale_id='sale'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(rows, 2, "the live installment plan must remain unchanged");
        assert!(db.directory.join(DB_FILE).is_file());
    }

    #[test]
    fn missing_sale_finances_prevents_publishing_a_new_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            // Simulate a legacy import made without the normal sale transaction.
            conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('missing','a','c',1000,'2026-11-21','t')", []).unwrap();
        }
        let destination = tmp.path().join("usb-drive");
        fs::create_dir(&destination).unwrap();
        let message = create_backup_core(&db, &destination).unwrap_err();
        assert!(message.contains("dettagli economici"), "{message}");
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 0);
        assert!(db.directory.join(DB_FILE).is_file());
    }

    #[test]
    fn rehashed_backup_with_overbudget_taxes_is_rejected_before_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('sale','a','c',1000,'2026-11-21','t')", []).unwrap();
            conn.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
                VALUES('sale',0,0,3334,3333,3333)", []).unwrap();
        }
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);
        {
            let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
            // Each field passes its own SQLite CHECK, but their sum exceeds the sale.
            snapshot.execute("UPDATE sale_finances SET tax_cents=950,shipping_cents=100
                WHERE sale_id='sale'", []).unwrap();
        }
        refresh_manifest_record(folder, DB_FILE);
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("dettagli economici"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        let conn = db.conn.lock().unwrap();
        let tax: i64 = conn.query_row("SELECT tax_cents FROM sale_finances WHERE sale_id='sale'",
            [], |row| row.get(0)).unwrap();
        assert_eq!(tax, 0, "the live archive must not be replaced by the corrupt snapshot");
    }

    #[test]
    fn rehashed_corrupted_receipt_media_is_rejected_before_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let relative = seed_receipt_evidence(&db);
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);

        // An attacker (or repair tool) updates the manifest checksum to match
        // corrupt media, but cannot change the original hash in SQLite.
        fs::write(folder.join(&relative), b"%PDF-1.7\nAltered evidence").unwrap();
        refresh_manifest_record(folder, &relative);
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("metadati SQLite"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        assert!(db.directory.join(&relative).is_file());
        assert!(db.directory.join(DB_FILE).is_file());
    }

    #[test]
    fn rehashed_sqlite_receipt_metadata_is_rejected_without_touching_live_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let relative = seed_receipt_evidence(&db);
        let destination = tmp.path().join("drive");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        {
            let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
            snapshot.execute("UPDATE sale_receipt_attachments SET sha256=?1 WHERE receipt_id='receipt'",
                params!["0".repeat(64)]).unwrap();
        }
        refresh_manifest_record(folder, DB_FILE);
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("metadati SQLite"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        let active_hash: String = db.conn.lock().unwrap().query_row(
            "SELECT sha256 FROM sale_receipt_attachments WHERE receipt_id='receipt'",
            [], |r| r.get(0)).unwrap();
        assert_eq!(active_hash, format!("{:x}", Sha256::digest(fs::read(db.directory.join(relative)).unwrap())));
    }

    #[test]
    fn active_receipt_corruption_prevents_publishing_a_new_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let relative = seed_receipt_evidence(&db);
        fs::write(db.directory.join(relative), b"%PDF-1.7\nCorrupt source").unwrap();
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let message = create_backup_core(&db, &destination).unwrap_err();
        assert!(message.contains("metadati SQLite"), "{message}");
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 0);
    }

    #[test]
    fn rehashed_wrong_receipt_mime_is_rejected_before_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        seed_receipt_evidence(&db);
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        {
            let snapshot = Connection::open(folder.join(DB_FILE)).unwrap();
            snapshot.execute("UPDATE sale_receipt_attachments SET mime_type='image/png'
                WHERE receipt_id='receipt'", []).unwrap();
        }
        refresh_manifest_record(folder, DB_FILE);
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("Formato allegato incasso"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
    }

    #[test]
    fn collector_photographs_travel_in_the_backup_and_are_required_on_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO collectors(id,name,first_name,created_at,updated_at) VALUES('c9','Anna','Anna','t','t')", []).unwrap();
            conn.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at) VALUES('i9','c9','PROFILE','media/collector-profile.jpg','t')", []).unwrap();
        }
        fs::write(db.directory.join("media/collector-profile.jpg"), [0xff, 0xd8, 0xff, 9]).unwrap();
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = PathBuf::from(&backup.directory);
        assert!(folder.join("media/collector-profile.jpg").is_file());
        fs::remove_file(folder.join("media/collector-profile.jpg")).unwrap();
        assert!(inspect_backup_core(&folder).is_err(), "a backup missing a collector photograph is refused");
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
    fn unlisted_media_is_rejected_before_restore_and_live_archive_stays_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let external = tmp.path().join("external");
        fs::create_dir(&external).unwrap();
        let backup = create_backup_core(&db, &external).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);
        // An orphan file is not referenced by SQLite, so checking only the
        // database and manifest hashes would previously miss this data loss.
        fs::write(folder.join("media/orphan.bin"), b"do not silently lose me").unwrap();
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("media/orphan.bin"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        assert_eq!(fs::read(db.directory.join("media/example.png")).unwrap(), vec![1,2,3,4]);
        assert!(!db.directory.join("media/orphan.bin").exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_backup_media_directory_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let external = tmp.path().join("external");
        fs::create_dir(&external).unwrap();
        let backup = create_backup_core(&db, &external).unwrap();
        let folder = Path::new(&backup.directory);
        let actual_media = tmp.path().join("outside-media");
        fs::rename(folder.join("media"), &actual_media).unwrap();
        std::os::unix::fs::symlink(&actual_media, folder.join("media")).unwrap();
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("directory regolare"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
        assert!(db.directory.join("media/example.png").is_file());
    }

    #[test]
    fn untracked_sqlite_sidecars_are_rejected_without_replacing_live_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        let destination = tmp.path().join("external-drive");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        assert!(inspect_backup_core(folder).unwrap().verified);
        for suffix in ["-wal", "-shm", "-journal"] {
            let sidecar = folder.join(format!("{DB_FILE}{suffix}"));
            fs::write(&sidecar, b"untracked SQLite state").unwrap();
            let message = inspect_backup_core(folder).unwrap_err();
            assert!(message.contains("SQLite ausiliario"), "{message}");
            assert!(restore_backup_core(&db, folder).is_err());
            assert!(db.directory.join(DB_FILE).is_file());
            assert_eq!(fs::read(db.directory.join("media/example.png")).unwrap(), [1, 2, 3, 4]);
            fs::remove_file(sidecar).unwrap();
        }
        assert!(inspect_backup_core(folder).unwrap().verified);
    }

    #[cfg(unix)]
    #[test]
    fn dangling_sqlite_wal_symlink_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        let destination = tmp.path().join("external");
        fs::create_dir(&destination).unwrap();
        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        let sidecar = folder.join(format!("{DB_FILE}-wal"));
        std::os::unix::fs::symlink(tmp.path().join("not-present"), sidecar).unwrap();
        let message = inspect_backup_core(folder).unwrap_err();
        assert!(message.contains("SQLite ausiliario"), "{message}");
        assert!(restore_backup_core(&db, folder).is_err());
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
    fn missing_live_media_directory_does_not_publish_a_partial_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        let destination = tmp.path().join("removable-drive");
        fs::create_dir(&destination).unwrap();
        fs::remove_dir(db.directory.join("media")).unwrap();

        assert!(create_backup_core(&db, &destination).is_err());
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 0);
        // A failed backup must not touch the live SQLite file.
        assert!(db.directory.join(DB_FILE).is_file());
        let count: i64 = db.conn.lock().unwrap().query_row(
            "SELECT COUNT(*) FROM artworks", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[cfg(unix)]
    #[test]
    fn linked_live_media_directory_cannot_leak_external_files_into_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let db = db::open(tmp.path().join("active")).unwrap();
        seed(&db);
        // Simulate a replaced media folder pointing to another private archive
        // (or to arbitrary files outside Repertica Studio).
        let private_files = tmp.path().join("another-archive");
        fs::rename(db.directory.join("media"), &private_files).unwrap();
        fs::write(private_files.join("unrelated-secret.txt"), b"external private data").unwrap();
        std::os::unix::fs::symlink(&private_files, db.directory.join("media")).unwrap();
        let destination = tmp.path().join("removable-drive");
        fs::create_dir(&destination).unwrap();

        let message = create_backup_core(&db, &destination).unwrap_err();
        assert!(message.contains("directory regolare"), "{message}");
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 0);
        assert_eq!(fs::read(private_files.join("unrelated-secret.txt")).unwrap(),
            b"external private data");
        assert!(db.directory.join(DB_FILE).is_file());
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
