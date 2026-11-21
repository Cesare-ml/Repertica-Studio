//! Read-only, offline diagnostic of artwork, collector and RAW media in the active archive.
//! Images are checked for reference safety, presence and file headers; RAW attachments
//! additionally have stored size and SHA-256 verified with bounded streaming reads.
//! Without per-image hashes the diagnostic cannot prove image contents are unchanged.
use crate::db::Database;
use rusqlite::Row;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::{Component, Path}};
use tauri::State;

const MAX_VISIBLE_ISSUES: usize = 100;
const MAX_IMAGE_BYTES: u64 = 12 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaIssue {
    artwork_id: String,
    image_id: Option<String>,
    role: String,
    path: String,
    problem: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAudit {
    checked: usize,
    healthy: usize,
    problems: usize,
    truncated: bool,
    issues: Vec<MediaIssue>,
}

struct MediaReference {
    artwork_id: String,
    image_id: Option<String>,
    role: String,
    path: String,
}

struct RawReference {
    artwork_id: String,
    path: String,
    size_bytes: i64,
    sha256: String,
}

fn error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

fn valid_relative(path: &str) -> bool {
    // Only flat, locally generated media paths are accepted; never follow
    // arbitrary filesystem paths coming from a modified database.
    if path.contains('\\') { return false; }
    let mut parts = Path::new(path).components();
    if !matches!(parts.next(), Some(Component::Normal(root)) if root == "media") {
        return false;
    }
    let Some(Component::Normal(filename)) = parts.next() else { return false; };
    if parts.next().is_some() { return false; }
    let Some(name) = filename.to_str() else { return false; };
    !name.is_empty() && name != "." && name != ".." && name.len() <= 255
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

fn expected_format(path: &str, header: &[u8]) -> bool {
    let ext = Path::new(path).extension().and_then(|x| x.to_str())
        .unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => header.starts_with(&[0xff, 0xd8, 0xff]),
        "png" => header.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]),
        "webp" => header.len() >= 12 && &header[..4] == b"RIFF"
            && &header[8..12] == b"WEBP",
        _ => false,
    }
}

fn check_media(root: &Path, relative: &str) -> Option<&'static str> {
    if !valid_relative(relative) { return Some("unsafe_path"); }
    let full = root.join(relative);
    let metadata = match fs::symlink_metadata(&full) {
        Ok(value) => value,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some("missing"),
        Err(_) => return Some("unreadable"),
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Some("not_regular");
    }
    if metadata.len() == 0 || metadata.len() > MAX_IMAGE_BYTES {
        return Some("invalid_size");
    }
    let mut header = [0_u8; 12];
    let mut file = match fs::File::open(full) {
        Ok(file) => file,
        Err(_) => return Some("unreadable"),
    };
    let count = match file.read(&mut header) {
        Ok(count) => count,
        Err(_) => return Some("unreadable"),
    };
    if !expected_format(relative, &header[..count]) {
        return Some("invalid_header");
    }
    None
}

/// RAW files may be 1 GiB; never read them into memory. Check the recorded
/// checksum, not just a container header that could survive file corruption.
fn check_raw(root: &Path, relative: &str, size_bytes: i64, sha256: &str) -> Option<&'static str> {
    if !valid_relative(relative) || !relative.starts_with("media/raw-") {
        return Some("unsafe_path");
    }
    if size_bytes <= 0 || size_bytes as u64 > crate::raw_files::MAX_RAW_BYTES
        || sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Some("invalid_metadata");
    }
    let full = root.join(relative);
    let metadata = match fs::symlink_metadata(&full) {
        Ok(value) => value,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some("missing"),
        Err(_) => return Some("unreadable"),
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Some("not_regular");
    }
    let expected = size_bytes as u64;
    if metadata.len() != expected {
        return Some("invalid_size");
    }
    let mut file = match fs::File::open(full) {
        Ok(file) => file,
        Err(_) => return Some("unreadable"),
    };
    let mut digest = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => return Some("unreadable"),
        };
        if count == 0 { break; }
        total = match total.checked_add(count as u64) {
            Some(value) if value <= expected => value,
            _ => return Some("invalid_size"),
        };
        digest.update(&buffer[..count]);
    }
    if total != expected {
        return Some("invalid_size");
    }
    let actual = format!("{:x}", digest.finalize());
    if !actual.eq_ignore_ascii_case(sha256) {
        return Some("hash_mismatch");
    }
    None
}

