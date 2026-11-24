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

#[derive(Deserialize)]
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
        if !valid_token(tenant_id, 3, 80) { return Err("invalid_tenant"); }
        let request = self.client.get(format!("{}?tenantId={tenant_id}", self.endpoint(PREVIEWS)?));
        let (_, list): (_, PreviewList) = self.request(request, token, &clock, Instant::now()+DEADLINE,false).await?;
        let mut seen = std::collections::HashSet::new();
        if list.auctions.len() > 100 || list.auctions.iter().any(|p| !valid_preview(p) || !seen.insert(p.auction_id.clone())) {
            return Err("auction_invalid_response");
        }
        // Read-only view: never creates a local sale or enables bidding.
        Ok(list.auctions)
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
            .filter(|t| t.status == "active" && matches!(t.role.as_str(),"owner" | "editor") && t.approved == Some(1))
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
