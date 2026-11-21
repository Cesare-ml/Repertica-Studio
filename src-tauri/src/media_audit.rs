//! Read-only, offline diagnostic of artwork media in the active archive.
//! This checks reference safety, presence and file headers; without stored
//! per-image hashes it cannot prove that image contents are unchanged.
use crate::db::Database;
use rusqlite::Row;
use serde::Serialize;
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

fn error(e: impl std::fmt::Display) -> String { e.to_string() }

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
    let references: Vec<MediaReference> = {
        let mut stmt = conn.prepare(
            "SELECT id, NULL, 'primary', image_path FROM artworks WHERE image_path IS NOT NULL
             UNION ALL
             SELECT artwork_id, id, 'gallery', image_path FROM artwork_images"
        ).map_err(error)?;
        stmt.query_map([], reference).map_err(error)?
            .collect::<Result<Vec<_>, _>>().map_err(error)?
    };

    let mut result = MediaAudit {
        checked: 0, healthy: 0, problems: 0, truncated: false,
        issues: Vec::new(),
    };
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
}
