//! Isolated loopback settlement transport; compiled only by the test target.
use super::*;
use super::settlement_ledger::{self as ledger, Binding};
use rusqlite::Connection;
use serde_json::{json, Value};

const PATH: &str = "/api/studio/settlement/delivery";

pub(super) struct NativeSettlementHttp {
    addr: SocketAddrV4,
    client: Client,
}

impl NativeSettlementHttp {
    pub(super) fn new(addr: SocketAddrV4) -> Result<Self, Failure> {
        allowed(addr)?;
        Ok(Self { addr, client: isolated_client()? })
    }

    fn authorize<'a>(&self, token: &'a NativeAccessToken, binding: &Binding,
        clock: &impl Fn() -> Result<u64, Failure>) -> Result<&'a str, Failure> {
        let bearer = token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
        if token.subject != binding.owner { return Err("settlement_binding_mismatch"); }
        allowed(self.addr)?;
        Ok(bearer)
    }

    async fn request(&self, token: &NativeAccessToken, binding: &Binding,
        body: Value, clock: &impl Fn() -> Result<u64, Failure>) -> Result<Vec<u8>, Failure> {
        let deadline = Instant::now() + DEADLINE;
        let bearer = self.authorize(token, binding, clock)?;
        let bytes = serde_json::to_vec(&body).map_err(|_| "settlement_invalid_request")?;
        if bytes.len() > 4096 { return Err("settlement_invalid_request"); }
        let response = self.client.post(format!("http://{}{PATH}", self.addr))
            .bearer_auth(bearer).header(header::CONTENT_TYPE, "application/json")
            .body(bytes).timeout(remaining(deadline)?).send().await.map_err(http_error)?;
        match response.status() {
            StatusCode::OK => (),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => return Err("settlement_access_denied"),
            StatusCode::NOT_FOUND => return Err("settlement_not_found"),
            StatusCode::CONFLICT => return Err("settlement_conflict"),
            StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => return Err("settlement_unavailable"),
            _ => return Err("settlement_http_rejected"),
        }
        // Keep the exact bytes: normalizing JSON before strict deserialization
        // would discard duplicate fields. Never parse a server error body.
        let bytes = read_json_bytes(response, deadline).await?;
        self.authorize(token, binding, clock)?;
        Ok(bytes)
    }

    pub(super) async fn poll(&self, c: &mut Connection, b: &Binding,
        token: &NativeAccessToken, clock: impl Fn() -> Result<u64, Failure>) -> Result<u64, Failure> {
        self.authorize(token, b, &clock)?;
        ledger::cursor(c, b)?;
        let bytes = self.request(token, b, json!({"action":"poll","auctionId":b.auction,
            "tenantId":b.tenant}), &clock).await?;
        ledger::poll(c, b, &bytes)
    }

    pub(super) async fn history(&self, c: &mut Connection, b: &Binding,
        token: &NativeAccessToken, clock: impl Fn() -> Result<u64, Failure>) -> Result<u64, Failure> {
        self.authorize(token, b, &clock)?;
        // Native durable journal selects the recovery cursor. No renderer
        // cursor, automatic ACK or retry; a single page per explicit action.
        let after = ledger::history_cursor(c, b)?;
        let bytes = self.request(token, b, json!({"action":"history","auctionId":b.auction,
            "tenantId":b.tenant,"afterSequence":after}), &clock).await?;
        ledger::history(c, b, after, &bytes)
    }

    pub(super) async fn ack(&self, c: &mut Connection, b: &Binding,
        token: &NativeAccessToken, clock: impl Fn() -> Result<u64, Failure>) -> Result<(), Failure> {
        self.ack_reviewed(c, b, token, None, clock).await
    }

    async fn ack_reviewed(&self, c: &mut Connection, b: &Binding,
        token: &NativeAccessToken, expected: Option<u64>, clock: impl Fn() -> Result<u64, Failure>) -> Result<(), Failure> {
        self.authorize(token, b, &clock)?;
        let target = ledger::begin_ack_attempt(c, b, expected)?;
        let bytes = self.request(token, b, json!({"action":"ack","auctionId":b.auction,
            "tenantId":b.tenant,"resultId":target.result_id,
            "deliverySequence":target.sequence}), &clock).await?;
        ledger::ack(c, b, &target, &bytes)
    }
}

// Read-only IPC-shaped dispatch for the isolated native composition. The
// renderer selects only a local artwork and archive scope. The resolver is
// native-owned, must independently verify the immutable published association
// and current authority, and must never infer it from a preview or ledger row.
// No HTTP request is made by this function and no release dispatcher mounts it.
pub(super) fn read_selected_summary(
    conn: &mut Connection,
    request: super::resource_http::ipc::Request,
    token: Option<&NativeAccessToken>,
    clock: impl Fn() -> Result<u64, Failure>,
    resolve: impl FnOnce(&Connection, &str, &str, &str) -> Result<Option<Binding>, Failure>,
) -> Result<Value, Failure> {
    use super::resource_http::ipc::Request;
    let Request::ReadSettlement { artwork_id, scope } = request else {
        return Err("settlement_invalid_request");
    };
    let token = token.ok_or("settlement_access_denied")?;
    token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
    let canonical_uuid = |s: &str| uuid::Uuid::parse_str(s).is_ok_and(|u| u.to_string() == s);
    if !canonical_uuid(&artwork_id) || !canonical_uuid(&scope) {
        return Err("settlement_binding_mismatch");
    }
    let current: String = conn.query_row(
        "SELECT scope FROM auction_archive_scope WHERE singleton=1", [], |r| r.get(0)
    ).map_err(|_| "settlement_storage_failed")?;
    if current != scope { return Err("settlement_binding_mismatch"); }
    let binding = resolve(conn, &scope, &artwork_id, &token.subject)?
        .ok_or("settlement_binding_unavailable")?;
    if binding.scope != scope || binding.local_work != artwork_id || binding.owner != token.subject {
        return Err("settlement_binding_mismatch");
    }
    // Ledger rechecks archive and existence inside its own read transaction.
    let summary = ledger::summary(conn, &binding)?;
    token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
    serde_json::to_value(summary).map_err(|_| "settlement_invalid_response")
}

