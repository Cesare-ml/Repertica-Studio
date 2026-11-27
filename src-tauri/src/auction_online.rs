//! Optional connection to Repertica Auctions (MVP 2, contract `artist-v1`).
//!
//! * Sign-in: OAuth 2.0 Authorization Code + PKCE S256 in the **system browser**, with a one-shot loopback
//!   redirect `http://127.0.0.1:<port>/repertica/oauth/callback` (RFC 8252), for the native client
//!   `repertica-desktop-auctions` registered in the Studio provider. No secret is embedded in the app.
//! * Artist API: a native HTTP port restricted to the fixed Auctions origin and to the artist-v1 paths.
//!
//! The access token lives only in this process' memory: it is never returned to the webview, logged or
//! written to disk, and it is forgotten on sign-out, expiry, a 401 answer or restart. Nothing is reachable
//! until `ONLINE_ENABLED` is switched on (production provider and service deployed) or, in debug builds
//! only, loopback development origins are configured (`REPERTICA_DEV_STUDIO_ORIGIN`,
//! `REPERTICA_DEV_AUCTIONS_ORIGIN`). Debug builds can also hand the login page to a test browser through
//! `REPERTICA_DEV_LOGIN_URL_FILE` instead of the system browser (manual end-to-end runs).
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// On since 27 November 2026: the identity provider (studio.repertica.com, page under /id/) and the Auctions
/// service (auctions.repertica.com) are deployed.
pub const ONLINE_ENABLED: bool = true;
const STUDIO_ORIGIN: &str = "https://studio.repertica.com";
const AUCTIONS_ORIGIN: &str = "https://auctions.repertica.com";
const CLIENT_ID: &str = "repertica-desktop-auctions";
const CALLBACK_PATH: &str = "/repertica/oauth/callback";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const HTTP_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RESPONSE: usize = 262_144;
const MAX_BODY: usize = 8_192;
const MAX_TOKEN_SECONDS: u64 = 900;
/// Artwork image sent for an auction: re-encoded here (JPEG, longest side 2400 px, no metadata).
const IMAGE_MAX_SIDE: u32 = 2400;
const MAX_IMAGE_UPLOAD: usize = 10 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Endpoints {
    pub(crate) studio: String,
    pub(crate) auctions: String,
}

fn loopback_origin(value: &str) -> bool {
    let Some(port) = value.strip_prefix("http://127.0.0.1:") else { return false };
    !port.is_empty() && port.len() <= 5 && port.bytes().all(|b| b.is_ascii_digit()) && !port.starts_with('0')
        && port.parse::<u32>().map(|p| (1024..=65_535).contains(&p)).unwrap_or(false)
}

impl Endpoints {
    fn production() -> Self {
        Self { studio: STUDIO_ORIGIN.into(), auctions: AUCTIONS_ORIGIN.into() }
    }

    /// Debug builds only: loopback origins of local synthetic services. Release builds ignore the variables.
    fn development() -> Option<Self> {
        if !cfg!(debug_assertions) {
            return None;
        }
        let studio = std::env::var("REPERTICA_DEV_STUDIO_ORIGIN").ok()?;
        let auctions = std::env::var("REPERTICA_DEV_AUCTIONS_ORIGIN").ok()?;
        (loopback_origin(&studio) && loopback_origin(&auctions)).then_some(Self { studio, auctions })
    }

    fn current() -> Option<Self> {
        // Debug builds pointed at local services use them; otherwise the production services, when enabled.
        Self::development().or_else(|| ONLINE_ENABLED.then(Self::production))
    }

    fn secure(&self) -> bool {
        self.studio.starts_with("https://") && self.auctions.starts_with("https://")
    }
}

pub(crate) struct Token {
    value: String,
    expires_at: Instant,
}

/// In-memory session. `generation` changes on every sign-out so that a late sign-in result is discarded.
#[derive(Default)]
pub struct AuctionSession {
    token: Mutex<Option<Token>>,
    generation: Arc<AtomicU64>,
    signing_in: Arc<AtomicBool>,
}

impl AuctionSession {
    fn bearer(&self) -> Option<String> {
        let mut guard = self.token.lock().ok()?;
        match guard.as_ref() {
            Some(token) if token.expires_at > Instant::now() + Duration::from_secs(5) => Some(token.value.clone()),
            _ => {
                *guard = None;
                None
            }
        }
    }

    fn remaining(&self) -> Option<u64> {
        let guard = self.token.lock().ok()?;
        guard.as_ref().and_then(|t| t.expires_at.checked_duration_since(Instant::now())).map(|d| d.as_secs())
    }

    fn store(&self, generation: u64, token: Token) -> bool {
        if self.generation.load(Ordering::SeqCst) != generation {
            return false;
        }
        match self.token.lock() {
            Ok(mut guard) => {
                *guard = Some(token);
                true
            }
            Err(_) => false,
        }
    }

    fn forget(&self) {
        if let Ok(mut guard) = self.token.lock() {
            *guard = None;
        }
    }

    fn sign_out(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.forget();
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    /// The online service can be reached from this build.
    available: bool,
    signed_in: bool,
    expires_in_seconds: Option<u64>,
    /// Debug build pointed at local synthetic services.
    development: bool,
    signing_in: bool,
}

fn status_of(session: &AuctionSession) -> ConnectionStatus {
    let endpoints = Endpoints::current();
    let signed_in = endpoints.is_some() && session.bearer().is_some();
    ConnectionStatus {
        available: endpoints.is_some(),
        signed_in,
        expires_in_seconds: if signed_in { session.remaining() } else { None },
        development: endpoints.as_ref().is_some_and(|e| e.studio != STUDIO_ORIGIN),
        signing_in: session.signing_in.load(Ordering::SeqCst),
    }
}

// ---------------------------------------------------------------------------------------------------------
// PKCE and authorization request

pub(crate) struct Pkce {
    pub(crate) verifier: String,
    pub(crate) challenge: String,
    pub(crate) state: String,
}

fn crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        // Fails only if another provider was installed concurrently, which is equally fine.
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}

