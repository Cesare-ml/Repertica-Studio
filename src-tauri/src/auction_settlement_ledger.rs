//! Native settlement parser and ledger, independent of transport and renderer.
//! v20 provisions historical evidence in the local archive.
//! No release provider or accounting.
//! Binding must eventually come from the trusted immutable published resolver;
//! private staging and preview receipts do not authorize settlement.
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Deserializer, Serialize};
use chrono::DateTime;

type Error = &'static str;
const MAX_SAFE:u64 = 9_007_199_254_740_991;
#[cfg(test)]
const SCHEMA:&str = r#"
CREATE TABLE auction_settlement_receipts (
  scope TEXT NOT NULL, auction TEXT NOT NULL, tenant TEXT NOT NULL,
  association TEXT NOT NULL, cloud_work TEXT NOT NULL, local_work TEXT NOT NULL,
  owner TEXT NOT NULL, sequence INTEGER NOT NULL CHECK(sequence>0),
  snapshot TEXT NOT NULL, acknowledged_at TEXT,
  PRIMARY KEY(scope,auction,sequence)
);
"#;

// Trusted native fixture, not constructed from the response or renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Binding {
    pub(crate) scope:String, pub(crate) auction:String, pub(crate) tenant:String, pub(crate) association:String,
    pub(crate) cloud_work:String, pub(crate) local_work:String, pub(crate) owner:String,
}
fn id(s:&str,max:usize)->bool {
    !s.is_empty() && s.len()<=max && s.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b':'|b'_'|b'-'))
}
fn uuid(s:&str)->bool { uuid::Uuid::parse_str(s).is_ok_and(|u|u.to_string()==s) }
fn positive(n:u64)->bool { n>0 && n<=MAX_SAFE }
fn date(s:&str)->bool { s.len()<=40 && DateTime::parse_from_rfc3339(s).is_ok() }
impl Binding {
    pub(crate) fn valid(&self)->bool {
        uuid(&self.scope)&&uuid(&self.local_work)&&id(&self.auction,128)&&id(&self.tenant,80)&&
        id(&self.association,128)&&id(&self.cloud_work,128)&&
        self.owner.strip_prefix("repertica:").and_then(|s|s.parse::<i32>().ok())
            .is_some_and(|n|n>0&&self.owner==format!("repertica:{n}"))
    }
}
fn required_ack<'de,D:Deserializer<'de>>(d:D)->Result<Option<String>,D::Error>{Option::deserialize(d)}
fn required_cursor<'de,D:Deserializer<'de>>(d:D)->Result<Option<u64>,D::Error>{Option::deserialize(d)}
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct Delivery {
    auction_id:String, result_id:String, closure_version:u64, delivery_sequence:u64,
    settlement_revision:u64, runtime_version:u64, fulfillment_revision:u64,
    amount:u64, currency:String, fulfilled_at:String, status:String,
    #[serde(deserialize_with="required_ack")]
    acknowledged_at:Option<String>,
}
impl Delivery {
    fn valid(&self,b:&Binding)->bool {
        self.auction_id==b.auction && positive(self.closure_version) &&
        self.result_id==format!("{}:{}:fulfillment",b.auction,self.closure_version) &&
        positive(self.delivery_sequence)&&positive(self.settlement_revision)&&positive(self.runtime_version)&&
        positive(self.fulfillment_revision)&&self.fulfillment_revision<=self.settlement_revision&&
        positive(self.amount)&&self.currency=="EUR"&&matches!(self.status.as_str(),"fulfilled"|"review_required")&&
        date(&self.fulfilled_at)&&self.acknowledged_at.as_deref().is_none_or(date)
    }
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Poll {ok:bool,artwork_id:String,association_id:String,delivery:Delivery}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct Ack {ok:bool,artwork_id:String,association_id:String,delivery:Delivery,replayed:bool}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct HistoryReply {ok:bool,artwork_id:String,association_id:String,history:History}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct History {
    latest_sequence:u64,items:Vec<Delivery>,
    #[serde(deserialize_with="required_cursor")]
    next_after_sequence:Option<u64>,
}
fn decode<T:serde::de::DeserializeOwned>(bytes:&[u8])->Result<T,Error>{
    if bytes.len()>65536 {return Err("settlement_response_too_large")}
    serde_json::from_slice(bytes).map_err(|_|"settlement_invalid_response")
}
fn envelope(b:&Binding,ok:bool,art:&str,link:&str)->Result<(),Error>{
    if !b.valid()||!ok||art!=b.cloud_work||link!=b.association{return Err("settlement_binding_mismatch")}
    Ok(())
}
fn db(_:rusqlite::Error)->Error{"settlement_storage_failed"}
fn latest(t:&Transaction<'_>,b:&Binding)->Result<u64,Error>{
    if !b.valid(){return Err("settlement_binding_mismatch")}
    let scope:String=t.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1",[],|r|r.get(0)).map_err(db)?;
    let work:bool=t.query_row("SELECT EXISTS(SELECT 1 FROM artworks WHERE id=?1)",[&b.local_work],|r|r.get(0)).map_err(db)?;
    if scope!=b.scope||!work{return Err("settlement_binding_mismatch")}
    // An arbitrary native Binding or a preview receipt is not sufficient,
    // even when the ledger is empty. Require the exact historical mapping
    // before proposing any request. Live authority is checked separately.
    let bound:bool=t.query_row("SELECT EXISTS(SELECT 1 FROM auction_published_bindings
        WHERE scope=?1 AND auction=?2 AND tenant=?3 AND association=?4 AND cloud_work=?5 AND local_work=?6 AND owner=?7)",
        params![b.scope,b.auction,b.tenant,b.association,b.cloud_work,b.local_work,b.owner],|r|r.get(0)).map_err(db)?;
    if !bound{return Err("settlement_binding_mismatch")}
    let conflict:bool=t.query_row("SELECT EXISTS(SELECT 1 FROM auction_settlement_receipts WHERE scope=?1 AND auction=?2
        AND (tenant<>?3 OR association<>?4 OR cloud_work<>?5 OR local_work<>?6 OR owner<>?7))",
        params![b.scope,b.auction,b.tenant,b.association,b.cloud_work,b.local_work,b.owner],|r|r.get(0)).map_err(db)?;
    if conflict{return Err("settlement_binding_mismatch")}
    let n=validate_history(t,b)?;
    validate_ack_journal(t)?;
    Ok(n)
}

