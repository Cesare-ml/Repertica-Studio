//! What depends on an artwork or a collector, and removing all of it together when the user agrees.
//!
//! An artwork carries its certificates and its sale (with the receipts and attached files of that sale);
//! a collector carries the sales made to them. The interface first asks for the impact, shows it, and only
//! then asks for the deletion with `cascade` set: without it nothing that has dependents is ever removed.
use crate::db::Database;
use rusqlite::{params, Connection, Transaction};
use serde::Serialize;

fn err(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SaleImpact {
    pub artwork_title: String,
    pub collector_name: String,
    pub amount_cents: i64,
    pub sold_at: String,
}

/// Everything that would be deleted together with the record itself.
#[derive(Debug, Default, Serialize)]
pub struct DeletionImpact {
    /// Serial numbers of the certificates of the artwork.
    pub certificates: Vec<String>,
    pub sales: Vec<SaleImpact>,
    pub receipts: i64,
    /// Documents attached to the receipts (their files are removed too).
    pub attachments: i64,
}

impl DeletionImpact {
    pub fn is_empty(&self) -> bool { self.certificates.is_empty() && self.sales.is_empty() }
}

/// `column` is `artwork_id` or `collector_id` (never user input).
fn sales_of(conn: &Connection, column: &str, id: &str) -> Result<(Vec<SaleImpact>, i64, i64), String> {
    let sales = {
        let mut statement = conn.prepare(&format!(
            "SELECT a.title,c.name,s.amount_cents,s.sold_at FROM sales s
             JOIN artworks a ON a.id=s.artwork_id JOIN collectors c ON c.id=s.collector_id
             WHERE s.{column}=?1 ORDER BY s.sold_at,s.created_at")).map_err(err)?;
        let rows = statement.query_map(params![id], |r| Ok(SaleImpact {
            artwork_title: r.get(0)?, collector_name: r.get(1)?, amount_cents: r.get(2)?, sold_at: r.get(3)? }))
            .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
        rows
    };
    let receipts: i64 = conn.query_row(&format!(
        "SELECT count(*) FROM sale_receipts r JOIN sales s ON s.id=r.sale_id WHERE s.{column}=?1"),
        params![id], |r| r.get(0)).map_err(err)?;
    let attachments: i64 = conn.query_row(&format!(
        "SELECT count(*) FROM sale_receipt_attachments t JOIN sale_receipts r ON r.id=t.receipt_id
         JOIN sales s ON s.id=r.sale_id WHERE s.{column}=?1"),
        params![id], |r| r.get(0)).map_err(err)?;
    Ok((sales, receipts, attachments))
}

pub(crate) fn artwork_impact(conn: &Connection, id: &str) -> Result<DeletionImpact, String> {
    let certificates = {
        let mut statement = conn.prepare("SELECT serial FROM certificates WHERE artwork_id=?1 ORDER BY created_at,serial").map_err(err)?;
        let rows = statement.query_map(params![id], |r| r.get::<_, String>(0)).map_err(err)?
            .collect::<Result<Vec<_>, _>>().map_err(err)?;
        rows
    };
    let (sales, receipts, attachments) = sales_of(conn, "artwork_id", id)?;
    Ok(DeletionImpact { certificates, sales, receipts, attachments })
}

pub(crate) fn collector_impact(conn: &Connection, id: &str) -> Result<DeletionImpact, String> {
    let (sales, receipts, attachments) = sales_of(conn, "collector_id", id)?;
    Ok(DeletionImpact { certificates: vec![], sales, receipts, attachments })
}

/// Delete the sales of an artwork or collector with their receipts and attachment records; the artwork of each
/// sale becomes available again. Returns the files of the attachments, to be removed after the commit.
pub(crate) fn remove_sales(tx: &Transaction, column: &str, id: &str) -> Result<Vec<String>, String> {
    let sales = {
        let mut statement = tx.prepare(&format!("SELECT id,artwork_id FROM sales WHERE {column}=?1")).map_err(err)?;
        let rows = statement.query_map(params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err)?;
        rows
    };
    let mut files = Vec::new();
    for (sale, artwork) in sales {
        {
            let mut statement = tx.prepare("SELECT t.media_path FROM sale_receipt_attachments t
                JOIN sale_receipts r ON r.id=t.receipt_id WHERE r.sale_id=?1").map_err(err)?;
            let paths = statement.query_map(params![sale], |r| r.get::<_, String>(0)).map_err(err)?
                .collect::<Result<Vec<_>, _>>().map_err(err)?;
            files.extend(paths);
        }
        tx.execute("DELETE FROM sale_receipt_attachments WHERE receipt_id IN (SELECT id FROM sale_receipts WHERE sale_id=?1)", params![sale]).map_err(err)?;
        tx.execute("DELETE FROM sale_receipts WHERE sale_id=?1", params![sale]).map_err(err)?;
        tx.execute("DELETE FROM sales WHERE id=?1", params![sale]).map_err(err)?;
        tx.execute("UPDATE artworks SET status='available',updated_at=?2 WHERE id=?1 AND status='sold'",
            params![artwork, chrono::Utc::now().to_rfc3339()]).map_err(err)?;
    }
    Ok(files)
}

/// The photograph frozen in a certificate's snapshot (None when unreadable or absent).
fn snapshot_image(snapshot: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(snapshot).ok()?
        .get("image_path")?.as_str().map(str::to_string)
}

/// Delete the certificates of an artwork; returns the photographs their snapshots kept alive.
pub(crate) fn remove_certificates(tx: &Transaction, artwork_id: &str) -> Result<Vec<String>, String> {
    let snapshots = {
        let mut statement = tx.prepare("SELECT snapshot_json FROM certificates WHERE artwork_id=?1").map_err(err)?;
        let rows = statement.query_map(params![artwork_id], |r| r.get::<_, String>(0)).map_err(err)?
            .collect::<Result<Vec<_>, _>>().map_err(err)?;
        rows
    };
    tx.execute("DELETE FROM certificates WHERE artwork_id=?1", params![artwork_id]).map_err(err)?;
    Ok(snapshots.iter().filter_map(|s| snapshot_image(s)).collect())
}

/// Remove files left without any reference (a file still used elsewhere is kept).
pub(crate) fn clean_files(conn: &Connection, db: &Database, paths: &[String]) {
    for path in paths { crate::media::remove_file_unless_catalogued(conn, &db.directory, path); }
}

fn delete_certificate_core(db: &Database, id: &str) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(err)?;
    let snapshot: Option<String> = {
        use rusqlite::OptionalExtension;
        conn.query_row("SELECT snapshot_json FROM certificates WHERE id=?1", params![id], |r| r.get(0)).optional().map_err(err)?
    };
    let snapshot = snapshot.ok_or_else(|| "Certificato non trovato".to_string())?;
    let tx = conn.transaction().map_err(err)?;
    tx.execute("DELETE FROM certificates WHERE id=?1", params![id]).map_err(err)?;
    tx.commit().map_err(err)?;
    // The cover shown on the certificate may have been replaced since; free it if nothing else uses it.
    if let Some(path) = snapshot_image(&snapshot) { clean_files(&conn, db, &[path]); }
    Ok(())
}

#[tauri::command]
pub fn artwork_deletion_impact(db: tauri::State<'_, Database>, id: String) -> Result<DeletionImpact, String> {
    let conn = db.conn.lock().map_err(err)?;
    artwork_impact(&conn, &id)
}
#[tauri::command]
pub fn collector_deletion_impact(db: tauri::State<'_, Database>, id: String) -> Result<DeletionImpact, String> {
    let conn = db.conn.lock().map_err(err)?;
    collector_impact(&conn, &id)
}
/// A certificate can be deleted at any time: the artwork and its sale are not touched.
#[tauri::command]
pub fn delete_certificate(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    delete_certificate_core(&db, &id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// artwork `a` sold to collector `c` (sale `s`, one receipt with an attached file), one certificate.
    fn fixture() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().into()).unwrap();
        std::fs::write(dir.path().join("media").join("receipt.pdf"), b"%PDF").unwrap();
        {
            let c = db.conn.lock().unwrap();
            c.execute_batch("
                INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','OP-1','Mare','sold','t','t');
                INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('b','OP-2','Monte','available','t','t');
                INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Anna Rossi','t','t');
                INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',50000,'2026-11-01','t');
                INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at) VALUES('r','s','2026-11-02',20000,'cash','t');
                INSERT INTO sale_receipt_attachments(receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at)
                    VALUES('r','media/receipt.pdf','ricevuta.pdf','application/pdf',4,'0000000000000000000000000000000000000000000000000000000000000000','t');
                INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at)
                    VALUES('k','a','RS-2026-000001','2026-11-03','Artista','{}','t');").unwrap();
        }
        (dir, db)
    }
    fn count(db: &Database, table: &str) -> i64 {
        db.conn.lock().unwrap().query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn impact_lists_certificates_sale_receipts_and_attachments() {
        let (_dir, db) = fixture();
        let conn = db.conn.lock().unwrap();
        let artwork = artwork_impact(&conn, "a").unwrap();
        assert_eq!(artwork.certificates, vec!["RS-2026-000001".to_string()]);
        assert_eq!(artwork.sales.len(), 1);
        assert_eq!((artwork.sales[0].collector_name.as_str(), artwork.sales[0].amount_cents), ("Anna Rossi", 50000));
        assert_eq!((artwork.receipts, artwork.attachments), (1, 1));
        assert!(!artwork.is_empty());
        let collector = collector_impact(&conn, "c").unwrap();
        assert_eq!(collector.sales.len(), 1);
        assert!(collector.certificates.is_empty());
        assert!(artwork_impact(&conn, "b").unwrap().is_empty());
    }

    #[test]
    fn a_certificate_can_be_deleted_but_not_edited() {
        let (_dir, db) = fixture();
        assert!(db.conn.lock().unwrap().execute("UPDATE certificates SET serial='x' WHERE id='k'", []).is_err());
        delete_certificate_core(&db, "k").unwrap();
        assert_eq!(count(&db, "certificates"), 0);
        // The artwork and its sale stay.
        assert_eq!((count(&db, "artworks"), count(&db, "sales")), (2, 1));
        assert!(delete_certificate_core(&db, "k").is_err());
    }

    #[test]
    fn deleting_an_artwork_needs_consent_and_then_removes_its_dependents() {
        let (dir, db) = fixture();
        assert!(crate::commands::delete_artwork_core(&db, "a", false).is_err());
        assert_eq!((count(&db, "artworks"), count(&db, "sales"), count(&db, "certificates")), (2, 1, 1));
        crate::commands::delete_artwork_core(&db, "a", true).unwrap();
        assert_eq!((count(&db, "artworks"), count(&db, "sales"), count(&db, "certificates")), (1, 0, 0));
        assert_eq!((count(&db, "sale_receipts"), count(&db, "sale_receipt_attachments")), (0, 0));
        assert!(!dir.path().join("media").join("receipt.pdf").exists(), "the attached document goes with it");
        // The collector is untouched.
        assert_eq!(count(&db, "collectors"), 1);
        // An artwork without dependents needs no consent.
        crate::commands::delete_artwork_core(&db, "b", false).unwrap();
    }

    #[test]
    fn deleting_a_collector_removes_their_sales_and_frees_the_artworks() {
        let (dir, db) = fixture();
        assert!(crate::collectors::delete_collector_core(&db, "c", false).unwrap_err().contains("vendita"));
        assert_eq!(count(&db, "sales"), 1);
        crate::collectors::delete_collector_core(&db, "c", true).unwrap();
        assert_eq!((count(&db, "collectors"), count(&db, "sales"), count(&db, "sale_receipts")), (0, 0, 0));
        assert!(!dir.path().join("media").join("receipt.pdf").exists());
        // The artwork and its certificate stay; the artwork can be sold again.
        assert_eq!((count(&db, "artworks"), count(&db, "certificates")), (2, 1));
        let status: String = db.conn.lock().unwrap().query_row("SELECT status FROM artworks WHERE id='a'", [], |r| r.get(0)).unwrap();
        assert_eq!(status, "available");
    }
}
