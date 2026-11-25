//! Explicit, local archive import. Never opens paths from imported data.
use crate::{commands::{clean, required}, db::Database};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::Path;
use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use tauri::State;
use uuid::Uuid;

const FORMAT: &str = "repertica-studio-artwork-metadata-v1";
const RELATIONAL_FORMAT: &str = "repertica-studio-archive-relations-v2";
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMAGES: usize = 1000;
const MAX_ROWS: usize = 1000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Package {
    format: String,
    artworks: Vec<Record>,
}
#[derive(Deserialize)]
struct FormatProbe { format: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationalPackage {
    format: String,
    artworks: Vec<Record>,
    collectors: Vec<CollectorRecord>,
    sales: Vec<SaleRecord>,
    #[serde(default)]
    covers: Vec<CoverRecord>,
    #[serde(default)]
    gallery: Vec<CoverRecord>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverRecord {
    artwork_code: String,
    sha256: String,
    data_base64: String,
    #[serde(default)]
    position: i64,
    caption: Option<String>,
    kind: Option<String>,
}
struct CheckedCover {
    artwork_code: String,
    sha256: String,
    bytes: Vec<u8>,
    extension: &'static str,
    position: i64,
    caption: Option<String>,
    kind: String,
}
#[derive(Serialize)]
struct CoverPreview {
    artwork_code: String,
    sha256: String,
    size_bytes: usize,
    position: i64,
    caption: Option<String>,
    kind: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CollectorRecord {
    key: String,
    first_name: String,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    notes: Option<String>,
    birth_date: Option<String>,
    birth_time: Option<String>,
    birth_place: Option<String>,
    instagram: Option<String>,
    facebook: Option<String>,
    tiktok: Option<String>,
    website: Option<String>,
    linkedin: Option<String>,
}
impl CollectorRecord {
    fn name(&self) -> String {
        match &self.last_name { Some(last) => format!("{} {}", self.first_name, last), None => self.first_name.clone() }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SaleRecord {
    artwork_code: String,
    collector_key: String,
    amount_cents: i64,
    currency: String,
    sold_at: String,
    notes: Option<String>,
    tax_cents: i64,
    shipping_cents: i64,
    legacy_shares_bp: [i64; 3],
    // Explicitly unsupported historical data must not be silently discarded.
    allocations: Vec<serde_json::Value>,
    installments: Vec<InstallmentRecord>,
    receipts: Vec<ReceiptRecord>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct InstallmentRecord {
    position: i64,
    due_date: String,
    amount_cents: i64,
    notes: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiptRecord {
    received_at: String,
    amount_cents: i64,
    method: String,
    reference: Option<String>,
    notes: Option<String>,
}
struct Parsed {
    artworks: Vec<Record>,
    collectors: Vec<CollectorRecord>,
    sales: Vec<SaleRecord>,
    covers: Vec<CheckedCover>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    code: String,
    title: String,
    year: Option<String>,
    technique: Option<String>,
    support: Option<String>,
    width_cm: Option<f64>,
    height_cm: Option<f64>,
    depth_cm: Option<f64>,
    description: Option<String>,
    location: Option<String>,
    status: String,
    #[serde(default)] signed: bool,
    #[serde(default)] final_varnish: bool,
    #[serde(default)] framed: bool,
    artist_coefficient_override: Option<f64>,
    fixed_price_cents: Option<i64>,
}
#[derive(Serialize)]
pub struct Preview {
    digest: String,
    artworks: Vec<Record>,
    collectors: Vec<CollectorRecord>,
    sales: Vec<SaleRecord>,
    covers: Vec<CoverPreview>,
}

fn invalid(row: usize) -> String { format!("invalid_row:{}", row + 1) }
fn duplicate(row: usize) -> String { format!("duplicate:{}", row + 1) }
fn database_error(_: impl std::fmt::Display) -> String { "database".into() }

fn parse(source: &str) -> Result<Parsed, String> {
    if source.len() > MAX_BYTES { return Err("size".into()); }
    let format_error = |_| "format".to_string();
    let probe: FormatProbe = serde_json::from_str(source).map_err(format_error)?;
    let (mut rows, mut collectors, mut sales, covers, gallery, relational) = match probe.format.as_str() {
        FORMAT => {
            let package: Package = serde_json::from_str(source).map_err(format_error)?;
            if package.format != FORMAT { return Err("format".into()); }
            (package.artworks, vec![], vec![], vec![], vec![], false)
        }
        RELATIONAL_FORMAT => {
            let package: RelationalPackage = serde_json::from_str(source).map_err(format_error)?;
            if package.format != RELATIONAL_FORMAT { return Err("format".into()); }
            (package.artworks, package.collectors, package.sales, package.covers, package.gallery, true)
        }
        _ => return Err("format".into()),
    };
    if rows.is_empty() || rows.len() > MAX_ROWS || collectors.len() > MAX_ROWS || sales.len() > MAX_ROWS {
        return Err("count".into());
    }
    let mut seen = HashSet::new();
    for (i, row) in rows.iter_mut().enumerate() {
        let err = || invalid(i);
        row.code = required(std::mem::take(&mut row.code), 80, "code").map_err(|_| err())?;
        row.title = required(std::mem::take(&mut row.title), 240, "title").map_err(|_| err())?;
        if row.code.chars().any(char::is_control) || row.title.chars().any(char::is_control) {
            return Err(err());
        }
        // Do not introduce an exhausted i64 archive-code sequence before #59.
        let suffix: String = row.code.chars().rev().take_while(char::is_ascii_digit).collect::<Vec<_>>().into_iter().rev().collect();
        if suffix.parse::<i64>().ok() == Some(i64::MAX) { return Err(err()); }
        for (value, limit) in [
            (&mut row.year, 40), (&mut row.technique, 300), (&mut row.support, 300),
            (&mut row.description, 40_000), (&mut row.location, 300),
        ] {
            *value = clean(value.take(), limit).map_err(|_| err())?;
            if value.as_ref().is_some_and(|v| v.contains('\0')) { return Err(err()); }
        }
        if [row.width_cm, row.height_cm, row.depth_cm, row.artist_coefficient_override]
            .into_iter().flatten().any(|v| !v.is_finite() || v <= 0.0 || v > 100_000.0)
            || row.fixed_price_cents.is_some_and(|v| !(0..=100_000_000_000).contains(&v))
            || !(matches!(row.status.as_str(), "available" | "reserved" | "archived") || (relational && row.status == "sold")) {
            return Err(err());
        }
        // Match SQLite's NOCASE semantics, not a lossy Unicode search fold.
        if !seen.insert(row.code.to_ascii_lowercase()) { return Err(duplicate(i)); }
    }
    validate_relations(&rows, &mut collectors, &mut sales)?;
    let covers = validate_covers(&rows, covers, gallery)?;
    Ok(Parsed { artworks: rows, collectors, sales, covers })
}

// Explicit self-contained photos; no path or URL is ever resolved from the package.
fn validate_covers(rows: &[Record], covers: Vec<CoverRecord>, gallery: Vec<CoverRecord>) -> Result<Vec<CheckedCover>, String> {
    if covers.len() > rows.len() || covers.len() + gallery.len() > MAX_IMAGES { return Err(relation_error()); }
    let mut seen = HashSet::new();
    let mut gallery_counts = BTreeMap::<String, i64>::new();
    let codes: HashSet<&str> = rows.iter().map(|r| r.code.as_str()).collect();
    let mut checked = Vec::new();
    let mut total_pixels = 0u64;
    for (mut photo, is_gallery) in covers.into_iter().map(|p| (p, false)).chain(gallery.into_iter().map(|p| (p, true))) {
        if !codes.contains(photo.artwork_code.as_str()) || !seen.insert((photo.artwork_code.clone(), photo.position)) {
            return Err(relation_error());
        }
        if is_gallery {
            if !(1..=20).contains(&photo.position) { return Err(relation_error()); }
            *gallery_counts.entry(photo.artwork_code.clone()).or_default() += 1;
            clean_relation(&mut photo.caption, 300)?;
        } else if photo.position != 0 || photo.caption.is_some() || photo.kind.is_some() {
            return Err(relation_error());
        }
        let kind = photo.kind.unwrap_or_else(|| "OTHER".to_string());
        if !matches!(kind.as_str(), "DETAIL" | "SIGNATURE" | "DIGITAL_MOCKUP" | "IN_SITU" | "OTHER" | "FRAMED" | "WORK_IN_PROGRESS") {
            return Err(relation_error());
        }
        let bytes = STANDARD.decode(&photo.data_base64).map_err(|_| relation_error())?;
        if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES || STANDARD.encode(&bytes) != photo.data_base64
            || format!("{:x}", Sha256::digest(&bytes)) != photo.sha256 {
            return Err(relation_error());
        }
        let (extension, _) = crate::commands::image_type(&bytes).ok_or_else(relation_error)?;
        let (w,h) = crate::imaging::dimensions(&bytes).map_err(|_| relation_error())?;
        let pixels = u64::from(w)*u64::from(h);
        if pixels == 0 || pixels > 16_000_000 { return Err(relation_error()); }
        total_pixels += pixels;
        if total_pixels > 128_000_000 { return Err(relation_error()); }
        image::load_from_memory(&bytes).map_err(|_| relation_error())?;
        checked.push(CheckedCover { artwork_code: photo.artwork_code, sha256: photo.sha256, bytes, extension,
            position: photo.position, caption: photo.caption, kind });
    }
    for (code, count) in gallery_counts {
        if !(1..=count).all(|position| seen.contains(&(code.clone(), position))) { return Err(relation_error()); }
    }
    checked.sort_by(|a,b| (&a.artwork_code, a.position).cmp(&(&b.artwork_code,b.position)));
    Ok(checked)
}

fn relation_error() -> String { "relations".into() }
fn collector_conflict() -> String { "collector_conflict".into() }
fn clean_relation(value: &mut Option<String>, limit: usize) -> Result<(), String> {
    *value = clean(value.take(), limit).map_err(|_| relation_error())?;
    if value.as_ref().is_some_and(|v| v.contains('\0')) { return Err(relation_error()); }
    Ok(())
}
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 80 && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn valid_date(date: &str) -> bool { date.len() == 10 && crate::commands::canonical_date(date) }
fn validate_relations(rows: &[Record], collectors: &mut [CollectorRecord], sales: &mut [SaleRecord]) -> Result<(), String> {
    let mut keys = HashSet::new();
    let mut names = HashSet::new();
    let mut emails = HashSet::new();
    for c in collectors {
        c.first_name = required(std::mem::take(&mut c.first_name), 240, "name").map_err(|_| relation_error())?;
        if !valid_key(&c.key) || !keys.insert(c.key.clone()) || c.first_name.chars().any(char::is_control) {
            return Err(relation_error());
        }
        for (value, limit) in [(&mut c.last_name,240),(&mut c.email,320),(&mut c.phone,120),
            (&mut c.address,2000),(&mut c.notes,15000)] { clean_relation(value,limit)?; }
        for (value, limit) in [(&mut c.birth_date,10),(&mut c.birth_time,5),(&mut c.birth_place,300),
            (&mut c.instagram,300),(&mut c.facebook,300),(&mut c.tiktok,300),(&mut c.website,500),(&mut c.linkedin,500)] {
            clean_relation(value,limit)?;
        }
        if c.birth_date.as_ref().is_some_and(|d| !valid_date(d)) ||
            c.birth_time.as_ref().is_some_and(|t| chrono::NaiveTime::parse_from_str(t,"%H:%M")
                .map(|v| v.format("%H:%M").to_string() != *t).unwrap_or(true)) {
            return Err(relation_error());
        }
        if c.last_name.as_ref().is_some_and(|v| v.chars().any(char::is_control))
            || c.email.as_ref().is_some_and(|v| !v.contains('@') || v.chars().any(|ch| ch.is_whitespace() || ch.is_control())) {
            return Err(relation_error());
        }
        if !names.insert(c.name().to_ascii_lowercase()) || c.email.as_ref().is_some_and(|e| !emails.insert(e.to_ascii_lowercase())) {
            return Err(collector_conflict());
        }
    }
    let sold: HashSet<&str> = rows.iter().filter(|r| r.status == "sold").map(|r| r.code.as_str()).collect();
    let mut linked = HashSet::new();
    let mut receipt_count = 0_usize;
    let mut installment_count = 0_usize;
    for sale in sales {
        if !sold.contains(sale.artwork_code.as_str()) || !linked.insert(sale.artwork_code.clone())
            || !keys.contains(&sale.collector_key) || sale.currency != "EUR"
            || !(1..=100_000_000_000).contains(&sale.amount_cents)
            || sale.tax_cents < 0 || sale.shipping_cents < 0
            || sale.tax_cents.checked_add(sale.shipping_cents).is_none_or(|v| v > sale.amount_cents)
            || !valid_date(&sale.sold_at)
            || sale.legacy_shares_bp.iter().any(|v| !(0..=10000).contains(v))
            || sale.legacy_shares_bp.iter().sum::<i64>() != 10000
            || !sale.allocations.is_empty() {
            return Err(relation_error());
        }
        clean_relation(&mut sale.notes,15000)?;
        installment_count = installment_count.checked_add(sale.installments.len()).ok_or_else(relation_error)?;
        if installment_count > MAX_ROWS { return Err(relation_error()); }
        // Legacy receipts may be explicitly assigned to a particular due date,
        // while Studio allocates them in order. Do not silently reinterpret history.
        if !sale.installments.is_empty() {
            if !(2..=60).contains(&sale.installments.len()) || !sale.receipts.is_empty() {
                return Err(relation_error());
            }
            let mut total = 0_i64;
            let mut previous = String::new();
            for (i, item) in sale.installments.iter_mut().enumerate() {
                if item.position != i as i64 + 1 || !valid_date(&item.due_date)
                    || item.due_date.as_str() < previous.as_str()
                    || !(1..=100_000_000_000).contains(&item.amount_cents) {
                    return Err(relation_error());
                }
                previous.clone_from(&item.due_date);
                if item.notes.as_ref().is_some_and(|n| n.chars().count() > 500) { return Err(relation_error()); }
                clean_relation(&mut item.notes,500)?;
                total = total.checked_add(item.amount_cents).ok_or_else(relation_error)?;
                if total > sale.amount_cents { return Err(relation_error()); }
            }
            if total != sale.amount_cents { return Err(relation_error()); }
        }
        receipt_count = receipt_count.checked_add(sale.receipts.len()).ok_or_else(relation_error)?;
        if receipt_count > 1000 { return Err(relation_error()); }
        let mut total = 0_i64;
        for receipt in &mut sale.receipts {
            if !valid_date(&receipt.received_at) || !(1..=100_000_000_000).contains(&receipt.amount_cents)
                || !matches!(receipt.method.as_str(),"bank_transfer"|"cash"|"card"|"other") { return Err(relation_error()); }
            total = total.checked_add(receipt.amount_cents).ok_or_else(relation_error)?;
            if total > sale.amount_cents { return Err(relation_error()); }
            clean_relation(&mut receipt.reference,200)?;
            clean_relation(&mut receipt.notes,2000)?;
        }
    }
    if linked.len() != sold.len() { return Err(relation_error()); }
    Ok(())
}

fn ensure_new(conn: &Connection, parsed: &Parsed) -> Result<(), String> {
    let rows = &parsed.artworks;
    for c in &parsed.collectors {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM collectors WHERE name=?1 COLLATE NOCASE OR (?2 IS NOT NULL AND email=?2 COLLATE NOCASE))",
            params![c.name(), c.email], |r| r.get(0)).map_err(database_error)?;
        if exists { return Err(collector_conflict()); }
    }
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM artworks WHERE code=?1 COLLATE NOCASE)")
        .map_err(database_error)?;
    for (i, row) in rows.iter().enumerate() {
        let exists: bool = stmt.query_row([&row.code], |r| r.get(0)).map_err(database_error)?;
        if exists { return Err(duplicate(i)); }
    }
    Ok(())
}

// Match the native GLOBAL and YEARLY code policies, including their shared
// counter across separator/padding changes. Arbitrary legacy codes remain
// importable but do not create a sequence unless they fit a native policy.
fn imported_sequence(code: &str) -> Option<(String, i64)> {
    for separator in ['-', '.', '_'] {
        let parts: Vec<&str> = code.split(separator).collect();
        let (prefix, scope, digits) = match parts.as_slice() {
            [prefix, digits] => (*prefix, "GLOBAL", *digits),
            [prefix, year, digits] if year.len() == 4
                && year.bytes().all(|b| b.is_ascii_digit())
                && year.parse::<i32>().ok().is_some_and(|y| (1000..=9999).contains(&y)) => (*prefix, *year, *digits),
            _ => continue,
        };
        if prefix.is_empty() || prefix.len() > 12
            || !prefix.bytes().all(|b| b.is_ascii_alphanumeric())
            || digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let number = digits.parse::<i64>().ok()?;
        if number > 0 {
            return Some((format!("{}|{}", prefix.to_ascii_uppercase(), scope), number));
        }
    }
    None
}

const RESERVE: &str = "INSERT INTO archive_code_sequences(scope_key,last_value) VALUES(?1,?2) ON CONFLICT(scope_key) DO UPDATE SET last_value=excluded.last_value WHERE archive_code_sequences.last_value < excluded.last_value";
const READ_SEQUENCE: &str = "SELECT COALESCE((SELECT last_value FROM archive_code_sequences WHERE scope_key=?1),0)";

fn reserve_imported_codes(conn: &Connection, rows: &[Record]) -> Result<(), String> {
    let mut targets = BTreeMap::<String, i64>::new();
    for (key, number) in rows.iter().filter_map(|row| imported_sequence(&row.code)) {
        targets.entry(key).and_modify(|n| *n = (*n).max(number)).or_insert(number);
    }
    // Capture all pre-existing high-water marks before touching any sequence.
    for (key, number) in targets.iter_mut() {
        let previous: i64 = conn.query_row(READ_SEQUENCE, [key], |r| r.get(0)).map_err(database_error)?;
        *number = (*number).max(previous);
    }
    for (key, number) in &targets {
        conn.execute(RESERVE, params![key, number]).map_err(database_error)?;
    }
    for (key, number) in &targets {
        let stored: i64 = conn.query_row(READ_SEQUENCE, [key], |r| r.get(0)).map_err(database_error)?;
        if stored != *number { return Err("database".into()); }
    }
    Ok(())
}

fn digest(source: &str) -> String { format!("{:x}", Sha256::digest(source.as_bytes())) }

fn preview(conn: &Connection, source: &str) -> Result<Preview, String> {
    let parsed = parse(source)?;
    ensure_new(conn, &parsed)?;
    let covers = parsed.covers.iter().map(|c| CoverPreview { artwork_code: c.artwork_code.clone(), sha256: c.sha256.clone(), size_bytes: c.bytes.len(), position: c.position, caption: c.caption.clone(), kind: c.kind.clone() }).collect();
    Ok(Preview { digest: digest(source), artworks: parsed.artworks, collectors: parsed.collectors, sales: parsed.sales, covers })
}

// One transaction for the entire package, no overwrite; optional small covers.
const INSERT: &str = "INSERT INTO artworks(id,code,title,year,technique,support,width_cm,height_cm,depth_cm,description,location,status,created_at,updated_at,signed,final_varnish,framed,artist_coefficient_override,fixed_price_cents)
VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13,?14,?15,?16,?17,?18)";
const VERIFY: &str = "SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1 AND code COLLATE BINARY IS ?2 AND title IS ?3 AND year IS ?4 AND technique IS ?5 AND support IS ?6 AND width_cm IS ?7 AND height_cm IS ?8 AND depth_cm IS ?9 AND description IS ?10 AND location IS ?11 AND status IS ?12 AND created_at IS ?13 AND updated_at IS ?13 AND signed IS ?14 AND final_varnish IS ?15 AND framed IS ?16 AND artist_coefficient_override IS ?17 AND fixed_price_cents IS ?18 AND image_path IS ?19 AND image_media_id IS NULL AND (SELECT count(*) FROM sales WHERE artwork_id=?1)=CASE WHEN ?12='sold' THEN 1 ELSE 0 END)";
#[cfg(test)]
fn apply(conn: &mut Connection, source: &str, expected_digest: &str) -> Result<usize, String> {
    apply_with_media(conn, None, source, expected_digest)
}
fn apply_with_media(conn: &mut Connection, directory: Option<&Path>, source: &str, expected_digest: &str) -> Result<usize, String> {
    let parsed = parse(source)?;
    if digest(source) != expected_digest { return Err("changed".into()); }
    if directory.is_none() && !parsed.covers.is_empty() { return Err("database".into()); }
    let mut written = BTreeMap::<(String, i64), (String, Option<String>)>::new();
    let outcome = (|| -> Result<usize, String> {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(database_error)?;
        ensure_new(&tx, &parsed)?;
        let rows = &parsed.artworks;
        let stamp = chrono::Utc::now().to_rfc3339();
        let ids: Vec<String> = rows.iter().map(|_| Uuid::new_v4().to_string()).collect();
        for (id, row) in ids.iter().zip(rows) {
            let affected = tx.execute(INSERT, params![id, row.code, row.title, row.year, row.technique,
                row.support, row.width_cm, row.height_cm, row.depth_cm, row.description, row.location,
                row.status, stamp, row.signed, row.final_varnish, row.framed,
                row.artist_coefficient_override, row.fixed_price_cents]).map_err(database_error)?;
            if affected != 1 { return Err("database".into()); }
        }
        // Reserve imported numbers before committing so deletion cannot make a code reusable.
        reserve_imported_codes(&tx, rows)?;
        let artwork_ids: BTreeMap<&str, &str> = rows.iter().zip(&ids).map(|(r,id)| (r.code.as_str(),id.as_str())).collect();
        for cover in &parsed.covers {
            let root = directory.ok_or_else(|| "database".to_string())?;
            let relative = crate::commands::store_media(root, &cover.bytes, cover.extension).map_err(database_error)?;
            let gallery_id = (cover.position > 0).then(|| Uuid::new_v4().to_string());
            written.insert((cover.artwork_code.clone(), cover.position), (relative.clone(), gallery_id.clone()));
            let id = artwork_ids.get(cover.artwork_code.as_str()).ok_or_else(relation_error)?;
            // The existing media catalogue registers uncatalogued artwork references on opening.
            if let Some(gallery_id) = gallery_id {
                exactly_one(tx.execute("INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order,kind) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![gallery_id, id, relative, cover.caption, stamp, cover.position, cover.kind]))?;
            } else {
                exactly_one(tx.execute("UPDATE artworks SET image_path=?2 WHERE id=?1 AND image_path IS NULL AND image_media_id IS NULL", params![id, relative]))?;
            }
        }
        // Relation verification must follow cover writes, so a cover-trigger cannot corrupt a sale after its checks.
        import_relations(&tx, &parsed, &artwork_ids, &stamp)?;
        // Check final state after the whole batch, including triggers that ignored or altered writes.
        for (id, row) in ids.iter().zip(rows) {
            let valid: bool = tx.query_row(VERIFY, params![id, row.code, row.title, row.year, row.technique,
                row.support, row.width_cm, row.height_cm, row.depth_cm, row.description, row.location,
                row.status, stamp, row.signed, row.final_varnish, row.framed,
                row.artist_coefficient_override, row.fixed_price_cents, written.get(&(row.code.clone(), 0)).map(|(path,_)| path)], |r| r.get(0)).map_err(database_error)?;
            if !valid { return Err("database".into()); }
            let expected = parsed.covers.iter().filter(|p| p.artwork_code == row.code && p.position > 0).count() as i64;
            let actual: i64 = tx.query_row("SELECT count(*) FROM artwork_images WHERE artwork_id=?1", [id], |r| r.get(0)).map_err(database_error)?;
            if actual != expected { return Err("database".into()); }
        }
        for cover in &parsed.covers {
            let root = directory.ok_or_else(|| "database".to_string())?;
            let (relative, gallery_id) = written.get(&(cover.artwork_code.clone(), cover.position)).ok_or_else(relation_error)?;
            if let Some(gallery_id) = gallery_id {
                let id = artwork_ids.get(cover.artwork_code.as_str()).ok_or_else(relation_error)?;
                let valid: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM artwork_images WHERE id=?1 AND artwork_id=?2 AND image_path IS ?3 AND caption IS ?4 AND created_at IS ?5 AND sort_order IS ?6 AND kind IS ?7 AND media_id IS NULL)",
                    params![gallery_id, id, relative, cover.caption, stamp, cover.position, cover.kind], |r| r.get(0)).map_err(database_error)?;
                if !valid { return Err("database".into()); }
            }
            let file = root.join(relative);
            let meta = std::fs::symlink_metadata(&file).map_err(database_error)?;
            if !meta.is_file() || meta.file_type().is_symlink() || meta.len() != cover.bytes.len() as u64 {
                return Err("database".into());
            }
            let stored = std::fs::read(&file).map_err(database_error)?;
            if format!("{:x}", Sha256::digest(&stored)) != cover.sha256 { return Err("database".into()); }
        }
        tx.commit().map_err(database_error)?;
        Ok(rows.len())
    })();
    if outcome.is_err() {
        if let Some(root) = directory {
            for (relative, _) in written.values() { let _ = std::fs::remove_file(root.join(relative)); }
        }
    }
    outcome
}

// Historical figures are explicitly supplied, not inferred from today's split or payment settings.
const INSERT_COLLECTOR: &str = "INSERT INTO collectors(id,name,first_name,last_name,email,phone,address,notes,birth_date,birth_time,birth_place,instagram,facebook,tiktok,website,linkedin,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?17)";
const VERIFY_COLLECTOR: &str = "SELECT EXISTS(SELECT 1 FROM collectors WHERE id=?1 AND name IS ?2 AND first_name IS ?3 AND last_name IS ?4 AND email IS ?5 AND phone IS ?6 AND address IS ?7 AND notes IS ?8 AND created_at IS ?17 AND updated_at IS ?17 AND birth_date IS ?9 AND birth_time IS ?10 AND birth_place IS ?11 AND instagram IS ?12 AND facebook IS ?13 AND tiktok IS ?14 AND website IS ?15 AND linkedin IS ?16 AND NOT EXISTS(SELECT 1 FROM collector_images WHERE collector_id=?1))";
const INSERT_SALE: &str = "INSERT INTO sales(id,artwork_id,collector_id,amount_cents,currency,sold_at,notes,created_at) VALUES(?1,?2,?3,?4,'EUR',?5,?6,?7)";
const VERIFY_SALE: &str = "SELECT EXISTS(SELECT 1 FROM sales WHERE id=?1 AND artwork_id=?2 AND collector_id=?3 AND amount_cents IS ?4 AND currency='EUR' AND sold_at IS ?5 AND notes IS ?6 AND created_at IS ?7 AND NOT EXISTS(SELECT 1 FROM sale_allocations WHERE sale_id=?1) AND (SELECT count(*) FROM sale_installments WHERE sale_id=?1)=?9 AND (SELECT count(*) FROM sale_receipts WHERE sale_id=?1)=?8)";
const INSERT_FINANCE: &str = "INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp) VALUES(?1,?2,?3,?4,?5,?6)";
const VERIFY_FINANCE: &str = "SELECT EXISTS(SELECT 1 FROM sale_finances WHERE sale_id=?1 AND tax_cents IS ?2 AND shipping_cents IS ?3 AND personal_bp IS ?4 AND activity_bp IS ?5 AND savings_bp IS ?6)";
const INSERT_INSTALLMENT: &str = "INSERT INTO sale_installments(id,sale_id,position,due_date,amount_cents,notes) VALUES(?1,?2,?3,?4,?5,?6)";
const VERIFY_INSTALLMENT: &str = "SELECT EXISTS(SELECT 1 FROM sale_installments WHERE id=?1 AND sale_id=?2 AND position IS ?3 AND due_date IS ?4 AND amount_cents IS ?5 AND notes IS ?6)";
const INSERT_RECEIPT: &str = "INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,notes,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)";
const VERIFY_RECEIPT: &str = "SELECT EXISTS(SELECT 1 FROM sale_receipts WHERE id=?1 AND sale_id=?2 AND received_at IS ?3 AND amount_cents IS ?4 AND method IS ?5 AND reference IS ?6 AND notes IS ?7 AND created_at IS ?8 AND NOT EXISTS(SELECT 1 FROM sale_receipt_attachments WHERE receipt_id=?1))";

fn exactly_one(result: rusqlite::Result<usize>) -> Result<(), String> {
    if result.map_err(database_error)? != 1 { return Err("database".into()); }
    Ok(())
}
fn verified(result: rusqlite::Result<bool>) -> Result<(), String> {
    if !result.map_err(database_error)? { return Err("database".into()); }
    Ok(())
}
fn import_relations(conn: &Connection, parsed: &Parsed, artwork_ids: &BTreeMap<&str,&str>, stamp: &str) -> Result<(), String> {
    let collector_ids: BTreeMap<&str,String> = parsed.collectors.iter().map(|c| (c.key.as_str(),Uuid::new_v4().to_string())).collect();
    let sale_ids: Vec<String> = parsed.sales.iter().map(|_| Uuid::new_v4().to_string()).collect();
    let receipt_ids: Vec<Vec<String>> = parsed.sales.iter().map(|s| s.receipts.iter().map(|_| Uuid::new_v4().to_string()).collect()).collect();
    let installment_ids: Vec<Vec<String>> = parsed.sales.iter().map(|s| s.installments.iter().map(|_| Uuid::new_v4().to_string()).collect()).collect();
    for c in &parsed.collectors {
        exactly_one(conn.execute(INSERT_COLLECTOR, params![collector_ids[ c.key.as_str() ],c.name(),c.first_name,c.last_name,c.email,c.phone,c.address,c.notes,c.birth_date,c.birth_time,c.birth_place,c.instagram,c.facebook,c.tiktok,c.website,c.linkedin,stamp]))?;
    }
    for (i, ((sale,id), receipts)) in parsed.sales.iter().zip(&sale_ids).zip(&receipt_ids).enumerate() {
        let a = artwork_ids.get(sale.artwork_code.as_str()).ok_or_else(relation_error)?;
        let c = collector_ids.get(sale.collector_key.as_str()).ok_or_else(relation_error)?;
        exactly_one(conn.execute(INSERT_SALE,params![id,a,c,sale.amount_cents,sale.sold_at,sale.notes,stamp]))?;
        exactly_one(conn.execute(INSERT_FINANCE,params![id,sale.tax_cents,sale.shipping_cents,sale.legacy_shares_bp[0],sale.legacy_shares_bp[1],sale.legacy_shares_bp[2]]))?;
        for (item,iid) in sale.installments.iter().zip(&installment_ids[i]) {
            exactly_one(conn.execute(INSERT_INSTALLMENT,params![iid,id,item.position,item.due_date,item.amount_cents,item.notes]))?;
        }
        for (receipt,rid) in sale.receipts.iter().zip(receipts) {
            exactly_one(conn.execute(INSERT_RECEIPT,params![rid,id,receipt.received_at,receipt.amount_cents,receipt.method,receipt.reference,receipt.notes,stamp]))?;
        }
    }
    // Verify after every write: a later trigger must not corrupt earlier records.
    for c in &parsed.collectors {
        verified(conn.query_row(VERIFY_COLLECTOR,params![collector_ids[c.key.as_str()],c.name(),c.first_name,c.last_name,c.email,c.phone,c.address,c.notes,c.birth_date,c.birth_time,c.birth_place,c.instagram,c.facebook,c.tiktok,c.website,c.linkedin,stamp],|r| r.get(0)))?;
    }
    for (i, ((sale,id), receipts)) in parsed.sales.iter().zip(&sale_ids).zip(&receipt_ids).enumerate() {
        let a = artwork_ids.get(sale.artwork_code.as_str()).ok_or_else(relation_error)?;
        let c = collector_ids.get(sale.collector_key.as_str()).ok_or_else(relation_error)?;
        verified(conn.query_row(VERIFY_SALE,params![id,a,c,sale.amount_cents,sale.sold_at,sale.notes,stamp,sale.receipts.len() as i64,sale.installments.len() as i64],|r| r.get(0)))?;
        verified(conn.query_row(VERIFY_FINANCE,params![id,sale.tax_cents,sale.shipping_cents,sale.legacy_shares_bp[0],sale.legacy_shares_bp[1],sale.legacy_shares_bp[2]],|r| r.get(0)))?;
        for (item,iid) in sale.installments.iter().zip(&installment_ids[i]) {
            verified(conn.query_row(VERIFY_INSTALLMENT,params![iid,id,item.position,item.due_date,item.amount_cents,item.notes],|r| r.get(0)))?;
        }
        for (receipt,rid) in sale.receipts.iter().zip(receipts) {
            verified(conn.query_row(VERIFY_RECEIPT,params![rid,id,receipt.received_at,receipt.amount_cents,receipt.method,receipt.reference,receipt.notes,stamp],|r| r.get(0)))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn preview_artwork_metadata(db: State<'_, Database>, source: String) -> Result<Preview, String> {
    let conn = db.conn.lock().map_err(database_error)?;
    preview(&conn, &source)
}
#[tauri::command]
pub fn import_artwork_metadata(db: State<'_, Database>, source: String, expected_digest: String) -> Result<usize, String> {
    let mut conn = db.conn.lock().map_err(database_error)?;
    apply_with_media(&mut conn, Some(&db.directory), &source, &expected_digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn package(rows: serde_json::Value) -> String {
        json!({"format": FORMAT, "artworks": rows}).to_string()
    }
    fn sample() -> String {
        package(json!([
            {"code":"TEST-01","title":" Étude ","status":"available","width_cm":30.5,"signed":true},
            {"code":"TEST-02","title":"Second","status":"archived","fixed_price_cents":12345}
        ]))
    }
    fn with_db(f: impl FnOnce(&mut Connection)) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().to_path_buf()).unwrap();
        let mut conn = db.conn.lock().unwrap();
        f(&mut conn);
    }
    fn count(conn: &Connection) -> i64 {
        conn.query_row("SELECT count(*) FROM artworks", [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn preview_is_read_only_and_import_is_atomic_and_repeat_safe() {
        with_db(|conn| {
            let source = sample();
            let plan = preview(conn, &source).unwrap();
            assert_eq!(plan.artworks[0].title, "Étude");
            assert_eq!(count(conn), 0);
            assert_eq!(apply(conn, &source, &plan.digest).unwrap(), 2);
            assert_eq!(count(conn), 2);
            let stored: (String, f64, bool) = conn.query_row(
                "SELECT title,width_cm,signed FROM artworks WHERE code='TEST-01'", [],
                |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
            assert_eq!(stored, ("Étude".into(),30.5,true));
            assert_eq!(apply(conn, &source, &plan.digest).unwrap_err(), "duplicate:1");
            assert_eq!(count(conn), 2);
        });
    }
    #[test]
    fn rejects_duplicates_and_unsupported_or_invalid_fields() {
        for rows in [
            json!([{"code":"A","title":"One","status":"available"},{"code":"a","title":"Two","status":"available"}]),
            json!([{"code":"A","title":"One","status":"sold"}]),
            json!([{"code":"A","title":"One","status":"available","image_path":"../secret"}]),
            json!([{"code":"A","title":"One","status":"available","width_cm":-5}]),
            json!([{"code":"A","title":" ","status":"available"}]),
            json!([{"code":"A","title":"One","status":"available","fixed_price_cents":1.5}]),
            json!([{"code":"A","title":"One","status":"available","description":"\u{0000}"}]),
            json!([]),
        ] { assert!(parse(&package(rows)).is_err()); }
        assert!(parse(&" ".repeat(MAX_BYTES + 1)).is_err());
        assert!(parse(&package(json!((0..=MAX_ROWS).map(|i| json!({"code":i.to_string(),"title":"X","status":"available"})).collect::<Vec<_>>()))).is_err());
    }
    #[test]
    fn changed_source_or_conflict_after_preview_never_inserts_partial_rows() {
        with_db(|conn| {
            let source = sample();
            let plan = preview(conn, &source).unwrap();
            assert_eq!(apply(conn, &(source.clone()+" "), &plan.digest).unwrap_err(), "changed");
            let other = package(json!([{"code":"test-02","title":"Existing","status":"available"}]));
            apply(conn, &other, &digest(&other)).unwrap();
            assert_eq!(apply(conn, &source, &plan.digest).unwrap_err(), "duplicate:2");
            assert_eq!(count(conn), 1);
        });
    }
    #[test]
    fn imported_codes_preserve_global_and_yearly_high_water_marks_after_deletion() {
        with_db(|conn| {
            let source = package(json!([
                {"code":"art-0042","title":"Global","status":"available"},
                {"code":"ART.0007","title":"Lower","status":"available"},
                {"code":"ART_2024_0012","title":"Yearly","status":"available"},
                {"code":"custom legacy 99","title":"Custom","status":"available"}
            ]));
            conn.execute("INSERT INTO archive_code_sequences(scope_key,last_value) VALUES('ART|GLOBAL',50)", []).unwrap();
            let plan = preview(conn, &source).unwrap();
            assert_eq!(conn.query_row("SELECT count(*) FROM archive_code_sequences", [], |r| r.get::<_,i64>(0)).unwrap(), 1);
            assert_eq!(apply(conn, &source, &plan.digest).unwrap(), 4);
            conn.execute("DELETE FROM artworks", []).unwrap();
            assert_eq!(crate::archive_code::preview(conn, None).unwrap(), "ART-0051");
            conn.execute("UPDATE archive_code_policy SET scope='YEARLY',separator='.',padding=3 WHERE singleton=1", []).unwrap();
            assert_eq!(crate::archive_code::preview(conn, Some("2024")).unwrap(), "ART.2024.013");
            assert_eq!(crate::archive_code::preview(conn, Some("2025")).unwrap(), "ART.2025.001");
            assert_eq!(conn.query_row("SELECT count(*) FROM archive_code_sequences", [], |r| r.get::<_,i64>(0)).unwrap(), 2);
        });
    }

    #[test]
    fn sequence_failure_rolls_back_the_entire_import_and_existing_counter() {
        for trigger in [
            "CREATE TRIGGER bad BEFORE INSERT ON archive_code_sequences WHEN NEW.scope_key='TEST|GLOBAL' BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER bad AFTER INSERT ON archive_code_sequences WHEN NEW.scope_key='TEST|GLOBAL' BEGIN UPDATE archive_code_sequences SET last_value=1 WHERE scope_key=NEW.scope_key; END;",
        ] {
            with_db(|conn| {
                conn.execute_batch(trigger).unwrap();
                conn.execute("INSERT INTO archive_code_sequences(scope_key,last_value) VALUES('ART|GLOBAL',50)", []).unwrap();
                let source = sample();
                assert_eq!(apply(conn, &source, &digest(&source)).unwrap_err(), "database");
                assert_eq!(count(conn), 0);
                assert_eq!(conn.query_row("SELECT count(*) FROM archive_code_sequences", [], |r| r.get::<_,i64>(0)).unwrap(), 1);
                assert_eq!(conn.query_row(READ_SEQUENCE, ["ART|GLOBAL"], |r| r.get::<_,i64>(0)).unwrap(), 50);
            });
        }
    }

    fn relational_sample() -> serde_json::Value {
        json!({"format":RELATIONAL_FORMAT,
            "artworks":[{"code":"HIST-01","title":"Sold work","status":"sold"},{"code":"HIST-02","title":"Available","status":"available"}],
            "collectors":[{"key":"legacy-1","first_name":"Ada","last_name":"Example","email":"ada@example.invalid"}],
            "sales":[{"artwork_code":"HIST-01","collector_key":"legacy-1","currency":"EUR","sold_at":"2025-03-04","amount_cents":10000,
                "tax_cents":1000,"shipping_cents":500,"legacy_shares_bp":[2000,3000,5000],"allocations":[],"installments":[],
                "receipts":[{"received_at":"2025-03-10","amount_cents":4000,"method":"bank_transfer","reference":"synthetic"}]}]})
    }
    #[test]
    fn relational_import_preserves_links_deductions_and_explicit_receipts() {
        with_db(|conn| {
            let source = relational_sample().to_string();
            let plan = preview(conn,&source).unwrap();
            assert_eq!(plan.collectors.len(),1);
            assert_eq!(plan.sales.len(),1);
            assert_eq!(count(conn),0);
            assert_eq!(apply(conn,&source,&plan.digest).unwrap(),2);
            let data: (String,String,i64,i64,i64,i64,i64) = conn.query_row(
                "SELECT a.status,c.name,s.amount_cents,f.tax_cents,f.shipping_cents,f.personal_bp,r.amount_cents FROM sales s JOIN artworks a ON a.id=s.artwork_id JOIN collectors c ON c.id=s.collector_id JOIN sale_finances f ON f.sale_id=s.id JOIN sale_receipts r ON r.sale_id=s.id",[],
                |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).unwrap();
            assert_eq!(data,("sold".into(),"Ada Example".into(),10000,1000,500,2000,4000));
            for table in ["sale_allocations","sale_installments"] {
                assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r| r.get::<_,i64>(0)).unwrap(),0);
            }
            assert!(apply(conn,&source,&plan.digest).is_err());
            assert_eq!(count(conn),2);
        });
    }
    fn unpaid_plan_sample() -> serde_json::Value {
        let mut value = relational_sample();
        value["sales"][0]["receipts"] = json!([]);
        value["sales"][0]["installments"] = json!([
            {"position":1,"due_date":"2025-03-31","amount_cents":4029,"notes":" first "},
            {"position":2,"due_date":"2025-04-30","amount_cents":5971,"notes":null}
        ]);
        value
    }
    #[test]
    fn collector_profile_import_preserves_details_and_rejects_tampering() {
        let mut value = relational_sample();
        let c = value["collectors"][0].as_object_mut().unwrap();
        for (key, val) in [("birth_date","1980-02-29"),("birth_time","07:45"),("birth_place","Example city"),("instagram","@example"),("facebook","example"),("tiktok","@example"),("website","https://example.invalid"),("linkedin","example-profile")] { c.insert(key.into(),json!(val)); }
        let source = value.to_string();
        with_db(|conn| {
            let plan = preview(conn,&source).unwrap();
            assert_eq!(plan.collectors[0].birth_time.as_deref(),Some("07:45"));
            assert_eq!(apply(conn,&source,&plan.digest).unwrap(),2);
            let stored: (String,String,String,String,String,String,String,String) = conn.query_row("SELECT birth_date,birth_time,birth_place,instagram,facebook,tiktok,website,linkedin FROM collectors",[],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).unwrap();
            assert_eq!(stored,("1980-02-29".into(),"07:45".into(),"Example city".into(),"@example".into(),"example".into(),"@example".into(),"https://example.invalid".into(),"example-profile".into()));
        });
        with_db(|conn| {
            let plan=preview(conn,&source).unwrap();
            conn.execute_batch("CREATE TRIGGER bad AFTER INSERT ON sale_receipts BEGIN UPDATE collectors SET birth_time='09:00'; END;").unwrap();
            assert_eq!(apply(conn,&source,&plan.digest).unwrap_err(),"database");
            for table in ["artworks","collectors","sales","sale_receipts","archive_code_sequences"] {
                assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get::<_,i64>(0)).unwrap(),0);
            }
        });
        for (key,val) in [("birth_date","1981-02-29"),("birth_time","07:45:01"),("birth_time","24:00")] {
            let mut bad=value.clone(); bad["collectors"][0][key]=json!(val);
            assert!(parse(&bad.to_string()).is_err());
        }
    }
    #[test]
    fn unpaid_plan_preserves_schedule_and_never_creates_receipts() {
        with_db(|conn| {
            let source = unpaid_plan_sample().to_string();
            let plan = preview(conn,&source).unwrap();
            assert_eq!(plan.sales[0].installments.len(),2);
            assert_eq!(plan.sales[0].installments[0].notes.as_deref(),Some("first"));
            assert_eq!(count(conn),0);
            assert_eq!(apply(conn,&source,&plan.digest).unwrap(),2);
            let mut stmt = conn.prepare("SELECT position,due_date,amount_cents,notes FROM sale_installments ORDER BY position").unwrap();
            let rows: Vec<(i64,String,i64,Option<String>)> = stmt.query_map([],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(rows,vec![(1,"2025-03-31".into(),4029,Some("first".into())),(2,"2025-04-30".into(),5971,None)]);
            assert_eq!(conn.query_row("SELECT count(*) FROM sale_receipts",[],|r| r.get::<_,i64>(0)).unwrap(),0);
            assert_eq!(conn.query_row("SELECT count(*) FROM sale_installments i JOIN sales s ON s.id=i.sale_id JOIN artworks a ON a.id=s.artwork_id WHERE a.code='HIST-01'",[],|r| r.get::<_,i64>(0)).unwrap(),2);
        });
    }
    #[test]
    fn rejects_ambiguous_or_invalid_installments_before_writing() {
        let original = unpaid_plan_sample();
        let mut invalid = vec![];
        for (key,value) in [("position",json!(3)),("due_date",json!("2025-02-28")),("amount_cents",json!(5970)),("notes",json!("x".repeat(501)))] {
            let mut v=original.clone(); v["sales"][0]["installments"][1][key]=value; invalid.push(v);
        }
        let mut v=original.clone();
        v["sales"][0]["receipts"]=relational_sample()["sales"][0]["receipts"].clone();
        invalid.push(v);
        let mut v=original.clone();
        v["sales"][0]["installments"].as_array_mut().unwrap().pop(); invalid.push(v);
        with_db(|conn| {
            for v in invalid {
                assert!(preview(conn,&v.to_string()).is_err());
                assert!(apply(conn,&v.to_string(),&digest(&v.to_string())).is_err());
            }
            assert_eq!(count(conn),0);
        });
    }
    #[test]
    fn installment_failure_rolls_back_all_relations_and_sequences() {
        for trigger in [
            "CREATE TRIGGER bad BEFORE INSERT ON sale_installments WHEN NEW.position=2 BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER bad AFTER INSERT ON sale_installments WHEN NEW.position=2 BEGIN UPDATE sale_installments SET amount_cents=1 WHERE position=1; END;",
            "CREATE TRIGGER bad AFTER INSERT ON sale_installments WHEN NEW.position=2 BEGIN UPDATE sale_installments SET notes='changed' WHERE position=2; END;",
        ] {
            with_db(|conn| {
                let source=unpaid_plan_sample().to_string();
                let plan=preview(conn,&source).unwrap();
                conn.execute_batch(trigger).unwrap();
                assert_eq!(apply(conn,&source,&plan.digest).unwrap_err(),"database");
                for table in ["artworks","collectors","sales","sale_finances","sale_receipts","sale_installments","archive_code_sequences"] {
                    assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r| r.get::<_,i64>(0)).unwrap(),0);
                }
            });
        }
    }
    #[test]
    fn invalid_relations_reject_missing_links_overpayment_and_unsupported_history() {
        let original = relational_sample();
        let mut variants = vec![];
        for (key,value) in [("collector_key",json!("missing")),("artwork_code",json!("HIST-02")),
            ("sold_at",json!("2025-02-30")),("legacy_shares_bp",json!([10000,1,0])),
            ("allocations",json!([{}])),("installments",json!([{}])),("currency",json!("USD"))] {
            let mut v=original.clone(); v["sales"][0][key]=value; variants.push(v);
        }
        let mut v=original.clone(); v["sales"][0]["receipts"][0]["amount_cents"]=json!(10001); variants.push(v);
        let mut v=original.clone(); v["sales"]=json!([]); variants.push(v);
        let mut v=original.clone(); v["sales"][0].as_object_mut().unwrap().remove("receipts"); variants.push(v);
        for v in variants { assert!(parse(&v.to_string()).is_err()); }
    }
    #[test]
    fn relational_import_rolls_back_when_later_write_is_ignored_or_tampered() {
        for trigger in [
            "CREATE TRIGGER bad BEFORE INSERT ON sale_receipts BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER bad AFTER INSERT ON sale_receipts BEGIN UPDATE collectors SET name='changed'; END;",
            "CREATE TRIGGER bad AFTER INSERT ON sale_receipts BEGIN UPDATE sale_finances SET tax_cents=0; END;",
        ] {
            with_db(|conn| {
                let source=relational_sample().to_string();
                let plan=preview(conn,&source).unwrap();
                conn.execute_batch(trigger).unwrap();
                assert_eq!(apply(conn,&source,&plan.digest).unwrap_err(),"database");
                for table in ["artworks","collectors","sales","sale_finances","sale_receipts","archive_code_sequences"] {
                    assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r| r.get::<_,i64>(0)).unwrap(),0);
                }
            });
        }
    }
    #[test]
    fn collector_conflict_after_preview_does_not_overwrite_existing_data() {
        with_db(|conn| {
            let source=relational_sample().to_string();
            let plan=preview(conn,&source).unwrap();
            conn.execute("INSERT INTO collectors(id,name,email,created_at,updated_at) VALUES('existing','Different','ADA@example.invalid','t','t')",[]).unwrap();
            assert_eq!(apply(conn,&source,&plan.digest).unwrap_err(),"collector_conflict");
            assert_eq!(count(conn),0);
            assert_eq!(conn.query_row("SELECT count(*) FROM collectors",[],|r| r.get::<_,i64>(0)).unwrap(),1);
        });
    }
    #[test]
    fn ignored_or_tampered_second_insert_rolls_back_first() {
        for trigger in [
            "CREATE TRIGGER bad BEFORE INSERT ON artworks WHEN NEW.code='TEST-02' BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER bad AFTER INSERT ON artworks WHEN NEW.code='TEST-02' BEGIN UPDATE artworks SET title='tampered' WHERE code='TEST-01'; END;",
            "CREATE TRIGGER bad AFTER INSERT ON artworks WHEN NEW.code='TEST-02' BEGIN UPDATE artworks SET code='test-01' WHERE code='TEST-01'; END;",
        ] {
            with_db(|conn| {
                conn.execute_batch(trigger).unwrap();
                let source = sample();
                assert_eq!(apply(conn, &source, &digest(&source)).unwrap_err(), "database");
                assert_eq!(count(conn), 0);
            });
        }
    }

