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
    if version > 1 {
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
            assert_eq!(v, 1);
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
    fn newer_schema_is_not_downgraded() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        db.conn.lock().unwrap().pragma_update(None, "user_version", 2).unwrap();
        drop(db);
        assert!(open(dir.path().into()).is_err());
    }
}
