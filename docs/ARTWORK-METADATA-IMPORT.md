# Local artwork metadata import (first slice of issue #5)

Settings → Import artwork JSON. Explicitly select a UTF-8 JSON file, preview all codes and titles, then confirm. The exact source is held in memory; native code verifies its SHA-256 and repeats all checks inside a SQLite transaction. No network, account, file paths from the package, migration, or overwrite.

Example (synthetic):
```json
{
  "format": "repertica-studio-artwork-metadata-v1",
  "artworks": [
    {"code":"ART-0100","title":"Example","status":"available","year":"2024","width_cm":30,"height_cm":40}
  ]
}
```

Required record fields: code (max 80 characters), title (max 240), status (available, reserved, archived; sold is rejected). Optional: year, technique, support, width_cm, height_cm, depth_cm, description, location, signed, final_varnish, framed, artist_coefficient_override, fixed_price_cents. Optional booleans default to false; missing other optional fields remain null. Unknown fields are rejected, not silently ignored. Max 1000 records and 2 MiB. Codes are preserved and compared using SQLite NOCASE; IDs and timestamps are generated locally. Existing codes or duplicates in the package block the whole operation. Failed validation or SQLite writes roll back the batch. Imported codes that match native global or yearly numbering also advance the corresponding persistent high-water mark in the same transaction, without decreasing an existing counter. Deleting an imported work cannot cause automatic numbering to reuse its number, including after separator or padding changes. Other legacy codes are preserved without creating a counter. Back up before import.

This is intentionally NOT the full Gestionale Opere archive format, a CSV round trip, a backup, or the shared Auctions artwork contract. No imported photos, collectors, sales, certificates, original UUIDs or relations; these require the separately coordinated portable exporter and transactional media migration. Do not convert sold works to available merely to bypass this limitation.

Tests: npm run test:unit; cargo test --manifest-path src-tauri/Cargo.toml archive_import. Full native acceptance requires installed Tauri desktop.

## Convert a selected Gestionale Opere metadata JSON file

The standalone adapter runs locally with Node.js 22+ and no packages:

```sh
node scripts/convert-gestionale-artwork-metadata.mjs selected.json import.json
node --test tests/gestionale-metadata-converter.test.mjs
```

Supply a UTF-8 JSON array explicitly prepared from the artist's own selected local records (not a database dump, API response containing additional fields, or a live connection). The script accepts only this projection of the legacy artwork fields; see the legacy `frontend/src/app/services/opere.service.ts` and archive import service for their meaning.

```json
[
  {
    "codice_archivio": "TEST-001",
    "titolo": "Synthetic example",
    "anno_realizzazione": 2024,
    "tecnica": "Oil",
    "supporto": "Canvas",
    "larghezza_cm": "30.25",
    "altezza_cm": "40",
    "profondita_cm": null,
    "descrizione": "Synthetic description",
    "stato": "DISPONIBILE",
    "firma_presente": true,
    "cornice_presente": false,
    "vernice_finale": "SATINATA",
    "coefficiente_artista_override": null,
    "prezzo_personalizzato": "25.29",
    "id_vendita": null
  }
]
```

Only `DISPONIBILE` and `RISERVATA` with an explicit `id_vendita: null` are accepted; omitted sale references and duplicate JSON object keys (including escaped spellings) are rejected before output; sales, donations, private collection and ambiguous states require the later historical migration. Do not remove sale references or change a real artwork's state to bypass this gate. The converter accepts explicitly known signature/frame booleans and the three known varnish finishes; because the target cannot represent unknown facts, resolve null values with the artist before conversion (use literal false for confirmed absence of varnish in the prepared file, not in the source database). The varnish finish itself is not preserved, only applied/not applied. Numeric strings for dimensions and decimal EUR prices are accepted; prices convert to integer cents without rounding. No extra private notes, owners, image URLs, customer data or credentials are accepted. Source and result each have a 2 MiB limit; the result is written only to a new file, never over the source or an existing export. Both CLIs now stage and sync the complete package in the destination directory before publishing it via a no-overwrite hard link. Choose a trusted, OS-protected local directory on a filesystem supporting hard links (e.g. NTFS, APFS, ext4); unsupported destinations fail rather than falling back to an unsafe copy. Windows permissions follow the directory ACL. After an interruption a `.repertica-export-*` staging directory may remain; do not import it. This is atomic publication, not a guarantee of survival after power loss.

