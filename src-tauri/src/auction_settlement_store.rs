//! Explicit, not yet mounted, v20 migration and native published-binding store.
//! Historical rows never confer authorization. The caller must supply a live,
//! exact publication-authority verifier owned by the native backend.
use super::settlement_ledger::Binding;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use chrono::{DateTime, SecondsFormat, Utc};

type Error = &'static str;
const SCHEMA: &str = include_str!("auction_settlement_v20.sql");

pub(crate) fn migrate(conn: &mut Connection) -> Result<(), Error> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let v: i64 = tx.pragma_query_value(None, "user_version", |r| r.get(0)).map_err(db)?;
    if v != 19 { return Err("settlement_schema_unavailable"); }
    tx.execute_batch(SCHEMA).map_err(db)?;
    tx.pragma_update(None, "user_version", 20).map_err(db)?;
    tx.commit().map_err(db)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublishedBinding {
    pub(crate) binding: Binding,
    pub(crate) request_id: String,
    pub(crate) selection_digest: String,
    pub(crate) published_at: String,
}

impl PublishedBinding {
    fn valid(&self) -> bool {
        self.binding.valid()
            && uuid::Uuid::parse_str(&self.request_id).is_ok_and(|u| u.to_string() == self.request_id)
            && self.selection_digest.len() == 64
            && self.selection_digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            && DateTime::parse_from_rfc3339(&self.published_at).is_ok_and(|d|
                d.with_timezone(&Utc).to_rfc3339_opts(SecondsFormat::Millis, true) == self.published_at)
    }
}

fn db(_: rusqlite::Error) -> Error { "settlement_storage_failed" }

fn current(conn: &Connection, scope: &str, work: &str) -> Result<(), Error> {
    let actual: String = conn.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1", [], |r| r.get(0)).map_err(db)?;
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)", [work], |r| r.get(0)).map_err(db)?;
    if actual != scope || !exists { return Err("settlement_binding_mismatch"); }
    Ok(())
}
fn decode_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PublishedBinding> {
    Ok(PublishedBinding {
        binding: Binding { scope:r.get(0)?, auction:r.get(1)?, tenant:r.get(2)?, association:r.get(3)?,
            cloud_work:r.get(4)?, local_work:r.get(5)?, owner:r.get(6)? },
        request_id:r.get(7)?, selection_digest:r.get(8)?, published_at:r.get(9)?,
    })
}
const COLUMNS: &str = "scope,auction,tenant,association,cloud_work,local_work,owner,request_id,selection_digest,published_at";

// Never call this from an IPC payload or from a private preview receipt.
// The native caller must prove the exact post-publication mapping with a
// current, independent authority, including the owner and local selection.
pub(crate) fn record(conn: &mut Connection, value: &PublishedBinding,
    verified_subject: &str, verify_live: impl Fn(&PublishedBinding) -> Result<(), Error>) -> Result<(), Error> {
    let b = &value.binding;
    if !value.valid() || b.owner != verified_subject { return Err("settlement_binding_mismatch"); }
    current(conn,&b.scope,&b.local_work)?;
    verify_live(value)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    current(&tx,&b.scope,&b.local_work)?;
    let select = format!("SELECT {COLUMNS} FROM auction_published_bindings WHERE scope=?1 AND auction=?2");
    let previous = tx.query_row(&select,params![b.scope,b.auction],decode_row).optional().map_err(db)?;
    if let Some(previous) = previous {
        if previous != *value { return Err("settlement_binding_conflict"); }
    } else {
        let n = tx.execute("INSERT INTO auction_published_bindings(scope,auction,tenant,association,cloud_work,local_work,owner,request_id,selection_digest,published_at)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![b.scope,b.auction,b.tenant,b.association,b.cloud_work,b.local_work,b.owner,
                value.request_id,value.selection_digest,value.published_at]).map_err(db)?;
        if n != 1 || tx.query_row(&select,params![b.scope,b.auction],decode_row).map_err(db)? != *value {
            return Err("settlement_binding_conflict");
        }
    }
    tx.commit().map_err(db)?;
    // A revocation racing the commit leaves only historical evidence. It does
    // not authorize subsequent use, and the operation reports the refusal.
    verify_live(value)
}

// Resolve a single selected artwork only. More than one historical auction
// must be explicitly selected via a future native-owned list; never guess.
pub(crate) fn resolve_selected(conn: &Connection, scope: &str, work: &str, subject: &str,
    verify_live: impl Fn(&PublishedBinding) -> Result<(), Error>) -> Result<Option<Binding>, Error> {
    current(conn,scope,work)?;
    let select = format!("SELECT {COLUMNS} FROM auction_published_bindings
        WHERE scope=?1 AND local_work=?2 AND owner=?3 ORDER BY auction LIMIT 2");
    let mut stmt = conn.prepare(&select).map_err(db)?;
    let mut rows = stmt.query(params![scope,work,subject]).map_err(db)?;
    let Some(row) = rows.next().map_err(db)? else { return Ok(None); };
    let value = decode_row(row).map_err(db)?;
    if rows.next().map_err(db)?.is_some() { return Err("settlement_binding_ambiguous"); }
    drop(rows);
    drop(stmt);
    if !value.valid() || value.binding.scope != scope || value.binding.local_work != work ||
        value.binding.owner != subject { return Err("settlement_binding_mismatch"); }
    verify_live(&value)?;
    current(conn,scope,work)?;
    Ok(Some(value.binding))
}
