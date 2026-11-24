//! Local-only draft persistence. No network, tokens or auction publication.
use crate::db::Database;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

pub(crate) fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch("CREATE TABLE IF NOT EXISTS auction_archive_scope (
        singleton INTEGER PRIMARY KEY CHECK(singleton=1),
        scope TEXT NOT NULL CHECK(length(scope)=36)
    );
    CREATE TABLE IF NOT EXISTS auction_drafts (
        artwork_id TEXT PRIMARY KEY NOT NULL,
        revision TEXT NOT NULL CHECK(length(revision)=36),
        record_json TEXT NOT NULL CHECK(json_valid(record_json) AND length(CAST(record_json AS BLOB))<=64000)
    );")?;
    tx.execute("INSERT OR IGNORE INTO auction_archive_scope(singleton,scope) VALUES(1,?1)",
        [Uuid::new_v4().to_string()])?;
    tx.pragma_update(None, "user_version", 19)?;
    tx.commit()
}

fn error(e: impl std::fmt::Display) -> String { crate::util::friendly(e) }
fn invalid() -> String { "invalid_stored_draft".into() }
fn bounded(s: &str, max: usize) -> bool { s.encode_utf16().count() <= max }
fn valid_id(s: &str) -> bool { !s.is_empty() && bounded(s, 256) && !s.chars().any(char::is_control) }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Draft {
    artwork_id: String, title: String, description: String, slug: String,
    starts_at: String, ends_at: String,
    starting_amount: Option<f64>, minimum_increment_amount: Option<f64>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pending {
    artwork_id: String, idempotency_key: String, body: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Payload {
    tenant_id: String, slug: String, title: String, description: String,
    starts_at: String, ends_at: String, currency: String,
    starting_amount: i64, minimum_increment_amount: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u8, scope: String, draft: Draft, pending: Option<Pending>,
}

fn valid_token(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

// Match the editor's ECMAScript trim semantics (not Rust's Unicode
// whitespace set), so a reviewed draft and its pending payload agree.
fn editor_trim(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, '\u{0009}'..='\u{000d}' | '\u{0020}' |
        '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' |
        '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'))
}

fn valid_slug(s: &str) -> bool {
    !s.is_empty() && s.len() <= 60 && s.split('-').all(|part|
        !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

fn utc_millis(s: &str) -> Option<i64> {
    // Require the same exact millisecond UTC form as the editor. Reject
    // leap seconds, which chrono can represent but JavaScript Date cannot.
    let bytes = s.as_bytes();
    if bytes.len() != 24 || bytes.get(17..19)? > &b"59"[..] ||
        !bytes[..4].iter().all(u8::is_ascii_digit) { return None; }
    let parsed = chrono::DateTime::parse_from_rfc3339(s).ok()?;
    if parsed.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true) != s {
        return None;
    }
    Some(parsed.timestamp_millis())
}

fn valid_pending_payload(b: &Payload) -> bool {
    let Some(start) = utc_millis(&b.starts_at) else { return false };
    let Some(end) = utc_millis(&b.ends_at) else { return false };
    valid_token(&b.tenant_id,3,80) && b.currency == "EUR" &&
        (1..=1_000_000_000).contains(&b.starting_amount) &&
        (1..=1_000_000_000).contains(&b.minimum_increment_amount) &&
        b.title == editor_trim(&b.title) && b.title.encode_utf16().count() >= 3 &&
        bounded(&b.title,160) && b.description == editor_trim(&b.description) &&
        bounded(&b.description,1500) && valid_slug(&b.slug) &&
        end > start && end - start <= 7 * 86_400_000
}

fn pending_matches_draft(p: &Pending, d: &Draft) -> Result<bool, String> {
    let b: Payload = serde_json::from_str(&p.body).map_err(|_| invalid())?;
    Ok(b.title == editor_trim(&d.title) && b.description == editor_trim(&d.description) &&
        b.slug == d.slug && b.starts_at == d.starts_at && b.ends_at == d.ends_at &&
        d.starting_amount == Some(b.starting_amount as f64) &&
        d.minimum_increment_amount == Some(b.minimum_increment_amount as f64))
}

fn parse_record(raw: &str, scope: &str, artwork_id: &str) -> Result<Record, String> {
    if raw.len() > 64000 { return Err(invalid()); }
    let record: Record = serde_json::from_str(raw).map_err(|_| invalid())?;
    let d = &record.draft;
    if record.version != 1 || record.scope != scope || d.artwork_id != artwork_id ||
        !bounded(&d.title,160) || !bounded(&d.description,1500) || !bounded(&d.slug,60) ||
        !bounded(&d.starts_at,64) || !bounded(&d.ends_at,64) ||
        [d.starting_amount,d.minimum_increment_amount].into_iter().flatten().any(|n| !n.is_finite()) {
        return Err(invalid());
    }
    if let Some(p) = &record.pending {
        if p.artwork_id != artwork_id || !valid_token(&p.idempotency_key,16,96) || p.body.len()>8192 {
            return Err(invalid());
        }
        let b: Payload = serde_json::from_str(&p.body).map_err(|_| invalid())?;
        if !valid_pending_payload(&b) { return Err(invalid()); }
    }
    // Native storage checks payload semantics and field allowlists. Exact
    // serialized body canonicality remains the editor's responsibility.
    // Do not compare stored schedule with today's clock: reconcile old attempts.
    Ok(record)
}

fn scope(conn: &Connection) -> Result<String,String> {
    let value: String = conn.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1",
        [], |r|r.get(0)).map_err(error)?;
    Uuid::parse_str(&value).map_err(|_| invalid())?;
    Ok(value)
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DraftSnapshot {
    artwork_id: String, scope: String, revision: Option<String>, record_json: Option<String>,
}

fn read(conn: &Connection, artwork_id: &str) -> Result<DraftSnapshot,String> {
    if !valid_id(artwork_id) { return Err(invalid()); }
    let scope = scope(conn)?;
    let row: Option<(String,String)> = conn.query_row(
        "SELECT revision,record_json FROM auction_drafts WHERE artwork_id=?1", [artwork_id],
        |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(error)?;
    let (revision,record_json) = match row {
        Some((revision,raw)) => {
            Uuid::parse_str(&revision).map_err(|_| invalid())?;
            parse_record(&raw,&scope,artwork_id)?;
            (Some(revision),Some(raw))
        },
        None => (None,None),
    };
    Ok(DraftSnapshot {artwork_id:artwork_id.into(),scope,revision,record_json})
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DraftWrite {
    artwork_id: String, scope: String, expected_revision: Option<String>, record_json: String,
}

fn write(conn: &mut Connection, input: DraftWrite) -> Result<DraftSnapshot,String> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(error)?;
    let previous = read(&tx,&input.artwork_id)?;
    if previous.scope != input.scope || previous.revision != input.expected_revision {
        return Err("auction_draft_conflict".into());
    }
    let next = parse_record(&input.record_json,&input.scope,&input.artwork_id)?;
    let old = previous.record_json.as_deref().map(|raw|parse_record(raw,&input.scope,&input.artwork_id)).transpose()?;
    let old_pending = old.as_ref().and_then(|r|r.pending.as_ref());
    if old_pending.is_some() && old_pending != next.pending.as_ref() {
        return Err("pending_preview_unresolved".into());
    }
    // Bind a first pending request to the reviewed draft at the native IPC
    // boundary too. Later edits may differ; the original pending body is frozen.
    if old_pending.is_none() {
        if let Some(p) = &next.pending {
            if !pending_matches_draft(p, &next.draft)? {
                return Err("preview_draft_changed".into());
            }
        }
    }
    // A new draft or attempt must still refer to an available local work.
    // Retain existing drafts/pending requests even if the artwork is later sold/deleted.
    if old.is_none() || (old_pending.is_none() && next.pending.is_some()) {
        let status: Option<String> = tx.query_row("SELECT status FROM artworks WHERE id=?1",
            [&input.artwork_id],|r|r.get(0)).optional().map_err(error)?;
        if status.as_deref() != Some("available") { return Err("artwork_not_available".into()); }
    }
    let revision = Uuid::new_v4().to_string();
    tx.execute("INSERT INTO auction_drafts(artwork_id,revision,record_json) VALUES(?1,?2,?3)
        ON CONFLICT(artwork_id) DO UPDATE SET revision=excluded.revision,record_json=excluded.record_json",
        params![input.artwork_id,revision,input.record_json]).map_err(error)?;
    let saved = read(&tx,&input.artwork_id)?;
    if saved.scope != input.scope || saved.revision.as_deref() != Some(revision.as_str()) ||
        saved.record_json.as_deref() != Some(input.record_json.as_str()) {
        return Err("auction_draft_write_failed".into());
    }
    tx.commit().map_err(error)?;
    Ok(saved)
}

#[tauri::command]
pub fn read_auction_draft(db: State<'_,Database>, artwork_id: String) -> Result<DraftSnapshot,String> {
    let conn = db.conn.lock().map_err(error)?;
    read(&conn,&artwork_id)
}

#[tauri::command]
pub fn write_auction_draft(db: State<'_,Database>, input: DraftWrite) -> Result<DraftSnapshot,String> {
    let mut conn = db.conn.lock().map_err(error)?;
    write(&mut conn,input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn seed(conn: &Connection) {
        conn.execute("INSERT INTO artworks(id,code,title,created_at,updated_at) VALUES('a','A','Art','t','t')",[]).unwrap();
    }
    fn record(scope: &str, pending: bool) -> String {
        let body = json!({"tenantId":"tenant-test","slug":"art","title":"Art","description":"",
            "startsAt":"2027-01-02T10:00:00.000Z","endsAt":"2027-01-03T10:00:00.000Z",
            "startingAmount":1000,"minimumIncrementAmount":100,"currency":"EUR"}).to_string();
        json!({"version":1,"scope":scope,"draft":{"artworkId":"a","title":"Art","description":"",
            "slug":"art","startsAt":"2027-01-02T10:00:00.000Z","endsAt":"2027-01-03T10:00:00.000Z",
            "startingAmount":1000,"minimumIncrementAmount":100},
            "pending":if pending {json!({"artworkId":"a","idempotencyKey":"preview-0123456789","body":body})}
                else {serde_json::Value::Null}}).to_string()
    }
    fn input(snapshot: &DraftSnapshot, raw: String) -> DraftWrite {
        DraftWrite {artwork_id:"a".into(),scope:snapshot.scope.clone(),
            expected_revision:snapshot.revision.clone(),record_json:raw}
    }

    #[test]
    fn stale_writes_and_pending_removal_are_rejected() {
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().into()).unwrap();
        let mut conn=db.conn.lock().unwrap();
        seed(&conn);
        let initial=read(&conn,"a").unwrap();
        let saved=write(&mut conn,input(&initial,record(&initial.scope,true))).unwrap();
        assert_eq!(write(&mut conn,input(&initial,record(&initial.scope,false))).unwrap_err(),"auction_draft_conflict");
        assert_eq!(write(&mut conn,input(&saved,record(&saved.scope,false))).unwrap_err(),"pending_preview_unresolved");
        let mut bad: serde_json::Value=serde_json::from_str(saved.record_json.as_ref().unwrap()).unwrap();
        bad["draft"]["accessToken"]=json!("synthetic");
        assert!(write(&mut conn,input(&saved,bad.to_string())).is_err());
        assert_eq!(read(&conn,"a").unwrap(),saved);
        conn.execute("DELETE FROM artworks WHERE id='a'",[]).unwrap();
        assert_eq!(read(&conn,"a").unwrap(),saved);
    }

    #[test]
    fn native_rejects_invalid_or_unreviewed_pending_payloads_atomically() {
        let dir=tempfile::tempdir().unwrap();
        let db=crate::db::open(dir.path().into()).unwrap();
        let mut conn=db.conn.lock().unwrap();
        seed(&conn);
        let initial=read(&conn,"a").unwrap();
        let draft=write(&mut conn,input(&initial,record(&initial.scope,false))).unwrap();
        let candidate: serde_json::Value=serde_json::from_str(&record(&draft.scope,true)).unwrap();
        for (field,value,expected) in [
            ("slug",json!("Bad--slug"),"invalid_stored_draft"),
            ("title",json!("  Art"),"invalid_stored_draft"),
            ("title",json!("Other title"),"preview_draft_changed"),
            ("startingAmount",json!(2000),"preview_draft_changed"),
            ("endsAt",json!("2027-01-10T10:00:00.000Z"),"invalid_stored_draft"),
            ("startsAt",json!("2027-01-02T10:00:60.000Z"),"invalid_stored_draft"),
            ("startsAt",json!("2027-01-02T10:00:00+00:00"),"invalid_stored_draft"),
        ] {
            let mut bad=candidate.clone();
            let mut body: serde_json::Value=serde_json::from_str(bad["pending"]["body"].as_str().unwrap()).unwrap();
            body[field]=value;
            bad["pending"]["body"]=json!(body.to_string());
            assert_eq!(write(&mut conn,input(&draft,bad.to_string())).unwrap_err(),expected);
            assert_eq!(read(&conn,"a").unwrap(),draft);
        }
        let mut reviewed=candidate;
        reviewed["draft"]["title"]=json!("\u{feff}Art ");
        let saved=write(&mut conn,input(&draft,reviewed.to_string())).unwrap();
        // The original reviewed body must survive subsequent editor changes.
        reviewed["draft"]["title"]=json!("Edited title");
        let edited=write(&mut conn,input(&saved,reviewed.to_string())).unwrap();
        let persisted: serde_json::Value=serde_json::from_str(edited.record_json.as_ref().unwrap()).unwrap();
        assert_eq!(persisted["pending"],reviewed["pending"]);
    }

    #[test]
    fn drafts_survive_reopen_and_verified_backup_restore() {
        let dir=tempfile::tempdir().unwrap();
        let live=dir.path().join("live");
        let dest=dir.path().join("backups");
        std::fs::create_dir(&dest).unwrap();
        let db=crate::db::open(live.clone()).unwrap();
        let saved={
            let mut conn=db.conn.lock().unwrap();
            seed(&conn);
            let initial=read(&conn,"a").unwrap();
            write(&mut conn,input(&initial,record(&initial.scope,true))).unwrap()
        };
        drop(db);
        let db=crate::db::open(live).unwrap();
        assert_eq!(read(&db.conn.lock().unwrap(),"a").unwrap(),saved);
        let backup=crate::backup::create_backup_core(&db,&dest).unwrap();
        {
            let mut conn=db.conn.lock().unwrap();
            let changed=write(&mut conn,input(&saved,record(&saved.scope,true))).unwrap();
            assert_ne!(changed.revision,saved.revision);
        }
        crate::backup::restore_backup_core(&db,std::path::Path::new(&backup.directory)).unwrap();
        assert_eq!(read(&db.conn.lock().unwrap(),"a").unwrap(),saved);
    }
}
