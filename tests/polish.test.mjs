import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const bytes = path => readFileSync(new URL('../' + path, import.meta.url));

test('the app icon is drawn from a large source so it stays sharp at every size', () => {
  const png = bytes('branding/icon.png');
  assert.equal(png.subarray(1, 4).toString(), 'PNG');
  const width = png.readUInt32BE(16);
  const height = png.readUInt32BE(20);
  assert.ok(width >= 1024 && height >= 1024 && width === height, `icon source is ${width}x${height}`);
});

test('printing a certificate adds no browser header, footer or white border', () => {
  const css = get('src/app/app.component.css');
  const print = css.slice(css.indexOf('@media print'));
  assert.match(print, /@page\{size:A4;margin:0\}/, 'zero page margin hides the date/title/address lines of the print dialog');
  assert.doesNotMatch(print, /margin:13mm/);
  const sheet = get('src/app/certificate-sheet.component.ts');
  assert.match(sheet, /\.print-only \.cs-page\{margin:0;/);
  assert.doesNotMatch(sheet, /margin:-13mm/, 'no negative-margin workaround any more');
});

test('the mail section is simply called "Posta"', () => {
  assert.match(get('src/app/app.component.ts'), /id: 'mail', label: 'Posta' \}/);
  for (const file of ['src/app/app.component.ts', 'src/app/i18n.ts', 'src/app/mail-i18n.ts', 'docs/BUSINESS-MAIL-DESKTOP.md']) {
    assert.doesNotMatch(get(file), /Posta professionale/, file);
  }
});

test('an exported brochure can be shown in the file manager', () => {
  const rust = get('src-tauri/src/reveal.rs');
  assert.match(rust, /pub fn reveal_in_folder/);
  assert.doesNotMatch(rust, /Command::new\("(cmd|sh|bash|powershell)/, 'no shell is involved');
  assert.match(get('src-tauri/src/lib.rs'), /reveal::reveal_in_folder/);
  assert.match(get('src/app/api.ts'), /'reveal_in_folder'/);
  const component = get('src/app/brochures.component.ts');
  assert.match(component, /exportedPath = path/);
  assert.match(component, /api\.revealInFolder\(this\.exportedPath\)/);
  assert.match(get('src/app/brochures.component.html'), /tr\('Apri cartella'\)/);
});

test('the brochure cover and last page use the elegant layout, and the preview mirrors the PDF', () => {
  const rust = get('src-tauri/src/brochures.rs');
  assert.match(rust, /Times-Roman/);
  assert.match(rust, /Times-Italic/);
  assert.match(rust, /fn elegant_frame/);
  assert.doesNotMatch(rust, /hero/, 'no artwork picture on the cover');
  assert.doesNotMatch(rust.slice(0, rust.indexOf('#[cfg(test)]')), /REPERTICA|Repertica/, 'brochures never name the application');
  const html = get('src/app/brochures.component.html');
  for (const klass of ['pv-frame', 'pv-cover-artist', 'pv-contact']) assert.ok(html.includes(klass), klass);
  assert.doesNotMatch(html, /pv-mount|REPERTICA|Repertica/, 'no picture on the cover and no product name in the preview');
  assert.match(html, /\[ngClass\]="'t-' \+ item\.template"/, 'every layout of the PDF has its own preview');
  const css = get('src/app/brochures.component.css');
  for (const layout of ['t-classic', 't-details', 't-full_bleed']) assert.ok(css.includes(layout), layout);
});