The adapter does not query or change Gestionale Opere and does not yet produce the selected source projection from its UI. It is not an automatic migration. Open the resulting file using the local Studio Settings preview, check codes/titles/states, and back up before confirming. Legacy media, sales, certificates, localized titles and other non-portable metadata remain in the original archive. A source-side explicit export UI and the relational/media migration are separate increments.


## Optional source-side exporter from a local PostgreSQL copy (technical preview)

For developers/administrators only, Node.js 22.16+ and PostgreSQL's `psql` on PATH. Prepare a **separate local copy** of the legacy archive, with a read-only role able to see all artwork and sale links in that copy (including inconsistent links). No pilot, remote database, SSH tunnel or production credentials. The script does not create the copy, roles or password file. Use OS-protected default local PostgreSQL password-file authentication if required; do not put passwords on the command line.

Example with synthetic identifiers (replace with explicitly selected owner and artwork IDs in your local copy):

```sh
node scripts/export-gestionale-artwork-metadata.mjs --local-copy archive_copy reader 5432 7 1,2 import.json
node --test tests/gestionale-source-export.test.mjs
```

The script uses a fixed loopback connection, ignores inherited PG* connection overrides, uses a bounded read-only transaction and writes a new Studio package only after validating all rows. One unknown, missing, other-owner, sold, sale-linked or unsupported record rejects the entire selection. Source owner IDs are a selection filter for a trusted local operator, **not** an authentication boundary; never expose this CLI as a service. No owner IDs, sale identifiers, contacts, images, private notes or connection details appear in the resulting package. Prices are read as decimal text and converted to exact integer cents. A null varnish value is unresolved, not automatically false; for an explicitly confirmed absence use the previous manual projection workflow without altering the source database. The previous JSON-only converter still works. Review the package in Studio's existing preview before confirming.

No source repository change, migration, runtime dependency in the desktop, or remote access is required. The selection and conversion tests use an adapted SQL CTE in SQLite and synthetic rows, not PostgreSQL. A real disposable PostgreSQL copy / psql run and installed-desktop acceptance remain NOT_RUN until available. This is an operator CLI, not a source UI or full historical/media migration.


## Historical relational package v2 (local, manually prepared)

The same Settings picker accepts `repertica-studio-archive-relations-v2`. v1 remains strictly metadata-only. v2 creates new local IDs and links new artworks, collectors, sales, financial details and explicit receipts in one SQLite transaction. Every sold artwork must have exactly one sale; other statuses cannot have sales. The preview displays collector keys, names, emails, linked artwork codes, sale dates and amounts, and individual receipt dates and amounts. Codes, collector names and emails are checked again immediately before importing; matching names or emails (ASCII case insensitive) reject the whole package rather than merging contacts. No existing record is overwritten. Make a verified backup first. Example (synthetic):

```json
{
  "format": "repertica-studio-archive-relations-v2",
  "artworks": [
    {
      "code": "HIST-01",
      "title": "Example",
      "status": "sold"
    }
  ],
  "collectors": [
    {
      "key": "legacy-1",
      "first_name": "Ada",
      "last_name": "Example",
      "email": "ada@example.invalid"
    }
  ],
  "sales": [
    {
      "artwork_code": "HIST-01",
      "collector_key": "legacy-1",
      "amount_cents": 10000,
      "currency": "EUR",
      "sold_at": "2025-03-04",
      "tax_cents": 1000,
      "shipping_cents": 500,
      "legacy_shares_bp": [
        2000,
        3000,
        5000
      ],
      "allocations": [],
      "installments": [],
      "receipts": [
        {
          "received_at": "2025-03-10",
          "amount_cents": 4000,
          "method": "bank_transfer",
          "reference": "synthetic"
        }
      ]
    }
  ]
}
```

