//! Native client for the agreed non-binding preview API. Test-only loopback,
//! no opener, IPC, pilot origin, automatic retries or writes to the local archive.
use super::{allowed, http_error, isolated_client, read_json, remaining, Failure,
    GrantClient, NativeAccessToken, DEADLINE};
use reqwest::{header, Client, RequestBuilder, StatusCode};
use serde::Deserialize;
use std::{net::SocketAddrV4, time::Instant};

#[path = "auction_preview_payload.rs"]
mod preview_payload;
use preview_payload::{parse_canonical_payload, valid_token, valid_slug, utc_millis};
#[path = "auction_draft_record.rs"]
mod draft_record;
use draft_record::{StoredReceipt, reconciliation, parse_record, Pending, pending_matches_draft};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

#[path = "auction_preview_ipc.rs"]
pub(super) mod ipc;
use ipc::Request;
use serde_json::{json, Value};

const TENANTS: &str = "/api/studio/tenants";
const PREVIEWS: &str = "/api/studio/previews";

pub(super) struct NativeAuctionsHttp {
    addr: SocketAddrV4,
    client: Client,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Tenant {
    pub(super) tenant_id: String,
    status: String,
    role: String,
    artist_slug: Option<String>,
    artist_name: Option<String>,
    approved: Option<u8>,
    // Required JSON boolean from the server's publication predicate. An old
    // or malformed response is not an authorization to prepare a request.
    publication_eligible: bool,
    published_this_month: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TenantList {
    month: String,
    monthly_per_artist_limit: u32,
    global_monthly_limit: u32,
    pub(super) tenants: Vec<Tenant>,
}

#[derive(Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Preview {
    pub(super) auction_id: String,
    slug: String,
    title: String,
    pub(super) status: String,
    starts_at: String,
    ends_at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewList { auctions: Vec<Preview> }

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PreviewReceipt {
    pub(super) auction_id: String,
    slug: String,
    status: String,
    bidding_enabled: bool,
    pub(super) idempotent_replay: bool,
}

fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}

fn valid_tenant(t: &Tenant) -> bool {
    valid_token(&t.tenant_id, 3, 80) &&
        matches!(t.status.as_str(), "pending" | "active" | "suspended") &&
        matches!(t.role.as_str(), "owner" | "editor" | "viewer") &&
        t.approved.is_none_or(|a| a <= 1) &&
        t.artist_slug.as_ref().is_none_or(|s| valid_slug(s)) &&
        t.artist_name.as_ref().is_none_or(|s| !s.is_empty() && s.len() <= 1024) &&
        t.published_this_month <= 12
}

fn valid_preview(p: &Preview) -> bool {
    canonical_uuid(&p.auction_id) && !p.slug.is_empty() && p.slug.len() <= 122 &&
        p.slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') &&
        !p.title.is_empty() && p.title.encode_utf16().count() <= 160 &&
        !p.status.is_empty() && p.status.len() <= 40 &&
        p.status.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') &&
        utc_millis(&p.starts_at).zip(utc_millis(&p.ends_at)).is_some_and(|(a,b)| a < b)
}

// A reviewed local snapshot is never a server confirmation. The native test
// adapter binds the exact bytes to the selected work, archive and revision.
fn reviewable_record(conn: &Connection, artwork_id: &str, scope: &str, revision: &str,
    body: &str) -> Result<String, String> {
    let conflict = || "auction_draft_conflict".to_string();
    if !canonical_uuid(scope) || !canonical_uuid(revision) { return Err(conflict()); }
    let current_scope: String = conn.query_row(
        "SELECT scope FROM auction_archive_scope WHERE singleton=1", [], |r| r.get(0))
        .map_err(|_| conflict())?;
    if current_scope != scope { return Err(conflict()); }
    let raw: String = conn.query_row(
        "SELECT record_json FROM auction_drafts WHERE artwork_id=?1 AND revision=?2",
        params![artwork_id,revision], |r| r.get(0)).optional()
        .map_err(|_| conflict())?.ok_or_else(conflict)?;
    let record = parse_record(&raw, scope, artwork_id)?;
    if record.pending.is_some() { return Err("pending_preview_unresolved".into()); }
    let status: Option<String> = conn.query_row("SELECT status FROM artworks WHERE id=?1",
        [artwork_id], |r| r.get(0)).optional().map_err(|_| conflict())?;
    if status.as_deref() != Some("available") { return Err("artwork_not_available".into()); }
    let candidate = Pending { artwork_id: artwork_id.into(), idempotency_key: String::new(), body: body.into() };
    if !pending_matches_draft(&candidate, &record.draft)? { return Err("preview_draft_changed".into()); }
    Ok(raw)
}

fn check_new_schedule(body: &str, now: u64) -> Result<(), String> {
    let payload = parse_canonical_payload(body).map_err(str::to_owned)?;
    let millis = i64::try_from(now).ok().and_then(|s| s.checked_mul(1000))
        .ok_or_else(|| "invalid_schedule".to_string())?;
    if utc_millis(&payload.starts_at).is_none_or(|s| s < millis) {
        return Err("invalid_schedule".into());
    }
    Ok(())
}

fn eligible_tenant(t: &Tenant) -> bool {
    t.publication_eligible && t.status == "active" && matches!(t.role.as_str(), "owner" | "editor") &&
        t.approved == Some(1) && t.artist_slug.is_some()
}

impl NativeAuctionsHttp {
    pub(super) fn new(addr: SocketAddrV4) -> Result<Self, Failure> {
        allowed(addr)?;
        Ok(Self { addr, client: isolated_client()? })
    }