fn random_token() -> Result<String, String> {
    crypto_provider();
    let mut bytes = [0u8; 32];
    rustls::crypto::ring::default_provider().secure_random.fill(&mut bytes).map_err(|_| "random_unavailable".to_string())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(crate) fn challenge_for(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn new_pkce() -> Result<Pkce, String> {
    let verifier = random_token()?;
    Ok(Pkce { challenge: challenge_for(&verifier), verifier, state: random_token()? })
}

pub(crate) fn redirect_uri(port: u16) -> String {
    format!("http://127.0.0.1:{port}{CALLBACK_PATH}")
}

/// The provider's login page: under /id/ on studio.repertica.com, where it lives beside the historical Studio pilot
/// (same issuer); at /login on local development providers.
pub(crate) fn login_page(endpoints: &Endpoints) -> String {
    if endpoints.studio == STUDIO_ORIGIN { format!("{STUDIO_ORIGIN}/id/login") } else { format!("{}/login", endpoints.studio) }
}

/// The Studio login page: it authenticates the person and then redirects the browser to the loopback URI.
pub(crate) fn login_url(endpoints: &Endpoints, redirect: &str, pkce: &Pkce) -> Result<String, String> {
    let mut url = tauri::Url::parse(&login_page(endpoints)).map_err(|_| "oauth_unavailable".to_string())?;
    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", redirect)
        .append_pair("state", &pkce.state)
        .append_pair("code_challenge", &pkce.challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(url.into())
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = {
        use std::os::windows::process::CommandExt;
        let mut c = std::process::Command::new("rundll32");
        c.args(["url.dll,FileProtocolHandler", url]).creation_flags(0x0800_0000);
        c
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };
    command.spawn().map(|_| ()).map_err(|_| "browser_unavailable".to_string())
}

/// The system browser. Debug builds only: when `REPERTICA_DEV_LOGIN_URL_FILE` names a file, the login page
/// address is written there for a test browser instead (it carries only the public PKCE challenge and state).
fn open_login(url: &str) -> Result<(), String> {
    if cfg!(debug_assertions) {
        if let Some(file) = std::env::var_os("REPERTICA_DEV_LOGIN_URL_FILE").filter(|v| !v.is_empty()) {
            return std::fs::write(file, url).map_err(|_| "browser_unavailable".to_string());
        }
    }
    open_browser(url)
}

// ---------------------------------------------------------------------------------------------------------
// Loopback callback

fn page(status: &str, title: &str, text: &str) -> String {
    let html = format!(
        "<!doctype html><html lang=\"it\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\">\
         <title>Repertica Studio</title><body style=\"font:16px system-ui,sans-serif;max-width:520px;margin:12vh auto;padding:0 20px;color:#1d2527\">\
         <h1 style=\"font:400 28px Georgia,serif\">{title}</h1><p>{text}</p></body></html>"
    );
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\n\
         X-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{html}",
        html.len()
    )
}

fn reply(stream: &mut TcpStream, response: &str) {
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn read_request_head(stream: &mut TcpStream) -> Option<String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    while buffer.len() < 8_192 {
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let text = String::from_utf8(buffer).ok()?;
    text.lines().next().map(str::to_string)
}

fn valid_code(value: &str) -> bool {
    (32..=192).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

enum Callback {
    Code(String),
    Denied,
    Ignore,
}

fn handle_callback(stream: &mut TcpStream, expected_state: &str) -> Callback {
    let Some(line) = read_request_head(stream) else { return Callback::Ignore };
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(version)) = (parts.next(), parts.next(), parts.next()) else {
        reply(stream, &page("400 Bad Request", "Richiesta non valida", "Torna a Repertica Studio."));
        return Callback::Ignore;
    };
    if method != "GET" || !version.starts_with("HTTP/1.") {
        reply(stream, &page("405 Method Not Allowed", "Richiesta non valida", "Torna a Repertica Studio."));
        return Callback::Ignore;
    }
    let Ok(url) = tauri::Url::parse(&format!("http://127.0.0.1{target}")) else {
        reply(stream, &page("400 Bad Request", "Richiesta non valida", "Torna a Repertica Studio."));
        return Callback::Ignore;
    };
    if url.path() != CALLBACK_PATH {
        reply(stream, &page("404 Not Found", "Pagina non trovata", "Torna a Repertica Studio."));
        return Callback::Ignore;
    }
    let pairs: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    let single = |name: &str| {
        let found: Vec<&String> = pairs.iter().filter(|(k, _)| k == name).map(|(_, v)| v).collect();
        (found.len() == 1).then(|| found[0].clone())
    };
    let unknown = pairs.iter().any(|(k, _)| !["code", "state", "error", "error_description"].contains(&k.as_str()));
    let state_ok = single("state").map(|s| constant_time_eq(&s, expected_state)).unwrap_or(false);
    if unknown || !state_ok {
        // Possibly a stale tab or a forged request: refuse it and keep waiting for the genuine redirect.
        reply(stream, &page("400 Bad Request", "Accesso non riconosciuto",
            "Questa risposta non corrisponde all'accesso avviato da Repertica Studio. / This response does not match the sign-in started by Repertica Studio."));
        return Callback::Ignore;
    }
    if single("error").is_some() {
        reply(stream, &page("200 OK", "Accesso annullato", "Puoi chiudere questa finestra e tornare a Repertica Studio. / You can close this window."));
        return Callback::Denied;
    }
    match single("code") {
        Some(code) if valid_code(&code) => {
            reply(stream, &page("200 OK", "Accesso completato",
                "Puoi chiudere questa finestra e tornare a Repertica Studio. / Signed in: you can close this window and return to Repertica Studio."));
            Callback::Code(code)
        }
        _ => {
            reply(stream, &page("400 Bad Request", "Accesso non riuscito", "Torna a Repertica Studio e riprova."));
            Callback::Ignore
        }
    }
}

/// Waits for the single genuine redirect on the loopback listener (or deadline / cancellation).
pub(crate) fn await_callback(listener: &TcpListener, expected_state: &str, deadline: Instant, cancelled: &dyn Fn() -> bool)
    -> Result<String, String> {
    listener.set_nonblocking(true).map_err(|_| "oauth_unavailable".to_string())?;
    loop {
        if cancelled() {
            return Err("sign_in_cancelled".into());
        }
        if Instant::now() >= deadline {
            return Err("sign_in_timeout".into());
        }
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                let _ = stream.set_nonblocking(false);
                match handle_callback(&mut stream, expected_state) {
                    Callback::Code(code) => return Ok(code),
                    Callback::Denied => return Err("oauth_denied".into()),
                    Callback::Ignore => {}
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------
// HTTP

fn http_client(endpoints: &Endpoints) -> Result<reqwest::Client, String> {
    crypto_provider();
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(HTTP_TIMEOUT)
        .https_only(endpoints.secure())
        .user_agent(concat!("ReperticaStudio/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| "network_unavailable".to_string())
}

async fn bounded_body(mut response: reqwest::Response) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "network_unavailable".to_string())? {
        body.extend_from_slice(&chunk);
        if body.len() > MAX_RESPONSE {
            return Err("invalid_response".into());
        }
    }
    Ok(body)
}

fn jwt_shaped(value: &str) -> bool {
    value.len() <= 8_192
        && value.split('.').count() == 3
        && value.split('.').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
}

/// Strict reading of the token endpoint answer (same rules as the web client).
pub(crate) fn parse_token_response(body: &[u8], now: Instant) -> Result<Token, String> {
    let bad = || "oauth_exchange_failed".to_string();
    let value: serde_json::Value = serde_json::from_slice(body).map_err(|_| bad())?;
    let object = value.as_object().ok_or_else(bad)?;
    if object.keys().any(|k| !["access_token", "token_type", "expires_in", "scope"].contains(&k.as_str())) {
        return Err(bad());
    }
    let token = object.get("access_token").and_then(|v| v.as_str()).filter(|t| jwt_shaped(t)).ok_or_else(bad)?;
    let seconds = object.get("expires_in").and_then(|v| v.as_u64()).filter(|s| (1..=MAX_TOKEN_SECONDS).contains(s)).ok_or_else(bad)?;
    if object.get("token_type").and_then(|v| v.as_str()) != Some("Bearer") || object.get("scope").and_then(|v| v.as_str()) != Some("profile") {
        return Err(bad());
    }
    Ok(Token { value: token.to_string(), expires_at: now + Duration::from_secs(seconds) })
}

async fn exchange_code(client: &reqwest::Client, endpoints: &Endpoints, code: &str, redirect: &str, verifier: &str)
    -> Result<Token, String> {
    // No Origin header: the provider accepts native clients only without one.
    let response = client
        .post(format!("{}/api/identity/token", endpoints.studio))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(serde_json::json!({ "grant_type": "authorization_code", "client_id": CLIENT_ID,
            "redirect_uri": redirect, "code": code, "code_verifier": verifier }).to_string())
        .send()
        .await
        .map_err(|_| "oauth_exchange_failed".to_string())?;
    if response.status() != reqwest::StatusCode::OK {
        return Err("oauth_exchange_failed".into());
    }
    let started = Instant::now();
    parse_token_response(&bounded_body(response).await.map_err(|_| "oauth_exchange_failed".to_string())?, started)
}

fn identifier(value: &str) -> bool {
    (1..=100).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn digits(value: &str, max: usize) -> bool {
    (1..=max).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_digit()) && !value.starts_with('0')
}

/// Only the artist-v1 operations, with strictly shaped parameters. The webview never chooses the origin.
pub(crate) fn allowed_request(method: &str, target: &str) -> bool {
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (target, None),
    };
    let segments: Vec<&str> = path.strip_prefix("/api/artist/").map(|rest| rest.split('/').collect()).unwrap_or_default();
    match (method, segments.as_slice(), query) {
        ("GET", ["tenants"], None) => true,
        ("GET", ["auctions"], Some(q)) => {
            let pairs: Vec<(&str, &str)> = q.split('&').filter_map(|p| p.split_once('=')).collect();
            pairs.len() == q.split('&').count() && pairs.len() == 2
                && pairs[0].0 == "tenantId" && identifier(pairs[0].1) && pairs[1].0 == "page" && digits(pairs[1].1, 3)
        }
        ("GET", ["auctions", id], None) => identifier(id),
        ("GET", ["auctions", id, "bids"], q) => {
            identifier(id) && q.map(|q| q.strip_prefix("cursor=").map(|c| digits(c, 16)).unwrap_or(false)).unwrap_or(true)
        }
        ("GET", ["auctions", id, "participants"], None) => identifier(id),
        ("GET", ["auctions", id, "terms"], None) => identifier(id),
        ("GET", ["email-policy"], Some(q)) => q.strip_prefix("tenantId=").map(identifier).unwrap_or(false),
        ("POST", ["email-policy", "blocks"], None) => true,
        ("POST", ["email-policy", "blocks", id, "removal"], None) => {
            id.len() == 24 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }
        ("POST", ["publications"], None) => true,
        ("POST", ["auctions", id, "bids", bid, "cancellation"], None) => identifier(id) && identifier(bid),
        _ => false,
    }
}

fn idempotency_key(value: &str) -> bool {
    (16..=96).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[derive(Serialize, Debug)]
pub struct ApiResponse {
    status: u16,
    body: serde_json::Value,
}

/// One artist API call. A transport failure is an error (the caller treats the outcome as uncertain);
/// any HTTP answer, including refusals, is returned as data.
pub(crate) async fn send_api(client: &reqwest::Client, endpoints: &Endpoints, bearer: &str, method: &str, target: &str,
    body: Option<&str>, key: Option<&str>) -> Result<ApiResponse, String> {
    if !allowed_request(method, target) {
        return Err("request_not_allowed".into());
    }
    let mut request = match method {
        "GET" => {
            if body.is_some() || key.is_some() {
                return Err("request_not_allowed".into());
            }
            client.get(format!("{}{target}", endpoints.auctions))
        }
        _ => {
            let body = body.filter(|b| b.len() <= MAX_BODY && serde_json::from_str::<serde_json::Value>(b).map(|v| v.is_object()).unwrap_or(false))
                .ok_or_else(|| "request_not_allowed".to_string())?;
            let key = key.filter(|k| idempotency_key(k)).ok_or_else(|| "request_not_allowed".to_string())?;
            client.post(format!("{}{target}", endpoints.auctions))
                .header("Content-Type", "application/json")
                .header("Idempotency-Key", key)
                .body(body.to_string())
        }
    };
    request = request.header("Accept", "application/json").header("Authorization", format!("Bearer {bearer}"));
    let response = request.send().await.map_err(|_| "network_unavailable".to_string())?;
    let status = response.status().as_u16();
    let bytes = bounded_body(response).await?;
    let body = if bytes.is_empty() { serde_json::Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null) };
    Ok(ApiResponse { status, body })
}

/// The artwork's main image from the local archive, re-encoded on this computer so that only pixels leave it:
/// JPEG, longest side at most 2400 px, without EXIF/GPS or other metadata. The original stays in the archive.
pub(crate) fn artwork_image_for_upload(db: &crate::db::Database, artwork_id: &str) -> Result<Vec<u8>, String> {
    if artwork_id.is_empty() || artwork_id.len() > 96 || !artwork_id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        return Err("artwork_not_found".into());
    }
    let relative: Option<String> = {
        let conn = db.conn.lock().map_err(|_| "archive_unavailable".to_string())?;
        match conn.query_row("SELECT image_path FROM artworks WHERE id = ?1", [artwork_id], |row| row.get::<_, Option<String>>(0)) {
            Ok(path) => path,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err("artwork_not_found".into()),
            Err(_) => return Err("archive_unavailable".into()),
        }
    };
    let relative = relative.filter(|p| !p.is_empty()).ok_or_else(|| "artwork_image_missing".to_string())?;
    let original = crate::media::read_media_file(&db.directory, &relative).map_err(|_| "artwork_image_missing".to_string())?;
    let encoded = crate::imaging::export(&original, false, 90, Some(IMAGE_MAX_SIDE), Some(IMAGE_MAX_SIDE))
        .map_err(|_| "artwork_image_invalid".to_string())?;
    if encoded.bytes.len() > MAX_IMAGE_UPLOAD {
        return Err("artwork_image_too_large".into());
    }
    Ok(encoded.bytes)
}

/// Uploads one image to the artist's private staging area (POST /api/artist/images).
pub(crate) async fn send_image(client: &reqwest::Client, endpoints: &Endpoints, bearer: &str, jpeg: Vec<u8>) -> Result<ApiResponse, String> {
    if jpeg.is_empty() || jpeg.len() > MAX_IMAGE_UPLOAD {
        return Err("artwork_image_too_large".into());
    }
    let response = client.post(format!("{}/api/artist/images", endpoints.auctions))
        .header("Content-Type", "image/jpeg")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {bearer}"))
        .body(jpeg)
        .send().await.map_err(|_| "network_unavailable".to_string())?;
    let status = response.status().as_u16();
    let bytes = bounded_body(response).await?;
    let body = if bytes.is_empty() { serde_json::Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null) };
    Ok(ApiResponse { status, body })
}

// ---------------------------------------------------------------------------------------------------------
// Commands

#[tauri::command]
pub fn auction_connection_status(session: tauri::State<'_, AuctionSession>) -> ConnectionStatus {
    status_of(&session)
}

#[tauri::command]
pub fn auction_sign_out(session: tauri::State<'_, AuctionSession>) -> ConnectionStatus {
    session.sign_out();
    status_of(&session)
}

struct SigningIn(Arc<AtomicBool>);
impl Drop for SigningIn {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// The whole sign-in: loopback listener, PKCE, browser, single callback, code exchange, in-memory token.
/// `open` shows the login page to the person (the system browser in the app, a simulated browser in tests).
pub(crate) async fn sign_in_flow(session: &AuctionSession, endpoints: &Endpoints, open: &(dyn Fn(&str) -> Result<(), String> + Sync),
    timeout: Duration) -> Result<(), String> {
    if session.signing_in.swap(true, Ordering::SeqCst) {
        return Err("sign_in_in_progress".into());
    }
    let _guard = SigningIn(session.signing_in.clone());
    let generation = session.generation.load(Ordering::SeqCst);
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|_| "oauth_unavailable".to_string())?;
    let port = listener.local_addr().map_err(|_| "oauth_unavailable".to_string())?.port();
    let redirect = redirect_uri(port);
    let pkce = new_pkce()?;
    open(&login_url(endpoints, &redirect, &pkce)?)?;
    let state = pkce.state.clone();
    let generations = session.generation.clone();
    let code = tauri::async_runtime::spawn_blocking(move || {
        await_callback(&listener, &state, Instant::now() + timeout, &|| generations.load(Ordering::SeqCst) != generation)
    })
    .await
    .map_err(|_| "oauth_unavailable".to_string())??;
    let client = http_client(endpoints)?;
    let token = exchange_code(&client, endpoints, &code, &redirect, &pkce.verifier).await?;
    if !session.store(generation, token) {
        return Err("sign_in_cancelled".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn auction_sign_in(session: tauri::State<'_, AuctionSession>) -> Result<ConnectionStatus, String> {
    let endpoints = Endpoints::current().ok_or_else(|| "auctions_not_available".to_string())?;
    sign_in_flow(&session, &endpoints, &open_login, LOGIN_TIMEOUT).await?;
    Ok(status_of(&session))
}

/// Uploads the main image of one archive artwork for an auction; the answer carries the staged image id.
#[tauri::command]
pub async fn auction_upload_artwork_image(session: tauri::State<'_, AuctionSession>, db: tauri::State<'_, crate::db::Database>,
    artwork_id: String) -> Result<ApiResponse, String> {
    let endpoints = Endpoints::current().ok_or_else(|| "auctions_not_available".to_string())?;
    let Some(bearer) = session.bearer() else {
        return Ok(ApiResponse { status: 401, body: serde_json::json!({ "error": "sign_in_required" }) });
    };
    let jpeg = artwork_image_for_upload(&db, &artwork_id)?;
    let client = http_client(&endpoints)?;
    let response = send_image(&client, &endpoints, &bearer, jpeg).await?;
    if response.status == 401 {
        session.forget();
    }
    Ok(response)
}

#[tauri::command]
pub async fn auction_api_request(session: tauri::State<'_, AuctionSession>, method: String, path: String,
    body: Option<String>, idempotency_key: Option<String>) -> Result<ApiResponse, String> {
    let endpoints = Endpoints::current().ok_or_else(|| "auctions_not_available".to_string())?;
    if !allowed_request(&method, &path) {
        return Err("request_not_allowed".into());
    }
    let Some(bearer) = session.bearer() else {
        // Not a lost answer: the person must sign in, nothing was sent.
        return Ok(ApiResponse { status: 401, body: serde_json::json!({ "error": "sign_in_required" }) });
    };
    let client = http_client(&endpoints)?;
    let response = send_api(&client, &endpoints, &bearer, &method, &path, body.as_deref(), idempotency_key.as_deref()).await?;
    if response.status == 401 {
        session.forget();
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn local() -> Endpoints {
        Endpoints { studio: "http://127.0.0.1:1".into(), auctions: "http://127.0.0.1:2".into() }
    }

    #[test]
    fn pkce_matches_the_rfc_7636_vector_and_tokens_are_fresh() {
        assert_eq!(challenge_for("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        let a = new_pkce().unwrap();
        let b = new_pkce().unwrap();
        assert_eq!(a.verifier.len(), 43);
        assert_eq!(a.state.len(), 43);
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.state, a.verifier);
        assert_eq!(a.challenge, challenge_for(&a.verifier));
    }

    #[test]
    fn login_url_carries_exactly_the_provider_parameters() {
        let pkce = Pkce { verifier: "v".repeat(43), challenge: "c".repeat(43), state: "s".repeat(43) };
        let url = tauri::Url::parse(&login_url(&Endpoints::production(), &redirect_uri(49152), &pkce).unwrap()).unwrap();
        assert_eq!(url.origin().ascii_serialization(), STUDIO_ORIGIN);
        assert_eq!(url.path(), "/id/login", "the provider page beside the Studio pilot");
        let pairs: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.into(), v.into())).collect();
        assert_eq!(pairs, vec![
            ("client_id".into(), "repertica-desktop-auctions".into()),
            ("redirect_uri".into(), "http://127.0.0.1:49152/repertica/oauth/callback".into()),
            ("state".into(), "s".repeat(43)),
            ("code_challenge".into(), "c".repeat(43)),
            ("code_challenge_method".into(), "S256".into()),
        ]);
        assert!(!url.as_str().contains(&"v".repeat(43)), "the verifier never leaves the process");
    }

    #[test]
    fn only_loopback_development_origins_are_accepted() {
        assert!(loopback_origin("http://127.0.0.1:4173"));
        for bad in ["https://127.0.0.1:4173", "http://localhost:4173", "http://127.0.0.1", "http://127.0.0.1:80",
            "http://127.0.0.1:04173", "http://127.0.0.1:4173/", "http://127.0.0.1:99999", "http://10.0.0.1:4173"] {
            assert!(!loopback_origin(bad), "{bad}");
        }
        assert!(Endpoints::production().secure());
        assert!(!local().secure());
        assert!(ONLINE_ENABLED, "the services are deployed");
        assert_eq!(login_page(&Endpoints::production()), "https://studio.repertica.com/id/login");
        assert_eq!(login_page(&local()), "http://127.0.0.1:1/login");
        assert_eq!(Endpoints::current(), Some(Endpoints::production()), "tests run without development origins");
    }

    #[test]
    fn artist_api_allowlist() {
        for ok in [("GET", "/api/artist/tenants"), ("GET", "/api/artist/auctions?tenantId=artist-a&page=1"),
            ("GET", "/api/artist/auctions/lot_1"), ("GET", "/api/artist/auctions/lot-1/bids"),
            ("GET", "/api/artist/auctions/lot-1/bids?cursor=105"), ("POST", "/api/artist/publications"),
            ("GET", "/api/artist/auctions/lot-1/participants"), ("GET", "/api/artist/auctions/lot_1/terms"),
            ("GET", "/api/artist/email-policy?tenantId=artist-a"), ("POST", "/api/artist/email-policy/blocks"),
            ("POST", "/api/artist/email-policy/blocks/0123456789abcdef01234567/removal"),
            ("POST", "/api/artist/auctions/lot-1/bids/0a52304b-27e4-4744-9c16-46976bae9284/cancellation")] {
            assert!(allowed_request(ok.0, ok.1), "{ok:?}");
        }
        for bad in [("POST", "/api/artist/tenants"), ("GET", "/api/artist/auctions"), ("GET", "/api/artist/auctions?page=1&tenantId=a"),
            ("GET", "/api/artist/auctions?tenantId=a&page=0"), ("GET", "/api/artist/auctions?tenantId=a&page=1&x=1"),
            ("GET", "/api/artist/auctions/../tenants"), ("GET", "/api/artist/auctions/a/bids?cursor=0"),
            ("GET", "/api/artist/auctions/a/bids?cursor=1&x=2"), ("GET", "/api/catalog/auctions"), ("GET", "https://evil.test/api/artist/tenants"),
            ("DELETE", "/api/artist/auctions/a"), ("POST", "/api/artist/auctions/a/bids/b/cancellation?x=1"),
            ("GET", "//evil.test/api/artist/tenants"), ("GET", "/api/artist/auctions/a%2Fb"),
            ("GET", "/api/artist/auctions/a/participants?x=1"), ("POST", "/api/artist/auctions/a/participants"),
            ("GET", "/api/artist/auctions/a/terms?x=1"), ("GET", "/api/artist/email-policy"),
            ("GET", "/api/artist/email-policy?tenantId=a&x=1"), ("GET", "/api/artist/email-policy?tenantId=a/b"),
            ("DELETE", "/api/artist/email-policy/blocks/0123456789abcdef01234567"), ("POST", "/api/artist/email-policy/blocks?x=1"),
            ("POST", "/api/artist/email-policy/blocks/0123456789ABCDEF01234567/removal"),
            ("POST", "/api/artist/email-policy/blocks/short/removal"), ("GET", "/api/artist/email-policy/blocks")] {
            assert!(!allowed_request(bad.0, bad.1), "{bad:?}");
        }
    }

    #[test]
    fn token_answers_are_read_strictly() {
        let now = Instant::now();
        let good = br#"{"access_token":"aaa.bbb.ccc","token_type":"Bearer","expires_in":900,"scope":"profile"}"#;
        let token = parse_token_response(good, now).unwrap();
        assert_eq!(token.value, "aaa.bbb.ccc");
        assert_eq!(token.expires_at, now + Duration::from_secs(900));
        for bad in [r#"{"access_token":"aaa.bbb.ccc","token_type":"Bearer","expires_in":901,"scope":"profile"}"#,
            r#"{"access_token":"aaa.bbb.ccc","token_type":"bearer","expires_in":900,"scope":"profile"}"#,
            r#"{"access_token":"aaa.bbb.ccc","token_type":"Bearer","expires_in":900,"scope":"artworks:create"}"#,
            r#"{"access_token":"not a jwt","token_type":"Bearer","expires_in":900,"scope":"profile"}"#,
            r#"{"access_token":"aaa.bbb.ccc","token_type":"Bearer","expires_in":900,"scope":"profile","refresh_token":"x"}"#,
            r#"[]"#, "not json"] {
            assert!(parse_token_response(bad.as_bytes(), now).is_err(), "{bad}");
        }
    }

    #[test]
    fn session_forgets_tokens_and_discards_late_sign_ins() {
        let session = AuctionSession::default();
        let generation = session.generation.load(Ordering::SeqCst);
        assert!(session.store(generation, Token { value: "aaa.bbb.ccc".into(), expires_at: Instant::now() + Duration::from_secs(600) }));
        assert_eq!(session.bearer().as_deref(), Some("aaa.bbb.ccc"));
        session.sign_out();
        assert!(session.bearer().is_none());
        assert!(!session.store(generation, Token { value: "late.late.late".into(), expires_at: Instant::now() + Duration::from_secs(600) }),
            "a sign-in finishing after sign-out is discarded");
        let current = session.generation.load(Ordering::SeqCst);
        session.store(current, Token { value: "old.old.old".into(), expires_at: Instant::now() + Duration::from_secs(2) });
        assert!(session.bearer().is_none(), "a token about to expire is not used");
        let json = serde_json::to_string(&status_of(&session)).unwrap();
        assert!(!json.contains("old.old.old") && !json.contains("aaa.bbb.ccc"), "the token never reaches the webview");
    }

    fn get(port: u16, target: &str) -> String {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        write!(stream, "GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    #[test]
    fn loopback_callback_accepts_only_the_genuine_redirect() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = "s".repeat(43);
        let code = "c".repeat(40);
        let expected = (state.clone(), code.clone());
        let browser = thread::spawn(move || {
            let (state, code) = expected;
            assert!(get(port, "/favicon.ico").starts_with("HTTP/1.1 404"));
            assert!(get(port, &format!("{CALLBACK_PATH}?code={code}&state=forged-state-0123456789")).starts_with("HTTP/1.1 400"));
            assert!(get(port, &format!("{CALLBACK_PATH}?code={code}&state={state}&state={state}")).starts_with("HTTP/1.1 400"));
            assert!(get(port, &format!("{CALLBACK_PATH}?code={code}&state={state}&next=https://evil.test")).starts_with("HTTP/1.1 400"));
            let ok = get(port, &format!("{CALLBACK_PATH}?code={code}&state={state}"));
            assert!(ok.starts_with("HTTP/1.1 200") && ok.contains("Accesso completato") && !ok.contains(&code));
        });
        let received = await_callback(&listener, &state, Instant::now() + Duration::from_secs(10), &|| false).unwrap();
        browser.join().unwrap();
        assert_eq!(received, code);
    }

    #[test]
    fn loopback_callback_reports_denial_timeout_and_cancellation() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = "s".repeat(43);
        let target = format!("{CALLBACK_PATH}?error=access_denied&state={state}");
        let browser = thread::spawn(move || get(port, &target));
        assert_eq!(await_callback(&listener, &state, Instant::now() + Duration::from_secs(10), &|| false).unwrap_err(), "oauth_denied");
        browser.join().unwrap();
        assert_eq!(await_callback(&listener, &state, Instant::now() + Duration::from_millis(200), &|| false).unwrap_err(), "sign_in_timeout");
        assert_eq!(await_callback(&listener, &state, Instant::now() + Duration::from_secs(10), &|| true).unwrap_err(), "sign_in_cancelled");
    }

    /// One-request HTTP server returning `answer`; hands back the raw request it received.
    fn serve_once(answer: &'static str) -> (u16, thread::JoinHandle<String>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let read = stream.read(&mut chunk).unwrap();
                request.extend_from_slice(&chunk[..read]);
                let text = String::from_utf8_lossy(&request).to_string();
                if let Some(head_end) = text.find("\r\n\r\n") {
                    let length = text.lines().find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length: ").map(|v| v.trim().parse::<usize>().unwrap())).unwrap_or(0);
                    if request.len() >= head_end + 4 + length { break; }
                }
                if read == 0 { break; }
            }
            stream.write_all(answer.as_bytes()).unwrap();
            String::from_utf8(request).unwrap()
        });
        (port, handle)
    }

    #[test]
    fn code_exchange_posts_the_pkce_grant_without_origin() {
        let (port, server) = serve_once("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 87\r\nConnection: close\r\n\r\n{\"access_token\":\"aaa.bbb.ccc\",\"token_type\":\"Bearer\",\"expires_in\":900,\"scope\":\"profile\"}");
        let endpoints = Endpoints { studio: format!("http://127.0.0.1:{port}"), auctions: "http://127.0.0.1:2".into() };
        let token = tauri::async_runtime::block_on(async {
            let client = http_client(&endpoints).unwrap();
            exchange_code(&client, &endpoints, &"c".repeat(40), &redirect_uri(49152), &"v".repeat(43)).await
        }).unwrap();
        assert_eq!(token.value, "aaa.bbb.ccc");
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /api/identity/token HTTP/1.1"));
        assert!(!request.to_ascii_lowercase().contains("\r\norigin:"), "native exchange never sends Origin");
        let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["client_id"], "repertica-desktop-auctions");
        assert_eq!(body["code_verifier"], "v".repeat(43));
        assert_eq!(body["redirect_uri"], "http://127.0.0.1:49152/repertica/oauth/callback");
    }

    #[test]
    fn api_requests_carry_the_token_only_to_the_fixed_origin() {
        let (port, server) = serve_once("HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: 23\r\nConnection: close\r\n\r\n{\"outcome\":\"cancelled\"}");
        let endpoints = Endpoints { studio: "http://127.0.0.1:1".into(), auctions: format!("http://127.0.0.1:{port}") };
        let path = "/api/artist/auctions/lot-1/bids/bid-1/cancellation";
        let response = tauri::async_runtime::block_on(async {
            let client = http_client(&endpoints).unwrap();
            assert_eq!(send_api(&client, &endpoints, "aaa.bbb.ccc", "POST", path, Some("[]"), Some("cancel-key-0000000001")).await.unwrap_err(),
                "request_not_allowed", "a non-object body is refused before sending");
            assert_eq!(send_api(&client, &endpoints, "aaa.bbb.ccc", "POST", path, Some("{}"), Some("short")).await.unwrap_err(), "request_not_allowed");
            assert_eq!(send_api(&client, &endpoints, "aaa.bbb.ccc", "GET", "https://evil.test/api/artist/tenants", None, None).await.unwrap_err(), "request_not_allowed");
            send_api(&client, &endpoints, "aaa.bbb.ccc", "POST", path, Some(r#"{"reason":"Offerente fittizio"}"#), Some("cancel-key-0000000001")).await
        }).unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.body["outcome"], "cancelled");
        let request = server.join().unwrap();
        assert!(request.starts_with(&format!("POST {path} HTTP/1.1")));
        assert!(request.contains("authorization: Bearer aaa.bbb.ccc") || request.contains("Authorization: Bearer aaa.bbb.ccc"));
        assert!(request.to_ascii_lowercase().contains("idempotency-key: cancel-key-0000000001"));
        assert!(request.ends_with(r#"{"reason":"Offerente fittizio"}"#));
    }

    #[test]
    fn full_sign_in_with_a_simulated_browser_then_an_artist_call() {
        let token_json = "{\"access_token\":\"aaa.bbb.ccc\",\"token_type\":\"Bearer\",\"expires_in\":900,\"scope\":\"profile\"}";
        assert_eq!(token_json.len(), 87);
        let (studio_port, studio) = serve_once("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 87\r\nConnection: close\r\n\r\n{\"access_token\":\"aaa.bbb.ccc\",\"token_type\":\"Bearer\",\"expires_in\":900,\"scope\":\"profile\"}");
        let (auctions_port, auctions) = serve_once("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 14\r\nConnection: close\r\n\r\n{\"tenants\":[]}");
        let endpoints = Endpoints { studio: format!("http://127.0.0.1:{studio_port}"), auctions: format!("http://127.0.0.1:{auctions_port}") };
        let seen = Arc::new(Mutex::new(None::<(String, String)>));
        let record = seen.clone();
        // The "browser": after a successful login the provider redirects to the loopback URI with code and state.
        let browser = move |login: &str| -> Result<(), String> {
            let url = tauri::Url::parse(login).map_err(|e| e.to_string())?;
            let query: std::collections::HashMap<String, String> = url.query_pairs().map(|(k, v)| (k.into(), v.into())).collect();
            assert_eq!(url.path(), "/login");
            assert_eq!(query["client_id"], CLIENT_ID);
            assert_eq!(query["code_challenge_method"], "S256");
            *record.lock().unwrap() = Some((query["code_challenge"].clone(), query["redirect_uri"].clone()));
            let redirect = tauri::Url::parse(&query["redirect_uri"]).unwrap();
            let target = format!("{}?code={}&state={}", redirect.path(), "c".repeat(40), query["state"]);
            let port = redirect.port().unwrap();
            thread::spawn(move || get(port, &target));
            Ok(())
        };
        let session = AuctionSession::default();
        tauri::async_runtime::block_on(sign_in_flow(&session, &endpoints, &browser, Duration::from_secs(10))).unwrap();
        assert_eq!(session.bearer().as_deref(), Some("aaa.bbb.ccc"));
        assert_eq!(status_of(&session).signed_in, ONLINE_ENABLED, "the connection is usable only when the services are enabled");

        let exchange = studio.join().unwrap();
        let body: serde_json::Value = serde_json::from_str(exchange.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        let (challenge, redirect) = seen.lock().unwrap().clone().unwrap();
        assert_eq!(challenge_for(body["code_verifier"].as_str().unwrap()), challenge, "the verifier matches the S256 challenge");
        assert_eq!(body["redirect_uri"], redirect);
        assert_eq!(body["code"], "c".repeat(40));

        let bearer = session.bearer().unwrap();
        let response = tauri::async_runtime::block_on(async {
            send_api(&http_client(&endpoints).unwrap(), &endpoints, &bearer, "GET", "/api/artist/tenants", None, None).await
        }).unwrap();
        assert_eq!(response.status, 200);
        assert!(auctions.join().unwrap().to_ascii_lowercase().contains("authorization: bearer aaa.bbb.ccc"));
        // A second concurrent sign-in is refused while one is running; after it ends the flag is released.
        assert!(!session.signing_in.load(Ordering::SeqCst));
    }

    /// Cross-process check against the REAL Auctions server running its synthetic preview (docs/AUCTIONS_DESKTOP_HANDOFF.md):
    /// * with the synthetic Studio provider: REPERTICA_E2E_STUDIO + REPERTICA_E2E_AUCTIONS;
    /// * with the REAL Studio provider in its isolated test runtime: also REPERTICA_E2E_BROWSER, a helper that does what
    ///   the provider's /login page does after sign-in (authorize, then follow the loopback redirect).
    /// Run with `cargo test -- --ignored live_`.
    #[test]
    #[ignore = "needs the local synthetic Auctions and Studio services"]
    fn live_synthetic_services_sign_in_list_and_cancel() {
        let studio = std::env::var("REPERTICA_E2E_STUDIO").expect("REPERTICA_E2E_STUDIO");
        let auctions = std::env::var("REPERTICA_E2E_AUCTIONS").expect("REPERTICA_E2E_AUCTIONS");
        assert!(loopback_origin(&studio) && loopback_origin(&auctions));
        let endpoints = Endpoints { studio, auctions };
        let helper = std::env::var("REPERTICA_E2E_BROWSER").ok();
        // The "browser": open the provider login page and follow its redirect to the loopback listener.
        let browser = move |login: &str| -> Result<(), String> {
            if let Some(helper) = helper.as_deref() {
                assert!(loopback_origin(helper));
                let port: u16 = helper.rsplit(':').next().unwrap().parse().unwrap();
                let mut simulate = tauri::Url::parse("http://127.0.0.1/simulate").unwrap();
                simulate.query_pairs_mut().append_pair("login", login);
                let target = format!("{}?{}", simulate.path(), simulate.query().unwrap());
                thread::spawn(move || {
                    let answer = get(port, &target);
                    assert!(answer.starts_with("HTTP/1.1 200"), "browser helper: {answer}");
                });
                return Ok(());
            }
            let url = tauri::Url::parse(login).map_err(|e| e.to_string())?;
            let target = format!("{}?{}", url.path(), url.query().unwrap_or(""));
            let port = url.port().unwrap();
            thread::spawn(move || {
                let answer = get(port, &target);
                let location = answer.lines().find_map(|l| l.strip_prefix("Location: ").or_else(|| l.strip_prefix("location: ")))
                    .expect("provider redirect").trim().to_string();
                let redirect = tauri::Url::parse(&location).unwrap();
                get(redirect.port().unwrap(), &format!("{}?{}", redirect.path(), redirect.query().unwrap()))
            });
            Ok(())
        };
        let session = AuctionSession::default();
        tauri::async_runtime::block_on(sign_in_flow(&session, &endpoints, &browser, Duration::from_secs(20))).unwrap();
        let bearer = session.bearer().expect("signed in");
        tauri::async_runtime::block_on(async {
            let client = http_client(&endpoints).unwrap();
            let call = |method: &'static str, path: String, body: Option<String>, key: Option<String>| {
                let (client, endpoints, bearer) = (client.clone(), endpoints.clone(), bearer.clone());
                async move { send_api(&client, &endpoints, &bearer, method, &path, body.as_deref(), key.as_deref()).await.unwrap() }
            };
            let tenants = call("GET", "/api/artist/tenants".into(), None, None).await;
            assert_eq!(tenants.status, 200);
            let tenant = tenants.body["tenants"][0]["id"].as_str().unwrap().to_string();
            let listed = call("GET", format!("/api/artist/auctions?tenantId={tenant}&page=1"), None, None).await;
            assert_eq!(listed.status, 200);
            let auction = listed.body["items"][0]["auctionId"].as_str().unwrap().to_string();
            let bids = call("GET", format!("/api/artist/auctions/{auction}/bids"), None, None).await;
            let target = bids.body["items"].as_array().unwrap().iter()
                .find(|b| b["state"] == "valid" && b["isLeading"] == false).expect("a valid bid")["bidId"].as_str().unwrap().to_string();
            assert!(bids.body["items"][0]["bidderEmail"].as_str().unwrap().ends_with("@example.test"));
            let key = format!("cancel-e2e-{}", URL_SAFE_NO_PAD.encode(Sha256::digest(target.as_bytes()))[..20].to_string());
            let path = format!("/api/artist/auctions/{auction}/bids/{target}/cancellation");
            let first = call("POST", path.clone(), Some(r#"{"reason":"Prova end-to-end dal Desktop"}"#.into()), Some(key.clone())).await;
            assert!(first.status == 201 || first.status == 200, "status {}", first.status);
            let replay = call("POST", path, Some(r#"{"reason":"Prova end-to-end dal Desktop"}"#.into()), Some(key)).await;
            assert_eq!(replay.status, 200, "idempotent replay");
            let after = call("GET", format!("/api/artist/auctions/{auction}/bids"), None, None).await;
            let cancelled = after.body["items"].as_array().unwrap().iter().find(|b| b["bidId"] == target.as_str()).unwrap();
            assert_eq!(cancelled["state"], "cancelled");
            let forged = send_api(&client, &endpoints, "aaa.bbb.ccc", "GET", "/api/artist/tenants", None, None).await.unwrap();
            assert_eq!(forged.status, 401, "the server refuses a token it did not issue");
        });
    }

    #[test]
    fn a_lost_connection_is_an_error_not_an_answer() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let closer = thread::spawn(move || { let (stream, _) = listener.accept().unwrap(); drop(stream); });
        let endpoints = Endpoints { studio: "http://127.0.0.1:1".into(), auctions: format!("http://127.0.0.1:{port}") };
        let result = tauri::async_runtime::block_on(async {
            let client = http_client(&endpoints).unwrap();
            send_api(&client, &endpoints, "aaa.bbb.ccc", "GET", "/api/artist/tenants", None, None).await
        });
        closer.join().unwrap();
        assert_eq!(result.unwrap_err(), "network_unavailable");
    }
    #[test]
    fn artwork_image_is_reencoded_locally_without_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::db::open(dir.path().join("archive")).unwrap();
        let picture = image::RgbImage::from_pixel(3000, 1500, image::Rgb([120, 80, 40]));
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 90).encode_image(&picture).unwrap();
        // An EXIF segment right after SOI, as cameras and phones write it (here with a fake GPS marker).
        let exif: &[u8] = b"Exif\0\0GPS-SECRET-COORDINATES";
        let mut original = vec![0xFF, 0xD8, 0xFF, 0xE1];
        original.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
        original.extend_from_slice(exif);
        original.extend_from_slice(&jpeg[2..]);
        std::fs::write(dir.path().join("archive").join("media").join("opera.jpg"), &original).unwrap();
        db.conn.lock().unwrap().execute("INSERT INTO artworks(id,code,title,image_path,created_at,updated_at)
            VALUES('art-1','A-1','Opera','media/opera.jpg','t','t')", []).unwrap();
        let sent = artwork_image_for_upload(&db, "art-1").unwrap();
        let decoded = image::load_from_memory(&sent).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (2400, 1200), "longest side 2400 px");
        assert!(!sent.windows(10).any(|w| w == b"GPS-SECRET"), "no metadata leaves the computer");
        assert_eq!(std::fs::read(dir.path().join("archive").join("media").join("opera.jpg")).unwrap(), original, "the original is untouched");
        assert_eq!(artwork_image_for_upload(&db, "missing").unwrap_err(), "artwork_not_found");
        assert_eq!(artwork_image_for_upload(&db, "../art-1").unwrap_err(), "artwork_not_found");
        db.conn.lock().unwrap().execute("INSERT INTO artworks(id,code,title,created_at,updated_at)
            VALUES('art-2','A-2','Senza foto','t','t')", []).unwrap();
        assert_eq!(artwork_image_for_upload(&db, "art-2").unwrap_err(), "artwork_image_missing");
    }

    #[test]
    fn image_upload_posts_the_jpeg_bytes_with_the_bearer() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let read = stream.read(&mut chunk).unwrap();
                request.extend_from_slice(&chunk[..read]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
                    let length = head.lines().find_map(|l| l.strip_prefix("content-length: ").map(|v| v.trim().parse::<usize>().unwrap())).unwrap_or(0);
                    if request.len() >= end + 4 + length { break; }
                }
                if read == 0 { break; }
            }
            let body = format!("{{\"imageId\":\"{}\",\"width\":4,\"height\":3}}", "a".repeat(64));
            stream.write_all(format!("HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).unwrap();
            request
        });
        let endpoints = Endpoints { studio: format!("http://127.0.0.1:{port}"), auctions: format!("http://127.0.0.1:{port}") };
        let client = http_client(&endpoints).unwrap();
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0x00, 0x01, 0x02, 0xFF, 0xD9];
        let response = tauri::async_runtime::block_on(send_image(&client, &endpoints, "aaa.bbb.ccc", jpeg.clone())).unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.body["imageId"], "a".repeat(64));
        let request = server.join().unwrap();
        let end = request.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
        assert!(head.starts_with("post /api/artist/images http/1.1"));
        assert!(head.contains("content-type: image/jpeg"));
        assert!(head.contains("authorization: bearer aaa.bbb.ccc"));
        assert_eq!(&request[end + 4..], &jpeg[..]);
        assert_eq!(tauri::async_runtime::block_on(send_image(&client, &endpoints, "aaa.bbb.ccc", Vec::new())).unwrap_err(), "artwork_image_too_large");
    }
}
