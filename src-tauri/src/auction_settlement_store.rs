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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    fn open() -> (tempfile::TempDir, Connection, PublishedBinding) {
        let dir=tempfile::tempdir().unwrap();
        let archive=repertica_studio_lib::db::open(dir.path().to_path_buf()).unwrap();
        let mut conn=archive.conn.into_inner().unwrap();
        let mut binding=super::super::settlement_ledger::binding();
        binding.scope=conn.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1",[],|r|r.get(0)).unwrap();
        conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES(?1,'FIX-1','Synthetic','t','t')",
            [&binding.local_work]).unwrap();
        migrate(&mut conn).unwrap();
        let value=PublishedBinding{binding,request_id:"dddddddd-dddd-4ddd-8ddd-dddddddddddd".into(),
            selection_digest:"a".repeat(64),published_at:"2026-11-24T03:00:00.000Z".into()};
        (dir,conn,value)
    }
    #[test]
    fn persistent_mapping_requires_current_exact_authority_and_never_guesses() {
        let (_dir,mut conn,v)=open();
        let b=&v.binding;
        assert_eq!(resolve_selected(&conn,&b.scope,&b.local_work,&b.owner,|_|panic!("no binding")),Ok(None));
        assert_eq!(record(&mut conn,&v,"repertica:99",|_|panic!("wrong owner")),Err("settlement_binding_mismatch"));
        let calls=Cell::new(0);
        assert_eq!(record(&mut conn,&v,&b.owner,|_|{
            calls.set(calls.get()+1);
            if calls.get()==1 { Ok(()) } else { Err("settlement_access_denied") }
        }),Err("settlement_access_denied"));
        // The historical row survives a raced revocation, but no summary can
        // be resolved without the live, independently verified authority.
        assert_eq!(resolve_selected(&conn,&b.scope,&b.local_work,&b.owner,|_|Err("settlement_access_denied")),
            Err("settlement_access_denied"));
        record(&mut conn,&v,&b.owner,|_|Ok(())).unwrap();
        let mut changed=v.clone();
        changed.selection_digest="b".repeat(64);
        assert_eq!(record(&mut conn,&changed,&b.owner,|_|Ok(())),Err("settlement_binding_conflict"));
        assert_eq!(resolve_selected(&conn,&b.scope,&b.local_work,"repertica:99",|_|panic!("wrong owner")),Ok(None));
        assert_eq!(resolve_selected(&conn,&b.scope,&b.local_work,&b.owner,|candidate|{
            assert_eq!(candidate,&v);Ok(())
        }),Ok(Some(b.clone())));
        let mut other=v.clone();
        other.binding.auction="second-auction".into();
        other.binding.association="second-association".into();
        other.request_id="eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee".into();
        record(&mut conn,&other,&b.owner,|_|Ok(())).unwrap();
        assert_eq!(resolve_selected(&conn,&b.scope,&b.local_work,&b.owner,|_|panic!("ambiguous")),
            Err("settlement_binding_ambiguous"));
        conn.execute("DELETE FROM artworks WHERE id=?1",[&b.local_work]).unwrap();
        assert_eq!(resolve_selected(&conn,&b.scope,&b.local_work,&b.owner,|_|panic!("deleted work")),
            Err("settlement_binding_mismatch"));
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM auction_published_bindings",[],|r|r.get::<_,u64>(0)).unwrap(),2);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM sales",[],|r|r.get::<_,u64>(0)).unwrap(),0);
    }
}
