import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('no cloud or Sites dependency in runtime', () => {
  const cargo = get('src-tauri/Cargo.toml');
  const api = get('src/app/api.ts');
  assert.doesNotMatch(api, /fetch\(|axios|http:\/\/|https:\/\//);
  assert.doesNotMatch(cargo, /reqwest|sqlx|postgres|aws-sdk/);
});
test('four MVP modules available via typed native IPC', () => {
  const native = get('src-tauri/src/lib.rs');
  for (const name of ['list_artworks','list_collectors','list_sales','list_certificates','issue_certificate']) {
    assert.match(native, new RegExp('(commands|collectors)::' + name));
  }
});
test('CPAL license attribution correctly names Repertica', () => {
  const license = get('LICENSE');
  assert.match(license, /Common Public Attribution License Version 1\.0/);
  assert.match(license, /Powered by Repertica/);
  assert.doesNotMatch(license, /Powered by VirgoStars/);
});

test('native CSV export is wired and the Save dialog is explicitly permitted', () => {
  const capability = JSON.parse(get('src-tauri/capabilities/default.json'));
  assert.ok(capability.permissions.includes('dialog:allow-save'));
  assert.match(get('src-tauri/src/lib.rs'), /export::export_ledger_csv/);
  assert.match(get('src/app/api.ts'), /'export_ledger_csv'/);
});

test('gallery IPC, schema v3 and backup consistency are wired', () => {
  const native = get('src-tauri/src/lib.rs');
  const db = get('src-tauri/src/db.rs');
  const backup = get('src-tauri/src/backup.rs');
  const api = get('src/app/api.ts');
  for (const command of ['list_artwork_gallery', 'add_artwork_gallery_image_from_path',
    'artwork_gallery_image_data', 'delete_artwork_gallery_image']) {
    assert.match(native, new RegExp('gallery::' + command));
    assert.ok(api.includes("'" + command + "'"));
  }
  assert.match(db, /CREATE TABLE IF NOT EXISTS artwork_images/);
  assert.match(db, /PRAGMA user_version=3/);
  assert.match(backup, /SELECT image_path FROM artwork_images/);
  assert.match(backup, /1\.\.=crate::db::SCHEMA_VERSION/);
});

test('gallery sorting commands and schema v5 are available offline', () => {
  assert.match(get('src-tauri/src/db.rs'), /PRAGMA user_version=5/);
  assert.match(get('src-tauri/src/lib.rs'), /gallery::move_artwork_gallery_image/);
  assert.match(get('src/app/api.ts'), /'move_artwork_gallery_image'/);
  assert.match(get('src/app/app.component.html'), /moveGalleryPhoto/);
});
test('annual receipt summaries have a native typed IPC and a separate finance panel', () => {
  assert.match(get('src-tauri/src/lib.rs'), /receipts::receipt_year_overview/);
  assert.match(get('src/app/api.ts'), /'receipt_year_overview'/);
  assert.match(get('src/app/app.component.html'), /receiptYear\.months/);
  assert.match(get('src/app/app.component.ts'), /api\.receiptYearOverview\(this\.financeYear\)/);
});
test('sale receipt v4 records are exposed through local typed IPC', () => {
  const schema=get('src-tauri/src/db.rs'),native=get('src-tauri/src/lib.rs'),api=get('src/app/api.ts');
  assert.match(schema, /CREATE TABLE sale_receipts/);
  assert.match(schema, /PRAGMA user_version=4/);
  for(const command of ['list_sale_receipts','record_sale_receipt','delete_sale_receipt','list_sale_balances']){
    assert.ok(native.includes('receipts::'+command));
    assert.ok(api.includes("'"+command+"'"));
  }
});

test('receipt PDF/image attachments use offline SQLite v6 and verified backup media', () => {
  const db = get('src-tauri/src/db.rs'), backup = get('src-tauri/src/backup.rs');
  const native = get('src-tauri/src/lib.rs'), module = get('src-tauri/src/receipt_attachments.rs');
  assert.match(db, /CREATE TABLE IF NOT EXISTS sale_receipt_attachments/);
  assert.match(db, /PRAGMA user_version=6/);
  assert.match(db, /PRAGMA user_version=7/);
  for (const command of ['attach_sale_receipt_file','get_sale_receipt_attachment',
    'sale_receipt_attachment_data','remove_sale_receipt_attachment']) {
    assert.ok(native.includes('receipt_attachments::' + command));
  }
  assert.match(backup, /SELECT media_path FROM sale_receipt_attachments/);
  assert.match(module, /Sha256::digest/);
  assert.match(module, /MAX_BYTES: usize = 12 \* 1024 \* 1024/);
});

test('CI runs the Rust suite on Windows, where backup renames behave differently', () => {
  const ci = get('.github/workflows/ci.yml');
  assert.match(ci, /windows-latest/);
  assert.match(ci, /cargo test --manifest-path src-tauri\/Cargo\.toml/);
});
test('backup closes the manifest handle before publishing the folder (Windows rename rule)', () => {
  const backup = get('src-tauri/src/backup.rs');
  const closing = backup.indexOf('Windows refuses to');
  const rename = backup.indexOf('fs::rename(&partial, &target)');
  assert.ok(closing > 0 && rename > closing);
});

test('photos are picked with the native dialog and read by path, not pushed as base64 through the UI', () => {
  const ui = get('src/app/app.component.ts');
  const html = get('src/app/app.component.html');
  const api = get('src/app/api.ts');
  assert.doesNotMatch(html, /type="file"/);
  assert.doesNotMatch(ui, /readAsDataURL/);
  assert.match(api, /'save_artwork_image_from_path'/);
  assert.match(get('src-tauri/src/lib.rs'), /commands::save_artwork_image_from_path/);
});

test('local MCP server is read-only, STDIO-only and started with --mcp', () => {
  const source = get('src-tauri/src/mcp.rs');
  const mcp = source.slice(0, source.indexOf('#[cfg(test)]')); // production code only
  const main = get('src-tauri/src/main.rs');
  assert.match(main, /"--mcp"/);
  assert.match(mcp, /SQLITE_OPEN_READ_ONLY/);
  assert.match(mcp, /query_only/);
  assert.doesNotMatch(mcp, /TcpListener|UdpSocket|reqwest|hyper|tokio/);
  assert.doesNotMatch(mcp, /INSERT |UPDATE |DELETE |DROP |ALTER /);
  assert.match(get('src-tauri/src/lib.rs'), /mcp::mcp_setup_info/);
});

test('first-run guide can be dismissed, reopened and covers backups', () => {
  const html = get('src/app/app.component.html');
  const ui = get('src/app/app.component.ts');
  assert.match(html, /showGuide/);
  assert.match(html, /dismissGuide\(\)/);
  assert.match(html, /reopenGuide\(\)/);
  assert.match(ui, /localStorage/);
  assert.match(html, /Assistenti IA locali \(MCP\)/);
});

test('new artwork is created in one step with automatic code, photos and optional sale', () => {
  const lib = get('src-tauri/src/lib.rs');
  const html = get('src/app/app.component.html');
  assert.match(lib, /commands::create_artwork_full/);
  assert.match(lib, /archive_code::next_archive_code/);
  assert.match(html, /Assegnato automaticamente al salvataggio/);
  assert.match(html, /Registra una vendita/);
  assert.doesNotMatch(html, /artworkDraft\.location/, 'location is no longer part of the artwork form');
  assert.doesNotMatch(html, /Ubicazione/);
});

test('collectors keep the archive fields, photographs and are covered by backups', () => {
  const lib = get('src-tauri/src/lib.rs');
  for (const command of ['create_collector_full', 'add_collector_image_from_path', 'collector_image_data', 'delete_collector_image']) {
    assert.match(lib, new RegExp('collectors::' + command));
  }
  assert.match(get('src-tauri/src/backup.rs'), /SELECT image_path FROM collector_images/);
  assert.match(get('src-tauri/src/media_audit.rs'), /FROM collector_images/);
  assert.match(get('src/app/app.component.html'), /collectorDraft\.birth_date/);
});

test('sales allocation is configurable, can be switched off and is frozen per sale', () => {
  const db = get('src-tauri/src/db.rs');
  assert.match(db, /CREATE TABLE IF NOT EXISTS allocation_parts/);
  assert.match(db, /CREATE TABLE IF NOT EXISTS sale_allocations/);
  assert.match(get('src-tauri/src/lib.rs'), /allocations::save_allocation_plan/);
  const html = get('src/app/app.component.html');
  assert.match(html, /allocationDraft\.enabled/);
  assert.match(html, /addAllocationPart\(\)/);
  assert.doesNotMatch(html, /personalPercent|activityPercent|savingsPercent/);
});

test('scheduled backup is opt-in, local and reports through a native event', () => {
  const rust = get('src-tauri/src/auto_backup.rs');
  assert.match(get('src-tauri/src/db.rs'), /enabled INTEGER NOT NULL DEFAULT 0/);
  assert.match(get('src-tauri/src/lib.rs'), /auto_backup::start/);
  assert.match(rust, /auto-backup-finished/);
  assert.doesNotMatch(rust, /TcpStream|reqwest|http/);
  assert.match(get('src/app/app.component.html'), /Attiva il backup automatico/);
});

test('media center, image editor, RAW attachment and installments are wired end to end', () => {
  const lib = get('src-tauri/src/lib.rs');
  for (const command of ['media::media_list', 'media::media_apply_edit', 'media::media_restore_revision', 'media::media_purge',
    'raw_files::attach_artwork_raw', 'raw_files::export_artwork_raw', 'installments::save_sale_installments', 'installments::generate_installment_plan']) {
    assert.ok(lib.includes(command), command);
  }
  const html = get('src/app/app.component.html');
  assert.match(html, /<rp-media-center/);
  assert.match(html, /<rp-media-editor/);
  assert.match(html, /<rp-payment-plan/);
  const api = get('src/app/api.ts');
  for (const name of ['mediaApplyEdit', 'mediaRestoreRevision', 'attachArtworkRaw', 'saveSaleInstallments']) assert.ok(api.includes(name), name);
});

test('the new artwork dialog shows photographs and the RAW file before the descriptive fields', () => {
  const html = get('src/app/app.component.html');
  const dialog = html.slice(html.indexOf('aria-labelledby="artwork-title"'));
  const photos = dialog.indexOf('Fotografie e file RAW');
  const firstField = dialog.indexOf("tr('Titolo *')");
  assert.ok(photos > 0 && firstField > photos, 'photographs come first');
  assert.ok(dialog.indexOf("tr('Codice archivio')") > photos);
});

test('edits never overwrite the original: the engine only writes new revision files', () => {
  const media = get('src-tauri/src/media.rs');
  const production = media.slice(0, media.indexOf('#[cfg(test)]'));
  assert.match(production, /'ORIGINAL'/);
  assert.match(production, /kind,image_path,width,height,size_bytes,sha256,operations_json,created_at\)\s*VALUES\(\?1,\?2,\?3,'EDIT'/);
  assert.doesNotMatch(production, /fs::write\(.*image_path/);
  assert.doesNotMatch(get('src-tauri/src/imaging.rs'), /std::net|TcpStream|reqwest/);
});

