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
        let after = ledger::cursor(c, b)?;
        let bytes = self.request(token, b, json!({"action":"history","auctionId":b.auction,
            "tenantId":b.tenant,"afterSequence":after}), &clock).await?;
        ledger::history(c, b, after, &bytes)
    }

    pub(super) async fn ack(&self, c: &mut Connection, b: &Binding,
        token: &NativeAccessToken, clock: impl Fn() -> Result<u64, Failure>) -> Result<(), Failure> {
        self.authorize(token, b, &clock)?;
        let target = ledger::ack_target(c, b)?;
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
