//! Native-only PKCE contract for the optional Auctions integration.
//! Not exposed as a Tauri command; the browser boundary accepts an injected
//! opener only. No platform opener, remote transport, or provider activation.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{io, net::{Ipv4Addr, TcpListener}};
use uuid::Uuid;

const CALLBACK_PATH: &str = "/repertica/oauth/callback";

#[derive(Clone, Copy)]
pub(crate) enum GrantClient {
    Auctions,
    StudioApi,
}

impl GrantClient {
    fn id(self) -> &'static str {
        match self {
            Self::Auctions => "repertica-desktop-auctions",
            Self::StudioApi => "repertica-desktop-studio-api",
        }
    }
    fn scope(self) -> &'static str {
        match self {
            Self::Auctions => "profile",
            Self::StudioApi => "artworks:create media:write",
        }
    }
    pub(crate) fn audience(self) -> &'static str {
        match self {
            Self::Auctions => "repertica-auctions",
            Self::StudioApi => "repertica-studio-api",
        }
    }
}

// Deliberately no Debug/Clone on the pending grant or exchange: these values
// must stay in the native process and must not be logged or sent over IPC.
pub(crate) struct PendingGrant {
    client: GrantClient,
    redirect_uri: String,
    state: String,
    verifier: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AuthorizationParams<'a> {
    client_id: &'static str,
    redirect_uri: &'a str,
    state: &'a str,
    code_challenge: String,
    code_challenge_method: &'static str,
    response_type: &'static str,
    scope: &'static str,
}


// No enabled provider in non-test builds. Native selection only, with no
// environment, IPC or arbitrary URL override.
#[derive(Clone, Copy)]
pub(crate) enum IdentityProvider {
    Disabled,
    #[cfg(test)]
    IsolatedFixture,
}

impl IdentityProvider {
    fn login(self) -> Result<&'static str, &'static str> {
        match self {
            Self::Disabled => Err("oauth_provider_disabled"),
            #[cfg(test)]
            Self::IsolatedFixture => Ok("https://studio.example.test/login"),
        }
    }
}

fn query_component(value: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            write!(&mut encoded, "%{byte:02X}").expect("writing to String");
        }
    }
    encoded
}

impl AuthorizationParams<'_> {
    fn browser_url(&self, provider: IdentityProvider) -> Result<String, &'static str> {
        let login = provider.login()?;
        let fields = [
            ("client_id", self.client_id),
            ("redirect_uri", self.redirect_uri),
            ("state", self.state),
            ("code_challenge", self.code_challenge.as_str()),
            ("code_challenge_method", self.code_challenge_method),
            ("response_type", self.response_type),
            ("scope", self.scope),
        ];
        let query = fields.iter().map(|(name, value)|
            format!("{name}={}", query_component(value)))
            .collect::<Vec<_>>().join("&");
        Ok(format!("{login}?{query}"))
    }
}

#[derive(Serialize)]
pub(crate) struct TokenExchange {
    grant_type: &'static str,
    client_id: &'static str,
    redirect_uri: String,
    code: String,
    code_verifier: String,
    // Retain expected audience locally; never send it as a caller-chosen grant.
    #[serde(skip)]
    expected_client: GrantClient,
}

impl TokenExchange {
    pub(crate) fn expected_audience(&self) -> &'static str {
        self.expected_client.audience()
    }
}

fn random_hex(count: usize) -> String {
    (0..count).map(|_| Uuid::new_v4().simple().to_string()).collect::<Vec<_>>().join("")
}

fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

