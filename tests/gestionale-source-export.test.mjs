import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { DatabaseSync } from 'node:sqlite';
import { exportOptions, localEnvironment, checkedPackage } from '../scripts/export-gestionale-artwork-metadata.mjs';

const validArgs = ['--local-copy', 'archive_copy', 'reader', '5432', '7', '1,2', 'new.json'];
test('requires an explicit local copy, bounded unique IDs and no connection strings', () => {
  const { psqlArgs } = exportOptions(validArgs);
  assert.equal(psqlArgs[psqlArgs.indexOf('-h') + 1], '127.0.0.1');
  assert.ok(psqlArgs.includes('artwork_ids={1,2}'));
  for (const [index, value] of [[0,'--remote'],[1,'postgresql://other/db'],[1,'host=other'],[2,'-x'],[3,'65536'],[4,'0'],[5,'1,1'],[5,'1,2;SELECT'],[5,'2147483648'],[5,Array.from({length:1001},(_,i)=>i+1).join(',')]]) {
    const args = [...validArgs]; args[index] = value;
    assert.throws(() => exportOptions(args), /Usage/);
  }
  assert.deepEqual(localEnvironment({PATH:'test',PGHOSTADDR:'other',PGSERVICE:'other',pgoptions:'other',PGPASSWORD:'example'}), {PATH:'test',PGCONNECT_TIMEOUT:'5',PGCLIENTENCODING:'UTF8'});
});

// Execute the exact shared relational selection/guard CTE in SQLite with only
// the PostgreSQL array input, schema names and psql owner parameter adapted.
// PostgreSQL JSON aggregation and psql execution require a separate real PG test.
const sql = readFileSync(new URL('../scripts/gestionale-artwork-export.sql', import.meta.url), 'utf8');
const selectionSql = sql.slice(sql.indexOf('WITH requested'), sql.indexOf('SELECT COALESCE(json_agg'))
  .replace("SELECT unnest(:'artwork_ids'::integer[])", 'SELECT value FROM json_each(?)')
  .replaceAll('public.', '').replace(":'owner_id'", '?') + 'SELECT * FROM guarded ORDER BY codice_archivio';

test('selection is all-or-nothing, owner scoped, and blocks any historical sale', () => {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec(`CREATE TABLE opere (id INTEGER PRIMARY KEY, id_utente_proprietario INTEGER, codice_archivio TEXT, titolo TEXT, anno_realizzazione INTEGER, tecnica TEXT, supporto TEXT, larghezza_cm TEXT, altezza_cm TEXT, profondita_cm TEXT, descrizione TEXT, stato TEXT, firma_presente BOOLEAN, cornice_presente BOOLEAN, vernice_finale TEXT, coefficiente_artista_override TEXT, prezzo_personalizzato TEXT, note_private TEXT);
      CREATE TABLE vendite (id INTEGER PRIMARY KEY, id_opera INTEGER, id_utente_proprietario INTEGER);
      INSERT INTO opere(id,id_utente_proprietario,codice_archivio,titolo,stato,firma_presente,cornice_presente,vernice_finale,prezzo_personalizzato,note_private) VALUES
      (1,7,'A','One','DISPONIBILE',1,0,'SATINATA','0.29','excluded'),
      (2,7,'B','Two','RISERVATA',0,1,'OPACA',NULL,'excluded'),
      (3,8,'C','Other','DISPONIBILE',1,0,'SATINATA',NULL,'excluded'),
      (4,7,'D','Sale','DISPONIBILE',1,0,'SATINATA',NULL,'excluded'),
      (5,7,'E','Unknown','DISPONIBILE',NULL,0,'SATINATA',NULL,'excluded'),
      (6,7,'F','Sold','VENDUTA',1,0,'SATINATA',NULL,'excluded'),
      (7,7,'G','Varnish','DISPONIBILE',1,0,NULL,NULL,'excluded');
      INSERT INTO vendite VALUES(1,4,8);`);
    const select = ids => db.prepare(selectionSql).all(JSON.stringify(ids), 7);
    const rows = select([1,2]);
    assert.equal(rows.length, 2);
    assert.equal(rows[0].prezzo_personalizzato, '0.29');
    assert.equal(rows[0].id_vendita, null);
    for (const field of ['id','id_utente_proprietario','note_private']) assert.ok(!(field in rows[0]));
    for (const ids of [[],[1,1],[1,3],[1,4],[1,5],[1,6],[1,7],[1,999]]) assert.equal(select(ids).length, 0);
    // SQLite represents booleans as integers; PG row_to_json emits booleans.
    const pgShaped = rows.map(r => ({...r, firma_presente: Boolean(r.firma_presente), cornice_presente: Boolean(r.cornice_presente)}));
    const result = JSON.parse(checkedPackage(Buffer.from(JSON.stringify(pgShaped))));
    assert.equal(result.artworks[0].fixed_price_cents,29);
    assert.equal(result.artworks[1].status,'reserved');
  } finally { db.close(); }
});

test('empty, malformed or oversized source results never become import packages', () => {
  assert.throws(() => checkedPackage(Buffer.from('[]')), /Selection rejected/);
  assert.throws(() => checkedPackage(Buffer.from('{}')));
  assert.throws(() => checkedPackage(Buffer.from([255])));
  assert.throws(() => checkedPackage(Buffer.alloc(2*1024*1024+1)), /2 MiB/);
});
