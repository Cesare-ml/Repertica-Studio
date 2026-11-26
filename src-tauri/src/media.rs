//! Media center: a catalogue of every picture in the archive with its history of versions.
//!
//! * Each picture is a **media item** with one or more **revisions**. Revision 1 is always the
//!   untouched original; every edit (crop, perspective, rotation, mirroring, light and colour)
//!   creates a new revision and never alters an older file, so any version can be restored and
//!   certificates that froze an old picture keep working.
//! * Artworks and collectors keep their own file path columns (`image_path`); `media_id` links
//!   them to the catalogue. Editing a picture re-points every place that uses it to the new
//!   current revision.
//! * Pictures already in the archive are registered lazily the first time the library is opened.
//! * A catalogued file is never deleted just because an artwork or gallery stops using it; it
//!   stays in the library until it is explicitly purged from the trash.
use crate::{
    commands::{image_type, read_image_file, store_media},
    db::Database,
    imaging::{self, Operation},
    util::{display_path, fold},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use tauri::{Manager, State};
use uuid::Uuid;

const COPYRIGHT: [&str; 4] = ["UNKNOWN", "OWNED", "LICENSED", "PUBLIC_DOMAIN"];
const SIMILARITY_THRESHOLD: u32 = 6;

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn now() -> String { Utc::now().to_rfc3339() }
fn sha256_hex(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

// ------------------------------------------------------------------ types

#[derive(Clone, Debug, Serialize)]
pub struct MediaTag { pub id: String, pub name: String }

#[derive(Clone, Debug, Serialize)]
pub struct MediaRevision {
    pub id: String, pub number: i64, pub kind: String, pub image_path: String, pub width: i64, pub height: i64,
    pub size_bytes: i64, pub sha256: String, pub operations_json: Option<String>, pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MediaItem {
    pub id: String, pub title: Option<String>, pub description: Option<String>, pub alt_text: Option<String>,
    pub credits: Option<String>, pub copyright_status: String, pub original_name: String, pub state: String,
    pub current_revision_id: String, pub image_path: String, pub width: i64, pub height: i64, pub size_bytes: i64,
    pub revision_number: i64, pub tags: Vec<MediaTag>, pub usage_count: i64, pub created_at: String, pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MediaUsage {
    /// `ARTWORK_COVER`, `ARTWORK_GALLERY`, `COLLECTOR_PROFILE` or `COLLECTOR_PHOTO`.
    pub kind: String, pub owner_id: String, pub label: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MediaDetail {
    pub item: MediaItem, pub revisions: Vec<MediaRevision>, pub usages: Vec<MediaUsage>, pub collection_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MediaPage { pub items: Vec<MediaItem>, pub total: usize }

#[derive(Clone, Debug, Serialize)]
pub struct MediaCollection { pub id: String, pub name: String, pub item_count: i64 }

#[derive(Debug, Deserialize)]
pub struct MediaMetadataInput {
    pub title: Option<String>, pub description: Option<String>, pub alt_text: Option<String>,
    pub credits: Option<String>, pub copyright_status: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PreviewData { pub data_url: String, pub width: u32, pub height: u32 }

#[derive(Debug, Serialize)]
pub struct ImportFailure { pub path: String, pub message: String }
#[derive(Debug, Serialize)]
pub struct ImportResult { pub imported: Vec<MediaItem>, pub failed: Vec<ImportFailure> }

#[derive(Debug, Serialize)]
pub struct DuplicateGroup { pub sha256: String, pub size_bytes: i64, pub items: Vec<MediaItem> }
#[derive(Debug, Serialize)]
pub struct SimilarPair { pub first: MediaItem, pub second: MediaItem, pub distance: u32, pub similarity: f64 }

#[derive(Debug, Deserialize)]
pub struct ExportOptions { pub format: Option<String>, pub quality: Option<u8>, pub max_width: Option<u32>, pub max_height: Option<u32> }

/// Where a picture of a new artwork comes from: a file on disk (optionally edited first) or the library.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct PhotoSource {
    #[serde(default)] pub path: Option<String>,
    #[serde(default)] pub media_id: Option<String>,
    #[serde(default)] pub operations: Vec<Operation>,
    /// Gallery photographs only: what the picture shows (see `gallery::PHOTO_KINDS`).
    #[serde(default)] pub kind: Option<String>,
}

// ------------------------------------------------------------------ catalogue helpers

fn usage_sql() -> &'static str {
    "(SELECT count(*) FROM artworks WHERE image_media_id=i.id)+(SELECT count(*) FROM artwork_images WHERE media_id=i.id)
     +(SELECT count(*) FROM collector_images WHERE media_id=i.id)"
}

const ITEM_SELECT: &str = "SELECT i.id,i.title,i.description,i.alt_text,i.credits,i.copyright_status,i.original_name,i.state,
        i.current_revision_id,r.image_path,r.width,r.height,r.size_bytes,r.number,i.created_at,i.updated_at";

fn item_row(r: &rusqlite::Row<'_>, usage: i64) -> rusqlite::Result<MediaItem> {
    Ok(MediaItem {
        id: r.get(0)?, title: r.get(1)?, description: r.get(2)?, alt_text: r.get(3)?, credits: r.get(4)?,
        copyright_status: r.get(5)?, original_name: r.get(6)?, state: r.get(7)?, current_revision_id: r.get(8)?,
        image_path: r.get(9)?, width: r.get(10)?, height: r.get(11)?, size_bytes: r.get(12)?, revision_number: r.get(13)?,
        tags: vec![], usage_count: usage, created_at: r.get(14)?, updated_at: r.get(15)?,
    })
}

fn tags_by_item(conn: &Connection) -> Result<HashMap<String, Vec<MediaTag>>, String> {
    let mut statement = conn.prepare(
        "SELECT it.media_id,t.id,t.name FROM media_item_tags it JOIN media_tags t ON t.id=it.tag_id ORDER BY t.name COLLATE NOCASE").map_err(err)?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, MediaTag { id: r.get(1)?, name: r.get(2)? })))
        .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    let mut map: HashMap<String, Vec<MediaTag>> = HashMap::new();
    for (media, tag) in rows { map.entry(media).or_default().push(tag); }
    Ok(map)
}

fn all_items(conn: &Connection, state: &str) -> Result<Vec<MediaItem>, String> {
    let mut statement = conn.prepare(&format!(
        "{ITEM_SELECT},{usage} FROM media_items i JOIN media_revisions r ON r.id=i.current_revision_id
         WHERE i.state=?1 ORDER BY i.created_at DESC,i.id", usage = usage_sql())).map_err(err)?;
    let mut items = statement.query_map(params![state], |r| { let usage: i64 = r.get(16)?; item_row(r, usage) })
        .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    let mut tags = tags_by_item(conn)?;
    for item in &mut items { item.tags = tags.remove(&item.id).unwrap_or_default(); }
    Ok(items)
}

fn item_by_id(conn: &Connection, id: &str) -> Result<MediaItem, String> {
    let mut statement = conn.prepare(&format!(
        "{ITEM_SELECT},{usage} FROM media_items i JOIN media_revisions r ON r.id=i.current_revision_id WHERE i.id=?1", usage = usage_sql())).map_err(err)?;
    let mut item = statement.query_row(params![id], |r| { let usage: i64 = r.get(16)?; item_row(r, usage) })
        .optional().map_err(err)?.ok_or_else(|| "Immagine non trovata nella libreria".to_string())?;
    item.tags = tags_by_item(conn)?.remove(id).unwrap_or_default();
    Ok(item)
}

fn revisions_of(conn: &Connection, media_id: &str) -> Result<Vec<MediaRevision>, String> {
    let mut statement = conn.prepare(
        "SELECT id,number,kind,image_path,width,height,size_bytes,sha256,operations_json,created_at
         FROM media_revisions WHERE media_id=?1 ORDER BY number DESC").map_err(err)?;
    let revisions = statement.query_map(params![media_id], |r| Ok(MediaRevision {
        id: r.get(0)?, number: r.get(1)?, kind: r.get(2)?, image_path: r.get(3)?, width: r.get(4)?, height: r.get(5)?,
        size_bytes: r.get(6)?, sha256: r.get(7)?, operations_json: r.get(8)?, created_at: r.get(9)? }))
        .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    Ok(revisions)
}

fn usages_of(conn: &Connection, media_id: &str) -> Result<Vec<MediaUsage>, String> {
    let mut usages = Vec::new();
    let mut run = |sql: &str, kind: &str| -> Result<(), String> {
        let mut statement = conn.prepare(sql).map_err(err)?;
        let rows = statement.query_map(params![media_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?)))
            .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
        for (owner_id, label, kind_override) in rows {
            usages.push(MediaUsage { kind: kind_override.unwrap_or_else(|| kind.to_string()), owner_id, label });
        }
        Ok(())
    };
    run("SELECT id,code||' — '||title,NULL FROM artworks WHERE image_media_id=?1", "ARTWORK_COVER")?;
    run("SELECT a.id,a.code||' — '||a.title,NULL FROM artwork_images g JOIN artworks a ON a.id=g.artwork_id WHERE g.media_id=?1", "ARTWORK_GALLERY")?;
    run("SELECT c.id,c.name,CASE g.kind WHEN 'PROFILE' THEN 'COLLECTOR_PROFILE' ELSE 'COLLECTOR_PHOTO' END
         FROM collector_images g JOIN collectors c ON c.id=g.collector_id WHERE g.media_id=?1", "COLLECTOR_PHOTO")?;
    Ok(usages)
}

pub fn detail_core(conn: &Connection, media_id: &str) -> Result<MediaDetail, String> {
    let mut statement = conn.prepare("SELECT collection_id FROM media_collection_items WHERE media_id=?1").map_err(err)?;
    let collection_ids = statement.query_map(params![media_id], |r| r.get::<_, String>(0)).map_err(err)?
        .collect::<Result<Vec<_>, _>>().map_err(err)?;
    Ok(MediaDetail { item: item_by_id(conn, media_id)?, revisions: revisions_of(conn, media_id)?, usages: usages_of(conn, media_id)?, collection_ids })
}

/// A file that belongs to the catalogue must survive the removal of any single reference to it.
pub fn is_catalogued(conn: &Connection, relative: &str) -> bool {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM media_revisions WHERE image_path=?1)", params![relative], |r| r.get(0)).unwrap_or(false)
}

// A path may still be used by a legacy row whose media_id was never populated,
// or shared with a RAW file or payment evidence in an imported archive. Each
// table has its own UNIQUE path constraint, not a cross-table constraint.
// Treat SQL failures as in-use so deleting a photo never deletes other records' bytes.
fn path_has_live_references(conn: &Connection, relative: &str) -> Result<bool, String> {
    let referenced: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM artworks WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM artwork_images WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM collector_images WHERE image_path=?1)
          OR EXISTS(SELECT 1 FROM artwork_raw_files WHERE media_path=?1)
          OR EXISTS(SELECT 1 FROM sale_receipt_attachments WHERE media_path=?1)",
        params![relative], |r| r.get(0),
    ).map_err(err)?;
    if referenced { return Ok(true); }
    certificate_uses(conn, relative)
}

fn require_regular_media_dir(directory: &Path) -> Result<(), String> {
    let kind = fs::symlink_metadata(directory.join("media")).map_err(err)?.file_type();
    if !kind.is_dir() || kind.is_symlink() {
        return Err("La cartella media non è una directory locale regolare".into());
    }
    Ok(())
}

/// Delete a legacy picture only when no catalogue, artwork, gallery, collector,
/// RAW, receipt attachment or certificate references remain. Cleanup must fail
/// closed rather than lose history.
pub fn remove_file_unless_catalogued(conn: &Connection, directory: &Path, relative: &str) {
    if !crate::commands::safe_relative(relative)
        || is_catalogued(conn, relative)
        || path_has_live_references(conn, relative).unwrap_or(true)
        || require_regular_media_dir(directory).is_err()
    { return; }
    let full = directory.join(relative);
    if fs::symlink_metadata(&full).is_ok_and(|m| m.file_type().is_file()) {
        let _ = fs::remove_file(full);
    }
}

fn certificate_uses(conn: &Connection, relative: &str) -> Result<bool, String> {
    let mut statement = conn.prepare("SELECT snapshot_json FROM certificates").map_err(err)?;
    let snapshots = statement.query_map([], |r| r.get::<_, String>(0)).map_err(err)?;
    for snapshot in snapshots {
        // A malformed snapshot (including a non-object or invalid image_path type)
        // cannot prove a historical picture is unused: keep the file.
        let value: serde_json::Value = match serde_json::from_str(&snapshot.map_err(err)?) {
            Ok(value) => value,
            Err(_) => return Ok(true),
        };
        let Some(fields) = value.as_object() else { return Ok(true); };
        match fields.get("image_path") {
            Some(serde_json::Value::String(path)) if path == relative => return Ok(true),
            Some(serde_json::Value::String(_)) | Some(serde_json::Value::Null) | None => {}
            Some(_) => return Ok(true),
        }
    }
    Ok(false)
}

// ------------------------------------------------------------------ registering pictures

fn insert_item(conn: &Connection, media_id: &str, original_name: &str, relative: &str, bytes: &[u8], kind_dimensions: (u32, u32), mime_note: Option<&str>) -> Result<(), String> {
    let _ = mime_note;
    let revision_id = Uuid::new_v4().to_string();
    let stamp = now();
    conn.execute("INSERT INTO media_items(id,original_name,copyright_status,state,current_revision_id,created_at,updated_at) VALUES(?1,?2,'UNKNOWN','ACTIVE',?3,?4,?4)",
        params![media_id, original_name.chars().take(240).collect::<String>(), revision_id, stamp]).map_err(err)?;
    conn.execute("INSERT INTO media_revisions(id,media_id,number,kind,image_path,width,height,size_bytes,sha256,operations_json,created_at)
        VALUES(?1,?2,1,'ORIGINAL',?3,?4,?5,?6,?7,NULL,?8)",
        params![revision_id, media_id, relative, kind_dimensions.0, kind_dimensions.1, bytes.len() as i64, sha256_hex(bytes), stamp]).map_err(err)?;
    Ok(())
}

/// Register a picture that already sits in the media folder.
fn register_existing(conn: &Connection, directory: &Path, relative: &str) -> Result<String, String> {
    if !crate::commands::safe_relative(relative) { return Err("Percorso media non valido".into()); }
    require_regular_media_dir(directory)?;
    let full = directory.join(relative);
    let metadata = fs::symlink_metadata(&full).map_err(err)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() { return Err("File media non valido".into()); }
    let bytes = fs::read(&full).map_err(err)?;
    let dims = imaging::dimensions(&bytes)?;
    let media_id = Uuid::new_v4().to_string();
    let name = Path::new(relative).file_name().and_then(|n| n.to_str()).unwrap_or("immagine").to_string();
    insert_item(conn, &media_id, &name, relative, &bytes, dims, None)?;
    Ok(media_id)
}

/// Copy `bytes` into the media folder and register them as a new item.
fn register_new(conn: &Connection, directory: &Path, bytes: &[u8], original_name: &str) -> Result<(String, String), String> {
    let (extension, _) = image_type(bytes).ok_or("Sono consentite solo immagini JPEG, PNG e WebP")?;
    let dims = imaging::dimensions(bytes)?;
    require_regular_media_dir(directory)?;
    let relative = store_media(directory, bytes, extension)?;
    let media_id = Uuid::new_v4().to_string();
    if let Err(e) = insert_item(conn, &media_id, original_name, &relative, bytes, dims, None) {
        let _ = fs::remove_file(directory.join(&relative));
        return Err(e);
    }
    Ok((media_id, relative))
}

/// Register every picture of artworks and collectors that is not in the library yet.
pub fn sync_references(db: &Database) -> Result<usize, String> {
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let mut registered = 0;
    let mut by_path: HashMap<String, String> = HashMap::new();
    {
        let mut statement = tx.prepare("SELECT image_path,media_id FROM media_revisions r JOIN media_items i ON i.id=r.media_id").map_err(err)?;
        for row in statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).map_err(err)? {
            let (path, id) = row.map_err(err)?;
            by_path.insert(path, id);
        }
    }
    for (table, key_column, link_column, where_extra) in [
        ("artworks", "id", "image_media_id", "image_path IS NOT NULL"),
        ("artwork_images", "id", "media_id", "1=1"),
        ("collector_images", "id", "media_id", "1=1"),
    ] {
        let rows: Vec<(String, String)> = {
            let mut statement = tx.prepare(&format!("SELECT {key_column},image_path FROM {table} WHERE {link_column} IS NULL AND {where_extra}")).map_err(err)?;
            let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
            rows
        };
        for (key, path) in rows {
            let media_id = match by_path.get(&path) {
                Some(id) => id.clone(),
                None => match register_existing(&tx, &db.directory, &path) {
                    Ok(id) => { registered += 1; by_path.insert(path.clone(), id.clone()); id }
                    Err(_) => continue, // a missing or unreadable file is reported by the photo diagnostics
                },
            };
            tx.execute(&format!("UPDATE {table} SET {link_column}=?1 WHERE {key_column}=?2"), params![media_id, key]).map_err(err)?;
        }
    }
    tx.commit().map_err(err)?;
    Ok(registered)
}

// ------------------------------------------------------------------ revisions

/// Point every artwork, gallery and collector photo that shows this picture to its current file.
fn repoint_references(conn: &Connection, media_id: &str, relative: &str) -> Result<(), String> {
    conn.execute("UPDATE artworks SET image_path=?1,updated_at=?3 WHERE image_media_id=?2", params![relative, media_id, now()]).map_err(err)?;
    conn.execute("UPDATE artwork_images SET image_path=?1 WHERE media_id=?2", params![relative, media_id]).map_err(err)?;
    conn.execute("UPDATE collector_images SET image_path=?1 WHERE media_id=?2", params![relative, media_id]).map_err(err)?;
    Ok(())
}

fn set_current(conn: &Connection, media_id: &str, revision_id: &str, relative: &str) -> Result<(), String> {
    conn.execute("UPDATE media_items SET current_revision_id=?1,updated_at=?2 WHERE id=?3", params![revision_id, now(), media_id]).map_err(err)?;
    repoint_references(conn, media_id, relative)
}

fn current_path(conn: &Connection, media_id: &str) -> Result<String, String> {
    conn.query_row("SELECT r.image_path FROM media_items i JOIN media_revisions r ON r.id=i.current_revision_id WHERE i.id=?1", params![media_id], |r| r.get(0))
        .optional().map_err(err)?.ok_or_else(|| "Immagine non trovata nella libreria".to_string())
}

/// Validate a stored photo path without following a symlink. Use the same
/// checks for reading a revision and for restoring it to an artwork.
fn regular_media_path(directory: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    if !crate::commands::safe_relative(relative) { return Err("Percorso media non valido".into()); }
    require_regular_media_dir(directory)?;
    let full = directory.join(relative);
    let metadata = fs::symlink_metadata(&full).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "Il file di questa versione non è più disponibile".to_string()
        } else {
            err(error)
        }
    })?;
    if !metadata.file_type().is_file() { return Err("File media non valido".into()); }
    Ok(full)
}

