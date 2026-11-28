//! Native preflight and per-package review/commit for a local batch directory.
//! Native commands are registered by the desktop entrypoint.
//! A directory is not a database transaction or a filesystem snapshot.
use super::{apply_with_media, digest, ensure_new, parse, preview, Preview, MAX_BYTES, MAX_ROWS, MAX_IMAGES, RELATIONAL_FORMAT};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs::{self, File, Metadata}, io::Read, path::{Path, PathBuf}};

const FORMAT: &str = "repertica-studio-media-batches-v1";
const MANIFEST_LIMIT: usize = 1024 * 1024;
const MAX_TOTAL_PHOTOS: usize = 21_000;

fn invalid() -> String { "batch_manifest".into() }
fn unsafe_path() -> String { "batch_path".into() }
fn changed() -> String { "batch_changed".into() }

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    file: String,
    sha256: String,
    bytes: usize,
    artworks: usize,
    collectors: usize,
    sales: usize,
    photographs: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    packages: Vec<Entry>,
}

#[derive(Serialize, Debug)]
pub struct BatchPlan {
    manifest_digest: String,
    packages: Vec<Entry>,
    artworks: usize,
    collectors: usize,
    sales: usize,
    photographs: usize,
}

// Windows junctions/reparse points are not all reported as symbolic links.
fn is_link(meta: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    { meta.file_type().is_symlink() }
}

fn root(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() { return Err(unsafe_path()); }
    let meta = fs::symlink_metadata(path).map_err(|_| unsafe_path())?;
    if !meta.is_dir() || is_link(&meta) { return Err(unsafe_path()); }
    fs::canonicalize(path).map_err(|_| unsafe_path())
}

// Only fixed manifest/package names are passed to this function. Read bounded
// bytes, not metadata-sized allocations. A trusted static directory is required;
// portable path checks are not a defense against an active local filesystem attacker.
fn read_regular(dir: &Path, name: &str, limit: usize) -> Result<String, String> {
    let path = dir.join(name);
    let meta = fs::symlink_metadata(&path).map_err(|_| unsafe_path())?;
    if !meta.is_file() || is_link(&meta) || meta.len() > limit as u64 { return Err(unsafe_path()); }
    let canonical = fs::canonicalize(&path).map_err(|_| unsafe_path())?;
    if canonical.parent() != Some(dir) { return Err(unsafe_path()); }
    let file = File::open(&canonical).map_err(|_| unsafe_path())?;
    let opened = file.metadata().map_err(|_| unsafe_path())?;
    if !opened.is_file() || is_link(&opened) || opened.len() != meta.len() { return Err(changed()); }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes).map_err(|_| unsafe_path())?;
    let after = fs::symlink_metadata(&path).map_err(|_| changed())?;
    if bytes.len() > limit || bytes.len() as u64 != opened.len() || !after.is_file()
        || is_link(&after) || after.len() != opened.len()
        || fs::canonicalize(&path).map_err(|_| changed())? != canonical {
        return Err(changed());
    }
    String::from_utf8(bytes).map_err(|_| invalid())
}

fn exact_entries(dir: &Path, packages: &[Entry]) -> Result<(), String> {
    let mut expected = HashSet::from(["manifest.json".to_string()]);
    expected.extend(packages.iter().map(|row| row.file.clone()));
    for item in fs::read_dir(dir).map_err(|_| unsafe_path())? {
        let item = item.map_err(|_| unsafe_path())?;
        let name = item.file_name().into_string().map_err(|_| invalid())?;
        if !expected.remove(&name) { return Err(invalid()); }
        let meta = fs::symlink_metadata(item.path()).map_err(|_| unsafe_path())?;
        if !meta.is_file() || is_link(&meta) { return Err(unsafe_path()); }
    }
    if !expected.is_empty() { return Err(invalid()); }
    Ok(())
}