    fn endpoint(&self, path: &'static str) -> Result<String, Failure> {
        allowed(self.addr)?;
        if !matches!(path, TENANTS | PREVIEWS) { return Err("auction_destination_denied"); }
        Ok(format!("http://{}{path}", self.addr))
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self, request: RequestBuilder, token: &NativeAccessToken,
        clock: &impl Fn() -> Result<u64, Failure>, deadline: Instant, create: bool,
    ) -> Result<(StatusCode, T), Failure> {
        let bearer = token.bearer_for(GrantClient::Auctions, clock()?)?;
        let response = request.bearer_auth(bearer).timeout(remaining(deadline)?)
            .send().await.map_err(http_error)?;
        let status = response.status();
        if status != StatusCode::OK && !(create && status == StatusCode::CREATED) {
            // Never leak a server error body or retry a mutation automatically.
            return Err(match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "auction_access_denied",
                StatusCode::CONFLICT => "auction_conflict",
                StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => "auction_unavailable",
                _ => "auction_http_rejected",
            });
        }
        Ok((status, read_json(response, deadline).await?))
    }

    async fn tenants_until(
        &self, token: &NativeAccessToken, clock: &impl Fn() -> Result<u64, Failure>, deadline: Instant,
    ) -> Result<TenantList, Failure> {
        let (_, list): (_, TenantList) = self.request(self.client.get(self.endpoint(TENANTS)?),
            token, clock, deadline, false).await?;
        let mut seen = std::collections::HashSet::new();
        if list.tenants.len() > 100 || list.month.len() != 7 ||
            chrono::NaiveDate::parse_from_str(&format!("{}-01",list.month), "%Y-%m-%d").is_err() ||
            list.monthly_per_artist_limit != 3 || list.global_monthly_limit != 12 ||
            list.tenants.iter().any(|t| !valid_tenant(t) || !seen.insert(t.tenant_id.clone())) {
            return Err("auction_invalid_response");
        }
        Ok(list)
    }

