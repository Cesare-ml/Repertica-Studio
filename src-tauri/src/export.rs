//! Offline CSV exports of artwork, collector, certificate, sale, receipt and expense registers.
//! CSV exports are for inspection/analysis, NOT a restore format or verified cash receipts.
//! All money is exported as integer cents to avoid rounding or locale ambiguity.
use crate::db::Database;
use rusqlite::{types::ValueRef, Connection, OptionalExtension};
use std::{fs::{self, OpenOptions}, io::Write, path::Path};
use tauri::State;

fn error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

// Spreadsheet formula injection is possible even inside quoted CSV cells.
// Prefix user-controlled strings that Excel/LibreOffice could evaluate.
fn append_text_cell(output: &mut String, value: &str) {
    let trimmed = value.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{feff}');
    output.push('"');
    if matches!(trimmed.chars().next(), Some('=' | '+' | '-' | '@')) {
        output.push('\'');
    }
    for ch in value.chars() {
        if ch == '"' { output.push('"'); }
        output.push(ch);
    }
    output.push('"');
}

// An external SQLite import can contain impossible/non-canonical dates even
// when foreign keys and page integrity pass. Reject them before publishing a
// CSV: spreadsheet sorting and month-based accounting would be misleading.
// Only check the date actually represented by the requested register.
fn validate_export_dates(conn: &Connection, kind: &str) -> Result<(), String> {
    let (table, date_column, label) = match kind {
        "sales" | "balances" | "allocations" => ("sales", "sold_at", "vendita"),
        "receipts" => ("sale_receipts", "received_at", "incasso"),
        "expenses" => ("expenses", "expense_date", "spesa"),
        "installments" => ("sale_installments", "due_date", "scadenza rata"),
        "certificates" => ("certificates", "issued_at", "emissione certificato"),
        _ => return Ok(()),
    };
    // All identifiers above are fixed literals, never interpolated user input.
    let sql = format!("SELECT id,{date_column} FROM {table} ORDER BY id");
    let mut statement = conn.prepare(&sql).map_err(error)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(error)?;
    for entry in rows {
        let (id, date) = entry.map_err(error)?;
        let canonical = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map(|parsed| parsed.format("%Y-%m-%d").to_string() == date)
            .unwrap_or(false);
        if !canonical {
            return Err(format!(
                "Esportazione annullata: data di {label} non valida ({id}). Verifica l'archivio."
            ));
        }
    }
    Ok(())
}

