import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { csvRegisterOptions } from '../src/app/csv-registers.ts';
import { translate } from '../src/app/i18n.ts';

const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const kinds = ['artworks', 'collectors', 'certificates', 'sales', 'expenses'];

test('all five backend CSV register kinds have unique local export labels and filenames', () => {
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
