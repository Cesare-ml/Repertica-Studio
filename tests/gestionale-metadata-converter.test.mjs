import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, readFile, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { convertLegacyRows, convertFile } from '../scripts/convert-gestionale-artwork-metadata.mjs';
const sample = { codice_archivio: 'TEST-01', titolo: ' Étude ', stato: 'DISPONIBILE', id_vendita: null,
  firma_presente: true, cornice_presente: false, vernice_finale: 'SATINATA',
  anno_realizzazione: 2024, larghezza_cm: '30.25', prezzo_personalizzato: '0.29' };

test('preserves explicit metadata and exact EUR cents', () => {
  const result = JSON.parse(convertLegacyRows([sample, { ...sample, codice_archivio:'TEST-02', stato:'RISERVATA', vernice_finale:false, prezzo_personalizzato:0 }]));
  assert.equal(result.format, 'repertica-studio-artwork-metadata-v1');
  assert.deepEqual(result.artworks[0], {code:'TEST-01',title:'Étude',year:'2024',technique:null,support:null,width_cm:30.25,height_cm:null,depth_cm:null,description:null,status:'available',signed:true,framed:false,final_varnish:true,artist_coefficient_override:null,fixed_price_cents:29});
  assert.equal(result.artworks[1].status,'reserved');
  assert.equal(result.artworks[1].fixed_price_cents,0);
  assert.equal(result.artworks[1].final_varnish,false);
});
test('rejects sales, donations, ambiguous states and private fields, even in the second row', () => {
  for (const change of [{stato:'VENDUTA'}, {stato:'DONATA'}, {stato:'DA_VERIFICARE'}, {stato:'COLLEZIONE_PRIVATA'}, {id_vendita:123}, {id_vendita:undefined}, {note_private:'synthetic'}, {url_immagine_principale:'https://example.invalid/photo'}]) {
    assert.throws(() => convertLegacyRows([sample,{...sample,codice_archivio:'B',...change}]), /Row 2:/);
  }
});
test('never invents unknown facts, rounds fractional cents or accepts invalid dimensions', () => {
  for (const change of [{firma_presente:null}, {cornice_presente:undefined}, {vernice_finale:null}, {prezzo_personalizzato:'0.001'}, {prezzo_personalizzato:'1000000000.01'}, {larghezza_cm:0}, {larghezza_cm:'1e4'}, {codice_archivio:'ART-9223372036854775807'}]) {
    assert.throws(() => convertLegacyRows([{...sample,...change}]), /Row 1:/);
  }
});
test('matches SQLite ASCII-only duplicate comparison and enforces limits', () => {
  assert.throws(() => convertLegacyRows([sample,{...sample,codice_archivio:'test-01'}]), /duplicate archive code/);
  assert.equal(JSON.parse(convertLegacyRows([{...sample,codice_archivio:'É'},{...sample,codice_archivio:'é'}])).artworks.length,2);
  assert.throws(() => convertLegacyRows([]), /1000/);
  assert.throws(() => convertLegacyRows(Array(1001).fill(sample)), /1000/);
  assert.throws(() => convertLegacyRows(Array.from({length:60},(_,i)=>({...sample,codice_archivio:`A-${i}`,descrizione:'x'.repeat(40000)}))), /2 MiB/);
});
test('file conversion is local, validates before writing and never overwrites', async () => {
  const dir = await mkdtemp(join(tmpdir(),'repertica-converter-'));
  try {
    const input = join(dir,'input.json'), output = join(dir,'output.json');
    await writeFile(input,JSON.stringify([sample]));
    await convertFile(input,output);
    assert.equal(await readFile(output,'utf8'),convertLegacyRows([sample]));
    await assert.rejects(convertFile(input,output), {code:'EEXIST'});
    await assert.rejects(convertFile(input,input), {code:'EEXIST'});
    const invalid = join(dir,'invalid.json'), absent = join(dir,'absent.json');
    await writeFile(invalid, JSON.stringify([{...sample,stato:'VENDUTA'}]));
    await assert.rejects(convertFile(invalid,absent), /commercial/);
    await assert.rejects(stat(absent), {code:'ENOENT'});
    await writeFile(invalid,Buffer.alloc(2*1024*1024+1));
    await assert.rejects(convertFile(invalid,absent), /2 MiB/);
    await assert.rejects(stat(absent), {code:'ENOENT'});
  } finally { await rm(dir,{recursive:true,force:true}); }
});

test('raw duplicate JSON fields cannot hide a sale or a commercial state', async () => {
  const dir = await mkdtemp(join(tmpdir(),'repertica-converter-duplicate-'));
  try {
    const input = join(dir,'input.json'), output = join(dir,'output.json');
    const valid = JSON.stringify([sample]);
    for (const raw of [
      valid.replace('"stato":"DISPONIBILE"', '"stato":"VENDUTA","stato":"DISPONIBILE"'),
      valid.replace('"id_vendita":null', '"id_vendita":123,"id_vendita":null'),
      valid.replace('"stato":"DISPONIBILE"', '"stato":"VENDUTA","st\\u0061to":"DISPONIBILE"'),
    ]) {
      await writeFile(input, raw);
      await assert.rejects(convertFile(input,output), /Duplicate JSON field/);
      await assert.rejects(stat(output), {code:'ENOENT'});
    }
    await writeFile(input,JSON.stringify([sample,{...sample,codice_archivio:'OTHER',titolo:'{ "stato": "example" }'}]));
    await convertFile(input,output);
    assert.equal(JSON.parse(await readFile(output,'utf8')).artworks.length,2);
  } finally { await rm(dir,{recursive:true,force:true}); }
});
