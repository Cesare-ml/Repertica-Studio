//! Offline CSV exports of artwork, collector, certificate, sale and expense registers.
//! CSV exports are for inspection/analysis, NOT a restore format or verified cash receipts.
//! All money is exported as integer cents to avoid rounding or locale ambiguity.
use crate::db::Database;
use rusqlite::{types::ValueRef, Connection};
use std::{fs::{self, OpenOptions}, io::Write, path::Path};
use tauri::State;

fn error(e: impl std::fmt::Display) -> String { e.to_string() }

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

fn render_csv(conn: &Connection, kind: &str) -> Result<String, String> {
    // Deliberately export all rows, not the on-screen year filter. Each record
    // carries its date; no historic record is silently omitted.
    let (header, query) = match kind {
        "artworks" => (
            "artwork_id;code;title;year;technique;support;width_cm;height_cm;depth_cm;description;location;status;image_path;created_at;updated_at",
            "SELECT id,code,title,year,technique,support,width_cm,height_cm,depth_cm,
                    description,location,status,image_path,created_at,updated_at
             FROM artworks ORDER BY code COLLATE NOCASE,id"
        ),
        "collectors" => (
            "collector_id;name;email;phone;address;notes;created_at;updated_at",
            "SELECT id,name,email,phone,address,notes,created_at,updated_at
             FROM collectors ORDER BY name COLLATE NOCASE,id"
        ),
        "certificates" => (
            "certificate_id;artwork_id;artwork_code;current_artwork_title;serial;issued_at;artist_name;collector_name;notes;snapshot_json;created_at",
            "SELECT c.id,c.artwork_id,a.code,a.title,c.serial,c.issued_at,c.artist_name,
                    c.collector_name,c.notes,c.snapshot_json,c.created_at
             FROM certificates c JOIN artworks a ON a.id=c.artwork_id
             ORDER BY c.issued_at,c.serial,c.id"
        ),
        "sales" => (
            "sale_id;sold_at;artwork_id;artwork_code;artwork_title;collector_id;collector_name;currency;gross_cents;tax_cents;shipping_cents;net_cents;personal_bp;activity_bp;savings_bp;notes;created_at",
            "SELECT s.id,s.sold_at,s.artwork_id,a.code,a.title,s.collector_id,c.name,s.currency,
                    s.amount_cents,COALESCE(f.tax_cents,0),COALESCE(f.shipping_cents,0),
                    s.amount_cents-COALESCE(f.tax_cents,0)-COALESCE(f.shipping_cents,0),
                    COALESCE(f.personal_bp,3334),COALESCE(f.activity_bp,3333),COALESCE(f.savings_bp,3333),
                    s.notes,s.created_at
             FROM sales s JOIN artworks a ON a.id=s.artwork_id
             JOIN collectors c ON c.id=s.collector_id
             LEFT JOIN sale_finances f ON f.sale_id=s.id
             ORDER BY s.sold_at,s.id"
        ),
        "expenses" => (
            "expense_id;expense_date;category;description;amount_cents;fund;supplier;purchase_url;notes;created_at;updated_at",
            "SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at
             FROM expenses ORDER BY expense_date,id"
        ),
        _ => return Err("Tipo di registro CSV non riconosciuto".into()),
    };
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
    Ok(target.to_string_lossy().into_owned())
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
        assert!(certificates.contains("\"{\"\"title\"\":\"\"Titolo originario\"\"}\""));
    }

    #[test]
    fn archive_register_exports_are_non_destructive_and_reject_replacing_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let db = db::open(dir.path().join("archive")).unwrap();
        for kind in ["artworks", "collectors", "certificates"] {
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