// Validate every persisted row, not only MAX(sequence) or the latest snapshot.
// A damaged/restored ledger must never propose a remote cursor or ACK which
// skips earlier evidence. Streaming rows avoids allocating the entire history.
// This helper is read-only and grants no authority; the caller checks binding.
pub(crate) fn validate_history(c:&Connection,b:&Binding)->Result<u64,Error>{
    if !b.valid(){return Err("settlement_binding_mismatch")}
    let mut stmt=c.prepare("SELECT sequence,snapshot,acknowledged_at FROM auction_settlement_receipts
        WHERE scope=?1 AND auction=?2 ORDER BY sequence").map_err(db)?;
    let mut rows=stmt.query(params![b.scope,b.auction]).map_err(db)?;
    let mut cursor=0u64;
    while let Some(row)=rows.next().map_err(db)?{
        let sequence:u64=row.get(0).map_err(db)?;
        let raw:String=row.get(1).map_err(db)?;
        let ack:Option<String>=row.get(2).map_err(db)?;
        if !positive(sequence)||sequence!=cursor+1{return Err("settlement_history_gap")}
        let d:Delivery=decode(raw.as_bytes()).map_err(|_|"settlement_delivery_conflict")?;
        if !d.valid(b)||d.delivery_sequence!=sequence||d.acknowledged_at.is_some()||
            ack.as_deref().is_some_and(|s|!date(s))||
            serde_json::to_string(&d).map_err(|_|"settlement_delivery_conflict")?!=raw{
            return Err("settlement_delivery_conflict")
        }
        cursor=sequence;
    }
    Ok(cursor)
}

// Read-only semantic validation, including rehashed backups built with FKs off.
// Receipt history and its full binding are validated separately before this.
pub(crate) fn validate_ack_journal(c:&Connection)->Result<(),Error>{
    let mut stmt=c.prepare("SELECT a.sequence,a.owner,a.result_id,a.checked_through,r.owner,r.snapshot,
        (SELECT MAX(h.sequence) FROM auction_settlement_receipts h WHERE h.scope=a.scope AND h.auction=a.auction)
        FROM auction_settlement_ack_attempts a LEFT JOIN auction_settlement_receipts r
        ON r.scope=a.scope AND r.auction=a.auction AND r.sequence=a.sequence").map_err(db)?;
    let mut rows=stmt.query([]).map_err(db)?;
    while let Some(row)=rows.next().map_err(db)? {
        let n:u64=row.get(0).map_err(db)?;
        let owner:String=row.get(1).map_err(db)?;
        let result_id:String=row.get(2).map_err(db)?;
        let checked:Option<u64>=row.get(3).map_err(db)?;
        let receipt_owner:Option<String>=row.get(4).map_err(db)?;
        let raw:Option<String>=row.get(5).map_err(db)?;
        let head:Option<u64>=row.get(6).map_err(db)?;
        let d:Delivery=decode(raw.as_deref().ok_or("settlement_ack_journal_invalid")?.as_bytes())
            .map_err(|_|"settlement_ack_journal_invalid")?;
        if !positive(n)||d.delivery_sequence!=n||d.result_id!=result_id||
            receipt_owner.as_deref()!=Some(owner.as_str())||
            checked.is_some_and(|v|v<=n||!positive(v)||head.is_none_or(|h|v>h)){
            return Err("settlement_ack_journal_invalid")
        }
    }
    Ok(())
}

// Read back the exact durable evidence inside the same transaction. A trigger
// or malformed archive may silently ignore or change a write; do not advance
// the cursor or acknowledge a receipt that was not actually stored.
fn verify_write(t:&Transaction<'_>,b:&Binding,n:u64,raw:&str,ack:Option<&str>)->Result<(),Error>{
    let stored:Option<(String,Option<String>)>=t.query_row(
        "SELECT snapshot,acknowledged_at FROM auction_settlement_receipts
         WHERE scope=?1 AND auction=?2 AND sequence=?3 AND tenant=?4 AND association=?5
           AND cloud_work=?6 AND local_work=?7 AND owner=?8",
        params![b.scope,b.auction,n,b.tenant,b.association,b.cloud_work,b.local_work,b.owner],
        |r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db)?;
    match stored {
        Some((actual,stamp)) if actual==raw && stamp.as_deref()==ack => Ok(()),
        _ => Err("settlement_storage_failed"),
    }
}
// Keep immutable snapshot bytes separate from monotonic acknowledgement evidence.
fn store(t:&Transaction<'_>,b:&Binding,mut d:Delivery,cursor:&mut u64)->Result<(),Error>{
    if !d.valid(b){return Err("settlement_invalid_delivery")}
    let n=d.delivery_sequence;
    let ack=d.acknowledged_at.take();
    let raw=serde_json::to_string(&d).map_err(|_|"settlement_invalid_delivery")?;
    if n<=*cursor {
        let old:Option<(String,Option<String>)>=t.query_row(
            "SELECT snapshot,acknowledged_at FROM auction_settlement_receipts WHERE scope=?1 AND auction=?2 AND sequence=?3",
            params![b.scope,b.auction,n],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db)?;
        let (old_raw,old_ack)=old.ok_or("settlement_history_gap")?;
        if old_raw!=raw||matches!((old_ack.as_deref(),ack.as_deref()),(Some(a),Some(c)) if a!=c){
            return Err("settlement_delivery_conflict")
        }
        if old_ack.is_none()&&ack.is_some(){
            let changed=t.execute("UPDATE auction_settlement_receipts SET acknowledged_at=?4 WHERE scope=?1 AND auction=?2 AND sequence=?3",
                params![b.scope,b.auction,n,ack]).map_err(db)?;
            if changed!=1{return Err("settlement_storage_failed")}
            verify_write(t,b,n,&raw,ack.as_deref())?;
        }
    }else{
        if n!=*cursor+1{return Err("settlement_history_gap")}
        let changed=t.execute("INSERT INTO auction_settlement_receipts(scope,auction,tenant,association,cloud_work,local_work,owner,sequence,snapshot,acknowledged_at)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![b.scope,b.auction,b.tenant,b.association,b.cloud_work,b.local_work,b.owner,n,raw,ack]).map_err(db)?;
        if changed!=1{return Err("settlement_storage_failed")}
        verify_write(t,b,n,&raw,ack.as_deref())?;
        *cursor=n;
    }
    Ok(())
}
// Check the current archive and immutable binding before any fixture network I/O.
pub(crate) fn cursor(c:&mut Connection,b:&Binding)->Result<u64,Error>{
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let n=latest(&t,b)?;
    t.commit().map_err(db)?;
    Ok(n)
}
// One explicit read; rewind to the oldest unscanned uncertain attempt,
// even after an app restart or more than two newer deliveries.
pub(crate) fn history_cursor(c:&mut Connection,b:&Binding)->Result<u64,Error>{
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let n=latest(&t,b)?;
    let pending:Option<u64>=t.query_row("SELECT MIN(a.sequence) FROM auction_settlement_ack_attempts a
        JOIN auction_settlement_receipts r USING(scope,auction,sequence)
        WHERE a.scope=?1 AND a.auction=?2 AND a.owner=?3 AND a.checked_through IS NULL
          AND r.acknowledged_at IS NULL",params![b.scope,b.auction,b.owner],|r|r.get(0)).map_err(db)?;
    let after=pending.map_or(n.saturating_sub(2),|p|n.saturating_sub(2).min(p-1));
    t.commit().map_err(db)?;
    Ok(after)
}
pub(crate) fn poll(c:&mut Connection,b:&Binding,bytes:&[u8])->Result<u64,Error>{
    let r:Poll=decode(bytes)?;
    envelope(b,r.ok,&r.artwork_id,&r.association_id)?;
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let mut n=latest(&t,b)?;
    store(&t,b,r.delivery,&mut n)?;
    t.commit().map_err(db)?;
    Ok(n)
}
pub(crate) fn history(c:&mut Connection,b:&Binding,after:u64,bytes:&[u8])->Result<u64,Error>{
    let r:HistoryReply=decode(bytes)?;
    envelope(b,r.ok,&r.artwork_id,&r.association_id)?;
    let h=r.history;
    if !positive(h.latest_sequence)||after>h.latest_sequence{return Err("settlement_invalid_history")}
    let count=50.min(h.latest_sequence-after);
    let end=after+count;
    if h.items.len() as u64!=count||h.next_after_sequence!=(if end<h.latest_sequence{Some(end)}else{None}){
        return Err("settlement_invalid_history")
    }
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let mut n=latest(&t,b)?;
    if after>n{return Err("settlement_history_gap")}
    // An overlapping recovery page must not report a server head behind
    // already committed local evidence. Reject before any receipt changes.
    if h.latest_sequence<n{return Err("settlement_invalid_history")}
    for (i,d) in h.items.into_iter().enumerate(){
        if d.delivery_sequence!=after+i as u64+1{return Err("settlement_invalid_history")}
        store(&t,b,d,&mut n)?;
    }
    // A complete overlapping page was validated and committed with its evidence.
    // An older attempt without a stamp has been scanned, NOT acknowledged.
    // Skip it for future rewind selection so a missing stamp cannot starve
    // forward history. A later authenticated stamp can still be ingested.
    t.execute("UPDATE auction_settlement_ack_attempts SET checked_through=?4
        WHERE scope=?1 AND auction=?2 AND owner=?3 AND checked_through IS NULL
          AND sequence>?5 AND sequence<?4 AND EXISTS(
            SELECT 1 FROM auction_settlement_receipts r WHERE r.scope=auction_settlement_ack_attempts.scope
              AND r.auction=auction_settlement_ack_attempts.auction AND r.sequence=auction_settlement_ack_attempts.sequence
              AND r.acknowledged_at IS NULL)",params![b.scope,b.auction,b.owner,end,after]).map_err(db)?;
    validate_ack_journal(&t)?;
    t.commit().map_err(db)?;
    Ok(n)
}
#[derive(Debug,PartialEq,Eq)]
pub(crate) struct AckTarget{pub(crate) result_id:String,pub(crate) sequence:u64}
fn stored_target(t:&Transaction<'_>,b:&Binding)->Result<AckTarget,Error>{
    let n=latest(t,b)?;
    if n==0{return Err("settlement_not_ingested")}
    let raw:String=t.query_row("SELECT snapshot FROM auction_settlement_receipts WHERE scope=?1 AND auction=?2 AND sequence=?3",
        params![b.scope,b.auction,n],|r|r.get(0)).map_err(db)?;
    let d:Delivery=decode(raw.as_bytes())?;
    if !d.valid(b)||d.delivery_sequence!=n{return Err("settlement_delivery_conflict")}
    Ok(AckTarget{result_id:d.result_id,sequence:n})
}
// Only committed data can propose an ACK; this function does not send anything.
pub(crate) fn ack_target(c:&mut Connection,b:&Binding)->Result<AckTarget,Error>{
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let r=stored_target(&t,b)?;
    t.commit().map_err(db)?;
    Ok(r)
}
// Commit the exact reviewed target BEFORE any network request. On a lost
// response or restart this is evidence of uncertainty, never send authority.
pub(crate) fn begin_ack_attempt(c:&mut Connection,b:&Binding,expected:Option<u64>)->Result<AckTarget,Error>{
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let target=stored_target(&t,b)?;
    if expected.is_some_and(|v|v!=target.sequence){return Err("settlement_stale_ack")}
    let stamp:Option<String>=t.query_row("SELECT acknowledged_at FROM auction_settlement_receipts
        WHERE scope=?1 AND auction=?2 AND sequence=?3",params![b.scope,b.auction,target.sequence],|r|r.get(0)).map_err(db)?;
    if stamp.is_none(){
        let select="SELECT owner,result_id FROM auction_settlement_ack_attempts WHERE scope=?1 AND auction=?2 AND sequence=?3";
        let old:Option<(String,String)>=t.query_row(select,params![b.scope,b.auction,target.sequence],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db)?;
        if old.is_none(){
            let changed=t.execute("INSERT INTO auction_settlement_ack_attempts(scope,auction,sequence,owner,result_id)
                VALUES(?1,?2,?3,?4,?5)",params![b.scope,b.auction,target.sequence,b.owner,target.result_id]).map_err(db)?;
            if changed!=1{return Err("settlement_storage_failed")}
        }
        let actual:(String,String)=t.query_row(select,params![b.scope,b.auction,target.sequence],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db)?;
        if actual!=(b.owner.clone(),target.result_id.clone()){return Err("settlement_ack_journal_invalid")}
    }
    t.commit().map_err(db)?;
    Ok(target)
}
pub(crate) fn ack(c:&mut Connection,b:&Binding,request:&AckTarget,bytes:&[u8])->Result<(),Error>{
    let r:Ack=decode(bytes)?;
    envelope(b,r.ok,&r.artwork_id,&r.association_id)?;
    let _replayed=r.replayed;
    if r.delivery.acknowledged_at.is_none(){return Err("settlement_invalid_ack")}
    let t=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    if stored_target(&t,b)?!=*request||r.delivery.result_id!=request.result_id||r.delivery.delivery_sequence!=request.sequence{
        return Err("settlement_stale_ack")
    }
    let mut n=request.sequence;
    store(&t,b,r.delivery,&mut n)?;
    t.commit().map_err(db)?;
    Ok(())
}

// Read-only projection for the selected, native-resolved binding. No owner,
// archive, cloud identifiers, raw JSON, or token cross the presentation boundary.
// Receipt acknowledgement is independent of fulfillment and never means paid.
#[derive(Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct SettlementSummary {
    pub(crate) sequence:u64,
    pub(crate) status:String,
    pub(crate) amount:u64,
    pub(crate) currency:String,
    pub(crate) fulfilled_at:String,
    pub(crate) acknowledged_at:Option<String>,
    pub(crate) ack_pending:u64,
    pub(crate) ack_scanned:u64,
    pub(crate) ack_confirmed:u64,
}

pub(crate) fn summary(c:&mut Connection,b:&Binding)->Result<Option<SettlementSummary>,Error>{
    let t=c.transaction_with_behavior(TransactionBehavior::Deferred).map_err(db)?;
    let n=latest(&t,b)?;
    if n==0 { t.commit().map_err(db)?; return Ok(None) }
    let (raw,ack):(String,Option<String>)=t.query_row(
        "SELECT snapshot,acknowledged_at FROM auction_settlement_receipts WHERE scope=?1 AND auction=?2 AND sequence=?3",
        params![b.scope,b.auction,n],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db)?;
    let d:Delivery=decode(raw.as_bytes())?;
    if !d.valid(b)||d.delivery_sequence!=n||d.acknowledged_at.is_some()||
       ack.as_deref().is_some_and(|s|!date(s)){return Err("settlement_delivery_conflict")}
    // Counts concern explicit local attempts only, never sales or all remote receipts.
    // The journal and selected binding were validated by latest() in this transaction.
    let (ack_pending,ack_scanned,ack_confirmed):(u64,u64,u64)=t.query_row(
        "SELECT COUNT(CASE WHEN r.acknowledged_at IS NULL AND a.checked_through IS NULL THEN 1 END),
                COUNT(CASE WHEN r.acknowledged_at IS NULL AND a.checked_through IS NOT NULL THEN 1 END),
                COUNT(CASE WHEN r.acknowledged_at IS NOT NULL THEN 1 END)
         FROM auction_settlement_ack_attempts a JOIN auction_settlement_receipts r USING(scope,auction,sequence)
         WHERE a.scope=?1 AND a.auction=?2 AND a.owner=?3",
        params![b.scope,b.auction,b.owner],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(db)?;
    let result=SettlementSummary{sequence:n,status:d.status,amount:d.amount,currency:d.currency,
        fulfilled_at:d.fulfilled_at,acknowledged_at:ack,ack_pending,ack_scanned,ack_confirmed};
    t.commit().map_err(db)?;
    Ok(Some(result))
}

#[cfg(test)]
mod fixtures {
    use super::*;
    
    use serde_json::{json,Value};
    const SCOPE:&str="aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const WORK:&str="bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    pub(crate) fn binding()->Binding{Binding{scope:SCOPE.into(),auction:"fixture-auction".into(),tenant:"fixture-tenant".into(),
        association:"fixture-published-link".into(),cloud_work:"fixture-cloud-work".into(),local_work:WORK.into(),owner:"repertica:1".into()}}
    pub(crate) fn database()->Connection{
        initialize(Connection::open_in_memory().unwrap())
    }
    fn initialize(c:Connection)->Connection{
        c.execute_batch("CREATE TABLE auction_archive_scope(singleton INTEGER PRIMARY KEY,scope TEXT);
            CREATE TABLE artworks(id TEXT PRIMARY KEY,status TEXT);CREATE TABLE sales(id TEXT PRIMARY KEY);").unwrap();
        c.execute("INSERT INTO auction_archive_scope VALUES(1,?1)",[SCOPE]).unwrap();
        c.execute("INSERT INTO artworks VALUES(?1,'available')",[WORK]).unwrap();
        // Deliberately unconstrained fixture so corruption tests can alter
        // historical rows. The real archive test uses the v20 schema.
        c.execute_batch("CREATE TABLE auction_published_bindings(scope TEXT, auction TEXT, tenant TEXT,
            association TEXT, cloud_work TEXT, local_work TEXT, owner TEXT);").unwrap();
        let b=binding();
        c.execute("INSERT INTO auction_published_bindings VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![b.scope,b.auction,b.tenant,b.association,b.cloud_work,b.local_work,b.owner]).unwrap();
        c.execute_batch(SCHEMA).unwrap();
        c.execute_batch(include_str!("auction_settlement_v22.sql")).unwrap();c
    }
    pub(crate) fn delivery(n:u64,status:&str)->Value{json!({"auction_id":"fixture-auction","result_id":"fixture-auction:1:fulfillment",
        "closure_version":1,"delivery_sequence":n,"settlement_revision":2,"runtime_version":3,"fulfillment_revision":2,
        "amount":25000,"currency":"EUR","fulfilled_at":"2026-11-25T04:00:00.000Z","status":status,"acknowledged_at":null})}
    pub(crate) fn reply(d:Value)->Value{json!({"ok":true,"artworkId":"fixture-cloud-work","associationId":"fixture-published-link","delivery":d})}
    pub(crate) fn page(items:Vec<Value>,last:u64,next:Option<u64>)->Value{json!({"ok":true,"artworkId":"fixture-cloud-work","associationId":"fixture-published-link",
        "history":{"latestSequence":last,"items":items,"nextAfterSequence":next}})}
    fn bytes(v:Value)->Vec<u8>{serde_json::to_vec(&v).unwrap()}
    pub(crate) fn count(c:&Connection)->u64{c.query_row("SELECT COUNT(*) FROM auction_settlement_receipts",[],|r|r.get(0)).unwrap()}
    #[test]
    fn replay_gap_and_review_restoration(){
        let mut c=database();let b=binding();
        let d3=delivery(3,"fulfilled");
        assert_eq!(poll(&mut c,&b,&bytes(reply(d3.clone()))),Err("settlement_history_gap"));
        let p=bytes(page(vec![delivery(1,"fulfilled"),delivery(2,"review_required"),d3],3,None));
        assert_eq!(history(&mut c,&b,0,&p),Ok(3));
        assert_eq!(history(&mut c,&b,0,&p),Ok(3));
        assert_eq!(count(&c),3);
        assert_eq!(ack_target(&mut c,&b).unwrap().sequence,3);
        assert_eq!(c.query_row("SELECT COUNT(*) FROM sales",[],|r|r.get::<_,u64>(0)).unwrap(),0);
        assert_eq!(c.query_row("SELECT status FROM artworks WHERE id=?1",[WORK],|r|r.get::<_,String>(0)).unwrap(),"available");
    }
    #[test]
    fn malformed_page_rolls_back_and_binding_cannot_change(){
        let mut c=database();let b=binding();
        let mut bad=delivery(2,"review_required");bad["amount"]=json!(0);
        assert!(history(&mut c,&b,0,&bytes(page(vec![delivery(1,"fulfilled"),bad],2,None))).is_err());
        assert_eq!(count(&c),0);
        let mut wrong=reply(delivery(1,"fulfilled"));wrong["artworkId"]=json!("different");
        assert_eq!(poll(&mut c,&b,&bytes(wrong)),Err("settlement_binding_mismatch"));
        let mut extra=reply(delivery(1,"fulfilled"));extra["delivery"]["winner_email"]=json!("synthetic@example.test");
        assert_eq!(poll(&mut c,&b,&bytes(extra)),Err("settlement_invalid_response"));
        let mut missing=reply(delivery(1,"fulfilled"));missing["delivery"].as_object_mut().unwrap().remove("acknowledged_at");
        assert_eq!(poll(&mut c,&b,&bytes(missing)),Err("settlement_invalid_response"));
        assert_eq!(count(&c),0);
        assert_eq!(poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))),Ok(1));
        let mut changed=delivery(1,"fulfilled");changed["amount"]=json!(99);
        assert_eq!(poll(&mut c,&b,&bytes(reply(changed))),Err("settlement_delivery_conflict"));
        let mut other=b.clone();other.owner="repertica:2".into();
        assert_eq!(poll(&mut c,&other,&bytes(reply(delivery(1,"fulfilled")))),Err("settlement_binding_mismatch"));
        c.execute("UPDATE auction_archive_scope SET scope='other'",[]).unwrap();
        assert_eq!(ack_target(&mut c,&b),Err("settlement_binding_mismatch"));
    }
    #[test]
    fn only_committed_latest_can_ack_and_replay_does_not_erase_evidence(){
        let mut c=database();let b=binding();
        assert_eq!(ack_target(&mut c,&b),Err("settlement_not_ingested"));
        poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))).unwrap();
        let old=ack_target(&mut c,&b).unwrap();
        let d2=delivery(2,"review_required");
        poll(&mut c,&b,&bytes(reply(d2.clone()))).unwrap();
        let mut d1=delivery(1,"fulfilled");d1["acknowledged_at"]=json!("2026-11-25T04:01:00.000Z");
        let mut stale=reply(d1);stale["replayed"]=json!(false);
        assert_eq!(ack(&mut c,&b,&old,&bytes(stale)),Err("settlement_stale_ack"));
        let target=ack_target(&mut c,&b).unwrap();
        let mut missing=reply(d2.clone());missing["replayed"]=json!(false);
        assert_eq!(ack(&mut c,&b,&target,&bytes(missing)),Err("settlement_invalid_ack"));
        let mut confirmed=d2.clone();confirmed["acknowledged_at"]=json!("2026-11-25T04:02:00.000Z");
        let mut response=reply(confirmed.clone());response["replayed"]=json!(false);
        assert_eq!(ack(&mut c,&b,&target,&bytes(response.clone())),Ok(()));
        response["replayed"]=json!(true);
        assert_eq!(ack(&mut c,&b,&target,&bytes(response)),Ok(()));
        poll(&mut c,&b,&bytes(reply(d2))).unwrap();
        let stamp:String=c.query_row("SELECT acknowledged_at FROM auction_settlement_receipts WHERE sequence=2",[],|r|r.get(0)).unwrap();
        assert_eq!(stamp,"2026-11-25T04:02:00.000Z");
        confirmed["acknowledged_at"]=json!("2026-11-25T04:03:00.000Z");
        let mut conflict=reply(confirmed);conflict["replayed"]=json!(true);
        assert_eq!(ack(&mut c,&b,&target,&bytes(conflict)),Err("settlement_delivery_conflict"));
        assert_eq!(count(&c),2);
    }
    #[test]
    fn cursor_and_numeric_limits_fail_closed(){
        let mut c=database();let b=binding();
        assert_eq!(history(&mut c,&b,0,&bytes(page(vec![delivery(1,"fulfilled")],2,None))),Err("settlement_invalid_history"));
        assert_eq!(history(&mut c,&b,1,&bytes(page(vec![delivery(2,"fulfilled")],2,None))),Err("settlement_history_gap"));
        let mut big=delivery(1,"fulfilled");big["amount"]=json!(9_007_199_254_740_992u64);
        assert_eq!(poll(&mut c,&b,&bytes(reply(big))),Err("settlement_invalid_delivery"));
        assert_eq!(poll(&mut c,&b,&vec![b' ';65537]),Err("settlement_response_too_large"));
        assert_eq!(count(&c),0);
    }
    
    #[test]
    fn summary_survives_reopen_and_never_marks_sale_or_payment(){
        let dir=tempfile::tempdir().unwrap();
        let path=dir.path().join("synthetic.sqlite");
        let b=binding();
        let mut c=initialize(Connection::open(&path).unwrap());
        assert_eq!(summary(&mut c,&b),Ok(None));
        poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))).unwrap();
        poll(&mut c,&b,&bytes(reply(delivery(2,"review_required")))).unwrap();
        let target=ack_target(&mut c,&b).unwrap();
        let mut d=delivery(2,"review_required");
        d["acknowledged_at"]=json!("2026-11-25T04:02:00.000Z");
        let mut a=reply(d);a["replayed"]=json!(false);
        ack(&mut c,&b,&target,&bytes(a)).unwrap();
        drop(c);
        let mut c=Connection::open(&path).unwrap();
        let view=summary(&mut c,&b).unwrap().unwrap();
        assert_eq!(serde_json::to_value(view).unwrap(),json!({
            "sequence":2,"status":"review_required","amount":25000,"currency":"EUR",
            "fulfilledAt":"2026-11-25T04:00:00.000Z","acknowledgedAt":"2026-11-25T04:02:00.000Z",
            "ackPending":0,"ackScanned":0,"ackConfirmed":0
        }));
        let mut wrong=b.clone();wrong.owner="repertica:2".into();
        assert_eq!(summary(&mut c,&wrong),Err("settlement_binding_mismatch"));
        assert_eq!(count(&c),2);
        assert_eq!(c.query_row("SELECT COUNT(*) FROM sales",[],|r|r.get::<_,u64>(0)).unwrap(),0);
        assert_eq!(c.query_row("SELECT status FROM artworks WHERE id=?1",[WORK],|r|r.get::<_,String>(0)).unwrap(),"available");
        c.execute("DELETE FROM auction_settlement_receipts WHERE sequence=1",[]).unwrap();
        assert_eq!(summary(&mut c,&b),Err("settlement_history_gap"));
    }
    #[test]
    fn damaged_earlier_receipt_blocks_every_cursor_and_ack(){
        for (mutation,expected) in [
            ("DELETE FROM auction_settlement_receipts WHERE sequence=1","settlement_history_gap"),
            ("UPDATE auction_settlement_receipts SET snapshot='{}' WHERE sequence=1","settlement_delivery_conflict"),
            ("UPDATE auction_settlement_receipts SET acknowledged_at='invalid' WHERE sequence=1","settlement_delivery_conflict"),
        ] {
            let mut c=database();let b=binding();
            history(&mut c,&b,0,&bytes(page(vec![delivery(1,"fulfilled"),delivery(2,"review_required")],2,None))).unwrap();
            c.execute(mutation,[]).unwrap();
            assert_eq!(cursor(&mut c,&b),Err(expected));
            assert_eq!(ack_target(&mut c,&b),Err(expected));
            assert_eq!(summary(&mut c,&b),Err(expected));
            assert_eq!(poll(&mut c,&b,&bytes(reply(delivery(3,"fulfilled")))),Err(expected));
        }
    }
    #[test]
    fn ignored_or_altered_insert_and_ack_are_not_success(){
        for trigger in [
            "CREATE TRIGGER broken BEFORE INSERT ON auction_settlement_receipts BEGIN SELECT RAISE(IGNORE); END",
            "CREATE TRIGGER broken AFTER INSERT ON auction_settlement_receipts BEGIN UPDATE auction_settlement_receipts SET snapshot='{}' WHERE sequence=NEW.sequence; END",
        ] {
            let mut c=database();let b=binding();
            c.execute_batch(trigger).unwrap();
            assert_eq!(poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))),Err("settlement_storage_failed"));
            assert_eq!(count(&c),0);
            assert_eq!(ack_target(&mut c,&b),Err("settlement_not_ingested"));
        }
        for trigger in [
            "CREATE TRIGGER broken BEFORE UPDATE ON auction_settlement_receipts BEGIN SELECT RAISE(IGNORE); END",
            "CREATE TRIGGER broken AFTER UPDATE ON auction_settlement_receipts WHEN NEW.acknowledged_at IS NOT NULL AND NEW.acknowledged_at<>'2026-11-25T05:00:00.000Z' BEGIN UPDATE auction_settlement_receipts SET acknowledged_at='2026-11-25T05:00:00.000Z' WHERE sequence=NEW.sequence; END",
        ] {
            let mut c=database();let b=binding();
            poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))).unwrap();
            let target=ack_target(&mut c,&b).unwrap();
            c.execute_batch(trigger).unwrap();
            let mut d=delivery(1,"fulfilled");d["acknowledged_at"]=json!("2026-11-25T04:02:00.000Z");
            let mut response=reply(d);response["replayed"]=json!(false);
            assert_eq!(ack(&mut c,&b,&target,&bytes(response)),Err("settlement_storage_failed"));
            assert_eq!(summary(&mut c,&b).unwrap().unwrap().acknowledged_at,None);
        }
    }
    #[test]
    fn durable_attempt_recovers_older_stamp_and_unconfirmed_scan_does_not_starve(){
        let dir=tempfile::tempdir().unwrap();
        let path=dir.path().join("ack-journal.sqlite");
        let b=binding();
        let mut c=initialize(Connection::open(&path).unwrap());
        poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))).unwrap();
        assert_eq!(begin_ack_attempt(&mut c,&b,Some(2)),Err("settlement_stale_ack"));
        let mut wrong=b.clone();wrong.owner="repertica:2".into();
        assert_eq!(begin_ack_attempt(&mut c,&wrong,Some(1)),Err("settlement_binding_mismatch"));
        let target=begin_ack_attempt(&mut c,&b,Some(1)).unwrap();
        assert_eq!(begin_ack_attempt(&mut c,&b,Some(1)).unwrap(),target);
        for n in 2..=5 {poll(&mut c,&b,&bytes(reply(delivery(n,"review_required")))).unwrap();}
        begin_ack_attempt(&mut c,&b,Some(5)).unwrap();
        for n in 6..=9 {poll(&mut c,&b,&bytes(reply(delivery(n,"review_required")))).unwrap();}
        drop(c);
        let mut c=Connection::open(&path).unwrap();
        let before=summary(&mut c,&b).unwrap().unwrap();
        assert_eq!((before.ack_pending,before.ack_scanned,before.ack_confirmed),(2,0,0));
        assert_eq!(history_cursor(&mut c,&b),Ok(0));
        let mut first=delivery(1,"fulfilled");
        first["acknowledged_at"]=json!("2026-11-25T04:02:00.000Z");
        let mut items=vec![first];
        for n in 2..=9 {items.push(delivery(n,"review_required"));}
        assert_eq!(history(&mut c,&b,0,&bytes(page(items,9,None))),Ok(9));
        let after=summary(&mut c,&b).unwrap().unwrap();
        assert_eq!((after.ack_pending,after.ack_scanned,after.ack_confirmed),(0,1,1));
        assert_eq!(after.acknowledged_at,None);
        assert_eq!(history_cursor(&mut c,&b),Ok(7));
        let (stamp,checked):(Option<String>,Option<u64>)=c.query_row(
            "SELECT r.acknowledged_at,a.checked_through FROM auction_settlement_ack_attempts a
             JOIN auction_settlement_receipts r USING(scope,auction,sequence) WHERE a.sequence=5",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!((stamp,checked),(None,Some(9)));
        assert_eq!(c.query_row("SELECT acknowledged_at FROM auction_settlement_receipts WHERE sequence=1",[],|r|r.get::<_,String>(0)).unwrap(),"2026-11-25T04:02:00.000Z");
        assert_eq!(c.query_row("SELECT COUNT(*) FROM sales",[],|r|r.get::<_,u64>(0)).unwrap(),0);
    }
    #[test]
    fn attempt_write_failure_prevents_request_and_snapshot_corruption_fails_closed(){
        let mut c=database();let b=binding();
        poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))).unwrap();
        c.execute_batch("CREATE TRIGGER ignore_attempt BEFORE INSERT ON auction_settlement_ack_attempts BEGIN SELECT RAISE(IGNORE); END").unwrap();
        assert_eq!(begin_ack_attempt(&mut c,&b,Some(1)),Err("settlement_storage_failed"));
        assert_eq!(c.query_row("SELECT COUNT(*) FROM auction_settlement_ack_attempts",[],|r|r.get::<_,u64>(0)).unwrap(),0);
        c.execute_batch("DROP TRIGGER ignore_attempt").unwrap();
        begin_ack_attempt(&mut c,&b,Some(1)).unwrap();
        c.execute_batch("DROP TRIGGER auction_ack_attempt_checked_only; UPDATE auction_settlement_ack_attempts SET owner='repertica:2'").unwrap();
        assert_eq!(history_cursor(&mut c,&b),Err("settlement_ack_journal_invalid"));
    }
    #[test]
    fn summary_rejects_corrupt_ack_and_archive_switch(){
        let mut c=database();let b=binding();
        poll(&mut c,&b,&bytes(reply(delivery(1,"fulfilled")))).unwrap();
        assert_eq!(summary(&mut c,&b).unwrap().unwrap().acknowledged_at,None);
        c.execute("UPDATE auction_settlement_receipts SET acknowledged_at='invalid'",[]).unwrap();
        assert_eq!(summary(&mut c,&b),Err("settlement_delivery_conflict"));
        c.execute("UPDATE auction_archive_scope SET scope='different'",[]).unwrap();
        assert_eq!(summary(&mut c,&b),Err("settlement_binding_mismatch"));
    }
    
}
#[cfg(test)]
pub(crate) use fixtures::{binding,database,delivery,reply,page,count};
