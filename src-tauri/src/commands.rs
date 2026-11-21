use crate::{db::Database, models::*};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};use std::{fs, path::Component};
use tauri::State;
use uuid::Uuid;

fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn now() -> String { Utc::now().to_rfc3339() }
fn clean(value: Option<String>, max: usize) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        Some(v) if v.trim().is_empty() => Ok(None),
        Some(v) if v.chars().count() <= max => Ok(Some(v.trim().to_string())),
        Some(_) => Err(format!("Testo troppo lungo (massimo {max} caratteri)")),
    }
}
fn required(v: String, max: usize, label: &str) -> Result<String, String> {
    clean(Some(v), max)?.ok_or_else(|| format!("{label} obbligatorio"))
}
fn dim(value: Option<f64>) -> Result<Option<f64>, String> {
    if value.is_some_and(|v| !v.is_finite() || v <= 0.0 || v > 100_000.0) {
        return Err("Le dimensioni devono essere valori positivi validi".into());
    }
    Ok(value)
}
fn artwork_row(row: &Row<'_>) -> rusqlite::Result<Artwork> {
    Ok(Artwork {
        id: row.get(0)?, code: row.get(1)?, title: row.get(2)?, year: row.get(3)?,
        technique: row.get(4)?, support: row.get(5)?, width_cm: row.get(6)?,
        height_cm: row.get(7)?, depth_cm: row.get(8)?, description: row.get(9)?,
        location: row.get(10)?, status: row.get(11)?, image_path: row.get(12)?,
        created_at: row.get(13)?, updated_at: row.get(14)?,
    })
}
const ARTWORK_FIELDS: &str =
    "id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,location,status,image_path,created_at,updated_at";
fn artwork_by_id(conn: &Connection, id: &str) -> Result<Artwork, String> {
    conn.query_row(
        &format!("SELECT {ARTWORK_FIELDS} FROM artworks WHERE id=?1"),
        params![id], artwork_row,
    ).optional().map_err(err)?.ok_or_else(|| "Opera non trovata".to_string())
}
fn collector_row(row: &Row<'_>) -> rusqlite::Result<Collector> {
    Ok(Collector {
        id: row.get(0)?, name: row.get(1)?, email: row.get(2)?, phone: row.get(3)?,
        address: row.get(4)?, notes: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)?,
    })
}
const COLLECTOR_FIELDS: &str = "id,name,email,phone,address,notes,created_at,updated_at";