fn reference(row: &Row<'_>) -> rusqlite::Result<MediaReference> {
    Ok(MediaReference {
        artwork_id: row.get(0)?,
        image_id: row.get(1)?,
        role: row.get(2)?,
        path: row.get(3)?,
    })
}

fn audit_core(db: &Database) -> Result<MediaAudit, String> {
    // Keep the application database mutex while checking files so a normal
    // gallery edit cannot race with the scan. The scan never modifies data.
    let conn = db.conn.lock().map_err(error)?;
    let media_dir = db.directory.join("media");
    let media_type = fs::symlink_metadata(&media_dir).map_err(error)?.file_type();
    if !media_type.is_dir() || media_type.is_symlink() {
        return Err("La cartella media non è una directory locale regolare".into());
    }
    let mut references: Vec<MediaReference> = {
        let mut stmt = conn.prepare(
            "SELECT id, NULL, 'primary', image_path FROM artworks WHERE image_path IS NOT NULL
             UNION ALL
             SELECT artwork_id, id, 'gallery', image_path FROM artwork_images
             UNION ALL
             SELECT collector_id, id, 'collector', image_path FROM collector_images
             UNION ALL
             SELECT media_id, id, 'media', image_path FROM media_revisions"
        ).map_err(error)?;
        let rows = stmt.query_map([], reference).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?;
        rows
    };

    let raw_references: Vec<RawReference> = {
        let mut stmt = conn.prepare(
            "SELECT artwork_id, media_path, size_bytes, sha256 FROM artwork_raw_files ORDER BY artwork_id"
        ).map_err(error)?;
        let rows = stmt.query_map([], |row| Ok(RawReference {
            artwork_id: row.get(0)?, path: row.get(1)?,
            size_bytes: row.get(2)?, sha256: row.get(3)?,
        })).map_err(error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(error)?
    };

    // Invalid certificate snapshots cannot be trusted to expose a historic image.
    // Keep a separate issue for each one instead of silently skipping it.
    let mut snapshot_issues: Vec<MediaIssue> = Vec::new();

    // A certificate keeps the cover path frozen at issuance. The file may no
    // longer be referenced by the artwork or its gallery after a replacement.
    // Inspect each historical reference independently, without editing SQLite.
    {
        let mut stmt = conn.prepare(
            "SELECT artwork_id, id, snapshot_json FROM certificates ORDER BY issued_at, id"
        ).map_err(error)?;
        let snapshots = stmt.query_map([], |row| Ok((
            row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?
        ))).map_err(error)?;
        for snapshot in snapshots {
            let (artwork_id, certificate_id, json) = snapshot.map_err(error)?;
            // Missing or null image_path is legitimate for a certificate without
            // a photograph; malformed JSON, non-objects and other value types are not.
            let image_path = match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(serde_json::Value::Object(fields)) => match fields.get("image_path") {
                    None | Some(serde_json::Value::Null) => Ok(None),
                    Some(serde_json::Value::String(path)) => Ok(Some(path.clone())),
                    _ => Err(()),
                },
                _ => Err(()),
            };
            match image_path {
                Ok(Some(path)) => references.push(MediaReference {
                    artwork_id,
                    image_id: Some(certificate_id),
                    role: "certificate".into(),
                    path,
                }),
                Ok(None) => {},
                Err(()) => snapshot_issues.push(MediaIssue {
                    artwork_id,
                    image_id: Some(certificate_id),
                    role: "certificate".into(),
                    // Never expose raw or untrusted snapshot JSON in a diagnostic.
                    path: "snapshot_json".into(),
                    problem: "invalid_snapshot",
                }),
            }
        }
    }

    let mut result = MediaAudit {
        checked: 0, healthy: 0, problems: 0, truncated: false,
        issues: Vec::new(),
    };
    // Include unreadable historical snapshots in the same bounded, read-only
    // report. A broken snapshot counts as one checked item, not a healthy image.
    for issue in snapshot_issues {
        result.checked += 1;
        result.problems += 1;
        if result.issues.len() < MAX_VISIBLE_ISSUES {
            result.issues.push(issue);
        } else {
            result.truncated = true;
        }
    }
    for entry in references {
        result.checked += 1;
        if let Some(problem) = check_media(&db.directory, &entry.path) {
            result.problems += 1;
            if result.issues.len() < MAX_VISIBLE_ISSUES {
                result.issues.push(MediaIssue {
                    artwork_id: entry.artwork_id,
                    image_id: entry.image_id,
                    role: entry.role,
                    path: entry.path,
                    problem,
                });
            } else {
                result.truncated = true;
            }
        } else {
            result.healthy += 1;
        }
    }

    // SHA-256 checks on large RAWs must not monopolize the SQLite mutex.
    // They are a best-effort read-only snapshot: external file edits can race the scan.
    drop(conn);
    for raw in raw_references {
        result.checked += 1;
        if let Some(problem) = check_raw(&db.directory, &raw.path, raw.size_bytes, &raw.sha256) {
            result.problems += 1;
            if result.issues.len() < MAX_VISIBLE_ISSUES {
                result.issues.push(MediaIssue {
                    artwork_id: raw.artwork_id, image_id: None,
                    role: "raw".into(), path: raw.path, problem,
                });
            } else {
                result.truncated = true;
            }
        } else {
            result.healthy += 1;
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn audit_artwork_media(db: State<'_, Database>) -> Result<MediaAudit, String> {
    audit_core(&db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn artwork(db: &Database, id: &str, image: Option<&str>) {
        db.conn.lock().unwrap().execute(
            "INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
             VALUES(?1,?1,?1,?2,'t','t')", params![id, image]
        ).unwrap();
    }

    #[test]
    fn reports_missing_corrupt_and_unsafe_files_without_changing_data() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        artwork(&db, "good", Some("media/good.png"));
        artwork(&db, "missing", Some("media/missing.jpg"));
        artwork(&db, "unsafe", Some("../outside.jpg"));
        artwork(&db, "bad", Some("media/bad.jpg"));
        fs::write(db.directory.join("media/good.png"), [137,80,78,71,13,10,26,10,0]).unwrap();
        fs::write(db.directory.join("media/bad.jpg"), b"not really jpeg").unwrap();
        fs::write(db.directory.join("media/orphan.png"), b"unreferenced").unwrap();
        db.conn.lock().unwrap().execute(
            "INSERT INTO artwork_images(id,artwork_id,image_path,created_at,sort_order)
             VALUES('extra','good','media/good-extra.webp','t',1)", []
        ).unwrap();
        fs::write(db.directory.join("media/good-extra.webp"), b"RIFF0000WEBP").unwrap();
        let scan = audit_core(&db).unwrap();
        assert_eq!((scan.checked, scan.healthy, scan.problems), (5, 2, 3));
        let problems: std::collections::HashSet<_> = scan.issues.iter().map(|i| i.problem).collect();
        assert_eq!(problems, ["missing", "unsafe_path", "invalid_header"].into_iter().collect());
        assert!(!scan.truncated);
        // A diagnostic is not allowed to move, delete or correct any image.
        assert!(db.directory.join("media/orphan.png").exists());
        assert!(db.directory.join("media/bad.jpg").exists());
        assert_eq!(db.conn.lock().unwrap().query_row(
            "SELECT count(*) FROM artwork_images", [], |r| r.get::<_, i64>(0)
        ).unwrap(), 1);
    }

    #[test]
    fn rejects_traversal_directories_and_wrong_headers() {
        assert!(!valid_relative("../media/pic.png"));
        assert!(!valid_relative("media/../pic.png"));
        assert!(!valid_relative("media/nested/pic.jpg"));
        assert!(!valid_relative("media\\pic.jpg"));
        assert!(valid_relative("media/pic.jpg"));
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("media")).unwrap();
        fs::create_dir(dir.path().join("media/dir.jpg")).unwrap();
        assert_eq!(check_media(dir.path(), "media/dir.jpg"), Some("not_regular"));
        fs::write(dir.path().join("media/bad.png"), b"not png").unwrap();
        assert_eq!(check_media(dir.path(), "media/bad.png"), Some("invalid_header"));
    }

    #[cfg(unix)]
    #[test]
    fn does_not_follow_media_symlinks() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("media")).unwrap();
        fs::write(dir.path().join("outside.jpg"), [0xff,0xd8,0xff]).unwrap();
        symlink(dir.path().join("outside.jpg"), dir.path().join("media/link.jpg")).unwrap();
        assert_eq!(check_media(dir.path(), "media/link.jpg"), Some("not_regular"));
    }
    #[test]
    fn historical_certificate_covers_are_audited_even_after_cover_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        artwork(&db, "a", Some("media/current.png"));
        fs::write(db.directory.join("media/current.png"),
            [137, 80, 78, 71, 13, 10, 26, 10, 0]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            for (id, serial, json) in [
                ("old", "RS-old", r#"{"image_path":"media/old.jpg"}"#),
                ("new", "RS-new", r#"{"image_path":"media/current.png"}"#),
                ("unsafe", "RS-unsafe", r#"{"image_path":"../outside.jpg"}"#),
                ("legacy", "RS-legacy", "{malformed json"),
            ] {
                conn.execute(
                    "INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                     VALUES(?1,'a',?2,'2026-11-21','Artist',?3,'t')",
                    params![id, serial, json],
                ).unwrap();
            }
        }
        let report = audit_core(&db).unwrap();
        // A malformed historical snapshot is a visible problem, never skipped.
        assert_eq!((report.checked, report.healthy, report.problems), (5, 2, 3));
        let legacy = report.issues.iter().find(|i| i.image_id.as_deref() == Some("legacy")).unwrap();
        assert_eq!(legacy.problem, "invalid_snapshot");
        assert_eq!(legacy.role, "certificate");
        assert_eq!(legacy.path, "snapshot_json");
        let historical = report.issues.iter().find(|i| i.image_id.as_deref() == Some("old")).unwrap();
        assert_eq!(historical.role, "certificate");
        assert_eq!(historical.artwork_id, "a");
        assert_eq!(historical.path, "media/old.jpg");
        assert_eq!(historical.problem, "missing");
        assert_eq!(report.issues.iter().find(|i| i.image_id.as_deref() == Some("unsafe"))
            .unwrap().problem, "unsafe_path");

        fs::write(db.directory.join("media/old.jpg"), [0xff, 0xd8, 0xff, 0]).unwrap();
        let repaired = audit_core(&db).unwrap();
        assert_eq!((repaired.checked, repaired.healthy, repaired.problems), (5, 3, 2));
        // Auditing must leave historical certificate records untouched.
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM certificates", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn malformed_certificate_snapshots_are_reported_without_exposing_or_changing_json() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        artwork(&db, "a", None);
        let snapshots = [
            ("syntax", "{not valid"),
            ("array", "[]"),
            ("bad_path_type", r#"{"image_path":42}"#),
            ("no_picture", r#"{"image_path":null}"#),
            ("legacy_without_path", "{}"),
        ];
        {
            let conn = db.conn.lock().unwrap();
            for (id, json) in snapshots {
                conn.execute(
                    "INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                     VALUES(?1,'a',?1,'2026-11-21','Artist',?2,'t')",
                    params![id, json],
                ).unwrap();
            }
        }

        let report = audit_core(&db).unwrap();
        assert_eq!((report.checked, report.healthy, report.problems), (3, 0, 3));
        assert!(!report.truncated);
        for id in ["syntax", "array", "bad_path_type"] {
            let issue = report.issues.iter().find(|i| i.image_id.as_deref() == Some(id)).unwrap();
            assert_eq!((issue.problem, issue.path.as_str(), issue.role.as_str()),
                ("invalid_snapshot", "snapshot_json", "certificate"));
        }
        let conn = db.conn.lock().unwrap();
        for (id, original) in snapshots {
            let saved: String = conn.query_row(
                "SELECT snapshot_json FROM certificates WHERE id = ?1",
                [id], |r| r.get(0)
            ).unwrap();
            assert_eq!(saved, original);
        }
    }

    #[test]
    fn audits_raw_size_and_sha256_and_preserves_original_records() {
        let (_dir, db) = {
            let dir = tempfile::tempdir().unwrap();
            let db = crate::db::open(dir.path().join("app")).unwrap();
            (dir, db)
        };
        let original = b"II*\0RAW example";
        let good_hash = format!("{:x}", Sha256::digest(original));
        for id in ["good", "changed", "missing", "unsafe", "badmeta"] {
            artwork(&db, id, None);
        }
        let mut changed = original.to_vec();
        changed[4] ^= 1; // Same size, still has its RAW signature.
        fs::write(db.directory.join("media/raw-good.dng"), original).unwrap();
        fs::write(db.directory.join("media/raw-changed.dng"), &changed).unwrap();
        fs::write(db.directory.join("media/raw-badmeta.dng"), original).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            for (id, path, digest) in [
                ("good", "media/raw-good.dng", good_hash.as_str()),
                ("changed", "media/raw-changed.dng", good_hash.as_str()),
                ("missing", "media/raw-missing.dng", good_hash.as_str()),
                ("unsafe", "../outside.dng", good_hash.as_str()),
                ("badmeta", "media/raw-badmeta.dng", "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
            ] {
                conn.execute(
                    "INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at)
                     VALUES(?1,?2,'photo.dng',?3,?4,'t')",
                    params![id, path, original.len() as i64, digest],
                ).unwrap();
            }
        }

        let audit = audit_core(&db).unwrap();
        assert_eq!((audit.checked, audit.healthy, audit.problems), (5, 1, 4));
        assert!(!audit.truncated);
        for (id, problem) in [
            ("changed", "hash_mismatch"), ("missing", "missing"),
            ("unsafe", "unsafe_path"), ("badmeta", "invalid_metadata"),
        ] {
            let issue = audit.issues.iter().find(|item| item.artwork_id == id).unwrap();
            assert_eq!((issue.role.as_str(), issue.image_id.as_deref(), issue.problem),
                ("raw", None, problem));
        }
        let conn = db.conn.lock().unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM artwork_raw_files", [], |r| r.get::<_, i64>(0)).unwrap(), 5);
        assert_eq!(fs::read(db.directory.join("media/raw-changed.dng")).unwrap(), changed);
    }

    #[cfg(unix)]
    #[test]
    fn raw_audit_rejects_symlinks_without_following_them() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        let outside = dir.path().join("external.dng");
        fs::write(&outside, b"II*\0outside").unwrap();
        let path = db.directory.join("media/raw-link.dng");
        symlink(&outside, &path).unwrap();
        assert_eq!(
            check_raw(&db.directory, "media/raw-link.dng", 11, &"0".repeat(64)),
            Some("not_regular")
        );
        assert_eq!(fs::read(&outside).unwrap(), b"II*\0outside");
    }

}
