//! Shared validation and test-only receipt reconciliation for one local preview.
//! A receipt is evidence of a non-binding preview only, never a sale or a payment.
use serde::{Deserialize, Serialize};
use super::preview_payload::{bounded, editor_trim, parse_canonical_payload, valid_token, valid_slug};

pub(super) fn invalid() -> String { "invalid_stored_draft".into() }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StoredReceipt {
    pub(super) tenant_id: String,
    pub(super) idempotency_key: String,
    pub(super) auction_id: String,
    pub(super) slug: String,
}

fn validate_receipt(receipt: &StoredReceipt, pending: &Pending) -> Result<(), String> {
    let payload = parse_canonical_payload(&pending.body).map_err(|_| invalid())?;
    let (artist, work) = receipt.slug.split_once("--").ok_or_else(invalid)?;
    if receipt.tenant_id != payload.tenant_id || receipt.idempotency_key != pending.idempotency_key ||
        !valid_slug(artist) || work != payload.slug ||
        !uuid::Uuid::parse_str(&receipt.auction_id).is_ok_and(|id| id.to_string() == receipt.auction_id) {
        return Err(invalid());
    }
    Ok(())
}

// Option normally accepts an absent field as None in serde. The editor's
// stored format requires explicit null for nullable fields, not omission.
fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Draft {
    pub(super) artwork_id: String, pub(super) title: String, description: String, slug: String,
    starts_at: String, ends_at: String,
    #[serde(deserialize_with = "required_nullable")]
    starting_amount: Option<f64>,
    #[serde(deserialize_with = "required_nullable")]
    minimum_increment_amount: Option<f64>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Pending {
    pub(super) artwork_id: String, pub(super) idempotency_key: String, pub(super) body: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    version: u8, scope: String, pub(super) draft: Draft,
    #[serde(deserialize_with = "required_nullable")]
    pub(super) pending: Option<Pending>,
    #[serde(default)]
    pub(super) receipt: Option<StoredReceipt>,
}

pub(super) fn pending_matches_draft(p: &Pending, d: &Draft) -> Result<bool, String> {
    let b = parse_canonical_payload(&p.body).map_err(|_| invalid())?;
    Ok(b.title == editor_trim(&d.title) && b.description == editor_trim(&d.description) &&
        b.slug == d.slug && b.starts_at == d.starts_at && b.ends_at == d.ends_at &&
        d.starting_amount == Some(b.starting_amount as f64) &&
        d.minimum_increment_amount == Some(b.minimum_increment_amount as f64))
}

pub(super) fn parse_record(raw: &str, scope: &str, artwork_id: &str) -> Result<Record, String> {
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
        parse_canonical_payload(&p.body).map_err(|_| invalid())?;
    }
    if let Some(receipt) = &record.receipt {
        let pending = record.pending.as_ref().ok_or_else(invalid)?;
        validate_receipt(receipt, pending)?;
    }
    // Keep the exact same public-field order and compact JSON as preparePreview.
    // A native write must not persist a body the editor cannot reopen.
    // Do not compare stored schedule with today's clock: reconcile old attempts.
    Ok(record)
}


// Only built into test targets until the isolated provider and Worker are accepted.
// The renderer never supplies or writes a receipt. Exact scope, revision and body
// are captured BEFORE HTTP; a restore or edit while in flight must fail closed.
#[cfg(test)]
pub(super) mod reconciliation {
    use super::*;
    use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

    pub(in super::super) struct Attempt {
        artwork_id: String,
        scope: String,
        revision: String,
        raw: String,
        pub(in super::super) pending: Pending,
        pub(in super::super) receipt: Option<StoredReceipt>,
    }

    pub(in super::super) fn load(conn: &Connection, artwork_id: &str, scope: &str, revision: &str)
        -> Result<Attempt, String> {
        let conflict = || "auction_draft_conflict".to_string();
        let db_scope: String = conn.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1",
            [], |r| r.get(0)).map_err(|_| conflict())?;
        if db_scope != scope || uuid::Uuid::parse_str(scope).is_err() ||
            uuid::Uuid::parse_str(revision).is_err() { return Err(conflict()); }
        let raw: String = conn.query_row(
            "SELECT record_json FROM auction_drafts WHERE artwork_id=?1 AND revision=?2",
            params![artwork_id,revision], |r| r.get(0)).optional()
            .map_err(|_| conflict())?.ok_or_else(conflict)?;
        let record = parse_record(&raw, scope, artwork_id)?;
        let pending = record.pending.ok_or_else(|| "auction_pending_required".to_string())?;
        Ok(Attempt { artwork_id: artwork_id.into(), scope: scope.into(), revision: revision.into(),
            raw, pending, receipt: record.receipt })
    }

    pub(in super::super) fn persist(conn: &mut Connection, attempt: &Attempt, receipt: &StoredReceipt)
        -> Result<StoredReceipt, String> {
        let failed = || "auction_receipt_write_failed".to_string();
        validate_receipt(receipt, &attempt.pending)?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_| failed())?;
        let current = load(&tx, &attempt.artwork_id, &attempt.scope, &attempt.revision)?;
        if current.raw != attempt.raw { return Err("auction_draft_conflict".into()); }
        if let Some(existing) = current.receipt {
            if existing != *receipt { return Err("auction_receipt_conflict".into()); }
            return Ok(existing);
        }
        let mut value: serde_json::Value = serde_json::from_str(&current.raw).map_err(|_| invalid())?;
        value["receipt"] = serde_json::to_value(receipt).map_err(|_| invalid())?;
        let raw = serde_json::to_string(&value).map_err(|_| invalid())?;
        parse_record(&raw, &attempt.scope, &attempt.artwork_id)?;
        let revision = uuid::Uuid::new_v4().to_string();
        let count = tx.execute("UPDATE auction_drafts SET revision=?1,record_json=?2 WHERE artwork_id=?3 AND revision=?4 AND record_json=?5",
            params![revision,raw,attempt.artwork_id,attempt.revision,attempt.raw]).map_err(|_| failed())?;
        if count != 1 { return Err(failed()); }
        let saved = load(&tx, &attempt.artwork_id, &attempt.scope, &revision)?;
        if saved.raw != raw || saved.receipt.as_ref() != Some(receipt) { return Err(failed()); }
        tx.commit().map_err(|_| failed())?;
        Ok(receipt.clone())
    }
}