#[tauri::command]
pub fn list_artworks(db: State<'_, Database>) -> Result<Vec<Artwork>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let mut statement = conn.prepare(&format!("SELECT {ARTWORK_FIELDS} FROM artworks ORDER BY updated_at DESC, title COLLATE NOCASE")).map_err(err)?;
    let items = statement.query_map([], artwork_row).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(items)
}
fn save_artwork_core(db: &Database, input: ArtworkInput) -> Result<Artwork, String> {
    let code = required(input.code, 80, "Codice")?;
    let title = required(input.title, 240, "Titolo")?;
    let year = clean(input.year, 40)?;
    let technique = clean(input.technique, 300)?;
    let support = clean(input.support, 300)?;
    let width = dim(input.width_cm)?;
    let height = dim(input.height_cm)?;
    let depth = dim(input.depth_cm)?;
    let description = clean(input.description, 40_000)?;
    let location = clean(input.location, 300)?;
    if !["available", "reserved", "sold", "archived"].contains(&input.status.as_str()) {
        return Err("Stato opera non valido".into());
    }
    let conn = db.conn.lock().map_err(err)?;
    let editing = input.id.is_some();
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let existing: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    if editing && !existing { return Err("Opera da modificare non trovata".into()); }
    let sale_exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM sales WHERE artwork_id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    if input.status == "sold" && !sale_exists { return Err("Registra una vendita per indicare l'opera come venduta".into()); }
    if input.status != "sold" && sale_exists { return Err("L'opera ha una vendita registrata e deve restare venduta".into()); }
    let code_taken: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM artworks WHERE code=?1 COLLATE NOCASE AND id<>?2)",
        params![code, id], |r| r.get(0)).map_err(err)?;
    if code_taken { return Err("Esiste già un'opera con questo codice".into()); }
    let stamp = now();
    if existing {
        conn.execute("UPDATE artworks SET code=?2,title=?3,year=?4,technique=?5,support=?6,width_cm=?7,height_cm=?8,depth_cm=?9,description=?10,location=?11,status=?12,updated_at=?13 WHERE id=?1",
            params![id,code,title,year,technique,support,width,height,depth,description,location,input.status,stamp]).map_err(err)?;
    } else {
        conn.execute("INSERT INTO artworks(id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,location,status,created_at,updated_at)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13)",
            params![id,code,title,year,technique,support,width,height,depth,description,location,input.status,stamp]).map_err(err)?;
    }
    artwork_by_id(&conn, &id)
}
#[tauri::command]
pub fn save_artwork(db: State<'_, Database>, input: ArtworkInput) -> Result<Artwork, String> {
    save_artwork_core(&db, input)
}
#[tauri::command]
pub fn delete_artwork(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let old_image = artwork_by_id(&conn, &id)?.image_path;
    // ON DELETE CASCADE removes gallery rows; clean their local files only after SQL succeeds.
    let gallery_paths: Vec<String> = {
        let mut statement = conn.prepare("SELECT image_path FROM artwork_images WHERE artwork_id=?1").map_err(err)?;
        // Collect while the prepared statement is still alive (rusqlite Rows borrows it).
        let paths = statement.query_map(params![id], |r| r.get(0)).map_err(err)?
            .collect::<Result<Vec<_>,_>>().map_err(err)?;
        paths
    };
    let affected = conn.execute("DELETE FROM artworks WHERE id=?1",params![id]).map_err(|_|"Non puoi eliminare un'opera collegata a vendite o certificati".to_string())?;
    if affected == 0 { return Err("Opera non trovata".into()); }
    if let Some(path) = old_image {
        if safe_relative(&path) { let _ = fs::remove_file(db.directory.join(path)); }
    }
    for path in gallery_paths {
        if safe_relative(&path) { let _ = fs::remove_file(db.directory.join(path)); }
    }
    Ok(())
}

