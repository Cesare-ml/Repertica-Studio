//! Composition of the two independent v19 histories. Local only; no IPC or transport.
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use uuid::Uuid;
#[path = "auction_preview_payload.rs"]
mod preview_payload;
#[path = "auction_draft_record.rs"]
mod draft_record;

const DRAFT_SQL: &str = include_str!("auction_draft_schema.sql");
const LEDGER_SQL: &str = include_str!("auction_settlement_v20.sql");
const MAIL_SQL: &str = "ALTER TABLE studio_settings ADD COLUMN mail_notifications INTEGER NOT NULL DEFAULT 1 CHECK(mail_notifications IN (0,1));";

type Result<T> = std::result::Result<T,String>;
fn error(_: rusqlite::Error) -> String { "auction_archive_schema_invalid".into() }
fn version(c: &Connection) -> Result<i64> {
    c.pragma_query_value(None,"user_version",|r|r.get(0)).map_err(error)
}
fn object(c: &Connection, name: &str, kind: &str) -> Result<bool> {
    let found:Option<String> = c.query_row("SELECT type FROM sqlite_schema WHERE name=?1",[name],|r|r.get(0)).optional().map_err(error)?;
    match found.as_deref() {
        None=>Ok(false), Some(actual) if actual==kind=>Ok(true), _=>Err("auction_archive_schema_invalid".into())
    }
}
const LEDGER_OBJECTS: &[(&str,&str)] = &[
    ("auction_published_bindings","table"),("auction_settlement_receipts","table"),
    ("auction_published_bindings_work","index"),
    ("auction_published_bindings_no_replace","trigger"),("auction_published_bindings_no_update","trigger"),
    ("auction_published_bindings_no_delete","trigger"),("auction_settlement_receipts_no_replace","trigger"),
    ("auction_settlement_receipts_ack_only","trigger"),("auction_settlement_receipts_no_delete","trigger"),
];
#[derive(Clone,Copy)]
struct Layout { mail:bool, drafts:bool, ledger:bool }
fn layout(c:&Connection,v:i64)->Result<Layout>{
    let mail:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('studio_settings') WHERE name='mail_notifications')",[],|r|r.get(0)).map_err(error)?;
    let scope=object(c,"auction_archive_scope","table")?;
    let drafts=object(c,"auction_drafts","table")?;
    let mut n=0;
    for &(name,kind) in LEDGER_OBJECTS { if object(c,name,kind)? { n+=1; } }
    let ledger=n==LEDGER_OBJECTS.len();
    if scope!=drafts || (n!=0&&!ledger) || (ledger&&!drafts)
        || (v==19&&!mail&&!drafts) || (v==20&&!ledger)
        || (v==21&&!(mail&&drafts&&ledger)) {
        return Err("auction_archive_schema_invalid".into());
    }
    Ok(Layout{mail,drafts,ledger})
}
fn validate_mail(c:&Connection)->Result<()>{
    let valid:bool=c.query_row("SELECT count(*)=1 AND COALESCE(SUM(singleton=1 AND typeof(mail_notifications)='integer' AND mail_notifications IN (0,1)),0)=1 FROM studio_settings",[],|r|r.get(0)).map_err(error)?;
    if !valid { return Err("auction_archive_mail_invalid".into()); }
    Ok(())
}
fn validate_drafts(c:&Connection)->Result<()>{
    let count:i64=c.query_row("SELECT count(*) FROM auction_archive_scope",[],|r|r.get(0)).map_err(error)?;
    let scope:String=c.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1",[],|r|r.get(0)).map_err(error)?;
    if count!=1 || !Uuid::parse_str(&scope).is_ok_and(|u|u.to_string()==scope) { return Err("auction_archive_scope_invalid".into()); }
    let mut stmt=c.prepare("SELECT artwork_id,revision,record_json FROM auction_drafts").map_err(error)?;
    let mut rows=stmt.query([]).map_err(error)?;
    while let Some(row)=rows.next().map_err(error)? {
        let id:String=row.get(0).map_err(error)?;
        let revision:String=row.get(1).map_err(error)?;
        let raw:String=row.get(2).map_err(error)?;
        if id.is_empty() || !preview_payload::bounded(&id,256) || id.chars().any(char::is_control) || Uuid::parse_str(&revision).is_err() {
            return Err("invalid_stored_draft".into());
        }
        draft_record::parse_record(&raw,&scope,&id)?;
    }
    Ok(())
}
fn validate(c:&Connection,l:Layout)->Result<()>{
    if l.mail { validate_mail(c)?; }
    if l.drafts { validate_drafts(c)?; }
    if l.ledger { crate::settlement_store::inspect_snapshot(c).map_err(str::to_owned)?; }
    Ok(())
}
/// Validate old and current backups read-only, without upgrading a snapshot.
pub(crate) fn inspect(c:&Connection)->Result<()>{
    let v=version(c)?;
    if !(1..=21).contains(&v) { return Err("auction_archive_schema_invalid".into()); }
    validate(c,layout(c,v)?)
}
/// Run only after core migrations through v18. All new DDL, scope creation,
/// semantic checks and the version bump share one immediate transaction.
pub(crate) fn upgrade(c:&mut Connection)->Result<()>{
    let v=version(c)?;
    if v==21 { return inspect(c); }
    if !(18..=20).contains(&v) { return Err("auction_archive_schema_invalid".into()); }
    let tx=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(error)?;
    let v=version(&tx)?;
    if !(18..=20).contains(&v) { return Err("auction_archive_schema_invalid".into()); }
    let l=layout(&tx,v)?;
    // A legacy v20 ledger must be validated before writing anything.
    // v18 development fixtures can carry a complete newer schema; validate
    // their ledger after setting the version within this same transaction.
    if l.mail { validate_mail(&tx)?; }
    if l.drafts { validate_drafts(&tx)?; }
    if l.ledger && v==20 { crate::settlement_store::inspect_snapshot(&tx).map_err(str::to_owned)?; }
    if !l.mail { tx.execute_batch(MAIL_SQL).map_err(error)?; }
    if !l.drafts {
        tx.execute_batch(DRAFT_SQL).map_err(error)?;
        tx.execute("INSERT INTO auction_archive_scope(singleton,scope) VALUES(1,?1)",[Uuid::new_v4().to_string()]).map_err(error)?;
    }
    if !l.ledger { tx.execute_batch(LEDGER_SQL).map_err(error)?; }
    tx.pragma_update(None,"user_version",21).map_err(error)?;
    inspect(&tx)?;
    let orphan:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",[],|r|r.get(0)).map_err(error)?;
    if orphan { return Err("auction_archive_schema_invalid".into()); }
    tx.commit().map_err(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db,backup,settlement_ledger as ledger,settlement_store as store};
    use rusqlite::params;
    use serde_json::json;
    const REV:&str="aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    fn scalar(c:&Connection,sql:&str)->String { c.query_row(sql,[],|r|r.get(0)).unwrap() }
    fn scope(c:&Connection)->String { scalar(c,"SELECT scope FROM auction_archive_scope") }
    fn drop_ledger(c:&Connection){c.execute_batch("DROP TABLE auction_settlement_receipts; DROP TABLE auction_published_bindings;").unwrap();}
    fn drop_drafts(c:&Connection){c.execute_batch("DROP TABLE auction_drafts; DROP TABLE auction_archive_scope;").unwrap();}
    fn drop_mail(c:&Connection){c.execute_batch("ALTER TABLE studio_settings DROP COLUMN mail_notifications;").unwrap();}
    fn pending(c:&Connection)->String {
        let body=r#"{"tenantId":"tenant-test","slug":"art","title":"Art","description":"","startsAt":"2027-01-02T10:00:00.000Z","endsAt":"2027-01-03T10:00:00.000Z","startingAmount":1000,"minimumIncrementAmount":100,"currency":"EUR"}"#;
        let raw=json!({"version":1,"scope":scope(c),"draft":{"artworkId":"a","title":"Art","description":"","slug":"art","startsAt":"2027-01-02T10:00:00.000Z","endsAt":"2027-01-03T10:00:00.000Z","startingAmount":1000,"minimumIncrementAmount":100},"pending":{"artworkId":"a","idempotencyKey":"preview-0123456789","body":body}}).to_string();
        // Historical draft for a deleted artwork must survive.
        c.execute("INSERT INTO auction_drafts VALUES('a',?1,?2)",params![REV,raw]).unwrap();
        raw
    }
    fn legacy_ack(c:&mut Connection) -> (String,String) {
        let mut b=ledger::binding(); b.scope=scope(c);
        c.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES(?1,'FIX-1','Synthetic','t','t')",[&b.local_work]).unwrap();
        let value=store::PublishedBinding{binding:b.clone(),request_id:"dddddddd-dddd-4ddd-8ddd-dddddddddddd".into(),selection_digest:"a".repeat(64),published_at:"2026-11-24T03:00:00.000Z".into()};
        store::record(c,&value,&b.owner,|_|Ok(())).unwrap();
        ledger::history(c,&b,0,&serde_json::to_vec(&ledger::page(vec![ledger::delivery(1,"fulfilled")],1,None)).unwrap()).unwrap();
        let target=ledger::ack_target(c,&b).unwrap();
        let mut d=ledger::delivery(1,"fulfilled"); d["acknowledged_at"]=json!("2026-11-25T04:02:00.000Z");
        let mut a=ledger::reply(d); a["replayed"]=json!(false);
        ledger::ack(c,&b,&target,&serde_json::to_vec(&a).unwrap()).unwrap();
        c.execute("DELETE FROM artworks WHERE id=?1",[&b.local_work]).unwrap();
        (scalar(c,"SELECT snapshot FROM auction_settlement_receipts"),scalar(c,"SELECT acknowledged_at FROM auction_settlement_receipts"))
    }
    #[test]
    fn upgrades_four_real_layouts_without_rekeying_or_resetting_mail(){
        for (old,mail,has_draft,has_ledger) in [(18,None,false,false),(19,Some(0_i64),false,false),(19,Some(1),false,false),(19,None,true,false),(20,None,true,true)] {
            let dir=tempfile::tempdir().unwrap();
            let (old_scope,raw,ack)={
                let db=db::open(dir.path().into()).unwrap();
                let mut c=db.conn.lock().unwrap();
                let old_scope=scope(&c);
                let raw=has_draft.then(||pending(&c));
                let ack=has_ledger.then(||legacy_ack(&mut c));
                if !has_ledger {drop_ledger(&c);}
                if !has_draft {drop_drafts(&c);}
                if let Some(value)=mail {c.execute("UPDATE studio_settings SET mail_notifications=?1",[value]).unwrap();} else {drop_mail(&c);}
                c.pragma_update(None,"user_version",old).unwrap();
                inspect(&c).unwrap();
                (old_scope,raw,ack)
            };
            let db=db::open(dir.path().into()).unwrap();
            let c=db.conn.lock().unwrap();
            assert_eq!(version(&c).unwrap(),21);
            assert_eq!(c.query_row("SELECT mail_notifications FROM studio_settings",[],|r|r.get::<_,i64>(0)).unwrap(),mail.unwrap_or(1));
            if let Some(raw)=raw {assert_eq!(scope(&c),old_scope);assert_eq!(scalar(&c,"SELECT record_json FROM auction_drafts"),raw);assert_eq!(scalar(&c,"SELECT revision FROM auction_drafts"),REV);}
            if let Some((snapshot,at))=ack {assert_eq!(scalar(&c,"SELECT snapshot FROM auction_settlement_receipts"),snapshot);assert_eq!(scalar(&c,"SELECT acknowledged_at FROM auction_settlement_receipts"),at);}
            assert_eq!(c.query_row("SELECT count(*) FROM sales",[],|r|r.get::<_,i64>(0)).unwrap(),0);
            let saved_scope=scope(&c);drop(c);drop(db);
            let reopened=db::open(dir.path().into()).unwrap();
            assert_eq!(scope(&reopened.conn.lock().unwrap()),saved_scope);
        }
    }
    #[test]
    fn rejects_partial_or_corrupt_legacy_without_repairing_identity(){
        for kind in 0..3 {
            let dir=tempfile::tempdir().unwrap();
            {
                let db=db::open(dir.path().into()).unwrap();
                let c=db.conn.lock().unwrap();
                drop_ledger(&c);drop_mail(&c);c.pragma_update(None,"user_version",19).unwrap();
                match kind {
                    0=>{c.execute("DELETE FROM auction_archive_scope",[]).unwrap();},
                    1=>{c.execute("DROP TABLE auction_drafts",[]).unwrap();},
                    _=>{pending(&c);c.execute("UPDATE auction_drafts SET record_json='{}'",[]).unwrap();}
                }
                assert!(inspect(&c).is_err());
            }
            assert!(db::open(dir.path().into()).is_err());
            let c=Connection::open(dir.path().join("repertica.sqlite")).unwrap();
            assert_eq!(version(&c).unwrap(),19);
            assert!(!layout(&c,19).is_ok_and(|l|l.mail));
            assert!(!object(&c,"auction_published_bindings","table").unwrap());
        }
    }
    #[test]
    fn late_failure_rolls_back_mail_and_ledger_ddl(){
        let dir=tempfile::tempdir().unwrap();
        let db=db::open(dir.path().into()).unwrap();
        let mut c=db.conn.lock().unwrap();
        drop_ledger(&c);drop_mail(&c);
        let raw=pending(&c);let old_scope=scope(&c);
        c.pragma_update(None,"user_version",19).unwrap();
        // Conflicting unrelated index on the would-be trigger name is detected
        // before DDL. An orphan created with FK off tests failure AFTER DDL.
        c.execute_batch("PRAGMA foreign_keys=OFF; INSERT INTO artwork_images(id,artwork_id,image_path,created_at) VALUES('x','missing','media/x.jpg','t'); PRAGMA foreign_keys=ON;").unwrap();
        assert!(upgrade(&mut c).is_err());
        assert_eq!(version(&c).unwrap(),19);
        let l=layout(&c,19).unwrap();assert!(!l.mail&&!l.ledger&&l.drafts);
        assert_eq!(scope(&c),old_scope);assert_eq!(scalar(&c,"SELECT record_json FROM auction_drafts"),raw);
    }
    #[test]
    fn application_backup_restore_preserves_mail_draft_and_ack(){
        let dir=tempfile::tempdir().unwrap();let destination=tempfile::tempdir().unwrap();
        let db=db::open(dir.path().into()).unwrap();
        let (raw,old_scope,ack)={
            let mut c=db.conn.lock().unwrap();c.execute("UPDATE studio_settings SET mail_notifications=0",[]).unwrap();
            (pending(&c),scope(&c),legacy_ack(&mut c))
        };
        let b=backup::create_backup_core(&db,destination.path()).unwrap();
        let folder=std::path::Path::new(&b.directory);
        backup::inspect_backup_core(folder).unwrap();
        db.conn.lock().unwrap().execute("UPDATE studio_settings SET mail_notifications=1",[]).unwrap();
        backup::restore_backup_core(&db,folder).unwrap();
        let c=db.conn.lock().unwrap();
        assert_eq!(scope(&c),old_scope);assert_eq!(scalar(&c,"SELECT record_json FROM auction_drafts"),raw);
        assert_eq!(scalar(&c,"SELECT snapshot FROM auction_settlement_receipts"),ack.0);
        assert_eq!(scalar(&c,"SELECT acknowledged_at FROM auction_settlement_receipts"),ack.1);
        assert_eq!(c.query_row("SELECT mail_notifications FROM studio_settings",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(c.query_row("SELECT count(*) FROM sales",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    }
}