// The review is native-only, short lived, and tied to exact durable evidence,
// association and settlement grant. The renderer only receives an opaque ID.
pub(super) struct ReviewedAck {
    id: String,
    binding: Binding,
    summary: ledger::SettlementSummary,
    grant_digest: ring::digest::Digest,
    expires_at: u64,
}
pub(super) fn review_selected_ack(
    conn: &mut Connection, work: &str, scope: &str, expected: u64,
    token: Option<&NativeAccessToken>, clock: impl Fn() -> Result<u64, Failure>,
    verify_live: impl Fn(&super::settlement_store::PublishedBinding) -> Result<(), Failure>,
) -> Result<(ReviewedAck, Value), Failure> {
    let token = token.ok_or("settlement_access_denied")?;
    let now = clock()?;
    let bearer = token.bearer_for(GrantClient::AuctionsSettlement, now)?;
    let binding = super::settlement_store::resolve_selected(conn, scope, work, &token.subject, verify_live)?
        .ok_or("settlement_binding_unavailable")?;
    let summary = ledger::summary(conn, &binding)?.ok_or("settlement_not_ingested")?;
    if expected != summary.sequence { return Err("settlement_stale_ack"); }
    token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
    let id = uuid::Uuid::new_v4().to_string();
    let response = json!({"confirmationId":id,"summary":summary});
    Ok((ReviewedAck { id, binding, summary,
        grant_digest: ring::digest::digest(&ring::digest::SHA256, bearer.as_bytes()),
        expires_at: now.saturating_add(60) }, response))
}

// One explicit operation; no automatic pagination, retries or ACK.
pub(super) async fn dispatch_selected_remote(
    addr: SocketAddrV4, conn: &mut Connection, request: super::resource_http::ipc::Request,
    token: Option<&NativeAccessToken>, clock: impl Fn() -> Result<u64, Failure>,
    verify_live: impl Fn(&super::settlement_store::PublishedBinding) -> Result<(), Failure>,
    reviewed: Option<ReviewedAck>,
) -> Result<Value, Failure> {
    use super::resource_http::ipc::Request;
    enum Operation { Poll, History, Ack(u64, String) }
    let (work, scope, op) = match request {
        Request::PollSettlement { artwork_id, scope } => (artwork_id, scope, Operation::Poll),
        Request::HistorySettlement { artwork_id, scope } => (artwork_id, scope, Operation::History),
        Request::AckSettlement { artwork_id, scope, expected_sequence, confirmation_id } =>
            (artwork_id, scope, Operation::Ack(expected_sequence, confirmation_id)),
        _ => return Err("settlement_invalid_request"),
    };
    let token = token.ok_or("settlement_access_denied")?;
    token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
    let uuid = |s: &str| uuid::Uuid::parse_str(s).is_ok_and(|u| u.to_string() == s);
    if !uuid(&work) || !uuid(&scope) { return Err("settlement_binding_mismatch"); }
    let resolve = |db: &Connection| super::settlement_store::resolve_selected(
        db, &scope, &work, &token.subject, &verify_live
    )?.ok_or("settlement_binding_unavailable");
    let binding = resolve(conn)?;
    let client = NativeSettlementHttp::new(addr)?;
    match op {
        Operation::Poll => { client.poll(conn, &binding, token, &clock).await?; }
        Operation::History => { client.history(conn, &binding, token, &clock).await?; }
        Operation::Ack(expected, id) => {
            let review = reviewed.ok_or("settlement_review_required")?;
            let now = clock()?;
            let bearer = token.bearer_for(GrantClient::AuctionsSettlement, now)?;
            let digest = ring::digest::digest(&ring::digest::SHA256, bearer.as_bytes());
            if id != review.id || now >= review.expires_at ||
                digest.as_ref() != review.grant_digest.as_ref() || binding != review.binding {
                return Err("settlement_review_required");
            }
            let summary = ledger::summary(conn, &binding)?.ok_or("settlement_not_ingested")?;
            if expected != summary.sequence || summary != review.summary { return Err("settlement_stale_ack"); }
            if summary.acknowledged_at.is_none() {
                client.ack_reviewed(conn, &binding, token, Some(expected), &clock).await?;
            }
        }
    }
    token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
    if resolve(conn)? != binding { return Err("settlement_binding_mismatch"); }
    token.bearer_for(GrantClient::AuctionsSettlement, clock()?)?;
    serde_json::to_value(ledger::summary(conn, &binding)?).map_err(|_| "settlement_invalid_response")
}