impl PendingGrant {
    /// Bind before obtaining authorization parameters. The caller must retain
    /// this listener, impose a deadline, validate HTTP method/Host and close it
    /// on success/cancel/error. No HTTP service or browser is launched here.
    pub(crate) fn bind(client: GrantClient) -> io::Result<(TcpListener, Self)> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = listener.local_addr()?.port();
        if port < 1024 {
            return Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "invalid_loopback_port"));
        }
        listener.set_nonblocking(true)?;
        let grant = Self::with_values(client, port, random_hex(2), random_hex(3));
        Ok((listener, grant))
    }

    // Private constructor: deterministic values are for tests only. Three
    // independent UUIDv4 values provide 366 random bits in a 96-char verifier.
    fn with_values(client: GrantClient, port: u16, state: String, verifier: String) -> Self {
        Self {
            client, redirect_uri: format!("http://127.0.0.1:{port}{CALLBACK_PATH}"),
            state, verifier: Some(verifier),
        }
    }

    pub(crate) fn authorization(&self) -> Option<AuthorizationParams<'_>> {
        Some(AuthorizationParams {
            client_id: self.client.id(), redirect_uri: &self.redirect_uri,
            state: &self.state, code_challenge: challenge(self.verifier.as_deref()?),
            code_challenge_method: "S256", response_type: "code", scope: self.client.scope(),
        })
    }

    /// Accept only the raw origin-form target generated by the agreed provider.
    /// Reject duplicate/extra fields and URL normalization. Malformed or
    /// wrong-state callbacks do not consume the pending grant. A valid callback
    /// consumes it exactly once, even if the later token exchange fails.
    pub(crate) fn accept_callback(&mut self, target: &str) -> Result<TokenExchange, &'static str> {
        if self.verifier.is_none() { return Err("oauth_grant_consumed"); }
        if target.len() > 1024 { return Err("invalid_oauth_callback"); }
        let (path, query) = target.split_once('?').ok_or("invalid_oauth_callback")?;
        if path != CALLBACK_PATH { return Err("invalid_oauth_callback"); }
        let (mut code, mut state) = (None, None);
        for part in query.split('&') {
            let (key, value) = part.split_once('=').ok_or("invalid_oauth_callback")?;
            match key {
                "code" if code.is_none() => code = Some(value),
                "state" if state.is_none() => state = Some(value),
                _ => return Err("invalid_oauth_callback"),
            }
        }
        let code = code.ok_or("invalid_oauth_callback")?;
        if !(32..=192).contains(&code.len()) ||
            !code.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_') {
            return Err("invalid_oauth_callback");
        }
        if state != Some(self.state.as_str()) { return Err("oauth_state_mismatch"); }
        let verifier = self.verifier.take().ok_or("oauth_grant_consumed")?;
        Ok(TokenExchange {
            grant_type: "authorization_code", client_id: self.client.id(),
            redirect_uri: self.redirect_uri.clone(), code: code.to_owned(),
            code_verifier: verifier, expected_client: self.client,
        })
    }
}


// A session owns both the grant and the bound socket: callers cannot pair a
// grant with a different listener. No browser or remote transport is started.
pub(crate) struct LoopbackSession {
    listener: TcpListener,
    grant: PendingGrant,
    deadline: std::time::Instant,
}

const MAX_HTTP_HEADER: usize = 8192;
const POLL: std::time::Duration = std::time::Duration::from_millis(20);
const CONNECTION_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);

fn callback_target<'a>(request: &'a [u8], expected_host: &str) -> Result<&'a str, &'static str> {
    if request.len() > MAX_HTTP_HEADER || !request.ends_with(b"\r\n\r\n") {
        return Err("invalid_oauth_http");
    }
    let request = std::str::from_utf8(request).map_err(|_| "invalid_oauth_http")?;
    let headers = request.strip_suffix("\r\n\r\n").ok_or("invalid_oauth_http")?;
    let mut lines = headers.split("\r\n");
    let mut first = lines.next().ok_or("invalid_oauth_http")?.split(' ');
    if first.next() != Some("GET") { return Err("invalid_oauth_http"); }
    let target = first.next().ok_or("invalid_oauth_http")?;
    if first.next() != Some("HTTP/1.1") || first.next().is_some() ||
        !target.starts_with('/') || target.bytes().any(|b| !b.is_ascii_graphic()) {
        return Err("invalid_oauth_http");
    }
    let mut host = None;
    let mut length_seen = false;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or("invalid_oauth_http")?;
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)) ||
            value.bytes().any(|b| b.is_ascii_control() && b != b'\t') {
            return Err("invalid_oauth_http");
        }
        let value = value.trim_matches(|c| c == ' ' || c == '\t');
        if name.eq_ignore_ascii_case("host") {
            if host.replace(value).is_some() { return Err("invalid_oauth_http"); }
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err("invalid_oauth_http");
        } else if name.eq_ignore_ascii_case("content-length") {
            if length_seen || value != "0" { return Err("invalid_oauth_http"); }
            length_seen = true;
        }
    }
    if host != Some(expected_host) { return Err("invalid_oauth_http"); }
    Ok(target)
}