pub(crate) fn read_media_file(directory: &Path, relative: &str) -> Result<Vec<u8>, String> {
    fs::read(regular_media_path(directory, relative)?).map_err(err)
}

/// Create a new revision by applying `operations` to the current version. Returns the new file path.
fn add_edit_revision(conn: &Connection, directory: &Path, media_id: &str, operations: &[Operation]) -> Result<String, String> {
    let source = read_media_file(directory, &current_path(conn, media_id)?)?;
    let processed = imaging::process(&source, operations)?;
    let relative = store_media(directory, &processed.bytes, processed.extension)?;
    let revision_id = Uuid::new_v4().to_string();
    let outcome = (|| -> Result<(), String> {
        let number: i64 = conn.query_row("SELECT COALESCE(MAX(number),0)+1 FROM media_revisions WHERE media_id=?1", params![media_id], |r| r.get(0)).map_err(err)?;
        conn.execute("INSERT INTO media_revisions(id,media_id,number,kind,image_path,width,height,size_bytes,sha256,operations_json,created_at)
            VALUES(?1,?2,?3,'EDIT',?4,?5,?6,?7,?8,?9,?10)",
            params![revision_id, media_id, number, relative, processed.width, processed.height, processed.bytes.len() as i64,
                sha256_hex(&processed.bytes), serde_json::to_string(operations).map_err(err)?, now()]).map_err(err)?;
        set_current(conn, media_id, &revision_id, &relative)
    })();
    if let Err(e) = outcome { let _ = fs::remove_file(directory.join(&relative)); return Err(e); }
    Ok(relative)
}

