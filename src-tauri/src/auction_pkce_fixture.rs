//! Native-only synthetic provider harness. No HTTP client, real token, key,
//! account, DNS resolution or cryptographic verification is used here.
//! The production provider remains disabled until separate QA approval.
use super::*;
use std::{collections::HashSet, time::{Duration, Instant}};

const ISSUER: &str = "https://studio.example.test";
const ENDPOINT: &str = "https://studio.example.test/api/identity/token";
const CODE: &str = "synthetic-authorization-code-0123456789";
const STATE: &str = "synthetic-state-for-desktop-12345";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const NOW: u64 = 1_800_000_000;

// Represents synthetic claims after a hypothetical trusted verifier. Never
// deserialize unverified JWTs into this type. Real RS256/kid/expiry checks
// and hard transport deadlines are release gates, not covered by this harness.
struct Claims {
    issuer: &'static str,
    audience: &'static str,
    scope: &'static str,
    token_use: &'static str,
    security_version: u64,
    iat: u64,
    exp: u64,
}

fn claims(client: GrantClient) -> Claims {
    Claims { issuer: ISSUER, audience: client.audience(), scope: client.scope(),
        token_use: "access", security_version: 1, iat: NOW, exp: NOW + 900 }
}

fn exchange(client: GrantClient) -> TokenExchange {
    PendingGrant::with_values(client, 49152, STATE.into(), VERIFIER.into())
        .accept_callback(&format!("{CALLBACK_PATH}?code={CODE}&state={STATE}")).unwrap()
}

fn deadline() -> Instant { Instant::now() + Duration::from_secs(5) }

// Consumes the grant on every exit. No retry/redirect following. A future
// native transport must enforce a hard I/O deadline; these pre/post checks
// alone cannot interrupt an uncooperative transport.
fn redeem(
    grant: TokenExchange,
    provider: IdentityProvider,
    destination: &str,
    deadline: Instant,
    transport: impl FnOnce(&str, &TokenExchange) -> Result<(u16, Claims), &'static str>,
) -> Result<(), &'static str> {
    provider.login()?;
    if destination != ENDPOINT { return Err("oauth_destination_denied"); }
    if Instant::now() >= deadline { return Err("oauth_timeout"); }
    let response = transport(ENDPOINT, &grant);
    if Instant::now() >= deadline { return Err("oauth_timeout"); }
    let (status, c) = response?;
    if status != 200 { return Err("oauth_token_rejected"); }
    if c.issuer != ISSUER || c.audience != grant.expected_client.audience() ||
        c.scope != grant.expected_client.scope() || c.token_use != "access" ||
        c.security_version == 0 || c.iat > NOW || c.exp <= NOW ||
        c.exp.checked_sub(c.iat).map_or(true, |ttl| ttl > 900) {
        return Err("oauth_claims_rejected");
    }
    Ok(())
}

#[test]
fn fixture_destination_allowlist_and_redirect_rejection() {
    for destination in [
        "https://studio.example.test.evil.test/api/identity/token",
        "http://studio.example.test/api/identity/token",
        "https://studio.example.test:443/api/identity/token",
        "https://studio.example.test/api/identity/token?redirect=1",
        "https://studio.example.test/api/identity/token#x",
        "https://studio.example.test@evil.test/api/identity/token",
        "https://studio.repertica.com/api/identity/token",
    ] {
        assert_eq!(redeem(exchange(GrantClient::Auctions), IdentityProvider::IsolatedFixture,
            destination, deadline(), |_, _| panic!("denied destination reached transport")),
            Err("oauth_destination_denied"));
    }
    assert_eq!(redeem(exchange(GrantClient::Auctions), IdentityProvider::Disabled,
        ENDPOINT, deadline(), |_, _| panic!("disabled provider reached transport")),
        Err("oauth_provider_disabled"));
    for status in [301, 302, 303, 307, 308, 400, 503] {
        let mut calls = 0;
        assert_eq!(redeem(exchange(GrantClient::Auctions), IdentityProvider::IsolatedFixture,
            ENDPOINT, deadline(), |endpoint, _| { calls += 1; assert_eq!(endpoint, ENDPOINT);
                Ok((status, claims(GrantClient::Auctions))) }), Err("oauth_token_rejected"));
        assert_eq!(calls, 1);
    }
}

#[test]
fn fixture_claims_reject_cross_audience_scope_and_invalid_lifetime() {
    for (client, other) in [(GrantClient::Auctions, GrantClient::StudioApi),
        (GrantClient::StudioApi, GrantClient::Auctions)] {
        for case in 0..8 {
            let mut c = claims(client);
            match case {
                0 => c.audience = other.audience(),
                1 => c.scope = other.scope(),
                2 => c.issuer = "https://other.example.test",
                3 => c.token_use = "refresh",
                4 => c.security_version = 0,
                5 => c.exp = NOW,
                6 => c.iat = NOW + 1,
                _ => c.exp = NOW + 901,
            }
            assert_eq!(redeem(exchange(client), IdentityProvider::IsolatedFixture,
                ENDPOINT, deadline(), |_, _| Ok((200, c))), Err("oauth_claims_rejected"));
        }
        assert_eq!(redeem(exchange(client), IdentityProvider::IsolatedFixture,
            ENDPOINT, deadline(), |_, _| Ok((200, claims(client)))), Ok(()));
    }
}

