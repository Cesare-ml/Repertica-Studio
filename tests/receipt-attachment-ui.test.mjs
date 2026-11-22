import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('payment document buttons use offline Tauri IPC and the native file dialogs', () => {
  const api = get('src/app/api.ts');
  const ts = get('src/app/app.component.ts');
  const html = get('src/app/app.component.html');
  const lib = get('src-tauri/src/lib.rs');
  const permissions = JSON.parse(get('src-tauri/capabilities/default.json')).permissions;
  for (const command of ['get_sale_receipt_attachment', 'attach_sale_receipt_file',
    'sale_receipt_attachment_data', 'remove_sale_receipt_attachment',
    'export_sale_receipt_attachment']) {
    assert.ok(api.includes("'" + command + "'"), command);
    assert.ok(lib.includes('receipt_attachments::' + command), command);
  }
  assert.ok(permissions.includes('dialog:allow-open'));
  assert.ok(permissions.includes('dialog:allow-save'));
  for (const method of ['attachReceiptFile', 'removeReceiptFile', 'viewReceiptImage',
    'exportReceiptFile']) {
    assert.ok(ts.includes(method + '('));
    assert.ok(html.includes(method + '('));
  }
  assert.match(ts, /api\.getReceiptAttachment\(receipt\.id\)/);
  assert.match(ts, /this\.receiptAttachmentsLoading = true/);
  assert.match(html, /\*ngIf="receiptAttachments\[receipt\.id\] as file/);
});

test('a PDF must be exported instead of weakening the desktop CSP', () => {
  const csp = JSON.parse(get('src-tauri/tauri.conf.json')).app.security.csp;
  assert.match(csp, /object-src 'none'/);
  assert.match(csp, /frame-src 'none'/);
  const html = get('src/app/app.component.html');
  assert.match(html, /file\.mime_type !== 'application\/pdf'/);
  assert.match(html, /Esporta documento/);
  assert.doesNotMatch(html, /<iframe|<object/);
  const native = get('src-tauri/src/receipt_attachments.rs');
  // The native export must write bytes verified against SQLite's stored digest.
  // Preview images may still use base64; exporting must not round-trip through it.
  const exportSource = native.split('fn export_core(')[1]?.split('#[tauri::command]')[0];
  assert.ok(exportSource, 'native export function exists');
  assert.match(exportSource, /let \(attachment, bytes\) = verified_receipt_core\(db, receipt_id\)\?/);
  assert.match(exportSource, /write_all\(&bytes\)/);
  assert.doesNotMatch(exportSource, /STANDARD\.decode|split_once\(";base64,"\)/);
  assert.match(native, /\.create_new\(true\)/);
  assert.match(native, /Non esportare documenti all'interno dell'archivio attivo/);
});