fn load_plan(dir: &Path) -> Result<BatchPlan, String> {
    let source = read_regular(dir, "manifest.json", MANIFEST_LIMIT)?;
    let manifest: Manifest = serde_json::from_str(&source).map_err(|_| invalid())?;
    if manifest.format != FORMAT || !(1..=MAX_ROWS).contains(&manifest.packages.len()) {
        return Err(invalid());
    }
    let mut plan = BatchPlan { manifest_digest: digest(&source), packages: manifest.packages,
        artworks: 0, collectors: 0, sales: 0, photographs: 0 };
    for (i, row) in plan.packages.iter().enumerate() {
        if row.file != format!("package-{:04}.json", i + 1)
            || row.sha256.len() != 64 || !row.sha256.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !(1..=MAX_BYTES).contains(&row.bytes) || !(1..=MAX_ROWS).contains(&row.artworks)
            || row.collectors > MAX_ROWS || row.sales > MAX_ROWS || row.photographs > MAX_IMAGES {
            return Err(invalid());
        }
        plan.artworks += row.artworks;
        plan.collectors += row.collectors;
        plan.sales += row.sales;
        plan.photographs += row.photographs;
    }
    if plan.artworks > MAX_ROWS || plan.collectors > MAX_ROWS || plan.sales > MAX_ROWS
        || plan.photographs > MAX_TOTAL_PHOTOS { return Err(invalid()); }
    exact_entries(dir, &plan.packages)?;
    Ok(plan)
}

fn package_source(dir: &Path, entry: &Entry) -> Result<String, String> {
    let source = read_regular(dir, &entry.file, MAX_BYTES)?;
    if source.len() != entry.bytes || digest(&source) != entry.sha256 { return Err(changed()); }
    // The batch exporter always emits all six keys, including empty photo lists.
    let value: serde_json::Value = serde_json::from_str(&source).map_err(|_| invalid())?;
    let obj = value.as_object().ok_or_else(invalid)?;
    if obj.len() != 6 || obj.get("format").and_then(|v| v.as_str()) != Some(RELATIONAL_FORMAT)
        || !["artworks", "collectors", "sales", "covers", "gallery"].iter()
            .all(|k| obj.get(*k).is_some_and(|v| v.is_array())) { return Err(invalid()); }
    Ok(source)
}

fn checked_package(source: &str, entry: &Entry) -> Result<super::Parsed, String> {
    // Reuse the normal native semantic/image decoder (including unknown/duplicate
    // fields), not just the manifest counts or the JS header-only preflight.
    let parsed = parse(source)?;
    if parsed.artworks.len() != entry.artworks || parsed.collectors.len() != entry.collectors
        || parsed.sales.len() != entry.sales || parsed.covers.len() != entry.photographs {
        return Err(invalid());
    }
    Ok(parsed)
}

fn same_manifest(dir: &Path, expected: &str) -> Result<BatchPlan, String> {
    let plan = load_plan(dir)?;
    if plan.manifest_digest != expected { return Err(changed()); }
    Ok(plan)
}

/// All packages, all semantic checks, cross-package conflicts, and the current
/// SQLite archive. No files or records written. Retain only one decoded package.
pub(crate) fn preflight(conn: &Connection, directory: &Path) -> Result<BatchPlan, String> {
    let dir = root(directory)?;
    let plan = load_plan(&dir)?;
    let mut codes = HashSet::new();
    let mut names = HashSet::new();
    let mut emails = HashSet::new();
    let mut collector_keys = HashSet::new();
    for entry in &plan.packages {
        let source = package_source(&dir, entry)?;
        let parsed = checked_package(&source, entry)?;
        ensure_new(conn, &parsed)?;
        for row in &parsed.artworks {
            if !codes.insert(row.code.to_ascii_lowercase()) { return Err("batch_duplicate".into()); }
        }
        // Batch exports must keep all purchases of a source collector together.
        // Single-package import keys are local, but the batch exporter uses one
        // source key per collector across the entire selection.
        let used_keys: HashSet<&str> = parsed.sales.iter().map(|sale| sale.collector_key.as_str()).collect();
        for row in &parsed.collectors {
            if !used_keys.contains(row.key.as_str()) { return Err("relations".into()); }
            if !collector_keys.insert(row.key.clone()) || !names.insert(row.name().to_ascii_lowercase())
                || row.email.as_ref().is_some_and(|email| !emails.insert(email.to_ascii_lowercase())) {
                return Err("batch_duplicate".into());
            }
        }
    }
    same_manifest(&dir, &plan.manifest_digest)?;
    Ok(plan)
}