#[test]
fn fixture_redemption_replay_and_expired_request() {
    let mut used_codes = HashSet::new();
    for expected in [Ok(()), Err("oauth_invalid_grant")] {
        assert_eq!(redeem(exchange(GrantClient::StudioApi), IdentityProvider::IsolatedFixture,
            ENDPOINT, deadline(), |_, grant| {
                assert_eq!(grant.client_id, "repertica-desktop-studio-api");
                assert_eq!(grant.redirect_uri, "http://127.0.0.1:49152/repertica/oauth/callback");
                assert_eq!(challenge(&grant.code_verifier), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
                if !used_codes.insert(grant.code.clone()) { return Err("oauth_invalid_grant"); }
                Ok((200, claims(GrantClient::StudioApi)))
            }), expected);
    }
    assert_eq!(redeem(exchange(GrantClient::Auctions), IdentityProvider::IsolatedFixture,
        ENDPOINT, Instant::now(), |_, _| panic!("expired request reached transport")),
        Err("oauth_timeout"));
    // The fake reply arriving after a deadline cannot yield success.
    let expires = Instant::now() + Duration::from_millis(50);
    assert_eq!(redeem(exchange(GrantClient::Auctions), IdentityProvider::IsolatedFixture,
        ENDPOINT, expires, |_, _| { std::thread::sleep(Duration::from_millis(60));
            Ok((200, claims(GrantClient::Auctions))) }), Err("oauth_timeout"));
}


// Test-only model of Identity #5's atomic code-store predicate. Unlike the
// earlier transport closure, it rejects altered grant fields instead of
// asserting on them. It is not a database, signature verifier or HTTP server.
struct BoundCode {
    client: GrantClient,
    code_hash: [u8; 32],
    client_id: &'static str,
    redirect_uri: String,
    challenge: String,
    security_version: u64,
    expires_at: u64,
    used: bool,
}

impl BoundCode {
    fn issue(grant: &TokenExchange, now: u64, version: u64) -> Self {
        Self {
            client: grant.expected_client,
            code_hash: Sha256::digest(grant.code.as_bytes()).into(),
            client_id: grant.client_id,
            redirect_uri: grant.redirect_uri.clone(),
            challenge: challenge(&grant.code_verifier),
            security_version: version,
            expires_at: now + 90,
            used: false,
        }
    }

    fn redeem(&mut self, grant: &TokenExchange, now: u64, current_version: u64,
              verified: bool) -> Result<(u16, Claims), &'static str> {
        let code_hash: [u8; 32] = Sha256::digest(grant.code.as_bytes()).into();
        if self.used || !verified || current_version == 0 ||
            current_version != self.security_version || now >= self.expires_at ||
            grant.grant_type != "authorization_code" ||
            code_hash != self.code_hash || grant.client_id != self.client_id ||
            grant.redirect_uri != self.redirect_uri ||
            challenge(&grant.code_verifier) != self.challenge {
            return Err("oauth_invalid_grant");
        }
        self.used = true;
        let mut c = claims(self.client);
        c.security_version = self.security_version;
        Ok((200, c))
    }
}

#[test]
fn fixture_code_binding_rejects_wrong_port_client_verifier_and_code() {
    for (client, other) in [(GrantClient::Auctions, GrantClient::StudioApi),
                           (GrantClient::StudioApi, GrantClient::Auctions)] {
        let mut store = BoundCode::issue(&exchange(client), NOW, 1);
        for case in 0..5 {
            let mut grant = exchange(client);
            match case {
                0 => grant.redirect_uri = "http://127.0.0.1:49153/repertica/oauth/callback".into(),
                1 => grant.client_id = other.id(),
                2 => grant.code_verifier = "B".repeat(43),
                3 => grant.code = "different-synthetic-authorization-code".into(),
                _ => grant.grant_type = "refresh_token",
            }
            assert_eq!(redeem(grant, IdentityProvider::IsolatedFixture, ENDPOINT, deadline(),
                |_, g| store.redeem(g, NOW, 1, true)), Err("oauth_invalid_grant"));
            assert!(!store.used);
        }
        assert_eq!(redeem(exchange(client), IdentityProvider::IsolatedFixture, ENDPOINT, deadline(),
            |_, g| store.redeem(g, NOW + 89, 1, true)), Ok(()));
        assert!(store.used);
        assert_eq!(redeem(exchange(client), IdentityProvider::IsolatedFixture, ENDPOINT, deadline(),
            |_, g| store.redeem(g, NOW + 89, 1, true)), Err("oauth_invalid_grant"));
    }
}

#[test]
fn fixture_code_binding_denies_expired_or_revoked_grants() {
    for client in [GrantClient::Auctions, GrantClient::StudioApi] {
        for (now, current_version, verified) in [
            (NOW + 90, 1, true),
            (NOW, 2, true),
            (NOW, 0, true),
            (NOW, 1, false),
        ] {
            let mut store = BoundCode::issue(&exchange(client), NOW, 1);
            assert_eq!(redeem(exchange(client), IdentityProvider::IsolatedFixture, ENDPOINT, deadline(),
                |_, g| store.redeem(g, now, current_version, verified)), Err("oauth_invalid_grant"));
            assert!(!store.used);
        }
    }
}


#[test]
fn fixture_code_uses_issued_audience_not_local_expected_audience() {
    for (client, other) in [(GrantClient::Auctions, GrantClient::StudioApi),
                           (GrantClient::StudioApi, GrantClient::Auctions)] {
        let mut store = BoundCode::issue(&exchange(client), NOW, 1);
        let mut grant = exchange(client);
        grant.expected_client = other;
        assert_eq!(redeem(grant, IdentityProvider::IsolatedFixture, ENDPOINT, deadline(),
            |_, g| store.redeem(g, NOW, 1, true)), Err("oauth_claims_rejected"));
        // The provider already redeemed the code; no silent retry is permitted.
        assert!(store.used);
        assert_eq!(redeem(exchange(client), IdentityProvider::IsolatedFixture, ENDPOINT, deadline(),
            |_, g| store.redeem(g, NOW, 1, true)), Err("oauth_invalid_grant"));
    }
}
