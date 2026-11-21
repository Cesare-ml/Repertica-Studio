use rusqlite::Connection;
use std::{error::Error, fs, path::PathBuf, sync::Mutex};

/// Version written by this build (`PRAGMA user_version`).
pub const SCHEMA_VERSION: i64 = 14;

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
    if version > SCHEMA_VERSION {
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
    if version < 4 {
        // No migration is allowed to infer that a historical sale was paid.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE sale_receipts (
                id TEXT PRIMARY KEY,
                sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
                received_at TEXT NOT NULL,
                amount_cents INTEGER NOT NULL CHECK(amount_cents BETWEEN 1 AND 100000000000),
                method TEXT NOT NULL CHECK(method IN ('bank_transfer','cash','card','other')),
                reference TEXT CHECK(reference IS NULL OR length(reference) <= 200),
                notes TEXT CHECK(notes IS NULL OR length(notes) <= 2000),
                created_at TEXT NOT NULL
            );
            CREATE INDEX idx_sale_receipts_sale ON sale_receipts(sale_id,received_at,id);
            CREATE TRIGGER sale_receipts_limit_insert BEFORE INSERT ON sale_receipts BEGIN
                SELECT CASE WHEN
                  (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts WHERE sale_id=NEW.sale_id)
                  + NEW.amount_cents > (SELECT amount_cents FROM sales WHERE id=NEW.sale_id)
                  THEN RAISE(ABORT,'receipt_exceeds_sale') END;
            END;
            CREATE TRIGGER sale_receipts_limit_update BEFORE UPDATE OF amount_cents,sale_id ON sale_receipts BEGIN
                SELECT CASE WHEN
                  (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts
                   WHERE sale_id=NEW.sale_id AND id<>OLD.id)
                  + NEW.amount_cents > (SELECT amount_cents FROM sales WHERE id=NEW.sale_id)
                  THEN RAISE(ABORT,'receipt_exceeds_sale') END;
            END;
            CREATE TRIGGER sales_no_shrink_below_receipts BEFORE UPDATE OF amount_cents ON sales
                WHEN NEW.amount_cents <
                (SELECT COALESCE(SUM(amount_cents),0) FROM sale_receipts WHERE sale_id=OLD.id)
                BEGIN SELECT RAISE(ABORT,'sale_below_receipts'); END;
            PRAGMA user_version=4;
            COMMIT;"
        )?;
    }
    if version < 5 {
        // Legacy gallery order was created_at/id; preserve it per artwork.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            ALTER TABLE artwork_images ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
            WITH ranked AS (
                SELECT id, ROW_NUMBER() OVER (
                    PARTITION BY artwork_id ORDER BY created_at, id
                ) AS position FROM artwork_images
            )
            UPDATE artwork_images SET sort_order=(
                SELECT position FROM ranked WHERE ranked.id=artwork_images.id
            );
            CREATE INDEX idx_artwork_images_position
                ON artwork_images(artwork_id,sort_order,id);
            PRAGMA user_version=5;
            COMMIT;"
        )?;
    }
    if version < 6 {
        // Attachments are separate local files linked to an explicit receipt.
        // Never infer an attachment from an existing sale or receipt.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS sale_receipt_attachments (
                receipt_id TEXT PRIMARY KEY REFERENCES sale_receipts(id) ON DELETE RESTRICT,
                media_path TEXT NOT NULL UNIQUE,
                original_name TEXT NOT NULL CHECK(length(original_name) BETWEEN 1 AND 240),
                mime_type TEXT NOT NULL CHECK(mime_type IN ('application/pdf','image/jpeg','image/png','image/webp')),
                size_bytes INTEGER NOT NULL CHECK(size_bytes BETWEEN 1 AND 12582912),
                sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
                created_at TEXT NOT NULL
            );
            PRAGMA user_version=6;
            COMMIT;"
        )?;
    }
    if version < 7 {
        // Automatic archive codes: one policy row and a forward-only counter per prefix and scope.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS archive_code_policy (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                prefix TEXT NOT NULL DEFAULT 'ART' CHECK(length(prefix) BETWEEN 1 AND 12 AND prefix NOT GLOB '*[^A-Z0-9]*'),
                scope TEXT NOT NULL DEFAULT 'GLOBAL' CHECK(scope IN ('GLOBAL','YEARLY')),
                padding INTEGER NOT NULL DEFAULT 4 CHECK(padding BETWEEN 2 AND 8),
                separator TEXT NOT NULL DEFAULT '-' CHECK(separator IN ('-','.','_'))
            );
            INSERT OR IGNORE INTO archive_code_policy(singleton) VALUES(1);
            CREATE TABLE IF NOT EXISTS archive_code_sequences (
                scope_key TEXT PRIMARY KEY,
                last_value INTEGER NOT NULL CHECK(last_value >= 0)
            );
            PRAGMA user_version=7;
            COMMIT;"
        )?;
    }
    if version < 8 {
        // Richer collector records and their photographs (profile photo plus further photos).
        let existing: Vec<String> = {
            let mut statement = conn.prepare("SELECT name FROM pragma_table_info('collectors')")?;
            let names = statement.query_map([], |r| r.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
            names
        };
        let mut script = String::from("BEGIN IMMEDIATE;");
        for column in ["first_name", "last_name", "birth_date", "birth_time", "birth_place",
                       "instagram", "facebook", "tiktok", "website", "linkedin"] {
            if !existing.iter().any(|name| name == column) {
                script.push_str(&format!("ALTER TABLE collectors ADD COLUMN {column} TEXT;"));
            }
        }
        script.push_str(
            "UPDATE collectors SET first_name=name WHERE first_name IS NULL;
            CREATE TABLE IF NOT EXISTS collector_images (
                id TEXT PRIMARY KEY,
                collector_id TEXT NOT NULL REFERENCES collectors(id) ON DELETE CASCADE,
                kind TEXT NOT NULL CHECK(kind IN ('PROFILE','PHOTO')),
                image_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_collector_profile_photo
                ON collector_images(collector_id) WHERE kind='PROFILE';
            CREATE INDEX IF NOT EXISTS idx_collector_images_owner
                ON collector_images(collector_id, created_at, id);
            PRAGMA user_version=8;
            COMMIT;");
        conn.execute_batch(&script)?;
    }
    if version < 9 {
        // Configurable split of sale proceeds. The three historical shares become the three
        // default parts and every existing sale keeps exactly the shares it was recorded with.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS allocation_settings (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1))
            );
            INSERT OR IGNORE INTO allocation_settings(singleton,enabled) VALUES(1,1);
            CREATE TABLE IF NOT EXISTS allocation_parts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 60),
                share_bp INTEGER NOT NULL CHECK(share_bp BETWEEN 0 AND 10000),
                pays_expenses INTEGER NOT NULL DEFAULT 0 CHECK(pays_expenses IN (0,1)),
                position INTEGER NOT NULL,
                active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1))
            );
            INSERT OR IGNORE INTO allocation_parts(id,name,share_bp,pays_expenses,position,active)
                SELECT 'personal','Personale',personal_bp,0,1,1 FROM studio_settings WHERE singleton=1;
            INSERT OR IGNORE INTO allocation_parts(id,name,share_bp,pays_expenses,position,active)
                SELECT 'activity','Attività',activity_bp,1,2,1 FROM studio_settings WHERE singleton=1;
            INSERT OR IGNORE INTO allocation_parts(id,name,share_bp,pays_expenses,position,active)
                SELECT 'savings','Risparmio',savings_bp,1,3,1 FROM studio_settings WHERE singleton=1;
            CREATE TABLE IF NOT EXISTS sale_allocations (
                sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
                part_id TEXT NOT NULL REFERENCES allocation_parts(id) ON DELETE RESTRICT,
                share_bp INTEGER NOT NULL CHECK(share_bp BETWEEN 0 AND 10000),
                PRIMARY KEY(sale_id,part_id)
            );
            INSERT OR IGNORE INTO sale_allocations(sale_id,part_id,share_bp)
                SELECT f.sale_id,'personal',f.personal_bp FROM sale_finances f
                WHERE NOT EXISTS (SELECT 1 FROM sale_allocations a WHERE a.sale_id=f.sale_id);
            INSERT OR IGNORE INTO sale_allocations(sale_id,part_id,share_bp)
                SELECT f.sale_id,'activity',f.activity_bp FROM sale_finances f
                WHERE NOT EXISTS (SELECT 1 FROM sale_allocations a WHERE a.sale_id=f.sale_id AND a.part_id='activity')
                  AND EXISTS (SELECT 1 FROM sale_allocations a WHERE a.sale_id=f.sale_id AND a.part_id='personal');
            INSERT OR IGNORE INTO sale_allocations(sale_id,part_id,share_bp)
                SELECT f.sale_id,'savings',f.savings_bp FROM sale_finances f
                WHERE NOT EXISTS (SELECT 1 FROM sale_allocations a WHERE a.sale_id=f.sale_id AND a.part_id='savings')
                  AND EXISTS (SELECT 1 FROM sale_allocations a WHERE a.sale_id=f.sale_id AND a.part_id='personal');
            CREATE TABLE IF NOT EXISTS expenses_v9 (
                id TEXT PRIMARY KEY,
                expense_date TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
                fund TEXT REFERENCES allocation_parts(id) ON DELETE RESTRICT,
                supplier TEXT,
                purchase_url TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT OR IGNORE INTO expenses_v9(id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at)
                SELECT id,expense_date,category,description,amount_cents,fund,supplier,purchase_url,notes,created_at,updated_at FROM expenses;
            DROP TABLE expenses;
            ALTER TABLE expenses_v9 RENAME TO expenses;
            CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses(expense_date);
            CREATE INDEX IF NOT EXISTS idx_expenses_category ON expenses(category);
            PRAGMA user_version=9;
            COMMIT;"
        )?;
    }
    if version < 10 {
        // Scheduled automatic backups: off by default, one configuration row and a log of the
        // folders this feature created (only those may ever be removed by the retention setting).
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS backup_schedule (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
                frequency TEXT NOT NULL DEFAULT 'DAILY' CHECK(frequency IN ('DAILY','WEEKLY','MONTHLY')),
                time_of_day TEXT NOT NULL DEFAULT '20:00',
                weekday INTEGER NOT NULL DEFAULT 0 CHECK(weekday BETWEEN 0 AND 6),
                day_of_month INTEGER NOT NULL DEFAULT 1 CHECK(day_of_month BETWEEN 1 AND 28),
                destination TEXT,
                keep_last INTEGER NOT NULL DEFAULT 0 CHECK(keep_last BETWEEN 0 AND 365),
                enabled_at TEXT,
                last_attempt_at TEXT,
                last_success_at TEXT,
                last_status TEXT CHECK(last_status IS NULL OR last_status IN ('OK','ERROR')),
                last_message TEXT,
                last_directory TEXT
            );
            INSERT OR IGNORE INTO backup_schedule(singleton) VALUES(1);
            CREATE TABLE IF NOT EXISTS auto_backup_log (
                directory TEXT PRIMARY KEY,
                created_at TEXT NOT NULL
            );
            PRAGMA user_version=10;
            COMMIT;"
        )?;
    }
    if version < 11 {
        // Installment (rateale) plans: scheduled payments of a sale. Receipts stay the only
        // record of money actually received.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS sale_installments (
                id TEXT PRIMARY KEY,
                sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
                position INTEGER NOT NULL CHECK(position >= 1),
                due_date TEXT NOT NULL,
                amount_cents INTEGER NOT NULL CHECK(amount_cents > 0 AND amount_cents <= 100000000000),
                notes TEXT CHECK(notes IS NULL OR length(notes) <= 500),
                UNIQUE(sale_id, position)
            );
            CREATE INDEX IF NOT EXISTS idx_sale_installments_due ON sale_installments(due_date);
            PRAGMA user_version=11;
            COMMIT;"
        )?;
    }
    if version < 12 {
        // One RAW camera file per artwork, stored in the private media folder.
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS artwork_raw_files (
                artwork_id TEXT PRIMARY KEY REFERENCES artworks(id) ON DELETE CASCADE,
                media_path TEXT NOT NULL UNIQUE,
                original_name TEXT NOT NULL CHECK(length(original_name) BETWEEN 1 AND 240),
                size_bytes INTEGER NOT NULL CHECK(size_bytes > 0),
                sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
                created_at TEXT NOT NULL
            );
            PRAGMA user_version=12;
            COMMIT;"
        )?;
    }
    if version < 13 {
        // Media center: catalogue of pictures with revisions, tags and collections, and links from
        // artworks / galleries / collector photos to the catalogue.
        let has_column = |table: &str, column: &str| -> Result<bool, rusqlite::Error> {
            conn.query_row(&format!("SELECT count(*) FROM pragma_table_info('{table}') WHERE name='{column}'"), [], |r| r.get::<_, i64>(0)).map(|n| n > 0)
        };
        let mut script = String::from("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS media_items (
                id TEXT PRIMARY KEY,
                title TEXT CHECK(title IS NULL OR length(title) <= 200),
                description TEXT CHECK(description IS NULL OR length(description) <= 5000),
                alt_text TEXT CHECK(alt_text IS NULL OR length(alt_text) <= 500),
                credits TEXT CHECK(credits IS NULL OR length(credits) <= 300),
                copyright_status TEXT NOT NULL DEFAULT 'UNKNOWN' CHECK(copyright_status IN ('UNKNOWN','OWNED','LICENSED','PUBLIC_DOMAIN')),
                original_name TEXT NOT NULL CHECK(length(original_name) BETWEEN 1 AND 240),
                state TEXT NOT NULL DEFAULT 'ACTIVE' CHECK(state IN ('ACTIVE','TRASHED')),
                current_revision_id TEXT NOT NULL,
                dhash TEXT,
                dhash_revision_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS media_revisions (
                id TEXT PRIMARY KEY,
                media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
                number INTEGER NOT NULL CHECK(number >= 1),
                kind TEXT NOT NULL CHECK(kind IN ('ORIGINAL','EDIT')),
                image_path TEXT NOT NULL UNIQUE,
                width INTEGER NOT NULL CHECK(width > 0),
                height INTEGER NOT NULL CHECK(height > 0),
                size_bytes INTEGER NOT NULL CHECK(size_bytes > 0),
                sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
                operations_json TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(media_id, number)
            );
            CREATE TABLE IF NOT EXISTS media_tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(name) BETWEEN 1 AND 60)
            );
            CREATE TABLE IF NOT EXISTS media_item_tags (
                media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
                tag_id TEXT NOT NULL REFERENCES media_tags(id) ON DELETE CASCADE,
                PRIMARY KEY(media_id, tag_id)
            );
            CREATE TABLE IF NOT EXISTS media_collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(name) BETWEEN 1 AND 60),
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS media_collection_items (
                collection_id TEXT NOT NULL REFERENCES media_collections(id) ON DELETE CASCADE,
                media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
                PRIMARY KEY(collection_id, media_id)
            );
            ");
        for (table, column) in [("artworks", "image_media_id"), ("artwork_images", "media_id"), ("collector_images", "media_id")] {
            if !has_column(table, column)? {
                script.push_str(&format!("ALTER TABLE {table} ADD COLUMN {column} TEXT REFERENCES media_items(id) ON DELETE SET NULL;"));
            }
        }
        script.push_str("CREATE INDEX IF NOT EXISTS idx_media_revisions_media ON media_revisions(media_id, number);
            CREATE INDEX IF NOT EXISTS idx_artworks_media ON artworks(image_media_id);
            CREATE INDEX IF NOT EXISTS idx_artwork_images_media ON artwork_images(media_id);
            CREATE INDEX IF NOT EXISTS idx_collector_images_media ON collector_images(media_id);
            PRAGMA user_version=13;
            COMMIT;");
        conn.execute_batch(&script)?;
    }
    if version < 14 {
        // Artwork flags (signed / final varnish / framed) and the type of each additional photograph.
        let has_column = |table: &str, column: &str| -> Result<bool, rusqlite::Error> {
            conn.query_row(&format!("SELECT count(*) FROM pragma_table_info('{table}') WHERE name='{column}'"), [], |r| r.get::<_, i64>(0)).map(|n| n > 0)
        };
        let mut script = String::from("BEGIN IMMEDIATE;");
        for column in ["signed", "final_varnish", "framed"] {
            if !has_column("artworks", column)? {
                script.push_str(&format!("ALTER TABLE artworks ADD COLUMN {column} INTEGER NOT NULL DEFAULT 0 CHECK({column} IN (0,1));"));
            }
        }
        if !has_column("artwork_images", "kind")? {
            script.push_str("ALTER TABLE artwork_images ADD COLUMN kind TEXT NOT NULL DEFAULT 'OTHER'
                CHECK(kind IN ('DETAIL','SIGNATURE','DIGITAL_MOCKUP','IN_SITU','OTHER','FRAMED','WORK_IN_PROGRESS'));");
        }
        script.push_str("PRAGMA user_version=14; COMMIT;");
        conn.execute_batch(&script)?;
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
            assert_eq!(v, SCHEMA_VERSION);
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at) VALUES('x','Test','a','a')",[]).unwrap();
        }
        drop(db);
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM collectors",[],|r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
    #[test]
    fn v14_migration_adds_flags_and_photo_kind_keeping_existing_rows() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES('a','A-1','Opera','t','t')", []).unwrap();
            c.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at) VALUES('g','a','media/g.jpg','t')", []).unwrap();
            c.execute_batch("ALTER TABLE artworks DROP COLUMN signed; ALTER TABLE artworks DROP COLUMN final_varnish;
                ALTER TABLE artworks DROP COLUMN framed; ALTER TABLE artwork_images DROP COLUMN kind;").unwrap();
            c.pragma_update(None, "user_version", 13).unwrap();
        }
        drop(db);
        let db = open(dir.path().into()).unwrap();
        let c = db.conn.lock().unwrap();
        let flags: (bool, bool, bool) = c.query_row("SELECT signed,final_varnish,framed FROM artworks WHERE id='a'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
        assert_eq!(flags, (false, false, false));
        let kind: String = c.query_row("SELECT kind FROM artwork_images WHERE id='g'", [], |r| r.get(0)).unwrap();
        assert_eq!(kind, "OTHER");
        assert!(c.execute("UPDATE artwork_images SET kind='ALTRO' WHERE id='g'", []).is_err(), "unknown kinds are refused by the database");
        assert!(c.execute("UPDATE artworks SET signed=2 WHERE id='a'", []).is_err());
        let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0)).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
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
    fn drop_receipts_for_old_schema_test(c: &Connection) {
        c.execute_batch("DROP TRIGGER sale_receipts_limit_insert;
            DROP TRIGGER sale_receipts_limit_update;
            DROP TRIGGER sales_no_shrink_below_receipts;
            DROP TABLE sale_receipt_attachments;
            DROP TABLE sale_receipts;").unwrap();
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
            c.execute("DROP TABLE artwork_images",[]).unwrap();
            drop_receipts_for_old_schema_test(&c);
            c.pragma_update(None,"user_version",1).unwrap();
        }
        drop(db);
        let migrated = open(dir.path().into()).unwrap();
        let c = migrated.conn.lock().unwrap();
        let v: i64 = c.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(v,SCHEMA_VERSION);
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
            drop_receipts_for_old_schema_test(&conn);
            conn.pragma_update(None,"user_version",2).unwrap();
        }
        drop(db);
        let upgraded = open(temp.path().into()).unwrap();
        let conn = upgraded.conn.lock().unwrap();
        let version: i64 = conn.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,SCHEMA_VERSION);
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
    fn gallery_order_migration_preserves_legacy_chronology_and_files() {
        let temp = tempfile::tempdir().unwrap();
        let db = open(temp.path().into()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            for id in ["a","b"] {
                c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                    VALUES(?1,?1,?1,'t','t')", rusqlite::params![id]).unwrap();
            }
            for (id, art, created) in [("z","a","2026-01-02"),("a1","a","2026-01-02"),
                ("m","a","2026-01-01"),("other","b","2026-01-02")] {
                c.execute("INSERT INTO artwork_images(id,artwork_id,image_path,created_at)
                    VALUES(?1,?2,?3,?4)",
                    rusqlite::params![id,art,format!("media/{id}.png"),created]).unwrap();
            }
            c.execute_batch("DROP INDEX idx_artwork_images_position;
                ALTER TABLE artwork_images DROP COLUMN sort_order;
                PRAGMA user_version=4;").unwrap();
        }
        drop(db);
        let upgraded = open(temp.path().into()).unwrap();
        let c = upgraded.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id,sort_order,image_path FROM artwork_images
            WHERE artwork_id='a' ORDER BY sort_order,id").unwrap();
        let actual: Vec<(String,i64,String)> = stmt.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))
            .unwrap().map(Result::unwrap).collect();
        assert_eq!(actual, vec![
            ("m".into(),1,"media/m.png".into()),("a1".into(),2,"media/a1.png".into()),
            ("z".into(),3,"media/z.png".into())]);
        assert_eq!(c.query_row("SELECT sort_order FROM artwork_images WHERE id='other'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        let version: i64 = c.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,SCHEMA_VERSION);
    }
    #[test]
    fn v5_to_v6_adds_empty_attachment_table_without_changing_receipts() {
        let temp = tempfile::tempdir().unwrap();
        let db = open(temp.path().into()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
                VALUES('a','A','Opera','t','t')", []).unwrap();
            c.execute("INSERT INTO collectors(id,name,created_at,updated_at)
                VALUES('c','Persona','t','t')", []).unwrap();
            c.execute("INSERT INTO sales(id,artwork_id,collector_id,amount_cents,sold_at,created_at)
                VALUES('s','a','c',1000,'2026-11-21','t')", []).unwrap();
            c.execute("INSERT INTO sale_receipts(id,sale_id,received_at,amount_cents,method,created_at)
                VALUES('r','s','2026-11-21',400,'cash','t')", []).unwrap();
            c.execute_batch("DROP TABLE sale_receipt_attachments; PRAGMA user_version=5;").unwrap();
        }
        drop(db);
        let upgraded = open(temp.path().into()).unwrap();
        let c = upgraded.conn.lock().unwrap();
        let version:i64 = c.pragma_query_value(None, "user_version", |r|r.get(0)).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        let paid:i64 = c.query_row("SELECT amount_cents FROM sale_receipts WHERE id='r'",[],|r|r.get(0)).unwrap();
        assert_eq!(paid, 400);
        let attachments:i64 = c.query_row("SELECT count(*) FROM sale_receipt_attachments",[],|r|r.get(0)).unwrap();
        assert_eq!(attachments, 0);
        assert!(c.execute("INSERT INTO sale_receipt_attachments(receipt_id,media_path,original_name,mime_type,size_bytes,sha256,created_at)
            VALUES('unknown','media/receipt-unknown.pdf','unknown.pdf','application/pdf',8,?1,'t')",
            rusqlite::params!["0".repeat(64)]).is_err());
    }
    #[test]
    fn v7_to_v8_keeps_collector_names_and_adds_photo_storage() {
        let temp = tempfile::tempdir().unwrap();
        let db = open(temp.path().into()).unwrap();
        {
            let c = db.conn.lock().unwrap();
            c.execute("INSERT INTO collectors(id,name,email,created_at,updated_at) VALUES('old','Mario Bianchi','m@example.org','t','t')", []).unwrap();
            c.execute_batch("DROP TABLE collector_images;
                DROP INDEX IF EXISTS idx_collector_profile_photo;
                ALTER TABLE collectors DROP COLUMN first_name;
                ALTER TABLE collectors DROP COLUMN last_name;
                ALTER TABLE collectors DROP COLUMN linkedin;
                PRAGMA user_version=7;").unwrap();
        }
        drop(db);
        let upgraded = open(temp.path().into()).unwrap();
        let c = upgraded.conn.lock().unwrap();
        let (name, first, last, email): (String, String, Option<String>, String) = c.query_row(
            "SELECT name,first_name,last_name,email FROM collectors WHERE id='old'", [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).unwrap();
        assert_eq!((name.as_str(), first.as_str(), last, email.as_str()), ("Mario Bianchi", "Mario Bianchi", None, "m@example.org"));
        let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0)).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        c.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at) VALUES('i','old','PROFILE','media/a.jpg','t')", []).unwrap();
        assert!(c.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at) VALUES('j','old','PROFILE','media/b.jpg','t')", []).is_err(), "one profile photo per collector");
        assert!(c.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at) VALUES('k','old','BANNER','media/c.jpg','t')", []).is_err());
        assert!(c.execute("INSERT INTO collector_images(id,collector_id,kind,image_path,created_at) VALUES('l','nobody','PHOTO','media/d.jpg','t')", []).is_err());
    }
    #[test]
    fn newer_schema_is_not_downgraded() {
        let dir = tempfile::tempdir().unwrap();
        let db = open(dir.path().into()).unwrap();
        db.conn.lock().unwrap().pragma_update(None, "user_version", SCHEMA_VERSION + 1).unwrap();
        drop(db);
        assert!(open(dir.path().into()).is_err());
    }
}
