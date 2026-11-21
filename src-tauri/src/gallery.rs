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
    caption: Option<String>, created_at: String, sort_order: i64,
}
const MAX_GALLERY_PREVIEW_BYTES: u64 = 12 * 1024 * 1024;
fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn row(r: &Row<'_>) -> rusqlite::Result<GalleryImage> {
    Ok(GalleryImage { id: r.get(0)?, artwork_id: r.get(1)?, image_path: r.get(2)?,
        caption: r.get(3)?, created_at: r.get(4)?, sort_order: r.get(5)? })
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
    conn.query_row("SELECT id,artwork_id,image_path,caption,created_at,sort_order FROM artwork_images WHERE id=?1",
        params![image_id], row).map_err(err)
}
fn list_core(db: &Database, artwork_id: &str) -> Result<Vec<GalleryImage>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let mut query = conn.prepare("SELECT id,artwork_id,image_path,caption,created_at,sort_order FROM artwork_images
        WHERE artwork_id=?1 ORDER BY sort_order,id").map_err(err)?;
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
    if let Err(e) = conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order)
        VALUES(?1,?2,?3,?4,?5,
            (SELECT COALESCE(MAX(sort_order),0)+1 FROM artwork_images WHERE artwork_id=?2))", params![id, artwork_id, relative, caption, stamp]) {
        let _ = fs::remove_file(&target); return Err(err(e));
    }
    conn.query_row("SELECT id,artwork_id,image_path,caption,created_at,sort_order FROM artwork_images WHERE id=?1",
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
/// Reorder one photo by one position, keeping media paths and all historical metadata.
fn move_core(db: &Database, artwork_id: &str, image_id: &str, direction: &str) -> Result<(), String> {
    if direction != "up" && direction != "down" { return Err("Direzione non valida".into()); }
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let position: i64 = tx.query_row(
        "SELECT sort_order FROM artwork_images WHERE id=?1 AND artwork_id=?2",
        params![image_id,artwork_id], |r|r.get(0),
    ).optional().map_err(err)?.ok_or("Fotografia non trovata per questa opera")?;
    let neighbor: Option<(String,i64)> = if direction == "up" {
        tx.query_row("SELECT id,sort_order FROM artwork_images
            WHERE artwork_id=?1 AND (sort_order<?2 OR (sort_order=?2 AND id<?3))
            ORDER BY sort_order DESC,id DESC LIMIT 1",
            params![artwork_id,position,image_id], |r|Ok((r.get(0)?,r.get(1)?)))
            .optional().map_err(err)?
    } else {
        tx.query_row("SELECT id,sort_order FROM artwork_images
            WHERE artwork_id=?1 AND (sort_order>?2 OR (sort_order=?2 AND id>?3))
            ORDER BY sort_order,id LIMIT 1",
            params![artwork_id,position,image_id], |r|Ok((r.get(0)?,r.get(1)?)))
            .optional().map_err(err)?
    };
    if let Some((neighbor_id,neighbor_position)) = neighbor {
        let changed = tx.execute("UPDATE artwork_images SET sort_order=CASE
                WHEN id=?1 THEN ?3 WHEN id=?2 THEN ?4 END
            WHERE artwork_id=?5 AND id IN (?1,?2)",
            params![image_id,neighbor_id,neighbor_position,position,artwork_id]).map_err(err)?;
        if changed != 2 { return Err("Scambio fotografie incompleto".into()); }
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
fn gallery_image_data_core(db: &Database, image_id: &str) -> Result<String, String> {
    let conn = db.conn.lock().map_err(err)?;
    let relative: String = conn.query_row("SELECT image_path FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0)).optional().map_err(err)?
        .ok_or("Fotografia non trovata")?;
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    let path = db.directory.join(relative);
    let typ = fs::symlink_metadata(&path).map_err(err)?.file_type();
    if !typ.is_file() || typ.is_symlink() { return Err("File media non valido".into()); }
    let file = fs::File::open(&path).map_err(err)?;
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
#[tauri::command]
pub fn artwork_gallery_image_data(db: State<'_, Database>, image_id: String) -> Result<String, String> {
    gallery_image_data_core(&db, &image_id)
}
fn delete_gallery_image_core(db: &Database, image_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let (artwork_id, relative): (String, String) = conn.query_row(
        "SELECT artwork_id,image_path FROM artwork_images WHERE id=?1",
        params![image_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional().map_err(err)?.ok_or("Fotografia non trovata")?;
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    // A previous cover can be in the gallery after promotion. Keep the bytes
    // if any immutable certificate snapshot still refers to that historic cover.
    let preserve = crate::commands::image_used_by_certificate(&conn, &artwork_id, &relative)?;
    conn.execute("DELETE FROM artwork_images WHERE id=?1", params![image_id]).map_err(err)?;
    if !preserve {
        // A failed unlink leaves an unreferenced file, never a broken DB link.
        let _ = fs::remove_file(db.directory.join(relative));
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
