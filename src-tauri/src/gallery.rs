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
    let caption = caption.map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    if caption.as_ref().is_some_and(|v| v.chars().count() > 300) { return Err("Didascalia troppo lunga (300 caratteri)".into()); }
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
}