pub fn apply_edit_core(db: &Database, media_id: &str, operations: &[Operation]) -> Result<MediaDetail, String> {
    imaging::validate(operations)?;
    sync_references(db)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let state: String = tx.query_row("SELECT state FROM media_items WHERE id=?1", params![media_id], |r| r.get(0)).optional().map_err(err)?
        .ok_or("Immagine non trovata nella libreria")?;
    if state != "ACTIVE" { return Err("Ripristina l'immagine dal cestino prima di modificarla".into()); }
    let created = add_edit_revision(&tx, &db.directory, media_id, operations);
    match created {
        Ok(_) => { tx.commit().map_err(err)?; }
        Err(e) => { return Err(e); } // the transaction rolls back on drop; the new file was removed
    }
    detail_core(&conn, media_id)
}

pub fn restore_revision_core(db: &Database, media_id: &str, revision_id: &str) -> Result<MediaDetail, String> {
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let path: String = tx.query_row("SELECT image_path FROM media_revisions WHERE id=?1 AND media_id=?2", params![revision_id, media_id], |r| r.get(0))
        .optional().map_err(err)?.ok_or("Versione non trovata")?;
    // Never repoint a cover or gallery to a path that escaped the media store,
    // including a symlink substituted for an older revision's file.
    regular_media_path(&db.directory, &path)?;
    set_current(&tx, media_id, revision_id, &path)?;
    tx.commit().map_err(err)?;
    detail_core(&conn, media_id)
}

/// Resolve a [`PhotoSource`] of a new artwork into a catalogued picture: `(media_id, current_path)`.
/// Pictures from disk are registered (and edited when operations are given); library pictures are used as they are.
pub fn materialize(conn: &Connection, directory: &Path, source: &PhotoSource, bytes: Option<&[u8]>, name: &str) -> Result<(String, String), String> {
    if let Some(media_id) = &source.media_id {
        let state: Option<String> = conn.query_row("SELECT state FROM media_items WHERE id=?1", params![media_id], |r| r.get(0)).optional().map_err(err)?;
        if state.as_deref() != Some("ACTIVE") { return Err("Immagine della libreria non disponibile".into()); }
        return Ok((media_id.clone(), current_path(conn, media_id)?));
    }
    let bytes = bytes.ok_or("Fotografia mancante")?;
    let (media_id, mut relative) = register_new(conn, directory, bytes, name)?;
    if !source.operations.is_empty() {
        relative = add_edit_revision(conn, directory, &media_id, &source.operations)?;
    }
    Ok((media_id, relative))
}

// ------------------------------------------------------------------ library operations

pub fn list_core(db: &Database, query: Option<&str>, tag_id: Option<&str>, collection_id: Option<&str>, state: &str, offset: usize, limit: usize) -> Result<MediaPage, String> {
    sync_references(db)?;
    let conn = db.conn.lock().map_err(err)?;
    let mut items = all_items(&conn, state)?;
    if let Some(tag) = tag_id.filter(|t| !t.is_empty()) { items.retain(|i| i.tags.iter().any(|t| t.id == tag)); }
    if let Some(collection) = collection_id.filter(|c| !c.is_empty()) {
        let mut statement = conn.prepare("SELECT media_id FROM media_collection_items WHERE collection_id=?1").map_err(err)?;
        let members: HashSet<String> = statement.query_map(params![collection], |r| r.get::<_, String>(0)).map_err(err)?
            .collect::<Result<_, _>>().map_err(err)?;
        items.retain(|i| members.contains(&i.id));
    }
    let terms: Vec<String> = query.unwrap_or("").split_whitespace().map(fold).collect();
    if !terms.is_empty() {
        items.retain(|item| {
            let haystack = fold(&[Some(item.original_name.as_str()), item.title.as_deref(), item.description.as_deref(), item.alt_text.as_deref(), item.credits.as_deref()]
                .into_iter().flatten().chain(item.tags.iter().map(|t| t.name.as_str())).collect::<Vec<_>>().join("\n"));
            terms.iter().all(|t| haystack.contains(t.as_str()))
        });
    }
    let total = items.len();
    Ok(MediaPage { items: items.into_iter().skip(offset).take(limit.clamp(1, 500)).collect(), total })
}

fn clean_text(value: Option<String>, max: usize) -> Result<Option<String>, String> {
    match value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
        Some(v) if v.chars().count() > max => Err(format!("Testo troppo lungo (massimo {max} caratteri)")),
        other => Ok(other),
    }
}

pub fn update_core(db: &Database, media_id: &str, input: MediaMetadataInput) -> Result<MediaDetail, String> {
    if !COPYRIGHT.contains(&input.copyright_status.as_str()) { return Err("Stato dei diritti non valido".into()); }
    let (title, description, alt_text, credits) = (clean_text(input.title, 200)?, clean_text(input.description, 5_000)?, clean_text(input.alt_text, 500)?, clean_text(input.credits, 300)?);
    let conn = db.conn.lock().map_err(err)?;
    let changed = conn.execute("UPDATE media_items SET title=?2,description=?3,alt_text=?4,credits=?5,copyright_status=?6,updated_at=?7 WHERE id=?1",
        params![media_id, title, description, alt_text, credits, input.copyright_status, now()]).map_err(err)?;
    if changed == 0 { return Err("Immagine non trovata nella libreria".into()); }
    detail_core(&conn, media_id)
}

pub fn import_core(db: &Database, paths: &[String]) -> Result<ImportResult, String> {
    let mut result = ImportResult { imported: vec![], failed: vec![] };
    for path in paths {
        let name = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("immagine").to_string();
        let outcome = (|| -> Result<String, String> {
            let bytes = read_image_file(path)?;
            let conn = db.conn.lock().map_err(err)?;
            Ok(register_new(&conn, &db.directory, &bytes, &name)?.0)
        })();
        match outcome {
            Ok(id) => { let conn = db.conn.lock().map_err(err)?; result.imported.push(item_by_id(&conn, &id)?); }
            Err(message) => result.failed.push(ImportFailure { path: display_path(Path::new(path)), message }),
        }
    }
    Ok(result)
}

pub fn set_state_core(db: &Database, media_id: &str, trashed: bool) -> Result<MediaDetail, String> {
    let conn = db.conn.lock().map_err(err)?;
    if trashed && !usages_of(&conn, media_id)?.is_empty() {
        return Err("L'immagine è in uso: rimuovila prima dalle opere o dai collezionisti".into());
    }
    let changed = conn.execute("UPDATE media_items SET state=?2,updated_at=?3 WHERE id=?1", params![media_id, if trashed { "TRASHED" } else { "ACTIVE" }, now()]).map_err(err)?;
    if changed == 0 { return Err("Immagine non trovata nella libreria".into()); }
    detail_core(&conn, media_id)
}

