# Economy and localization (v0.2 development increment)

## Financial vocabulary and source

Based on the **local** domains of `Gestionale-Opere` (treasury, `uscite_finanziarie`, `categorie_uscite`, artist coefficient).

- **Artist coefficient** is an area-pricing multiplier, not the traditional Italian `(width+height)*coefficient*10` formula. Suggested price = width × height × price per cm² × multiplier.
- **Gross sales** are registered sales, **not bank receipts**.
- **Net sales** = gross minus recorded tax and shipping amounts.
- **Margin** = net sales minus recorded expenses; not a certified tax/accounting profit.
- **Allocation** divides each sale's net amount among personal, activity and savings by percentages that sum to exactly 100%.
- Allocations use basis points and integer cents; the savings share absorbs any rounding remainder so the shares always sum to the net amount.
- Allocation percentages are copied into `sale_finances` when a sale is entered. Later changes in settings affect **new** sales, not historic ones.
- **Integrity protection:** when saving a split, repeated identifiers for the same allocation part are rejected, even if the split is switched off (unfinished drafts are otherwise allowed). Before recording a new sale, the active percentages are revalidated against the current SQLite contents: there must be 1–12 parts with individually valid basis-point values adding up to exactly 10,000. A damaged or externally edited plan cannot silently freeze an incorrect historical allocation; the sale transaction must roll back. Existing sale snapshots are never rewritten.
- Expenses are assigned to activity or savings, with date, category (e.g. painting supplies, shipping, equipment, marketing), description, supplier and optional purchase link.
- Monthly statistics group sales by their recorded sale date and expenses by their expense date; they do **not** claim money has been collected or paid. Future versions can separate receivables and actual cash movements.
- `treasury_start_date` optionally filters summary calculations; it does not delete past data.

## SQLite migration

Schema 1 → 2 is additive and creates `studio_settings`, `sale_finances`, `expenses`.
Existing sales receive historic default shares 33.34% / 33.33% / 33.33% and zero recorded taxes/shipping because those fields were not previously available. No inferred historic tax or shipping.
An existing v1 database is automatically migrated on startup; `PRAGMA user_version` is updated transactionally.
The backup manifest supports schema v1 and v2; v1 backups can be restored and migrated at startup.

## Localization

- Tauri OS locale plugin reads the system locale, with `navigator.language` as fallback.
- Supported selections: English, Italian, Spanish, French, German.
- Unknown or unsupported locale resolves to English. In a supported locale, untranslated strings fall back to English.
- Language override in Settings persists in local SQLite (null = system), no network.
- UI dictionaries are deliberately in source control. New frontend text needs an English translation.
- Technical SQLite/Rust errors should eventually become stable error codes to be localized by clients; the current MVP still contains some Italian legacy errors.

## Scope

Offline-only. No Sites, SaaS, AI API, server billing, external account, invoicing, tax filing, banking or receipt reconciling. Data stays in the local SQLite archive and is included in backup snapshots.

## Test acceptance criteria

1. Existing v1 archive migrates safely; rollback on error, v1 backups accepted.
2. Invalid share allocations are rejected; accepted values add to exactly 100%.
3. Sales capture allocations; later settings changes do not rewrite old sale snapshots.
4. 12 monthly rows, correct gross/net/expense/margin and no automatic receipt claims.
5. Expenses can be entered, edited, removed and appear in reports.
6. OS language uses correct language or English fallback; override survives restart.
7. Build Angular, Rust tests, macOS DMGs; verify native visual behavior.

## CSV portability: sales and expense ledgers

Use **Sales → Export sales CSV** or **Economy → Export expenses CSV** to choose a local file with the native Save dialog (including external drives or mounted shares). Exports contain **all recorded rows** irrespective of the year selected in the dashboard, sorted by date then stable ID. The original SQLite data is never modified.

- UTF-8 with BOM, semicolon-separated fields, RFC-style double-quote escaping and CRLF records. Text that could be interpreted as a spreadsheet formula is prefixed with an apostrophe on export. Use the raw exported CSV as the interchange source; spreadsheet applications may change formatting on save.
- Monetary values are **integer cents** (e.g. `gross_cents=15001` means €150.01), allocation shares are **basis points** (5000 = 50%), with EUR currency explicitly attached to sales. No floating-point conversion.
- Sales include stable IDs and artwork/collector references, snapshot percentages, recorded taxes/shipping, net sales and notes. Expenses include stable ID, date, category, fund, amount, supplier, purchase URL and notes. These are **recorded sales and registered expenses, not confirmed bank receipts or payments**.
- This is a **one-way tabular export**, not a full portable backup or an import format. For lossless recovery, use the verified **SQLite + media backup** in Settings. CSV files may contain customer personal data and must be stored securely.
- Existing export filenames are deliberately **not overwritten**. If the selected file exists, choose a new filename. Saving inside the app's internal data directory is rejected.
- Desktop acceptance pending: native Save dialog, exported Unicode and non-ASCII paths, files on removable disks, large registers and imports into common spreadsheet applications.
