//! Additional photographs live in the local media directory; the original primary image remains unchanged.
use crate::db::Database;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;
use std::{fs, io::Read, path::{Component, Path}};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GalleryImage {
    id: String, artwork_id: String, image_path: String,
    caption: Option<String>, created_at: String, sort_order: i64, kind: String,
}
/// What an additional photograph shows. Stored as a stable code, translated in the interface.
pub(crate) const PHOTO_KINDS: [&str; 7] = ["DETAIL", "SIGNATURE", "DIGITAL_MOCKUP", "IN_SITU", "OTHER", "FRAMED", "WORK_IN_PROGRESS"];
/// Missing or empty means "other"; anything unknown is rejected.
pub(crate) fn normalize_kind(kind: Option<&str>) -> Result<String, String> {
    match kind.map(str::trim).filter(|k| !k.is_empty()) {
        None => Ok("OTHER".into()),
        Some(k) if PHOTO_KINDS.contains(&k) => Ok(k.to_string()),
        Some(_) => Err("Tipo di fotografia non valido".into()),
    }
}
const GALLERY_COLUMNS: &str = "id,artwork_id,image_path,caption,created_at,sort_order,kind";
const MAX_GALLERY_PREVIEW_BYTES: u64 = 12 * 1024 * 1024;
fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn row(r: &Row<'_>) -> rusqlite::Result<GalleryImage> {
    Ok(GalleryImage { id: r.get(0)?, artwork_id: r.get(1)?, image_path: r.get(2)?,
        caption: r.get(3)?, created_at: r.get(4)?, sort_order: r.get(5)?, kind: r.get(6)? })
}
fn relative_media_path(path: &str) -> bool {
    let mut components = Path::new(path).components();
    matches!(components.next(), Some(Component::Normal(first)) if first == "media")
        && matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none() && !path.contains('\\')
}
// Never let an externally replaced media/ symlink redirect reads, imports,
// promotions or deletions outside the private application directory.
fn require_regular_media_dir(db: &Database) -> Result<(), String> {
    let file_type = fs::symlink_metadata(db.directory.join("media")).map_err(err)?.file_type();
    if !file_type.is_dir() || file_type.is_symlink() {
        return Err("La cartella media non è una directory locale regolare".into());
    }
    Ok(())
}