Explicitly supply sale currency EUR, canonical dates, exact integer cents, taxes/shipping, three legacy finance percentages in basis points (sum 10000), receipts (empty means confirmed none), an empty allocations array and an explicit installments array (empty means no plan). Current Studio allocation settings are **not** applied retroactively; explicit legacy percentages are preserved as compatibility fields, not converted into active allocations. Receipt method must be bank_transfer, cash, card or other; total receipts cannot exceed the sale amount. Collector key is an ASCII alphanumeric, hyphen or underscore identifier up to 80 characters, scoped to this package; optional collector fields are last_name, email, phone, address and notes. Sale/receipt notes and receipt reference are optional. Unknown fields, unresolved links, invalid dates, duplicate codes, ambiguous collector contacts, nonempty allocations, invalid or paid installment plans, and unsupported history reject the entire package. Limits: 2 MiB, 1000 artworks, collectors, sales, total receipts and total installments each. Media, certificates, receipt attachments, collector photos, historical allocation mapping and installment plans with any receipts are not yet supported. Never discard original data to make a record pass this format. Use a separately prepared explicit projection from a local copy; no production or pilot data is accessed by Studio. This is not a complete archive migration.

## Export relazionale v2 da copia locale (subset esplicito)

```sh
node scripts/export-gestionale-archive-relations.mjs --local-copy ARCHIVE_COPY READONLY_USER 5432 OWNER_ID ARTWORK_IDS SHARES.json NEW_OUTPUT.json
```

Only use a local isolated PostgreSQL copy and a read-only role; host is fixed to 127.0.0.1. No live pilot, credentials, network service or database changes. The explicitly supplied UTF-8 SHARES.json must contain one entry per selected sold work, e.g. `[{"artwork_code":"TEST-1","legacy_shares_bp":[5000,3000,2000]}]`. Values must be independently confirmed by the user from historical records: legacy treasury computes splits from settings and cannot establish historical shares. Never infer them from current settings. Use [] for an entirely unsold selection.

The extractor refuses ambiguous/cross-owner or non-sale links, multiple sales per artwork, installment schedules with any receipts, receipts linked to schedules, known time of sale, sale-level payment/channel/campaign, and collector photos. Receipt methods must map explicitly (Bonifico, Contanti, Carta, Altro); unknown methods are rejected. Money is converted from exact decimal strings to integer cents. The v2 importer still validates and previews the entire file. This remains a partial export: artwork media, certificates, collector photos, paid/partially paid installment schedules and other historical information are not migrated; retain the original archive and do not treat this as a backup. Test against a real isolated PostgreSQL fixture before use. No source data is committed.

## Unpaid historical installment plans

The relational format accepts explicit unpaid schedules, without changing the schema. Example for a sale of 10000 cents:

```json
"installments": [
  {"position":1,"due_date":"2025-03-31","amount_cents":4029,"notes":"First installment"},
  {"position":2,"due_date":"2025-04-30","amount_cents":5971,"notes":null}
],
"receipts": []
```

Positions must be contiguous from 1, due dates canonical and nondecreasing, amounts positive and totaling the sale price exactly, notes at most 500 characters, 2–60 installments per plan and at most 1000 installments across a package. The importer previews each due date and amount, inserts the entire plan in the same SQLite transaction as the artwork, collector and sale, and checks each stored installment before committing. It does not generate receipts. The local-copy PostgreSQL exporter selects schedules by legacy `numero`, `data_scadenza`, `importo_previsto` and `note`, using the same constraints. It rejects plans with **any** receipts because legacy receipts may be assigned to specific installments and Studio applies receipts in due-date order; these histories require an explicitly reconciled migration. Do not strip receipts to make a plan importable. After import, Studio's ordinary receipt workflow is unchanged. Real PostgreSQL and native desktop acceptance must still be tested. No new runtime dependencies.

