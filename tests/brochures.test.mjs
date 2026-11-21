import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('brochure documents persist locally with additive schema and native commands', () => {
  const db = read('src-tauri/src/db.rs');
  const rust = read('src-tauri/src/brochures.rs');
  const lib = read('src-tauri/src/lib.rs');
  const api = read('src/app/api.ts');
  assert.match(db, /SCHEMA_VERSION: i64 = \d+/);
  assert.match(db, /if version < 15/);
  assert.match(db, /CREATE TABLE IF NOT EXISTS brochures/);
  assert.match(db, /payload_json TEXT NOT NULL/);
  for (const command of ['list_brochures', 'save_brochure', 'delete_brochure', 'export_brochure_pdf']) {
    assert.ok(lib.includes('brochures::' + command), command);
    assert.ok(api.includes("'" + command + "'"), command);
    assert.ok(rust.includes('pub fn ' + command), command);
  }
  assert.doesNotMatch(rust, /reqwest|TcpStream|std::net|http:\/\//);
  assert.ok(rust.includes('fn load_photo('));
  assert.ok(rust.includes('fn render_pdf('));
  assert.ok(rust.includes('fn validate_output('));
});
test('offline editor lets users reorder, edit, persist and export works', () => {
  const ui = read('src/app/brochures.component.ts');
  const html = read('src/app/brochures.component.html');
  const app = read('src/app/app.component.html');
  const docs = read('docs/BROCHURES-OFFLINE.md');
  assert.match(app, /<rp-brochures/);
  for (const action of ['addArtwork','move','remove','changeImage','saveDraft','duplicate','exportPdf']) {
    assert.match(ui, new RegExp(action + '\\('));
  }
  for (const layout of ['classic','full_bleed','details']) assert.ok(html.includes('value="' + layout + '"'), layout);
  assert.ok(html.includes('[(ngModel)]="b.language"'));
  assert.ok(html.includes('[(ngModel)]="item.title_override"'));
  assert.ok(html.includes('[(ngModel)]="item.description_override"'));
  assert.match(docs, /snapshot immutabili/);
});
test('brochure editor exposes every new label in the translations', () => {
  const ts = read('src/app/brochures.component.ts');
  const html = read('src/app/brochures.component.html');
  const i18n = read('src/app/i18n.ts');
  const keys = [...new Set([...ts.matchAll(/tr\('([^']+)'\)/g), ...html.matchAll(/tr\('([^']+)'\)/g)].map(x => x[1]))];
  const markers = ['const EN:', 'const ES:', 'const FR:', 'const DE:'];
  for (let i=0;i<markers.length;i++) {
    const start=i18n.indexOf(markers[i]);
    const end=i18n.indexOf('\n};',start);
    const entries=i18n.slice(start,end);
    for (const key of keys) {
      assert.ok(entries.includes(JSON.stringify(key)+':') || entries.includes("'" + key + "':"), markers[i] + ': missing ' + key);
    }
  }
});