fn mime(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) { Some(("jpg", "image/jpeg")) }
    else if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) { Some(("png", "image/png")) }
    else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(("webp", "image/webp"))
    } else { None }
}
fn normalize_caption(caption: Option<String>) -> Result<Option<String>, String> {
    let caption = caption.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty());
    if caption.as_ref().is_some_and(|value| value.chars().count() > 300) {
        return Err("Didascalia troppo lunga (300 caratteri)".into());
    }
    Ok(caption)
}
fn update_caption_core(db: &Database, image_id: &str, caption: Option<String>) -> Result<GalleryImage, String> {
    let caption = normalize_caption(caption)?;
    let conn = db.conn.lock().map_err(err)?;
    let updated = conn.execute("UPDATE artwork_images SET caption=?2 WHERE id=?1",
        params![image_id, caption]).map_err(err)?;
    if updated == 0 { return Err("Fotografia non trovata".into()); }
    conn.query_row(&format!("SELECT {GALLERY_COLUMNS} FROM artwork_images WHERE id=?1"),
        params![image_id], row).map_err(err)
}
fn list_core(db: &Database, artwork_id: &str) -> Result<Vec<GalleryImage>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let mut query = conn.prepare(&format!("SELECT {GALLERY_COLUMNS} FROM artwork_images
        WHERE artwork_id=?1 ORDER BY sort_order,id")).map_err(err)?;
    let result = query.query_map(params![artwork_id], row).map_err(err)?
        .collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(result)
}
#[tauri::command]
pub fn list_artwork_gallery(db: State<'_, Database>, artwork_id: String) -> Result<Vec<GalleryImage>, String> {
    list_core(&db, &artwork_id)
}
#[tauri::command]
pub fn add_artwork_gallery_image(db: State<'_, Database>, artwork_id: String, data_base64: String, caption: Option<String>) -> Result<GalleryImage, String> {
    if data_base64.len() > 16_800_000 { return Err("Immagine troppo grande (massimo 12 MiB)".into()); }
    let bytes = STANDARD.decode(&data_base64).map_err(|_| "Immagine non valida")?;
    add_gallery_bytes(&db, &artwork_id, &bytes, caption)
}
#[tauri::command]
pub fn add_artwork_gallery_image_from_path(db: State<'_, Database>, artwork_id: String, source_path: String, caption: Option<String>) -> Result<GalleryImage, String> {
    let bytes = crate::commands::read_image_file(&source_path)?;
    add_gallery_bytes(&db, &artwork_id, &bytes, caption)
}
fn add_gallery_bytes(db: &Database, artwork_id: &str, bytes: &[u8], caption: Option<String>) -> Result<GalleryImage, String> {
    if bytes.is_empty() || bytes.len() > crate::commands::MAX_IMAGE_BYTES { return Err("Immagine vuota o troppo grande".into()); }
    let (ext, _) = mime(bytes).ok_or("Sono consentite solo immagini JPEG, PNG o WebP")?;
    let caption = normalize_caption(caption)?;
    require_regular_media_dir(db)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let count: i64 = conn.query_row("SELECT count(*) FROM artwork_images WHERE artwork_id=?1", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if count >= 20 { return Err("Massimo 20 fotografie aggiuntive per opera".into()); }
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let id = Uuid::new_v4().to_string();
    let relative = format!("media/{}.{}", Uuid::new_v4(), ext);
    let target = db.directory.join(&relative);
    let temp = db.directory.join(format!("{relative}.tmp"));
    fs::write(&temp, bytes).map_err(err)?;
    if let Err(e) = fs::rename(&temp, &target) { let _ = fs::remove_file(&temp); return Err(err(e)); }
    let stamp = Utc::now().to_rfc3339();
    // Imported archives may contain duplicate positions or i64::MAX. SQLite
    // promotes MAX(sort_order)+1 to REAL at that limit, which cannot be read
    // back as an integer. Compact just this artwork's visible order before
    // appending, and commit all position updates and the new row together.
    let inserted = (|| -> Result<GalleryImage, String> {
        let tx = conn.transaction().map_err(err)?;
        let ordered_ids: Vec<String> = {
            let mut query = tx.prepare(
                "SELECT id FROM artwork_images WHERE artwork_id=?1 ORDER BY sort_order,id"
            ).map_err(err)?;
            let rows = query.query_map(params![artwork_id], |r| r.get::<_, String>(0)).map_err(err)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(err)?
        };
        for (index, existing_id) in ordered_ids.iter().enumerate() {
            let position = i64::try_from(index + 1).map_err(|_| "Troppe fotografie per l'ordinamento")?;
            tx.execute(
                "UPDATE artwork_images SET sort_order=?3 WHERE id=?1 AND artwork_id=?2",
                params![existing_id, artwork_id, position],
            ).map_err(err)?;
        }
        let next_position = i64::try_from(ordered_ids.len() + 1)
            .map_err(|_| "Troppe fotografie per l'ordinamento")?;
        tx.execute(
            "INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![id, artwork_id, relative, caption, stamp, next_position],
        ).map_err(err)?;
        let image = tx.query_row(&format!("SELECT {GALLERY_COLUMNS} FROM artwork_images WHERE id=?1"),
            params![id], row).map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(image)
    })();
    // A failed SQLite transaction must not strand an imported photograph on
    // disk; rollback restores any positions that were being normalized.
    if inserted.is_err() { let _ = fs::remove_file(&target); }
    inserted
}
fn update_kind_core(db: &Database, image_id: &str, kind: &str) -> Result<GalleryImage, String> {
    let kind = normalize_kind(Some(kind))?;
    let conn = db.conn.lock().map_err(err)?;
    let updated = conn.execute("UPDATE artwork_images SET kind=?2 WHERE id=?1", params![image_id, kind]).map_err(err)?;
    if updated == 0 { return Err("Fotografia non trovata".into()); }
    conn.query_row(&format!("SELECT {GALLERY_COLUMNS} FROM artwork_images WHERE id=?1"), params![image_id], row).map_err(err)
}
#[tauri::command]
pub fn update_artwork_gallery_kind(db: State<'_, Database>, image_id: String, kind: String) -> Result<GalleryImage, String> {
    update_kind_core(&db, &image_id, &kind)
}
#[tauri::command]
pub fn update_artwork_gallery_caption(db: State<'_, Database>, image_id: String, caption: Option<String>) -> Result<GalleryImage, String> {
    update_caption_core(&db, &image_id, caption)
}

/** Promote a gallery photo without copying or deleting media. If a cover exists,
 * it becomes the gallery image in the same transaction. Certificates may contain
 * historical media paths, so both original files must remain untouched. */
fn promote_core(db: &Database, artwork_id: &str, image_id: &str) -> Result<(), String> {
    require_regular_media_dir(db)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let new_primary: String = tx.query_row(
        "SELECT image_path FROM artwork_images WHERE id=?1 AND artwork_id=?2",
        params![image_id, artwork_id], |r| r.get(0),
    ).optional().map_err(err)?.ok_or("Fotografia non trovata per questa opera")?;
    let old_primary: Option<String> = tx.query_row(
        "SELECT image_path FROM artworks WHERE id=?1",
        params![artwork_id], |r| r.get(0),
    ).optional().map_err(err)?.ok_or("Opera non trovata")?;

    // Reject invalid references before changing either side. Never follow media symlinks.
    for path in std::iter::once(new_primary.as_str()).chain(old_primary.as_deref()) {
        if !relative_media_path(path) { return Err("Percorso media non valido".into()); }
        let file_type = fs::symlink_metadata(db.directory.join(path)).map_err(err)?.file_type();
        if !file_type.is_file() || file_type.is_symlink() {
            return Err("File media non valido".into());
        }
    }
    if old_primary.as_deref() == Some(new_primary.as_str()) {
        return Err("Immagini principale e aggiuntiva duplicate".into());
    }
    // Imported archives and external tools can leave a regular file containing
    // anything (or a sparse, oversized file) behind a gallery reference.
    // Reject it before the transaction changes the cover or gallery metadata.
    // Only its first 12 bytes are needed to detect our supported formats.
    let candidate = fs::File::open(db.directory.join(&new_primary)).map_err(err)?;
    let size = candidate.metadata().map_err(err)?.len();
    if size == 0 || size > MAX_GALLERY_PREVIEW_BYTES {
        return Err("Fotografia da promuovere vuota o troppo grande (massimo 12 MiB)".into());
    }
    let mut signature = Vec::with_capacity(12);
    candidate.take(12).read_to_end(&mut signature).map_err(err)?;
    if mime(&signature).is_none() {
        return Err("Formato fotografia da promuovere non valido".into());
    }

    // The catalogue link travels with the picture it describes.
    let new_media: Option<String> = tx.query_row("SELECT media_id FROM artwork_images WHERE id=?1", params![image_id], |r| r.get(0)).map_err(err)?;
    let old_media: Option<String> = tx.query_row("SELECT image_media_id FROM artworks WHERE id=?1", params![artwork_id], |r| r.get(0)).map_err(err)?;
    tx.execute("UPDATE artworks SET image_path=?2,image_media_id=?4,updated_at=?3 WHERE id=?1",
        params![artwork_id, new_primary, Utc::now().to_rfc3339(), new_media]).map_err(err)?;
    if let Some(old_path) = old_primary {
        // The old cover has no caption. Do not retain the promoted photo's
        // potentially misleading caption on a different image.
        tx.execute("UPDATE artwork_images SET image_path=?2,caption=NULL,kind='OTHER',media_id=?3 WHERE id=?1",
            params![image_id, old_path, old_media]).map_err(err)?;
    } else {
        // No old cover to swap: the gallery reference becomes the primary one.
        tx.execute("DELETE FROM artwork_images WHERE id=?1", params![image_id]).map_err(err)?;
    }
    tx.commit().map_err(err)
}
#[tauri::command]
pub fn promote_artwork_gallery_image(db: State<'_, Database>, artwork_id: String, image_id: String) -> Result<(), String> {
    promote_core(&db, &artwork_id, &image_id)
}
/// Reorder one photo by one visible position, keeping its media and metadata.
/// Imported or manually edited archives may have duplicate/gapped sort_order values:
/// swapping their numeric values would leave the visible order unchanged. On a
/// successful move, assign a dense, deterministic sequence to this artwork only.
fn move_core(db: &Database, artwork_id: &str, image_id: &str, direction: &str) -> Result<(), String> {
    if direction != "up" && direction != "down" { return Err("Direzione non valida".into()); }
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let mut ordered_ids: Vec<String> = {
        let mut statement = tx.prepare(
            "SELECT id FROM artwork_images WHERE artwork_id=?1 ORDER BY sort_order,id"
        ).map_err(err)?;
        let rows = statement.query_map(params![artwork_id], |r| r.get::<_, String>(0)).map_err(err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(err)?
    };
    let index = ordered_ids.iter().position(|id| id.as_str() == image_id)
        .ok_or("Fotografia non trovata per questa opera")?;
    let neighbor = if direction == "up" {
        index.checked_sub(1)
    } else if index + 1 < ordered_ids.len() {
        Some(index + 1)
    } else {
        None
    };
    let Some(neighbor) = neighbor else {
        return tx.commit().map_err(err); // Already first/last: no-op.
    };
    ordered_ids.swap(index, neighbor);
    for (index, id) in ordered_ids.iter().enumerate() {
        let position = i64::try_from(index + 1).map_err(|_| "Troppe fotografie per il riordino")?;
        let changed = tx.execute(
            "UPDATE artwork_images SET sort_order=?3 WHERE artwork_id=?1 AND id=?2",
            params![artwork_id, id, position],
        ).map_err(err)?;
        if changed != 1 { return Err("Riordino fotografie incompleto".into()); }
    }
    tx.commit().map_err(err)
}
#[tauri::command]
pub fn move_artwork_gallery_image(
    db: State<'_, Database>, artwork_id: String, image_id: String, direction: String,
) -> Result<(), String> {
    move_core(&db, &artwork_id, &image_id, &direction)
}
// Preview bytes enter the Tauri IPC response as base64. Even if a media file
// was replaced on disk after import, never allocate an unbounded buffer for it.
// Resolve the reference under the SQLite lock, then release it before any
// potentially slow filesystem read or base64 conversion. USB and NAS media
// previews must not block edits to unrelated artworks and collectors.
fn gallery_image_data_with_reader(
    db: &Database, image_id: &str, reader: impl FnOnce(&Path) -> Result<String, String>,
) -> Result<String, String> {
    require_regular_media_dir(db)?;
    let relative: String = {
        let conn = db.conn.lock().map_err(err)?;
        conn.query_row("SELECT image_path FROM artwork_images WHERE id=?1",
            params![image_id], |r| r.get(0)).optional().map_err(err)?
            .ok_or("Fotografia non trovata")?
    };
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    reader(&db.directory.join(relative))
}

fn read_gallery_image_file(path: &Path) -> Result<String, String> {
    let typ = fs::symlink_metadata(path).map_err(err)?.file_type();
    if !typ.is_file() || typ.is_symlink() { return Err("File media non valido".into()); }
    let file = fs::File::open(path).map_err(err)?;
    let length = file.metadata().map_err(err)?.len();
    if length == 0 || length > MAX_GALLERY_PREVIEW_BYTES {
        return Err("Fotografia vuota o troppo grande (massimo 12 MiB)".into());
    }
    // Metadata is only an early rejection; cap the actual read as well in case
    // the file grows between the size check and the read.
    let mut bytes = Vec::new();
    file.take(MAX_GALLERY_PREVIEW_BYTES + 1)
        .read_to_end(&mut bytes).map_err(err)?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_GALLERY_PREVIEW_BYTES {
        return Err("Fotografia vuota o troppo grande (massimo 12 MiB)".into());
    }
    let (_, media_type) = mime(&bytes).ok_or("Formato file non valido")?;
    Ok(format!("data:{media_type};base64,{}", STANDARD.encode(bytes)))
}

fn gallery_image_data_core(db: &Database, image_id: &str) -> Result<String, String> {
    gallery_image_data_with_reader(db, image_id, read_gallery_image_file)
}
#[tauri::command]
pub fn artwork_gallery_image_data(db: State<'_, Database>, image_id: String) -> Result<String, String> {
    gallery_image_data_core(&db, &image_id)
}
// Deleting a gallery row must not remove bytes still referenced by another
// artwork's cover or by ANY historical certificate, even for another artwork.
// Corrupt snapshots are deliberately treated as a reason to keep the file.
fn gallery_media_has_other_references(
    conn: &rusqlite::Connection, image_id: &str, relative: &str,
) -> Result<bool, String> {
    let active: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT id FROM artworks WHERE image_path=?1
             UNION ALL
             SELECT id FROM artwork_images WHERE image_path=?1 AND id<>?2
         )",
        params![relative, image_id], |row| row.get(0),
    ).map_err(err)?;
    if active { return Ok(true); }

    let mut stmt = conn.prepare("SELECT snapshot_json FROM certificates").map_err(err)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(err)?;
    for row in rows {
        let snapshot = row.map_err(err)?;
        match serde_json::from_str::<serde_json::Value>(&snapshot) {
            Ok(serde_json::Value::Object(fields)) => match fields.get("image_path") {
                Some(serde_json::Value::String(path)) if path == relative => return Ok(true),
                None | Some(serde_json::Value::Null) | Some(serde_json::Value::String(_)) => {},
                _ => return Ok(true),
            },
            _ => return Ok(true),
        }
    }
    Ok(false)
}

pub(crate) fn delete_gallery_image_core(db: &Database, image_id: &str) -> Result<(), String> {
    require_regular_media_dir(db)?;
    let conn = db.conn.lock().map_err(err)?;
    let relative: String = conn.query_row(
        "SELECT image_path FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0),
    ).optional().map_err(err)?.ok_or("Fotografia non trovata")?;
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    // Preserve bytes also referenced elsewhere; do not scope this to one artwork.
    let preserve = gallery_media_has_other_references(&conn, image_id, &relative)?;
    conn.execute("DELETE FROM artwork_images WHERE id=?1", params![image_id]).map_err(err)?;
    if !preserve {
        // A failed unlink leaves an unreferenced file, never a broken DB link.
        crate::media::remove_file_unless_catalogued(&conn, &db.directory, &relative);
    }
    Ok(())
}
#[tauri::command]
pub fn delete_artwork_gallery_image(db: State<'_, Database>, image_id: String) -> Result<(), String> {
    delete_gallery_image_core(&db, &image_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reorder_is_scoped_persistent_and_preserves_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        {
            let c = db.conn.lock().unwrap();
            for id in ["a","b"] {
                c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES(?1,?1,?1,'t','t')",params![id]).unwrap();
            }
            for (id,art,pos) in [("one","a",1),("two","a",2),("three","a",3),("other","b",1)] {
                c.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order)
                    VALUES(?1,?2,?3,?4,'fixed',?5)",
                    params![id,art,format!("media/{id}.png"),id,pos]).unwrap();
            }
        }
        move_core(&db,"a","three","up").unwrap();
        assert_eq!(list_core(&db,"a").unwrap().iter().map(|i|i.id.as_str()).collect::<Vec<_>>(),
            vec!["one","three","two"]);
        move_core(&db,"a","three","down").unwrap();
        move_core(&db,"a","one","up").unwrap(); // boundary is no-op
        assert!(move_core(&db,"b","one","up").is_err());
        assert!(move_core(&db,"a","one","sideways").is_err());
        assert!(move_core(&db,"a","absent","down").is_err());
        for img in list_core(&db,"a").unwrap() {
            assert_eq!(img.caption.as_deref(),Some(img.id.as_str()));
            assert_eq!(img.image_path,format!("media/{}.png",img.id));
            assert_eq!(img.created_at,"fixed");
        }
        assert_eq!(list_core(&db,"b").unwrap()[0].id,"other");
        drop(db);
        let reopened = crate::db::open(tmp.path().join("app")).unwrap();
        assert_eq!(list_core(&reopened,"a").unwrap().iter().map(|i|i.id.as_str()).collect::<Vec<_>>(),
            vec!["one","two","three"]);
    }
    #[test]
    fn reordering_repairs_duplicate_and_extreme_positions_without_touching_other_artworks() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            for id in ["a", "b"] {
                conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES(?1,?1,?1,'created','updated')", params![id]).unwrap();
            }
            for (id, owner, position) in [
                ("first", "a", 0_i64),
                ("second", "a", 0_i64),
                ("third", "a", i64::MAX),
                ("unrelated", "b", i64::MAX),
            ] {
                conn.execute("INSERT INTO artwork_images
                    (id,artwork_id,image_path,caption,created_at,sort_order,kind)
                    VALUES(?1,?2,?3,?1,'original',?4,'DETAIL')",
                    params![id, owner, format!("media/{id}.jpg"), position]).unwrap();
            }
        }
        // The old implementation swapped two equal sort_order values, doing nothing.
        move_core(&db, "a", "second", "up").unwrap();
        assert_eq!(list_core(&db, "a").unwrap().iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
            vec!["second", "first", "third"]);
        move_core(&db, "a", "third", "up").unwrap();
        assert_eq!(list_core(&db, "a").unwrap().iter().map(|i| (i.id.as_str(), i.sort_order)).collect::<Vec<_>>(),
            vec![("second", 1), ("third", 2), ("first", 3)]);
        // Boundary moves and cross-artwork IDs must never alter these references.
        move_core(&db, "a", "second", "up").unwrap();
        assert!(move_core(&db, "a", "unrelated", "down").is_err());
        assert_eq!(list_core(&db, "b").unwrap()[0].sort_order, i64::MAX);
        for photo in list_core(&db, "a").unwrap() {
            assert_eq!(photo.image_path, format!("media/{}.jpg", photo.id));
            assert_eq!(photo.caption.as_deref(), Some(photo.id.as_str()));
            assert_eq!(photo.kind, "DETAIL");
            assert_eq!(photo.created_at, "original");
        }
        drop(db);
        let reopened = crate::db::open(dir.path().join("app")).unwrap();
        assert_eq!(list_core(&reopened, "a").unwrap().iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
            vec!["second", "third", "first"]);
    }

    #[test]
    fn appending_gallery_photo_repairs_extreme_positions_and_preserves_other_artworks() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            for id in ["a", "b"] {
                conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES(?1,?1,?1,'t','t')", params![id]).unwrap();
            }
            for (id, owner, position) in [
                ("first", "a", 0_i64), ("second", "a", 0_i64),
                ("third", "a", i64::MAX), ("other", "b", i64::MAX),
            ] {
                conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order)
                    VALUES(?1,?2,?3,?1,'original',?4)",
                    params![id, owner, format!("media/{id}.jpg"), position]).unwrap();
            }
        }
        let appended = add_gallery_bytes(&db, "a", &[0xff, 0xd8, 0xff, 0x01],
            Some("  Fotografia aggiunta  ".into())).unwrap();
        assert_eq!(appended.sort_order, 4);
        assert_eq!(appended.caption.as_deref(), Some("Fotografia aggiunta"));
        assert_eq!(fs::read(db.directory.join(&appended.image_path)).unwrap(),
            [0xff, 0xd8, 0xff, 0x01]);
        assert_eq!(list_core(&db, "a").unwrap().iter()
            .map(|photo| photo.sort_order).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        for original in list_core(&db, "a").unwrap().into_iter().take(3) {
            assert_eq!(original.image_path, format!("media/{}.jpg", original.id));
            assert_eq!(original.caption.as_deref(), Some(original.id.as_str()));
            assert_eq!(original.created_at, "original");
        }
        assert_eq!(list_core(&db, "b").unwrap()[0].sort_order, i64::MAX);
        drop(db);
        let reopened = crate::db::open(tmp.path().join("app")).unwrap();
        assert_eq!(list_core(&reopened, "a").unwrap().len(), 4);
        assert_eq!(list_core(&reopened, "a").unwrap()[3].id, appended.id);
    }

    #[test]
    fn rejected_gallery_append_rolls_back_positions_and_removes_new_media() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','A','t','t');
                INSERT INTO artwork_images(id,artwork_id,image_path,created_at,sort_order)
                VALUES('existing','a','media/existing.jpg','t',9223372036854775807);
                CREATE TRIGGER reject_new_gallery BEFORE INSERT ON artwork_images
                BEGIN SELECT RAISE(ABORT, 'simulated_insert_failure'); END;").unwrap();
        }
        assert!(add_gallery_bytes(&db, "a", &[0xff, 0xd8, 0xff, 0x01], None).is_err());
        let photos = list_core(&db, "a").unwrap();
        assert_eq!(photos.len(), 1);
        assert_eq!(photos[0].sort_order, i64::MAX);
        assert_eq!(photos[0].id, "existing");
        assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), 0);
    }

    #[test]
    fn deleting_promoted_old_cover_preserves_certified_photo() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        let historical = "media/certified.jpg";
        fs::write(db.directory.join(historical), [0xff, 0xd8, 0xff, 0x01]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','Artwork','t','t')",[]).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('former-cover','a',?1,'t')",params![historical]).unwrap();
            conn.execute("INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                VALUES('cert','a','RS-99','2026-11-21','Artist',?1,'t')",
                params![r#"{"image_path":"media/certified.jpg"}"#]).unwrap();
        }
        delete_gallery_image_core(&db, "former-cover").unwrap();
        assert!(db.directory.join(historical).is_file());
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM artwork_images WHERE id='former-cover'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn deleting_unreferenced_gallery_photo_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        let relative = "media/extra.jpg";
        fs::write(db.directory.join(relative), [0xff, 0xd8, 0xff, 0x02]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','Artwork','t','t')",[]).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('extra','a',?1,'t')",params![relative]).unwrap();
        }
        delete_gallery_image_core(&db, "extra").unwrap();
        assert!(!db.directory.join(relative).exists());
    }


    #[test]
    fn gallery_deletion_keeps_shared_covers_and_other_artworks_historical_photos() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','First','t','t')", []).unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('b','B','Second','media/shared.jpg','t','t')", []).unwrap();
            for (id, path) in [
                ("shared", "media/shared.jpg"),
                ("historic", "media/historic.jpg"),
                ("unshared", "media/unshared.jpg"),
            ] {
                conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                    VALUES(?1,'a',?2,'t')", params![id, path]).unwrap();
                fs::write(db.directory.join(path), [0xff, 0xd8, 0xff, 0x01]).unwrap();
            }
            // Historical photo of B is deliberately not referenced by a cover
            // or gallery row belonging to B.
            conn.execute("INSERT INTO certificates(
                id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                VALUES('cert-b','b','RS-B','2026-11-21','Artist',?1,'t')",
                params![r#"{"image_path":"media/historic.jpg"}"#]).unwrap();
        }
        for id in ["shared", "historic", "unshared"] {
            delete_gallery_image_core(&db, id).unwrap();
        }
        assert!(db.directory.join("media/shared.jpg").exists());
        assert!(db.directory.join("media/historic.jpg").exists());
        assert!(!db.directory.join("media/unshared.jpg").exists());
        let conn = db.conn.lock().unwrap();
        let rows: i64 = conn.query_row(
            "SELECT COUNT(*) FROM artwork_images WHERE artwork_id='a'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(rows, 0);
    }

    #[test]
    fn promotion_refuses_corrupted_or_oversized_gallery_files_without_changing_references() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        let cover = "media/cover.jpg";
        let gallery = "media/candidate.png";
        let old_bytes = [0xff, 0xd8, 0xff, 0x01];
        let png_bytes = [137, 80, 78, 71, 13, 10, 26, 10];
        fs::write(db.directory.join(cover), old_bytes).unwrap();
        fs::write(db.directory.join(gallery), b"invalid image bytes").unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('art','A','Title',?1,'created','original')", params![cover]).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
                VALUES('photo','art',?1,'Caption','created')", params![gallery]).unwrap();
        }

        let assert_unchanged = || {
            let conn = db.conn.lock().unwrap();
            let primary: String = conn.query_row("SELECT image_path FROM artworks WHERE id='art'",
                [], |r| r.get(0)).unwrap();
            let extra: String = conn.query_row("SELECT image_path FROM artwork_images WHERE id='photo'",
                [], |r| r.get(0)).unwrap();
            let timestamp: String = conn.query_row("SELECT updated_at FROM artworks WHERE id='art'",
                [], |r| r.get(0)).unwrap();
            assert_eq!((primary.as_str(), extra.as_str(), timestamp.as_str()),
                (cover, gallery, "original"));
            assert!(db.directory.join(cover).is_file());
            assert!(db.directory.join(gallery).is_file());
        };
        assert!(promote_core(&db, "art", "photo").unwrap_err().contains("Formato"));
        assert_unchanged();

        fs::File::create(db.directory.join(gallery)).unwrap()
            .set_len(MAX_GALLERY_PREVIEW_BYTES + 1).unwrap();
        assert!(promote_core(&db, "art", "photo").unwrap_err().contains("12 MiB"));
        assert_unchanged();

        fs::write(db.directory.join(gallery), png_bytes).unwrap();
        promote_core(&db, "art", "photo").unwrap();
        let conn = db.conn.lock().unwrap();
        let primary: String = conn.query_row("SELECT image_path FROM artworks WHERE id='art'",
            [], |r| r.get(0)).unwrap();
        let extra: String = conn.query_row("SELECT image_path FROM artwork_images WHERE id='photo'",
            [], |r| r.get(0)).unwrap();
        assert_eq!((primary.as_str(), extra.as_str()), (gallery, cover));
        assert_eq!(fs::read(db.directory.join(cover)).unwrap(), old_bytes);
        assert_eq!(fs::read(db.directory.join(gallery)).unwrap(), png_bytes);
    }

    #[cfg(unix)]
    #[test]
    fn swapped_media_directory_symlink_cannot_redirect_gallery_operations() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('a','A','First','media/cover.jpg','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('extra','a','media/extra.jpg','t')", []).unwrap();
        }
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("cover.jpg"), [0xff, 0xd8, 0xff]).unwrap();
        fs::write(outside.join("extra.jpg"), [0xff, 0xd8, 0xff]).unwrap();
        fs::remove_dir(db.directory.join("media")).unwrap();
        symlink(&outside, db.directory.join("media")).unwrap();

        assert!(gallery_image_data_core(&db, "extra").is_err());
        assert!(promote_core(&db, "a", "extra").is_err());
        assert!(add_gallery_bytes(&db, "a", &[0xff, 0xd8, 0xff], None).is_err());
        assert!(delete_gallery_image_core(&db, "extra").is_err());
        assert!(outside.join("cover.jpg").is_file());
        assert!(outside.join("extra.jpg").is_file());
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 2);
        let conn = db.conn.lock().unwrap();
        let row_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM artwork_images WHERE id='extra'", [], |r| r.get(0),
        ).unwrap();
        assert_eq!(row_count, 1);
    }

    #[test]
    fn gallery_preview_releases_sqlite_lock_before_media_read() {
        let temp = tempfile::tempdir().unwrap();
        let db = crate::db::open(temp.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','A','Art','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('photo','art','media/photo.png','t')", []).unwrap();
        }
        // Reader injection checks lock lifetime deterministically, without
        // timing assumptions or a slow disk. The actual reader is used below.
        let value = gallery_image_data_with_reader(&db, "photo", |path| {
            assert_eq!(path, db.directory.join("media/photo.png"));
            assert!(db.conn.try_lock().is_ok(), "preview I/O must not hold SQLite mutex");
            Ok("ready".to_owned())
        }).unwrap();
        assert_eq!(value, "ready");

        let png = [137, 80, 78, 71, 13, 10, 26, 10];
        fs::write(db.directory.join("media/photo.png"), png).unwrap();
        assert_eq!(gallery_image_data_core(&db, "photo").unwrap(),
            format!("data:image/png;base64,{}", STANDARD.encode(png)));
        assert!(gallery_image_data_with_reader(&db, "missing", |_| panic!("no read for missing photo"))
            .is_err());
    }

    #[test]
    fn gallery_preview_bounds_reads_and_does_not_modify_media_or_references() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        let file = db.directory.join("media/photo.jpg");
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','A','Title','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('photo','art','media/photo.jpg','t')", []).unwrap();
        }
        let valid = [0xff, 0xd8, 0xff, 0x01];
        fs::write(&file, valid).unwrap();
        assert_eq!(gallery_image_data_core(&db, "photo").unwrap(),
            format!("data:image/jpeg;base64,{}", STANDARD.encode(valid)));

        // A sparse file imitates external replacement without allocating 12 MiB.
        fs::File::create(&file).unwrap()
            .set_len(MAX_GALLERY_PREVIEW_BYTES + 1).unwrap();
        assert!(gallery_image_data_core(&db, "photo").unwrap_err().contains("12 MiB"));
        fs::write(&file, []).unwrap();
        assert!(gallery_image_data_core(&db, "photo").is_err());
        fs::write(&file, b"not a jpeg").unwrap();
        assert!(gallery_image_data_core(&db, "photo").unwrap_err().contains("Formato"));
        fs::remove_file(&file).unwrap();
        assert!(gallery_image_data_core(&db, "photo").is_err());
        assert!(gallery_image_data_core(&db, "unknown").is_err());

        // A failed preview must never delete the DB reference or edit a photo.
        let conn = db.conn.lock().unwrap();
        let reference: String = conn.query_row(
            "SELECT image_path FROM artwork_images WHERE id='photo'",
            [], |row| row.get(0)).unwrap();
        assert_eq!(reference, "media/photo.jpg");
    }

    #[cfg(unix)]
    #[test]
    fn gallery_preview_refuses_symlinks() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        let external = tmp.path().join("outside.jpg");
        fs::write(&external, [0xff, 0xd8, 0xff]).unwrap();
        symlink(&external, db.directory.join("media/link.jpg")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','A','Title','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('link','art','media/link.jpg','t')", []).unwrap();
        }
        assert!(gallery_image_data_core(&db, "link").unwrap_err().contains("non valido"));
        assert!(external.is_file());
    }

    #[test]
    fn validation_rejects_traversal_and_invalid_formats() {
        assert!(!relative_media_path("media/../secret"));
        assert!(!relative_media_path("media/subdir/picture.png"));
        assert!(!relative_media_path("media\\picture.png"));
        assert!(relative_media_path("media/picture.png"));
        assert!(mime(b"not a picture").is_none());
    }
    #[test]
    fn captions_can_be_changed_or_cleared_without_touching_media_or_other_artworks() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('a','A','Art','media/primary.jpg','t','t')", []).unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('b','B','Another','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
                VALUES('photo','a','media/extra.jpg','Old','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
                VALUES('other','b','media/other.jpg','Keep','t')", []).unwrap();
        }
        let updated = update_caption_core(&db, "photo", Some("  Nuova didascalia  ".into())).unwrap();
        assert_eq!(updated.caption.as_deref(), Some("Nuova didascalia"));
        assert_eq!(updated.image_path, "media/extra.jpg");
        assert_eq!(updated.created_at, "t");
        assert!(update_caption_core(&db, "photo", Some("x".repeat(301))).is_err());
        assert_eq!(normalize_caption(Some("à".repeat(300))).unwrap().unwrap().chars().count(), 300);
        assert!(update_caption_core(&db, "missing", None).is_err());
        assert_eq!(update_caption_core(&db, "photo", Some("  ".into())).unwrap().caption, None);
        let conn = db.conn.lock().unwrap();
        let untouched: String = conn.query_row(
            "SELECT caption FROM artwork_images WHERE id='other'", [], |r| r.get(0)).unwrap();
        let primary: String = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(untouched, "Keep");
        assert_eq!(primary, "media/primary.jpg");
    }
    #[test]
    fn photo_kind_is_validated_changeable_and_reset_when_a_photo_becomes_the_cover() {
        assert_eq!(normalize_kind(None).unwrap(), "OTHER");
        assert_eq!(normalize_kind(Some("  ")).unwrap(), "OTHER");
        for kind in PHOTO_KINDS { assert_eq!(normalize_kind(Some(kind)).unwrap(), kind); }
        assert!(normalize_kind(Some("detail")).is_err(), "codes are exact");
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        fs::write(db.directory.join("media/cover.png"), [137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
        fs::write(db.directory.join("media/side.png"), [137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at) VALUES('a','A','Art','media/cover.png','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at,kind) VALUES('side','a','media/side.png','t','FRAMED')", []).unwrap();
        }
        assert_eq!(list_core(&db, "a").unwrap()[0].kind, "FRAMED");
        assert_eq!(update_kind_core(&db, "side", "WORK_IN_PROGRESS").unwrap().kind, "WORK_IN_PROGRESS");
        assert!(update_kind_core(&db, "side", "boh").is_err());
        assert!(update_kind_core(&db, "missing", "DETAIL").is_err());
        assert_eq!(list_core(&db, "a").unwrap()[0].kind, "WORK_IN_PROGRESS");
        promote_core(&db, "a", "side").unwrap();
        assert_eq!(list_core(&db, "a").unwrap()[0].kind, "OTHER", "the former cover is a plain photo");
    }
    #[test]
    fn gallery_is_per_artwork_and_keeps_primary_photo() {
        let temp = tempfile::tempdir().unwrap();
        let db = crate::db::open(temp.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('art','A-01','Title','media/primary.png','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('image','art','media/extra.png','t')", []).unwrap();
        }
        assert_eq!(list_core(&db, "art").unwrap().len(), 1);
        assert!(list_core(&db, "unknown").is_err());
        let conn = db.conn.lock().unwrap();
        let primary: String = conn.query_row("SELECT image_path FROM artworks WHERE id='art'", [], |r|r.get(0)).unwrap();
        assert_eq!(primary, "media/primary.png");
        assert!(conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
            VALUES('orphan','missing','media/other.png','t')", []).is_err());
    }
    #[test]
    fn promoting_photo_swaps_cover_atomically_and_preserves_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        fs::write(db.directory.join("media/cover.png"), [137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
        fs::write(db.directory.join("media/side.png"), [137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('a','A','Art','media/cover.png','t','t')", []).unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('b','B','Other','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
                VALUES('side','a','media/side.png','Back of painting','t')", []).unwrap();
            assert!(conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('orphan','x','media/other.png','t')", []).is_err());
        }
        assert!(promote_core(&db, "b", "side").is_err());
        assert!(promote_core(&db, "a", "missing").is_err());
        promote_core(&db, "a", "side").unwrap();
        let conn = db.conn.lock().unwrap();
        let main: String = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        let (gallery, caption, created): (String, Option<String>, String) = conn.query_row(
            "SELECT image_path,caption,created_at FROM artwork_images WHERE id='side'",
            [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!(main, "media/side.png");
        assert_eq!(gallery, "media/cover.png");
        assert_eq!(caption, None);
        assert_eq!(created, "t");
        assert!(db.directory.join(&main).is_file());
        assert!(db.directory.join(&gallery).is_file());
        let other: Option<String> = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='b'", [], |r| r.get(0)).unwrap();
        assert_eq!(other, None);
    }

    #[test]
    fn first_cover_promotion_removes_only_gallery_row_not_media() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        fs::write(db.directory.join("media/first.jpg"), [0xff, 0xd8, 0xff, 0x01]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','Art','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('photo','a','media/first.jpg','t')", []).unwrap();
        }
        promote_core(&db, "a", "photo").unwrap();
        let conn = db.conn.lock().unwrap();
        let main: String = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        let remaining: i64 = conn.query_row(
            "SELECT COUNT(*) FROM artwork_images WHERE artwork_id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(main, "media/first.jpg");
        assert_eq!(remaining, 0);
        assert!(db.directory.join(&main).is_file());
    }

    #[test]
    fn promotion_rejects_missing_media_without_changing_references() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','Art','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('missing','a','media/missing.png','t')", []).unwrap();
        }
        assert!(promote_core(&db, "a", "missing").is_err());
        let conn = db.conn.lock().unwrap();
        let main: Option<String> = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(main, None);
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM artwork_images WHERE id='missing'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

}

#[cfg(test)]
pub(crate) fn delete_gallery_image_for_test(db: &Database, id: &str) { delete_gallery_image_core(db, id).unwrap(); }