fn read_callback(
    stream: &mut std::net::TcpStream,
    deadline: std::time::Instant,
    cancelled: &std::sync::atomic::AtomicBool,
) -> io::Result<Vec<u8>> {
    use std::{io::Read, sync::atomic::Ordering, time::Instant};
    stream.set_nonblocking(true)?;
    let mut request = Vec::new();
    let mut buffer = [0u8; 1024];
    loop {
        if cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "oauth_callback_interrupted"));
        }
        match stream.read(&mut buffer) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "invalid_oauth_http")),
            Ok(n) => {
                if request.len() + n > MAX_HTTP_HEADER {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid_oauth_http"));
                }
                request.extend_from_slice(&buffer[..n]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    // No body, pipelining or bytes after the header.
                    if end + 4 != request.len() {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid_oauth_http"));
                    }
                    return Ok(request);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => std::thread::sleep(POLL),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

fn browser_reply(stream: &mut std::net::TcpStream, accepted: bool) {
    use std::io::Write;
    // Static text, no reflected parameters, secrets, external resources or redirects.
    // A closed browser must not undo a valid single-use callback.
    let (status, body) = if accepted {
        ("200 OK", "Authorization response received. Return to Repertica Studio.")
    } else {
        ("400 Bad Request", "Invalid authorization response.")
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    // Nonblocking best effort; never extend the deadline to deliver UI text.
    let _ = stream.write_all(response.as_bytes());
}

impl LoopbackSession {
    pub(crate) fn bind(client: GrantClient, timeout: std::time::Duration) -> io::Result<Self> {
        let (listener, grant) = PendingGrant::bind(client)?;
        let deadline = std::time::Instant::now() + timeout.min(std::time::Duration::from_secs(300));
        Ok(Self { listener, grant, deadline })
    }

    pub(crate) fn authorization(&self) -> Option<AuthorizationParams<'_>> {
        self.grant.authorization()
    }


    /// Native worker boundary. The platform opener must be supplied by the
    /// caller after explicit opt-in and QA approval; there is no default opener.
    /// The closure must return promptly after asking the OS to open the URL.
    /// No arbitrary destination or grant fields can be supplied by the renderer.
    /// Errors are deliberately generic and never include the URL.
    pub(crate) fn open_and_wait(
        self,
        cancelled: &std::sync::atomic::AtomicBool,
        provider: IdentityProvider,
        open_browser: impl FnOnce(&str) -> Result<(), ()>,
    ) -> Result<TokenExchange, &'static str> {
        use std::{sync::atomic::Ordering, time::Instant};
        if cancelled.load(Ordering::Acquire) { return Err("oauth_cancelled"); }
        if Instant::now() >= self.deadline { return Err("oauth_timeout"); }
        let url = self.authorization().ok_or("oauth_grant_consumed")?.browser_url(provider)?;
        if cancelled.load(Ordering::Acquire) { return Err("oauth_cancelled"); }
        if Instant::now() >= self.deadline { return Err("oauth_timeout"); }
        open_browser(&url).map_err(|_| "oauth_browser_open_failed")?;
        self.wait(cancelled)
    }

    /// Blocking native worker only (never the UI thread). Consumes the session,
    /// so the listening socket closes on every exit path. Invalid requests
    /// cannot consume the grant or extend its original deadline.
    pub(crate) fn wait(
        mut self,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<TokenExchange, &'static str> {
        use std::{sync::atomic::Ordering, time::Instant};
        let host = format!("127.0.0.1:{}", self.listener.local_addr().map_err(|_| "oauth_listener_error")?.port());
        loop {
            if cancelled.load(Ordering::Acquire) { return Err("oauth_cancelled"); }
            if Instant::now() >= self.deadline { return Err("oauth_timeout"); }
            match self.listener.accept() {
                Ok((mut stream, peer)) => {
                    if peer.ip() != std::net::IpAddr::V4(Ipv4Addr::LOCALHOST) { continue; }
                    let deadline = self.deadline.min(Instant::now() + CONNECTION_LIMIT);
                    let request = match read_callback(&mut stream, deadline, cancelled) {
                        Ok(request) => request,
                        Err(_) => continue,
                    };
                    if cancelled.load(Ordering::Acquire) { return Err("oauth_cancelled"); }
                    if Instant::now() >= self.deadline { return Err("oauth_timeout"); }
                    let result = callback_target(&request, &host)
                        .and_then(|target| self.grant.accept_callback(target));
                    browser_reply(&mut stream, result.is_ok());
                    if let Ok(exchange) = result { return Ok(exchange); }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => std::thread::sleep(POLL),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return Err("oauth_listener_error"),
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    // Synthetic fixture from Online #5 at ba34bb4; RFC 7636 example verifier.
    const STATE: &str = "synthetic-state-for-desktop-12345";
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const CODE: &str = "synthetic-authorization-code-0123456789";
    const REDIRECT: &str = "http://127.0.0.1:49152/repertica/oauth/callback";

    fn fixture(client: GrantClient) -> PendingGrant {
        PendingGrant::with_values(client, 49152, STATE.into(), VERIFIER.into())
    }


    #[test]
    fn browser_url_pins_provider_and_encodes_exact_grant() {
        let auth = fixture(GrantClient::StudioApi);
        let url = auth.authorization().unwrap().browser_url(IdentityProvider::IsolatedFixture).unwrap();
        assert_eq!(url, format!(
            "https://studio.example.test/login?client_id=repertica-desktop-studio-api&redirect_uri=http%3A%2F%2F127.0.0.1%3A49152%2Frepertica%2Foauth%2Fcallback&state={STATE}&code_challenge={CHALLENGE}&code_challenge_method=S256&response_type=code&scope=artworks%3Acreate%20media%3Awrite"
        ));
        assert!(!url.contains(VERIFIER));
        assert!(!url.contains(CODE));
        assert_eq!(query_component("a&b=#% +é"), "a%26b%3D%23%25%20%2B%C3%A9");
        let auctions = fixture(GrantClient::Auctions);
        let url = auctions.authorization().unwrap().browser_url(IdentityProvider::IsolatedFixture).unwrap();
        assert!(url.contains("?client_id=repertica-desktop-auctions&"));
        assert!(url.ends_with("&scope=profile"));
    }

    #[test]
    fn injected_browser_opener_completes_local_flow_without_remote_request() {
        use std::{io::Write, net::TcpStream, sync::atomic::AtomicBool, time::Duration};
        let session = LoopbackSession::bind(GrantClient::StudioApi, Duration::from_secs(5)).unwrap();
        let addr = session.listener.local_addr().unwrap();
        let state = session.grant.state.clone();
        let result = session.open_and_wait(&AtomicBool::new(false), IdentityProvider::IsolatedFixture, |url| {
            assert!(url.starts_with("https://studio.example.test/login?"));
            assert!(url.contains(&format!("state={state}&")));
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(format!(
                "GET {CALLBACK_PATH}?code={CODE}&state={state} HTTP/1.1\r\nHost: {addr}\r\n\r\n"
            ).as_bytes()).unwrap();
            Ok(())
        }).unwrap();
        assert_eq!(result.expected_audience(), "repertica-studio-api");
        assert!(TcpStream::connect(addr).is_err());
    }

    #[test]
    fn opener_failure_cancel_and_expiry_close_listener() {
        use std::{net::TcpStream, sync::atomic::AtomicBool, time::Duration};
        for (timeout, cancelled, expected) in [
            (Duration::from_secs(5), false, "oauth_browser_open_failed"),
            (Duration::from_secs(5), true, "oauth_cancelled"),
            (Duration::ZERO, false, "oauth_timeout"),
        ] {
            let session = LoopbackSession::bind(GrantClient::Auctions, timeout).unwrap();
            let addr = session.listener.local_addr().unwrap();
            let mut opened = false;
            let result = session.open_and_wait(&AtomicBool::new(cancelled), IdentityProvider::IsolatedFixture, |_| {
                opened = true;
                Err(())
            });
            assert_eq!(result.err(), Some(expected));
            assert_eq!(opened, expected == "oauth_browser_open_failed");
            assert!(TcpStream::connect(addr).is_err());
        }
    }

    #[test]
    fn disabled_provider_never_calls_opener() {
        use std::{net::TcpStream, sync::atomic::AtomicBool, time::Duration};
        let session = LoopbackSession::bind(GrantClient::Auctions, Duration::from_secs(5)).unwrap();
        let addr = session.listener.local_addr().unwrap();
        assert_eq!(session.open_and_wait(&AtomicBool::new(false), IdentityProvider::Disabled,
            |_| panic!("disabled provider must never open a browser")).err(),
            Some("oauth_provider_disabled"));
        assert!(TcpStream::connect(addr).is_err());
    }

    #[test]
    fn provider_fixture_and_separate_grants() {
        for (client, id, audience, scope) in [
            (GrantClient::Auctions, "repertica-desktop-auctions", "repertica-auctions", "profile"),
            (GrantClient::StudioApi, "repertica-desktop-studio-api", "repertica-studio-api", "artworks:create media:write"),
        ] {
            let mut grant = fixture(client);
            let auth = serde_json::to_value(grant.authorization().unwrap()).unwrap();
            assert_eq!(auth["client_id"], id);
            assert_eq!(auth["redirect_uri"], REDIRECT);
            assert_eq!(auth["state"], STATE);
            assert_eq!(auth["scope"], scope);
            assert_eq!(auth["code_challenge"], CHALLENGE);
            assert_eq!(auth["code_challenge_method"], "S256");
            assert!(auth.get("code_verifier").is_none());
            let target = format!("{CALLBACK_PATH}?state={STATE}&code={CODE}");
            let exchange = grant.accept_callback(&target).unwrap();
            assert_eq!(exchange.expected_audience(), audience);
            let body = serde_json::to_value(exchange).unwrap();
            assert_eq!(body["client_id"], id);
            assert_eq!(body["redirect_uri"], REDIRECT);
            assert_eq!(body["grant_type"], "authorization_code");
            assert_eq!(body["code"], CODE);
            assert_eq!(body["code_verifier"], VERIFIER);
            assert_eq!(body.as_object().unwrap().len(), 5);
            assert!(grant.authorization().is_none());
            assert_eq!(grant.accept_callback(&target).err(), Some("oauth_grant_consumed"));
        }
    }

    #[test]
    fn malformed_callbacks_do_not_consume_grant() {
        let mut grant = fixture(GrantClient::Auctions);
        for target in [
            format!("/wrong?code={CODE}&state={STATE}"),
            format!("/repertica/%6fauth/callback?code={CODE}&state={STATE}"),
            format!("{REDIRECT}?code={CODE}&state={STATE}"),
            format!("{CALLBACK_PATH}?code={CODE}&state=wrong"),
            format!("{CALLBACK_PATH}?code={CODE}&state={STATE}&state={STATE}"),
            format!("{CALLBACK_PATH}?code={CODE}&code={CODE}&state={STATE}"),
            format!("{CALLBACK_PATH}?code={CODE}&state={STATE}&extra=1"),
            format!("{CALLBACK_PATH}?code=short&state={STATE}"),
            format!("{CALLBACK_PATH}?code={CODE}%2F&state={STATE}"),
            format!("{CALLBACK_PATH}?code={CODE}&state={STATE}#fragment"),
            format!("{CALLBACK_PATH}?code={CODE}"),
        ] {
            assert!(grant.accept_callback(&target).is_err());
            assert!(grant.authorization().is_some());
        }
        assert!(grant.accept_callback(&format!("{CALLBACK_PATH}?code={CODE}&state={STATE}")).is_ok());
    }

    #[test]
    fn bound_listener_and_fresh_independent_grants() {
        let (listener, first) = PendingGrant::bind(GrantClient::Auctions).unwrap();
        let (second_listener, second) = PendingGrant::bind(GrantClient::StudioApi).unwrap();
        let addr = listener.local_addr().unwrap();
        assert_eq!(addr.ip(), std::net::IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert!(addr.port() >= 1024);
        assert_ne!(addr.port(), second_listener.local_addr().unwrap().port());
        assert_eq!(first.redirect_uri, format!("http://127.0.0.1:{}{CALLBACK_PATH}", addr.port()));
        assert_ne!(first.state, second.state);
        assert_ne!(first.verifier, second.verifier);
        assert_eq!(first.verifier.as_ref().unwrap().len(), 96);
        assert_eq!(first.authorization().unwrap().code_challenge.len(), 43);
    }

    #[test]
    fn http_parser_requires_exact_host_get_and_bounded_headers() {
        let host = "127.0.0.1:49152";
        let target = format!("{CALLBACK_PATH}?code={CODE}&state={STATE}");
        let good = format!("GET {target} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: test\r\n\r\n");
        assert_eq!(callback_target(good.as_bytes(), host), Ok(target.as_str()));
        for bad in [
            good.replace("GET ", "POST "),
            good.replace(host, "localhost:49152"),
            good.replace(host, "127.0.0.1:49153"),
            good.replace("Host:", " Host:"),
            good.replace("User-Agent: test", &format!("hOsT: {host}")),
            good.replace("User-Agent: test", "Transfer-Encoding: chunked"),
            good.replace("User-Agent: test", "Content-Length: 1"),
            good.replace("User-Agent: test", "Content-Length: 0\r\nContent-Length: 0"),
            good.replace("\r\n", "\n"),
            format!("{good}extra"),
            good.replace("test", &"x".repeat(MAX_HTTP_HEADER)),
        ] {
            assert!(callback_target(bad.as_bytes(), host).is_err());
        }
    }

    #[test]
    fn loopback_wait_rejects_wrong_host_then_accepts_once() {
        use std::{io::{Read, Write}, net::TcpStream, sync::atomic::AtomicBool, time::Duration};
        let session = LoopbackSession::bind(GrantClient::Auctions, Duration::from_secs(5)).unwrap();
        let addr = session.listener.local_addr().unwrap();
        let state = session.grant.state.clone();
        let worker = std::thread::spawn(move || session.wait(&AtomicBool::new(false)));
        let target = format!("{CALLBACK_PATH}?code={CODE}&state={state}");
        for (host, expected) in [("localhost:49152".to_owned(), "400"), (addr.to_string(), "200")] {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            stream.write_all(format!("GET {target} HTTP/1.1\r\nHost: {host}\r\n\r\n").as_bytes()).unwrap();
            let mut reply = String::new();
            stream.read_to_string(&mut reply).unwrap();
            assert!(reply.starts_with(&format!("HTTP/1.1 {expected}")));
            assert!(!reply.contains(CODE));
            assert!(!reply.contains(&state));
        }
        let exchange = worker.join().unwrap().unwrap();
        assert_eq!(exchange.expected_audience(), "repertica-auctions");
        assert!(TcpStream::connect(addr).is_err());
    }

    #[test]
    fn loopback_wait_closes_on_cancel_and_timeout() {
        use std::{net::TcpStream, sync::atomic::AtomicBool, time::Duration};
        let session = LoopbackSession::bind(GrantClient::Auctions, Duration::from_secs(5)).unwrap();
        let addr = session.listener.local_addr().unwrap();
        assert_eq!(session.wait(&AtomicBool::new(true)).err(), Some("oauth_cancelled"));
        assert!(TcpStream::connect(addr).is_err());

        let session = LoopbackSession::bind(GrantClient::Auctions, Duration::ZERO).unwrap();
        let addr = session.listener.local_addr().unwrap();
        assert_eq!(session.wait(&AtomicBool::new(false)).err(), Some("oauth_timeout"));
        assert!(TcpStream::connect(addr).is_err());
    }
}

#[cfg(test)]
#[path = "auction_pkce_fixture.rs"]
mod fixture_exchange;