/// Delete a trashed picture and all its files for good.
pub fn purge_core(db: &Database, media_id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let state: String = conn.query_row("SELECT state FROM media_items WHERE id=?1", params![media_id], |r| r.get(0)).optional().map_err(err)?
        .ok_or("Immagine non trovata nella libreria")?;
    if state != "TRASHED" { return Err("Sposta prima l'immagine nel cestino".into()); }
    if !usages_of(&conn, media_id)?.is_empty() { return Err("L'immagine è in uso e non può essere eliminata".into()); }
    let revisions = revisions_of(&conn, media_id)?;
    require_regular_media_dir(&db.directory)?;
    // Check all versions BEFORE deleting the catalogue row. media_id alone is
    // insufficient: older archives can still link a photo exclusively by path.
    for revision in &revisions {
        if !crate::commands::safe_relative(&revision.image_path) {
            return Err("Percorso media non valido: eliminazione annullata".into());
        }
        if path_has_live_references(&conn, &revision.image_path)? {
            return Err("Una scheda, una galleria, un collezionista, un RAW, un allegato incasso o un certificato conserva questo file".into());
        }
        match fs::symlink_metadata(db.directory.join(&revision.image_path)) {
            Ok(meta) if !meta.file_type().is_file() => {
                return Err("Un file della libreria non è regolare: eliminazione annullata".into());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(err(e)),
            _ => {}
        }
    }
    conn.execute("DELETE FROM media_items WHERE id=?1", params![media_id]).map_err(err)?;
    for revision in &revisions {
        let thumb = db.directory.join("thumbs").join(format!("{}.jpg", revision.id));
        let _ = fs::remove_file(thumb);
        let _ = fs::remove_file(db.directory.join(&revision.image_path));
    }
    Ok(())
}

// ------------------------------------------------------------------ tags and collections

pub fn tags_core(conn: &Connection) -> Result<Vec<MediaTag>, String> {
    let mut statement = conn.prepare("SELECT id,name FROM media_tags ORDER BY name COLLATE NOCASE").map_err(err)?;
    let tags = statement.query_map([], |r| Ok(MediaTag { id: r.get(0)?, name: r.get(1)? })).map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    Ok(tags)
}

fn clean_name(name: &str, what: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 60 { return Err(format!("{what}: da 1 a 60 caratteri")); }
    Ok(name.to_string())
}

pub fn create_tag_core(conn: &Connection, name: &str) -> Result<MediaTag, String> {
    let name = clean_name(name, "Nome del tag")?;
    if let Some(existing) = tags_core(conn)?.into_iter().find(|t| t.name.to_lowercase() == name.to_lowercase()) { return Ok(existing); }
    let id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO media_tags(id,name) VALUES(?1,?2)", params![id, name]).map_err(err)?;
    Ok(MediaTag { id, name })
}

pub fn set_tags_core(db: &Database, media_id: &str, tag_ids: &[String]) -> Result<MediaDetail, String> {
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    item_by_id(&tx, media_id)?;
    tx.execute("DELETE FROM media_item_tags WHERE media_id=?1", params![media_id]).map_err(err)?;
    for tag in tag_ids.iter().collect::<HashSet<_>>() {
        tx.execute("INSERT INTO media_item_tags(media_id,tag_id) VALUES(?1,?2)", params![media_id, tag]).map_err(|_| "Tag non trovato".to_string())?;
    }
    tx.commit().map_err(err)?;
    detail_core(&conn, media_id)
}

pub fn collections_core(conn: &Connection) -> Result<Vec<MediaCollection>, String> {
    let mut statement = conn.prepare(
        "SELECT c.id,c.name,(SELECT count(*) FROM media_collection_items WHERE collection_id=c.id) FROM media_collections c ORDER BY c.name COLLATE NOCASE").map_err(err)?;
    let collections = statement.query_map([], |r| Ok(MediaCollection { id: r.get(0)?, name: r.get(1)?, item_count: r.get(2)? })).map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
    Ok(collections)
}

pub fn create_collection_core(conn: &Connection, name: &str) -> Result<MediaCollection, String> {
    let name = clean_name(name, "Nome della raccolta")?;
    let id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO media_collections(id,name,created_at) VALUES(?1,?2,?3)", params![id, name, now()])
        .map_err(|_| "Esiste già una raccolta con questo nome".to_string())?;
    Ok(MediaCollection { id, name, item_count: 0 })
}

// ------------------------------------------------------------------ previews, thumbnails, export

fn data_url(encoded: &imaging::Encoded) -> PreviewData {
    PreviewData { data_url: format!("data:{};base64,{}", encoded.mime, STANDARD.encode(&encoded.bytes)), width: encoded.width, height: encoded.height }
}

pub fn thumbnail_core(db: &Database, media_id: &str) -> Result<String, String> {
    let (revision_id, path) = {
        let conn = db.conn.lock().map_err(err)?;
        conn.query_row("SELECT r.id,r.image_path FROM media_items i JOIN media_revisions r ON r.id=i.current_revision_id WHERE i.id=?1", params![media_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).optional().map_err(err)?.ok_or("Immagine non trovata nella libreria")?
    };
    // Revisions never change, so a thumbnail can be cached for good (the cache is not part of backups).
    let cache = db.directory.join("thumbs").join(format!("{revision_id}.jpg"));
    if let Ok(bytes) = fs::read(&cache) {
        return Ok(format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)));
    }
    let thumb = imaging::thumbnail(&read_media_file(&db.directory, &path)?, 360)?;
    if fs::create_dir_all(db.directory.join("thumbs")).is_ok() { let _ = fs::write(&cache, &thumb.bytes); }
    Ok(data_url(&thumb).data_url)
}

pub fn preview_core(db: &Database, media_id: &str, revision_id: Option<&str>, max_side: u32, operations: &[Operation]) -> Result<PreviewData, String> {
    let path = {
        let conn = db.conn.lock().map_err(err)?;
        match revision_id {
            Some(revision) => conn.query_row("SELECT image_path FROM media_revisions WHERE id=?1 AND media_id=?2", params![revision, media_id], |r| r.get(0)).optional().map_err(err)?
                .ok_or("Versione non trovata")?,
            None => current_path(&conn, media_id)?,
        }
    };
    Ok(data_url(&imaging::preview(&read_media_file(&db.directory, &path)?, max_side.clamp(64, 3000), operations)?))
}

/// Preview of a file on disk that is not in the archive yet (a photograph about to be added).
pub fn file_preview_core(path: &str, max_side: u32, operations: &[Operation]) -> Result<PreviewData, String> {
    let bytes = read_image_file(path)?;
    Ok(data_url(&imaging::preview(&bytes, max_side.clamp(64, 3000), operations)?))
}

pub fn export_core(db: &Database, media_id: &str, kind: &str, options: &ExportOptions, destination: &Path) -> Result<String, String> {
    let (current, original) = {
        let conn = db.conn.lock().map_err(err)?;
        let revisions = revisions_of(&conn, media_id)?;
        if revisions.is_empty() { return Err("Immagine non trovata nella libreria".into()); }
        let current = current_path(&conn, media_id)?;
        let original = revisions.iter().find(|r| r.number == 1).map(|r| r.image_path.clone()).unwrap_or_else(|| current.clone());
        (current, original)
    };
    let source = read_media_file(&db.directory, if kind == "original" { &original } else { &current })?;
    let bytes = match kind {
        "current" | "original" => source,
        "custom" => {
            let png = options.format.as_deref() == Some("png");
            imaging::export(&source, png, options.quality.unwrap_or(90), options.max_width, options.max_height)?.bytes
        }
        _ => return Err("Tipo di esportazione non valido".into()),
    };
    let live = fs::canonicalize(&db.directory).map_err(err)?;
    let parent = fs::canonicalize(destination.parent().ok_or("Percorso di destinazione non valido")?).map_err(err)?;
    if parent.starts_with(&live) { return Err("Non esportare documenti all'interno dell'archivio attivo".into()); }
    let target = parent.join(destination.file_name().ok_or("Nome file non valido")?);
    let mut output = OpenOptions::new().write(true).create_new(true).open(&target)
        .map_err(|e| format!("Impossibile creare il file (scegli un nuovo nome se esiste già): {}", err(e)))?;
    let written = output.write_all(&bytes).and_then(|_| output.sync_all());
    drop(output);
    if let Err(e) = written { let _ = fs::remove_file(&target); return Err(err(e)); }
    Ok(display_path(&target))
}

// ------------------------------------------------------------------ duplicates

pub fn exact_duplicates_core(db: &Database) -> Result<Vec<DuplicateGroup>, String> {
    sync_references(db)?;
    let conn = db.conn.lock().map_err(err)?;
    let mut groups: HashMap<String, Vec<MediaItem>> = HashMap::new();
    let mut hashes: HashMap<String, String> = HashMap::new();
    for item in all_items(&conn, "ACTIVE")? {
        let sha: String = conn.query_row("SELECT sha256 FROM media_revisions WHERE id=?1", params![item.current_revision_id], |r| r.get(0)).map_err(err)?;
        hashes.insert(item.id.clone(), sha.clone());
        groups.entry(sha).or_default().push(item);
    }
    let mut result: Vec<DuplicateGroup> = groups.into_iter().filter(|(_, items)| items.len() > 1)
        .map(|(sha256, items)| DuplicateGroup { size_bytes: items[0].size_bytes, sha256, items }).collect();
    result.sort_by(|a, b| b.items.len().cmp(&a.items.len()).then(a.sha256.cmp(&b.sha256)));
    Ok(result)
}