/// Zero-based index into the reviewed manifest. A new DB preview is required for
/// every package, including after previous packages have committed.
pub(crate) fn review(conn: &Connection, directory: &Path, expected_manifest: &str, index: usize) -> Result<Preview, String> {
    let dir = root(directory)?;
    let plan = same_manifest(&dir, expected_manifest)?;
    let entry = plan.packages.get(index).ok_or_else(invalid)?;
    let source = package_source(&dir, entry)?;
    checked_package(&source, entry)?;
    let result = preview(conn, &source)?;
    same_manifest(&dir, expected_manifest)?;
    Ok(result)
}

/// One explicit confirmation, one transaction. No automatic retry: if the
/// response is lost, the UI must treat the outcome as unknown and inspect the DB.
/// Already committed packages are not undone by later failures.
pub(crate) fn commit(conn: &mut Connection, media_root: &Path, directory: &Path,
    expected_manifest: &str, index: usize, expected_preview: &str) -> Result<usize, String> {
    let dir = root(directory)?;
    let plan = same_manifest(&dir, expected_manifest)?;
    let entry = plan.packages.get(index).ok_or_else(invalid)?;
    if entry.sha256 != expected_preview { return Err(changed()); }
    let source = package_source(&dir, entry)?;
    checked_package(&source, entry)?;
    same_manifest(&dir, expected_manifest)?;
    apply_with_media(conn, Some(media_root), &source, expected_preview)
}

// IPC wrappers registered by the shared entrypoint.
// Keep the archive lock through the complete native operation.
#[tauri::command]
pub fn preflight_archive_batches(db: tauri::State<'_, crate::db::Database>,
    directory: String) -> Result<BatchPlan, String> {
    let conn = db.conn.lock().map_err(super::database_error)?;
    preflight(&conn, Path::new(&directory))
}

#[tauri::command]
pub fn review_archive_batch(db: tauri::State<'_, crate::db::Database>,
    directory: String, expected_manifest: String, index: usize) -> Result<Preview, String> {
    let conn = db.conn.lock().map_err(super::database_error)?;
    review(&conn, Path::new(&directory), &expected_manifest, index)
}

