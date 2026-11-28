//! Additional photographs live in the local media directory; the original primary image remains unchanged.
use crate::db::Database;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;
use std::{fs, io::Read, path::{Component, Path}};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, PartialEq, Eq)]
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
// Internal snapshot includes the catalogue link without changing the public IPC model.
fn gallery_snapshot(conn: &rusqlite::Connection, artwork_id: &str) -> Result<Vec<(GalleryImage, Option<String>)>, String> {
    let mut statement = conn.prepare(&format!(
        "SELECT {GALLERY_COLUMNS},media_id FROM artwork_images WHERE artwork_id=?1 ORDER BY sort_order,id"
    )).map_err(err)?;
    let rows = statement.query_map(params![artwork_id], |r| Ok((row(r)?, r.get(7)?))).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
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
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let artwork_id: String = tx.query_row("SELECT artwork_id FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0)).optional().map_err(err)?.ok_or("Fotografia non trovata")?;
    let mut expected = gallery_snapshot(&tx, &artwork_id)?;
    let index = expected.iter().position(|(photo, _)| photo.id == image_id)
        .ok_or("Fotografia non trovata")?;
    let updated = tx.execute("UPDATE artwork_images SET caption=?2 WHERE id=?1",
        params![image_id, &caption]).map_err(err)?;
    if updated != 1 { return Err("Didascalia non aggiornata: verifica l'archivio SQLite".into()); }
    expected[index].0.caption = caption;
    if gallery_snapshot(&tx, &artwork_id)? != expected {
        return Err("Didascalia non salvata correttamente: verifica l'archivio SQLite".into());
    }
    let (saved, _) = expected.swap_remove(index);
    tx.commit().map_err(err)?;
    Ok(saved)
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
    add_gallery_bytes_with_stage(db, artwork_id, bytes, caption, |temp, target, bytes| {
        fs::write(temp, bytes).map_err(err)?;
        fs::rename(temp, target).map_err(err)
    })
}

// Staging can involve a slow or removable disk. Never hold the shared SQLite
// mutex during that I/O. The transaction below re-checks both the artwork and
// the gallery limit, since either may have changed while the file was written.
// The injected writer allows a deterministic lock-order regression test.
fn add_gallery_bytes_with_stage<F>(
    db: &Database, artwork_id: &str, bytes: &[u8], caption: Option<String>, stage: F,
) -> Result<GalleryImage, String>
where
    F: FnOnce(&Path, &Path, &[u8]) -> Result<(), String>,
{
    if bytes.is_empty() || bytes.len() > crate::commands::MAX_IMAGE_BYTES {
        return Err("Immagine vuota o troppo grande".into());
    }
    let (ext, _) = mime(bytes).ok_or("Sono consentite solo immagini JPEG, PNG o WebP")?;
    let caption = normalize_caption(caption)?;
    require_regular_media_dir(db)?;
    let id = Uuid::new_v4().to_string();
    let relative = format!("media/{}.{}", Uuid::new_v4(), ext);
    let target = db.directory.join(&relative);
    let temp = db.directory.join(format!("{relative}.tmp"));

    if let Err(error) = stage(&temp, &target, bytes) {
        // fs::write may leave a partial .tmp on a full/disconnected disk.
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(&target);
        return Err(error);
    }

    let inserted = (|| -> Result<GalleryImage, String> {
        let mut conn = db.conn.lock().map_err(err)?;
        let tx = conn.transaction().map_err(err)?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)",
            params![artwork_id], |r| r.get(0),
        ).map_err(err)?;
        if !exists { return Err("Opera non trovata".into()); }

        // Count, compact and verify the entire gallery in the same transaction.
        let mut expected = gallery_snapshot(&tx, artwork_id)?;
        if expected.len() >= 20 {
            return Err("Massimo 20 fotografie aggiuntive per opera".into());
        }
        for (index, (photo, _)) in expected.iter_mut().enumerate() {
            let position = i64::try_from(index + 1).map_err(|_| "Troppe fotografie per l'ordinamento")?;
            let changed = tx.execute(
                "UPDATE artwork_images SET sort_order=?3 WHERE id=?1 AND artwork_id=?2",
                params![&photo.id, artwork_id, position],
            ).map_err(err)?;
            if changed != 1 { return Err("Ordinamento fotografie incompleto".into()); }
            photo.sort_order = position;
        }
        let next_position = i64::try_from(expected.len() + 1)
            .map_err(|_| "Troppe fotografie per l'ordinamento")?;
        let stamp = Utc::now().to_rfc3339();
        let inserted = tx.execute(
            "INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![id, artwork_id, relative, caption, stamp, next_position],
        ).map_err(err)?;
        if inserted != 1 { return Err("Fotografia non inserita".into()); }
        expected.push((GalleryImage {
            id, artwork_id: artwork_id.to_owned(), image_path: relative,
            caption, created_at: stamp, sort_order: next_position, kind: "OTHER".into(),
        }, None));
        if gallery_snapshot(&tx, artwork_id)? != expected {
            return Err("Inserimento fotografia alterato da SQLite: verifica l'archivio".into());
        }
        let (image, _) = expected.pop().ok_or("Fotografia non inserita")?;
        tx.commit().map_err(err)?;
        Ok(image)
    })();
    // If the artwork disappeared, the gallery filled up or SQLite rejected
    // the transaction, remove the staged file; original data stays untouched.
    if inserted.is_err() { let _ = fs::remove_file(&target); }
    inserted
}
fn update_kind_core(db: &Database, image_id: &str, kind: &str) -> Result<GalleryImage, String> {
    let kind = normalize_kind(Some(kind))?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let artwork_id: String = tx.query_row("SELECT artwork_id FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0)).optional().map_err(err)?.ok_or("Fotografia non trovata")?;
    let mut expected = gallery_snapshot(&tx, &artwork_id)?;
    let index = expected.iter().position(|(photo, _)| photo.id == image_id)
        .ok_or("Fotografia non trovata")?;
    let updated = tx.execute("UPDATE artwork_images SET kind=?2 WHERE id=?1",
        params![image_id, &kind]).map_err(err)?;
    if updated != 1 { return Err("Tipo fotografia non aggiornato: verifica l'archivio SQLite".into()); }
    expected[index].0.kind = kind;
    if gallery_snapshot(&tx, &artwork_id)? != expected {
        return Err("Tipo fotografia non salvato correttamente: verifica l'archivio SQLite".into());
    }
    let (saved, _) = expected.swap_remove(index);
    tx.commit().map_err(err)?;
    Ok(saved)
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
    let original = tx.query_row(
        &format!("SELECT {GALLERY_COLUMNS} FROM artwork_images WHERE id=?1 AND artwork_id=?2"),
        params![image_id, artwork_id], row,
    ).optional().map_err(err)?.ok_or("Fotografia non trovata per questa opera")?;
    let new_primary = &original.image_path;
    let (old_primary, previous): (Option<String>, String) = tx.query_row(
        "SELECT image_path,updated_at FROM artworks WHERE id=?1",
        params![artwork_id], |r| Ok((r.get(0)?,r.get(1)?)),
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
    let candidate = fs::File::open(db.directory.join(new_primary)).map_err(err)?;
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
    // An imported archive can contain BEFORE triggers that silently suppress
    // a write with RAISE(IGNORE). All parts of the swap must actually change
    // one row, otherwise commit would leave the cover/gallery out of sync.
    let stamp = crate::util::next_archive_revision(&previous)?;
    let changed_cover = tx.execute("UPDATE artworks SET image_path=?2,image_media_id=?4,updated_at=?3 WHERE id=?1",
        params![artwork_id, new_primary, &stamp, &new_media]).map_err(err)?;
    if changed_cover != 1 {
        return Err("Promozione incompleta: aggiornamento copertina rifiutato da SQLite".into());
    }
    let changed_gallery = if let Some(old_path) = old_primary.as_deref() {
        // The old cover has no caption. Do not retain the promoted photo's
        // potentially misleading caption on a different image.
        tx.execute("UPDATE artwork_images SET image_path=?2,caption=NULL,kind='OTHER',media_id=?3 WHERE id=?1",
            params![image_id, old_path, &old_media]).map_err(err)?
    } else {
        // No old cover to swap: the gallery reference becomes the primary one.
        tx.execute("DELETE FROM artwork_images WHERE id=?1", params![image_id]).map_err(err)?
    };
    if changed_gallery != 1 {
        return Err("Promozione incompleta: modifica della galleria rifiutata da SQLite".into());
    }
    // AFTER triggers may rewrite a successful UPDATE or recreate a deleted row.
    // Read both sides after the final write, including nullable catalogue links.
    let cover_saved: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1 AND image_path=?2 AND image_media_id IS ?3 AND updated_at=?4)",
        params![artwork_id, new_primary, &new_media, &stamp], |r| r.get(0)).map_err(err)?;
    let gallery_saved: bool = if let Some(old_path) = old_primary.as_deref() {
        tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM artwork_images WHERE id=?1 AND artwork_id=?2 AND image_path=?3 AND media_id IS ?4 AND caption IS NULL AND kind='OTHER' AND created_at=?5 AND sort_order=?6)",
            params![image_id, artwork_id, old_path, &old_media, &original.created_at, original.sort_order],
            |r| r.get(0)).map_err(err)?
    } else {
        tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM artwork_images WHERE id=?1)",
            params![image_id], |r| r.get(0)).map_err(err)?
    };
    if !cover_saved || !gallery_saved {
        return Err("Promozione incompleta: copertina o galleria alterata da SQLite".into());
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
    // Keep the complete gallery snapshot, including nullable catalogue links.
    // A successful UPDATE alone does not prove that an imported SQLite trigger
    // preserved the requested order or the other photographs' metadata.
    let mut expected = gallery_snapshot(&tx, artwork_id)?;
    let index = expected.iter().position(|(photo, _)| photo.id == image_id)
        .ok_or("Fotografia non trovata per questa opera")?;
    let neighbor = if direction == "up" {
        index.checked_sub(1)
    } else if index + 1 < expected.len() {
        Some(index + 1)
    } else {
        None
    };
    let Some(neighbor) = neighbor else {
        return tx.commit().map_err(err); // Already first/last: no-op.
    };
    expected.swap(index, neighbor);
    for (index, (photo, _)) in expected.iter_mut().enumerate() {
        let position = i64::try_from(index + 1).map_err(|_| "Troppe fotografie per il riordino")?;
        let changed = tx.execute(
            "UPDATE artwork_images SET sort_order=?3 WHERE artwork_id=?1 AND id=?2",
            params![artwork_id, &photo.id, position],
        ).map_err(err)?;
        if changed != 1 { return Err("Riordino fotografie incompleto".into()); }
        photo.sort_order = position;
    }
    if gallery_snapshot(&tx, artwork_id)? != expected {
        return Err("Riordino fotografie alterato da SQLite: verifica l'archivio".into());
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
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let relative: String = tx.query_row(
        "SELECT image_path FROM artwork_images WHERE id=?1",
        params![image_id], |r| r.get(0),
    ).optional().map_err(err)?.ok_or("Fotografia non trovata")?;
    if !relative_media_path(&relative) { return Err("Percorso media non valido".into()); }
    // Preserve bytes also referenced elsewhere; do not scope this to one artwork.
    let preserve = gallery_media_has_other_references(&tx, image_id, &relative)?;
    // RAISE(IGNORE) may suppress a delete; an AFTER DELETE trigger in an
    // imported database can even reinsert the row while reporting one deletion.
    // Check the final state *inside* the transaction before touching media.
    let deleted = tx.execute("DELETE FROM artwork_images WHERE id=?1", params![image_id]).map_err(err)?;
    if deleted != 1 { return Err("Eliminazione fotografia non riuscita: il collegamento non è stato rimosso".into()); }
    let remains: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM artwork_images WHERE id=?1)",
        params![image_id], |r| r.get(0),
    ).map_err(err)?;
    if remains { return Err("Eliminazione fotografia non riuscita: il collegamento è ancora presente".into()); }
    tx.commit().map_err(err)?;
    if !preserve {
        // A failed unlink leaves an unreferenced file, never a broken DB link.
        // This also rechecks references created with a different image ID.
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
    fn gallery_upload_stages_without_sqlite_mutex_and_cleans_rejected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        let jpeg = [0xff, 0xd8, 0xff, 0x01];
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','ART','Artwork','t','t')", []).unwrap();
        }
        let stage = |temp: &Path, target: &Path, bytes: &[u8]| {
            assert!(db.conn.try_lock().is_ok(), "media writes must not hold the SQLite mutex");
            fs::write(temp, bytes).map_err(err)?;
            fs::rename(temp, target).map_err(err)
        };
        let saved = add_gallery_bytes_with_stage(&db, "art", &jpeg, Some("Detail".into()), stage)
            .unwrap();
        assert_eq!(saved.sort_order, 1);
        assert_eq!(saved.caption.as_deref(), Some("Detail"));
        assert_eq!(fs::read(db.directory.join(&saved.image_path)).unwrap(), jpeg);
        let before = fs::read_dir(db.directory.join("media")).unwrap().count();

        // A deleted / unknown artwork must not leave a new file behind.
        let missing = add_gallery_bytes_with_stage(&db, "missing", &jpeg, None, |temp, target, bytes| {
            assert!(db.conn.try_lock().is_ok());
            fs::write(temp, bytes).map_err(err)?;
            fs::rename(temp, target).map_err(err)
        });
        assert!(missing.unwrap_err().contains("Opera non trovata"));
        assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), before);
        assert_eq!(list_core(&db, "art").unwrap().len(), 1);

        // Even a partial write that fails must not strand .tmp files.
        let failed = add_gallery_bytes_with_stage(&db, "art", &jpeg, None, |temp, _, _| {
            fs::write(temp, b"partial").map_err(err)?;
            Err("Unità di destinazione non disponibile".into())
        });
        assert!(failed.unwrap_err().contains("destinazione"));
        assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), before);
    }

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
    fn gallery_reorder_rolls_back_ignored_or_rewritten_rows_and_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            for id in ["a", "b"] {
                conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES(?1,?1,?1,'t','t')", params![id]).unwrap();
            }
            for (id, art, pos) in [("one","a",1),("two","a",2),("three","a",3),("other","b",1)] {
                conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,kind)
                    VALUES(?1,?2,?3,?1,'fixed',?4,'DETAIL')",
                    params![id, art, format!("media/{id}.jpg"), pos]).unwrap();
                fs::write(db.directory.join(format!("media/{id}.jpg")), id.as_bytes()).unwrap();
            }
        }
        let original = list_core(&db, "a").unwrap();
        let unrelated = list_core(&db, "b").unwrap();
        for trigger in [
            "CREATE TRIGGER sabotage BEFORE UPDATE OF sort_order ON artwork_images
                WHEN NEW.id='three' BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER sabotage AFTER UPDATE OF sort_order ON artwork_images
                WHEN NEW.id='three' AND NEW.sort_order=2
                BEGIN UPDATE artwork_images SET sort_order=OLD.sort_order WHERE id=NEW.id; END;",
            "CREATE TRIGGER sabotage AFTER UPDATE OF sort_order ON artwork_images
                WHEN NEW.id='two' BEGIN UPDATE artwork_images SET caption='changed' WHERE id='one'; END;",
            "CREATE TRIGGER sabotage AFTER UPDATE OF sort_order ON artwork_images
                WHEN NEW.id='two' BEGIN DELETE FROM artwork_images WHERE id='one'; END;",
        ] {
            db.conn.lock().unwrap().execute_batch(trigger).unwrap();
            assert!(move_core(&db, "a", "three", "up").is_err(), "{trigger}");
            assert_eq!(list_core(&db, "a").unwrap(), original);
            assert_eq!(list_core(&db, "b").unwrap(), unrelated);
            for id in ["one", "two", "three", "other"] {
                assert_eq!(fs::read(db.directory.join(format!("media/{id}.jpg"))).unwrap(), id.as_bytes());
            }
            db.conn.lock().unwrap().execute_batch("DROP TRIGGER sabotage").unwrap();
        }
        move_core(&db, "a", "three", "up").unwrap();
        let mut expected = original;
        expected.swap(1, 2);
        for (index, photo) in expected.iter_mut().enumerate() {
            photo.sort_order = (index + 1) as i64;
        }
        assert_eq!(list_core(&db, "a").unwrap(), expected);
        assert_eq!(list_core(&db, "b").unwrap(), unrelated);
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
    fn gallery_append_and_metadata_preserve_catalogue_links_and_other_photos() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','A','t','t'),('b','B','B','t','t');
                INSERT INTO media_items(id,original_name,current_revision_id,created_at,updated_at)
                VALUES('m','one.jpg','r','t','t');
                INSERT INTO media_revisions(id,media_id,number,kind,image_path,width,height,size_bytes,sha256,created_at)
                VALUES('r','m',1,'ORIGINAL','media/one.jpg',1,1,3,
                    '0000000000000000000000000000000000000000000000000000000000000000','t');
                INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,kind,media_id)
                VALUES('one','a','media/one.jpg','old','t',8,'DETAIL','m'),
                    ('two','a','media/two.jpg',NULL,'t',9,'OTHER',NULL),
                    ('other','b','media/other.jpg',NULL,'t',1,'OTHER',NULL);").unwrap();
            for id in ["one", "two", "other"] {
                fs::write(db.directory.join(format!("media/{id}.jpg")), id.as_bytes()).unwrap();
            }
        }
        let snapshot = |art: &str| gallery_snapshot(&db.conn.lock().unwrap(), art).unwrap();
        let original = snapshot("a");
        let unrelated = snapshot("b");
        for (operation, trigger) in [
            ("append", "BEFORE UPDATE OF sort_order ON artwork_images WHEN NEW.id='one'
                BEGIN SELECT RAISE(IGNORE); END"),
            ("append", "AFTER UPDATE OF sort_order ON artwork_images WHEN NEW.id='one'
                BEGIN UPDATE artwork_images SET media_id=NULL WHERE id=NEW.id; END"),
            ("append", "AFTER INSERT ON artwork_images
                BEGIN UPDATE artwork_images SET caption='wrong' WHERE id=NEW.id; END"),
            ("append", "AFTER INSERT ON artwork_images
                BEGIN DELETE FROM artwork_images WHERE id='one'; END"),
            ("caption", "AFTER UPDATE OF caption ON artwork_images
                BEGIN UPDATE artwork_images SET media_id=NULL WHERE id=NEW.id; END"),
            ("caption", "AFTER UPDATE OF caption ON artwork_images WHEN NEW.id='one'
                BEGIN UPDATE artwork_images SET kind='FRAMED' WHERE id='two'; END"),
            ("kind", "AFTER UPDATE OF kind ON artwork_images
                BEGIN UPDATE artwork_images SET media_id=NULL WHERE id=NEW.id; END"),
        ] {
            db.conn.lock().unwrap().execute_batch(&format!("CREATE TRIGGER sabotage {trigger};")).unwrap();
            let result = match operation {
                "append" => add_gallery_bytes(&db, "a", &[0xff, 0xd8, 0xff, 1], Some("new".into())),
                "caption" => update_caption_core(&db, "one", Some("new".into())),
                _ => update_kind_core(&db, "one", "FRAMED"),
            };
            assert!(result.is_err(), "{trigger}");
            assert_eq!(snapshot("a"), original);
            assert_eq!(snapshot("b"), unrelated);
            assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), 3);
            for id in ["one", "two", "other"] {
                assert_eq!(fs::read(db.directory.join(format!("media/{id}.jpg"))).unwrap(), id.as_bytes());
            }
            db.conn.lock().unwrap().execute_batch("DROP TRIGGER sabotage").unwrap();
        }
        update_caption_core(&db, "one", Some("new".into())).unwrap();
        update_kind_core(&db, "one", "FRAMED").unwrap();
        let added = add_gallery_bytes(&db, "a", &[0xff, 0xd8, 0xff, 1], Some("new".into())).unwrap();
        let saved = snapshot("a");
        assert_eq!(saved.len(), 3);
        assert_eq!(saved[0].0.caption.as_deref(), Some("new"));
        assert_eq!(saved[0].0.kind, "FRAMED");
        assert_eq!(saved[0].1.as_deref(), Some("m"));
        assert_eq!(saved[2].0, added);
        assert_eq!(saved[2].1, None);
        assert_eq!(snapshot("b"), unrelated);
        assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), 4);
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
    fn gallery_deletion_rejected_by_imported_trigger_preserves_photo_and_reference() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        let relative = "media/blocked.jpg";
        let file = db.directory.join(relative);
        let jpeg = [0xff, 0xd8, 0xff, 0x01];
        fs::write(&file, jpeg).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','ART','Artwork','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('photo','art',?1,'t')", params![relative]).unwrap();
            // Some externally edited SQLite archives suppress a deletion
            // without producing an SQL error.
            conn.execute_batch("CREATE TRIGGER block_gallery_photo
                BEFORE DELETE ON artwork_images WHEN OLD.id='photo'
                BEGIN SELECT RAISE(IGNORE); END;").unwrap();
        }

        let error = delete_gallery_image_core(&db, "photo").unwrap_err();
        assert!(error.contains("non riuscita"));
        assert_eq!(fs::read(&file).unwrap(), jpeg);
        assert_eq!(list_core(&db, "art").unwrap().len(), 1);

        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("DROP TRIGGER block_gallery_photo").unwrap();
        }
        delete_gallery_image_core(&db, "photo").unwrap();
        assert!(!file.exists());
        assert!(list_core(&db, "art").unwrap().is_empty());
    }

    #[test]
    fn gallery_deletion_rolls_back_if_an_imported_trigger_recreates_the_row() {
        let tmp = tempfile::tempdir().unwrap();
        let db = crate::db::open(tmp.path().join("app")).unwrap();
        let relative = "media/restored.jpg";
        let path = db.directory.join(relative);
        let jpeg = [0xff, 0xd8, 0xff, 0x07];
        fs::write(&path, jpeg).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','ART','Artwork','t','t');").unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,kind)
                VALUES('photo','art',?1,'Original caption','t',4,'DETAIL')", params![relative]).unwrap();
            // A DELETE reports one affected row, but this trigger restores it.
            // Without a transactional final-state check, the file may be unlinked.
            conn.execute_batch("CREATE TRIGGER restore_gallery_photo
                AFTER DELETE ON artwork_images WHEN OLD.id='photo'
                BEGIN
                    INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,kind)
                    VALUES(OLD.id,OLD.artwork_id,OLD.image_path,OLD.caption,OLD.created_at,OLD.sort_order,OLD.kind);
                END;").unwrap();
        }
        let error = delete_gallery_image_core(&db, "photo").unwrap_err();
        assert!(error.contains("non riuscita"), "{error}");
        assert_eq!(fs::read(&path).unwrap(), jpeg);
        let images = list_core(&db, "art").unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, "photo");
        assert_eq!(images[0].caption.as_deref(), Some("Original caption"));
        assert_eq!(images[0].kind, "DETAIL");

        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("DROP TRIGGER restore_gallery_photo").unwrap();
        }
        delete_gallery_image_core(&db, "photo").unwrap();
        assert!(!path.exists());
        assert!(list_core(&db, "art").unwrap().is_empty());
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
    fn imported_sqlite_triggers_cannot_silently_corrupt_gallery_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("app")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','ART','Artwork','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,kind,created_at)
                VALUES('photo','art','media/photo.jpg','Original','OTHER','t')", []).unwrap();
            conn.execute_batch(
                "CREATE TRIGGER ignore_caption BEFORE UPDATE OF caption ON artwork_images
                 BEGIN SELECT RAISE(IGNORE); END;"
            ).unwrap();
        }
        assert!(update_caption_core(&db, "photo", Some("New".into())).unwrap_err().contains("non aggiornata"));
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "DROP TRIGGER ignore_caption;
                 CREATE TRIGGER undo_caption AFTER UPDATE OF caption ON artwork_images
                 BEGIN
                   UPDATE artwork_images SET caption='Original' WHERE id=NEW.id;
                   UPDATE artworks SET title='Unexpected' WHERE id=NEW.artwork_id;
                 END;"
            ).unwrap();
        }
        assert!(update_caption_core(&db, "photo", Some("New".into())).unwrap_err().contains("non salvata"));
        {
            let conn = db.conn.lock().unwrap();
            let title: String = conn.query_row("SELECT title FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
            assert_eq!(title, "Artwork", "trigger side effects must roll back");
            conn.execute_batch(
                "DROP TRIGGER undo_caption;
                 CREATE TRIGGER rewrite_link AFTER UPDATE OF caption ON artwork_images
                 BEGIN UPDATE artwork_images SET image_path='media/other.jpg' WHERE id=NEW.id; END;"
            ).unwrap();
        }
        assert!(update_caption_core(&db, "photo", Some("New".into())).unwrap_err().contains("non salvata"));
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "DROP TRIGGER rewrite_link;
                 CREATE TRIGGER ignore_kind BEFORE UPDATE OF kind ON artwork_images
                 BEGIN SELECT RAISE(IGNORE); END;"
            ).unwrap();
        }
        assert!(update_kind_core(&db, "photo", "DETAIL").unwrap_err().contains("non aggiornato"));
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "DROP TRIGGER ignore_kind;
                 CREATE TRIGGER undo_kind AFTER UPDATE OF kind ON artwork_images
                 BEGIN UPDATE artwork_images SET kind='OTHER' WHERE id=NEW.id; END;"
            ).unwrap();
        }
        assert!(update_kind_core(&db, "photo", "DETAIL").unwrap_err().contains("non salvato"));
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch("DROP TRIGGER undo_kind;").unwrap();
        }
        let preserved = list_core(&db, "art").unwrap();
        assert_eq!(preserved.len(), 1);
        assert_eq!(preserved[0].caption.as_deref(), Some("Original"));
        assert_eq!(preserved[0].kind, "OTHER");
        assert_eq!(preserved[0].image_path, "media/photo.jpg");
        assert_eq!(update_caption_core(&db, "photo", Some("New".into())).unwrap().caption.as_deref(), Some("New"));
        assert_eq!(update_kind_core(&db, "photo", "DETAIL").unwrap().kind, "DETAIL");
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
        db.conn.lock().unwrap().execute("UPDATE artworks SET updated_at='2099-01-01T00:00:00.000000000Z' WHERE id='a'", []).unwrap();
        promote_core(&db, "a", "side").unwrap();
        let conn = db.conn.lock().unwrap();
        let revision: String = conn.query_row("SELECT updated_at FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(revision, "2099-01-01T00:00:00.000000001Z");
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
    fn promotion_rolls_back_when_imported_triggers_silently_ignore_a_swap() {
        let temp = tempfile::tempdir().unwrap();
        let db = crate::db::open(temp.path().join("app")).unwrap();
        let cover = "media/cover.png";
        let gallery = "media/gallery.png";
        let png = [137, 80, 78, 71, 13, 10, 26, 10];
        fs::write(db.directory.join(cover), png).unwrap();
        fs::write(db.directory.join(gallery), png).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('art','A','Art',?1,'t','t')", params![cover]).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at)
                VALUES('photo','art',?1,'Back','t')", params![gallery]).unwrap();
        }

        // With a cover, both the artwork UPDATE and the gallery UPDATE can
        // independently be suppressed. Neither must commit a half-swap.
        for (create, drop_trigger) in [
            ("CREATE TRIGGER block_cover BEFORE UPDATE ON artworks
              BEGIN SELECT RAISE(IGNORE); END;", "DROP TRIGGER block_cover;"),
            ("CREATE TRIGGER block_gallery BEFORE UPDATE ON artwork_images
              BEGIN SELECT RAISE(IGNORE); END;", "DROP TRIGGER block_gallery;"),
            ("CREATE TRIGGER undo_cover AFTER UPDATE OF image_path ON artworks
              BEGIN UPDATE artworks SET image_path=OLD.image_path WHERE id=NEW.id; END;",
              "DROP TRIGGER undo_cover;"),
            ("CREATE TRIGGER undo_gallery AFTER UPDATE OF image_path ON artwork_images
              BEGIN UPDATE artwork_images SET caption='Back' WHERE id=NEW.id; END;",
              "DROP TRIGGER undo_gallery;"),
            ("CREATE TRIGGER move_gallery AFTER UPDATE OF image_path ON artwork_images
              BEGIN UPDATE artwork_images SET sort_order=99 WHERE id=NEW.id; END;",
              "DROP TRIGGER move_gallery;"),
            ("CREATE TRIGGER undo_revision AFTER UPDATE OF image_path ON artwork_images
              BEGIN UPDATE artworks SET updated_at='t' WHERE id=NEW.artwork_id; END;",
              "DROP TRIGGER undo_revision;"),
        ] {
            db.conn.lock().unwrap().execute_batch(create).unwrap();
            let error = promote_core(&db, "art", "photo").unwrap_err();
            assert!(error.contains("Promozione incompleta"), "{error}");
            {
                let conn = db.conn.lock().unwrap();
                let primary: String = conn.query_row(
                    "SELECT image_path FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
                let (extra, caption): (String, String) = conn.query_row(
                    "SELECT image_path,caption FROM artwork_images WHERE id='photo'",
                    [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
                let updated: String = conn.query_row(
                    "SELECT updated_at FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
                assert_eq!((primary.as_str(), extra.as_str(), caption.as_str(), updated.as_str()),
                    (cover, gallery, "Back", "t"));
                let position: i64 = conn.query_row("SELECT sort_order FROM artwork_images WHERE id='photo'",
                    [], |r| r.get(0)).unwrap();
                assert_eq!(position, 0);
            }
            assert!(db.directory.join(cover).is_file());
            assert!(db.directory.join(gallery).is_file());
            db.conn.lock().unwrap().execute_batch(drop_trigger).unwrap();
        }
        promote_core(&db, "art", "photo").unwrap();
        let conn = db.conn.lock().unwrap();
        let primary: String = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
        assert_eq!(primary, gallery);
    }

    #[test]
    fn first_cover_promotion_rolls_back_when_gallery_delete_is_ignored() {
        let temp = tempfile::tempdir().unwrap();
        let db = crate::db::open(temp.path().join("app")).unwrap();
        let photo = "media/first.png";
        fs::write(db.directory.join(photo), [137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('art','A','Art','t','t')", []).unwrap();
            conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                VALUES('photo','art',?1,'t')", params![photo]).unwrap();
            conn.execute_batch("CREATE TRIGGER block_gallery_delete BEFORE DELETE ON artwork_images
                BEGIN SELECT RAISE(IGNORE); END;").unwrap();
        }
        let error = promote_core(&db, "art", "photo").unwrap_err();
        assert!(error.contains("Promozione incompleta"), "{error}");
        {
            let conn = db.conn.lock().unwrap();
            let primary: Option<String> = conn.query_row(
                "SELECT image_path FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
            let remaining: i64 = conn.query_row(
                "SELECT count(*) FROM artwork_images WHERE id='photo'", [], |r| r.get(0)).unwrap();
            let updated: String = conn.query_row(
                "SELECT updated_at FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
            assert_eq!((primary, remaining, updated.as_str()), (None, 1, "t"));
            conn.execute_batch("DROP TRIGGER block_gallery_delete;
                CREATE TRIGGER recreate_gallery AFTER DELETE ON artwork_images BEGIN
                    INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,kind,media_id)
                    VALUES(OLD.id,OLD.artwork_id,OLD.image_path,OLD.caption,OLD.created_at,OLD.sort_order,OLD.kind,OLD.media_id);
                END;").unwrap();
        }
        assert!(promote_core(&db, "art", "photo").unwrap_err().contains("Promozione incompleta"));
        {
            let conn = db.conn.lock().unwrap();
            let (primary, updated): (Option<String>, String) = conn.query_row(
                "SELECT image_path,updated_at FROM artworks WHERE id='art'", [],
                |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
            let remaining: String = conn.query_row("SELECT image_path FROM artwork_images WHERE id='photo'",
                [], |r| r.get(0)).unwrap();
            assert_eq!((primary, updated.as_str(), remaining.as_str()), (None, "t", photo));
            conn.execute_batch("DROP TRIGGER recreate_gallery;").unwrap();
        }
        assert!(db.directory.join(photo).is_file());
        promote_core(&db, "art", "photo").unwrap();
        let conn = db.conn.lock().unwrap();
        let primary: String = conn.query_row(
            "SELECT image_path FROM artworks WHERE id='art'", [], |r| r.get(0)).unwrap();
        let remaining: i64 = conn.query_row(
            "SELECT count(*) FROM artwork_images WHERE id='photo'", [], |r| r.get(0)).unwrap();
        assert_eq!((primary.as_str(), remaining), (photo, 0));
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
