import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { csvRegisterOptions } from '../src/app/csv-registers.ts';
import { translate } from '../src/app/i18n.ts';

const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const kinds = ['artworks', 'gallery', 'collectors', 'certificates', 'sales', 'balances', 'receipts', 'expenses'];

test('all eight backend CSV register kinds have unique local export labels and filenames', () => {
  assert.deepEqual(Object.keys(csvRegisterOptions).sort(), [...kinds].sort());
  const backend = read('src-tauri/src/export.rs');
  const filenames = new Set();
  for (const kind of kinds) {
    const option = csvRegisterOptions[kind];
    assert.match(backend, new RegExp('"' + kind + '"\\s*=>'));
    assert.ok(option.title && option.filename && option.success);
    assert.match(option.filename, /^[a-z]+$/);
    assert.ok(!filenames.has(option.filename));
    filenames.add(option.filename);
    assert.notEqual(translate(option.title, 'en'), option.title);
    assert.notEqual(translate(option.success, 'en'), option.success);
  }
});

test('every archive register export has an accessible button and uses native save IPC', () => {
  const html = read('src/app/app.component.html');
  const component = read('src/app/app.component.ts');
  const api = read('src/app/api.ts');
  for (const kind of kinds) {
    assert.ok(html.includes("exportLedger('" + kind + "')"));
    assert.ok(api.includes("'export_ledger_csv'"));
  }
  assert.match(component, /save\(\{/);
  assert.match(component, /csvRegisterOptions\[kind\]/);
  assert.match(component, /api\.exportLedgerCsv\(kind, destination\)/);
});

test('sale receipt CSV is a separate on-demand register, never mislabeled as sale revenue', () => {
  const backend = read('src-tauri/src/export.rs');
  const html = read('src/app/app.component.html');
  assert.match(backend, /"receipt_id;sale_id;received_at;artwork_id;/);
  assert.match(backend, /FROM sale_receipts r JOIN sales s ON s.id=r.sale_id/);
  assert.match(html, /exportLedger\('receipts'\)/);
  for (const locale of ['en', 'es', 'fr', 'de']) {
    assert.notEqual(translate('Esporta incassi CSV', locale), 'Esporta incassi CSV');
    assert.notEqual(translate('Registro incassi esportato in CSV locale.', locale), 'Registro incassi esportato in CSV locale.');
  }
});

test('receipt export includes optional one-to-one evidence metadata without binary files', () => {
  const backend = read('src-tauri/src/export.rs');
  const documentation = read('docs/DATA-PORTABILITY.md');
  assert.match(backend, /LEFT JOIN sale_receipt_attachments evidence ON evidence\.receipt_id=r\.id/);
  assert.match(backend, /attachment_path;attachment_original_name;attachment_mime;attachment_size_bytes;attachment_sha256;attachment_created_at/);
  assert.match(backend, /evidence\.media_path,evidence\.original_name,evidence\.mime_type/);
  assert.match(documentation, /non il CSV/);
});

test('sale balances export includes exact cents and does not infer receipt records', () => {
  const backend = read('src-tauri/src/export.rs');
  const html = read('src/app/app.component.html');
  assert.match(backend, /"sale_id;sold_at;artwork_id;artwork_code;artwork_title;collector_id;collector_name;currency;gross_cents;received_cents;outstanding_cents;created_at"/);
  assert.match(backend, /FROM sale_receipts GROUP BY sale_id/);
  assert.match(html, /exportLedger\('balances'\)/);
  for (const locale of ['en', 'es', 'fr', 'de']) {
    assert.notEqual(translate('Esporta saldi vendite CSV', locale), 'Esporta saldi vendite CSV');
    assert.notEqual(translate('Saldi vendite esportati in CSV locale.', locale), 'Saldi vendite esportati in CSV locale.');
  }
});

test('gallery metadata export preserves image UUIDs, artwork relation, captions and order', () => {
  const backend = read('src-tauri/src/export.rs');
  const html = read('src/app/app.component.html');
  assert.match(backend, /"image_id;artwork_id;artwork_code;artwork_title;image_path;caption;sort_order;created_at"/);
  assert.match(backend, /FROM artwork_images i JOIN artworks a ON a.id=i.artwork_id/);
  assert.match(backend, /ORDER BY a.code COLLATE NOCASE,a.id,i.sort_order,i.created_at,i.id/);
  assert.match(html, /exportLedger\('gallery'\)/);
  for (const locale of ['en', 'es', 'fr', 'de']) {
    assert.notEqual(translate('Esporta metadati galleria CSV', locale), 'Esporta metadati galleria CSV');
    assert.notEqual(translate('Metadati galleria esportati in CSV locale.', locale),
      'Metadati galleria esportati in CSV locale.');
  }
});
