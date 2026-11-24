# Optional offline auction drafts (development)

Schema v19 proposes two dedicated SQLite tables: archive identity (random UUID)
and one draft record per local artwork. The identity and records travel with a
verified SQLite backup; the existing backup format remains unchanged. Native
commands use the existing database mutex and a transaction with revision
compare-and-swap. They do not modify artworks, sales, collectors, or media.
Drafts are deliberately retained if a work is deleted; an unresolved request
must never be silently lost through a cascade. New drafts/attempts require an
available work.

The editor must hold the loaded snapshot and save through the native IPC port.
On a revision conflict, reload and review. Pending key and body cannot be
replaced or cleared yet. Native storage validates the field allowlist; the
TypeScript preview contract validates canonical request semantics. There is no
HTTP transport, upload, credential storage, UI, or result reconciliation yet.
Never automatically send a pending request after a restore, particularly an
older snapshot: explicit account/tenant reconciliation is needed first.
Backup inspection currently checks SQLite integrity/hash, not auction-specific
JSON semantics; the native read rejects malformed draft records.

Coordination: reserve schema 19 with core before approval; backup.rs is not
modified. This branch adds native round-trip/restore regression tests and
frontend port tests. Native PKCE now has an agreed provider-side contract in Online #5;
activation is still blocked pending isolated end-to-end tests. No pilot or production access.

## Editor session boundary

The editor session factory in `auction-native-api.ts` connects the single-work
editor state to native IPC. It loads only the selected artwork's draft, saves
explicitly using its original revision, and blocks switching/closing with
unsaved changes unless the artist explicitly discards them. Pending key/body
remain immutable and visible to the future UI; a sold work may reopen an
existing draft but cannot create a new one. Overlong source titles are not
truncated: the source title is retained and the new public-title field is
empty for explicit editing. This is an offline session controller, not a
mounted Angular form or a consent to publish. The UI entry point must be
coordinated with core PRs #30/#35/#109 before editing shared components.

The five controller tests use a mock persistence port; the earlier native
bridge tests cover record validation and immutable requests separately.

## Native PKCE contract (not enabled)

Identity handoff: [Online #5 contract at ba34bb4](https://github.com/Cesare-ml/Repertica-Studio-Online/blob/ba34bb4a64c235e9fa69e6895340678cbab3a3f7/docs/SSO_CONTRACT.md).
The isolated Rust module `auction_pkce.rs` binds literal IPv4 loopback on an
OS-assigned port, generates independent state and verifier, derives S256 and
accepts only a single valid raw callback target. The grant fixes client ID,
scope and redirect for redemption, and retains the expected audience locally.
The Rust integration test includes the synthetic provider fixture and checks
wrong state/path, duplicate fields, replay and independent bound listeners.

No Tauri command is registered for PKCE. The isolated native loopback session
now owns the grant and socket, bounds the HTTP header to 8192 bytes, requires
GET/HTTP 1.1 and exact Host including port, rejects duplicate Host, request
bodies and transfer encoding, and enforces a two-second connection read budget
and a session deadline capped at five minutes. The session closes on success,
cancellation or expiry; invalid callbacks do not consume the grant. Browser
responses contain static text only and no secrets. Added three Rust regression
tests for HTTP parsing, real loopback success after wrong Host, and socket
closure on cancellation/timeout. These tests require CI or a Rust toolchain.
Before wiring the UI, implement trusted HTTPS issuer and system browser launch,
and native token exchange without redirects or IPC token exposure. Validate token audience and scope against the intended
operation, with provider signature/expiry checks on the server. No network
activation, token persistence, upload, or production configuration in this
increment. Existing archive startup stays unchanged.

Run the new contract tests with
`cargo test --manifest-path src-tauri/Cargo.toml --test auction_pkce_contract`.
This is not the provider/browser end-to-end test.
