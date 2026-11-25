import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { certificateTexts, declarationEn, declarationIt, dimensionsText, formatIssueDate, localizedValue, normalizeLanguage, sha256Hex, certificateFingerprint } from '../src/app/certificate-text.ts';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('the declaration names the artist and follows the language of the certificate', () => {
  const it = certificateTexts('it', 'Maria Rossi');
  assert.deepEqual(it.titleLines, ['Certificato di Autenticità']);
  assert.match(it.declarations[0], /^Il sottoscritto Maria Rossi dichiara/);
  const en = certificateTexts('en', 'Maria Rossi');
  assert.deepEqual(en.titleLines, ['Certificate of Authenticity']);
  assert.match(en.declarations[0], /^I, Maria Rossi, certify/);
  const both = certificateTexts('bilingual', 'Maria Rossi');
  assert.equal(both.titleLines.length, 2);
  assert.deepEqual(both.declarations, [declarationIt('Maria Rossi'), declarationEn('Maria Rossi')]);
  assert.equal(both.labels.technique, 'Tecnica / Technique');
  assert.equal(normalizeLanguage('xx'), 'it');
  assert.equal(normalizeLanguage(null), 'it');
});

test('dates, sizes and values are written for the printed sheet', () => {
  assert.match(formatIssueDate('2026-11-12', 'it'), /12 novembre 2026/);
  assert.match(formatIssueDate('2026-11-12', 'en'), /12 November 2026/);
  assert.equal(formatIssueDate('', 'it'), '');
  assert.equal(formatIssueDate('boh', 'it'), 'boh');
  assert.equal(dimensionsText({ height_cm: 40, width_cm: 30, depth_cm: null }), '40 × 30 cm');
  assert.equal(dimensionsText({ height_cm: null, width_cm: null, depth_cm: null }), '');
  const english = text => ({ Olio: 'Oil' })[text] ?? text;
  assert.equal(localizedValue('Olio', 'it', english), 'Olio');
  assert.equal(localizedValue('Olio', 'en', english), 'Oil');
  assert.equal(localizedValue('Olio', 'bilingual', english), 'Olio / Oil');
  assert.equal(localizedValue('Tecnica strana', 'bilingual', english), 'Tecnica strana');
  assert.equal(localizedValue(null, 'en', english), '');
});

test('historical certificate dates never roll over to an invented day', () => {
  assert.match(formatIssueDate('2028-02-29', 'it'), /29 febbraio 2028/);
  assert.match(formatIssueDate('2028-02-29', 'en'), /29 February 2028/);
  assert.match(formatIssueDate('2026-11-12T07:35:00Z', 'it'), /12 novembre 2026/);
  for (const invalid of ['2026-02-29', '2026-04-31', '2026-13-01', '2026-00-15', '0000-12-01']) {
    assert.equal(formatIssueDate(invalid, 'it'), invalid, invalid);
    assert.equal(formatIssueDate(invalid, 'en'), invalid, invalid);
  }
  assert.equal(formatIssueDate('2026-04-31T10:00:00Z', 'it'), '2026-04-31');
});

test('the SHA-256 of the issued record is the standard one', async () => {
  assert.equal(await sha256Hex('abc'), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('printed verification fingerprint contains the complete SHA-256, not half of it', async () => {
  const serial = 'CERT-0012';
  const original = '{"id":"a","title":"Paesaggio"}';
  const full = await certificateFingerprint(serial, original);
  assert.match(full, /^[a-f0-9]{64}$/);
  assert.equal(full, await sha256Hex(`${serial}\n${original}`));
  assert.notEqual(full, await certificateFingerprint(serial, '{"id":"a","title":"Alterato"}'));
  assert.notEqual(full, await certificateFingerprint('CERT-0013', original));
  const sheet = get('src/app/certificate-sheet.component.ts');
  assert.match(sheet, /this\.hash = hex;/, 'display the complete 64-character digest');
  assert.doesNotMatch(sheet, /hex\.slice\(0,\s*32\)/, 'a half digest cannot be labelled SHA-256');
});

test('the certificate keeps the layout of the Gestionale and the artist comes from the settings', () => {
  const sheet = get('src/app/certificate-sheet.component.ts');
  for (const rule of ['210mm', '297mm', '#fffdf8', '#9a762d', "'Palatino Linotype'", 'cs-verify', 'cs-sign']) assert.ok(sheet.includes(rule), rule);
  assert.match(get('src-tauri/src/commands.rs'), /SELECT artist_name,artist_alias FROM studio_settings/);
  assert.match(get('src-tauri/src/db.rs'), /ALTER TABLE studio_settings ADD COLUMN artist_name/);
  const html = get('src/app/app.component.html');
  assert.match(html, /<rp-certificate-sheet/);
  const dialogStart = html.indexOf('aria-labelledby="certificate-title"');
  assert.doesNotMatch(html.slice(dialogStart, dialogStart + 2500), /certificateDraft\.artist_name/, 'no artist field in the emission dialog');
  assert.match(html, /saveArtistProfile/);
});

test('exports live in Settings only, not on the archive pages', () => {
  const html = get('src/app/app.component.html');
  const settingsStart = html.indexOf("section === 'settings'");
  const settingsEnd = html.indexOf('</section>', settingsStart);
  const outside = html.slice(0, settingsStart) + html.slice(settingsEnd);
  assert.doesNotMatch(outside, /exportLedger\(/, 'no export button outside Settings');
  const settings = html.slice(settingsStart, settingsEnd);
  for (const kind of ['artworks', 'gallery', 'raw_files', 'collectors', 'collector_photos', 'sales', 'receipts', 'balances', 'installments', 'allocations', 'certificates', 'expenses']) {
    assert.ok(settings.includes(`exportLedger('${kind}')`), kind);
  }
});
