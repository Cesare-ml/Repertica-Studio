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
frontend port tests. Native PKCE is blocked on the provider's loopback callback
and audience agreement. No pilot or production access.

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