    pub(super) async fn list_tenants(
        &self, token: &NativeAccessToken, clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<TenantList, Failure> {
        self.tenants_until(token, &clock, Instant::now()+DEADLINE).await
    }

    pub(super) async fn list_previews(
        &self, token: &NativeAccessToken, tenant_id: &str, clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<Vec<Preview>, Failure> {
        self.list_previews_until(token, tenant_id, &clock, Instant::now()+DEADLINE).await
    }

    async fn list_previews_until(
        &self, token: &NativeAccessToken, tenant_id: &str,
        clock: &impl Fn() -> Result<u64, Failure>, deadline: Instant,
    ) -> Result<Vec<Preview>, Failure> {
        if !valid_token(tenant_id, 3, 80) { return Err("invalid_tenant"); }
        let request = self.client.get(format!("{}?tenantId={tenant_id}", self.endpoint(PREVIEWS)?));
        let (_, list): (_, PreviewList) = self.request(request, token, clock, deadline,false).await?;
        let mut seen = std::collections::HashSet::new();
        if list.auctions.len() > 100 || list.auctions.iter().any(|p| !valid_preview(p) || !seen.insert(p.auction_id.clone())) {
            return Err("auction_invalid_response");
        }
        // Read-only view: never creates a local sale or enables bidding.
        Ok(list.auctions)
    }

    // Only the fresh, validated server membership may supply eligible IDs.
    // No token, private profile or archive is projected into the chooser.
    pub(super) async fn eligible_tenants(
        &self, token: &NativeAccessToken, clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<Vec<String>, Failure> {
        let list = self.list_tenants(token, clock).await?;
        Ok(list.tenants.into_iter().filter(eligible_tenant).map(|t| t.tenant_id).collect())
    }

    /// Explicit confirmation of the exact public body on a SAVED draft. Native
    /// generates the key and persists before any future POST. This method
    /// performs only a membership GET; there is no automatic publication.
    pub(super) async fn prepare_reviewed_preview(
        &self, conn: &mut Connection, token: &NativeAccessToken, artwork_id: &str,
        scope: &str, revision: &str, reviewed_body: &str,
        clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<String, String> {
        let payload = parse_canonical_payload(reviewed_body).map_err(str::to_owned)?;
        check_new_schedule(reviewed_body, clock().map_err(str::to_owned)?)?;
        let original = reviewable_record(conn, artwork_id, scope, revision, reviewed_body)?;
        let eligible = self.eligible_tenants(token, &clock).await.map_err(str::to_owned)?;
        if !eligible.contains(&payload.tenant_id) { return Err("auction_access_denied".into()); }
        let now = clock().map_err(str::to_owned)?;
        token.bearer_for(GrantClient::Auctions, now).map_err(str::to_owned)?;
        check_new_schedule(reviewed_body, now)?;
        let failed = || "auction_draft_write_failed".to_string();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|_| failed())?;
        let current = reviewable_record(&tx, artwork_id, scope, revision, reviewed_body)?;
        if current != original { return Err("auction_draft_conflict".into()); }
        let key = uuid::Uuid::new_v4().to_string();
        let mut value: serde_json::Value = serde_json::from_str(&current).map_err(|_| failed())?;
        value["pending"] = serde_json::json!({"artworkId":artwork_id,"idempotencyKey":key,"body":reviewed_body});
        let raw = serde_json::to_string(&value).map_err(|_| failed())?;
        parse_record(&raw, scope, artwork_id)?;
        let next_revision = uuid::Uuid::new_v4().to_string();
        let count = tx.execute(
            "UPDATE auction_drafts SET revision=?1,record_json=?2 WHERE artwork_id=?3 AND revision=?4 AND record_json=?5",
            params![next_revision,raw,artwork_id,revision,current]).map_err(|_| failed())?;
        if count != 1 { return Err(failed()); }
        let saved = reconciliation::load(&tx, artwork_id, scope, &next_revision)?;
        if saved.pending.idempotency_key != key || saved.pending.body != reviewed_body || saved.receipt.is_some() {
            return Err(failed());
        }
        // Detect a trigger that silently rewrites even a valid non-pending field.
        let actual: String = tx.query_row("SELECT record_json FROM auction_drafts WHERE artwork_id=?1",
            [artwork_id], |r| r.get(0)).map_err(|_| failed())?;
        if actual != raw { return Err(failed()); }
        tx.commit().map_err(|_| failed())?;
        Ok(next_revision)
    }

    /// Explicit reconciliation of one previously saved request in isolated tests.
    /// No automatic send, no IPC, no archive-wide synchronization. A rejected
    /// or lost response leaves the exact pending attempt untouched.
    pub(super) async fn reconcile_preview(
        &self, conn: &mut rusqlite::Connection, token: &NativeAccessToken,
        artwork_id: &str, scope: &str, revision: &str,
        clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<StoredReceipt, String> {
        let attempt = reconciliation::load(conn, artwork_id, scope, revision)?;
        if let Some(receipt) = &attempt.receipt { return Ok(receipt.clone()); }
        let payload = parse_canonical_payload(&attempt.pending.body).map_err(str::to_owned)?;
        let response = self.create_preview(token, &attempt.pending.idempotency_key, &attempt.pending.body, clock)
            .await.map_err(str::to_owned)?;
        let receipt = StoredReceipt { tenant_id: payload.tenant_id,
            idempotency_key: attempt.pending.idempotency_key.clone(), auction_id: response.auction_id,
            slug: response.slug };
        reconciliation::persist(conn, &attempt, &receipt)
    }

    /// The future native caller must supply the exact key and body from a successfully
    /// persisted pending record. Do not call from IPC or automatically after a restore.
    /// Do not clear/rewrite that pending record based on a transport error.
    pub(super) async fn create_preview(
        &self, token: &NativeAccessToken, idempotency_key: &str, body: &str,
        clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<PreviewReceipt, Failure> {
        if !valid_token(idempotency_key,16,96) { return Err("invalid_idempotency_key"); }
        let payload = parse_canonical_payload(body)?;
        let deadline = Instant::now()+DEADLINE;
        // No role inferred from JWT or accepted from the renderer; the server
        // independently authorizes POST after this fresh membership check.
        let tenants = self.tenants_until(token, &clock, deadline).await?;
        let tenant = tenants.tenants.iter().find(|t| t.tenant_id == payload.tenant_id)
            .filter(|t| eligible_tenant(t))
            .ok_or("auction_access_denied")?;
        let artist = tenant.artist_slug.as_ref().ok_or("auction_access_denied")?;
        let expected_slug = format!("{artist}--{}",payload.slug);
        // A past schedule may be a valid idempotent replay. The server rejects NEW past requests.
        let request = self.client.post(self.endpoint(PREVIEWS)?)
            .header(header::CONTENT_TYPE,"application/json")
            .header("Idempotency-Key",idempotency_key)
            .body(body.to_owned());
        let (status, receipt): (_, PreviewReceipt) = self.request(request,token,&clock,deadline,true).await?;
        if !canonical_uuid(&receipt.auction_id) || receipt.slug != expected_slug ||
            receipt.status != "preview" || receipt.bidding_enabled ||
            receipt.idempotent_replay != (status == StatusCode::OK) {
            return Err("auction_invalid_response");
        }
        Ok(receipt)
    }
}


/// Native process owns the verified profile token and the current one-shot
/// confirmation. The renderer-facing projection contains only the public body
/// and an opaque confirmation ID; no token, URL or idempotency key.
/// This module is compiled only in the isolated PKCE test target.
pub(super) struct NativePreviewSession {
    api: NativeAuctionsHttp,
    token: NativeAccessToken,
    reviewed: Option<ReviewedAction>,
    selected: Option<(String, bool)>,
}

/// Explicit allowlist for the future renderer's organization chooser. The
/// transport response (including raw eligibility flags) is never forwarded.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublicOrganization {
    tenant_id: String,
    artist_name: Option<String>,
    role: String,
    status: String,
    publication_eligible: bool,
    published_this_month: u32,
}

impl From<Tenant> for PublicOrganization {
    fn from(t: Tenant) -> Self {
        let publication_eligible = eligible_tenant(&t);
        Self { tenant_id: t.tenant_id, artist_name: t.artist_name, role: t.role,
            status: t.status, publication_eligible, published_this_month: t.published_this_month }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublicOrganizationPreviews {
    organization: PublicOrganization,
    previews: Vec<Preview>,
}

/// Receipt projection excludes the persisted idempotency key. The underlying
/// native record retains it for exact retry/reconciliation.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublicPreviewReceipt {
    tenant_id: String,
    auction_id: String,
    slug: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublicReview {
    pub(super) confirmation_id: String,
    pub(super) public_body: String,
    pub(super) action: &'static str,
}

struct ReviewedAction {
    id: String,
    artwork_id: String,
    scope: String,
    revision: String,
    raw: String,
    body: String,
    reconcile: bool,
}

impl NativePreviewSession {
    pub(super) fn new(api: NativeAuctionsHttp, token: NativeAccessToken,
        now: u64) -> Result<Self, Failure> {
        token.bearer_for(GrantClient::Auctions, now)?;
        Ok(Self { api, token, reviewed: None, selected: None })
    }

    pub(super) fn cancel_review(&mut self) { self.reviewed = None; }

    pub(super) fn clear_selection(&mut self) {
        self.cancel_review();
        self.selected = None;
    }

    /// No cached membership or automatic polling. Refresh invalidates both
    /// selection and outstanding confirmation, even if the request fails.
    pub(super) async fn organizations(
        &mut self, clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<Vec<PublicOrganization>, Failure> {
        self.clear_selection();
        let list = self.api.list_tenants(&self.token, clock).await?;
        Ok(list.tenants.into_iter().map(PublicOrganization::from).collect())
    }

    /// Check fresh membership before requesting the selected tenant's list.
    /// A viewer can read but cannot review a mutation. Both GETs share one
    /// deadline; failed selection never leaves an earlier selection active.
    pub(super) async fn select_organization(
        &mut self, tenant_id: &str, clock: impl Fn() -> Result<u64, Failure>,
    ) -> Result<PublicOrganizationPreviews, Failure> {
        self.clear_selection();
        if !valid_token(tenant_id, 3, 80) { return Err("invalid_tenant"); }
        let deadline = Instant::now()+DEADLINE;
        let list = self.api.tenants_until(&self.token, &clock, deadline).await?;
        let tenant = list.tenants.into_iter().find(|t| t.tenant_id == tenant_id && t.status == "active")
            .ok_or("auction_access_denied")?;
        let organization = PublicOrganization::from(tenant);
        let previews = self.api.list_previews_until(&self.token, tenant_id, &clock, deadline).await?;
        self.token.bearer_for(GrantClient::Auctions, clock()?)?;
        self.selected = Some((tenant_id.to_owned(), organization.publication_eligible));
        Ok(PublicOrganizationPreviews { organization, previews })
    }

    fn require_selected(&self, body: &str) -> Result<(), String> {
        let payload = parse_canonical_payload(body).map_err(str::to_owned)?;
        if !self.selected.as_ref().is_some_and(|(id, eligible)| *eligible && id == &payload.tenant_id) {
            return Err("auction_access_denied".into());
        }
        Ok(())
    }

    fn stage(&mut self, artwork_id: &str, scope: &str, revision: &str,
        raw: String, body: &str, reconcile: bool) -> PublicReview {
        let id = uuid::Uuid::new_v4().to_string();
        self.reviewed = Some(ReviewedAction { id: id.clone(), artwork_id: artwork_id.into(),
            scope: scope.into(), revision: revision.into(), raw, body: body.into(), reconcile });
        PublicReview { confirmation_id: id, public_body: body.into(),
            action: if reconcile { "reconcile" } else { "prepare" } }
    }

    /// Review is read-only and does not perform I/O beyond the local archive.
    /// Any previous review is invalidated, including when a new review fails.
    pub(super) fn review_new(&mut self, conn: &Connection, artwork_id: &str,
        scope: &str, revision: &str, public_body: &str, now: u64) -> Result<PublicReview, String> {
        self.cancel_review();
        self.token.bearer_for(GrantClient::Auctions, now).map_err(str::to_owned)?;
        self.require_selected(public_body)?;
        check_new_schedule(public_body, now)?;
        let raw = reviewable_record(conn, artwork_id, scope, revision, public_body)?;
        Ok(self.stage(artwork_id, scope, revision, raw, public_body, false))
    }

    /// Review exactly the saved pending request, not the current editable fields.
    /// Already confirmed receipts require no network or further confirmation.
    pub(super) fn review_pending(&mut self, conn: &Connection, artwork_id: &str,
        scope: &str, revision: &str, now: u64) -> Result<PublicReview, String> {
        self.cancel_review();
        self.token.bearer_for(GrantClient::Auctions, now).map_err(str::to_owned)?;
        let attempt = reconciliation::load(conn, artwork_id, scope, revision)?;
        if attempt.receipt.is_some() { return Err("auction_already_confirmed".into()); }
        self.require_selected(&attempt.pending.body)?;
        let raw: String = conn.query_row(
            "SELECT record_json FROM auction_drafts WHERE artwork_id=?1 AND revision=?2",
            params![artwork_id, revision], |r| r.get(0)).map_err(|_| "auction_draft_conflict".to_string())?;
        Ok(self.stage(artwork_id, scope, revision, raw, &attempt.pending.body, true))
    }

    fn take(&mut self, id: &str, reconcile: bool) -> Result<ReviewedAction, String> {
        let reviewed = self.reviewed.take().ok_or_else(|| "auction_review_required".to_string())?;
        if reviewed.id != id || reviewed.reconcile != reconcile {
            return Err("auction_review_required".into());
        }
        Ok(reviewed)
    }

    /// Confirmation consumes its capability before any await, even on failure.
    /// Native preparation checks fresh tenant membership and generates the key.
    pub(super) async fn confirm_prepare(&mut self, conn: &mut Connection, id: &str,
        clock: impl Fn() -> Result<u64, Failure>) -> Result<String, String> {
        let r = self.take(id, false)?;
        let raw = reviewable_record(conn, &r.artwork_id, &r.scope, &r.revision, &r.body)?;
        if raw != r.raw { return Err("auction_draft_conflict".into()); }
        self.api.prepare_reviewed_preview(conn, &self.token, &r.artwork_id,
            &r.scope, &r.revision, &r.body, clock).await
    }

    pub(super) async fn confirm_reconcile(&mut self, conn: &mut Connection, id: &str,
        clock: impl Fn() -> Result<u64, Failure>) -> Result<PublicPreviewReceipt, String> {
        let r = self.take(id, true)?;
        self.token.bearer_for(GrantClient::Auctions, clock().map_err(str::to_owned)?)
            .map_err(str::to_owned)?;
        let attempt = reconciliation::load(conn, &r.artwork_id, &r.scope, &r.revision)?;
        let raw: String = conn.query_row(
            "SELECT record_json FROM auction_drafts WHERE artwork_id=?1 AND revision=?2",
            params![r.artwork_id, r.revision], |row| row.get(0))
            .map_err(|_| "auction_draft_conflict".to_string())?;
        if raw != r.raw || attempt.pending.body != r.body || attempt.receipt.is_some() {
            return Err("auction_draft_conflict".into());
        }
        let receipt = self.api.reconcile_preview(conn, &self.token, &r.artwork_id, &r.scope,
            &r.revision, clock).await?;
        Ok(PublicPreviewReceipt { tenant_id: receipt.tenant_id,
            auction_id: receipt.auction_id, slug: receipt.slug })
    }
}


/// The exact IPC request shape runs against a verified session only inside the
/// isolated acceptance target. The renderer cannot instantiate this session;
/// a test must first finish native PKCE + signed JWT exchange. No Tauri
/// command registers this dispatcher in a distributed application.
pub(super) async fn dispatch_isolated(
    session: &mut Option<NativePreviewSession>, conn: &mut Connection, request: Request,
    clock: impl Fn() -> Result<u64, Failure>,
) -> Result<Value, String> {
    let request = match request {
        Request::Disconnect {} => {
            *session = None;
            return Ok(json!({"status":"disconnected"}));
        }
        Request::Status {} => {
            let active = session.as_ref().is_some_and(|s|
                clock().ok().is_some_and(|now| s.token.bearer_for(GrantClient::Auctions, now).is_ok()));
            if !active { *session = None; }
            return Ok(json!({"status":if active {"connected"} else {"disconnected"}}));
        }
        other => other,
    };
    // Expiry or clock failure must discard the entire native session before
    // handling another operation. The persisted draft is not touched.
    let active = session.as_ref().is_some_and(|s|
        clock().ok().is_some_and(|now| s.token.bearer_for(GrantClient::Auctions, now).is_ok()));
    if !active {
        *session = None;
        return Err("auction_not_connected".into());
    }
    let s = session.as_mut().ok_or_else(|| "auction_not_connected".to_string())?;
    let project = |value: Result<Value, serde_json::Error>| value.map_err(|_| "auction_invalid_response".to_string());
    match request {
        Request::Organizations {} => project(serde_json::to_value(s.organizations(&clock).await.map_err(str::to_owned)?)),
        Request::SelectOrganization { tenant_id } =>
            project(serde_json::to_value(s.select_organization(&tenant_id, &clock).await.map_err(str::to_owned)?)),
        Request::ClearSelection {} => { s.clear_selection(); Ok(json!(null)) },
        Request::CancelReview {} => { s.cancel_review(); Ok(json!(null)) },
        Request::ReviewNew { artwork_id, scope, revision, public_body } =>
            project(serde_json::to_value(s.review_new(conn, &artwork_id, &scope, &revision, &public_body, clock().map_err(str::to_owned)?)?)),
        Request::ReviewPending { artwork_id, scope, revision } =>
            project(serde_json::to_value(s.review_pending(conn, &artwork_id, &scope, &revision, clock().map_err(str::to_owned)?)?)),
        Request::ConfirmPrepare { confirmation_id } =>
            Ok(json!({"revision":s.confirm_prepare(conn, &confirmation_id, &clock).await?})),
        Request::ConfirmReconcile { confirmation_id } =>
            project(serde_json::to_value(s.confirm_reconcile(conn, &confirmation_id, &clock).await?)),
        // Profile grants cannot read private settlement data.
        Request::ReadSettlement { .. } => Err("settlement_access_denied".into()),
        Request::Status {} | Request::Disconnect {} => Err("auction_invalid_request".into()),
    }
}

/// Compose the profile and separately scoped settlement grants only inside
/// the isolated acceptance target. The renderer cannot supply a token or
/// publication verifier. A profile change/disconnect invalidates settlement
/// authority; a settlement grant never enables preview mutations.
pub(super) async fn dispatch_workspace_isolated(
    profile: &mut Option<NativePreviewSession>,
    settlement: &mut Option<NativeAccessToken>,
    conn: &mut Connection,
    request: Request,
    clock: impl Fn() -> Result<u64, Failure>,
    verify_live: impl Fn(&super::settlement_store::PublishedBinding) -> Result<(), Failure>,
) -> Result<Value, String> {
    let now = clock();
    let profile_active = profile.as_ref().is_some_and(|s|
        now.is_ok_and(|t| s.token.bearer_for(GrantClient::Auctions, t).is_ok()));
    if !profile_active || matches!(&request, Request::Disconnect {}) {
        *settlement = None;
    } else if let (Some(p), Some(s)) = (profile.as_ref(), settlement.as_ref()) {
        if p.token.subject != s.subject ||
            !now.is_ok_and(|t| s.bearer_for(GrantClient::AuctionsSettlement, t).is_ok()) {
            *settlement = None;
        }
    }
    if matches!(&request, Request::ReadSettlement { .. }) {
        if !profile_active {
            *profile = None;
            return Err("settlement_access_denied".into());
        }
        let outcome = super::settlement_http::read_selected_summary(
            conn, request, settlement.as_ref(), &clock,
            |db, scope, work, subject| super::settlement_store::resolve_selected(
                db, scope, work, subject, verify_live),
        );
        // Never project a result if either grant expires while resolving it.
        let after = clock();
        if !profile.as_ref().is_some_and(|s|
            after.is_ok_and(|t| s.token.bearer_for(GrantClient::Auctions, t).is_ok())) {
            *profile = None;
            *settlement = None;
            return Err("settlement_access_denied".into());
        }
        if !settlement.as_ref().is_some_and(|s|
            after.is_ok_and(|t| s.bearer_for(GrantClient::AuctionsSettlement, t).is_ok())) {
            *settlement = None;
            return Err("settlement_access_denied".into());
        }
        return outcome.map_err(str::to_owned);
    }
    let outcome = dispatch_isolated(profile, conn, request, clock).await;
    if profile.is_none() { *settlement = None; }
    outcome
}
