import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { SUPPORTS, TECHNIQUES, OTHER_CHOICE, menuStateFor, valueFor } from '../src/app/artwork-options.ts';

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

test('the new artwork form shows the price by itself and offers menus instead of free text', () => {
  const html = get('src/app/app.component.html');
  const dialog = html.slice(html.indexOf('aria-labelledby="artwork-title"'));
  assert.doesNotMatch(dialog, /Calcola stima/, 'no manual estimate button');
  assert.match(dialog, /draftPrice\.cents/);
  assert.match(dialog, /setTechniqueChoice/);
  assert.match(dialog, /setSupportChoice/);
  assert.match(dialog, /class="chip-toggle"/);
  assert.match(dialog, /class="price-modes"/);
  assert.doesNotMatch(get('src/app/app.component.ts'), /scheduleEstimate|previewPrice/, 'the price is computed in the interface, instantly');
});

test('imported technique and support values normalize compatibility spaces without changing custom text', () => {
  assert.deepEqual(
    menuStateFor(TECHNIQUES, '\u00a0TECNICA\u202F  MISTA\t'),
    { choice: 'Tecnica mista', other: '' },
    'non-breaking spaces and repeated whitespace still select the known technique'
  );
  assert.deepEqual(
    menuStateFor(SUPPORTS, 'Ｃａｒｔａ　Ｆａｂｒｉａｎｏ'),
    { choice: 'Carta Fabriano', other: '' },
    'compatibility-width characters from copied catalogues map to the known support'
  );
  assert.deepEqual(
    menuStateFor(SUPPORTS, ' Carta\u00a0\t da  acquerello '),
    { choice: 'Carta da acquerello', other: '' }
  );
  assert.deepEqual(
    menuStateFor(TECHNIQUES, '  TECNICA\u00a0  SPECIALE  '),
    { choice: OTHER_CHOICE, other: 'TECNICA\u00a0  SPECIALE' },
    'unknown historic techniques are preserved as custom values, not discarded'
  );
  assert.equal(valueFor(OTHER_CHOICE, ' TECNICA\u00a0  SPECIALE '), 'TECNICA\u00a0  SPECIALE');
});
