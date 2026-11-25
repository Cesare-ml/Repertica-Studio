//! Shared native allowlist for a persisted, explicitly reviewed preview request.
use serde::{Deserialize, Serialize};

pub(super) fn bounded(s: &str, max: usize) -> bool { s.encode_utf16().count() <= max }

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Payload {
    pub(super) tenant_id: String, pub(super) slug: String, pub(super) title: String, pub(super) description: String,
    pub(super) starts_at: String, pub(super) ends_at: String,
    pub(super) starting_amount: i64, pub(super) minimum_increment_amount: i64, pub(super) currency: String,
}

pub(super) fn valid_token(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

// Match the editor's ECMAScript trim semantics (not Rust's Unicode
// whitespace set), so a reviewed draft and its pending payload agree.
pub(super) fn editor_trim(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, '\u{0009}'..='\u{000d}' | '\u{0020}' |
        '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' |
        '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'))
}

pub(super) fn valid_slug(s: &str) -> bool {
    !s.is_empty() && s.len() <= 60 && s.split('-').all(|part|
        !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

pub(super) fn utc_millis(s: &str) -> Option<i64> {
    // Require the same exact millisecond UTC form as the editor. Reject
    // leap seconds, which chrono can represent but JavaScript Date cannot.
    let bytes = s.as_bytes();
    if bytes.len() != 24 || bytes.get(17..19)? > &b"59"[..] ||
        !bytes[..4].iter().all(u8::is_ascii_digit) { return None; }
    let parsed = chrono::DateTime::parse_from_rfc3339(s).ok()?;
    if parsed.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true) != s {
        return None;
    }
    Some(parsed.timestamp_millis())
}

pub(super) fn valid_pending_payload(b: &Payload) -> bool {
    let Some(start) = utc_millis(&b.starts_at) else { return false };
    let Some(end) = utc_millis(&b.ends_at) else { return false };
    valid_token(&b.tenant_id,3,80) && b.currency == "EUR" &&
        (1..=1_000_000_000).contains(&b.starting_amount) &&
        (1..=1_000_000_000).contains(&b.minimum_increment_amount) &&
        b.title == editor_trim(&b.title) && b.title.encode_utf16().count() >= 3 &&
        bounded(&b.title,160) && b.description == editor_trim(&b.description) &&
        bounded(&b.description,1500) && valid_slug(&b.slug) &&
        end > start && end - start <= 7 * 86_400_000
}

/// Keep the exact same public-field order and compact JSON as preparePreview.
pub(super) fn parse_canonical_payload(body: &str) -> Result<Payload, &'static str> {
    if body.len() > 8192 { return Err("invalid_preview_payload"); }
    let value: Payload = serde_json::from_str(body).map_err(|_| "invalid_preview_payload")?;
    if !valid_pending_payload(&value) ||
        serde_json::to_string(&value).map_err(|_| "invalid_preview_payload")? != body {
        return Err("invalid_preview_payload");
    }
    Ok(value)
}