## Extended collector profiles

The same v2 package optionally preserves birth date (YYYY-MM-DD), birth time (HH:MM), birthplace, Instagram, Facebook, TikTok, website and LinkedIn. The local PostgreSQL projection includes these fields for the explicitly selected artwork's collector; seconds or fractions of a second that cannot be represented in Studio are rejected, never rounded. Existing packages omitting these fields still import. The local preview displays the supplied profile values as plain text, not links, and the importer verifies every field before committing. Profile/gallery photos remain unsupported and the SQL exporter still rejects collectors with such photos. Treat exported files as private: they can contain sensitive personal information. No upload or sync occurs.

## Explicit artwork covers (v2 only; initial limits superseded below)

A manually prepared v2 package can include an optional top-level `covers` array: one entry per selected artwork, exact `artwork_code`, lowercase SHA-256 digest `sha256`, and canonical standard base64 `data_base64` containing the original JPEG, PNG or WebP bytes. No paths/URLs accepted. Limit 1 MiB per cover, 16 megapixels, and unchanged 2 MiB for the entire JSON including base64. The preview displays code, byte count and digest, not image data. The native importer validates all images before writing, stores the bytes unchanged and links each cover inside the existing SQLite transaction. It rechecks the stored bytes and references before commit; on reported failure it rolls back and removes only files created by this attempt. Like ordinary local file operations, this is not a crash-atomic guarantee across the database and filesystem. The existing media catalogue registers uncatalogued artwork references on opening. No image is downloaded. Gallery photos, collector photos, RAW, certificate photos, larger packages and automatic discovery/export of Gestionale media remain unsupported. Keep the original archive and verified backups. This is not a full migration.

For an already-exported v2 package, choose the original cover files manually from a trusted local copy (never a live pilot):

```sh
node scripts/add-local-artwork-covers.mjs export.json export-with-covers.json ART-001 cover.png ART-002 cover2.jpg
node --test tests/add-local-artwork-covers.test.mjs
```

The command refuses existing output, repeated/unknown artwork codes, symlink final inputs, oversized files and unsupported image headers; the input is never changed. It generates the SHA-256 manifest and embeds the selected bytes without their source paths. It does not decode images itself; the native Studio preview is the full validation gate. Use a trusted local directory on a filesystem supporting hard links. This is explicit manual cover selection, not automatic mapping of legacy media or an export of the whole archive.

## Gallery photographs and bounded larger packages (v2)

The Settings importer now accepts a maximum **16 MiB JSON package** (including base64) and **8 MiB per photograph**, with a 16 megapixel cap per photograph and 128 megapixels total per package. The earlier metadata-only export tools retain their 2 MiB limit; the cover/gallery augmentation tools use the new limits. No more than 1000 photographs total, and 20 gallery entries per work. Split large selections into independent packages without splitting a sale from its artwork or reimporting a collector; repeated collectors are not automatically merged. Very large collections still require a streaming or directory-based format.

Optional top-level `gallery` array in v2: each entry has the same `artwork_code`, `sha256`, `data_base64` fields as a cover, plus `position` (1–20, unique and contiguous for each work), optional `caption` (300 characters), and optional `kind` (DETAIL, SIGNATURE, DIGITAL_MOCKUP, IN_SITU, FRAMED, WORK_IN_PROGRESS or OTHER; default OTHER). Gallery order, captions, kind and original bytes are preserved; the gallery is independent of the cover. Preview displays position and caption. The native importer validates the complete package and verifies each inserted row and file hash before committing, with rollback and cleanup on errors. Existing v1/v2 packages remain compatible. No crash-atomic guarantee across SQLite and filesystem.