pub fn similar_core(db: &Database) -> Result<Vec<SimilarPair>, String> {
    sync_references(db)?;
    let items = { let conn = db.conn.lock().map_err(err)?; all_items(&conn, "ACTIVE")? };
    let mut hashes: Vec<(MediaItem, u64)> = Vec::new();
    for item in items {
        let stored: Option<(Option<String>, Option<String>)> = {
            let conn = db.conn.lock().map_err(err)?;
            conn.query_row("SELECT dhash,dhash_revision_id FROM media_items WHERE id=?1", params![item.id], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(err)?
        };
        let cached = match stored { Some((Some(h), Some(rev))) if rev == item.current_revision_id => u64::from_str_radix(&h, 16).ok(), _ => None };
        let hash = match cached {
            Some(h) => h,
            None => {
                let Ok(bytes) = read_media_file(&db.directory, &item.image_path) else { continue };
                let Ok(h) = imaging::dhash(&bytes) else { continue };
                let conn = db.conn.lock().map_err(err)?;
                conn.execute("UPDATE media_items SET dhash=?2,dhash_revision_id=?3 WHERE id=?1", params![item.id, format!("{h:016x}"), item.current_revision_id]).map_err(err)?;
                h
            }
        };
        hashes.push((item, hash));
    }
    let mut pairs = Vec::new();
    for i in 0..hashes.len() {
        for j in i + 1..hashes.len() {
            let distance = imaging::hamming(hashes[i].1, hashes[j].1);
            // Byte-identical files are reported by the exact comparison, not here.
            if distance <= SIMILARITY_THRESHOLD && hashes[i].0.image_path != hashes[j].0.image_path {
                pairs.push(SimilarPair { first: hashes[i].0.clone(), second: hashes[j].0.clone(), distance, similarity: 1.0 - f64::from(distance) / 64.0 });
            }
        }
    }
    pairs.sort_by_key(|p| p.distance);
    Ok(pairs)
}

// ------------------------------------------------------------------ using library pictures on artworks

pub fn artwork_cover_from_media(db: &Database, artwork_id: &str, media_id: &str) -> Result<(), String> {
    sync_references(db)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let item = item_by_id(&tx, media_id)?;
    if item.state != "ACTIVE" { return Err("Immagine della libreria non disponibile".into()); }
    let old: Option<String> = tx.query_row("SELECT image_path FROM artworks WHERE id=?1", params![artwork_id], |r| r.get::<_, Option<String>>(0)).optional().map_err(err)?
        .ok_or("Opera non trovata")?;
    tx.execute("UPDATE artworks SET image_path=?2,image_media_id=?3,updated_at=?4 WHERE id=?1", params![artwork_id, item.image_path, media_id, now()]).map_err(err)?;
    tx.commit().map_err(err)?;
    if let Some(old) = old.filter(|o| *o != item.image_path) {
        // The previous cover goes away only if nothing else needs it.
        if !certificate_uses(&conn, &old)? { remove_file_unless_catalogued(&conn, &db.directory, &old); }
    }
    Ok(())
}

pub fn artwork_gallery_from_media(db: &Database, artwork_id: &str, media_id: &str, caption: Option<String>, kind: Option<String>) -> Result<(), String> {
    let kind = crate::gallery::normalize_kind(kind.as_deref())?;
    sync_references(db)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let item = item_by_id(&tx, media_id)?;
    if item.state != "ACTIVE" { return Err("Immagine della libreria non disponibile".into()); }
    let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if !exists { return Err("Opera non trovata".into()); }
    let count: i64 = tx.query_row("SELECT count(*) FROM artwork_images WHERE artwork_id=?1", params![artwork_id], |r| r.get(0)).map_err(err)?;
    if count >= 20 { return Err("Massimo 20 fotografie aggiuntive per opera".into()); }
    let in_use: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM artwork_images WHERE image_path=?1) OR EXISTS(SELECT 1 FROM collector_images WHERE image_path=?1)",
        params![item.image_path], |r| r.get(0)).map_err(err)?;
    if in_use { return Err("Questa immagine è già usata in una galleria".into()); }
    let caption = clean_text(caption, 300)?;
    tx.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,media_id,kind)
        VALUES(?1,?2,?3,?4,?5,(SELECT COALESCE(MAX(sort_order),0)+1 FROM artwork_images WHERE artwork_id=?2),?6,?7)",
        params![Uuid::new_v4().to_string(), artwork_id, item.image_path, caption, now(), media_id, kind]).map_err(err)?;
    tx.commit().map_err(err)
}

// ------------------------------------------------------------------ commands

fn blocking<T: Send + 'static>(app: tauri::AppHandle, work: impl FnOnce(&Database) -> Result<T, String> + Send + 'static) -> impl std::future::Future<Output = Result<T, String>> {
    async move {
        tauri::async_runtime::spawn_blocking(move || { let db = app.state::<Database>(); work(&db) }).await.map_err(err)?
    }
}