    #[test]
    fn small_cover_is_verified_and_rolled_back_with_archive() {
        // Generate valid PNG using the existing image dependency instead of trusting a fixture header.
        let mut cursor = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1,1).write_to(&mut cursor,image::ImageFormat::Png).unwrap();
        let bytes = cursor.into_inner();
        let mut value = relational_sample();
        value["covers"] = json!([{"artwork_code":"HIST-01","sha256":format!("{:x}",Sha256::digest(&bytes)),"data_base64":STANDARD.encode(&bytes)}]);
        let source=value.to_string();
        for trigger in [None, Some("CREATE TRIGGER bad BEFORE UPDATE OF image_path ON artworks BEGIN SELECT RAISE(IGNORE); END;"), Some("CREATE TRIGGER bad AFTER UPDATE OF image_path ON artworks BEGIN UPDATE artworks SET image_path=NULL WHERE id=NEW.id; END;")] {
            let dir=tempfile::tempdir().unwrap();
            let db=crate::db::open(dir.path().to_path_buf()).unwrap();
            let mut conn=db.conn.lock().unwrap();
            let plan=preview(&conn,&source).unwrap();
            assert_eq!(plan.covers.len(),1);
            assert_eq!(std::fs::read_dir(dir.path().join("media")).unwrap().count(),0);
            if let Some(sql)=trigger { conn.execute_batch(sql).unwrap(); }
            let result=apply_with_media(&mut conn,Some(dir.path()),&source,&plan.digest);
            if trigger.is_some() {
                assert!(result.is_err());
                assert_eq!(count(&conn),0);
                assert_eq!(conn.query_row("SELECT count(*) FROM sales",[],|r|r.get::<_,i64>(0)).unwrap(),0);
                assert_eq!(std::fs::read_dir(dir.path().join("media")).unwrap().count(),0);
            } else {
                assert_eq!(result.unwrap(),2);
                let path:String=conn.query_row("SELECT image_path FROM artworks WHERE code='HIST-01'",[],|r|r.get(0)).unwrap();
                assert_eq!(std::fs::read(dir.path().join(path)).unwrap(),bytes);
            }
        }
        let mut bad=value.clone(); bad["covers"][0]["sha256"]=json!("0".repeat(64));
        assert!(parse(&bad.to_string()).is_err());
        let mut bad=value.clone(); bad["covers"][0]["artwork_code"]=json!("missing");
        assert!(parse(&bad.to_string()).is_err());
        let mut bad=value.clone(); bad["covers"].as_array_mut().unwrap().push(value["covers"][0].clone());
        assert!(parse(&bad.to_string()).is_err());
    }
    #[test]
    fn gallery_import_preserves_order_and_rolls_back_ignored_or_tampered_rows() {
        let mut cursor = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1,1).write_to(&mut cursor,image::ImageFormat::Png).unwrap();
        let bytes = cursor.into_inner();
        let photo = |position, caption, kind| json!({"artwork_code":"HIST-01","sha256":format!("{:x}",Sha256::digest(&bytes)),"data_base64":STANDARD.encode(&bytes),"position":position,"caption":caption,"kind":kind});
        let mut value = relational_sample();
        value["gallery"] = json!([photo(2,"Second","DETAIL"),photo(1,"First","SIGNATURE")]);
        let source = value.to_string();
        for trigger in [None,
            Some("CREATE TRIGGER bad BEFORE INSERT ON artwork_images WHEN NEW.sort_order=2 BEGIN SELECT RAISE(IGNORE); END;"),
            Some("CREATE TRIGGER bad AFTER INSERT ON artwork_images WHEN NEW.sort_order=2 BEGIN UPDATE artwork_images SET caption='tampered' WHERE sort_order=1; END;"),
            Some("CREATE TRIGGER bad AFTER INSERT ON artwork_images WHEN NEW.sort_order=2 BEGIN UPDATE artwork_images SET sort_order=3 WHERE id=NEW.id; END;")] {
            let dir=tempfile::tempdir().unwrap();
            let db=crate::db::open(dir.path().to_path_buf()).unwrap();
            let mut conn=db.conn.lock().unwrap();
            let plan=preview(&conn,&source).unwrap();
            assert_eq!(plan.covers.iter().map(|p|p.position).collect::<Vec<_>>(),vec![1,2]);
            if let Some(sql)=trigger { conn.execute_batch(sql).unwrap(); }
            let result=apply_with_media(&mut conn,Some(dir.path()),&source,&plan.digest);
            if trigger.is_some() {
                assert!(result.is_err());
                assert_eq!(count(&conn),0);
                assert_eq!(conn.query_row("SELECT count(*) FROM artwork_images",[],|r|r.get::<_,i64>(0)).unwrap(),0);
                assert_eq!(conn.query_row("SELECT count(*) FROM sales",[],|r|r.get::<_,i64>(0)).unwrap(),0);
                assert_eq!(std::fs::read_dir(dir.path().join("media")).unwrap().count(),0);
            } else {
                assert_eq!(result.unwrap(),2);
                let mut stmt=conn.prepare("SELECT image_path,caption,kind,sort_order FROM artwork_images ORDER BY sort_order").unwrap();
                let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
                assert_eq!(rows.len(),2);
                assert_eq!((&rows[0].1[..],&rows[0].2[..],rows[0].3),("First","SIGNATURE",1));
                assert_eq!((&rows[1].1[..],&rows[1].2[..],rows[1].3),("Second","DETAIL",2));
                assert_ne!(rows[0].0,rows[1].0);
                for row in rows { assert_eq!(std::fs::read(dir.path().join(row.0)).unwrap(),bytes); }
            }
        }
        for invalid in [json!([photo(2,"Missing first","OTHER")]),json!([photo(1,"A","OTHER"),photo(1,"B","DETAIL")]),json!([photo(1,"A","INVALID")]),json!([photo(21,"A","OTHER")])] {
            let mut bad=value.clone(); bad["gallery"]=invalid;
            assert!(parse(&bad.to_string()).is_err());
        }
    }

}