Explicit local selection from an existing v2 package (can already include covers):

```sh
node scripts/add-local-artwork-gallery.mjs export-with-covers.json export-with-gallery.json ART-001 1 DETAIL "Detail" detail.jpg ART-001 2 SIGNATURE "" signature.png
node --test tests/add-local-artwork-gallery.test.mjs
```

The CLI accepts repeated groups: artwork code, position, kind, caption (empty for none), local path. It rejects missing/duplicate positions, unrecognized codes and kinds, symlink final inputs, oversized files and existing output. Native Studio preview remains the full decode/validation gate. No automatic extraction of media paths, RAW, collector photos or certificates yet. Keep source archives and verified backups.

## Automatic association of registered artwork media from the offline copy

After creating a v2 relational package, use the **same immutable, isolated local PostgreSQL copy**, same owner and exact selected artwork IDs, and an explicit absolute directory containing the matching local media snapshot:

```sh
node scripts/export-gestionale-artwork-media.mjs --local-copy ARCHIVE_COPY READONLY_USER 5432 OWNER_ID ARTWORK_IDS export.json /absolute/local/media-root NEW_OUTPUT.json
node --test tests/gestionale-artwork-media.test.mjs
```

The query associates covers and gallery rows via upload ownership, the active current revision and its AVAILABLE LOCAL result blob. The relative storage key must resolve inside the trusted offline directory (no symlinks); bytes must match recorded size and SHA-256. Gallery ordering follows legacy display order then row ID. Known gallery kinds are mapped explicitly. Unknown kinds, missing or cross-owner uploads/revisions/blobs, cloud blobs, old unregistered uploads, missing files, invalid headers and package/selection mismatch fail the operation. Source projection paths are not exported; the input remains unchanged. Output uses the existing v2 cover/gallery format. Native preview still performs full decode, pixel bounds and transactional validation. Only active revision bytes are migrated, not original/edited history, RAW, certificates or collector photos. The 16 MiB JSON and 8 MiB/photo limits remain. Use a static OS-protected copy, never a concurrently modified archive or live pilot. PostgreSQL and installed desktop acceptance remain required; the Node tests use synthetic rows and image headers.

## Bounded batch export for larger photo selections

Use the same command with a **new directory** instead of NEW_OUTPUT.json, followed by `--batch-directory`:

```sh
node scripts/export-gestionale-artwork-media.mjs --local-copy ARCHIVE_COPY READONLY_USER 5432 OWNER_ID ARTWORK_IDS export.json /absolute/local/media-root NEW_DIRECTORY --batch-directory
node --test tests/archive-media-batches.test.mjs
```

This creates numbered, ordinary v2 JSON packages, each independently importable in the existing Studio preview, and writes manifest.json last with SHA-256, byte and record counts. Do not import a directory without its completed manifest. Keep all purchases of a collector, the collector itself, and their photographs together; ambiguous contacts and broken links fail before splitting. The exporter writes at most one batch at a time, refuses an existing output directory and attempts to remove the new directory on a reported failure. Use a trusted private parent directory, immutable offline source and a filesystem supporting hard links. This is not crash-atomic or a single transaction across packages. Back up before importing, review each package and verify the manifest hashes before import; the manifest is not automatically consumed by Studio. Each package still has the 16 MiB limit, 8 MiB/photo and native pixel/row limits. A single linked collector group or artwork that exceeds the package limits cannot be split safely by this tool. Larger indivisible groups may also be rejected by native pixel validation. The metadata source remains bounded by the existing relational exporter and PostgreSQL query limits; this is not full archive or streaming import. Never remove historical links to bypass a limit.


## Read-only preflight of a completed batch directory

```sh
node scripts/verify-archive-media-batches.mjs EXPORT_DIRECTORY
node --test tests/verify-archive-media-batches.test.mjs
```