// CSV must not silently lose records when foreign keys were disabled during
// an external SQLite import. INNER JOIN would otherwise omit orphaned rows.
fn validate_export_relations(conn: &Connection, kind: &str) -> Result<(), String> {
    let broken: Option<(String, Option<i64>, String)> = conn.query_row(
        "SELECT \"table\", rowid, parent FROM pragma_foreign_key_check LIMIT 1",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional().map_err(error)?;
    if let Some((table, rowid, parent)) = broken {
        return Err(format!(
            "Esportazione annullata: relazione SQLite non valida nella tabella {table} (riga {rowid:?}, riferimento a {parent}). Verifica o ripristina l'archivio prima di esportare."
        ));
    }
    validate_export_dates(conn, kind)?;
    // Missing dependent sale_finances rows do not violate foreign keys. Do not
    // use COALESCE to invent taxes and shipping in financial exports.
    if matches!(kind, "sales" | "allocations") {
        let missing_sale: Option<String> = conn.query_row(
            "SELECT s.id FROM sales s WHERE NOT EXISTS
                (SELECT 1 FROM sale_finances f WHERE f.sale_id=s.id)
             ORDER BY s.id LIMIT 1",
            [], |row| row.get(0)
        ).optional().map_err(error)?;
        if let Some(id) = missing_sale {
            return Err(format!(
                "Esportazione annullata: dati finanziari mancanti per la vendita {id}. Nessuna imposta o spedizione è stata stimata."
            ));
        }
    }
    // Imported SQLite data may satisfy foreign keys but contain deductions
    // larger than the price, or CHECK constraints bypassed by external tools.
    // Never export an apparently authoritative negative net sale. Compute in
    // i64 with checked arithmetic so even corrupt extreme values cannot wrap.
    if matches!(kind, "sales" | "allocations") {
        let mut stmt = conn.prepare(
            "SELECT s.id,s.amount_cents,f.tax_cents,f.shipping_cents
             FROM sales s JOIN sale_finances f ON f.sale_id=s.id
             ORDER BY s.id"
        ).map_err(error)?;
        let entries = stmt.query_map([], |row| Ok((
            row.get::<_, String>(0)?, row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?, row.get::<_, i64>(3)?,
        ))).map_err(error)?;
        for entry in entries {
            let (sale_id, gross, taxes, shipping) = entry.map_err(error)?;
            let net = gross.checked_sub(taxes).and_then(|v| v.checked_sub(shipping));
            if gross <= 0 || taxes < 0 || shipping < 0 || net.is_none_or(|v| v < 0) {
                return Err(format!(
                    "Esportazione annullata: imposte, spedizione o importo incoerenti per la vendita {sale_id}. Verifica l'archivio prima di esportare."
                ));
            }
        }
    }
    // An external SQLite import can bypass the receipt limit triggers. A CSV
    // reporting negative outstanding money would be misleading even though all
    // foreign keys still resolve and the underlying database is readable.
    if matches!(kind, "balances" | "receipts") {
        let invalid_balance: Option<String> = conn.query_row(
            "SELECT s.id FROM sales s
             LEFT JOIN (
                 SELECT sale_id, SUM(amount_cents) AS received_cents,
                        MIN(amount_cents) AS minimum_receipt
                 FROM sale_receipts GROUP BY sale_id
             ) totals ON totals.sale_id=s.id
             WHERE s.amount_cents <= 0
                OR COALESCE(totals.received_cents,0) NOT BETWEEN 0 AND s.amount_cents
                OR (totals.minimum_receipt IS NOT NULL AND totals.minimum_receipt <= 0)
             ORDER BY s.id LIMIT 1",
            [], |row| row.get(0)
        ).optional().map_err(error)?;
        if let Some(id) = invalid_balance {
            return Err(format!(
                "Esportazione annullata: incassi incoerenti con l'importo della vendita {id}. Verifica l'archivio prima di esportare."
            ));
        }
    }
    Ok(())
}

fn render_csv(conn: &Connection, kind: &str) -> Result<String, String> {
    // Deliberately export all rows, not the on-screen year filter. Each record
    // carries its date; no historic record is silently omitted.
    let (header, query) = match kind {
        "artworks" => (
            "artwork_id;code;title;year;technique;support;width_cm;height_cm;depth_cm;description;location;status;image_path;created_at;updated_at;signed;final_varnish;framed;artist_coefficient_override;fixed_price_cents",
            "SELECT id,code,title,year,technique,support,width_cm,height_cm,depth_cm,
                    description,location,status,image_path,created_at,updated_at,signed,final_varnish,framed,artist_coefficient_override,fixed_price_cents
             FROM artworks ORDER BY code COLLATE NOCASE,id"
        ),
        "gallery" => (
            "image_id;artwork_id;artwork_code;artwork_title;image_path;caption;sort_order;created_at;kind",
            "SELECT i.id,i.artwork_id,a.code,a.title,i.image_path,i.caption,i.sort_order,i.created_at,i.kind
             FROM artwork_images i JOIN artworks a ON a.id=i.artwork_id
             ORDER BY a.code COLLATE NOCASE,a.id,i.sort_order,i.created_at,i.id"
        ),
        "collectors" => (
            "collector_id;name;email;phone;address;notes;created_at;updated_at;first_name;last_name;birth_date;birth_time;birth_place;instagram;facebook;tiktok;website;linkedin;profile_photo_path;photo_count",
            "SELECT c.id,c.name,c.email,c.phone,c.address,c.notes,c.created_at,c.updated_at,
                    COALESCE(c.first_name,c.name),c.last_name,c.birth_date,c.birth_time,c.birth_place,
                    c.instagram,c.facebook,c.tiktok,c.website,c.linkedin,
                    (SELECT image_path FROM collector_images WHERE collector_id=c.id AND kind='PROFILE'),
                    (SELECT count(*) FROM collector_images WHERE collector_id=c.id)
             FROM collectors c ORDER BY c.name COLLATE NOCASE,c.id"
        ),
        "collector_photos" => (
            "photo_id;collector_id;collector_name;kind;image_path;media_id;created_at",
            "SELECT p.id,p.collector_id,c.name,p.kind,p.image_path,p.media_id,p.created_at
             FROM collector_images p JOIN collectors c ON c.id=p.collector_id
             ORDER BY c.name COLLATE NOCASE,c.id,
                      CASE p.kind WHEN 'PROFILE' THEN 0 ELSE 1 END,p.created_at,p.id"
        ),
        "raw_files" => (
            "artwork_id;artwork_code;artwork_title;media_path;original_name;size_bytes;sha256;created_at",
            "SELECT r.artwork_id,a.code,a.title,r.media_path,r.original_name,r.size_bytes,r.sha256,r.created_at
             FROM artwork_raw_files r JOIN artworks a ON a.id=r.artwork_id
             ORDER BY a.code COLLATE NOCASE,a.id"
        ),
        "certificates" => (
            "certificate_id;artwork_id;artwork_code;current_artwork_title;serial;issued_at;artist_name;collector_name;notes;snapshot_json;created_at;language;issued_place;declarant_name",
            "SELECT c.id,c.artwork_id,a.code,a.title,c.serial,c.issued_at,c.artist_name,
                    c.collector_name,c.notes,c.snapshot_json,c.created_at,c.language,c.issued_place,c.declarant_name
             FROM certificates c JOIN artworks a ON a.id=c.artwork_id
             ORDER BY c.issued_at,c.serial,c.id"
        ),
        "sales" => (
            "sale_id;sold_at;artwork_id;artwork_code;artwork_title;collector_id;collector_name;currency;gross_cents;tax_cents;shipping_cents;net_cents;personal_bp;activity_bp;savings_bp;notes;created_at",
            "SELECT s.id,s.sold_at,s.artwork_id,a.code,a.title,s.collector_id,c.name,s.currency,
                    s.amount_cents,COALESCE(f.tax_cents,0),COALESCE(f.shipping_cents,0),
                    s.amount_cents-COALESCE(f.tax_cents,0)-COALESCE(f.shipping_cents,0),
                    COALESCE((SELECT share_bp FROM sale_allocations WHERE sale_id=s.id AND part_id='personal'),0),
                    COALESCE((SELECT share_bp FROM sale_allocations WHERE sale_id=s.id AND part_id='activity'),0),
                    COALESCE((SELECT share_bp FROM sale_allocations WHERE sale_id=s.id AND part_id='savings'),0),
                    s.notes,s.created_at
             FROM sales s JOIN artworks a ON a.id=s.artwork_id
             JOIN collectors c ON c.id=s.collector_id
             LEFT JOIN sale_finances f ON f.sale_id=s.id
             ORDER BY s.sold_at,s.id"
        ),
        "balances" => (
            "sale_id;sold_at;artwork_id;artwork_code;artwork_title;collector_id;collector_name;currency;gross_cents;received_cents;outstanding_cents;created_at",
            "SELECT s.id,s.sold_at,s.artwork_id,a.code,a.title,s.collector_id,c.name,s.currency,
                    s.amount_cents,COALESCE(received.amount_cents,0),
                    s.amount_cents-COALESCE(received.amount_cents,0),s.created_at
             FROM sales s JOIN artworks a ON a.id=s.artwork_id
             JOIN collectors c ON c.id=s.collector_id
             LEFT JOIN (
                SELECT sale_id,SUM(amount_cents) AS amount_cents
                FROM sale_receipts GROUP BY sale_id
             ) received ON received.sale_id=s.id
             ORDER BY s.sold_at,s.id"
        ),
        "receipts" => (
            "receipt_id;sale_id;received_at;artwork_id;artwork_code;artwork_title;collector_id;collector_name;amount_cents;currency;method;reference;notes;created_at;attachment_path;attachment_original_name;attachment_mime;attachment_size_bytes;attachment_sha256;attachment_created_at",
            "SELECT r.id,r.sale_id,r.received_at,s.artwork_id,a.code,a.title,s.collector_id,c.name,
                    r.amount_cents,s.currency,r.method,r.reference,r.notes,r.created_at,
                    evidence.media_path,evidence.original_name,evidence.mime_type,
                    evidence.size_bytes,evidence.sha256,evidence.created_at
             FROM sale_receipts r JOIN sales s ON s.id=r.sale_id
             JOIN artworks a ON a.id=s.artwork_id
             JOIN collectors c ON c.id=s.collector_id
             LEFT JOIN sale_receipt_attachments evidence ON evidence.receipt_id=r.id
             ORDER BY r.received_at,r.created_at,r.id"
        ),
        "expenses" => (
            "expense_id;expense_date;category;description;amount_cents;fund;supplier;purchase_url;notes;created_at;updated_at;fund_name",
            "SELECT e.id,e.expense_date,e.category,e.description,e.amount_cents,e.fund,e.supplier,e.purchase_url,e.notes,
                    e.created_at,e.updated_at,p.name
             FROM expenses e LEFT JOIN allocation_parts p ON p.id=e.fund ORDER BY e.expense_date,e.id"
        ),
        "installments" => (
            "installment_id;sale_id;position;due_date;amount_cents;notes;artwork_code;artwork_title;collector_name",
            "SELECT i.id,i.sale_id,i.position,i.due_date,i.amount_cents,i.notes,a.code,a.title,c.name
             FROM sale_installments i JOIN sales s ON s.id=i.sale_id
             JOIN artworks a ON a.id=s.artwork_id JOIN collectors c ON c.id=s.collector_id
             ORDER BY i.due_date,i.sale_id,i.position"
        ),
        "allocations" => (
            "sale_id;sold_at;artwork_id;artwork_code;artwork_title;net_cents;part_id;part_name;share_bp",
            "SELECT s.id,s.sold_at,s.artwork_id,a.code,a.title,
                    s.amount_cents-COALESCE(f.tax_cents,0)-COALESCE(f.shipping_cents,0),
                    al.part_id,p.name,al.share_bp
             FROM sale_allocations al JOIN allocation_parts p ON p.id=al.part_id
             JOIN sales s ON s.id=al.sale_id JOIN artworks a ON a.id=s.artwork_id
             LEFT JOIN sale_finances f ON f.sale_id=s.id
             ORDER BY s.sold_at,s.id,p.position,p.id"
        ),
        _ => return Err("Tipo di registro CSV non riconosciuto".into()),
    };
    validate_export_relations(conn, kind)?;
    let mut out = String::from("\u{feff}"); // UTF-8 BOM for Excel
    out.push_str(header);
    out.push_str("\r\n");

    let mut stmt = conn.prepare(query).map_err(error)?;
    let columns = stmt.column_count();
    let mut rows = stmt.query([]).map_err(error)?;
    while let Some(row) = rows.next().map_err(error)? {
        for i in 0..columns {
            if i != 0 { out.push(';'); }
            match row.get_ref(i).map_err(error)? {
                ValueRef::Null => append_text_cell(&mut out, ""),
                ValueRef::Text(bytes) => {
                    let text = std::str::from_utf8(bytes).map_err(error)?;
                    append_text_cell(&mut out, text);
                }
                ValueRef::Integer(value) => out.push_str(&value.to_string()),
                ValueRef::Real(value) if value.is_finite() => out.push_str(&value.to_string()),
                ValueRef::Real(_) | ValueRef::Blob(_) => {
                    return Err("Il registro contiene un tipo SQLite non previsto".into());
                }
            }
        }
        out.push_str("\r\n");
    }
    Ok(out)
}

fn export_file(db: &Database, kind: &str, destination: &Path) -> Result<String, String> {
    if !destination.extension().and_then(|s| s.to_str()).is_some_and(|s| s.eq_ignore_ascii_case("csv")) {
        return Err("Seleziona un percorso con estensione .csv".into());
    }
    let filename = destination.file_name().ok_or("Nome file non valido")?;
    let parent = destination.parent().ok_or("Cartella di esportazione non valida")?;
    let parent = fs::canonicalize(parent).map_err(error)?;
    if !parent.is_dir() { return Err("La destinazione non è una cartella".into()); }
    let live = fs::canonicalize(&db.directory).map_err(error)?;
    if parent.starts_with(live) {
        return Err("Non esportare file nella cartella interna dell'archivio".into());
    }
    // Export a consistent point-in-time view while holding the same application
    // mutex as the backup and all CRUD commands. The file write is outside it.
    let csv = {
        let conn = db.conn.lock().map_err(error)?;
        render_csv(&conn, kind)?
    };
    let target = parent.join(filename);
    // Never truncate an existing document, DB, or an external backup by mistake.
    let mut file = OpenOptions::new().write(true).create_new(true).open(&target)
        .map_err(|e| format!("Impossibile creare il CSV (scegli un nuovo nome se esiste già): {e}"))?;
    let result = file.write_all(csv.as_bytes()).and_then(|_| file.sync_all());
    drop(file);
    if let Err(e) = result {
        let _ = fs::remove_file(&target);
        return Err(format!("Esportazione CSV incompleta: {e}"));
    }
    Ok(crate::util::display_path(&target))
}

#[tauri::command]
pub fn export_ledger_csv(db: State<'_, Database>, kind: String, destination: String) -> Result<String, String> {
    export_file(&db, &kind, Path::new(&destination))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;


    #[test]
    fn corrupted_imported_dates_never_become_authoritative_csv_registers() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','ART-1','Opera','t','t');
            INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c','Collezionista','t','t');
            INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',5000,'2026-11-01','t');
            INSERT INTO sale_finances(sale_id,personal_bp,activity_bp,savings_bp)
                VALUES('s',3334,3333,3333);
            INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                VALUES('r','s','2026-11-02',1000,'cash','t');
            INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at)
                VALUES('e','2026-11-02','materiali','Cornice',100,'activity','t','t');
            INSERT INTO sale_installments(id,sale_id,position,due_date,amount_cents)
                VALUES('i1','s',1,'2026-11-02',2500),
                      ('i2','s',2,'2026-12-02',2500);
            INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                VALUES('cert','a','RS-0001','2026-11-02','Artista','{}','t');
        "#).unwrap();
        for (table, column, id, kinds) in [
            ("sales", "sold_at", "s", &["sales", "balances", "allocations"][..]),
            ("sale_receipts", "received_at", "r", &["receipts"][..]),
            ("expenses", "expense_date", "e", &["expenses"][..]),
            ("sale_installments", "due_date", "i1", &["installments"][..]),
            ("certificates", "issued_at", "cert", &["certificates"][..]),
        ] {
            let update = format!("UPDATE {table} SET {column}=?1 WHERE id=?2");
            for bad_date in ["2026-02-30", "2026-2-3", "invalid"] {
                conn.execute(&update, rusqlite::params![bad_date, id]).unwrap();
                for kind in kinds {
                    let error = render_csv(&conn, kind).unwrap_err();
                    assert!(error.contains("data di"), "{kind}: {error}");
                    assert!(error.contains(id), "{kind}: {error}");
                }
            }
            conn.execute(&update, rusqlite::params!["2026-11-02", id]).unwrap();
            for kind in kinds {
                assert!(render_csv(&conn, kind).is_ok(), "{kind} should recover");
            }
        }
        // The native export must reject before opening a destination file.
        conn.execute("UPDATE expenses SET expense_date='2026-02-30' WHERE id='e'", []).unwrap();
        drop(conn);
        let destination = dir.path().join("expenses.csv");
        let failure = export_file(&db, "expenses", &destination).unwrap_err();
        assert!(failure.contains("data di spesa"), "{failure}");
        assert!(!destination.exists(), "no partial CSV should be published");
    }

    #[test]
    fn csv_rejects_orphaned_sale_instead_of_omitting_it_from_joined_registers() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            // Simulate third-party SQLite changes made with FK checks disabled.
            conn.execute_batch(r#"
                PRAGMA foreign_keys=OFF;
                INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES('a','A-1','Opera','t','t');
                INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                    VALUES('s','a','missing-collector',4200,'2026-11-01','t');
                PRAGMA foreign_keys=ON;
            "#).unwrap();
            let failure = render_csv(&conn, "balances").unwrap_err();
            assert!(failure.contains("relazione SQLite non valida"), "{failure}");
        }
        let output = dir.path().join("balances.csv");
        assert!(export_file(&db, "balances", &output).is_err());
        assert!(!output.exists(), "do not create a misleading partial CSV");
        let conn = db.conn.lock().unwrap();
        conn.execute("INSERT INTO collectors(id,name,created_at,updated_at)
            VALUES('missing-collector','Anna','t','t')", []).unwrap();
        let restored = render_csv(&conn, "balances").unwrap();
        assert!(restored.contains("\"s\";\"2026-11-01\";\"a\";"), "{restored}");
    }

    #[test]
    fn financial_csv_does_not_invent_zero_taxes_when_sale_finances_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A-1','Opera','t','t');
            INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c','Anna','t','t');
            INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',4200,'2026-11-01','t');
        "#).unwrap();
        for kind in ["sales", "allocations"] {
            let failure = render_csv(&conn, kind).unwrap_err();
            assert!(failure.contains("vendita s"), "{kind}: {failure}");
        }
        // Genuine zero tax/shipping values are allowed once the row exists.
        conn.execute("INSERT INTO sale_finances(sale_id,personal_bp,activity_bp,savings_bp)
            VALUES('s',3334,3333,3333)", []).unwrap();
        assert!(render_csv(&conn, "sales").unwrap().contains(";4200;0;0;4200;"));
        assert!(render_csv(&conn, "allocations").is_ok());
    }

    #[test]
    fn financial_csv_rejects_deductions_above_sale_amount_before_creating_file() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(r#"
                INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES('a','ART-1','Opera','t','t');
                INSERT INTO collectors(id,name,created_at,updated_at)
                    VALUES('c','Cliente','t','t');
                INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                    VALUES('s','a','c',4200,'2026-11-01','t');
                INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
                    VALUES('s',0,0,3334,3333,3333);
            "#).unwrap();
            for (taxes, shipping) in [(4201, 0), (2000, 2201)] {
                conn.execute(
                    "UPDATE sale_finances SET tax_cents=?1,shipping_cents=?2 WHERE sale_id='s'",
                    rusqlite::params![taxes, shipping],
                ).unwrap();
                for kind in ["sales", "allocations"] {
                    let failure = render_csv(&conn, kind).unwrap_err();
                    assert!(failure.contains("vendita s"), "{kind}: {failure}");
                    assert!(failure.contains("incoerenti"), "{kind}: {failure}");
                }
            }
        }
        let destination = dir.path().join("sales.csv");
        assert!(export_file(&db, "sales", &destination).is_err());
        assert!(!destination.exists(), "invalid money must never reach a CSV");
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "UPDATE sale_finances SET tax_cents=1200,shipping_cents=300 WHERE sale_id='s'",
            [],
        ).unwrap();
        assert!(render_csv(&conn, "sales").unwrap().contains(";4200;1200;300;2700;"));
        assert!(render_csv(&conn, "allocations").is_ok());
    }

    #[test]
    fn financial_csv_rejects_negative_imported_deductions_and_recovers() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','ART-1','Opera','t','t');
            INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c','Cliente','t','t');
            INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',4200,'2026-11-01','t');
            INSERT INTO sale_finances(sale_id,personal_bp,activity_bp,savings_bp)
                VALUES('s',3334,3333,3333);
            PRAGMA ignore_check_constraints=ON;
            UPDATE sale_finances SET tax_cents=-1 WHERE sale_id='s';
            PRAGMA ignore_check_constraints=OFF;
        "#).unwrap();
        for kind in ["sales", "allocations"] {
            let failure = render_csv(&conn, kind).unwrap_err();
            assert!(failure.contains("vendita s"), "{kind}: {failure}");
        }
        conn.execute("UPDATE sale_finances SET tax_cents=0 WHERE sale_id='s'", []).unwrap();
        assert!(render_csv(&conn, "sales").is_ok());
        assert!(render_csv(&conn, "allocations").is_ok());
    }

    #[test]
    fn receipts_csv_is_distinct_from_sales_and_preserves_references_and_exact_cents() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
            VALUES('art','ART-1','Landscape','t','t')", []).unwrap();
        conn.execute("INSERT INTO collectors(id,name,created_at,updated_at)
            VALUES('buyer','Client','t','t')", []).unwrap();
        conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
            VALUES('sale','art','buyer',10001,'2026-11-01','t')", []).unwrap();
        conn.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,notes,created_at)
            VALUES('r1','sale','2026-11-10',1500,'bank_transfer','  =2+3','comment;line','t')", []).unwrap();
        conn.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,reference,created_at)
            VALUES('r2','sale','2026-11-12',2001,'cash','cash-02','t')", []).unwrap();

        // Finance metadata must also exist for this test's sales export.
        conn.execute("INSERT INTO sale_finances(sale_id,personal_bp,activity_bp,savings_bp)
            VALUES('sale',3334,3333,3333)", []).unwrap();
        let csv = render_csv(&conn, "receipts").unwrap();
        assert!(csv.starts_with("\u{feff}receipt_id;sale_id;received_at;artwork_id;"));
        assert!(csv.contains("\"r1\";\"sale\";\"2026-11-10\";\"art\";\"ART-1\";\"Landscape\";\"buyer\";\"Client\";1500;\"EUR\";\"bank_transfer\";\"'  =2+3\";\"comment;line\";"));
        assert!(csv.contains("\"r2\";\"sale\";\"2026-11-12\";\"art\";\"ART-1\";\"Landscape\";\"buyer\";\"Client\";2001;\"EUR\";\"cash\";\"cash-02\";"));
        assert_eq!(csv.matches("\r\n").count(), 3);
        let sales = render_csv(&conn, "sales").unwrap();
        assert!(sales.contains(";10001;"));
        assert!(!sales.starts_with("\u{feff}receipt_id;"));
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sale_receipts", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn receipts_csv_links_optional_local_evidence_metadata_without_exporting_file_bytes() {
        use sha2::{Digest, Sha256};

        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let proof = b"%PDF-1.7\nPayment evidence";
        let relative = "media/receipt-proof.pdf";
        fs::write(db.directory.join(relative), proof).unwrap();
        let sha = format!("{:x}", Sha256::digest(proof));
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A-1','Opera','t','t');
            INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c','Collector','t','t');
            INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',5000,'2026-11-01','t');
            INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                VALUES('r1','s','2026-11-05',2000,'cash','t'),
                      ('r2','s','2026-11-06',1000,'card','t');
        "#).unwrap();
        conn.execute(
            "INSERT INTO sale_receipt_attachments(
                receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at
             ) VALUES('r1',?1,'=proof.pdf','application/pdf',?2,?3,'t')",
            rusqlite::params![relative, proof.len() as i64, sha],
        ).unwrap();

        let csv = render_csv(&conn, "receipts").unwrap();
        assert!(csv.starts_with(
            "\u{feff}receipt_id;sale_id;received_at;artwork_id;artwork_code;artwork_title;collector_id;collector_name;amount_cents;currency;method;reference;notes;created_at;attachment_path;attachment_original_name;attachment_mime;attachment_size_bytes;attachment_sha256;attachment_created_at\r\n"
        ));
        assert!(csv.contains(&format!(
            ";\"{relative}\";\"'=proof.pdf\";\"application/pdf\";{};\"{sha}\";\"t\"\r\n",
            proof.len()
        )), "{csv}");
        let without_evidence = csv.lines().find(|line| line.starts_with("\"r2\";")).unwrap();
        assert!(without_evidence.ends_with(";\"\";\"\";\"\";\"\";\"\";\"\""), "{without_evidence}");
        assert_eq!(csv.matches("\r\n").count(), 3);
        assert!(!csv.contains("Payment evidence")); // Bytes stay in the verified backup, not the CSV.
    }

    #[test]
    fn exports_exact_cents_historic_shares_and_escapes_spreadsheet_content() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
            VALUES('a','A-1','Test \"quadro\"; unico','t','t')", []).unwrap();
        conn.execute("INSERT INTO collectors(id,name,created_at,updated_at)
            VALUES('c','=2+2','t','t')", []).unwrap();
        conn.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,notes,created_at)
            VALUES('s','a','c',15001,'2026-11-01','  =HYPERLINK(\"https://example.test\")','t')", []).unwrap();
        conn.execute("INSERT INTO sale_finances(sale_id,tax_cents,shipping_cents,personal_bp,activity_bp,savings_bp)
            VALUES('s',200,301,5000,3000,2000)", []).unwrap();
        conn.execute_batch("INSERT INTO sale_allocations(sale_id,part_id,share_bp) VALUES('s','personal',5000),('s','activity',3000),('s','savings',2000);").unwrap();
        conn.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,notes,created_at,updated_at)
            VALUES('e','2026-11-02','materiali','+SUM(A1:A9)',1250,'activity','Prima riga
seconda riga','t','t')", []).unwrap();
        let sales = render_csv(&conn, "sales").unwrap();
        assert!(sales.starts_with("\u{feff}sale_id;"));
        assert!(sales.contains("\"Test \"\"quadro\"\"; unico\""));
        assert!(sales.contains("\"'=2+2\""));
        assert!(sales.contains("\"'  =HYPERLINK"));
        assert!(sales.contains(";15001;200;301;14500;5000;3000;2000;"));
        let expenses = render_csv(&conn, "expenses").unwrap();
        assert!(expenses.contains("\"'+SUM(A1:A9)\""));
        assert!(expenses.contains("\"Prima riga\nseconda riga\""));
        assert!(render_csv(&conn, "unknown").is_err());
    }



    #[test]
    fn balance_and_receipts_csv_reject_overpayment_or_negative_receipts_from_imports() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(r#"
                INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES('a','A-1','Opera','t','t');
                INSERT INTO collectors(id,name,created_at,updated_at)
                    VALUES('c','Cliente','t','t');
                INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                    VALUES('s','a','c',5000,'2026-11-01','t');
                INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                    VALUES('r1','s','2026-11-01',3000,'cash','t');
                -- Simulate an imported archive whose normal limit triggers were removed.
                DROP TRIGGER sale_receipts_limit_insert;
                DROP TRIGGER sale_receipts_limit_update;
                INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                    VALUES('r2','s','2026-11-02',3000,'cash','t');
            "#).unwrap();
            for kind in ["balances", "receipts"] {
                let failure = render_csv(&conn, kind).unwrap_err();
                assert!(failure.contains("incassi incoerenti"), "{kind}: {failure}");
            }
        }
        let destination = dir.path().join("inconsistent-balances.csv");
        assert!(export_file(&db, "balances", &destination).is_err());
        assert!(!destination.exists(), "a misleading report must not be published");

        let conn = db.conn.lock().unwrap();
        conn.execute("DELETE FROM sale_receipts WHERE id='r2'", []).unwrap();
        // An invalid negative receipt can also be masked by an otherwise
        // plausible positive total; do not allow it into a financial register.
        conn.execute_batch(r#"
            PRAGMA ignore_check_constraints=ON;
            UPDATE sale_receipts SET amount_cents=-1 WHERE id='r1';
            PRAGMA ignore_check_constraints=OFF;
        "#).unwrap();
        for kind in ["balances", "receipts"] {
            assert!(render_csv(&conn, kind).unwrap_err().contains("incassi incoerenti"));
        }
        conn.execute("UPDATE sale_receipts SET amount_cents=3000 WHERE id='r1'", []).unwrap();
        for kind in ["balances", "receipts"] {
            assert!(render_csv(&conn, kind).is_ok(), "{kind} should recover after repair");
        }
    }

    #[test]
    fn balances_csv_preserves_partial_full_and_unpaid_sales_without_inventing_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES
                ('a1','ART-1','One','t','t'),
                ('a2','ART-2','Two','t','t'),
                ('a3','ART-3','Three','t','t');
            INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c1','=Collector','t','t');
            INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES
                ('s1','a1','c1',10001,'2026-11-01','t'),
                ('s2','a2','c1',8000,'2026-11-02','t'),
                ('s3','a3','c1',5000,'2026-11-03','t');
            INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at) VALUES
                ('r1','s1','2026-11-10',1500,'cash','t'),
                ('r2','s1','2026-11-11',2001,'card','t'),
                ('r3','s2','2026-11-12',8000,'bank_transfer','t');
        "#).unwrap();
        let csv = render_csv(&conn, "balances").unwrap();
        assert!(csv.starts_with("\u{feff}sale_id;sold_at;artwork_id;"));
        assert!(csv.contains("\"s1\";\"2026-11-01\";\"a1\";\"ART-1\";\"One\";\"c1\";\"'=Collector\";\"EUR\";10001;3501;6500;\"t\""));
        assert!(csv.contains("\"s2\";\"2026-11-02\";\"a2\";\"ART-2\";\"Two\";\"c1\";\"'=Collector\";\"EUR\";8000;8000;0;\"t\""));
        assert!(csv.contains("\"s3\";\"2026-11-03\";\"a3\";\"ART-3\";\"Three\";\"c1\";\"'=Collector\";\"EUR\";5000;0;5000;\"t\""));
        assert_eq!(csv.matches("\r\n").count(), 4);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM sale_receipts", [], |r| r.get::<_, i64>(0)).unwrap(), 3);
    }

    #[test]
    fn gallery_csv_preserves_all_photo_metadata_and_order_without_exporting_image_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A-1','Opera','t','t');
            INSERT INTO artwork_images(id,artwork_id,image_path,caption,created_at,sort_order)
                VALUES('second','a','media/second.jpg','Seconda','t',2),
                      ('first','a','media/first.png','=SUM(1;2)','t',1);
        "#).unwrap();
        let csv = render_csv(&conn, "gallery").unwrap();
        assert!(csv.starts_with(
            "\u{feff}image_id;artwork_id;artwork_code;artwork_title;image_path;caption;sort_order;created_at;kind\r\n"
        ));
        let first = "\"first\";\"a\";\"A-1\";\"Opera\";\"media/first.png\";\"'=SUM(1;2)\";1;\"t\";\"OTHER\"";
        let second = "\"second\";\"a\";\"A-1\";\"Opera\";\"media/second.jpg\";\"Seconda\";2;\"t\";\"OTHER\"";
        assert!(csv.contains(first), "{csv}");
        assert!(csv.contains(second), "{csv}");
        assert!(csv.find(first) < csv.find(second));
        assert_eq!(csv.matches("\r\n").count(), 3);
        assert_eq!(conn.query_row(
            "SELECT COUNT(*) FROM artwork_images", [], |r| r.get::<_, i64>(0)
        ).unwrap(), 2);
    }

    #[test]
    fn collector_photos_csv_preserves_every_profile_and_extra_photo_with_owner_links() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c1','=Ada','t','t'),('c2','Bea','t','t');
            INSERT INTO media_items(id,original_name,current_revision_id,created_at,updated_at)
                VALUES('media-1','profile.jpg','revision-1','t','t');
            INSERT INTO collector_images(id,collector_id,kind,image_path,media_id,created_at)
                VALUES('extra','c1','PHOTO','media/detail.jpg',NULL,'t2'),
                      ('profile','c1','PROFILE','media/profile.jpg','media-1','t1'),
                      ('other','c2','PHOTO','media/other.png',NULL,'t3');
        "#).unwrap();

        let csv = render_csv(&conn, "collector_photos").unwrap();
        assert!(csv.starts_with(
            "\u{feff}photo_id;collector_id;collector_name;kind;image_path;media_id;created_at\r\n"
        ));
        let profile = "\"profile\";\"c1\";\"'=Ada\";\"PROFILE\";\"media/profile.jpg\";\"media-1\";\"t1\"";
        let extra = "\"extra\";\"c1\";\"'=Ada\";\"PHOTO\";\"media/detail.jpg\";\"\";\"t2\"";
        let other = "\"other\";\"c2\";\"Bea\";\"PHOTO\";\"media/other.png\";\"\";\"t3\"";
        assert!(csv.contains(profile), "{csv}");
        assert!(csv.contains(extra), "{csv}");
        assert!(csv.contains(other), "{csv}");
        assert!(csv.find(profile) < csv.find(extra), "profile photo must be first");
        assert!(csv.find(extra) < csv.find(other), "photos must keep their collector link");
        assert_eq!(csv.matches("\r\n").count(), 4);
        assert_eq!(conn.query_row(
            "SELECT COUNT(*) FROM collector_images", [], |r| r.get::<_, i64>(0)
        ).unwrap(), 3);
    }

    #[test]
    fn raw_files_csv_preserves_original_names_hashes_and_artwork_links_without_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute_batch(r#"
            INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a1','A-1','Original','t','t');
        "#).unwrap();
        let sha = "a".repeat(64);
        conn.execute(
            "INSERT INTO artwork_raw_files(artwork_id,media_path,original_name,size_bytes,sha256,created_at)
                VALUES('a1','media/source.dng','+suspicious.dng',12345,?1,'t')",
            rusqlite::params![sha],
        ).unwrap();

        let csv = render_csv(&conn, "raw_files").unwrap();
        assert!(csv.starts_with(
            "\u{feff}artwork_id;artwork_code;artwork_title;media_path;original_name;size_bytes;sha256;created_at\r\n"
        ));
        assert!(csv.contains(&format!(
            "\"a1\";\"A-1\";\"Original\";\"media/source.dng\";\"'+suspicious.dng\";12345;\"{sha}\";\"t\""
        )), "{csv}");
        assert_eq!(csv.matches("\r\n").count(), 2);
        assert!(!csv.contains("RAW FILE BYTES"));
        assert_eq!(conn.query_row(
            "SELECT COUNT(*) FROM artwork_raw_files", [], |r| r.get::<_, i64>(0)
        ).unwrap(), 1);
    }

    #[test]
    fn exports_archive_registers_with_stable_ids_and_certificate_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute("INSERT INTO artworks(id,code,title,width_cm,height_cm,status,image_path,created_at,updated_at)
            VALUES('art','=OP-1','Titolo attuale',12.5,9.25,'available','media/art.jpg','t','t')", []).unwrap();
        conn.execute("INSERT INTO collectors(id,name,email,created_at,updated_at)
            VALUES('person','  @formula','person@example.test','t','t')", []).unwrap();
        conn.execute(r#"INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
            VALUES('cert','art','RS-0001','2026-11-21','+ARTIST','{"title":"Titolo originario"}','t')"#, []).unwrap();

        let artworks = render_csv(&conn, "artworks").unwrap();
        assert!(artworks.starts_with("\u{feff}artwork_id;code;title;"));
        assert!(artworks.contains("\"art\";\"'=OP-1\";\"Titolo attuale\""));
        assert!(artworks.contains(";12.5;9.25;"));
        assert!(artworks.contains("\"media/art.jpg\""));

        let collectors = render_csv(&conn, "collectors").unwrap();
        assert!(collectors.starts_with("\u{feff}collector_id;name;email;"));
        assert!(collectors.contains("\"person\";\"'  @formula\";\"person@example.test\""));

        let certificates = render_csv(&conn, "certificates").unwrap();
        assert!(certificates.starts_with("\u{feff}certificate_id;artwork_id;artwork_code;current_artwork_title;"));
        assert!(certificates.contains("\"cert\";\"art\";\"'=OP-1\";\"Titolo attuale\";\"RS-0001\""));
        assert!(certificates.contains("\"'+ARTIST\""));
        assert!(certificates.lines().next().unwrap().ends_with(";language;issued_place;declarant_name"));
        let artworks = render_csv(&conn, "artworks").unwrap();
        assert!(artworks.lines().next().unwrap().ends_with(";artist_coefficient_override;fixed_price_cents"), "{artworks}");
        assert!(certificates.contains("\"{\"\"title\"\":\"\"Titolo originario\"\"}\""));
    }

    #[test]
    fn archive_register_exports_are_non_destructive_and_reject_replacing_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        for kind in ["artworks", "collectors", "certificates", "receipts"] {
            let csv = dir.path().join(format!("{kind}.csv"));
            export_file(&db, kind, &csv).unwrap();
            assert!(fs::read_to_string(&csv).unwrap().contains("created_at"));
            assert!(export_file(&db, kind, &csv).is_err());
        }
        assert!(export_file(&db, "artworks", &db.directory.join("artworks.csv")).is_err());
    }

    #[test]
    fn export_is_explicit_and_never_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        let csv = dir.path().join("ledger.csv");
        let saved = export_file(&db, "expenses", &csv).unwrap();
        assert!(Path::new(&saved).is_file());
        assert!(fs::read_to_string(&csv).unwrap().contains("expense_id;"));
        assert!(export_file(&db, "expenses", &csv).is_err());
        assert!(export_file(&db, "expenses", &db.directory.join("data.csv")).is_err());
        assert!(export_file(&db, "expenses", &dir.path().join("wrong.txt")).is_err());
        assert!(!dir.path().join("wrong.txt").exists());
    }
}
