# Isolated Auctions schema composition (25 November 2026)

Draft review branch based on pinned main `fbb8faea65292dbcc9b282b8a64b29bfd4856c2e`; reuses native validators and SQL from #164 `e8bdc12d1a045c7fe2e2699373870b53c7a1ed6c`, and exact symlink preflight/tests from #117 `c62783678b651ecf8b516db3b27092928048437c`. No merge, no release, no change to the Aste UI or localStorage. This branch is a schema-only composition; it does not replace #164 or implement the combined IPC yet.

- Actual v18 -> proposed v21: default mail notifications and fresh archive scope only when absent.
- Main v19 (mail) -> v21: preserve 0/1; create the missing draft and ledger tables.
- #164 v19 (drafts) -> v21: preserve scope, record bytes, pending key/body and revision; add mail default and ledger.
- #164 v20 (drafts + ledger) -> v21: preserve history and ACK bytes, add mail default only if absent.
- Partial layouts and invalid data fail closed; no scope repair/rekey or auto-replay. DDL and the version bump for the composition run in one transaction after the existing core v18 upgrade. Backup inspection is read-only, including legacy backups; ordinary restore/open use the same compatibility gate.

Tests: cargo test --manifest-path src-tauri/Cargo.toml (NOT_RUN locally: Rust toolchain unavailable). Rust tests cover the five synthetic layouts, rollback, reopen, and actual backup/restore; CI must verify compilation and behavior. Main's cascade/certificate semantics retained; UI and IPC integration not included. Existing localStorage data is not imported, deleted or treated as authoritative. Next: explicit per-archive review/import, reconcile #164 editor and main Aste manager with a single native draft store, then isolated IPC/provider acceptance. No real provider, opener, offers, accounting, or network.
