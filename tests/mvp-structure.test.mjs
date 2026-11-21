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
    assert.ok(native.includes('commands::' + name));
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
  for (const command of ['list_artwork_gallery', 'add_artwork_gallery_image',
    'artwork_gallery_image_data', 'delete_artwork_gallery_image']) {
    assert.match(native, new RegExp('gallery::' + command));
    assert.ok(api.includes("'" + command + "'"));
  }
  assert.match(db, /CREATE TABLE IF NOT EXISTS artwork_images/);
  assert.match(db, /PRAGMA user_version=3/);
  assert.match(backup, /SELECT image_path FROM artwork_images/);
  assert.match(backup, /1\.\.=4/);
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
