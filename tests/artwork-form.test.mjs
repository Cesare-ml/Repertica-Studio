import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { SUPPORTS, TECHNIQUES, OTHER_CHOICE, canEstimate, menuStateFor, valueFor } from '../src/app/artwork-options.ts';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('a stored technique or support maps to the menu, anything else becomes "Altro" with its text', () => {
  assert.deepEqual(menuStateFor(TECHNIQUES, 'Olio'), { choice: 'Olio', other: '' });
  assert.deepEqual(menuStateFor(TECHNIQUES, ' acquerello '), { choice: 'Acquerello', other: '' }, 'case and spaces are ignored');
  assert.deepEqual(menuStateFor(TECHNIQUES, 'Olio su tela'), { choice: OTHER_CHOICE, other: 'Olio su tela' });
  assert.deepEqual(menuStateFor(SUPPORTS, null), { choice: '', other: '' });
  assert.deepEqual(menuStateFor(SUPPORTS, '   '), { choice: '', other: '' });
});

test('the value to store follows the menu', () => {
  assert.equal(valueFor('Olio', ''), 'Olio');
  assert.equal(valueFor('', 'ignorato'), null);
  assert.equal(valueFor(OTHER_CHOICE, '  Tecnica rara '), 'Tecnica rara');
  assert.equal(valueFor(OTHER_CHOICE, '   '), null, 'an empty "Altro" stores nothing');
});

test('the menus contain the options of the Gestionale Opere and are translated', () => {
  for (const option of ['Olio', 'Acrilico', 'Tecnica mista', 'Acquerello', 'Grafite', 'Pastello']) assert.ok(TECHNIQUES.includes(option), option);
  for (const option of ['Tela', 'Carta', 'Tavola', 'Legno', 'Cartone telato']) assert.ok(SUPPORTS.includes(option), option);
  const dictionary = get('src/app/i18n.ts');
  for (const option of [...TECHNIQUES, ...SUPPORTS]) assert.ok(dictionary.includes(`"${option}"`), `translation of ${option}`);
});

test('an estimate needs two positive finite sizes', () => {
  assert.ok(canEstimate(30, 40));
  for (const [w, h] of [[0, 40], [30, null], [undefined, 40], [-1, 5], [Number.NaN, 4], [Infinity, 4]]) assert.ok(!canEstimate(w, h), `${w} ${h}`);
});

test('the new artwork form estimates the price by itself and offers menus instead of free text', () => {
  const html = get('src/app/app.component.html');
  const dialog = html.slice(html.indexOf('aria-labelledby="artwork-title"'));
  assert.doesNotMatch(dialog, /Calcola stima/, 'no manual estimate button');
  assert.match(dialog, /scheduleEstimate\(\)/);
  assert.match(dialog, /setTechniqueChoice/);
  assert.match(dialog, /setSupportChoice/);
  assert.match(dialog, /class="chip-toggle"/);
  const ts = get('src/app/app.component.ts');
  assert.match(ts, /scheduleEstimate\(\): void/);
  const body = ts.slice(ts.indexOf('async previewPrice'), ts.indexOf('private syncMenus'));
  assert.doesNotMatch(body, /errorMessage/, 'a failed estimate never shows an error');
});
