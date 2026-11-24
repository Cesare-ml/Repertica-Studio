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