#[tauri::command]
pub fn list_collectors(db: State<'_, Database>) -> Result<Vec<Collector>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let mut statement = conn.prepare(&format!("SELECT {COLLECTOR_FIELDS} FROM collectors ORDER BY name COLLATE NOCASE")).map_err(err)?;
    let items = statement.query_map([], collector_row).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(items)
}
fn save_collector_core(db: &Database, input: CollectorInput) -> Result<Collector, String> {
    let name = required(input.name, 240, "Nome")?;
    let email = clean(input.email, 320)?;
    let phone = clean(input.phone, 100)?;
    let address = clean(input.address, 1000)?;
    let notes = clean(input.notes, 15_000)?;
    let conn = db.conn.lock().map_err(err)?;
    let editing = input.id.is_some();
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let stamp = now();
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM collectors WHERE id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    if editing && !exists { return Err("Collezionista da modificare non trovato".into()); }
    if exists {
        conn.execute("UPDATE collectors SET name=?2,email=?3,phone=?4,address=?5,notes=?6,updated_at=?7 WHERE id=?1",
            params![id,name,email,phone,address,notes,stamp]).map_err(err)?;
    } else {
        conn.execute("INSERT INTO collectors(id,name,email,phone,address,notes,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?7)",
            params![id,name,email,phone,address,notes,stamp]).map_err(err)?;
    }
    conn.query_row(&format!("SELECT {COLLECTOR_FIELDS} FROM collectors WHERE id=?1"),params![id],collector_row).map_err(err)
}
#[tauri::command]
pub fn save_collector(db: State<'_, Database>, input: CollectorInput) -> Result<Collector, String> {
    save_collector_core(&db, input)
}
#[tauri::command]
pub fn delete_collector(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn=db.conn.lock().map_err(err)?;
    let affected=conn.execute("DELETE FROM collectors WHERE id=?1",params![id])
        .map_err(|_| "Non puoi eliminare un collezionista collegato a una vendita".to_string())?;
    if affected==0 { return Err("Collezionista non trovato".into()); }
    Ok(())
}
fn sale_row(row: &Row<'_>) -> rusqlite::Result<Sale> {
    Ok(Sale { id:row.get(0)?, artwork_id:row.get(1)?, collector_id:row.get(2)?,
        amount_cents:row.get(3)?,currency:row.get(4)?,sold_at:row.get(5)?, notes:row.get(6)?,
        artwork_title:row.get(7)?,collector_name:row.get(8)?,created_at:row.get(9)? })
}
#[tauri::command]
pub fn list_sales(db: State<'_, Database>) -> Result<Vec<Sale>, String> {
    let conn=db.conn.lock().map_err(err)?;
    let mut statement=conn.prepare("SELECT s.id,s.artwork_id,s.collector_id,s.amount_cents,s.currency,s.sold_at,s.notes,a.title,c.name,s.created_at
        FROM sales s JOIN artworks a ON a.id=s.artwork_id JOIN collectors c ON c.id=s.collector_id ORDER BY s.sold_at DESC, s.created_at DESC").map_err(err)?;
    let items = statement.query_map([], sale_row).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(items)
}
// Dates are stored as text and sorted/grouped lexicographically: accept only canonical YYYY-MM-DD.
fn canonical_date(value: &str) -> bool {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|d| d.format("%Y-%m-%d").to_string() == value)
        .unwrap_or(false)
}
#[tauri::command]
pub fn record_sale(db: State<'_, Database>, input: SaleInput) -> Result<(), String> {
    record_sale_core(&db, input)
}
fn record_sale_core(db: &Database, input: SaleInput) -> Result<(), String> {
    if input.amount_cents<=0 || input.amount_cents>100_000_000_000 { return Err("Importo non valido".into()); }
    if !canonical_date(&input.sold_at) { return Err("Data di vendita non valida".into()); }
    let notes=clean(input.notes,15_000)?;
    let mut conn=db.conn.lock().map_err(err)?;
    let tx=conn.transaction().map_err(err)?;
    let status:Option<String>=tx.query_row("SELECT status FROM artworks WHERE id=?1",params![input.artwork_id],|r|r.get(0)).optional().map_err(err)?;
    let status=status.ok_or_else(||"Opera non trovata".to_string())?;
    if status=="sold" || status=="archived" { return Err("Quest'opera non è disponibile per una vendita".into()); }
    let sale_id=Uuid::new_v4().to_string();
    tx.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,currency,sold_at,notes,created_at) VALUES(?1,?2,?3,?4,'EUR',?5,?6,?7)",
        params![sale_id, input.artwork_id, input.collector_id,input.amount_cents,input.sold_at,notes,now()])
        .map_err(|_|"Vendita non registrata: verifica cliente, opera e vendite esistenti".to_string())?;
    crate::finance::attach_sale_finances(&tx,&sale_id,input.amount_cents,input.tax_cents,input.shipping_cents)?;
    tx.execute("UPDATE artworks SET status='sold',updated_at=?2 WHERE id=?1",params![input.artwork_id,now()]).map_err(err)?;
    tx.commit().map_err(err)
}
#[tauri::command]
pub fn delete_sale(db: State<'_, Database>, id: String) -> Result<(), String> {
    delete_sale_core(&db, &id)
}
fn delete_sale_core(db: &Database, id: &str) -> Result<(), String> {
    let mut conn=db.conn.lock().map_err(err)?;
    let tx=conn.transaction().map_err(err)?;
    let artwork:Option<String>=tx.query_row("SELECT artwork_id FROM sales WHERE id=?1",params![id],|r|r.get(0)).optional().map_err(err)?;
    let artwork=artwork.ok_or_else(||"Vendita non trovata".to_string())?;
    // Registered collections are financial records: never cascade-hide them behind a raw FK error.
    let receipts:i64=tx.query_row("SELECT count(*) FROM sale_receipts WHERE sale_id=?1",params![id],|r|r.get(0)).map_err(err)?;
    if receipts>0 {
        return Err("La vendita ha incassi registrati: elimina prima gli incassi (e i relativi allegati)".into());
    }
    tx.execute("DELETE FROM sales WHERE id=?1",params![id]).map_err(err)?;
    tx.execute("UPDATE artworks SET status='available',updated_at=?2 WHERE id=?1",params![artwork,now()]).map_err(err)?;
    tx.commit().map_err(err)
}
fn certificate_row(row: &Row<'_>) -> rusqlite::Result<Certificate> {
    Ok(Certificate { id:row.get(0)?,artwork_id:row.get(1)?,serial:row.get(2)?,issued_at:row.get(3)?,
        artist_name:row.get(4)?,collector_name:row.get(5)?,notes:row.get(6)?,
        snapshot_json:row.get(7)?,artwork_title:row.get(8)?,created_at:row.get(9)? })
}
#[tauri::command]
pub fn list_certificates(db: State<'_, Database>) -> Result<Vec<Certificate>, String> {
    let conn=db.conn.lock().map_err(err)?;
    let mut statement=conn.prepare("SELECT c.id,c.artwork_id,c.serial,c.issued_at,c.artist_name,c.collector_name,c.notes,c.snapshot_json,a.title,c.created_at
        FROM certificates c JOIN artworks a ON a.id=c.artwork_id ORDER BY c.created_at DESC").map_err(err)?;
    let items = statement.query_map([], certificate_row).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(items)
}
#[tauri::command]
pub fn issue_certificate(db: State<'_, Database>, input: CertificateInput) -> Result<Certificate, String> {
    let artist_name=required(input.artist_name,240,"Nome dell'artista")?;
    let collector_name=clean(input.collector_name,240)?;
    let notes=clean(input.notes,10_000)?;
    let mut conn=db.conn.lock().map_err(err)?;
    let tx=conn.transaction().map_err(err)?;
    let artwork=artwork_by_id(&tx,&input.artwork_id)?;
    let snapshot_json=serde_json::to_string(&artwork).map_err(err)?;
    tx.execute("UPDATE certificate_counter SET last_number=last_number+1 WHERE singleton=1",[]).map_err(err)?;
    let number:i64=tx.query_row("SELECT last_number FROM certificate_counter WHERE singleton=1",[],|r|r.get(0)).map_err(err)?;
    let serial=format!("RS-{}-{number:06}",Utc::now().year());
    let id=Uuid::new_v4().to_string();
    let stamp=now();
    let issued_at=Utc::now().format("%Y-%m-%d").to_string();
    tx.execute("INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,collector_name,notes,snapshot_json,created_at)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![id,input.artwork_id,serial,issued_at,artist_name,collector_name,notes,snapshot_json,stamp]).map_err(err)?;
    let cert=tx.query_row("SELECT c.id,c.artwork_id,c.serial,c.issued_at,c.artist_name,c.collector_name,c.notes,c.snapshot_json,a.title,c.created_at FROM certificates c JOIN artworks a ON a.id=c.artwork_id WHERE c.id=?1",
        params![id],certificate_row).map_err(err)?;
    tx.commit().map_err(err)?;
    Ok(cert)
}
#[tauri::command]
pub fn overview(db: State<'_, Database>) -> Result<Overview, String> {
    let conn=db.conn.lock().map_err(err)?;
    let count=|table:&str| -> Result<i64,String> {
        conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get(0)).map_err(err)
    };
    let revenue_cents=conn.query_row("SELECT COALESCE(SUM(amount_cents),0) FROM sales",[],|r|r.get(0)).map_err(err)?;
    Ok(Overview { artworks:count("artworks")?,collectors:count("collectors")?,sales:count("sales")?,
        certificates:count("certificates")?,revenue_cents })
}
#[tauri::command]
pub fn data_directory(db: State<'_, Database>) -> String {
    db.directory.to_string_lossy().into_owned()
}
// Certificates freeze the artwork's image_path in snapshot_json. A later cover
// replacement must not destroy that historical photograph. Malformed legacy
// snapshots are treated conservatively: retain the file instead of risking loss.
pub(crate) fn image_used_by_certificate(conn: &Connection, artwork_id: &str, image_path: &str) -> Result<bool, String> {
    let mut statement = conn.prepare("SELECT snapshot_json FROM certificates WHERE artwork_id=?1").map_err(err)?;
    let snapshots = statement.query_map(params![artwork_id], |row| row.get::<_, String>(0)).map_err(err)?;
    for snapshot in snapshots {
        let snapshot = snapshot.map_err(err)?;
        match serde_json::from_str::<serde_json::Value>(&snapshot) {
            Ok(value) if value.get("image_path").and_then(|path| path.as_str()) == Some(image_path) => {
                return Ok(true);
            }
            Err(_) => return Ok(true),
            _ => {}
        }
    }
    Ok(false)
}

fn save_artwork_image_core(db: &Database, artwork_id: &str, data_base64: &str) -> Result<(), String> {
    if data_base64.len()>16_800_000 { return Err("Immagine troppo grande (massimo 12 MiB)".into()); }
    let bytes=STANDARD.decode(data_base64).map_err(|_|"Immagine non valida".to_string())?;
    if bytes.len()>12*1024*1024 || bytes.is_empty() { return Err("Immagine troppo grande o vuota".into()); }
    let (ext,_mime)=image_type(&bytes).ok_or_else(||"Sono consentite solo immagini JPEG, PNG e WebP".to_string())?;
    let conn=db.conn.lock().map_err(err)?;
    let old=artwork_by_id(&conn,artwork_id)?.image_path;
    // Resolve historical dependencies before touching SQLite or the filesystem.
    let preserve_old = match old.as_deref() {
        Some(path) => image_used_by_certificate(&conn, artwork_id, path)?,
        None => false,
    };
    let name=format!("{}.{ext}",Uuid::new_v4());
    let relative=format!("media/{name}");
    let target=db.directory.join(&relative);
    let tmp=db.directory.join(format!("media/{name}.tmp"));
    fs::write(&tmp,&bytes).map_err(err)?;
    if let Err(e)=fs::rename(&tmp,&target) { let _=fs::remove_file(&tmp); return Err(err(e)); }
    if let Err(e)=conn.execute("UPDATE artworks SET image_path=?2,updated_at=?3 WHERE id=?1",params![artwork_id,relative,now()]) {
        let _=fs::remove_file(&target); return Err(err(e));
    }
    if let Some(old_path)=old {
        if !preserve_old && safe_relative(&old_path) {
            let _=fs::remove_file(db.directory.join(old_path));
        }
    }
    Ok(())
}
#[tauri::command]
pub fn save_artwork_image(db: State<'_, Database>, artwork_id: String, data_base64: String) -> Result<(), String> {
    save_artwork_image_core(&db, &artwork_id, &data_base64)
}
fn safe_relative(path: &str) -> bool {
    let p=std::path::Path::new(path);
    p.components().all(|c|matches!(c,Component::Normal(_))) && p.starts_with("media")
}
fn image_type(bytes:&[u8])->Option<(&'static str,&'static str)> {
    if bytes.starts_with(&[0xff,0xd8,0xff]) { return Some(("jpg","image/jpeg")); }
    if bytes.starts_with(&[137,80,78,71,13,10,26,10]) { return Some(("png","image/png")); }
    if bytes.len()>=12 && &bytes[0..4]==b"RIFF" && &bytes[8..12]==b"WEBP" { return Some(("webp","image/webp")); }
    None
}
#[tauri::command]
pub fn artwork_image_data(db: State<'_, Database>, artwork_id: String) -> Result<Option<String>, String> {
    let conn=db.conn.lock().map_err(err)?;
    let Some(relative)=artwork_by_id(&conn,&artwork_id)?.image_path else {return Ok(None);};
    if !safe_relative(&relative) {return Err("Percorso media non valido".into());}
    let bytes=fs::read(db.directory.join(relative)).map_err(err)?;
    let (_ext,mime)=image_type(&bytes).ok_or_else(||"Formato file non valido".to_string())?;
    Ok(Some(format!("data:{mime};base64,{}",STANDARD.encode(bytes))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_images_and_paths() {
        assert!(image_type(b"not an image").is_none());
        assert_eq!(image_type(&[0xff,0xd8,0xff,0]),Some(("jpg","image/jpeg")));
        assert!(!safe_relative("media/../secret"));
        assert!(!safe_relative("/tmp/secret"));
        assert!(safe_relative("media/abc.png"));
        assert!(dim(Some(f64::NAN)).is_err());
    }

    fn artwork_with_cover() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        db.conn.lock().unwrap().execute(
            "INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES('a','A-1','Test','t','t')",
            [],
        ).unwrap();
        let first = STANDARD.encode([0xff, 0xd8, 0xff, 0x01]);
        save_artwork_image_core(&db, "a", &first).unwrap();
        (dir, db)
    }

    fn cover_path(db: &Database) -> String {
        let conn = db.conn.lock().unwrap();
        artwork_by_id(&conn, "a").unwrap().image_path.unwrap()
    }

    #[test]
    fn replacement_preserves_historical_certificate_photo_and_snapshot() {
        let (_dir, db) = artwork_with_cover();
        let original = cover_path(&db);
        let snapshot = {
            let conn = db.conn.lock().unwrap();
            serde_json::to_string(&artwork_by_id(&conn, "a").unwrap()).unwrap()
        };
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                 VALUES('cert','a','RS-1','2026-11-21','Artist',?1,'t')",
                params![snapshot],
            ).unwrap();
        }
        save_artwork_image_core(&db, "a", &STANDARD.encode([0xff, 0xd8, 0xff, 0x02])).unwrap();
        assert_ne!(cover_path(&db), original);
        assert!(db.directory.join(&original).is_file(), "historical file must survive");
        let conn = db.conn.lock().unwrap();
        let stored: String = conn.query_row(
            "SELECT snapshot_json FROM certificates WHERE id='cert'", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(stored, snapshot);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&stored).unwrap()["image_path"].as_str(),
            Some(original.as_str())
        );
        drop(conn);
        // Backups copy the complete local media store, including historical photos
        // no longer referenced by the active artwork or gallery.
        let destination = tempfile::tempdir().unwrap();
        let backup = crate::backup::create_backup_core(&db, destination.path()).unwrap();
        let value = serde_json::to_value(backup).unwrap();
        let backup_dir = value["directory"].as_str().unwrap();
        assert!(std::path::Path::new(backup_dir).join(original).is_file());
    }

    #[test]
    fn replacement_removes_unreferenced_previous_photo() {
        let (_dir, db) = artwork_with_cover();
        let original = cover_path(&db);
        save_artwork_image_core(&db, "a", &STANDARD.encode([0xff, 0xd8, 0xff, 0x03])).unwrap();
        assert_ne!(cover_path(&db), original);
        assert!(!db.directory.join(original).exists());
    }

    #[test]
    fn malformed_historical_snapshot_does_not_trigger_photo_deletion() {
        let (_dir, db) = artwork_with_cover();
        let original = cover_path(&db);
        db.conn.lock().unwrap().execute(
            "INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
             VALUES('bad','a','RS-2','2026-11-21','Artist','{invalid','t')",
            [],
        ).unwrap();
        save_artwork_image_core(&db, "a", &STANDARD.encode([0xff, 0xd8, 0xff, 0x04])).unwrap();
        assert!(db.directory.join(original).is_file());
    }

    fn artwork_edit_input(id: Option<String>, title: &str) -> ArtworkInput {
        ArtworkInput {
            id, code: "WORK-STALE-01".into(), title: title.into(), year: None,
            technique: None, support: None, width_cm: None, height_cm: None,
            depth_cm: None, description: None, location: None,
            status: "available".into(),
        }
    }

    fn collector_edit_input(id: Option<String>, name: &str) -> CollectorInput {
        CollectorInput {
            id, name: name.into(), email: None, phone: None,
            address: None, notes: None,
        }
    }

    #[test]
    fn artwork_edit_never_recreates_a_deleted_record() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let created = save_artwork_core(&db, artwork_edit_input(None, "Original")).unwrap();
        let changed = save_artwork_core(&db,
            artwork_edit_input(Some(created.id.clone()), "Edited")).unwrap();
        assert_eq!(changed.id, created.id);
        assert_eq!(changed.title, "Edited");
        db.conn.lock().unwrap().execute("DELETE FROM artworks WHERE id=?1",
            params![created.id]).unwrap();
        let error = save_artwork_core(&db,
            artwork_edit_input(Some(created.id.clone()), "Resurrected")).unwrap_err();
        assert!(error.contains("non trovata"));
        let count: i64 = db.conn.lock().unwrap().query_row(
            "SELECT count(*) FROM artworks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 0, "a stale editor must not silently insert an artwork");
    }

    fn sale_fixture() -> (tempfile::TempDir, Database, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let artwork = save_artwork_core(&db, artwork_edit_input(None, "Opera")).unwrap();
        let collector = save_collector_core(&db, collector_edit_input(None, "Persona")).unwrap();
        (dir, db, artwork.id, collector.id)
    }

    fn sale_input(artwork: &str, collector: &str, date: &str) -> SaleInput {
        SaleInput {
            artwork_id: artwork.into(), collector_id: collector.into(), amount_cents: 10_000,
            sold_at: date.into(), notes: None, tax_cents: 0, shipping_cents: 0,
        }
    }

    #[test]
    fn duplicate_artwork_code_gets_a_clear_message_for_new_and_edited_records() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let first = save_artwork_core(&db, artwork_edit_input(None, "Prima")).unwrap();
        let error = save_artwork_core(&db, artwork_edit_input(None, "Seconda")).unwrap_err();
        assert!(error.contains("codice"), "{error}");
        let mut other = artwork_edit_input(None, "Altra");
        other.code = "WORK-OTHER".into();
        let other = save_artwork_core(&db, other).unwrap();
        let mut clash = artwork_edit_input(Some(other.id.clone()), "Altra");
        clash.code = first.code.to_lowercase();
        assert!(save_artwork_core(&db, clash).unwrap_err().contains("codice"));
        // Re-saving a record with its own code is not a conflict.
        let mut same = artwork_edit_input(Some(first.id.clone()), "Prima modificata");
        same.code = first.code.clone();
        assert_eq!(save_artwork_core(&db, same).unwrap().title, "Prima modificata");
    }

    #[test]
    fn sale_dates_must_be_canonical_iso_dates() {
        let (_dir, db, artwork, collector) = sale_fixture();
        for bad in ["2026-1-5", "2026-02-30", "21/11/2026", "2026-11-21T10:00:00", " 2026-11-21", ""] {
            let error = record_sale_core(&db, sale_input(&artwork, &collector, bad)).unwrap_err();
            assert!(error.contains("Data"), "{bad}: {error}");
        }
        record_sale_core(&db, sale_input(&artwork, &collector, "2026-11-21")).unwrap();
        let conn = db.conn.lock().unwrap();
        let status: String = conn.query_row("SELECT status FROM artworks WHERE id=?1",
            params![artwork], |r| r.get(0)).unwrap();
        assert_eq!(status, "sold");
    }

    #[test]
    fn sale_with_recorded_receipts_cannot_be_cancelled_and_stays_intact() {
        let (_dir, db, artwork, collector) = sale_fixture();
        record_sale_core(&db, sale_input(&artwork, &collector, "2026-11-21")).unwrap();
        let sale_id: String = db.conn.lock().unwrap()
            .query_row("SELECT id FROM sales", [], |r| r.get(0)).unwrap();
        db.conn.lock().unwrap().execute(
            "INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
             VALUES('r',?1,'2026-11-21',500,'cash','t')", params![sale_id]).unwrap();
        let error = delete_sale_core(&db, &sale_id).unwrap_err();
        assert!(error.contains("incassi"), "{error}");
        let conn = db.conn.lock().unwrap();
        let sales: i64 = conn.query_row("SELECT count(*) FROM sales", [], |r| r.get(0)).unwrap();
        let status: String = conn.query_row("SELECT status FROM artworks WHERE id=?1",
            params![artwork], |r| r.get(0)).unwrap();
        assert_eq!((sales, status.as_str()), (1, "sold"));
        conn.execute("DELETE FROM sale_receipts WHERE id='r'", []).unwrap();
        drop(conn);
        delete_sale_core(&db, &sale_id).unwrap();
        let conn = db.conn.lock().unwrap();
        let status: String = conn.query_row("SELECT status FROM artworks WHERE id=?1",
            params![artwork], |r| r.get(0)).unwrap();
        assert_eq!(status, "available");
    }

    #[test]
    fn collector_edit_never_recreates_a_deleted_record() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let created = save_collector_core(&db, collector_edit_input(None, "Original")).unwrap();
        let changed = save_collector_core(&db,
            collector_edit_input(Some(created.id.clone()), "Edited")).unwrap();
        assert_eq!(changed.id, created.id);
        assert_eq!(changed.name, "Edited");
        db.conn.lock().unwrap().execute("DELETE FROM collectors WHERE id=?1",
            params![created.id]).unwrap();
        let error = save_collector_core(&db,
            collector_edit_input(Some(created.id.clone()), "Resurrected")).unwrap_err();
        assert!(error.contains("non trovato"));
        let count: i64 = db.conn.lock().unwrap().query_row(
            "SELECT count(*) FROM collectors", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 0, "a stale editor must not silently insert a collector");
    }
}
