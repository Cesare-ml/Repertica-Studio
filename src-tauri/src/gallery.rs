//! Additional photographs live in the local media directory; the original primary image remains unchanged.
use crate::db::Database;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;
use std::{fs, path::{Component, Path}};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GalleryImage {
    id: String, artwork_id: String, image_path: String,
    caption: Option<String>, created_at: String,
}
fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn row(r: &Row<'_>) -> rusqlite::Result<GalleryImage> {
    Ok(GalleryImage { id: r.get(0)?, artwork_id: r.get(1)?, image_path: r.get(2)?,
        caption: r.get(3)?, created_at: r.get(4)? })
}
fn relative_media_path(path: &str) -> bool {
    let mut components = Path::new(path).components();
    matches!(components.next(), Some(Component::Normal(first)) if first == "media")
        && matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none() && !path.contains('\\')
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
    conn.query_row("SELECT id,artwork_id,image_path,caption,created_at FROM artwork_images WHERE id=?1",
        params![image_id], row).map_err(err)
}
fn list_core(db: &Database, artwork_id: &str) -> Result<Vec<GalleryImage>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let mut query = conn.prepare("SELECT id,artwork_id,image_path,caption,created_at FROM artwork_images
        WHERE artwork_id=?1 ORDER BY created_at,id").map_err(err)?;
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
    if bytes.is_empty() || bytes.len() > 12 * 1024 * 1024 { return Err("Immagine vuota o troppo grande".into()); }
    let (ext, _) = mime(&bytes).ok_or("Sono consentite solo immagini JPEG, PNG o WebP")?;
    let caption = normalize_caption(caption)?;
    let conn = db.conn.lock().map_err(err)?;
    let count: i64 = conn.query_row("SELECT count(*) FROM artwork_images WHERE artwork_id=?1", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if count >= 20 { return Err("Massimo 20 fotografie aggiuntive per opera".into()); }
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let id = Uuid::new_v4().to_string();
    let relative = format!("media/{}.{}", Uuid::new_v4(), ext);
    let target = db.directory.join(&relative);
    let temp = db.directory.join(format!("{relative}.tmp"));
    fs::write(&temp, &bytes).map_err(err)?;
    if let Err(e) = fs::rename(&temp, &target) { let _ = fs::remove_file(&temp); return Err(err(e)); }
    let stamp = Utc::now().to_rfc3339();
    if let Err(e) = conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
        VALUES(?1,?2,?3,?4,?5)", params![id, artwork_id, relative, caption, stamp]) {
        let _ = fs::remove_file(&target); return Err(err(e));
    }
    conn.query_row("SELECT id,artwork_id,image_path,caption,created_at FROM artwork_images WHERE id=?1",
        params![id], row).map_err(err)
}
#[tauri::command]
pub fn update_artwork_gallery_caption(db: State<'_, Database>, image_id: String, caption: Option<String>) -> Result<GalleryImage, String> {
    update_caption_core(&db, &image_id, caption)
}

/** Promote a gallery photo without copying or deleting media. If a cover exists,
 * it becomes the gallery image in the same transaction. Certificates may contain
 * historical media paths, so both original files must remain untouched. */
fn promote_core(db: &Database, artwork_id: &str, image_id: &str) -> Result<(), String> {
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

    tx.execute("UPDATE artworks SET image_path=?2,updated_at=?3 WHERE id=?1",
        params![artwork_id, new_primary, Utc::now().to_rfc3339()]).map_err(err)?;
    if let Some(old_path) = old_primary {
        // The old cover has no caption. Do not retain the promoted photo's
        // potentially misleading caption on a different image.
        tx.execute("UPDATE artwork_images SET image_path=?2,caption=NULL WHERE id=?1",
            params![image_id, old_path]).map_err(err)?;
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
#[tauri::command]
pub fn artwork_gallery_image_data(db: State<'_, Database>, image_id: String) -> Result<String, String> {
    let conn = db.conn.lock().map_err(err)?;
    let relative: String = conn.query_row("SELECT image_path FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0)).optional().map_err(err)?
        .ok_or("Fotografia non trovata")?;
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    let path = db.directory.join(relative);
    let typ = fs::symlink_metadata(&path).map_err(err)?.file_type();
    if !typ.is_file() || typ.is_symlink() { return Err("File media non valido".into()); }
    let bytes = fs::read(path).map_err(err)?;
    let (_, media_type) = mime(&bytes).ok_or("Formato file non valido")?;
    Ok(format!("data:{media_type};base64,{}", STANDARD.encode(bytes)))
}
#[tauri::command]
pub fn delete_artwork_gallery_image(db: State<'_, Database>, image_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let relative: String = conn.query_row("SELECT image_path FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0)).optional().map_err(err)?.ok_or("Fotografia non trovata")?;
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    conn.execute("DELETE FROM artwork_images WHERE id=?1", params![image_id]).map_err(err)?;
    // A failed unlink leaves only an unreferenced media file, never a broken database link.
    let _ = fs::remove_file(db.directory.join(relative));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        fs::write(db.directory.join("media/cover.png"), b"old cover").unwrap();
        fs::write(db.directory.join("media/side.png"), b"new cover").unwrap();
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
        fs::write(db.directory.join("media/first.jpg"), b"photo").unwrap();
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
