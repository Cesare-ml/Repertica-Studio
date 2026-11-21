use crate::{db::Database, models::*};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::{fs, path::Component};
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
#[tauri::command]
pub fn save_artwork(db: State<'_, Database>, input: ArtworkInput) -> Result<Artwork, String> {
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
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let existing: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    let sale_exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM sales WHERE artwork_id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    if input.status == "sold" && !sale_exists { return Err("Registra una vendita per indicare l'opera come venduta".into()); }
    if input.status != "sold" && sale_exists { return Err("L'opera ha una vendita registrata e deve restare venduta".into()); }
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
pub fn delete_artwork(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let old_image = artwork_by_id(&conn, &id)?.image_path;
    // ON DELETE CASCADE removes gallery rows; clean their local files only after SQL succeeds.
    let gallery_paths: Vec<String> = {
        let mut statement = conn.prepare("SELECT image_path FROM artwork_images WHERE artwork_id=?1").map_err(err)?;
        statement.query_map(params![id], |r| r.get(0)).map_err(err)?
            .collect::<Result<Vec<_>,_>>().map_err(err)?
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
#[tauri::command]
pub fn save_collector(db: State<'_, Database>, input: CollectorInput) -> Result<Collector, String> {
    let name = required(input.name, 240, "Nome")?;
    let email = clean(input.email, 320)?;
    let phone = clean(input.phone, 100)?;
    let address = clean(input.address, 1000)?;
    let notes = clean(input.notes, 15_000)?;
    let conn = db.conn.lock().map_err(err)?;
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let stamp = now();
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM collectors WHERE id=?1)",params![id],|r|r.get(0)).map_err(err)?;
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
#[tauri::command]
pub fn record_sale(db: State<'_, Database>, input: SaleInput) -> Result<(), String> {
    if input.amount_cents<=0 || input.amount_cents>100_000_000_000 { return Err("Importo non valido".into()); }
    NaiveDate::parse_from_str(&input.sold_at,"%Y-%m-%d").map_err(|_|"Data di vendita non valida".to_string())?;
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
    let mut conn=db.conn.lock().map_err(err)?;
    let tx=conn.transaction().map_err(err)?;
    let artwork:Option<String>=tx.query_row("SELECT artwork_id FROM sales WHERE id=?1",params![id],|r|r.get(0)).optional().map_err(err)?;
    let artwork=artwork.ok_or_else(||"Vendita non trovata".to_string())?;
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
#[tauri::command]
pub fn save_artwork_image(db: State<'_, Database>, artwork_id: String, data_base64: String) -> Result<(), String> {
    if data_base64.len()>16_800_000 { return Err("Immagine troppo grande (massimo 12 MiB)".into()); }
    let bytes=STANDARD.decode(&data_base64).map_err(|_|"Immagine non valida".to_string())?;
    if bytes.len()>12*1024*1024 || bytes.is_empty() { return Err("Immagine troppo grande o vuota".into()); }
    let (ext,_mime)=image_type(&bytes).ok_or_else(||"Sono consentite solo immagini JPEG, PNG e WebP".to_string())?;
    let conn=db.conn.lock().map_err(err)?;
    let old=artwork_by_id(&conn,&artwork_id)?.image_path;
    let name=format!("{}.{ext}",Uuid::new_v4());
    let relative=format!("media/{name}");
    let target=db.directory.join(&relative);
    let tmp=db.directory.join(format!("media/{name}.tmp"));
    fs::write(&tmp,&bytes).map_err(err)?;
    if let Err(e)=fs::rename(&tmp,&target) { let _=fs::remove_file(&tmp); return Err(err(e)); }
    if let Err(e)=conn.execute("UPDATE artworks SET image_path=?2,updated_at=?3 WHERE id=?1",params![artwork_id,relative,now()]) {
        let _=fs::remove_file(&target); return Err(err(e));
    }
    if let Some(old_path)=old { if safe_relative(&old_path) { let _=fs::remove_file(db.directory.join(old_path)); } }
    Ok(())
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
}