Before opening any batch in Studio, run the verifier against the original, trusted, unchanged export directory. It checks manifest structure, exact contiguous package names, absence of missing/unlisted entries, regular files, byte counts, SHA-256, package counts and repeated artwork codes or collector contacts across batches. It prints aggregate counts only and never writes files or accesses a database. Hashes check consistency with the manifest, not authenticity. This is a preflight, not a filesystem snapshot, semantic validation, image decoding or an atomic multi-package import. Keep the directory unchanged after verification, make a verified backup and review each package in Studio's native preview. If verification fails, do not import any part; regenerate the complete export from the trusted local copy.

## Native directory preflight (internal contract, not wired to desktop yet)

The dedicated Rust module `archive_import::batches` prepares a read-only plan for an explicit absolute directory using the manifest, current local SQLite archive and the same native package/image parser as ordinary imports. It rejects missing or extra entries, unsafe paths, changed hashes, invalid counts, invalid records/media, duplicate artwork codes and collector contacts across packages, and conflicts with existing records before the first import. It keeps at most one decoded package at a time; no source paths from JSON are resolved.

Integration contract: hold the archive mutex for each call. `preflight(conn, directory)` returns manifest digest, ordered entries and counts. `review(conn, directory, manifest_digest, zero_based_index)` returns the normal native Preview (including digest); present it to the user. Only after explicit confirmation call `commit(conn, media_root, directory, manifest_digest, index, preview_digest)`. Each package has its own existing atomic SQLite/media transaction and count result. Recheck the manifest, exact entries and selected bytes on review/commit. Later package failure never undoes a previous commit. Do not automatically retry after a lost response; show unknown outcome and require archive inspection. After partial success, a fresh full preflight intentionally conflicts with already imported records; no silent skip, merge, persistent progress or automatic resume. Back up before the first commit; keep the export directory immutable and OS-protected. This is neither a filesystem snapshot nor a defense against an active local attacker.

No new IPC commands, dialog, UI, shared contract or schema changes in this increment. The desktop wiring is reserved for the integration owner; do not present this as available in the installed app. Rust tests cover read-only preflight, explicit per-package commit, rollback, duplicate/conflicting records, tampered files and manifest, wrong counts and Unix symlink rejection. Run `cargo test --manifest-path src-tauri/Cargo.toml archive_import::batches` on Linux/Windows; native acceptance remains separate.

The dedicated frontend state module `src/app/archive-batch-progress.ts` is also ready for integration: call `beginBatch` on the verified plan, `acceptBatchReview` only for the current index/digest, then `confirmBatch` on a fresh user confirmation. Disable all actions while committing. `settleBatch` increases counts only on the exact successful native acknowledgement; use `rejected` only on an explicit rejection, and `unknown` for an IPC/transport failure or count mismatch. Stop on either; do not automatically retry. State is deliberately in memory and not proof of persistent import history. No additional wiring in shared files. Local test command: `node --experimental-strip-types --test tests/archive-batch-progress.test.mjs`.


### Importazione cartella nell’app desktop (implementata, collaudo nativo necessario)
Da Impostazioni → Importazione/esportazione, scegli la cartella contenente il manifest dei pacchetti. Il preflight nativo verifica tutto il set senza scrivere nell’archivio. Seleziona «Verifica il prossimo pacchetto», controlla l’anteprima completa di opere, foto e relazioni commerciali, quindi conferma esplicitamente il singolo pacchetto. Ripeti per ciascun pacchetto. I contatori riportano solo i commit confermati. L’interruzione non annulla i pacchetti precedenti; una risposta IPC mancante o incoerente richiede controllo manuale dell’archivio prima di chiudere la sessione. Non ripetere alla cieca una cartella parzialmente importata: l’archivio non deduplica o fonde i record già presenti. La sessione rimane in memoria durante la navigazione, non è un journal persistente e non sopravvive alla chiusura dell’app. L’importazione singola e il ripristino sono inibiti finché la sessione è aperta. Il backup preventivo rimane raccomandato. Nessun invio in rete.