#[tauri::command]
pub async fn media_list(app: tauri::AppHandle, query: Option<String>, tag_id: Option<String>, collection_id: Option<String>,
    state: Option<String>, offset: Option<usize>, limit: Option<usize>) -> Result<MediaPage, String> {
    blocking(app, move |db| list_core(db, query.as_deref(), tag_id.as_deref(), collection_id.as_deref(),
        if state.as_deref() == Some("TRASHED") { "TRASHED" } else { "ACTIVE" }, offset.unwrap_or(0), limit.unwrap_or(60))).await
}
#[tauri::command]
pub fn media_detail(db: State<'_, Database>, media_id: String) -> Result<MediaDetail, String> {
    let conn = db.conn.lock().map_err(err)?;
    detail_core(&conn, &media_id)
}
#[tauri::command]
pub fn media_update(db: State<'_, Database>, media_id: String, input: MediaMetadataInput) -> Result<MediaDetail, String> { update_core(&db, &media_id, input) }
#[tauri::command]
pub async fn media_thumbnail(app: tauri::AppHandle, media_id: String) -> Result<String, String> { blocking(app, move |db| thumbnail_core(db, &media_id)).await }
#[tauri::command]
pub async fn media_preview(app: tauri::AppHandle, media_id: String, revision_id: Option<String>, max_side: Option<u32>, operations: Option<Vec<Operation>>) -> Result<PreviewData, String> {
    blocking(app, move |db| preview_core(db, &media_id, revision_id.as_deref(), max_side.unwrap_or(1400), &operations.unwrap_or_default())).await
}
#[tauri::command]
pub async fn media_file_preview(path: String, max_side: Option<u32>, operations: Option<Vec<Operation>>) -> Result<PreviewData, String> {
    tauri::async_runtime::spawn_blocking(move || file_preview_core(&path, max_side.unwrap_or(1400), &operations.unwrap_or_default())).await.map_err(err)?
}
#[tauri::command]
pub async fn media_apply_edit(app: tauri::AppHandle, media_id: String, operations: Vec<Operation>) -> Result<MediaDetail, String> {
    blocking(app, move |db| apply_edit_core(db, &media_id, &operations)).await
}
#[tauri::command]
pub fn media_restore_revision(db: State<'_, Database>, media_id: String, revision_id: String) -> Result<MediaDetail, String> { restore_revision_core(&db, &media_id, &revision_id) }
#[tauri::command]
pub async fn media_import(app: tauri::AppHandle, paths: Vec<String>) -> Result<ImportResult, String> { blocking(app, move |db| import_core(db, &paths)).await }
#[tauri::command]
pub fn media_tags(db: State<'_, Database>) -> Result<Vec<MediaTag>, String> { let conn = db.conn.lock().map_err(err)?; tags_core(&conn) }
#[tauri::command]
pub fn media_create_tag(db: State<'_, Database>, name: String) -> Result<MediaTag, String> { let conn = db.conn.lock().map_err(err)?; create_tag_core(&conn, &name) }
#[tauri::command]
pub fn media_set_tags(db: State<'_, Database>, media_id: String, tag_ids: Vec<String>) -> Result<MediaDetail, String> { set_tags_core(&db, &media_id, &tag_ids) }
#[tauri::command]
pub fn media_collections(db: State<'_, Database>) -> Result<Vec<MediaCollection>, String> { let conn = db.conn.lock().map_err(err)?; collections_core(&conn) }
#[tauri::command]
pub fn media_create_collection(db: State<'_, Database>, name: String) -> Result<MediaCollection, String> { let conn = db.conn.lock().map_err(err)?; create_collection_core(&conn, &name) }
#[tauri::command]
pub fn media_delete_collection(db: State<'_, Database>, collection_id: String) -> Result<(), String> {
    db.conn.lock().map_err(err)?.execute("DELETE FROM media_collections WHERE id=?1", params![collection_id]).map(|_| ()).map_err(err)
}
#[tauri::command]
pub fn media_collection_add(db: State<'_, Database>, collection_id: String, media_id: String) -> Result<MediaDetail, String> {
    let conn = db.conn.lock().map_err(err)?;
    conn.execute("INSERT OR IGNORE INTO media_collection_items(collection_id,media_id) VALUES(?1,?2)", params![collection_id, media_id]).map_err(|_| "Raccolta o immagine non trovata".to_string())?;
    detail_core(&conn, &media_id)
}
#[tauri::command]
pub fn media_collection_remove(db: State<'_, Database>, collection_id: String, media_id: String) -> Result<MediaDetail, String> {
    let conn = db.conn.lock().map_err(err)?;
    conn.execute("DELETE FROM media_collection_items WHERE collection_id=?1 AND media_id=?2", params![collection_id, media_id]).map_err(err)?;
    detail_core(&conn, &media_id)
}
#[tauri::command]
pub fn media_trash(db: State<'_, Database>, media_id: String) -> Result<MediaDetail, String> { set_state_core(&db, &media_id, true) }
#[tauri::command]
pub fn media_untrash(db: State<'_, Database>, media_id: String) -> Result<MediaDetail, String> { set_state_core(&db, &media_id, false) }
#[tauri::command]
pub fn media_purge(db: State<'_, Database>, media_id: String) -> Result<(), String> { purge_core(&db, &media_id) }
#[tauri::command]
pub async fn media_id_for_path(app: tauri::AppHandle, path: String) -> Result<Option<String>, String> {
    blocking(app, move |db| {
        sync_references(db)?;
        let conn = db.conn.lock().map_err(err)?;
        conn.query_row("SELECT media_id FROM media_revisions WHERE image_path=?1", params![path], |r| r.get(0)).optional().map_err(err)
    }).await
}
#[tauri::command]
pub async fn media_export(app: tauri::AppHandle, media_id: String, kind: String, options: Option<ExportOptions>, destination: String) -> Result<String, String> {
    blocking(app, move |db| export_core(db, &media_id, &kind, &options.unwrap_or(ExportOptions { format: None, quality: None, max_width: None, max_height: None }), Path::new(&destination))).await
}
#[tauri::command]
pub async fn media_exact_duplicates(app: tauri::AppHandle) -> Result<Vec<DuplicateGroup>, String> { blocking(app, exact_duplicates_core).await }
#[tauri::command]
pub async fn media_similar(app: tauri::AppHandle) -> Result<Vec<SimilarPair>, String> { blocking(app, similar_core).await }
#[tauri::command]
pub fn artwork_set_cover_from_media(db: State<'_, Database>, artwork_id: String, media_id: String) -> Result<(), String> { artwork_cover_from_media(&db, &artwork_id, &media_id) }
#[tauri::command]
pub fn artwork_add_gallery_from_media(db: State<'_, Database>, artwork_id: String, media_id: String, caption: Option<String>, kind: Option<String>) -> Result<(), String> {
    artwork_gallery_from_media(&db, &artwork_id, &media_id, caption, kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
        let image = image::RgbImage::from_pixel(width, height, image::Rgb(color));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 90).encode_image(&image).unwrap();
        bytes
    }
    /// A two-tone picture so that edits visibly change it.
    fn split_jpeg(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbImage::from_fn(width, height, |x, _| if x < width / 2 { image::Rgb([220, 20, 20]) } else { image::Rgb([20, 20, 220]) });
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 95).encode_image(&image).unwrap();
        bytes
    }
    fn setup() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        (dir, db)
    }
    fn put(db: &Database, name: &str, bytes: &[u8]) -> String {
        fs::write(db.directory.join("media").join(name), bytes).unwrap();
        format!("media/{name}")
    }
    fn artwork_with_cover(db: &Database, id: &str, cover: &str) {
        db.conn.lock().unwrap().execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at) VALUES(?1,?1,'Opera',?2,'t','t')", params![id, cover]).unwrap();
    }
    fn files(db: &Database) -> usize { fs::read_dir(db.directory.join("media")).unwrap().count() }
    fn rotate() -> Vec<Operation> { vec![Operation::Rotate { degrees: 90 }] }
    fn first_item(db: &Database) -> String { list_core(db, None, None, None, "ACTIVE", 0, 10).unwrap().items[0].id.clone() }
    fn metadata(copyright: &str) -> MediaMetadataInput {
        MediaMetadataInput { title: None, description: None, alt_text: None, credits: None, copyright_status: copyright.into() }
    }

    #[test]
    fn existing_pictures_are_registered_once_and_linked() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &jpeg(40, 30, [200, 100, 50]));
        let gallery = put(&db, "g1.jpg", &jpeg(20, 20, [10, 200, 50]));
        let profile = put(&db, "p1.jpg", &jpeg(10, 10, [50, 50, 50]));
        artwork_with_cover(&db, "a", &cover);
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at,sort_order) VALUES('g','a',?1,'t',1)", params![gallery]).unwrap();
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Anna','t','t')", []).unwrap();
            c.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at) VALUES('i','c','PROFILE',?1,'t')", params![profile]).unwrap();
            c.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at) VALUES('missing','M','M','media/assente.jpg','t','t')", []).unwrap();
        }
        assert_eq!(sync_references(&db).unwrap(), 3);
        assert_eq!(sync_references(&db).unwrap(), 0, "registering is idempotent");
        let page = list_core(&db, None, None, None, "ACTIVE", 0, 50).unwrap();
        assert_eq!(page.total, 3);
        let item = page.items.iter().find(|i| i.image_path == cover).unwrap();
        assert_eq!((item.width, item.height, item.revision_number, item.usage_count), (40, 30, 1, 1));
        let linked: Option<String> = db.conn.lock().unwrap().query_row("SELECT image_media_id FROM artworks WHERE id='missing'", [], |r| r.get(0)).unwrap();
        assert!(linked.is_none(), "an unreadable file is skipped, not invented");
        let detail = detail_core(&db.conn.lock().unwrap(), &item.id).unwrap();
        assert_eq!(detail.usages[0].kind, "ARTWORK_COVER");
    }

    #[test]
    fn editing_creates_a_revision_keeps_the_original_and_repoints_every_use() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &split_jpeg(40, 20));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        let edited = apply_edit_core(&db, &id, &rotate()).unwrap();
        assert_eq!((edited.item.width, edited.item.height, edited.item.revision_number), (20, 40, 2));
        assert_eq!(edited.revisions.iter().map(|r| (r.number, r.kind.as_str())).collect::<Vec<_>>(), vec![(2, "EDIT"), (1, "ORIGINAL")]);
        assert!(edited.revisions[0].operations_json.as_deref().unwrap().contains("rotate"));
        let artwork_path: String = db.conn.lock().unwrap().query_row("SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(artwork_path, edited.item.image_path, "the artwork shows the new version");
        assert_eq!(fs::read(db.directory.join(&cover)).unwrap(), split_jpeg(40, 20), "the original file is never touched");
        let restored = restore_revision_core(&db, &id, &edited.revisions[1].id).unwrap();
        assert_eq!((restored.item.revision_number, restored.item.width), (1, 40));
        let artwork_path: String = db.conn.lock().unwrap().query_row("SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(artwork_path, cover);
        assert!(restore_revision_core(&db, &id, "inesistente").is_err());
        assert!(apply_edit_core(&db, &id, &[]).is_err());
        assert!(apply_edit_core(&db, "sconosciuto", &rotate()).is_err());
    }

    #[test]
    fn restoring_missing_revision_never_repoints_an_artwork() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &split_jpeg(40, 20));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        let edited = apply_edit_core(&db, &id, &rotate()).unwrap();
        let original_revision_id = edited.revisions[1].id.clone();
        fs::remove_file(db.directory.join(&cover)).unwrap();

        assert!(restore_revision_core(&db, &id, &original_revision_id).unwrap_err().contains("non è più disponibile"));
        let current = detail_core(&db.conn.lock().unwrap(), &id).unwrap();
        assert_eq!(current.item.current_revision_id, edited.item.current_revision_id);
        let linked: String = db.conn.lock().unwrap().query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(linked, edited.item.image_path, "a failed restore must leave the current cover intact");
    }

    #[cfg(unix)]
    #[test]
    fn restoring_symlinked_revision_rejects_external_file_and_preserves_links() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &split_jpeg(40, 20));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        let edited = apply_edit_core(&db, &id, &rotate()).unwrap();
        let original_revision_id = edited.revisions[1].id.clone();
        let external = tempfile::tempdir().unwrap();
        let outside = external.path().join("outside.jpg");
        let outside_bytes = jpeg(20, 20, [7, 9, 11]);
        fs::write(&outside, &outside_bytes).unwrap();
        fs::remove_file(db.directory.join(&cover)).unwrap();
        std::os::unix::fs::symlink(&outside, db.directory.join(&cover)).unwrap();

        assert!(restore_revision_core(&db, &id, &original_revision_id).unwrap_err().contains("File media non valido"));
        assert!(read_media_file(&db.directory, &cover).is_err());
        let current = detail_core(&db.conn.lock().unwrap(), &id).unwrap();
        assert_eq!(current.item.current_revision_id, edited.item.current_revision_id);
        let linked: String = db.conn.lock().unwrap().query_row(
            "SELECT image_path FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(linked, edited.item.image_path);
        assert_eq!(fs::read(outside).unwrap(), outside_bytes, "external files must remain untouched");
    }

    #[test]
    fn editing_a_picture_used_in_several_places_updates_all_of_them() {
        let (_dir, db) = setup();
        let shared = put(&db, "shared.jpg", &split_jpeg(30, 30));
        artwork_with_cover(&db, "a", &shared);
        db.conn.lock().unwrap().execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at,sort_order) VALUES('g','a',?1,'t',1)", params![shared]).unwrap();
        sync_references(&db).unwrap();
        assert_eq!(list_core(&db, None, None, None, "ACTIVE", 0, 10).unwrap().total, 1, "one file used twice is one item");
        let edited = apply_edit_core(&db, &first_item(&db), &[Operation::Flip { horizontal: true, vertical: false }]).unwrap();
        let gallery: String = db.conn.lock().unwrap().query_row("SELECT image_path FROM artwork_images WHERE id='g'", [], |r| r.get(0)).unwrap();
        assert_eq!(gallery, edited.item.image_path);
        assert_eq!(edited.item.usage_count, 2);
    }

    #[test]
    fn a_catalogued_file_survives_when_a_photo_is_removed_and_can_be_purged_from_the_trash() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &jpeg(20, 20, [1, 2, 3]));
        artwork_with_cover(&db, "a", &cover);
        db.conn.lock().unwrap().execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at,sort_order) VALUES('g','a','media/cover.jpg','t',1)", []).unwrap();
        sync_references(&db).unwrap();
        let id = first_item(&db);
        crate::gallery::delete_gallery_image_for_test(&db, "g");
        assert!(db.directory.join(&cover).is_file(), "the library keeps the file");
        assert!(set_state_core(&db, &id, true).unwrap_err().contains("in uso"), "still used by the cover");
        db.conn.lock().unwrap().execute("UPDATE artworks SET image_path=NULL,image_media_id=NULL", []).unwrap();
        assert!(purge_core(&db, &id).unwrap_err().contains("cestino"));
        let trashed = set_state_core(&db, &id, true).unwrap();
        assert_eq!(trashed.item.state, "TRASHED");
        assert_eq!(list_core(&db, None, None, None, "ACTIVE", 0, 10).unwrap().total, 0);
        assert_eq!(list_core(&db, None, None, None, "TRASHED", 0, 10).unwrap().total, 1);
        assert!(apply_edit_core(&db, &id, &rotate()).unwrap_err().contains("cestino"));
        assert_eq!(set_state_core(&db, &id, false).unwrap().item.state, "ACTIVE");
        set_state_core(&db, &id, true).unwrap();
        purge_core(&db, &id).unwrap();
        assert_eq!(files(&db), 0, "purging removes every file of the picture");
        assert!(purge_core(&db, &id).is_err());
    }

    #[test]
    fn a_picture_frozen_in_a_certificate_can_never_be_purged() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &jpeg(20, 20, [9, 9, 9]));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        db.conn.lock().unwrap().execute("INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at) VALUES('k','a','RS-1','2026-11-21','A',?1,'t')",
            params![format!("{{\"image_path\":\"{cover}\"}}")]).unwrap();
        db.conn.lock().unwrap().execute("UPDATE artworks SET image_path=NULL,image_media_id=NULL WHERE id='a'", []).unwrap();
        set_state_core(&db, &id, true).unwrap();
        assert!(purge_core(&db, &id).unwrap_err().contains("certificato"));
        assert!(db.directory.join(&cover).is_file());
    }

    #[test]
    fn purge_refuses_legacy_artwork_and_collector_paths_without_media_links() {
        let (_dir, db) = setup();
        let cover = put(&db, "shared.jpg", &jpeg(20, 20, [8, 3, 1]));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE artworks SET image_media_id=NULL WHERE id='a'", []).unwrap();
        }
        set_state_core(&db, &id, true).unwrap();
        assert!(purge_core(&db, &id).unwrap_err().contains("conserva"));
        assert!(db.directory.join(&cover).is_file());
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE artworks SET image_path=NULL WHERE id='a'", []).unwrap();
            conn.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Anna','t','t')", []).unwrap();
            conn.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at)
                VALUES('p','c','PROFILE',?1,'t')", params![cover]).unwrap();
        }
        assert!(purge_core(&db, &id).unwrap_err().contains("conserva"));
        assert!(db.directory.join(&cover).is_file());
        db.conn.lock().unwrap().execute("DELETE FROM collector_images WHERE id='p'", []).unwrap();
        purge_core(&db, &id).unwrap();
        assert!(!db.directory.join(&cover).exists());
    }

    #[test]
    fn legacy_cleanup_preserves_shared_photos_and_unreadable_certificate_history() {
        let (_dir, db) = setup();
        let shared = put(&db, "shared.jpg", &jpeg(20, 20, [2, 4, 6]));
        artwork_with_cover(&db, "a", &shared);
        {
            let conn = db.conn.lock().unwrap();
            remove_file_unless_catalogued(&conn, &db.directory, &shared);
            assert!(db.directory.join(&shared).exists(), "a live artwork still owns the file");
            conn.execute("UPDATE artworks SET image_path=NULL WHERE id='a'", []).unwrap();
            conn.execute("INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                VALUES('k','a','RS-1','2026-11-21','A',?1,'t')", params![r#"{"image_path":17}"#]).unwrap();
            remove_file_unless_catalogued(&conn, &db.directory, &shared);
        }
        assert!(db.directory.join(&shared).exists(), "invalid historical path must never permit deletion");
    }


    #[test]
    fn cleanup_keeps_files_shared_with_raw_and_receipt_evidence() {
        let (_dir, db) = setup();
        let raw = put(&db, "legacy-raw.jpg", &jpeg(12, 12, [8, 4, 3]));
        let evidence_bytes = jpeg(12, 12, [4, 8, 3]);
        let evidence = put(&db, "legacy-receipt.jpg", &evidence_bytes);
        artwork_with_cover(&db, "a", &raw);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE artworks SET image_path=NULL WHERE id='a'", []).unwrap();
            // Imported or externally edited archives can reuse a filename in
            // another table; the path constraints are only unique per table.
            conn.execute(
                "INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at)
                 VALUES('a',?1,'source.dng',100,?2,'t')",
                params![raw, "a".repeat(64)],
            ).unwrap();
            remove_file_unless_catalogued(&conn, &db.directory, &raw);
            assert!(db.directory.join(&raw).is_file(), "RAW reference must preserve the file");
            conn.execute("DELETE FROM artwork_raw_files WHERE artwork_id='a'", []).unwrap();
            remove_file_unless_catalogued(&conn, &db.directory, &raw);
            assert!(!db.directory.join(&raw).exists(), "unreferenced legacy photo can be removed");

            conn.execute_batch(
                "INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Anna','t','t');
                 INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                    VALUES('s','a','c',1000,'2026-11-21','t');
                 INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                    VALUES('r','s','2026-11-21',1000,'cash','t');"
            ).unwrap();
            conn.execute(
                "INSERT INTO sale_receipt_attachments(
                    receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at
                 ) VALUES('r',?1,'receipt.jpg','image/jpeg',?2,?3,'t')",
                params![evidence, evidence_bytes.len() as i64, sha256_hex(&evidence_bytes)],
            ).unwrap();
            remove_file_unless_catalogued(&conn, &db.directory, &evidence);
            assert!(db.directory.join(&evidence).is_file(), "receipt evidence must survive photo cleanup");
            conn.execute("DELETE FROM sale_receipt_attachments WHERE receipt_id='r'", []).unwrap();
            remove_file_unless_catalogued(&conn, &db.directory, &evidence);
            assert!(!db.directory.join(&evidence).exists());
        }
    }

    #[test]
    fn media_purge_refuses_raw_or_receipt_refs_even_without_a_media_id_link() {
        let (_dir, db) = setup();
        let photo = put(&db, "original.jpg", &jpeg(12, 12, [2, 5, 9]));
        artwork_with_cover(&db, "a", &photo);
        sync_references(&db).unwrap();
        let item = first_item(&db);
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE artworks SET image_path=NULL,image_media_id=NULL WHERE id='a'", []).unwrap();
            conn.execute(
                "INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at)
                 VALUES('a',?1,'source.dng',100,?2,'t')",
                params![photo, "a".repeat(64)],
            ).unwrap();
        }
        set_state_core(&db, &item, true).unwrap();
        assert!(purge_core(&db, &item).unwrap_err().contains("conserva"));
        assert!(db.directory.join(&photo).is_file());
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("DELETE FROM artwork_raw_files WHERE artwork_id='a'", []).unwrap();
            conn.execute_batch(
                "INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Anna','t','t');
                 INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                    VALUES('s','a','c',1000,'2026-11-21','t');
                 INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                    VALUES('r','s','2026-11-21',1000,'cash','t');"
            ).unwrap();
            conn.execute(
                "INSERT INTO sale_receipt_attachments(
                    receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at
                 ) VALUES('r',?1,'receipt.jpg','image/jpeg',100,?2,'t')",
                params![photo, "b".repeat(64)],
            ).unwrap();
        }
        assert!(purge_core(&db, &item).unwrap_err().contains("conserva"));
        assert!(db.directory.join(&photo).is_file());
        db.conn.lock().unwrap().execute("DELETE FROM sale_receipt_attachments WHERE receipt_id='r'", []).unwrap();
        purge_core(&db, &item).unwrap();
        assert!(!db.directory.join(&photo).exists());
    }

    #[cfg(unix)]
    #[test]
    fn purge_rejects_replaced_media_directory_without_deleting_catalogue_or_external_files() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &jpeg(20, 20, [4, 5, 6]));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        db.conn.lock().unwrap().execute("UPDATE artworks SET image_path=NULL,image_media_id=NULL WHERE id='a'", []).unwrap();
        set_state_core(&db, &id, true).unwrap();
        let external = tempfile::tempdir().unwrap();
        fs::write(external.path().join("cover.jpg"), b"external private file").unwrap();
        fs::rename(db.directory.join("media"), db.directory.join("media-original")).unwrap();
        std::os::unix::fs::symlink(external.path(), db.directory.join("media")).unwrap();
        assert!(purge_core(&db, &id).unwrap_err().contains("cartella media"));
        assert_eq!(db.conn.lock().unwrap().query_row(
            "SELECT count(*) FROM media_items WHERE id=?1", params![id], |r| r.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(fs::read(external.path().join("cover.jpg")).unwrap(), b"external private file");
        assert!(read_media_file(&db.directory, &cover).is_err());
    }

    #[test]
    fn search_tags_collections_and_metadata_work_together() {
        let (_dir, db) = setup();
        assert!(import_core(&db, &[]).unwrap().imported.is_empty());
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("Notte estiva.jpg"), dir.path().join("mattina.jpg"));
        fs::write(&a, jpeg(30, 20, [10, 20, 30])).unwrap();
        fs::write(&b, jpeg(30, 20, [200, 200, 20])).unwrap();
        let imported = import_core(&db, &[a.to_string_lossy().into(), b.to_string_lossy().into()]).unwrap();
        assert_eq!(imported.imported.len(), 2);
        let night = imported.imported.iter().find(|i| i.original_name.starts_with("Notte")).unwrap().id.clone();
        let detail = update_core(&db, &night, MediaMetadataInput { title: Some(" Dipinto della Città ".into()), description: Some("olio".into()),
            alt_text: None, credits: Some("Foto: studio".into()), copyright_status: "OWNED".into() }).unwrap();
        assert_eq!((detail.item.title.as_deref(), detail.item.copyright_status.as_str()), (Some("Dipinto della Città"), "OWNED"));
        assert!(update_core(&db, &night, metadata("FREE")).is_err());
        assert!(update_core(&db, "x", metadata("OWNED")).is_err());
        let tag = create_tag_core(&db.conn.lock().unwrap(), "Notturni").unwrap();
        assert_eq!(create_tag_core(&db.conn.lock().unwrap(), "notturni").unwrap().id, tag.id, "tags are unique ignoring case");
        assert!(create_tag_core(&db.conn.lock().unwrap(), "  ").is_err());
        set_tags_core(&db, &night, std::slice::from_ref(&tag.id)).unwrap();
        assert!(set_tags_core(&db, &night, &["sconosciuto".to_string()]).is_err());
        let collection = create_collection_core(&db.conn.lock().unwrap(), "Mostra 2026").unwrap();
        assert!(create_collection_core(&db.conn.lock().unwrap(), "mostra 2026").unwrap_err().contains("già"));
        db.conn.lock().unwrap().execute("INSERT INTO media_collection_items(collection_id,media_id) VALUES(?1,?2)", params![collection.id, night]).unwrap();
        let find = |q: Option<&str>, t: Option<&str>, c: Option<&str>| list_core(&db, q, t, c, "ACTIVE", 0, 50).unwrap().items.len();
        assert_eq!(find(None, None, None), 2);
        assert_eq!(find(Some("citta DIPINTO"), None, None), 1, "case and accents are ignored");
        assert_eq!(find(Some("notturni"), None, None), 1, "tag names are searchable");
        assert_eq!(find(Some("mattina"), None, None), 1);
        assert_eq!(find(None, Some(&tag.id), None), 1);
        assert_eq!(find(None, None, Some(&collection.id)), 1);
        assert_eq!(find(Some("mattina"), Some(&tag.id), None), 0);
        assert_eq!(collections_core(&db.conn.lock().unwrap()).unwrap()[0].item_count, 1);
        assert_eq!(list_core(&db, None, None, None, "ACTIVE", 1, 1).unwrap().items.len(), 1, "paging");
    }

    #[test]
    fn importing_reports_each_unusable_file_and_keeps_the_good_ones() {
        let (_dir, db) = setup();
        let dir = tempfile::tempdir().unwrap();
        let good = dir.path().join("ok.jpg");
        let text = dir.path().join("testo.jpg");
        fs::write(&good, jpeg(8, 8, [1, 1, 1])).unwrap();
        fs::write(&text, b"not a picture").unwrap();
        let result = import_core(&db, &[good.to_string_lossy().into(), text.to_string_lossy().into(), "relativo.jpg".into()]).unwrap();
        assert_eq!((result.imported.len(), result.failed.len()), (1, 2));
        assert_eq!(files(&db), 1, "failed imports leave no file");
    }

    #[test]
    fn exports_never_overwrite_or_write_inside_the_archive() {
        let (dir, db) = setup();
        let cover = put(&db, "cover.jpg", &split_jpeg(60, 30));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        apply_edit_core(&db, &id, &rotate()).unwrap();
        let none = ExportOptions { format: None, quality: None, max_width: None, max_height: None };
        let original = dir.path().join("originale.jpg");
        export_core(&db, &id, "original", &none, &original).unwrap();
        assert_eq!(fs::read(&original).unwrap(), split_jpeg(60, 30));
        let current = dir.path().join("attuale.jpg");
        let reported = export_core(&db, &id, "current", &none, &current).unwrap();
        assert!(!reported.starts_with(r"\\?\"));
        assert_eq!(crate::imaging::dimensions(&fs::read(&current).unwrap()).unwrap(), (30, 60));
        let custom = dir.path().join("web.png");
        export_core(&db, &id, "custom", &ExportOptions { format: Some("png".into()), quality: None, max_width: Some(10), max_height: None }, &custom).unwrap();
        assert!(fs::read(&custom).unwrap().starts_with(&[137, 80, 78, 71]));
        assert_eq!(crate::imaging::dimensions(&fs::read(&custom).unwrap()).unwrap(), (10, 20));
        assert!(export_core(&db, &id, "current", &none, &current).is_err(), "no overwriting");
        assert!(export_core(&db, &id, "custom", &none, &db.directory.join("media").join("x.jpg")).is_err(), "no export into the archive");
        assert!(export_core(&db, &id, "weird", &none, &dir.path().join("z.jpg")).is_err());
    }

    #[test]
    fn identical_and_similar_pictures_are_found() {
        let (_dir, db) = setup();
        let base = split_jpeg(64, 48);
        let add = |id: &str, name: &str, bytes: &[u8]| {
            let path = put(&db, name, bytes);
            db.conn.lock().unwrap().execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at) VALUES(?1,?1,'T',?2,'t','t')", params![id, path]).unwrap();
            path
        };
        add("w0", "a.jpg", &base);
        add("w1", "b.jpg", &base);
        let bright = add("w2", "c.png", &crate::imaging::process(&base, &[Operation::Adjust { exposure: 0.1, contrast: 0.0, saturation: 0.0 }]).unwrap().bytes);
        let other = add("w3", "d.jpg", &jpeg(64, 48, [10, 240, 10]));
        let groups = exact_duplicates_core(&db).unwrap();
        assert_eq!((groups.len(), groups[0].items.len()), (1, 2), "a.jpg and b.jpg are byte-identical");
        let similar = similar_core(&db).unwrap();
        assert!(similar.iter().any(|p| p.first.image_path == bright || p.second.image_path == bright), "the slightly brighter copy is similar");
        assert!(similar.iter().all(|p| p.first.image_path != other && p.second.image_path != other), "an unrelated picture is not");
        assert!(similar.iter().all(|p| p.first.image_path != p.second.image_path));
    }

    #[test]
    fn library_pictures_can_become_a_cover_or_a_gallery_photo_without_losing_protected_files() {
        let (dir, db) = setup();
        let old = put(&db, "old.jpg", &jpeg(20, 20, [1, 1, 1]));
        artwork_with_cover(&db, "a", &old);
        let file = dir.path().join("nuova.jpg");
        fs::write(&file, jpeg(20, 20, [200, 1, 1])).unwrap();
        let imported = import_core(&db, &[file.to_string_lossy().into()]).unwrap().imported;
        let new_id = imported[0].id.clone();
        db.conn.lock().unwrap().execute("INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at) VALUES('k','a','RS-1','2026-11-21','A',?1,'t')",
            params![format!("{{\"image_path\":\"{old}\"}}")]).unwrap();
        artwork_cover_from_media(&db, "a", &new_id).unwrap();
        assert!(db.directory.join(&old).is_file(), "the cover frozen in a certificate is kept");
        let cover: (String, String) = db.conn.lock().unwrap().query_row("SELECT image_path,image_media_id FROM artworks WHERE id='a'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(cover, (imported[0].image_path.clone(), new_id.clone()));
        let second = dir.path().join("galleria.jpg");
        fs::write(&second, jpeg(20, 20, [1, 200, 1])).unwrap();
        let gallery_id = import_core(&db, &[second.to_string_lossy().into()]).unwrap().imported[0].id.clone();
        artwork_gallery_from_media(&db, "a", &gallery_id, Some(" Dettaglio ".into()), Some("DETAIL".into())).unwrap();
        assert!(artwork_gallery_from_media(&db, "a", &gallery_id, None, None).unwrap_err().contains("già usata"));
        assert!(artwork_gallery_from_media(&db, "inesistente", &new_id, None, None).is_err());
        let caption: String = db.conn.lock().unwrap().query_row("SELECT caption FROM artwork_images", [], |r| r.get(0)).unwrap();
        assert_eq!(caption, "Dettaglio");
    }

    #[test]
    fn thumbnails_are_cached_per_revision() {
        let (_dir, db) = setup();
        let cover = put(&db, "cover.jpg", &split_jpeg(800, 400));
        artwork_with_cover(&db, "a", &cover);
        sync_references(&db).unwrap();
        let id = first_item(&db);
        let first = thumbnail_core(&db, &id).unwrap();
        assert!(first.starts_with("data:image/jpeg;base64,"));
        assert_eq!(fs::read_dir(db.directory.join("thumbs")).unwrap().count(), 1);
        assert_eq!(thumbnail_core(&db, &id).unwrap(), first);
        apply_edit_core(&db, &id, &rotate()).unwrap();
        assert_ne!(thumbnail_core(&db, &id).unwrap(), first, "a new revision gets its own thumbnail");
        assert_eq!(fs::read_dir(db.directory.join("thumbs")).unwrap().count(), 2);
        let preview = preview_core(&db, &id, None, 200, &[]).unwrap();
        assert!(preview.width <= 200 && preview.height <= 200);
        assert!(thumbnail_core(&db, "sconosciuto").is_err());
    }
}