#[tauri::command]
pub fn commit_archive_batch(db: tauri::State<'_, crate::db::Database>,
    directory: String, expected_manifest: String, index: usize,
    expected_preview: String) -> Result<usize, String> {
    let mut conn = db.conn.lock().map_err(super::database_error)?;
    commit(&mut conn, &db.directory, Path::new(&directory), &expected_manifest, index, &expected_preview)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(dir: &Path, codes: &[&str]) {
        let mut entries = vec![];
        for (i, code) in codes.iter().enumerate() {
            let file = format!("package-{:04}.json", i + 1);
            let source = json!({"format":RELATIONAL_FORMAT,"artworks":[{"code":code,"title":"Synthetic","status":"available"}],
                "collectors":[],"sales":[],"covers":[],"gallery":[]}).to_string();
            fs::write(dir.join(&file), &source).unwrap();
            entries.push(json!({"file":file,"sha256":digest(&source),"bytes":source.len(),
                "artworks":1,"collectors":0,"sales":0,"photographs":0}));
        }
        fs::write(dir.join("manifest.json"), json!({"format":FORMAT,"packages":entries}).to_string()).unwrap();
    }
    fn count(conn: &Connection) -> i64 {
        conn.query_row("SELECT count(*) FROM artworks", [], |r| r.get(0)).unwrap()
    }

    // This gate is explicitly invoked by the real PostgreSQL exporter test.
    // Ordinary cargo tests have no PostgreSQL source fixture; an explicit gate
    // invocation without the fixture must fail, never silently pass.
    #[test]
    #[ignore = "requires the disposable PostgreSQL exporter fixture"]
    fn postgres_export_native_roundtrip() {
        use crate::backup::{create_backup_core, inspect_backup_core, restore_backup_core};
        use base64::{engine::general_purpose::STANDARD, Engine};
        let export = PathBuf::from(std::env::var_os("REPERTICA_TEST_BATCH_DIRECTORY")
            .expect("missing disposable PostgreSQL fixture"));
        let root = tempfile::tempdir().unwrap();
        let active = root.path().join("active");
        let destination = root.path().join("backup");
        fs::create_dir(&destination).unwrap();
        let db = crate::db::open(active.clone()).unwrap();
        let plan = {
            let conn = db.conn.lock().unwrap();
            let plan = preflight(&conn, &export).unwrap();
            assert_eq!((plan.packages.len(), plan.artworks, plan.collectors, plan.sales, plan.photographs),
                       (1, 3, 1, 2, 3));
            assert_eq!(count(&conn), 0);
            plan
        };
        let source = fs::read_to_string(export.join("package-0001.json")).unwrap();
        let package: serde_json::Value = serde_json::from_str(&source).unwrap();
        let cover = STANDARD.decode(package["covers"][0]["data_base64"].as_str().unwrap()).unwrap();
        let gallery: Vec<Vec<u8>> = package["gallery"].as_array().unwrap().iter()
            .map(|p| STANDARD.decode(p["data_base64"].as_str().unwrap()).unwrap()).collect();
        assert_eq!(gallery.len(), 2);

        let assert_archive = |conn: &Connection| -> Vec<String> {
            for (table, expected) in [("artworks",3),("collectors",1),("sales",2),
                ("sale_receipts",1),("sale_installments",2),("artwork_images",2)] {
                let n: i64 = conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap();
                assert_eq!(n, expected, "{table}");
            }
            let art: (String, f64, i64, String) = conn.query_row(
                "SELECT title,width_cm,fixed_price_cents,image_path FROM artworks WHERE code='PG-1' AND status='available'",
                [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
            assert_eq!((art.0.as_str(), art.1, art.2), ("Città 🎨", 30.25, 29));
            assert_eq!(fs::read(active.join(&art.3)).unwrap(), cover);
            let buyer: (String,String,String) = conn.query_row(
                "SELECT name,birth_date,birth_time FROM collectors", [],
                |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
            assert_eq!(buyer, ("Synthetic Collector".into(),"1980-02-29".into(),"07:45".into()));
            let sale: (i64,i64,i64,i64,i64,i64,i64,String) = conn.query_row(
                "SELECT s.amount_cents,f.tax_cents,f.shipping_cents,f.personal_bp,f.activity_bp,f.savings_bp,r.amount_cents,r.method
                 FROM sales s JOIN artworks a ON a.id=s.artwork_id JOIN sale_finances f ON f.sale_id=s.id
                 JOIN sale_receipts r ON r.sale_id=s.id WHERE a.code='PG-2' AND a.status='sold'",
                [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).unwrap();
            assert_eq!(sale,(10025,1000,525,5000,3000,2000,29,"bank_transfer".into()));
            let linked: i64 = conn.query_row("SELECT count(DISTINCT collector_id) FROM sales", [], |r| r.get(0)).unwrap();
            assert_eq!(linked, 1);
            let mut stmt = conn.prepare(
                "SELECT i.position,i.due_date,i.amount_cents,i.notes FROM sale_installments i
                 JOIN sales s ON s.id=i.sale_id JOIN artworks a ON a.id=s.artwork_id
                 WHERE a.code='PG-3' AND a.status='sold' AND s.amount_cents=10025
                 AND NOT EXISTS(SELECT 1 FROM sale_receipts r WHERE r.sale_id=s.id) ORDER BY i.position").unwrap();
            let installments: Vec<(i64,String,i64,Option<String>)> = stmt.query_map([], |r|
                Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(installments, vec![(1,"2024-03-31".into(),4025,Some("first".into())),
                                         (2,"2024-04-30".into(),6000,None)]);
            let mut stmt = conn.prepare(
                "SELECT g.image_path,g.sort_order,g.kind,g.caption FROM artwork_images g
                 JOIN artworks a ON a.id=g.artwork_id WHERE a.code='PG-2' ORDER BY g.sort_order").unwrap();
            let rows: Vec<(String,i64,String,String)> = stmt.query_map([], |r|
                Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(rows.len(),2);
            assert_eq!((rows[0].1,rows[0].2.as_str(),rows[0].3.as_str()),(1,"DETAIL","Dettaglio 🎨"));
            assert_eq!((rows[1].1,rows[1].2.as_str(),rows[1].3.as_str()),(2,"SIGNATURE","Firma"));
            let mut paths = vec![art.3];
            for (i,row) in rows.iter().enumerate() {
                assert_eq!(fs::read(active.join(&row.0)).unwrap(), gallery[i]);
                paths.push(row.0.clone());
            }
            assert_eq!(paths.iter().collect::<HashSet<_>>().len(),3);
            assert_eq!(conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get::<_,i64>(0)).unwrap(),0);
            paths
        };
        let paths = {
            let mut conn = db.conn.lock().unwrap();
            let reviewed = review(&conn, &export, &plan.manifest_digest, 0).unwrap();
            assert_eq!(count(&conn), 0);
            // Fail late in a real exported package, after the transaction has
            // reached its commercial records. Verify both SQLite and media
            // cleanup before the same reviewed package is committed normally.
            conn.execute_batch("CREATE TRIGGER roundtrip_reject_receipt BEFORE INSERT ON sale_receipts BEGIN SELECT RAISE(IGNORE); END;").unwrap();
            assert!(commit(&mut conn, &active, &export, &plan.manifest_digest, 0, &reviewed.digest).is_err());
            for table in ["artworks","collectors","sales","sale_finances","sale_receipts",
                          "sale_installments","artwork_images","archive_code_sequences"] {
                let n: i64 = conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap();
                assert_eq!(n, 0, "rollback: {table}");
            }
            assert_eq!(fs::read_dir(active.join("media")).unwrap().count(), 0);
            assert_eq!(fs::read_to_string(export.join("package-0001.json")).unwrap(), source);
            conn.execute_batch("DROP TRIGGER roundtrip_reject_receipt;").unwrap();
            assert_eq!(commit(&mut conn, &active, &export, &plan.manifest_digest, 0, &reviewed.digest).unwrap(),3);
            assert!(commit(&mut conn, &active, &export, &plan.manifest_digest, 0, &reviewed.digest).is_err());
            assert_archive(&conn)
        };

        let backup = create_backup_core(&db, &destination).unwrap();
        let folder = Path::new(&backup.directory);
        inspect_backup_core(folder).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            assert_eq!(conn.execute("UPDATE artworks SET title='later local edit' WHERE code='PG-1'", []).unwrap(),1);
        }
        let backed_cover = folder.join(&paths[0]);
        fs::write(&backed_cover, b"damaged").unwrap();
        assert!(inspect_backup_core(folder).is_err());
        assert!(restore_backup_core(&db, folder).is_err());
        {
            let conn = db.conn.lock().unwrap();
            let title: String = conn.query_row("SELECT title FROM artworks WHERE code='PG-1'", [], |r| r.get(0)).unwrap();
            assert_eq!(title,"later local edit");
        }
        assert_eq!(fs::read(active.join(&paths[0])).unwrap(),cover);
        fs::write(&backed_cover, &cover).unwrap();
        let restored = restore_backup_core(&db, folder).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            assert_eq!(assert_archive(&conn), paths);
        }
        let previous = Connection::open_with_flags(Path::new(&restored.previous_archive).join("repertica.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let title: String = previous.query_row("SELECT title FROM artworks WHERE code='PG-1'", [], |r| r.get(0)).unwrap();
        assert_eq!(title, "later local edit");
        assert_eq!(fs::read_to_string(export.join("package-0001.json")).unwrap(),source);
    }

    #[test]
    fn preflight_and_review_do_not_write_and_commit_is_explicit_per_package() {
        let export = tempfile::tempdir().unwrap();
        let archive = tempfile::tempdir().unwrap();
        fixture(export.path(), &["BATCH-1", "BATCH-2"]);
        let db = crate::db::open(archive.path().to_path_buf()).unwrap();
        let mut conn = db.conn.lock().unwrap();
        let plan = preflight(&conn, export.path()).unwrap();
        assert_eq!(plan.artworks, 2);
        let one = review(&conn, export.path(), &plan.manifest_digest, 0).unwrap();
        assert_eq!(count(&conn), 0);
        assert_eq!(commit(&mut conn, archive.path(), export.path(), &plan.manifest_digest, 0, &one.digest).unwrap(), 1);
        assert_eq!(count(&conn), 1);
        assert!(commit(&mut conn, archive.path(), export.path(), &plan.manifest_digest, 0, &one.digest).is_err());
        let two = review(&conn, export.path(), &plan.manifest_digest, 1).unwrap();
        conn.execute_batch("CREATE TRIGGER stop_second BEFORE INSERT ON artworks BEGIN SELECT RAISE(IGNORE); END;").unwrap();
        assert!(commit(&mut conn, archive.path(), export.path(), &plan.manifest_digest, 1, &two.digest).is_err());
        assert_eq!(count(&conn), 1);
        conn.execute_batch("DROP TRIGGER stop_second;").unwrap();
        assert_eq!(commit(&mut conn, archive.path(), export.path(), &plan.manifest_digest, 1, &two.digest).unwrap(), 1);
        assert_eq!(count(&conn), 2);
    }

    #[test]
    fn rejects_cross_package_duplicates_and_manifest_or_file_changes() {
        let export = tempfile::tempdir().unwrap();
        let archive = tempfile::tempdir().unwrap();
        let db = crate::db::open(archive.path().to_path_buf()).unwrap();
        let mut conn = db.conn.lock().unwrap();
        fixture(export.path(), &["BATCH-A", "batch-a"]);
        assert_eq!(preflight(&conn, export.path()).unwrap_err(), "batch_duplicate");
        fixture(export.path(), &["BATCH-A", "BATCH-B"]);
        let plan = preflight(&conn, export.path()).unwrap();
        let first = review(&conn, export.path(), &plan.manifest_digest, 0).unwrap();
        let file = export.path().join("package-0001.json");
        let original = fs::read_to_string(&file).unwrap();
        fs::write(&file, original.replace("Synthetic", "Tampering")).unwrap();
        assert!(commit(&mut conn, archive.path(), export.path(), &plan.manifest_digest, 0, &first.digest).is_err());
        fs::write(&file, original).unwrap();
        let manifest = export.path().join("manifest.json");
        let old = fs::read_to_string(&manifest).unwrap();
        fs::write(&manifest, format!("{} ", old)).unwrap();
        assert!(review(&conn, export.path(), &plan.manifest_digest, 0).is_err());
        fs::write(&manifest, old).unwrap();
        fs::write(export.path().join("unexpected"), b"").unwrap();
        assert!(preflight(&conn, export.path()).is_err());
        assert_eq!(count(&conn), 0);
    }

    #[test]
    fn batch_preflight_rejects_split_source_keys_and_orphan_collectors() {
        let export = tempfile::tempdir().unwrap();
        let archive = tempfile::tempdir().unwrap();
        let db = crate::db::open(archive.path().to_path_buf()).unwrap();
        let conn = db.conn.lock().unwrap();
        let write_packages = |second_key: &str, orphan: bool| {
            let mut entries = vec![];
            for (i, (code, key, name)) in [("SOLD-A", "source-1", "Ada"), ("SOLD-B", second_key, "Bob")].into_iter().enumerate() {
                let file = format!("package-{:04}.json", i + 1);
                let mut collectors = vec![json!({"key":key,"first_name":name})];
                if orphan && i == 1 {
                    collectors.push(json!({"key":"unused","first_name":"Unused"}));
                }
                let source = json!({"format":RELATIONAL_FORMAT,
                    "artworks":[{"code":code,"title":"Synthetic","status":"sold"}],
                    "collectors":collectors,
                    "sales":[{"artwork_code":code,"collector_key":key,"currency":"EUR",
                        "sold_at":"2025-03-04","amount_cents":10000,"tax_cents":1000,
                        "shipping_cents":500,"legacy_shares_bp":[2000,3000,5000],
                        "allocations":[],"receipts":[],"installments":[]}],
                    "covers":[],"gallery":[]}).to_string();
                fs::write(export.path().join(&file), &source).unwrap();
                entries.push(json!({"file":file,"sha256":digest(&source),"bytes":source.len(),
                    "artworks":1,"collectors":collectors.len(),"sales":1,"photographs":0}));
            }
            fs::write(export.path().join("manifest.json"),
                json!({"format":FORMAT,"packages":entries}).to_string()).unwrap();
        };
        write_packages("source-1", false);
        assert_eq!(preflight(&conn, export.path()).unwrap_err(), "batch_duplicate");
        write_packages("source-2", true);
        assert_eq!(preflight(&conn, export.path()).unwrap_err(), "relations");
        for table in ["artworks", "collectors", "sales", "archive_code_sequences"] {
            let n: i64 = conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap();
            assert_eq!(n, 0, "{table}");
        }
        write_packages("source-2", false);
        let plan = preflight(&conn, export.path()).unwrap();
        assert_eq!((plan.artworks, plan.collectors, plan.sales), (2, 2, 2));
        assert_eq!(count(&conn), 0);
    }

    #[test]
    fn rejects_wrong_counts_and_existing_archive_conflict_before_any_commit() {
        let export = tempfile::tempdir().unwrap();
        let archive = tempfile::tempdir().unwrap();
        let db = crate::db::open(archive.path().to_path_buf()).unwrap();
        let mut conn = db.conn.lock().unwrap();
        fixture(export.path(), &["BATCH-1", "BATCH-2"]);
        let manifest = export.path().join("manifest.json");
        let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&manifest).unwrap()).unwrap();
        value["packages"][1]["artworks"] = json!(2);
        fs::write(&manifest, value.to_string()).unwrap();
        assert_eq!(preflight(&conn, export.path()).unwrap_err(), "batch_manifest");
        fixture(export.path(), &["BATCH-1", "BATCH-2"]);
        let plan = preflight(&conn, export.path()).unwrap();
        let second = review(&conn, export.path(), &plan.manifest_digest, 1).unwrap();
        commit(&mut conn, archive.path(), export.path(), &plan.manifest_digest, 1, &second.digest).unwrap();
        assert!(preflight(&conn, export.path()).is_err());
        assert_eq!(count(&conn), 1);
    }

    #[test]
    fn explicit_remaining_folder_keeps_committed_rows_and_rejects_overlap() {
        let export = tempfile::tempdir().unwrap();
        let remaining = tempfile::tempdir().unwrap();
        let archive = tempfile::tempdir().unwrap();
        fixture(export.path(), &["BATCH-1", "BATCH-2", "BATCH-3"]);
        let db = crate::db::open(archive.path().to_path_buf()).unwrap();
        let mut conn = db.conn.lock().unwrap();
        let original = preflight(&conn, export.path()).unwrap();
        let first = review(&conn, export.path(), &original.manifest_digest, 0).unwrap();
        commit(&mut conn, archive.path(), export.path(), &original.manifest_digest, 0, &first.digest).unwrap();
        assert!(preflight(&conn, export.path()).is_err());
        fixture(remaining.path(), &["BATCH-2", "BATCH-3"]);
        assert_eq!(fs::read(remaining.path().join("package-0001.json")).unwrap(),
                   fs::read(export.path().join("package-0002.json")).unwrap());
        let plan = preflight(&conn, remaining.path()).unwrap();
        assert_eq!(count(&conn), 1);
        assert_eq!(plan.artworks, 2);
        for index in 0..2 {
            let candidate = review(&conn, remaining.path(), &plan.manifest_digest, index).unwrap();
            commit(&mut conn, archive.path(), remaining.path(), &plan.manifest_digest, index, &candidate.digest).unwrap();
        }
        assert_eq!(count(&conn), 3);
        assert!(preflight(&conn, remaining.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_package_even_when_target_has_identical_bytes() {
        let export = tempfile::tempdir().unwrap();
        let archive = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let db = crate::db::open(archive.path().to_path_buf()).unwrap();
        let conn = db.conn.lock().unwrap();
        fixture(export.path(), &["BATCH-1"]);
        let file = export.path().join("package-0001.json");
        let target = outside.path().join("same.json");
        fs::copy(&file, &target).unwrap();
        fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink(target, file).unwrap();
        assert!(preflight(&conn, export.path()).is_err());
        assert_eq!(count(&conn), 0);
    }
}
