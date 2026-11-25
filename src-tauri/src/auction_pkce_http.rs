//! Native OAuth exchange and RS256 verification; no renderer entry point.
//! The only constructible HTTP destination is a test-only IPv4 loopback.
//! A reviewed real provider must be added explicitly; no env or URL override.
use super::{GrantClient, IdentityProvider, TokenExchange, URL_SAFE_NO_PAD};
use base64::Engine as _;
use reqwest::{header, Client, RequestBuilder, StatusCode};
use ring::signature::{RsaPublicKeyComponents, RSA_PKCS1_2048_8192_SHA256};
use serde::Deserialize;
use std::{net::{Ipv4Addr, SocketAddrV4}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

const ISSUER: &str = "https://studio.example.test";
const DISCOVERY: &str = "/.well-known/oauth-authorization-server";
const JWKS: &str = "/.well-known/jwks.json";
const TOKEN: &str = "/api/identity/token";
const MAX_BODY: usize = 64 * 1024;
const MAX_TOKEN: usize = 8192;
const DEADLINE: Duration = Duration::from_secs(5);

type Failure = &'static str;

pub(super) struct NativeIdentityHttp {
    addr: SocketAddrV4,
    client: Client,
}

// Not Debug, Serialize or Clone. Never passed over IPC or stored in SQLite.
pub(super) struct NativeAccessToken {
    bearer: String,
    client: GrantClient,
    expires_at: u64,
    subject: String,
}

impl NativeAccessToken {
    pub(super) fn bearer_for(&self, client: GrantClient, now: u64) -> Result<&str, Failure> {
        // Audience alone is insufficient: profile and settlement share it.
        if self.client.id() != client.id() || now >= self.expires_at {
            return Err("oauth_token_rejected");
        }
        Ok(&self.bearer)
    }
}

fn allowed(addr: SocketAddrV4) -> Result<(), Failure> {
    if *addr.ip() != Ipv4Addr::LOCALHOST || addr.port() < 1024 {
        return Err("oauth_destination_denied");
    }
    Ok(())
}

fn remaining(deadline: Instant) -> Result<Duration, Failure> {
    deadline.checked_duration_since(Instant::now()).filter(|d| !d.is_zero()).ok_or("oauth_timeout")
}

fn http_error(error: reqwest::Error) -> Failure {
    if error.is_timeout() { "oauth_timeout" } else { "oauth_http_failed" }
}

async fn bounded_json<T: serde::de::DeserializeOwned>(
    request: RequestBuilder, deadline: Instant,
) -> Result<T, Failure> {
    let response = request.timeout(remaining(deadline)?).send().await.map_err(http_error)?;
    // Do not parse an error/redirect body or follow a Location.
    if response.status() != StatusCode::OK { return Err("oauth_http_rejected"); }
    read_json(response, deadline).await
}

async fn read_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response, deadline: Instant,
) -> Result<T, Failure> {
    let bytes = read_json_bytes(response, deadline).await?;
    serde_json::from_slice(&bytes).map_err(|_| "oauth_invalid_response")
}

