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
  assert.match(native, /let url = data_core\(db, receipt_id\)\?/);
  assert.match(native, /\.create_new\(true\)/);
  assert.match(native, /Non esportare documenti all'interno dell'archivio attivo/);
});
