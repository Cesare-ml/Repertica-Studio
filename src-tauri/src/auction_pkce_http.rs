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
    audience: &'static str,
    expires_at: u64,
}

impl NativeAccessToken {
    pub(super) fn bearer_for(&self, client: GrantClient, now: u64) -> Result<&str, Failure> {
        if self.audience != client.audience() || now >= self.expires_at {
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
    let mut response = request.timeout(remaining(deadline)?).send().await.map_err(http_error)?;
    // Do not parse an error/redirect body or follow a Location.
    if response.status() != StatusCode::OK { return Err("oauth_http_rejected"); }
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
    serde_json::from_slice(&bytes).map_err(|_| "oauth_invalid_response")
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
}

fn decode(segment: &str) -> Result<Vec<u8>, Failure> {
    if segment.is_empty() { return Err("oauth_token_rejected"); }
    URL_SAFE_NO_PAD.decode(segment).map_err(|_| "oauth_token_rejected")
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 80 &&
        value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn verified_expiry(token: &str, jwks: &Jwks, client: GrantClient, now: u64) -> Result<u64, Failure> {
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
        subject.is_none() || !safe_identifier(&claims.jti) || claims.iat > now || claims.exp <= now ||
        claims.exp.checked_sub(claims.iat).is_none_or(|ttl| ttl == 0 || ttl > 900) ||
        claims.nbf.is_some_and(|nbf| nbf > now) {
        return Err("oauth_claims_rejected");
    }
    Ok(claims.exp)
}

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
        // The updater unifies reqwest with rustls-no-provider. Explicitly select
        // ring for this test process before building even a loopback HTTP client.
        // Concurrent tests may have already installed the same default.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = Client::builder()
            .no_proxy().redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
            .referer(false).http1_only().connect_timeout(Duration::from_secs(2)).timeout(DEADLINE)
            .build().map_err(|_| "oauth_http_failed")?;
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
        let expires_at = verified_expiry(&reply.access_token, &keys, grant.expected_client, clock()?)?;
        remaining(deadline)?;
        Ok(NativeAccessToken { bearer: reply.access_token, audience: grant.expected_audience(), expires_at })
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
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim_end(), expected);
        let mut size = 0usize;
        for _ in 0..64 {
            line.clear();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            if line == "\r\n" { break; }
            assert!(!line.to_ascii_lowercase().starts_with("origin:"));
            assert!(!line.to_ascii_lowercase().starts_with("authorization:"));
            if let Some((name,value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") { size = value.trim().parse().unwrap(); }
            }
        }
        assert!(size < 8192);
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
        assert_eq!(result.bearer_for(GrantClient::Auctions, NOW+900), Err("oauth_token_rejected"));
    }

    #[test]
    fn signed_wrong_scope_audience_expiry_and_ambiguous_key_fail_closed() {
        let f = fixture();
        let keys: Jwks = serde_json::from_value(f["jwks"].clone()).unwrap();
        for name in ["wrong_scope","wrong_audience","expired"] {
            assert_eq!(verified_expiry(f[name].as_str().unwrap(), &keys, GrantClient::Auctions, NOW),
                Err("oauth_claims_rejected"));
        }
        let valid = f["valid"].as_str().unwrap();
        assert_eq!(verified_expiry(valid, &keys, GrantClient::Auctions, NOW), Ok(NOW+900));
        let duplicate: Jwks = serde_json::from_value(json!({"keys":[f["jwks"]["keys"][0],f["jwks"]["keys"][0]]})).unwrap();
        assert_eq!(verified_expiry(valid, &duplicate, GrantClient::Auctions, NOW), Err("oauth_token_rejected"));
        let mut tampered = valid.to_owned();
        let index = tampered.rfind('.').unwrap()+1;
        tampered.replace_range(index..index+1, if valid.as_bytes()[index]==b'A' {"B"} else {"A"});
        assert_eq!(verified_expiry(&tampered, &keys, GrantClient::Auctions, NOW), Err("oauth_token_rejected"));
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
}