async fn read_json_bytes(
    mut response: reqwest::Response, deadline: Instant,
) -> Result<Vec<u8>, Failure> {
    let json = response.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next()).is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"));
    if !json { return Err("oauth_invalid_response"); }
    if response.content_length().is_some_and(|len| len > MAX_BODY as u64) {
        return Err("oauth_response_too_large");
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(http_error)? {
        remaining(deadline)?;
        if chunk.len() > MAX_BODY - bytes.len() { return Err("oauth_response_too_large"); }
        bytes.extend_from_slice(&chunk);
    }
    remaining(deadline)?;
    Ok(bytes)
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenReply {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kty: String,
    alg: String,
    #[serde(rename = "use")]
    purpose: String,
    kid: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JwtHeader {
    alg: String,
    typ: String,
    kid: String,
}

#[derive(Deserialize)]
struct Claims {
    iss: String,
    aud: String,
    sub: String,
    scope: String,
    token_use: String,
    security_version: u32,
    iat: u64,
    exp: u64,
    nbf: Option<u64>,
    jti: String,
    sid: Option<String>,
}

fn decode(segment: &str) -> Result<Vec<u8>, Failure> {
    if segment.is_empty() { return Err("oauth_token_rejected"); }
    URL_SAFE_NO_PAD.decode(segment).map_err(|_| "oauth_token_rejected")
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 80 &&
        value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

// The shared provider requires a canonical RFC 4122 v4 live-session ID.
// This checks its shape only: revocation remains server-authoritative.
fn valid_session_id(sid: &str) -> bool {
    uuid::Uuid::parse_str(sid).is_ok_and(|id| id.to_string() == sid &&
        id.get_version_num() == 4 && id.get_variant() == uuid::Variant::RFC4122)
}

fn verified_expiry(token: &str, jwks: &Jwks, client: GrantClient, now: u64) -> Result<u64, Failure> {
    Ok(verified_claims(token, jwks, client, now)?.exp)
}

fn verified_claims(token: &str, jwks: &Jwks, client: GrantClient, now: u64) -> Result<Claims, Failure> {
    if token.len() > MAX_TOKEN || jwks.keys.len() > 32 { return Err("oauth_token_rejected"); }
    let mut segments = token.split('.');
    let h = segments.next().ok_or("oauth_token_rejected")?;
    let p = segments.next().ok_or("oauth_token_rejected")?;
    let s = segments.next().ok_or("oauth_token_rejected")?;
    if segments.next().is_some() { return Err("oauth_token_rejected"); }
    let header: JwtHeader = serde_json::from_slice(&decode(h)?).map_err(|_| "oauth_token_rejected")?;
    if header.alg != "RS256" || header.typ != "JWT" || !safe_identifier(&header.kid) {
        return Err("oauth_token_rejected");
    }
    // Never choose the first key, follow a token URL, or accept an ambiguous kid.
    let mut keys = jwks.keys.iter().filter(|key| key.kid == header.kid);
    let key = keys.next().ok_or("oauth_token_rejected")?;
    if keys.next().is_some() || key.kty != "RSA" || key.alg != "RS256" || key.purpose != "sig" {
        return Err("oauth_token_rejected");
    }
    let public = RsaPublicKeyComponents { n: decode(&key.n)?, e: decode(&key.e)? };
    let signed = format!("{h}.{p}");
    public.verify(&RSA_PKCS1_2048_8192_SHA256, signed.as_bytes(), &decode(s)?)
        .map_err(|_| "oauth_token_rejected")?;
    // Claims are parsed only after cryptographic verification.
    let claims: Claims = serde_json::from_slice(&decode(p)?).map_err(|_| "oauth_token_rejected")?;
    let subject = claims.sub.strip_prefix("repertica:").and_then(|s| s.parse::<i32>().ok())
        .filter(|id| *id > 0 && claims.sub == format!("repertica:{id}"));
    if claims.iss != ISSUER || claims.aud != client.audience() || claims.scope != client.scope() ||
        claims.token_use != "access" || claims.security_version == 0 || claims.security_version > i32::MAX as u32 ||
        subject.is_none() || !safe_identifier(&claims.jti) ||
        !claims.sid.as_deref().is_some_and(valid_session_id) || claims.iat > now || claims.exp <= now ||
        claims.exp.checked_sub(claims.iat).is_none_or(|ttl| ttl == 0 || ttl > 900) ||
        claims.nbf.is_some_and(|nbf| nbf > now) {
        return Err("oauth_claims_rejected");
    }
    Ok(claims)
}

fn isolated_client() -> Result<Client, Failure> {
    // The updater unifies reqwest with rustls-no-provider. Explicitly select
    // ring for this test process before building even a loopback HTTP client.
    // Concurrent tests may have already installed the same default.
    let _ = rustls::crypto::ring::default_provider().install_default();
    Client::builder()
        .no_proxy().redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
        .referer(false).http1_only().connect_timeout(Duration::from_secs(2)).timeout(DEADLINE)
        .build().map_err(|_| "oauth_http_failed")
}

#[path = "auction_resource_http.rs"]
mod resource_http;

// Settlement is deliberately a different client and remains in the test target.
// The transport shares the native ledger; only the isolated test target can construct it.
#[cfg(test)]
#[path = "auction_settlement_ledger.rs"]
mod settlement_ledger;
#[path = "auction_settlement_store.rs"]
mod settlement_store;

#[cfg(test)]
#[path = "auction_settlement_http.rs"]
mod settlement_http;


impl NativeIdentityHttp {
    pub(super) fn new(provider: IdentityProvider) -> Result<Self, Failure> {
        // This branch cannot succeed in a non-test build.
        let addr = match provider {
            IdentityProvider::Disabled => return Err("oauth_provider_disabled"),
            #[cfg(test)]
            IdentityProvider::IsolatedFixture => return Err("oauth_destination_denied"),
            #[cfg(test)]
            IdentityProvider::IsolatedHttp(addr) => addr,
        };
        allowed(addr)?;
        let client = isolated_client()?;
        Ok(Self { addr, client })
    }

    fn endpoint(&self, path: &'static str) -> Result<String, Failure> {
        allowed(self.addr)?;
        if !matches!(path, DISCOVERY | JWKS | TOKEN) { return Err("oauth_destination_denied"); }
        Ok(format!("http://{}{path}", self.addr))
    }

    pub(super) async fn exchange(&self, grant: TokenExchange) -> Result<NativeAccessToken, Failure> {
        self.exchange_with_clock(grant, DEADLINE, || SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs()).map_err(|_| "oauth_clock_invalid")).await
    }

    // The test clock is never supplied by the renderer. The production wrapper above
    // uses system time at verification, not the clock at request start.
    async fn exchange_with_clock(
        &self, grant: TokenExchange, timeout: Duration, clock: impl FnOnce() -> Result<u64, Failure>,
    ) -> Result<NativeAccessToken, Failure> {
        let deadline = Instant::now() + timeout.min(DEADLINE);
        let discovery: Discovery = bounded_json(self.client.get(self.endpoint(DISCOVERY)?), deadline).await?;
        // Advertised HTTPS destinations are compared, NEVER contacted.
        if discovery.issuer != ISSUER || discovery.authorization_endpoint != format!("{ISSUER}/login") ||
            discovery.token_endpoint != format!("{ISSUER}{TOKEN}") || discovery.jwks_uri != format!("{ISSUER}{JWKS}") {
            return Err("oauth_destination_denied");
        }
        let keys: Jwks = bounded_json(self.client.get(self.endpoint(JWKS)?), deadline).await?;
        if keys.keys.is_empty() || keys.keys.len() > 32 { return Err("oauth_token_rejected"); }
        let request = serde_json::to_vec(&grant).map_err(|_| "oauth_invalid_grant")?;
        // grant is consumed; no automatic retries on uncertain outcomes.
        let reply: TokenReply = bounded_json(self.client.post(self.endpoint(TOKEN)?)
            .header(header::CONTENT_TYPE, "application/json").body(request), deadline).await?;
        if reply.token_type != "Bearer" || reply.expires_in != 900 || reply.scope != grant.expected_client.scope() {
            return Err("oauth_token_rejected");
        }
        let claims = verified_claims(&reply.access_token, &keys, grant.expected_client, clock()?)?;
        remaining(deadline)?;
        Ok(NativeAccessToken { bearer: reply.access_token, client: grant.expected_client, expires_at: claims.exp, subject: claims.sub })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{PendingGrant, CALLBACK_PATH};
    use serde_json::{json, Value};
    use std::{io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, thread};
    const NOW: u64 = 1_800_000_000;
    const STATE: &str = "synthetic-state-for-desktop-12345";
    const CODE: &str = "synthetic-authorization-code-0123456789";
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

    fn fixture() -> Value {
        serde_json::from_str(include_str!("auction_pkce_http_fixture.json")).unwrap()
    }
    fn grant() -> TokenExchange {
        PendingGrant::with_values(GrantClient::Auctions, 49152, STATE.into(), VERIFIER.into())
            .accept_callback(&format!("{CALLBACK_PATH}?code={CODE}&state={STATE}")).unwrap()
    }
    fn discovery() -> Value {
        json!({"issuer":ISSUER,"authorization_endpoint":format!("{ISSUER}/login"),
            "token_endpoint":format!("{ISSUER}{TOKEN}"),"jwks_uri":format!("{ISSUER}{JWKS}")})
    }
    fn listen() -> (TcpListener, SocketAddrV4) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let std::net::SocketAddr::V4(addr) = listener.local_addr().unwrap() else { panic!("not IPv4"); };
        listener.set_nonblocking(true).unwrap();
        (listener, addr)
    }
    fn accept(listener: &TcpListener) -> TcpStream {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((s,_)) => {
                    s.set_nonblocking(false).unwrap();
                    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                    s.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
                    return s;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                    thread::sleep(Duration::from_millis(5)),
                Err(_) => panic!("isolated request not received"),
            }
        }
    }
    fn read_request(stream: &mut TcpStream, expected: &str) -> Vec<u8> {
        read_resource_request(stream, expected, None, None)
    }
    fn read_resource_request(stream: &mut TcpStream, expected: &str, bearer: Option<&str>, key: Option<&str>) -> Vec<u8> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim_end(), expected);
        let mut size = 0usize;
        let mut auth = None;
        let mut attempt = None;
        for _ in 0..64 {
            line.clear();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            if line == "\r\n" { break; }
            if let Some((name,value)) = line.split_once(':') {
                assert!(!name.eq_ignore_ascii_case("origin"));
                assert!(!name.eq_ignore_ascii_case("cookie"));
                if name.eq_ignore_ascii_case("authorization") {
                    assert!(auth.replace(value.trim().to_owned()).is_none());
                }
                if name.eq_ignore_ascii_case("idempotency-key") {
                    assert!(attempt.replace(value.trim().to_owned()).is_none());
                }
                if name.eq_ignore_ascii_case("content-length") { size = value.trim().parse().unwrap(); }
            }
        }
        assert_eq!(auth, bearer.map(|b|format!("Bearer {b}")));
        assert_eq!(attempt.as_deref(),key);
        assert!(size <= 8192);
        let mut body = vec![0; size];
        reader.read_exact(&mut body).unwrap();
        body
    }
    fn reply(stream: &mut TcpStream, status: &str, body: &Value) {
        let body = serde_json::to_vec(body).unwrap();
        write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
        stream.write_all(&body).unwrap();
    }

    #[test]
    fn native_http_uses_fixed_paths_and_verifies_fetched_jwks() {
        let (listener, addr) = listen();
        let worker = thread::spawn(move || {
            let f = fixture();
            for (path, body) in [(DISCOVERY, discovery()), (JWKS, f["jwks"].clone())] {
                let mut s = accept(&listener);
                assert!(read_request(&mut s, &format!("GET {path} HTTP/1.1")).is_empty());
                reply(&mut s, "200 OK", &body);
            }
            let mut s = accept(&listener);
            let body: Value = serde_json::from_slice(&read_request(&mut s, &format!("POST {TOKEN} HTTP/1.1"))).unwrap();
            assert_eq!(body, serde_json::to_value(grant()).unwrap());
            reply(&mut s, "200 OK", &json!({"access_token":f["valid"],"token_type":"Bearer","expires_in":900,"scope":"profile"}));
        });
        let result = tauri::async_runtime::block_on(async {
            NativeIdentityHttp::new(IdentityProvider::IsolatedHttp(addr)).unwrap()
                .exchange_with_clock(grant(), DEADLINE, || Ok(NOW)).await
        }).unwrap();
        worker.join().unwrap();
        assert_eq!(result.bearer_for(GrantClient::Auctions, NOW).unwrap(), fixture()["valid"].as_str().unwrap());
        assert_eq!(result.bearer_for(GrantClient::StudioApi, NOW), Err("oauth_token_rejected"));
        assert_eq!(result.bearer_for(GrantClient::AuctionsSettlement, NOW), Err("oauth_token_rejected"));
        assert_eq!(result.bearer_for(GrantClient::Auctions, NOW+900), Err("oauth_token_rejected"));
    }

    #[test]
    fn signed_wrong_scope_audience_expiry_and_ambiguous_key_fail_closed() {
        let f = fixture();
        let keys: Jwks = serde_json::from_value(f["jwks"].clone()).unwrap();
        for name in ["wrong_scope","wrong_audience","expired","missing_sid","malformed_sid","wrong_sid_version","wrong_sid_variant"] {
            assert_eq!(verified_expiry(f[name].as_str().unwrap(), &keys, GrantClient::Auctions, NOW),
                Err("oauth_claims_rejected"));
        }
        let valid = f["valid"].as_str().unwrap();
        assert_eq!(verified_expiry(valid, &keys, GrantClient::Auctions, NOW), Ok(NOW+900));
        assert_eq!(verified_expiry(valid, &keys, GrantClient::AuctionsSettlement, NOW), Err("oauth_claims_rejected"));
        let duplicate: Jwks = serde_json::from_value(json!({"keys":[f["jwks"]["keys"][0],f["jwks"]["keys"][0]]})).unwrap();
        assert_eq!(verified_expiry(valid, &duplicate, GrantClient::Auctions, NOW), Err("oauth_token_rejected"));
        let mut tampered = valid.to_owned();
        let index = tampered.rfind('.').unwrap()+1;
        tampered.replace_range(index..index+1, if valid.as_bytes()[index]==b'A' {"B"} else {"A"});
        assert_eq!(verified_expiry(&tampered, &keys, GrantClient::Auctions, NOW), Err("oauth_token_rejected"));
    }

    #[test]
    fn native_http_settlement_grant_never_authorizes_profile_resource() {
        // Independent synthetic RS256 key; only public key and signed token.
        const FIXTURE: &str = r#"{"jwks":{"keys":[{"kty":"RSA","alg":"RS256","use":"sig","kid":"settlement-fixture","n":"om0wTl8ddUlbPbKvTeQmIC4wZZA_f2E7wjEKbXGEJ3zORDsUHLKDD28etvCsObBMvD1R-iPVK5GpeeSBksj2CrC6FxU3pUoX8kqF_C6d8HToj5E9TyK7ghW3QNR11rzhWUfa0rV4rJ7iVmdcaJnMibNtqrEhm4iPPMSz4fhYmbr583ZTuFmuxzaaFvJBTQZaTl7ARxoyQW60dZweXi3vOBlBydgaDiQaEUZvXRUdPwgFgok_IaFKeQhb0mfZod85IA3m8QXDSL-8GVbMc-6IUl2Onr0oUp1V6nnnQPjdAoDiiqFmp0pQt2z_nsX3r1fTEaNTIn96VFtXu0ci0A-P3w","e":"AQAB"}]},"valid":"eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InNldHRsZW1lbnQtZml4dHVyZSJ9.eyJpc3MiOiJodHRwczovL3N0dWRpby5leGFtcGxlLnRlc3QiLCJhdWQiOiJyZXBlcnRpY2EtYXVjdGlvbnMiLCJzdWIiOiJyZXBlcnRpY2E6NDIiLCJzY29wZSI6ImF1Y3Rpb25zOnNldHRsZW1lbnQtcmVhZCBhdWN0aW9uczpzZXR0bGVtZW50LWFjayIsInRva2VuX3VzZSI6ImFjY2VzcyIsInNlY3VyaXR5X3ZlcnNpb24iOjEsImlhdCI6MTgwMDAwMDAwMCwiZXhwIjoxODAwMDAwOTAwLCJqdGkiOiJzZXR0bGVtZW50LWZpeHR1cmUiLCJzaWQiOiIxMjM0NTY3OC0xMjM0LTQyMzQtODIzNC0xMjM0NTY3ODlhYmMifQ.RYg_Dye2LXgqESNQWsByMR0wGEM-ywYCr3POsvkivPyrBgSKzfq-wMNW2dtuagRsbYwtucGJT1knnbYlzxdz7LZ3oHGdOZwtNjBtY5DLltwiWGbrwTbF2Vqm5fbBwOSt1sUMq3eM5aV_ShtrYFfmPpDAWnwsjXaIZRvwHQKu8QwQGZ-HlgeNKR0Vlw2g-hMSQs3eIhITffQuNpUcSHOInw-HuvDTauFQ0fM6RaylsgBSOIU3iOYyuJusxzsocczg0i6rdTgxZLodmUJSOB6I7XN9Sv4ypN6RxxJndzUPeHOhBLau5VGQMGp49RMt41NW1gsVLF68pWiVJwKUjEYsGg"}"#;
        fn settlement_grant() -> TokenExchange {
            PendingGrant::with_values(GrantClient::AuctionsSettlement, 49152, STATE.into(), VERIFIER.into())
                .accept_callback(&format!("{CALLBACK_PATH}?code={CODE}&state={STATE}")).unwrap()
        }
        let f: Value = serde_json::from_str(FIXTURE).unwrap();
        let keys: Jwks = serde_json::from_value(f["jwks"].clone()).unwrap();
        let signed = f["valid"].as_str().unwrap();
        assert_eq!(verified_expiry(signed, &keys, GrantClient::AuctionsSettlement, NOW), Ok(NOW+900));
        assert_eq!(verified_expiry(signed, &keys, GrantClient::Auctions, NOW), Err("oauth_claims_rejected"));
        let (listener, addr) = listen();
        let worker = thread::spawn(move || {
            let f: Value = serde_json::from_str(FIXTURE).unwrap();
            for (path, body) in [(DISCOVERY, discovery()), (JWKS, f["jwks"].clone())] {
                let mut s = accept(&listener);
                assert!(read_request(&mut s, &format!("GET {path} HTTP/1.1")).is_empty());
                reply(&mut s, "200 OK", &body);
            }
            let mut s = accept(&listener);
            let body: Value = serde_json::from_slice(&read_request(&mut s, &format!("POST {TOKEN} HTTP/1.1"))).unwrap();
            assert_eq!(body, serde_json::to_value(settlement_grant()).unwrap());
            reply(&mut s, "200 OK", &json!({"access_token":f["valid"],"token_type":"Bearer","expires_in":900,"scope":GrantClient::AuctionsSettlement.scope()}));
            listener
        });
        let result = tauri::async_runtime::block_on(async {
            NativeIdentityHttp::new(IdentityProvider::IsolatedHttp(addr)).unwrap()
                .exchange_with_clock(settlement_grant(), DEADLINE, || Ok(NOW)).await
        }).unwrap();
        let listener = worker.join().unwrap();
        assert_eq!(result.bearer_for(GrantClient::AuctionsSettlement, NOW), Ok(signed));
        assert_eq!(result.bearer_for(GrantClient::Auctions, NOW), Err("oauth_token_rejected"));
        assert_eq!(result.bearer_for(GrantClient::StudioApi, NOW), Err("oauth_token_rejected"));
        assert_eq!(result.bearer_for(GrantClient::AuctionsSettlement, NOW+900), Err("oauth_token_rejected"));
        // Actual resource adapter rejects the same-audience settlement token
        // before network I/O; no profile membership or preview request leaves.
        let resource = resource_http::NativeAuctionsHttp::new(addr).unwrap();
        let rejected = tauri::async_runtime::block_on(resource.list_tenants(&result, || Ok(NOW)));
        assert_eq!(rejected.err(), Some("oauth_token_rejected"));
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));

        // The settlement transport uses the verified subject, not an arbitrary
        // owner from a renderer, and only an explicitly bound local artwork.
        use super::settlement_ledger as ledger;
        let client = settlement_http::NativeSettlementHttp::new(addr).unwrap();
        // Use ordinary archive creation with the reconciled v21 migration.
        // The remote provider remains an isolated fixture only.
        let dir = tempfile::tempdir().unwrap();
        let archive = repertica_studio_lib::db::open(dir.path().to_path_buf()).unwrap();
        let mut db = archive.conn.into_inner().unwrap();
        let mut binding = ledger::binding();
        binding.scope = db.query_row("SELECT scope FROM auction_archive_scope WHERE singleton=1",
            [], |r| r.get(0)).unwrap();
        db.execute("INSERT INTO artworks(id,code,title,status,created_at,updated_at) VALUES(?1,'FIX-1','Synthetic','available','2026-11-25','2026-11-25')",
            [&binding.local_work]).unwrap();
        assert_eq!(db.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),21);
        let profile = verified_resource_token();
        assert_eq!(tauri::async_runtime::block_on(client.poll(&mut db,&binding,&result,||Ok(NOW))),
            Err("settlement_binding_mismatch"));
        binding.owner = "repertica:42".into();
        // Matching OAuth subject but no immutable binding is still denied
        // before the first network request, even with an empty ledger.
        assert_eq!(tauri::async_runtime::block_on(client.poll(&mut db,&binding,&result,||Ok(NOW))),
            Err("settlement_binding_mismatch"));
        let published = settlement_store::PublishedBinding {
            binding: binding.clone(),
            request_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd".into(),
            selection_digest: "a".repeat(64),
            published_at: "2026-11-24T03:00:00.000Z".into(),
        };
        let verify = |candidate: &settlement_store::PublishedBinding| {
            if candidate == &published { Ok(()) } else { Err("settlement_access_denied") }
        };
        assert_eq!(settlement_store::record(&mut db,&published,&result.subject,
            |_| Err("settlement_access_denied")),Err("settlement_access_denied"));
        settlement_store::record(&mut db,&published,&result.subject,verify).unwrap();
        settlement_store::record(&mut db,&published,&result.subject,verify).unwrap();
        let mut wrong_archive = binding.clone();
        wrong_archive.scope = "cccccccc-cccc-4ccc-8ccc-cccccccccccc".into();
        assert_eq!(tauri::async_runtime::block_on(client.poll(&mut db,&wrong_archive,&result,||Ok(NOW))),
            Err("settlement_binding_mismatch"));
        assert_eq!(tauri::async_runtime::block_on(client.poll(&mut db,&binding,&profile,||Ok(NOW))),
            Err("oauth_token_rejected"));
        assert_eq!(tauri::async_runtime::block_on(client.poll(&mut db,&binding,&result,||Ok(NOW+900))),
            Err("oauth_token_rejected"));
        assert_eq!(tauri::async_runtime::block_on(client.ack(&mut db,&binding,&result,||Ok(NOW))),
            Err("settlement_not_ingested"));
        assert!(settlement_http::NativeSettlementHttp::new(SocketAddrV4::new(Ipv4Addr::new(192,0,2,1),addr.port())).is_err());
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));

        let bearer = signed.to_owned();
        let worker = thread::spawn(move || {
            let poll = json!({"action":"poll","auctionId":"fixture-auction","tenantId":"fixture-tenant"});
            let history = json!({"action":"history","auctionId":"fixture-auction","tenantId":"fixture-tenant","afterSequence":0});
            let ack = json!({"action":"ack","auctionId":"fixture-auction","tenantId":"fixture-tenant",
                "resultId":"fixture-auction:1:fulfillment","deliverySequence":3});
            let latest = ledger::delivery(3,"review_required");
            let mut acknowledged = latest.clone();
            acknowledged["acknowledged_at"] = json!("2026-11-25T04:02:00.000Z");
            let mut confirmation = ledger::reply(acknowledged);
            confirmation["replayed"] = json!(false);
            for (expected,status,body) in [
                (poll.clone(),"200 OK",ledger::reply(latest.clone())),
                (history,"200 OK",ledger::page(vec![ledger::delivery(1,"fulfilled"),
                    ledger::delivery(2,"fulfilled"),latest.clone()],3,None)),
                (ack.clone(),"503 Service Unavailable",json!({"ok":false,"code":"unavailable"})),
                (ack,"200 OK",confirmation),
                (poll.clone(),"200 OK",ledger::reply(latest)),
                (poll.clone(),"403 Forbidden",json!({"ok":false,"code":"denied"})),
            ] {
                let mut stream = accept(&listener);
                let body_bytes = read_resource_request(&mut stream,
                    "POST /api/studio/settlement/delivery HTTP/1.1",Some(&bearer),None);
                assert_eq!(serde_json::from_slice::<Value>(&body_bytes).unwrap(),expected);
                reply(&mut stream,status,&body);
            }
            // The same fixed destination must reject ambiguous JSON without
            // normalizing duplicate keys into a plausible receipt.
            let mut stream = accept(&listener);
            assert_eq!(serde_json::from_slice::<Value>(&read_resource_request(&mut stream,
                "POST /api/studio/settlement/delivery HTTP/1.1",Some(&bearer),None)).unwrap(),poll);
            let valid = serde_json::to_string(&ledger::reply(ledger::delivery(3,"review_required"))).unwrap();
            let duplicate = format!("{{\"ok\":false,{}", &valid[1..]);
            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",duplicate.len()).unwrap();
            stream.write_all(duplicate.as_bytes()).unwrap();
            drop(stream);
            let mut stream = accept(&listener);
            assert_eq!(serde_json::from_slice::<Value>(&read_resource_request(&mut stream,
                "POST /api/studio/settlement/delivery HTTP/1.1",Some(&bearer),None)).unwrap(),poll);
            // A redirect even to the same local server is forbidden.
            write!(stream,"HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{}/other\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",addr).unwrap();
            listener
        });
        tauri::async_runtime::block_on(async {
            assert_eq!(client.poll(&mut db,&binding,&result,||Ok(NOW)).await,Err("settlement_history_gap"));
            assert_eq!(ledger::count(&db),0);
            assert_eq!(client.history(&mut db,&binding,&result,||Ok(NOW)).await,Ok(3));
            assert_eq!(client.ack(&mut db,&binding,&result,||Ok(NOW)).await,Err("settlement_unavailable"));
            let stamp:Option<String> = db.query_row("SELECT acknowledged_at FROM auction_settlement_receipts WHERE sequence=3",[],|r|r.get(0)).unwrap();
            assert!(stamp.is_none());
            // An explicit second attempt uses the same committed target.
            assert_eq!(client.ack(&mut db,&binding,&result,||Ok(NOW)).await,Ok(()));
            assert_eq!(client.poll(&mut db,&binding,&result,||Ok(NOW)).await,Ok(3));
            assert_eq!(client.poll(&mut db,&binding,&result,||Ok(NOW)).await,Err("settlement_access_denied"));
            assert_eq!(client.poll(&mut db,&binding,&result,||Ok(NOW)).await,Err("settlement_invalid_response"));
            assert_eq!(client.poll(&mut db,&binding,&result,||Ok(NOW)).await,Err("settlement_http_rejected"));
        });
        let listener = worker.join().unwrap();
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
        assert_eq!(ledger::count(&db),3);
        // Exercise the exact typed read-only request after real loopback ingestion.
        // The synthetic resolver is native-only and cannot be supplied over IPC.
        let request = || serde_json::from_value(json!({"action":"readSettlement",
            "artworkId":binding.local_work,"scope":binding.scope})).unwrap();
        // Reopen through the ordinary app entry point, not raw SQLite:
        // reconciled schema v21 must retain the immutable binding and ACK.
        drop(db);
        let archive = repertica_studio_lib::db::open(dir.path().to_path_buf()).unwrap();
        let mut db = archive.conn.into_inner().unwrap();
        assert_eq!(db.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),21);
        assert_eq!(db.pragma_query_value(None,"foreign_keys",|r|r.get::<_,i64>(0)).unwrap(),1);
        let resolve = |conn: &rusqlite::Connection, scope: &str, work: &str, subject: &str| {
            settlement_store::resolve_selected(conn,scope,work,subject,verify)
        };
        assert_eq!(settlement_http::read_selected_summary(&mut db,request(),Some(&result),||Ok(NOW),
            |conn,scope,work,subject| settlement_store::resolve_selected(conn,scope,work,subject,
                |_| Err("settlement_access_denied"))),Err("settlement_access_denied"));
        let summary = settlement_http::read_selected_summary(&mut db,request(),Some(&result),||Ok(NOW),resolve).unwrap();
        assert_eq!(summary,json!({"sequence":3,"status":"review_required","amount":25000,"currency":"EUR",
            "fulfilledAt":"2026-11-25T04:00:00.000Z","acknowledgedAt":"2026-11-25T04:02:00.000Z"}));
        assert_eq!(settlement_http::read_selected_summary(&mut db,request(),Some(&profile),||Ok(NOW),
            |_,_,_,_| panic!("profile must not resolve")),Err("oauth_token_rejected"));
        assert_eq!(settlement_http::read_selected_summary(&mut db,request(),None,||Ok(NOW),
            |_,_,_,_| panic!("anonymous must not resolve")),Err("settlement_access_denied"));
        assert_eq!(settlement_http::read_selected_summary(&mut db,request(),Some(&result),||Ok(NOW),
            |_,_,_,_| Ok(None)),Err("settlement_binding_unavailable"));
        let mut foreign = binding.clone();
        foreign.owner = "repertica:1".into();
        assert_eq!(settlement_http::read_selected_summary(&mut db,request(),Some(&result),||Ok(NOW),
            |_,_,_,_| Ok(Some(foreign))),Err("settlement_binding_mismatch"));
        db.execute("UPDATE auction_archive_scope SET scope='cccccccc-cccc-4ccc-8ccc-cccccccccccc'",[]).unwrap();
        assert_eq!(settlement_http::read_selected_summary(&mut db,request(),Some(&result),||Ok(NOW),
            |_,_,_,_| panic!("restored archive must not resolve")),Err("settlement_binding_mismatch"));
        db.execute("UPDATE auction_archive_scope SET scope=?1",[&binding.scope]).unwrap();
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
        let stamp:String = db.query_row("SELECT acknowledged_at FROM auction_settlement_receipts WHERE sequence=3",[],|r|r.get(0)).unwrap();
        assert_eq!(stamp,"2026-11-25T04:02:00.000Z");
        assert_eq!(db.query_row("SELECT COUNT(*) FROM sales",[],|r|r.get::<_,u64>(0)).unwrap(),0);

        // Exercise explicit operations on the same signed fixture and archive.
        let poll_request = || serde_json::from_value(json!({"action":"pollSettlement",
            "artworkId":binding.local_work,"scope":binding.scope})).unwrap();
        let history_request = || serde_json::from_value(json!({"action":"historySettlement",
            "artworkId":binding.local_work,"scope":binding.scope})).unwrap();
        let ack_request = |sequence: u64| serde_json::from_value(json!({"action":"ackSettlement",
            "artworkId":binding.local_work,"scope":binding.scope,"expectedSequence":sequence})).unwrap();
        let listener = {
            let mut active = Some(resource_http::NativePreviewSession::new(
                resource_http::NativeAuctionsHttp::new(addr).unwrap(),verified_resource_token(),NOW).unwrap());
            let claims = verified_claims(signed,&keys,GrantClient::AuctionsSettlement,NOW).unwrap();
            let mut scoped = Some(NativeAccessToken { bearer:signed.to_owned(),
                client:GrantClient::AuctionsSettlement, expires_at:claims.exp, subject:claims.sub });
            use super::resource_http::dispatch_workspace_isolated as dispatch;
            tauri::async_runtime::block_on(async {
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,poll_request(),||Ok(NOW),
                    |_| Err("settlement_access_denied")).await.unwrap_err(),"settlement_access_denied");
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(2),||Ok(NOW),verify)
                    .await.unwrap_err(),"settlement_stale_ack");
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(3),||Ok(NOW),verify)
                    .await.unwrap(),summary);
            });
            assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
            let bearer = signed.to_owned();
            let worker = thread::spawn(move || {
                let poll = json!({"action":"poll","auctionId":"fixture-auction","tenantId":"fixture-tenant"});
                let history = |after: u64| json!({"action":"history","auctionId":"fixture-auction","tenantId":"fixture-tenant","afterSequence":after});
                let ack = |sequence: u64| json!({"action":"ack","auctionId":"fixture-auction","tenantId":"fixture-tenant",
                    "resultId":"fixture-auction:1:fulfillment","deliverySequence":sequence});
                let fourth = ledger::delivery(4,"review_required");
                let fifth = ledger::delivery(5,"review_required");
                let mut acknowledged = fourth.clone();
                acknowledged["acknowledged_at"] = json!("2026-11-25T04:03:00.000Z");
                let mut fifth_ack = fifth.clone();
                fifth_ack["acknowledged_at"] = json!("2026-11-25T04:04:00.000Z");
                let mut confirmation = ledger::reply(fifth_ack);
                confirmation["replayed"] = json!(false);
                // The fixture records the fourth ACK but deliberately drops
                // the HTTP response. A subsequent delivery makes ACK(4) stale.
                // Only the explicit overlapping history read returns its stamp.
                for (expected,status,body) in [
                    (history(1),"200 OK",Some(ledger::page(vec![ledger::delivery(2,"fulfilled"),
                        ledger::delivery(3,"review_required"),fourth.clone()],4,None))),
                    (poll.clone(),"200 OK",Some(ledger::reply(fourth))),
                    (ack(4),"",None),
                    (ack(4),"409 Conflict",Some(json!({"ok":false,"code":"stale"}))),
                    (poll, "200 OK",Some(ledger::reply(fifth.clone()))),
                    (history(3),"200 OK",Some(ledger::page(vec![acknowledged.clone(),fifth],5,None))),
                    (history(3),"200 OK",Some(ledger::page(vec![acknowledged],4,None))),
                    (ack(5),"200 OK",Some(confirmation)),
                ] {
                    let mut stream = accept(&listener);
                    let bytes = read_resource_request(&mut stream,
                        "POST /api/studio/settlement/delivery HTTP/1.1",Some(&bearer),None);
                    assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(),expected);
                    if let Some(body) = body { reply(&mut stream,status,&body); }
                    // None: response lost, no implicit retry permitted.
                }
                listener
            });
            tauri::async_runtime::block_on(async {
                let updated = dispatch(&mut active,&mut scoped,&mut db,history_request(),||Ok(NOW),verify).await.unwrap();
                assert_eq!(updated["sequence"],4);
                assert!(updated["acknowledgedAt"].is_null());
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,poll_request(),||Ok(NOW),verify)
                    .await.unwrap(),updated);
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(3),||Ok(NOW),verify)
                    .await.unwrap_err(),"settlement_stale_ack");
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(4),||Ok(NOW),verify)
                    .await.unwrap_err(),"oauth_http_failed");
                assert_eq!(ledger::summary(&mut db,&binding).unwrap().unwrap().acknowledged_at,None);
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(4),||Ok(NOW),verify)
                    .await.unwrap_err(),"settlement_conflict");
                let fifth = dispatch(&mut active,&mut scoped,&mut db,poll_request(),||Ok(NOW),verify).await.unwrap();
                assert_eq!(fifth["sequence"],5);
                assert!(fifth["acknowledgedAt"].is_null());
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,history_request(),||Ok(NOW),verify)
                    .await.unwrap(),fifth);
                let stamp: Option<String> = db.query_row("SELECT acknowledged_at FROM auction_settlement_receipts WHERE sequence=4",[],|r|r.get(0)).unwrap();
                assert_eq!(stamp.as_deref(),Some("2026-11-25T04:03:00.000Z"));
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,history_request(),||Ok(NOW),verify)
                    .await.unwrap_err(),"settlement_invalid_history");
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(4),||Ok(NOW),verify)
                    .await.unwrap_err(),"settlement_stale_ack");
                let ack = dispatch(&mut active,&mut scoped,&mut db,ack_request(5),||Ok(NOW),verify).await.unwrap();
                assert_eq!(ack["acknowledgedAt"],"2026-11-25T04:04:00.000Z");
                assert_eq!(dispatch(&mut active,&mut scoped,&mut db,ack_request(5),||Ok(NOW),verify)
                    .await.unwrap(),ack);
            });
            worker.join().unwrap()
        };
        drop(db);
        let archive = repertica_studio_lib::db::open(dir.path().to_path_buf()).unwrap();
        let mut db = archive.conn.into_inner().unwrap();
        let summary = serde_json::to_value(ledger::summary(&mut db,&binding).unwrap()).unwrap();
        assert_eq!(ledger::count(&db),5);
        assert_eq!(summary["sequence"],5);
        assert_eq!(summary["acknowledgedAt"],"2026-11-25T04:04:00.000Z");
        let recovered: String = db.query_row("SELECT acknowledged_at FROM auction_settlement_receipts WHERE sequence=4",[],|r|r.get(0)).unwrap();
        assert_eq!(recovered,"2026-11-25T04:03:00.000Z");
        // Same typed workspace dispatch as the profile UI; the settlement
        // grant is independent and the verifier is native-only.
        use super::resource_http::{dispatch_workspace_isolated, NativePreviewSession};
        let mut active = Some(NativePreviewSession::new(resource,profile,NOW).unwrap());
        let mut settlement = None;
        tauri::async_runtime::block_on(async {
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(NOW), |_| panic!("missing grant must not resolve"))
                .await.unwrap_err(),"settlement_access_denied");
            settlement = Some(verified_resource_token());
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(NOW), |_| panic!("wrong scope must not resolve"))
                .await.unwrap_err(),"settlement_access_denied");
            assert!(settlement.is_none());
            settlement = Some(result);
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(NOW), |_| Err("settlement_access_denied"))
                .await.unwrap_err(),"settlement_access_denied");
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(NOW),verify).await.unwrap(),summary);
            // Archive switching must not leak a cached result through IPC.
            db.execute("UPDATE auction_archive_scope SET scope='cccccccc-cccc-4ccc-8ccc-cccccccccccc'",[]).unwrap();
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(NOW), |_| panic!("wrong archive must not resolve"))
                .await.unwrap_err(),"settlement_binding_mismatch");
            db.execute("UPDATE auction_archive_scope SET scope=?1",[&binding.scope]).unwrap();
            let disconnect = serde_json::from_value(json!({"action":"disconnect"})).unwrap();
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                disconnect,||Ok(NOW), |_| panic!("disconnect must not resolve"))
                .await.unwrap(),json!({"status":"disconnected"}));
            assert!(active.is_none() && settlement.is_none());
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(NOW), |_| panic!("disconnected must not resolve"))
                .await.unwrap_err(),"settlement_access_denied");

            // Reconstitute the fixture token only from verified signed claims,
            // like verified_resource_token above; do not contact the server.
            let claims = verified_claims(signed,&keys,GrantClient::AuctionsSettlement,NOW).unwrap();
            settlement = Some(NativeAccessToken { bearer:signed.to_owned(),
                client:GrantClient::AuctionsSettlement, expires_at:claims.exp, subject:claims.sub });
            active = Some(NativePreviewSession::new(
                resource_http::NativeAuctionsHttp::new(addr).unwrap(),verified_resource_token(),NOW).unwrap());
            let time = std::cell::Cell::new(NOW);
            assert_eq!(dispatch_workspace_isolated(&mut active,&mut settlement,&mut db,
                request(),||Ok(time.get()), |candidate| {
                    verify(candidate)?;
                    time.set(NOW+900);
                    Ok(())
                }).await.unwrap_err(),"settlement_access_denied");
            assert!(active.is_none() && settlement.is_none());
        });
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
        assert_eq!(db.query_row("SELECT COUNT(*) FROM sales",[],|r|r.get::<_,u64>(0)).unwrap(),0);
    }

    #[test]
    fn disabled_and_non_loopback_providers_never_construct_http() {
        assert_eq!(NativeIdentityHttp::new(IdentityProvider::Disabled).err(), Some("oauth_provider_disabled"));
        assert_eq!(NativeIdentityHttp::new(IdentityProvider::IsolatedFixture).err(), Some("oauth_destination_denied"));
        let bad = SocketAddrV4::new(Ipv4Addr::new(192,0,2,1), 49152);
        assert_eq!(NativeIdentityHttp::new(IdentityProvider::IsolatedHttp(bad)).err(), Some("oauth_destination_denied"));
    }

    #[test]
    fn redirects_and_stalled_bodies_are_not_followed_or_retried() {
        let (redirect_target, target) = listen();
        let (listener, addr) = listen();
        let worker = thread::spawn(move || {
            let mut s = accept(&listener);
            read_request(&mut s, &format!("GET {DISCOVERY} HTTP/1.1"));
            write!(s,"HTTP/1.1 302 Found\r\nLocation: http://{target}{DISCOVERY}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        let error = tauri::async_runtime::block_on(async {
            NativeIdentityHttp::new(IdentityProvider::IsolatedHttp(addr)).unwrap()
                .exchange_with_clock(grant(), DEADLINE, || Ok(NOW)).await.err()
        });
        worker.join().unwrap();
        assert_eq!(error, Some("oauth_http_rejected"));
        assert_eq!(redirect_target.accept().err().unwrap().kind(), std::io::ErrorKind::WouldBlock);

        let (listener, addr) = listen();
        let worker = thread::spawn(move || {
            let mut s = accept(&listener);
            read_request(&mut s, &format!("GET {DISCOVERY} HTTP/1.1"));
            s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 200\r\nConnection: close\r\n\r\n{").unwrap();
            thread::sleep(Duration::from_millis(800));
        });
        let error = tauri::async_runtime::block_on(async {
            NativeIdentityHttp::new(IdentityProvider::IsolatedHttp(addr)).unwrap()
                .exchange_with_clock(grant(), Duration::from_millis(500), || Ok(NOW)).await.err()
        });
        worker.join().unwrap();
        assert_eq!(error, Some("oauth_timeout"));
    }

    const RESOURCE_KEY: &str = "synthetic-preview-key-001";
    const RESOURCE_BODY: &str = r#"{"tenantId":"tenant-test","slug":"art","title":"Art","description":"","startsAt":"2027-01-15T08:01:40.000Z","endsAt":"2027-01-16T08:01:40.000Z","startingAmount":1000,"minimumIncrementAmount":100,"currency":"EUR"}"#;
    const RESOURCE_ID: &str = "4c2a24d0-bf50-410a-b0f2-1c0b3c96e9a0";

    fn verified_resource_token() -> NativeAccessToken {
        let f = fixture();
        let keys: Jwks = serde_json::from_value(f["jwks"].clone()).unwrap();
        let bearer = f["valid"].as_str().unwrap().to_owned();
        let claims = verified_claims(&bearer,&keys,GrantClient::Auctions,NOW).unwrap();
        NativeAccessToken { bearer, client: GrantClient::Auctions, expires_at: claims.exp, subject: claims.sub }
    }
    fn tenants(role: &str) -> Value {
        json!({"month":"2027-01","monthlyPerArtistLimit":3,"globalMonthlyLimit":12,"tenants":[
            {"tenantId":"tenant-test","status":"active","role":role,"artistSlug":"artist",
             "artistName":"Synthetic artist","approved":1,"publishedThisMonth":1,"publicationEligible":role != "viewer"}]})
    }
    fn receipt(replayed: bool, bidding: bool) -> Value {
        json!({"auctionId":RESOURCE_ID,"slug":"artist--art","status":"preview",
            "biddingEnabled":bidding,"idempotentReplay":replayed})
    }
    fn resource_tenants(listener: &TcpListener, bearer: &str, role: &str) {
        let mut s = accept(listener);
        assert!(read_resource_request(&mut s,"GET /api/studio/tenants HTTP/1.1",Some(bearer),None).is_empty());
        reply(&mut s,"200 OK",&tenants(role));
    }

    #[test]
    fn native_resource_client_lists_and_replays_exact_preview_request() {
        use super::resource_http::NativeAuctionsHttp;
        let (listener, addr) = listen();
        let worker = thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            resource_tenants(&listener,bearer,"owner");
            let mut s=accept(&listener);
            assert!(read_resource_request(&mut s,"GET /api/studio/previews?tenantId=tenant-test HTTP/1.1",Some(bearer),None).is_empty());
            reply(&mut s,"200 OK",&json!({"auctions":[{"auctionId":RESOURCE_ID,"slug":"artist--art","title":"Art",
                "status":"preview","startsAt":"2027-01-15T08:01:40.000Z","endsAt":"2027-01-16T08:01:40.000Z"}]}));
            for (replayed,status) in [(false,"201 Created"),(true,"200 OK")] {
                resource_tenants(&listener,bearer,"owner");
                let mut s=accept(&listener);
                let body=read_resource_request(&mut s,"POST /api/studio/previews HTTP/1.1",Some(bearer),Some(RESOURCE_KEY));
                assert_eq!(body.as_slice(),RESOURCE_BODY.as_bytes());
                reply(&mut s,status,&receipt(replayed,false));
            }
            // A fresh read-only membership must stop a third create before POST.
            resource_tenants(&listener,bearer,"viewer");
            listener
        });
        tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            let list=api.list_tenants(&token,||Ok(NOW)).await.unwrap();
            assert_eq!(list.tenants[0].tenant_id,"tenant-test");
            let list=api.list_previews(&token,"tenant-test",||Ok(NOW)).await.unwrap();
            assert_eq!(list[0].auction_id,RESOURCE_ID);
            assert_eq!(list[0].status,"preview");
            let first=api.create_preview(&token,RESOURCE_KEY,RESOURCE_BODY,||Ok(NOW)).await.unwrap();
            // Explicit replay after the original start, while the signed token is still valid.
            let replay=api.create_preview(&token,RESOURCE_KEY,RESOURCE_BODY,||Ok(NOW+200)).await.unwrap();
            assert_eq!(first.auction_id,replay.auction_id);
            assert!(!first.idempotent_replay && replay.idempotent_replay);
            assert_eq!(api.create_preview(&token,RESOURCE_KEY,RESOURCE_BODY,||Ok(NOW)).await.err(),Some("auction_access_denied"));
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn native_resource_rejects_private_fields_expired_tokens_and_unconfirmed_receipts() {
        use super::resource_http::NativeAuctionsHttp;
        let (listener, addr)=listen();
        tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            assert!(NativeAuctionsHttp::new(SocketAddrV4::new(Ipv4Addr::new(192,0,2,1),49152)).is_err());
            let api=NativeAuctionsHttp::new(addr).unwrap();
            let mut private=RESOURCE_BODY.to_owned();
            private.pop();
            private.push_str(r#","collectorEmail":"private@example.test"}"#);
            assert_eq!(api.create_preview(&token,RESOURCE_KEY,&private,||Ok(NOW)).await.err(),Some("invalid_preview_payload"));
            assert_eq!(api.list_previews(&token,"tenant-test&other=1",||Ok(NOW)).await.err(),Some("invalid_tenant"));
            assert_eq!(api.list_tenants(&token,||Ok(NOW+900)).await.err(),Some("oauth_token_rejected"));
        });
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);

        let worker=thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            for (status,body) in [
                ("201 Created",receipt(false,true)),
                ("201 Created",receipt(true,false)),
                ("409 Conflict",json!({"error":"idempotency_conflict"}))
            ] {
                resource_tenants(&listener,bearer,"owner");
                let mut s=accept(&listener);
                assert_eq!(read_resource_request(&mut s,"POST /api/studio/previews HTTP/1.1",Some(bearer),Some(RESOURCE_KEY)).as_slice(),RESOURCE_BODY.as_bytes());
                reply(&mut s,status,&body);
            }
            listener
        });
        tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            for expected in ["auction_invalid_response","auction_invalid_response","auction_conflict"] {
                assert_eq!(api.create_preview(&token,RESOURCE_KEY,RESOURCE_BODY,||Ok(NOW)).await.err(),Some(expected));
            }
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn native_reconciliation_reuses_saved_bytes_and_keeps_receipt_after_reopen() {
        use super::resource_http::NativeAuctionsHttp;
        use rusqlite::{Connection,params};
        let (listener,addr)=listen();
        let worker=thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            // First response is unsafe, second is a successful idempotent replay.
            for (status,body) in [("201 Created",receipt(false,true)),("200 OK",receipt(true,false))] {
                resource_tenants(&listener,bearer,"owner");
                let mut s=accept(&listener);
                assert_eq!(read_resource_request(&mut s,"POST /api/studio/previews HTTP/1.1",Some(bearer),Some(RESOURCE_KEY)).as_slice(),RESOURCE_BODY.as_bytes());
                reply(&mut s,status,&body);
            }
            listener
        });
        let dir=tempfile::tempdir().unwrap();
        let file=dir.path().join("isolated-preview.sqlite");
        let scope=uuid::Uuid::new_v4().to_string();
        let revision=uuid::Uuid::new_v4().to_string();
        let raw=json!({"version":1,"scope":scope,"draft":{"artworkId":"a","title":"Art","description":"",
            "slug":"art","startsAt":"2027-01-15T08:01:40.000Z","endsAt":"2027-01-16T08:01:40.000Z",
            "startingAmount":1000,"minimumIncrementAmount":100},
            "pending":{"artworkId":"a","idempotencyKey":RESOURCE_KEY,"body":RESOURCE_BODY}}).to_string();
        let mut conn=Connection::open(&file).unwrap();
        conn.execute_batch("CREATE TABLE auction_archive_scope(singleton INTEGER PRIMARY KEY,scope TEXT NOT NULL);
            CREATE TABLE auction_drafts(artwork_id TEXT PRIMARY KEY,revision TEXT NOT NULL,record_json TEXT NOT NULL);").unwrap();
        conn.execute("INSERT INTO auction_archive_scope VALUES(1,?1)",[&scope]).unwrap();
        conn.execute("INSERT INTO auction_drafts VALUES('a',?1,?2)",params![revision,raw]).unwrap();
        let confirmed=tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            assert!(api.reconcile_preview(&mut conn,&token,"a",&uuid::Uuid::new_v4().to_string(),&revision,||Ok(NOW)).await.is_err());
            assert_eq!(api.reconcile_preview(&mut conn,&token,"a",&scope,&revision,||Ok(NOW)).await.unwrap_err(),"auction_invalid_response");
            assert_eq!(conn.query_row("SELECT record_json FROM auction_drafts",[],|r|r.get::<_,String>(0)).unwrap(),raw);
            api.reconcile_preview(&mut conn,&token,"a",&scope,&revision,||Ok(NOW+200)).await.unwrap()
        });
        drop(conn);
        let mut conn=Connection::open(&file).unwrap();
        let (new_revision, stored):(String,String)=conn.query_row("SELECT revision,record_json FROM auction_drafts",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_ne!(revision,new_revision);
        let value:Value=serde_json::from_str(&stored).unwrap();
        assert_eq!(value["pending"]["body"],RESOURCE_BODY);
        assert_eq!(value["receipt"]["auctionId"],RESOURCE_ID);
        tauri::async_runtime::block_on(async {
            let api=NativeAuctionsHttp::new(addr).unwrap();
            let token=verified_resource_token();
            assert_eq!(api.reconcile_preview(&mut conn,&token,"a",&scope,&new_revision,||Ok(NOW)).await.unwrap(),confirmed);
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn native_pkce_preparation_and_explicit_reconciliation_preserve_exact_bytes() {
        use super::resource_http::{NativeAuctionsHttp, NativePreviewSession, dispatch_isolated};
        use rusqlite::{Connection,params};
        let (listener,addr)=listen();
        // Simulate the browser only at the injected opener boundary. The native
        // callback really binds a fresh loopback port, validates state/Host and
        // closes its listener before the HTTP exchange. No platform opener.
        let session = super::super::LoopbackSession::bind(GrantClient::Auctions, Duration::from_secs(5)).unwrap();
        let callback_addr = session.listener.local_addr().unwrap();
        let state = session.grant.state.clone();
        let verifier = session.grant.verifier.as_ref().unwrap().clone();
        let challenge = super::super::challenge(&verifier);
        let grant = session.open_and_wait(&std::sync::atomic::AtomicBool::new(false),
            IdentityProvider::IsolatedHttp(addr), |url| {
                assert!(url.starts_with(&format!("http://{addr}/login?")));
                assert!(url.contains("client_id=repertica-desktop-auctions&"));
                assert!(url.contains(&format!("state={state}&")));
                assert!(url.contains(&format!("code_challenge={challenge}&code_challenge_method=S256")));
                let mut stream = TcpStream::connect(callback_addr).unwrap();
                stream.write_all(format!(
                    "GET {CALLBACK_PATH}?code={CODE}&state={state} HTTP/1.1\r\nHost: {callback_addr}\r\n\r\n"
                ).as_bytes()).unwrap();
                Ok(())
            }).unwrap();
        assert_eq!(grant.expected_audience(), "repertica-auctions");
        assert!(TcpStream::connect(callback_addr).is_err());
        let expected_grant = json!({"grant_type":"authorization_code",
            "client_id":"repertica-desktop-auctions",
            "redirect_uri":format!("http://{callback_addr}{CALLBACK_PATH}"),
            "code":CODE,"code_verifier":verifier});
        let (key_tx, key_rx) = std::sync::mpsc::channel::<String>();
        let worker=thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            // Use the actual native exchange, discovery and signed JWKS before
            // the resource calls, rather than injecting a pre-verified token.
            for (path, body) in [(DISCOVERY, discovery()), (JWKS, f["jwks"].clone())] {
                let mut s = accept(&listener);
                assert!(read_request(&mut s, &format!("GET {path} HTTP/1.1")).is_empty());
                reply(&mut s, "200 OK", &body);
            }
            let mut s = accept(&listener);
            let body: Value = serde_json::from_slice(&read_request(&mut s, &format!("POST {TOKEN} HTTP/1.1"))).unwrap();
            assert_eq!(body, expected_grant);
            reply(&mut s, "200 OK", &json!({"access_token":bearer,"token_type":"Bearer","expires_in":900,"scope":"profile"}));
            drop(s);
            let mut private = tenants("owner");
            private["tenants"][0]["publicationEligible"] = json!(false);
            let mut s = accept(&listener);
            assert!(read_resource_request(&mut s,"GET /api/studio/tenants HTTP/1.1",Some(bearer),None).is_empty());
            reply(&mut s,"200 OK",&private);
            drop(s);
            for role in ["viewer","owner","owner"] { resource_tenants(&listener,bearer,role); }
            // The chooser and selection check fresh memberships. Unknown tenant
            // must not cause a preview request. Viewer can read only.
            resource_tenants(&listener,bearer,"owner"); // refresh
            resource_tenants(&listener,bearer,"owner"); // unknown tenant
            for role in ["viewer","owner"] {
                resource_tenants(&listener,bearer,role);
                let mut s=accept(&listener);
                assert!(read_resource_request(&mut s,"GET /api/studio/previews?tenantId=tenant-test HTTP/1.1",Some(bearer),None).is_empty());
                reply(&mut s,"200 OK",&json!({"auctions":[{"auctionId":RESOURCE_ID,"slug":"artist--art","title":"Art",
                    "status":"preview","startsAt":"2027-01-15T08:01:40.000Z","endsAt":"2027-01-16T08:01:40.000Z"}]}));
            }
            resource_tenants(&listener,bearer,"owner"); // confirm prepare
            // Only the key generated and persisted by native preparation is
            // accepted by the fixture; explicit reconciliation follows.
            let key = key_rx.recv_timeout(Duration::from_secs(15)).unwrap();
            resource_tenants(&listener,bearer,"owner");
            let mut s = accept(&listener);
            assert_eq!(read_resource_request(&mut s,"POST /api/studio/previews HTTP/1.1",Some(bearer),Some(&key)).as_slice(),RESOURCE_BODY.as_bytes());
            reply(&mut s,"201 Created",&receipt(false,false));
            listener
        });
        let mut conn=Connection::open_in_memory().unwrap();
        let scope=uuid::Uuid::new_v4().to_string();
        let revision=uuid::Uuid::new_v4().to_string();
        let raw=json!({"version":1,"scope":scope,"draft":{"artworkId":"a","title":"Art","description":"",
            "slug":"art","startsAt":"2027-01-15T08:01:40.000Z","endsAt":"2027-01-16T08:01:40.000Z",
            "startingAmount":1000,"minimumIncrementAmount":100},"pending":null}).to_string();
        conn.execute_batch("CREATE TABLE auction_archive_scope(singleton INTEGER PRIMARY KEY,scope TEXT NOT NULL);
            CREATE TABLE auction_drafts(artwork_id TEXT PRIMARY KEY,revision TEXT NOT NULL,record_json TEXT NOT NULL);
            CREATE TABLE artworks(id TEXT PRIMARY KEY,status TEXT NOT NULL);
            INSERT INTO artworks VALUES('a','available');").unwrap();
        conn.execute("INSERT INTO auction_archive_scope VALUES(1,?1)",[&scope]).unwrap();
        conn.execute("INSERT INTO auction_drafts VALUES('a',?1,?2)",params![revision,raw]).unwrap();
        fn stored(c: &Connection) -> (String,String) {
            c.query_row("SELECT revision,record_json FROM auction_drafts",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap()
        }
        tauri::async_runtime::block_on(async {
            let token = NativeIdentityHttp::new(IdentityProvider::IsolatedHttp(addr)).unwrap()
                .exchange_with_clock(grant, DEADLINE, || Ok(NOW)).await.unwrap();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            let altered=RESOURCE_BODY.replace(r#""title":"Art""#,r#""title":"Other""#);
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,&altered,||Ok(NOW)).await.unwrap_err(),"preview_draft_changed");
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,RESOURCE_BODY,||Ok(NOW+200)).await.unwrap_err(),"invalid_schedule");
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&uuid::Uuid::new_v4().to_string(),RESOURCE_BODY,||Ok(NOW)).await.unwrap_err(),"auction_draft_conflict");
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,RESOURCE_BODY,||Ok(NOW)).await.unwrap_err(),"auction_access_denied");
            // An approved but private profile must also deny preparation.
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,RESOURCE_BODY,||Ok(NOW)).await.unwrap_err(),"auction_access_denied");
            let other=RESOURCE_BODY.replace("tenant-test","tenant-other");
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,&other,||Ok(NOW)).await.unwrap_err(),"auction_access_denied");
            assert_eq!(stored(&conn),(revision.clone(),raw.clone()));
            conn.execute_batch("CREATE TRIGGER ignore_preview BEFORE UPDATE ON auction_drafts BEGIN SELECT RAISE(IGNORE); END;").unwrap();
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,RESOURCE_BODY,||Ok(NOW)).await.unwrap_err(),"auction_draft_write_failed");
            assert_eq!(stored(&conn),(revision.clone(),raw.clone()));
            conn.execute_batch("DROP TRIGGER ignore_preview").unwrap();
            // The orchestration owns the verified profile token and one-shot review.
            let mut session = NativePreviewSession::new(api, token, NOW).unwrap();
            assert_eq!(session.review_new(&conn,"a",&scope,&revision,RESOURCE_BODY,NOW).err().as_deref(),Some("auction_access_denied"));
            let orgs = serde_json::to_value(session.organizations(||Ok(NOW)).await.unwrap()).unwrap();
            assert_eq!(orgs, json!([{"tenantId":"tenant-test","artistName":"Synthetic artist","role":"owner",
                "status":"active","publicationEligible":true,"publishedThisMonth":1}]));
            assert_eq!(session.select_organization("tenant-test&other=1",||Ok(NOW)).await.err(),Some("invalid_tenant"));
            assert_eq!(session.select_organization("tenant-other",||Ok(NOW)).await.err(),Some("auction_access_denied"));
            let viewer=serde_json::to_value(session.select_organization("tenant-test",||Ok(NOW)).await.unwrap()).unwrap();
            assert_eq!(viewer["organization"]["publicationEligible"],false);
            assert_eq!(viewer["previews"][0]["auctionId"],RESOURCE_ID);
            assert_eq!(session.review_new(&conn,"a",&scope,&revision,RESOURCE_BODY,NOW).err().as_deref(),Some("auction_access_denied"));
            let owner=serde_json::to_value(session.select_organization("tenant-test",||Ok(NOW)).await.unwrap()).unwrap();
            assert_eq!(owner["organization"]["publicationEligible"],true);
            assert_eq!(owner["previews"][0]["status"],"preview");
            assert_eq!(owner["previews"][0].as_object().unwrap().len(),6);
            let other=RESOURCE_BODY.replace("tenant-test","tenant-other");
            assert_eq!(session.review_new(&conn,"a",&scope,&revision,&other,NOW).err().as_deref(),Some("auction_access_denied"));
            assert_eq!(session.confirm_prepare(&mut conn,"unreviewed",||Ok(NOW)).await.unwrap_err(),"auction_review_required");
            let review = session.review_new(&conn,"a",&scope,&revision,RESOURCE_BODY,NOW).unwrap();
            assert_eq!(review.public_body, RESOURCE_BODY);
            assert_eq!(review.action, "prepare");
            assert_eq!(stored(&conn),(revision.clone(),raw.clone()));
            let projection = serde_json::to_value(&review).unwrap();
            assert_eq!(projection.as_object().unwrap().len(),3);
            assert!(projection.get("idempotencyKey").is_none());
            // A confirmation of a different operation invalidates the review.
            assert_eq!(session.confirm_reconcile(&mut conn,&review.confirmation_id,||Ok(NOW)).await.err().as_deref(),Some("auction_review_required"));
            assert_eq!(session.confirm_prepare(&mut conn,&review.confirmation_id,||Ok(NOW)).await.unwrap_err(),"auction_review_required");
            let review = session.review_new(&conn,"a",&scope,&revision,RESOURCE_BODY,NOW).unwrap();
            session.cancel_review();
            assert_eq!(session.confirm_prepare(&mut conn,&review.confirmation_id,||Ok(NOW)).await.unwrap_err(),"auction_review_required");
            let review = session.review_new(&conn,"a",&scope,&revision,RESOURCE_BODY,NOW).unwrap();
            let changed_revision=uuid::Uuid::new_v4().to_string();
            conn.execute("UPDATE auction_drafts SET revision=?1 WHERE artwork_id='a'",[&changed_revision]).unwrap();
            assert_eq!(session.confirm_prepare(&mut conn,&review.confirmation_id,||Ok(NOW)).await.unwrap_err(),"auction_draft_conflict");
            conn.execute("UPDATE auction_drafts SET revision=?1 WHERE artwork_id='a'",[&revision]).unwrap();
            // Same serialized request shape as the release Tauri command,
            // but only this native fixture may own a connected session.
            let mut active = Some(session);
            let status = dispatch_isolated(&mut active,&mut conn,serde_json::from_value(json!({"action":"status"})).unwrap(),||Ok(NOW)).await.unwrap();
            assert_eq!(status,json!({"status":"connected"}));
            let review = dispatch_isolated(&mut active,&mut conn,serde_json::from_value(json!({
                "action":"reviewNew","artworkId":"a","scope":scope,"revision":revision,"publicBody":RESOURCE_BODY
            })).unwrap(),||Ok(NOW)).await.unwrap();
            assert_eq!(review["publicBody"],RESOURCE_BODY);
            assert_eq!(review["action"],"prepare");
            let confirm = json!({"action":"confirmPrepare","confirmationId":review["confirmationId"]});
            let prepared = dispatch_isolated(&mut active,&mut conn,serde_json::from_value(confirm.clone()).unwrap(),||Ok(NOW)).await.unwrap();
            let next=prepared["revision"].as_str().unwrap().to_owned();
            assert_eq!(dispatch_isolated(&mut active,&mut conn,serde_json::from_value(confirm).unwrap(),||Ok(NOW)).await.unwrap_err(),"auction_review_required");
            let mut session=active.take().unwrap();
            let (actual,record)=stored(&conn);
            assert_ne!(revision,next);
            assert_eq!(actual,next);
            let value:Value=serde_json::from_str(&record).unwrap();
            assert_eq!(value["pending"]["body"],RESOURCE_BODY);
            assert_eq!(value["pending"]["artworkId"],"a");
            assert!(uuid::Uuid::parse_str(value["pending"]["idempotencyKey"].as_str().unwrap()).is_ok());
            assert!(value.get("receipt").is_none());
            assert_eq!(session.review_new(&conn,"a",&scope,&next,RESOURCE_BODY,NOW).err().as_deref(),Some("pending_preview_unresolved"));
            assert_eq!(stored(&conn),(actual.clone(),record.clone()));
            key_tx.send(value["pending"]["idempotencyKey"].as_str().unwrap().to_owned()).unwrap();
            let review = session.review_pending(&conn,"a",&scope,&next,NOW).unwrap();
            assert_eq!(review.public_body, RESOURCE_BODY);
            assert_eq!(review.action, "reconcile");
            assert_eq!(stored(&conn),(actual.clone(),record.clone()));
            let mut active=Some(session);
            let confirm=json!({"action":"confirmReconcile","confirmationId":review.confirmation_id});
            let confirmed=dispatch_isolated(&mut active,&mut conn,serde_json::from_value(confirm.clone()).unwrap(),||Ok(NOW)).await.unwrap();
            assert_eq!(dispatch_isolated(&mut active,&mut conn,serde_json::from_value(confirm.clone()).unwrap(),||Ok(NOW)).await.unwrap_err(),"auction_review_required");
            let mut session=active.take().unwrap();
            let (confirmed_revision, confirmed_record) = stored(&conn);
            assert_ne!(confirmed_revision, next);
            let saved: Value = serde_json::from_str(&confirmed_record).unwrap();
            assert_eq!(saved["pending"], value["pending"]);
            assert_eq!(saved["receipt"]["auctionId"], RESOURCE_ID);
            assert_eq!(session.review_pending(&conn,"a",&scope,&confirmed_revision,NOW).err().as_deref(),Some("auction_already_confirmed"));
            assert_eq!(serde_json::to_value(&confirmed).unwrap(),
                json!({"tenantId":"tenant-test","auctionId":RESOURCE_ID,"slug":"artist--art"}));
            session.clear_selection();
            assert_eq!(session.review_new(&conn,"a",&scope,&confirmed_revision,RESOURCE_BODY,NOW).err().as_deref(),Some("auction_access_denied"));
            assert_eq!(stored(&conn),(confirmed_revision.clone(),confirmed_record.clone()));
            let mut active=Some(session);
            let disconnected=dispatch_isolated(&mut active,&mut conn,serde_json::from_value(json!({"action":"disconnect"})).unwrap(),||Ok(NOW)).await.unwrap();
            assert_eq!(disconnected,json!({"status":"disconnected"}));
            assert!(active.is_none());
            assert_eq!(dispatch_isolated(&mut active,&mut conn,serde_json::from_value(confirm).unwrap(),||Ok(NOW)).await.unwrap_err(),"auction_not_connected");
            assert_eq!(stored(&conn),(confirmed_revision,confirmed_record));
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn native_revoked_session_preserves_reviewed_draft_and_pending_request() {
        use super::resource_http::NativeAuctionsHttp;
        use rusqlite::{Connection,params};
        // Independent denial scenarios: no revoked token is later treated as
        // reauthorized. A signed, unexpired fixture is not proof of live status.
        let (listener,addr)=listen();
        let worker=thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            let mut s=accept(&listener);
            assert!(read_resource_request(&mut s,"GET /api/studio/tenants HTTP/1.1",Some(bearer),None).is_empty());
            reply(&mut s,"401 Unauthorized",&json!({"error":"synthetic_revoked_session"}));
            listener
        });
        let mut conn=Connection::open_in_memory().unwrap();
        let scope=uuid::Uuid::new_v4().to_string();
        let revision=uuid::Uuid::new_v4().to_string();
        let raw=json!({"version":1,"scope":scope,"draft":{"artworkId":"a","title":"Art","description":"",
            "slug":"art","startsAt":"2027-01-15T08:01:40.000Z","endsAt":"2027-01-16T08:01:40.000Z",
            "startingAmount":1000,"minimumIncrementAmount":100},"pending":null}).to_string();
        conn.execute_batch("CREATE TABLE auction_archive_scope(singleton INTEGER PRIMARY KEY,scope TEXT NOT NULL);
            CREATE TABLE auction_drafts(artwork_id TEXT PRIMARY KEY,revision TEXT NOT NULL,record_json TEXT NOT NULL);
            CREATE TABLE artworks(id TEXT PRIMARY KEY,status TEXT NOT NULL);
            INSERT INTO artworks VALUES('a','available');").unwrap();
        conn.execute("INSERT INTO auction_archive_scope VALUES(1,?1)",[&scope]).unwrap();
        conn.execute("INSERT INTO auction_drafts VALUES('a',?1,?2)",params![revision,raw]).unwrap();
        fn stored(c: &Connection) -> (String,String) {
            c.query_row("SELECT revision,record_json FROM auction_drafts",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap()
        }
        tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            assert_eq!(api.prepare_reviewed_preview(&mut conn,&token,"a",&scope,&revision,RESOURCE_BODY,||Ok(NOW)).await.unwrap_err(),"auction_access_denied");
            assert_eq!(stored(&conn),(revision.clone(),raw.clone()));
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
        drop(listener);

        // Separate fixture: the request had been explicitly prepared before
        // revocation. Membership GET succeeds, then the POST is denied.
        let mut pending:Value=serde_json::from_str(&raw).unwrap();
        pending["pending"]=json!({"artworkId":"a","idempotencyKey":RESOURCE_KEY,"body":RESOURCE_BODY});
        let pending=pending.to_string();
        let next=uuid::Uuid::new_v4().to_string();
        conn.execute("UPDATE auction_drafts SET revision=?1,record_json=?2 WHERE artwork_id='a'",params![next,pending]).unwrap();
        let (listener,addr)=listen();
        let worker=thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            resource_tenants(&listener,bearer,"owner");
            let mut s=accept(&listener);
            assert_eq!(read_resource_request(&mut s,"POST /api/studio/previews HTTP/1.1",Some(bearer),Some(RESOURCE_KEY)).as_slice(),RESOURCE_BODY.as_bytes());
            reply(&mut s,"403 Forbidden",&json!({"error":"synthetic_revoked_session"}));
            listener
        });
        tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            assert_eq!(api.reconcile_preview(&mut conn,&token,"a",&scope,&next,||Ok(NOW)).await.unwrap_err(),"auction_access_denied");
            assert_eq!(stored(&conn),(next,pending));
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn native_resource_requires_server_publication_eligibility() {
        use super::resource_http::NativeAuctionsHttp;
        let (listener, addr) = listen();
        let worker = thread::spawn(move || {
            let f = fixture();
            let bearer = f["valid"].as_str().unwrap();
            let mut private = tenants("owner");
            private["tenants"][0]["publicationEligible"] = json!(false);
            let mut suspended = tenants("owner");
            suspended["tenants"][0]["status"] = json!("suspended");
            suspended["tenants"][0]["publicationEligible"] = json!(false);
            let mut absent_profile = tenants("editor");
            absent_profile["tenants"][0]["artistSlug"] = Value::Null;
            absent_profile["tenants"][0]["artistName"] = Value::Null;
            absent_profile["tenants"][0]["approved"] = Value::Null;
            absent_profile["tenants"][0]["publicationEligible"] = json!(false);
            let mut missing = tenants("owner");
            missing["tenants"][0].as_object_mut().unwrap().remove("publicationEligible");
            let mut numeric = tenants("owner");
            numeric["tenants"][0]["publicationEligible"] = json!(1);
            let mut inconsistent = tenants("viewer");
            inconsistent["tenants"][0]["publicationEligible"] = json!(true);
            for body in [private.clone(), suspended, absent_profile, tenants("viewer"),
                inconsistent, missing, numeric, private, tenants("editor")] {
                let mut s = accept(&listener);
                assert!(read_resource_request(&mut s,"GET /api/studio/tenants HTTP/1.1",Some(bearer),None).is_empty());
                reply(&mut s,"200 OK",&body);
            }
            listener
        });
        tauri::async_runtime::block_on(async {
            let token = verified_resource_token();
            let api = NativeAuctionsHttp::new(addr).unwrap();
            for _ in 0..5 {
                assert!(api.eligible_tenants(&token, || Ok(NOW)).await.unwrap().is_empty());
            }
            for _ in 0..2 {
                assert_eq!(api.eligible_tenants(&token, || Ok(NOW)).await.err(), Some("oauth_invalid_response"));
            }
            assert_eq!(api.create_preview(&token, RESOURCE_KEY, RESOURCE_BODY, || Ok(NOW)).await.err(),
                Some("auction_access_denied"));
            assert_eq!(api.eligible_tenants(&token, || Ok(NOW)).await.unwrap(), vec!["tenant-test".to_owned()]);
        });
        let listener = worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }

    #[test]
    fn native_resource_post_redirect_never_forwards_bearer_or_body() {
        use super::resource_http::NativeAuctionsHttp;
        let (redirect_target,target)=listen();
        let (listener,addr)=listen();
        let worker=thread::spawn(move || {
            let f=fixture();
            let bearer=f["valid"].as_str().unwrap();
            resource_tenants(&listener,bearer,"owner");
            let mut s=accept(&listener);
            assert_eq!(read_resource_request(&mut s,"POST /api/studio/previews HTTP/1.1",Some(bearer),Some(RESOURCE_KEY)).as_slice(),RESOURCE_BODY.as_bytes());
            write!(s,"HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{target}/api/studio/previews\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            listener
        });
        tauri::async_runtime::block_on(async {
            let token=verified_resource_token();
            let api=NativeAuctionsHttp::new(addr).unwrap();
            assert_eq!(api.create_preview(&token,RESOURCE_KEY,RESOURCE_BODY,||Ok(NOW)).await.err(),Some("auction_http_rejected"));
        });
        let listener=worker.join().unwrap();
        assert_eq!(listener.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
        assert_eq!(redirect_target.accept().err().unwrap().kind(),std::io::ErrorKind::WouldBlock);
    }
}
