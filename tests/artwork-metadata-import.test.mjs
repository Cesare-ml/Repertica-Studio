import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { translate } from '../src/app/i18n.ts';
import { MAX_IMPORT_BYTES, readImportFile, importErrorParts } from '../src/app/artwork-metadata-import.ts';

test('explicit file selection accepts UTF-8 and refuses malformed bytes', async () => {
  assert.equal(await readImportFile(new Blob(['{"title":"Étude"}'])), '{"title":"Étude"}');
  await assert.rejects(readImportFile(new Blob([Uint8Array.of(0xc3,0x28)])), /format/);
});
test('size checked both before and after read', async () => {
  await assert.rejects(readImportFile({size:MAX_IMPORT_BYTES+1,arrayBuffer:async()=>{throw Error('must not read');}}), /size/);
  await assert.rejects(readImportFile({size:1,arrayBuffer:async()=>new ArrayBuffer(MAX_IMPORT_BYTES+1)}), /size/);
});
test('native import errors expose only known codes and bounded row numbers', () => {
  assert.deepEqual(importErrorParts('duplicate:42'), {key:'duplicate',row:'42'});
  assert.deepEqual(importErrorParts(new Error('format')), {key:'format',row:''});
  assert.deepEqual(importErrorParts('unknown:private details'), {key:'database',row:''});
  assert.deepEqual(importErrorParts('invalid_row:99999'), {key:'invalid_row',row:''});
});

 test('relational import errors remain distinguishable without leaking details', () => {
  assert.deepEqual(importErrorParts('relations'), {key:'relations',row:''});
  assert.deepEqual(importErrorParts('collector_conflict'), {key:'collector_conflict',row:''});
  assert.deepEqual(importErrorParts(new Error('private source value')), {key:'database',row:''});
});

test('receipt label is declared once in each translated dictionary', () => {
  const source = readFileSync(new URL('../src/app/i18n.ts', import.meta.url), 'utf8');
  for (const lang of ['EN','ES','FR','DE']) {
    const start = source.indexOf('const ' + lang + ':');
    assert.ok(start >= 0);
    const end = source.indexOf('\n};', start);
    assert.ok(end > start);
    const dictionary = source.slice(start, end);
    assert.equal(dictionary.match(/^\s*"Incassi registrati":/gm)?.length, 1, lang);
    assert.notEqual(translate('Incassi registrati', lang.toLowerCase()), 'Incassi registrati');
  }
});
