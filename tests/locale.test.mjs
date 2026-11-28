import test from 'node:test';
import assert from 'node:assert/strict';
import { chooseLanguage, translateLabel, translate } from '../src/app/i18n.ts';

const dictionaries = {
  it: {}, en: { Salva: 'Save', Backup: 'Backup', Empty: '' },
  es: { Salva: 'Guardar' }, fr: { Salva: 'Enregistrer' }, de: { Salva: 'Speichern' },
};

test('native and browser locales select the same supported language', () => {
  for (const [input, expected] of [
    ['it-IT', 'it'], ['en_US.UTF-8', 'en'], ['ES-es', 'es'],
    ['fr.UTF-8', 'fr'], ['de@euro', 'de'], [' de.UTF-8@euro ', 'de'],
    ['fr_CA.UTF-8', 'fr'], ['it_IT@euro', 'it'],
  ]) assert.equal(chooseLanguage(input), expected, input);
});

test('unknown or absent locale falls back to English', () => {
  for (const input of [null, undefined, '', ' ', 'C.UTF-8', 'POSIX', 'pt-BR', 'constructor']) {
    assert.equal(chooseLanguage(input), 'en', String(input));
  }
});

test('local dictionaries retain Italian source and English fallback', () => {
  assert.equal(translateLabel('Salva', 'it', dictionaries), 'Salva');
  assert.equal(translateLabel('Salva', 'es', dictionaries), 'Guardar');
  assert.equal(translateLabel('Salva', 'fr', dictionaries), 'Enregistrer');
  assert.equal(translateLabel('Salva', 'de', dictionaries), 'Speichern');
  assert.equal(translateLabel('Backup', 'fr', dictionaries), 'Backup');
  assert.equal(translateLabel('Missing', 'es', dictionaries), 'Missing');
  assert.equal(translateLabel('Empty', 'de', dictionaries), '');
  assert.equal(translateLabel('Salva', 'unsupported', dictionaries), 'Save');
});

test('unknown labels never resolve to inherited object methods', () => {
  for (const key of ['constructor', 'toString', '__proto__', 'hasOwnProperty']) {
    for (const language of ['it', 'en', 'es', 'fr', 'de']) {
      assert.equal(translateLabel(key, language, dictionaries), key);
    }
  }
  const explicit = { ...dictionaries, en: { ...dictionaries.en, toString: 'Explicit label' } };
  assert.equal(translateLabel('toString', 'de', explicit), 'Explicit label');
});

test('public translation wrapper uses safe fallback for inherited keys', () => {
  for (const language of ['it', 'en', 'es', 'fr', 'de']) {
    assert.equal(translate('constructor', language), 'constructor');
    assert.equal(translate('__proto__', language), '__proto__');
  }
  assert.equal(translate('Opere', chooseLanguage('fr.UTF-8')), "Œuvres");
});

test('stale archive edits retain translations after native locale normalization', () => {
  const key = 'La scheda è cambiata dopo l’apertura. Chiudi e riapri la modifica per vedere i dati aggiornati.';
  for (const [locale, expected] of [
    ['it_IT.UTF-8', key],
    ['en_US.UTF-8', 'This record changed after you opened it. Close and reopen the editor to see the latest data.'],
    ['es_ES@euro', 'La ficha ha cambiado desde que la abriste. Cierra y vuelve a abrir la edición para ver los datos actualizados.'],
    ['fr.UTF-8', 'La fiche a changé depuis son ouverture. Fermez et rouvrez l’éditeur pour voir les données à jour.'],
    ['de@euro', 'Der Datensatz wurde seit dem Öffnen geändert. Schließe und öffne die Bearbeitung erneut, um die aktuellen Daten zu sehen.'],
    ['C.UTF-8', 'This record changed after you opened it. Close and reopen the editor to see the latest data.'],
  ]) assert.equal(translate(key, chooseLanguage(locale)), expected, locale);
});
