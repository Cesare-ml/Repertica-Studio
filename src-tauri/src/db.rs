use rusqlite::Connection;
use std::{error::Error, fs, path::PathBuf, sync::Mutex};

pub struct Database {
    pub conn: Mutex<Connection>,
    pub directory: PathBuf,
}

pub fn open(directory: PathBuf) -> Result<Database, Box<dyn Error>> {
    fs::create_dir_all(&directory)?;
    fs::create_dir_all(directory.join("media"))?;
    let conn = Connection::open(directory.join("repertica.sqlite"))?;
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > 3 {
        return Err(format!("Archivio creato da una versione più recente (schema {version})").into());
    }
    if version == 0 {
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE artworks (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE COLLATE NOCASE,
                title TEXT NOT NULL,
                year TEXT,
                technique TEXT,
                support TEXT,
                width_cm REAL CHECK(width_cm IS NULL OR width_cm > 0),
                height_cm REAL CHECK(height_cm IS NULL OR height_cm > 0),
                depth_cm REAL CHECK(depth_cm IS NULL OR depth_cm > 0),
                description TEXT,
                location TEXT,
                status TEXT NOT NULL DEFAULT 'available'
                    CHECK(status IN ('available','reserved','sold','archived')),
                image_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE collectors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                address TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE sales (
                id TEXT PRIMARY KEY,
                artwork_id TEXT NOT NULL UNIQUE REFERENCES artworks(id) ON DELETE RESTRICT,
                collector_id TEXT NOT NULL REFERENCES collectors(id) ON DELETE RESTRICT,
                amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
                currency TEXT NOT NULL DEFAULT 'EUR' CHECK(currency = 'EUR'),
                sold_at TEXT NOT NULL,
                notes TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE certificates (
                id TEXT PRIMARY KEY,
                artwork_id TEXT NOT NULL REFERENCES artworks(id) ON DELETE RESTRICT,
                serial TEXT NOT NULL UNIQUE,
                issued_at TEXT NOT NULL,
                artist_name TEXT NOT NULL,
                collector_name TEXT,
                notes TEXT,
                snapshot_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE certificate_counter (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                last_number INTEGER NOT NULL CHECK(last_number >= 0)
            );
            INSERT INTO certificate_counter(singleton,last_number) VALUES(1,0);
            CREATE INDEX idx_artworks_title ON artworks(title);
            CREATE INDEX idx_artworks_status ON artworks(status);
            CREATE INDEX idx_sales_date ON sales(sold_at);
            CREATE INDEX idx_certificates_artwork ON certificates(artwork_id);
            PRAGMA user_version=1;
            COMMIT;"
        )?;
    }
    if version < 2 {
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE studio_settings (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                artist_coefficient REAL NOT NULL DEFAULT 1.0 CHECK(artist_coefficient > 0 AND artist_coefficient <= 100000),
                base_price_cm2_cents INTEGER NOT NULL DEFAULT 16 CHECK(base_price_cm2_cents >= 0 AND base_price_cm2_cents <= 100000),
                personal_bp INTEGER NOT NULL DEFAULT 3334 CHECK(personal_bp BETWEEN 0 AND 10000),
                activity_bp INTEGER NOT NULL DEFAULT 3333 CHECK(activity_bp BETWEEN 0 AND 10000),
                savings_bp INTEGER NOT NULL DEFAULT 3333 CHECK(savings_bp BETWEEN 0 AND 10000),
                treasury_start_date TEXT,
                default_tax_bp INTEGER NOT NULL DEFAULT 0 CHECK(default_tax_bp BETWEEN 0 AND 10000),
                ui_language TEXT CHECK(ui_language IS NULL OR ui_language IN ('it','en','es','fr','de')),
                CHECK(personal_bp + activity_bp + savings_bp = 10000)
            );
            INSERT INTO studio_settings(singleton) VALUES(1);
            CREATE TABLE sale_finances (
                sale_id TEXT PRIMARY KEY REFERENCES sales(id) ON DELETE CASCADE,
                tax_cents INTEGER NOT NULL DEFAULT 0 CHECK(tax_cents >= 0),
                shipping_cents INTEGER NOT NULL DEFAULT 0 CHECK(shipping_cents >= 0),
                personal_bp INTEGER NOT NULL,
                activity_bp INTEGER NOT NULL,
                savings_bp INTEGER NOT NULL,
                CHECK(personal_bp + activity_bp + savings_bp = 10000)
            );
            INSERT INTO sale_finances(sale_id,personal_bp,activity_bp,savings_bp)
                SELECT id,3334,3333,3333 FROM sales;
            CREATE TABLE expenses (
                id TEXT PRIMARY KEY,
                expense_date TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
                fund TEXT NOT NULL CHECK(fund IN ('activity','savings')),
                supplier TEXT,
                purchase_url TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX idx_expenses_date ON expenses(expense_date);
            CREATE INDEX idx_expenses_category ON expenses(category);
            PRAGMA user_version=2;
            COMMIT;"
        )?;
    }
    if version < 3 {
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS artwork_images (
                id TEXT PRIMARY KEY,
                artwork_id TEXT NOT NULL REFERENCES artworks(id) ON DELETE CASCADE,
                image_path TEXT NOT NULL UNIQUE,
                caption TEXT CHECK(caption IS NULL OR length(caption) <= 300),
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_artwork_images_artwork
                ON artwork_images(artwork_id, created_at, id);
            PRAGMA user_version=3;
            COMMIT;"
        )?;
    }
    // Certificates are append-only even when accessed through a raw SQLite client.
    // Idempotent creation also upgrades early version-1 development databases.
    conn.execute_batch("CREATE TRIGGER IF NOT EXISTS certificates_no_update
        BEFORE UPDATE ON certificates BEGIN SELECT RAISE(ABORT, 'certificate_immutable'); END;
        CREATE TRIGGER IF NOT EXISTS certificates_no_delete
        BEFORE DELETE ON certificates BEGIN SELECT RAISE(ABORT, 'certificate_immutable'); END;")?;
    Ok(Database { conn: Mutex::new(conn), directory })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_schema_and_reopens_without_resetting_data() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            let v: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0)).unwrap();
            assert_eq!(v, 3);
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('x','Test','a','a')",[]).unwrap();
        }
        drop(db);
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM collectors",[],|r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
    #[test]
    fn foreign_keys_restrict_orphaned_sales() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        assert!(c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',100,'2026-11-21','now')",[]).is_err());
    }
    #[test]
    fn certificates_cannot_be_edited_or_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','OP-01','Test','available','now','now')",[]).unwrap();
        c.execute("INSERT INTO certificates(id,artwork_id,serial,issued_at,artist_name,snapshot_json,created_at) VALUES('cert','a','RS-01','2026-11-21','Artista','{}','now')",[]).unwrap();
        assert!(c.execute("UPDATE certificates SET serial='altered' WHERE id='cert'",[]).is_err());
        assert!(c.execute("DELETE FROM certificates WHERE id='cert'",[]).is_err());
        assert!(c.execute("DELETE FROM artworks WHERE id='a'",[]).is_err());
    }
    #[test]
    fn sale_is_unique_and_prevents_orphaned_contacts() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        c.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES('a','OP-01','Test','available','now','now')",[]).unwrap();
        c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Persona','now','now')",[]).unwrap();
        c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',15000,'2026-11-21','now')",[]).unwrap();
        assert!(c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s2','a','c',15000,'2026-11-21','now')",[]).is_err());
        assert!(c.execute("DELETE FROM collectors WHERE id='c'",[]).is_err());
    }
    #[test]
    fn finance_migration_preserves_old_sales() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES('a','X','Art','t','t')",[]).unwrap();
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('c','Collector','t','t')",[]).unwrap();
            c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at) VALUES('s','a','c',9000,'2026-02-11','t')",[]).unwrap();
            c.execute("DELETE FROM studio_settings",[]).unwrap();
            c.execute("DROP TABLE sale_finances",[]).unwrap();
            c.execute("DROP TABLE expenses",[]).unwrap();
            c.execute("DROP TABLE studio_settings",[]).unwrap();
            c.pragma_update(None,"user_version",1).unwrap();
        }
        drop(db);
        let migrated = open(dir.path().into()).unwrap();
        let c = migrated.conn.lock().unwrap();
        let v: i64 = c.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(v,3);
        let n: i64 = c.query_row("SELECT count(*) FROM sale_finances WHERE sale_id='s'",[],|r|r.get(0)).unwrap();
        assert_eq!(n,1);
    }
    #[test]
    fn financial_percentages_must_sum_to_one_hundred() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        assert!(c.execute("UPDATE studio_settings SET personal_bp=5000 WHERE singleton=1",[]).is_err());
        assert!(c.execute("INSERT INTO expenses(id,expense_date,category,description,amount_cents,fund,created_at,updated_at) VALUES('x','2026-10-01','materials','Brushes',100,'other','t','t')",[]).is_err());
    }
    #[test]
    fn gallery_migration_preserves_primary_image_and_enforces_relations() {
        let temp = tempfile::tempdir().unwrap();
        let db = open(temp.path().into()).unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
                VALUES('a','A','Original','media/primary.jpg','t','t')", []).unwrap();
            conn.execute("DROP TABLE artwork_images",[]).unwrap();
            conn.pragma_update(None,"user_version",2).unwrap();
        }
        drop(db);
        let upgraded = open(temp.path().into()).unwrap();
        let conn = upgraded.conn.lock().unwrap();
        let version: i64 = conn.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,3);
        let image: String = conn.query_row("SELECT image_path FROM artworks WHERE id='a'",[],|r|r.get(0)).unwrap();
        assert_eq!(image,"media/primary.jpg");
        assert!(conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
            VALUES('bad','nonexistent','media/extra.jpg','t')",[]).is_err());
        conn.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
            VALUES('img','a','media/extra.jpg','t')",[]).unwrap();
        conn.execute("DELETE FROM artworks WHERE id='a'",[]).unwrap();
        let count: i64=conn.query_row("SELECT count(*) FROM artwork_images",[],|r|r.get(0)).unwrap();
        assert_eq!(count,0);
    }
    #[test]
    fn newer_schema_is_not_downgraded() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        db.conn.lock().unwrap().pragma_update(None, "user_version", 4).unwrap();
        drop(db);
        assert!(open(dir.path().into()).is_err());
    }
}
