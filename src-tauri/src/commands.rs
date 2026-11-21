use crate::{db::Database, models::*};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Datelike, Local, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};use std::{fs, path::Component};
use tauri::{Manager, State};
use uuid::Uuid;

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn now() -> String { Utc::now().to_rfc3339() }
pub(crate) fn clean(value: Option<String>, max: usize) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        Some(v) if v.trim().is_empty() => Ok(None),
        Some(v) if v.chars().count() <= max => Ok(Some(v.trim().to_string())),
        Some(_) => Err(format!("Testo troppo lungo (massimo {max} caratteri)")),
    }
}
pub(crate) fn required(v: String, max: usize, label: &str) -> Result<String, String> {
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
        signed: row.get(15)?, final_varnish: row.get(16)?, framed: row.get(17)?,
    })
}
const ARTWORK_FIELDS: &str =
    "id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,location,status,image_path,created_at,updated_at,signed,final_varnish,framed";
fn artwork_by_id(conn: &Connection, id: &str) -> Result<Artwork, String> {
    conn.query_row(
        &format!("SELECT {ARTWORK_FIELDS} FROM artworks WHERE id=?1"),
        params![id], artwork_row,
    ).optional().map_err(err)?.ok_or_else(|| "Opera non trovata".to_string())
}
#[tauri::command]
pub fn list_artworks(db: State<'_, Database>) -> Result<Vec<Artwork>, String> {
    let conn = db.conn.lock().map_err(err)?;
    let mut statement = conn.prepare(&format!("SELECT {ARTWORK_FIELDS} FROM artworks ORDER BY updated_at DESC, title COLLATE NOCASE")).map_err(err)?;
    let items = statement.query_map([], artwork_row).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
    Ok(items)
}
struct ArtworkFields {
    code: Option<String>, title: String, year: Option<String>, technique: Option<String>,
    support: Option<String>, width: Option<f64>, height: Option<f64>, depth: Option<f64>,
    description: Option<String>, location: Option<String>, status: String,
    signed: bool, final_varnish: bool, framed: bool,
}
fn normalize_artwork(input: &ArtworkInput) -> Result<ArtworkFields, String> {
    if !["available", "reserved", "sold", "archived"].contains(&input.status.as_str()) {
        return Err("Stato opera non valido".into());
    }
    Ok(ArtworkFields {
        code: clean(input.code.clone(), 80)?,
        title: required(input.title.clone(), 240, "Titolo")?,
        year: clean(input.year.clone(), 40)?,
        technique: clean(input.technique.clone(), 300)?,
        support: clean(input.support.clone(), 300)?,
        width: dim(input.width_cm)?, height: dim(input.height_cm)?, depth: dim(input.depth_cm)?,
        description: clean(input.description.clone(), 40_000)?,
        location: clean(input.location.clone(), 300)?,
        status: input.status.clone(),
        signed: input.signed, final_varnish: input.final_varnish, framed: input.framed,
    })
}
fn code_in_use(conn: &Connection, code: &str, except_id: &str) -> Result<bool, String> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE code=?1 COLLATE NOCASE AND id<>?2)",
        params![code, except_id], |r| r.get(0)).map_err(err)
}
fn save_artwork_core(db: &Database, input: ArtworkInput) -> Result<Artwork, String> {
    let fields = normalize_artwork(&input)?;
    let mut conn = db.conn.lock().map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let editing = input.id.is_some();
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let existing: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    if editing && !existing { return Err("Opera da modificare non trovata".into()); }
    let sale_exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM sales WHERE artwork_id=?1)",params![id],|r|r.get(0)).map_err(err)?;
    if fields.status == "sold" && !sale_exists { return Err("Registra una vendita per indicare l'opera come venduta".into()); }
    if fields.status != "sold" && sale_exists { return Err("L'opera ha una vendita registrata e deve restare venduta".into()); }
    let code = match fields.code {
        Some(code) => code,
        None if existing => artwork_by_id(&tx, &id)?.code,
        None => crate::archive_code::allocate(&tx, fields.year.as_deref())?,
    };
    if code_in_use(&tx, &code, &id)? { return Err("Esiste già un'opera con questo codice".into()); }
    let stamp = now();
    let (title, year, technique, support, description, location) =
        (fields.title, fields.year, fields.technique, fields.support, fields.description, fields.location);
    if existing {
        tx.execute("UPDATE artworks SET code=?2,title=?3,year=?4,technique=?5,support=?6,width_cm=?7,height_cm=?8,depth_cm=?9,description=?10,location=?11,status=?12,updated_at=?13,signed=?14,final_varnish=?15,framed=?16 WHERE id=?1",
            params![id,code,title,year,technique,support,fields.width,fields.height,fields.depth,description,location,fields.status,stamp,fields.signed,fields.final_varnish,fields.framed]).map_err(err)?;
    } else {
        tx.execute("INSERT INTO artworks(id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,location,status,created_at,updated_at,signed,final_varnish,framed)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13,?14,?15,?16)",
            params![id,code,title,year,technique,support,fields.width,fields.height,fields.depth,description,location,fields.status,stamp,fields.signed,fields.final_varnish,fields.framed]).map_err(err)?;
    }
    let saved = artwork_by_id(&tx, &id)?;
    tx.commit().map_err(err)?;
    Ok(saved)
}
/// Write a picture into the media store (temporary file, then rename) and return its relative path.
pub(crate) fn store_media(directory: &std::path::Path, bytes: &[u8], extension: &str) -> Result<String, String> {
    let name = format!("{}.{extension}", Uuid::new_v4());
    let relative = format!("media/{name}");
    let temporary = directory.join(format!("media/{name}.tmp"));
    fs::write(&temporary, bytes).map_err(err)?;
    if let Err(e) = fs::rename(&temporary, directory.join(&relative)) {
        let _ = fs::remove_file(&temporary);
        return Err(err(e));
    }
    Ok(relative)
}
/// Create an artwork with an automatic archive code, its photographs and an optional sale in
/// a single transaction. Every picture is validated before anything is written, and files
/// written during a failed attempt are removed, so nothing is left half-created.
fn create_artwork_full_core(db: &Database, input: NewArtworkInput) -> Result<Artwork, String> {
    if input.artwork.id.is_some() { return Err("Questa operazione crea una nuova opera".into()); }
    let mut fields = normalize_artwork(&input.artwork)?;
    if input.gallery.len() > 20 { return Err("Massimo 20 fotografie aggiuntive per opera".into()); }
    // Pictures from disk are read and checked before anything is written.
    let load = |source: &crate::media::PhotoSource| -> Result<Option<(Vec<u8>, String)>, String> {
        match (&source.path, &source.media_id) {
            (Some(path), None) => {
                let bytes = read_image_file(path)?;
                image_type(&bytes).ok_or_else(|| "Sono consentite solo immagini JPEG, PNG e WebP".to_string())?;
                crate::imaging::dimensions(&bytes)?;
                if !source.operations.is_empty() { crate::imaging::validate(&source.operations)?; }
                let name = std::path::Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("immagine").to_string();
                Ok(Some((bytes, name)))
            }
            (None, Some(_)) if source.operations.is_empty() => Ok(None),
            (None, Some(_)) => Err("Le modifiche si applicano alle fotografie da file: modifica prima l'immagine nel media center".into()),
            _ => Err("Fotografia non valida".into()),
        }
    };
    let cover = input.cover.as_ref().map(|s| load(s)).transpose()?;
    let gallery = input.gallery.iter().map(load).collect::<Result<Vec<_>, _>>()?;
    let gallery_kinds = input.gallery.iter().map(|s| crate::gallery::normalize_kind(s.kind.as_deref())).collect::<Result<Vec<_>, _>>()?;
    match &input.sale {
        Some(sale) => {
            if sale.amount_cents <= 0 || sale.amount_cents > 100_000_000_000 { return Err("Importo non valido".into()); }
            if !canonical_date(&sale.sold_at) { return Err("Data di vendita non valida".into()); }
            fields.status = "sold".into();
        }
        None if fields.status == "sold" => return Err("Registra una vendita per indicare l'opera come venduta".into()),
        None => {}
    }
    let sale_notes = match &input.sale { Some(s) => clean(s.notes.clone(), 15_000)?, None => None };
    let new_collector = match &input.sale {
        Some(s) if s.collector_id.as_deref().is_none_or(|v| v.trim().is_empty()) =>
            Some(required(s.new_collector_name.clone().unwrap_or_default(), 240, "Nome del collezionista")?),
        _ => None,
    };

    // The (possibly large) RAW file is copied before the archive is locked.
    let staged_raw = input.raw_path.as_ref()
        .map(|path| crate::raw_files::stage(&db.directory, std::path::Path::new(path), crate::raw_files::MAX_RAW_BYTES))
        .transpose()?;
    let outcome = (|| -> Result<Artwork, String> {
        let mut conn = db.conn.lock().map_err(err)?;
        // Files created by a failed attempt are the ones that were not there before.
        let existing: std::collections::HashSet<std::ffi::OsString> = fs::read_dir(db.directory.join("media"))
            .map(|entries| entries.flatten().map(|e| e.file_name()).collect()).unwrap_or_default();
        let attempt = (|| -> Result<Artwork, String> {
        let tx = conn.transaction().map_err(err)?;
        let id = Uuid::new_v4().to_string();
        let code = match fields.code.clone() {
            Some(code) => code,
            None => crate::archive_code::allocate(&tx, fields.year.as_deref())?,
        };
        if code_in_use(&tx, &code, &id)? { return Err("Esiste già un'opera con questo codice".into()); }
        let mut cover_ref: Option<(String, String)> = None;
        if let Some(source) = &input.cover {
            let loaded = cover.as_ref().and_then(|c| c.as_ref());
            cover_ref = Some(crate::media::materialize(&tx, &db.directory, source, loaded.map(|(b, _)| b.as_slice()), loaded.map(|(_, n)| n.as_str()).unwrap_or("immagine"))?);
        }
        let stamp = now();
        tx.execute("INSERT INTO artworks(id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,location,status,image_path,image_media_id,created_at,updated_at,signed,final_varnish,framed)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?15,?16,?17,?18)",
            params![id, code, fields.title, fields.year, fields.technique, fields.support, fields.width, fields.height,
                fields.depth, fields.description, fields.location, fields.status,
                cover_ref.as_ref().map(|r| r.1.clone()), cover_ref.as_ref().map(|r| r.0.clone()), stamp,
                fields.signed, fields.final_varnish, fields.framed]).map_err(err)?;
        for (position, source) in input.gallery.iter().enumerate() {
            let loaded = gallery[position].as_ref();
            let (media_id, relative) = crate::media::materialize(&tx, &db.directory, source, loaded.map(|(b, _)| b.as_slice()), loaded.map(|(_, n)| n.as_str()).unwrap_or("immagine"))?;
            tx.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,media_id,kind) VALUES(?1,?2,?3,NULL,?4,?5,?6,?7)",
                params![Uuid::new_v4().to_string(), id, relative, stamp, position as i64 + 1, media_id, gallery_kinds[position]]).map_err(err)?;
        }
        if let Some(sale) = &input.sale {
            let collector_id = match &new_collector {
                Some(name) => {
                    let collector_id = Uuid::new_v4().to_string();
                    tx.execute("INSERT INTO collectors(id,name,first_name,created_at,updated_at) VALUES(?1,?2,?2,?3,?3)",
                        params![collector_id, name, stamp]).map_err(err)?;
                    collector_id
                }
                None => sale.collector_id.clone().unwrap_or_default(),
            };
            let sale_id = Uuid::new_v4().to_string();
            tx.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,currency,sold_at,notes,created_at) VALUES(?1,?2,?3,?4,'EUR',?5,?6,?7)",
                params![sale_id, id, collector_id, sale.amount_cents, sale.sold_at, sale_notes, stamp])
                .map_err(|_| "Vendita non registrata: verifica il collezionista scelto".to_string())?;
            crate::finance::attach_sale_finances(&tx, &sale_id, sale.amount_cents, sale.tax_cents, sale.shipping_cents)?;
            if let Some(payment) = &sale.payment { crate::installments::apply_payment(&tx, &sale_id, sale.amount_cents, payment)?; }
        }
        if let Some(staged) = &staged_raw { crate::raw_files::link(&tx, &id, staged)?; }
        let saved = artwork_by_id(&tx, &id)?;
        tx.commit().map_err(err)?;
        Ok(saved)
        })();
        if attempt.is_err() {
            if let Ok(entries) = fs::read_dir(db.directory.join("media")) {
                for entry in entries.flatten() {
                    if !existing.contains(&entry.file_name()) && staged_raw.as_ref().is_none_or(|s| !s.relative.ends_with(&*entry.file_name().to_string_lossy())) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
        attempt
    })();
    if outcome.is_err() { if let Some(staged) = &staged_raw { staged.discard(); } }
    outcome
}
#[tauri::command]
pub async fn create_artwork_full(app: tauri::AppHandle, input: NewArtworkInput) -> Result<Artwork, String> {
    // Off the UI thread: copying a large RAW file can take a moment.
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Database>();
        create_artwork_full_core(&db, input)
    }).await.map_err(err)?
}
#[tauri::command]
pub fn save_artwork(db: State<'_, Database>, input: ArtworkInput) -> Result<Artwork, String> {
    save_artwork_core(&db, input)
}
#[tauri::command]
pub fn delete_artwork(db: State<'_, Database>, id: String) -> Result<(), String> {
    delete_artwork_core(&db, &id)
}
fn delete_artwork_core(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(err)?;
    let id = id.to_string();
    let old_image = artwork_by_id(&conn, &id)?.image_path;
    let raw_path: Option<String> = conn.query_row("SELECT media_path FROM artwork_raw_files WHERE artwork_id=?1", params![id], |r| r.get(0)).optional().map_err(err)?;
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
    if let Some(path) = raw_path { crate::raw_files::remove_file(&db.directory, &path); }
    if let Some(path) = old_image { crate::media::remove_file_unless_catalogued(&conn, &db.directory, &path); }
    for path in gallery_paths { crate::media::remove_file_unless_catalogued(&conn, &db.directory, &path); }
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
pub(crate) fn canonical_date(value: &str) -> bool {
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
    if let Some(payment)=&input.payment { crate::installments::apply_payment(&tx,&sale_id,input.amount_cents,payment)?; }
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
    issue_certificate_core(&db, input)
}
fn issue_certificate_core(db: &Database, input: CertificateInput) -> Result<Certificate, String> {
    let artist_name=required(input.artist_name,240,"Nome dell'artista")?;
    let collector_name=clean(input.collector_name,240)?;
    let notes=clean(input.notes,10_000)?;
    let mut conn=db.conn.lock().map_err(err)?;
    let tx=conn.transaction().map_err(err)?;
    let artwork=artwork_by_id(&tx,&input.artwork_id)?;
    let snapshot_json=serde_json::to_string(&artwork).map_err(err)?;
    tx.execute("UPDATE certificate_counter SET last_number=last_number+1 WHERE singleton=1",[]).map_err(err)?;
    let number:i64=tx.query_row("SELECT last_number FROM certificate_counter WHERE singleton=1",[],|r|r.get(0)).map_err(err)?;
    // The issue date and serial year follow the artist's local calendar, not UTC:
    // a certificate signed on 31 December evening must not carry the next year.
    let today=Local::now();
    let serial=format!("RS-{}-{number:06}",today.year());
    let id=Uuid::new_v4().to_string();
    let stamp=now();
    let issued_at=today.format("%Y-%m-%d").to_string();
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

pub(crate) const MAX_IMAGE_BYTES: usize = 12 * 1024 * 1024;
/// Read a picture chosen with the native file dialog. The size is checked from file
/// metadata before reading, so an oversized file never travels through memory or IPC.
pub(crate) fn read_image_file(source: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let path = std::path::Path::new(source);
    if !path.is_absolute() { return Err("Seleziona la fotografia tramite il dialogo di sistema".into()); }
    let metadata = fs::symlink_metadata(path).map_err(err)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("La fotografia non è un file regolare".into());
    }
    if metadata.len() == 0 || metadata.len() > MAX_IMAGE_BYTES as u64 {
        return Err("Immagine vuota o troppo grande (massimo 12 MiB)".into());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    fs::File::open(path).map_err(err)?.take(MAX_IMAGE_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(err)?;
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES { return Err("Immagine vuota o troppo grande (massimo 12 MiB)".into()); }
    Ok(bytes)
}
fn save_artwork_image_core(db: &Database, artwork_id: &str, data_base64: &str) -> Result<(), String> {
    if data_base64.len()>16_800_000 { return Err("Immagine troppo grande (massimo 12 MiB)".into()); }
    let bytes=STANDARD.decode(data_base64).map_err(|_|"Immagine non valida".to_string())?;
    save_artwork_image_bytes(db, artwork_id, &bytes)
}
fn save_artwork_image_bytes(db: &Database, artwork_id: &str, bytes: &[u8]) -> Result<(), String> {
    if bytes.len()>MAX_IMAGE_BYTES || bytes.is_empty() { return Err("Immagine troppo grande o vuota".into()); }
    let (ext,_mime)=image_type(bytes).ok_or_else(||"Sono consentite solo immagini JPEG, PNG e WebP".to_string())?;
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
    fs::write(&tmp,bytes).map_err(err)?;
    if let Err(e)=fs::rename(&tmp,&target) { let _=fs::remove_file(&tmp); return Err(err(e)); }
    if let Err(e)=conn.execute("UPDATE artworks SET image_path=?2,image_media_id=NULL,updated_at=?3 WHERE id=?1",params![artwork_id,relative,now()]) {
        let _=fs::remove_file(&target); return Err(err(e));
    }
    if let Some(old_path)=old {
        if !preserve_old { crate::media::remove_file_unless_catalogued(&conn, &db.directory, &old_path); }
    }
    Ok(())
}
#[tauri::command]
pub fn save_artwork_image(db: State<'_, Database>, artwork_id: String, data_base64: String) -> Result<(), String> {
    save_artwork_image_core(&db, &artwork_id, &data_base64)
}
#[tauri::command]
pub fn save_artwork_image_from_path(db: State<'_, Database>, artwork_id: String, source_path: String) -> Result<(), String> {
    let bytes = read_image_file(&source_path)?;
    save_artwork_image_bytes(&db, &artwork_id, &bytes)
}
pub(crate) fn safe_relative(path: &str) -> bool {
    let p=std::path::Path::new(path);
    p.components().all(|c|matches!(c,Component::Normal(_))) && p.starts_with("media")
}
pub(crate) fn image_type(bytes:&[u8])->Option<(&'static str,&'static str)> {
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
            id, code: Some("WORK-STALE-01".into()), title: title.into(), year: None,
            technique: None, support: None, width_cm: None, height_cm: None,
            depth_cm: None, description: None, location: None,
            status: "available".into(), signed: false, final_varnish: false, framed: false,
        }
    }

    fn collector_edit_input(id: Option<String>, name: &str) -> crate::collectors::CollectorInput {
        crate::collectors::CollectorInput {
            id, first_name: name.into(), last_name: None, email: None, phone: None, address: None,
            notes: None, birth_date: None, birth_time: None, birth_place: None, instagram: None,
            facebook: None, tiktok: None, website: None, linkedin: None,
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
        let collector = crate::collectors::save_collector_core(&db, collector_edit_input(None, "Persona")).unwrap();
        (dir, db, artwork.id, collector.id)
    }

    fn sale_input(artwork: &str, collector: &str, date: &str) -> SaleInput {
        SaleInput {
            artwork_id: artwork.into(), collector_id: collector.into(), amount_cents: 10_000,
            sold_at: date.into(), notes: None, tax_cents: 0, shipping_cents: 0, payment: None,
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
        other.code = Some("WORK-OTHER".into());
        let other = save_artwork_core(&db, other).unwrap();
        let mut clash = artwork_edit_input(Some(other.id.clone()), "Altra");
        clash.code = Some(first.code.to_lowercase());
        assert!(save_artwork_core(&db, clash).unwrap_err().contains("codice"));
        // Re-saving a record with its own code is not a conflict.
        let mut same = artwork_edit_input(Some(first.id.clone()), "Prima modificata");
        same.code = Some(first.code.clone());
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
    fn photos_chosen_from_disk_are_checked_before_being_read_and_stored() {
        let (dir, db, artwork, _collector) = sale_fixture();
        let picture = dir.path().join("foto.jpg");
        fs::write(&picture, [0xff, 0xd8, 0xff, 0x09]).unwrap();
        assert!(read_image_file("foto.jpg").unwrap_err().contains("dialogo"));
        assert!(read_image_file(dir.path().to_str().unwrap()).unwrap_err().contains("regolare"));
        let empty = dir.path().join("vuota.png");
        fs::write(&empty, []).unwrap();
        assert!(read_image_file(empty.to_str().unwrap()).unwrap_err().contains("12 MiB"));
        let huge = dir.path().join("enorme.png");
        fs::write(&huge, vec![0_u8; MAX_IMAGE_BYTES + 1]).unwrap();
        assert!(read_image_file(huge.to_str().unwrap()).unwrap_err().contains("12 MiB"));
        let bytes = read_image_file(picture.to_str().unwrap()).unwrap();
        save_artwork_image_bytes(&db, &artwork, &bytes).unwrap();
        let stored = {
            let conn = db.conn.lock().unwrap();
            artwork_by_id(&conn, &artwork).unwrap().image_path.unwrap()
        };
        assert_eq!(fs::read(db.directory.join(stored)).unwrap(), bytes);
        let text = dir.path().join("note.jpg");
        fs::write(&text, b"not an image").unwrap();
        let wrong = read_image_file(text.to_str().unwrap()).unwrap();
        assert!(save_artwork_image_bytes(&db, &artwork, &wrong).unwrap_err().contains("JPEG"));
    }

    fn new_artwork(title: &str, year: Option<&str>) -> NewArtworkInput {
        let mut artwork = artwork_edit_input(None, title);
        artwork.code = None;
        artwork.year = year.map(String::from);
        NewArtworkInput { artwork, cover: None, gallery: vec![], raw_path: None, sale: None }
    }
    /// A small real JPEG (the colour depends on `marker`, so different markers give different files).
    fn picture(dir: &std::path::Path, name: &str, marker: u8) -> String {
        let path = dir.join(name);
        let image = image::RgbImage::from_pixel(16, 12, image::Rgb([marker.wrapping_mul(40), 120, 200u8.wrapping_sub(marker)]));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 90)
            .encode_image(&image).unwrap();
        fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }
    fn src(path: String) -> crate::media::PhotoSource { crate::media::PhotoSource { path: Some(path), ..Default::default() } }
    fn media_files(db: &Database) -> usize {
        fs::read_dir(db.directory.join("media")).unwrap().count()
    }
    fn count(db: &Database, table: &str) -> i64 {
        db.conn.lock().unwrap().query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn artwork_flags_and_photo_kinds_are_saved_edited_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let plain = create_artwork_full_core(&db, new_artwork("Senza", None)).unwrap();
        assert!(!plain.signed && !plain.final_varnish && !plain.framed, "flags default to off");

        let mut input = new_artwork("Con", None);
        input.artwork.signed = true;
        input.artwork.framed = true;
        input.gallery = vec![
            crate::media::PhotoSource { kind: Some("SIGNATURE".into()), ..src(picture(dir.path(), "a.jpg", 1)) },
            src(picture(dir.path(), "b.jpg", 2)),
        ];
        let created = create_artwork_full_core(&db, input).unwrap();
        assert!(created.signed && created.framed && !created.final_varnish);
        let kinds: Vec<String> = db.conn.lock().unwrap().prepare("SELECT kind FROM artwork_images WHERE artwork_id=?1 ORDER BY sort_order").unwrap()
            .query_map(params![created.id], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(kinds, ["SIGNATURE", "OTHER"], "a missing kind means 'other'");

        // The flags can be changed afterwards.
        let mut edit = artwork_edit_input(Some(created.id.clone()), "Con");
        edit.code = None;
        edit.final_varnish = true;
        let saved = save_artwork_core(&db, edit).unwrap();
        assert!(saved.final_varnish && !saved.signed && !saved.framed);

        // An unknown kind is rejected and leaves nothing behind.
        let before = (count(&db, "artworks"), media_files(&db));
        let mut bad = new_artwork("Errata", None);
        bad.gallery = vec![crate::media::PhotoSource { kind: Some("PAESAGGIO".into()), ..src(picture(dir.path(), "c.jpg", 3)) }];
        assert!(create_artwork_full_core(&db, bad).unwrap_err().contains("Tipo di fotografia"));
        assert_eq!(before, (count(&db, "artworks"), media_files(&db)));
    }

    #[test]
    fn new_artworks_get_automatic_sequential_codes() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let first = create_artwork_full_core(&db, new_artwork("Uno", None)).unwrap();
        let second = save_artwork_core(&db, { let mut a = artwork_edit_input(None, "Due"); a.code = None; a }).unwrap();
        assert_eq!((first.code.as_str(), second.code.as_str()), ("ART-0001", "ART-0002"));
        let mut edit = artwork_edit_input(Some(second.id.clone()), "Due modificata");
        edit.code = None;
        assert_eq!(save_artwork_core(&db, edit).unwrap().code, "ART-0002", "editing keeps the assigned code");
        let mut manual = new_artwork("Manuale", None);
        manual.artwork.code = Some("art-0001".into());
        assert!(create_artwork_full_core(&db, manual).unwrap_err().contains("codice"));
        assert_eq!(create_artwork_full_core(&db, new_artwork("Tre", None)).unwrap().code, "ART-0003");
    }

    #[test]
    fn creating_with_photos_and_a_sale_stores_everything_in_one_step() {
        let (dir, db, _artwork, collector) = sale_fixture();
        let mut input = new_artwork("Con foto e vendita", Some("2025"));
        input.cover = Some(src(picture(dir.path(), "copertina.jpg", 1)));
        input.gallery = vec![src(picture(dir.path(), "a.jpg", 2)), src(picture(dir.path(), "b.jpg", 3))];
        input.artwork.status = "available".into();
        input.sale = Some(NewSaleInput {
            collector_id: Some(collector.clone()), new_collector_name: None, amount_cents: 250_000,
            sold_at: "2026-11-21".into(), notes: Some(" Consegna a mano ".into()), tax_cents: 0, shipping_cents: 5_000, payment: None,
        });
        let saved = create_artwork_full_core(&db, input).unwrap();
        assert_eq!(saved.status, "sold", "a sale makes the artwork sold");
        assert!(saved.image_path.is_some());
        let conn = db.conn.lock().unwrap();
        let rows: Vec<(i64, String)> = conn.prepare("SELECT sort_order,image_path FROM artwork_images WHERE artwork_id=?1 ORDER BY sort_order").unwrap()
            .query_map(params![saved.id], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2]);
        for (_, path) in &rows { assert!(db.directory.join(path).is_file()); }
        assert!(db.directory.join(saved.image_path.unwrap()).is_file());
        let (amount, shipping, notes): (i64, i64, String) = conn.query_row(
            "SELECT s.amount_cents,f.shipping_cents,s.notes FROM sales s JOIN sale_finances f ON f.sale_id=s.id WHERE s.artwork_id=?1",
            params![saved.id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!((amount, shipping, notes.as_str()), (250_000, 5_000, "Consegna a mano"));
    }

    #[test]
    fn a_new_collector_can_be_created_together_with_the_sale() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let mut input = new_artwork("Venduta subito", None);
        input.sale = Some(NewSaleInput {
            collector_id: None, new_collector_name: Some("  Anna Rossi ".into()), amount_cents: 90_000,
            sold_at: "2026-11-21".into(), notes: None, tax_cents: 0, shipping_cents: 0, payment: None,
        });
        create_artwork_full_core(&db, input).unwrap();
        let name: String = db.conn.lock().unwrap().query_row("SELECT name FROM collectors", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "Anna Rossi");
        let mut anonymous = new_artwork("Senza acquirente", None);
        anonymous.sale = Some(NewSaleInput {
            collector_id: None, new_collector_name: None, amount_cents: 1, sold_at: "2026-11-21".into(),
            notes: None, tax_cents: 0, shipping_cents: 0, payment: None,
        });
        assert!(create_artwork_full_core(&db, anonymous).unwrap_err().contains("collezionista"));
        assert_eq!((count(&db, "artworks"), count(&db, "collectors")), (1, 1));
    }

    #[test]
    fn a_failed_creation_leaves_no_artwork_sale_collector_file_or_used_code() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        // Invalid pictures are rejected before anything is written.
        let mut wrong_type = new_artwork("Tipo errato", None);
        let text = dir.path().join("testo.jpg");
        fs::write(&text, b"not an image").unwrap();
        wrong_type.gallery = vec![src(text.to_string_lossy().into_owned())];
        assert!(create_artwork_full_core(&db, wrong_type).is_err());
        // A failure after the files were written (the sale breaks a rule) is rolled back and cleaned up.
        let mut bad_sale = new_artwork("Vendita errata", None);
        bad_sale.cover = Some(src(picture(dir.path(), "c.jpg", 4)));
        bad_sale.gallery = vec![src(picture(dir.path(), "g.jpg", 5))];
        bad_sale.sale = Some(NewSaleInput {
            collector_id: Some("inesistente".into()), new_collector_name: None, amount_cents: 1_000,
            sold_at: "2026-11-21".into(), notes: None, tax_cents: 0, shipping_cents: 0, payment: None,
        });
        assert!(create_artwork_full_core(&db, bad_sale).is_err());
        let mut too_much = new_artwork("Spese eccessive", None);
        too_much.cover = Some(src(picture(dir.path(), "c2.jpg", 6)));
        too_much.sale = Some(NewSaleInput {
            collector_id: None, new_collector_name: Some("Chi".into()), amount_cents: 1_000,
            sold_at: "2026-11-21".into(), notes: None, tax_cents: 900, shipping_cents: 900, payment: None,
        });
        assert!(create_artwork_full_core(&db, too_much).is_err());
        assert_eq!((count(&db, "artworks"), count(&db, "artwork_images"), count(&db, "sales"), count(&db, "collectors")), (0, 0, 0, 0));
        assert_eq!(media_files(&db), 0, "no orphan files");
        assert_eq!(create_artwork_full_core(&db, new_artwork("Prima vera", None)).unwrap().code, "ART-0001",
            "failed attempts must not consume archive numbers");
        let mut sold_without_sale = new_artwork("Venduta senza vendita", None);
        sold_without_sale.artwork.status = "sold".into();
        assert!(create_artwork_full_core(&db, sold_without_sale).is_err());
        let mut many = new_artwork("Troppe foto", None);
        many.gallery = vec![crate::media::PhotoSource::default(); 21];
        assert!(create_artwork_full_core(&db, many).unwrap_err().contains("20"));
    }

    #[test]
    fn a_sale_can_be_registered_with_an_instant_payment_or_an_installment_plan() {
        use crate::installments::{InstallmentInput, PaymentInput};
        let (_dir, db, artwork, collector) = sale_fixture();
        let mut input = sale_input(&artwork, &collector, "2026-11-21");
        input.payment = Some(PaymentInput { mode: "INSTALLMENTS".into(), received_cents: Some(2_000), received_at: Some("2026-11-21".into()),
            method: Some("bank_transfer".into()), reference: Some(" Acconto ".into()), installments: vec![
                InstallmentInput { due_date: "2026-12-21".into(), amount_cents: 4_000, notes: None },
                InstallmentInput { due_date: "2027-01-21".into(), amount_cents: 6_000, notes: Some("Saldo".into()) }] });
        record_sale_core(&db, input).unwrap();
        let conn = db.conn.lock().unwrap();
        let received: (i64, String) = conn.query_row("SELECT amount_cents,reference FROM sale_receipts", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(received, (2_000, "Acconto".to_string()));
        let plan: i64 = conn.query_row("SELECT count(*) FROM sale_installments", [], |r| r.get(0)).unwrap();
        assert_eq!(plan, 2);
    }

    #[test]
    fn an_invalid_payment_cancels_the_whole_sale() {
        use crate::installments::{InstallmentInput, PaymentInput};
        let (_dir, db, artwork, collector) = sale_fixture();
        let mut input = sale_input(&artwork, &collector, "2026-11-21");
        input.payment = Some(PaymentInput { mode: "INSTALLMENTS".into(), received_cents: None, received_at: None, method: None, reference: None,
            installments: vec![InstallmentInput { due_date: "2026-12-21".into(), amount_cents: 4_000, notes: None },
                InstallmentInput { due_date: "2027-01-21".into(), amount_cents: 5_000, notes: None }] });
        assert!(record_sale_core(&db, input).unwrap_err().contains("sommare"));
        let conn = db.conn.lock().unwrap();
        let (sales, status): (i64, String) = (
            conn.query_row("SELECT count(*) FROM sales", [], |r| r.get(0)).unwrap(),
            conn.query_row("SELECT status FROM artworks WHERE id=?1", params![artwork], |r| r.get(0)).unwrap());
        assert_eq!((sales, status.as_str()), (0, "available"), "no half-registered sale remains");
    }

    #[test]
    fn a_raw_file_can_be_attached_at_creation_and_disappears_with_the_artwork() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        let source = dir.path().join("scatto.NEF");
        fs::write(&source, [b'I', b'I', b'*', 0, 1, 2, 3]).unwrap();
        let mut input = new_artwork("Con RAW", None);
        input.raw_path = Some(source.to_string_lossy().into_owned());
        let saved = create_artwork_full_core(&db, input).unwrap();
        let path: String = db.conn.lock().unwrap().query_row("SELECT media_path FROM artwork_raw_files WHERE artwork_id=?1", params![saved.id], |r| r.get(0)).unwrap();
        assert!(db.directory.join(&path).is_file());
        // A failing creation leaves no RAW copy behind.
        let mut failing = new_artwork("Fallita", None);
        failing.raw_path = Some(source.to_string_lossy().into_owned());
        failing.artwork.status = "sold".into();
        assert!(create_artwork_full_core(&db, failing).is_err());
        assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), 1);
        delete_artwork_core(&db, &saved.id).unwrap();
        assert!(!db.directory.join(&path).exists(), "the RAW file is removed with its artwork");
    }
    #[test]
    fn photographs_can_be_edited_while_the_artwork_is_created_and_the_original_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        let mut input = new_artwork("Con modifiche", None);
        let mut cover = src(picture(dir.path(), "copertina.jpg", 3));
        cover.operations = vec![crate::imaging::Operation::Rotate { degrees: 90 }];
        input.cover = Some(cover);
        let mut gallery = src(picture(dir.path(), "extra.jpg", 4));
        gallery.operations = vec![crate::imaging::Operation::Flip { horizontal: true, vertical: false }];
        input.gallery = vec![gallery, src(picture(dir.path(), "senza.jpg", 5))];
        let saved = create_artwork_full_core(&db, input).unwrap();
        let conn = db.conn.lock().unwrap();
        let (revisions, edits): (i64, i64) = conn.query_row("SELECT count(*),sum(kind='EDIT') FROM media_revisions", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((revisions, edits), (5, 2), "three originals plus two edited versions");
        let cover_path = saved.image_path.clone().unwrap();
        let (width, height) = crate::imaging::dimensions(&fs::read(db.directory.join(&cover_path)).unwrap()).unwrap();
        assert_eq!((width, height), (12, 16), "the artwork shows the rotated version (the source was 16x12)");
        let current: String = conn.query_row("SELECT r.image_path FROM artworks a JOIN media_items i ON i.id=a.image_media_id JOIN media_revisions r ON r.id=i.current_revision_id", [], |r| r.get(0)).unwrap();
        assert_eq!(current, cover_path);
        let original_exists: i64 = conn.query_row("SELECT count(*) FROM media_revisions WHERE kind='ORIGINAL'", [], |r| r.get(0)).unwrap();
        assert_eq!(original_exists, 3);
        let linked: i64 = conn.query_row("SELECT count(*) FROM artwork_images WHERE media_id IS NOT NULL", [], |r| r.get(0)).unwrap();
        assert_eq!(linked, 2);
    }

    #[test]
    fn a_library_picture_can_be_used_for_a_new_artwork_but_not_edited_there() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        let file = picture(dir.path(), "libreria.jpg", 6);
        let item = crate::media::import_core(&db, &[file]).unwrap().imported.remove(0);
        let mut input = new_artwork("Dalla libreria", None);
        input.cover = Some(crate::media::PhotoSource { media_id: Some(item.id.clone()), ..Default::default() });
        let saved = create_artwork_full_core(&db, input).unwrap();
        assert_eq!(saved.image_path.as_deref(), Some(item.image_path.as_str()));
        let revisions: i64 = db.conn.lock().unwrap().query_row("SELECT count(*) FROM media_revisions", [], |r| r.get(0)).unwrap();
        assert_eq!(revisions, 1, "no copy is made");
        let mut editing = new_artwork("Con modifica", None);
        editing.cover = Some(crate::media::PhotoSource { media_id: Some(item.id.clone()),
            operations: vec![crate::imaging::Operation::Rotate { degrees: 90 }], ..Default::default() });
        assert!(create_artwork_full_core(&db, editing).unwrap_err().contains("media center"));
        let mut missing = new_artwork("Mancante", None);
        missing.cover = Some(crate::media::PhotoSource { media_id: Some("sconosciuta".into()), ..Default::default() });
        assert!(create_artwork_full_core(&db, missing).is_err());
        let mut both = new_artwork("Ambigua", None);
        both.cover = Some(crate::media::PhotoSource { media_id: Some(item.id), path: Some("x".into()), ..Default::default() });
        assert!(create_artwork_full_core(&db, both).is_err());
    }

    #[test]
    fn a_failure_after_editing_removes_every_new_file_original_and_edited() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        let mut input = new_artwork("Fallirà", None);
        let mut cover = src(picture(dir.path(), "c.jpg", 2));
        cover.operations = vec![crate::imaging::Operation::Rotate { degrees: 90 }];
        input.cover = Some(cover);
        input.sale = Some(NewSaleInput { collector_id: Some("inesistente".into()), new_collector_name: None, amount_cents: 100,
            sold_at: "2026-11-21".into(), notes: None, tax_cents: 0, shipping_cents: 0, payment: None });
        assert!(create_artwork_full_core(&db, input).is_err());
        assert_eq!(fs::read_dir(db.directory.join("media")).unwrap().count(), 0);
        let items: i64 = db.conn.lock().unwrap().query_row("SELECT count(*) FROM media_items", [], |r| r.get(0)).unwrap();
        assert_eq!(items, 0, "the catalogue rolled back with the artwork");
    }

    #[test]
    fn certificates_use_the_local_calendar_and_unique_increasing_serials() {
        let (_dir, db, artwork, _collector) = sale_fixture();
        let input = || CertificateInput {
            artwork_id: artwork.clone(), artist_name: "Artista".into(),
            collector_name: None, notes: None,
        };
        let first = issue_certificate_core(&db, input()).unwrap();
        let second = issue_certificate_core(&db, input()).unwrap();
        let today = Local::now();
        assert_eq!(first.issued_at, today.format("%Y-%m-%d").to_string());
        assert!(first.serial.starts_with(&format!("RS-{}-", today.year())), "{}", first.serial);
        assert!(first.serial.ends_with("000001") && second.serial.ends_with("000002"));
        assert!(issue_certificate_core(&db, CertificateInput {
            artwork_id: "missing".into(), artist_name: "A".into(), collector_name: None, notes: None,
        }).is_err());
        let counter: i64 = db.conn.lock().unwrap()
            .query_row("SELECT last_number FROM certificate_counter", [], |r| r.get(0)).unwrap();
        assert_eq!(counter, 2, "a rejected certificate must not consume a number");
    }

    #[test]
    fn collector_edit_never_recreates_a_deleted_record() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let created = crate::collectors::save_collector_core(&db, collector_edit_input(None, "Original")).unwrap();
        let changed = crate::collectors::save_collector_core(&db,
            collector_edit_input(Some(created.id.clone()), "Edited")).unwrap();
        assert_eq!(changed.id, created.id);
        assert_eq!(changed.name, "Edited");
        db.conn.lock().unwrap().execute("DELETE FROM collectors WHERE id=?1",
            params![created.id]).unwrap();
        let error = crate::collectors::save_collector_core(&db,
            collector_edit_input(Some(created.id.clone()), "Resurrected")).unwrap_err();
        assert!(error.contains("non trovato"));
        let count: i64 = db.conn.lock().unwrap().query_row(
            "SELECT count(*) FROM collectors", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 0, "a stale editor must not silently insert a collector");
    }
}
