// Explicit integration gate: real PostgreSQL, a fresh temporary cluster only.
// Run as a non-root Linux user with initdb, pg_ctl, psql and Node 22 on PATH.
// No database URL, existing cluster, production data or credentials are accepted.
import assert from 'node:assert/strict';
import test from 'node:test';
import { spawnSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm, stat } from 'node:fs/promises';
import { createServer } from 'node:net';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { exportLocalCopy, localEnvironment } from '../scripts/export-gestionale-artwork-metadata.mjs';
import { exportRelations } from '../scripts/export-gestionale-archive-relations.mjs';
import { exportArtworkMedia } from '../scripts/export-gestionale-artwork-media.mjs';
import { verifyArchiveMediaBatches } from '../scripts/verify-archive-media-batches.mjs';

const env=localEnvironment(process.env);
function command(bin,args,input) {
  const result=spawnSync(bin,args,{env,input,encoding:'utf8',timeout:30000,maxBuffer:1024*1024});
  assert.ifError(result.error);
  assert.equal(result.status,0,bin+' failed: '+result.stderr);
  return result.stdout;
}
async function freePort() {
  const server=createServer();
  await new Promise((resolve,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',resolve);});
  const port=String(server.address().port);
  await new Promise((resolve,reject)=>server.close(error=>error?reject(error):resolve()));
  return port;
}
const schema=`
CREATE TABLE opere (id integer PRIMARY KEY,id_utente_proprietario integer,codice_archivio text,titolo text,
 anno_realizzazione integer,tecnica text,supporto text,larghezza_cm numeric,altezza_cm numeric,profondita_cm numeric,
 descrizione text,stato text,firma_presente boolean,cornice_presente boolean,vernice_finale text,
 coefficiente_artista_override numeric,prezzo_personalizzato numeric,note_private text);
CREATE TABLE clienti (id integer PRIMARY KEY,id_utente_proprietario integer,nome text,cognome text,email text,
 telefono text,indirizzo text,note text,data_nascita date,ora_nascita time,luogo_nascita text,
 instagram text,facebook text,tiktok text,sito_web text,linkedin text,percorso_foto text);
CREATE TABLE vendite (id integer PRIMARY KEY,id_opera integer,id_cliente integer,id_utente_proprietario integer,
 prezzo_vendita numeric,tasse numeric,costi_spedizione numeric,data_ora_vendita timestamp,note text,tipo_movimento text,
 orario_vendita_conosciuto boolean,metodo_pagamento text,canale_vendita text,campagna text);
CREATE TABLE vendite_incassi (id integer PRIMARY KEY,id_vendita integer,importo numeric,data_incasso date,
 metodo_pagamento text,note text,id_scadenza integer);
CREATE TABLE vendite_scadenze (id integer PRIMARY KEY,id_vendita integer,numero integer,importo_previsto numeric,
 data_scadenza date,note text);
CREATE TABLE immagini_cliente (id_cliente integer);
ALTER TABLE opere ADD COLUMN id_upload_immagine_principale integer;
CREATE TABLE immagini_opera (id integer PRIMARY KEY,id_opera integer,id_upload integer,
 ordine_visualizzazione integer,tipo text,descrizione text);
CREATE TABLE uploads (id integer PRIMARY KEY,id_utente_proprietario integer,stato text,current_revision_id integer);
CREATE TABLE media_revisions (id integer PRIMARY KEY,id_upload integer,status text,result_blob_id integer);
CREATE TABLE media_blobs (id integer PRIMARY KEY,id_utente_proprietario integer,storage_provider text,status text,
 storage_key text,sha256 text,dimensione_byte bigint);
INSERT INTO opere (id,id_utente_proprietario,codice_archivio,titolo,anno_realizzazione,larghezza_cm,altezza_cm,
 stato,firma_presente,cornice_presente,vernice_finale,prezzo_personalizzato,note_private) VALUES
 (1,7,'PG-1','Città 🎨',2024,30.25,40,'DISPONIBILE',true,false,'OPACA',0.29,'never-export-private'),
 (2,7,'PG-2','Sale',2024,30,40,'VENDUTA',true,false,'LUCIDA',NULL,NULL),
 (3,7,'PG-3','Plan',2024,30,40,'VENDUTA',true,false,'SATINATA',NULL,NULL),
 (4,8,'PG-4','Foreign',2024,30,40,'DISPONIBILE',true,false,'OPACA',NULL,NULL),
 (5,7,'PG-5','Foreign collector',2024,30,40,'VENDUTA',true,false,'OPACA',NULL,NULL),
 (6,7,'PG-6','Inconsistent sale',2024,30,40,'DISPONIBILE',true,false,'OPACA',NULL,NULL);
INSERT INTO clienti (id,id_utente_proprietario,nome,cognome,data_nascita,ora_nascita) VALUES
 (10,7,'Synthetic','Collector','1980-02-29','07:45:00'),(11,8,'Foreign','Collector',NULL,NULL);
INSERT INTO vendite (id,id_opera,id_cliente,id_utente_proprietario,prezzo_vendita,tasse,costi_spedizione,
 data_ora_vendita,note,tipo_movimento,orario_vendita_conosciuto) VALUES
 (20,2,10,7,100.25,10,5.25,'2024-02-29','receipt sale','VENDITA',false),
 (21,3,10,7,100.25,0,0,'2024-02-29','unpaid plan','VENDITA',false),
 (22,5,11,7,100,0,0,'2024-02-29',NULL,'VENDITA',false),
 (23,6,11,8,100,0,0,'2024-02-29',NULL,'VENDITA',false);
INSERT INTO vendite_incassi VALUES (30,20,0.29,'2024-03-01','Bonifico','synthetic receipt',NULL);
INSERT INTO vendite_scadenze VALUES
 (41,21,1,40.25,'2024-03-31','first'),(42,21,2,60,'2024-04-30',NULL);
CREATE ROLE fixture_reader LOGIN;
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO fixture_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO fixture_reader;
`;

test('real PostgreSQL local exporters preserve explicit finance and active media, rejecting unsafe selections atomically',async()=>{
  assert.notEqual(process.getuid?.(),0,'initdb must run as a non-root user');
  const root=await mkdtemp('/tmp/repertica-pg-');
  const data=join(root,'data'),socket=join(root,'socket'),log=join(root,'postgres.log');
  let initialized=false;
  try {
    await mkdir(socket,{mode:0o700});
    const port=await freePort();
    // Only the disposable cluster listens on loopback; use its private Unix socket
    // for all setup and mutations. A failed bind aborts before the exporters run.
    command('initdb',['-D',data,'-U','fixture_admin','--auth-local=trust','--auth-host=trust','--encoding=UTF8','--no-locale']);
    initialized=true;
    // The admin may connect only through the private socket. TCP trust is
    // restricted to the SELECT-only fixture role and synthetic database.
    await writeFile(join(data,'pg_hba.conf'),
      'local all fixture_admin trust\nhost postgres fixture_reader 127.0.0.1/32 trust\n');
    command('pg_ctl',['-D',data,'-l',log,'-o','-h 127.0.0.1 -p '+port+' -k '+socket,'-w','-t','15','start']);
    const admin=['-X','-w','-qAt','-h',socket,'-p',port,'-U','fixture_admin','-d','postgres','-v','ON_ERROR_STOP=1'];
    const sql=input=>command('psql',admin,input);
    sql(schema);
    const denied=spawnSync('psql',['-X','-w','-qAt','-h','127.0.0.1','-p',port,'-U','fixture_admin','-d','postgres','-c','SELECT 1'],{env,encoding:'utf8',timeout:5000});
    assert.ifError(denied.error);
    assert.notEqual(denied.status,0,'admin TCP connection must be denied');
    assert.equal(sql("SELECT has_table_privilege('fixture_reader','public.opere','SELECT'),has_table_privilege('fixture_reader','public.opere','UPDATE');").trim(),'t|f');
    const meta=join(root,'metadata.json'),relations=join(root,'relations.json'),shares=join(root,'shares.json');
    const allShares=['PG-2','PG-3'].map(artwork_code=>({artwork_code,legacy_shares_bp:[5000,3000,2000]}));
    await writeFile(shares,JSON.stringify(allShares));
    const metadataArgs=(ids,out)=>['--local-copy','postgres','fixture_reader',port,'7',ids,out];
    const relationalArgs=(ids,out)=>[...metadataArgs(ids,out).slice(0,6),shares,out];
    const missing=async path=>assert.rejects(stat(path),{code:'ENOENT'});
    await exportLocalCopy(metadataArgs('1',meta));
    const m=JSON.parse(await readFile(meta,'utf8'));
    assert.equal(m.artworks.length,1);
    assert.equal(m.artworks[0].title,'Città 🎨');
    assert.equal(m.artworks[0].fixed_price_cents,29);
    assert.equal(m.artworks[0].width_cm,30.25);
    assert.ok(!(await readFile(meta,'utf8')).includes('never-export-private'));
    const original=await readFile(meta);
    await assert.rejects(exportLocalCopy(metadataArgs('1',meta)));
    assert.deepEqual(await readFile(meta),original);
    for(const ids of ['1,4','1,6','1,2','1,999','1,1']) {
      const out=join(root,'bad-metadata.json');
      await assert.rejects(exportLocalCopy(metadataArgs(ids,out)));
      await missing(out);
    }
    await exportRelations(relationalArgs('1,2,3',relations));
    const result=JSON.parse(await readFile(relations,'utf8'));
    assert.equal(result.artworks.length,3);
    assert.equal(result.collectors.length,1);
    assert.equal(result.collectors[0].birth_time,'07:45');
    assert.equal(result.collectors[0].birth_date,'1980-02-29');
    const receipt=result.sales.find(s=>s.artwork_code==='PG-2');
    const plan=result.sales.find(s=>s.artwork_code==='PG-3');
    assert.equal(receipt.collector_key,result.collectors[0].key);
    assert.equal(plan.collector_key,receipt.collector_key);
    assert.equal(receipt.amount_cents,10025);
    assert.equal(receipt.tax_cents,1000);
    assert.equal(receipt.shipping_cents,525);
    assert.equal(receipt.receipts[0].amount_cents,29);
    assert.equal(receipt.receipts[0].method,'bank_transfer');
    assert.deepEqual(receipt.legacy_shares_bp,[5000,3000,2000]);
    assert.deepEqual(plan.receipts,[]);
    assert.deepEqual(plan.installments.map(i=>[i.position,i.amount_cents,i.due_date]),[[1,4025,'2024-03-31'],[2,6000,'2024-04-30']]);
    const rejected=join(root,'bad-relations.json');
    for(const ids of ['1,2,3,4','1,2,3,5','1,2,3,6','1,2,3,999']) {
      await assert.rejects(exportRelations(relationalArgs(ids,rejected)));
      await missing(rejected);
    }
    sql("INSERT INTO vendite_incassi VALUES (31,21,1,'2024-03-01','Bonifico',NULL,NULL);");
    await assert.rejects(exportRelations(relationalArgs('1,2,3',rejected)));
    await missing(rejected);
    sql("DELETE FROM vendite_incassi WHERE id=31; INSERT INTO immagini_cliente VALUES(10);");
    await assert.rejects(exportRelations(relationalArgs('1,2,3',rejected)));
    await missing(rejected);
    // Real, independently generated 2×2 RGB PNGs, not header-only stubs.
    const images=[
      'iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAFklEQVR4nGO8I6fBwMDAxMDAwMDAAAAOdgEmkr/FpgAAAABJRU5ErkJggg==',
      'iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAFklEQVR4nGMUqbjDwMDAxMDAwMDAAAAQQgFsyLsfzAAAAABJRU5ErkJggg==',
      'iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAC0lEQVR4nGNgQAYAAA4AAamRc7EAAAAASUVORK5CYII=',
    ].map(s=>Buffer.from(s,'base64'));
    const digest=b=>createHash('sha256').update(b).digest('hex');
    const mediaRoot=join(root,'media'), mediaOutput=join(root,'with-media.json');
    const batchOutput=join(root,'media-batches');
    await mkdir(join(mediaRoot,'blobs'),{recursive:true});
    for(const [i,b] of images.entries()) {
      await writeFile(join(mediaRoot,'blobs',i+'.png'),b);
      // All interpolated values originate from fixed synthetic bytes/integers.
      sql(`INSERT INTO media_blobs VALUES (${100+i},7,'LOCAL','AVAILABLE','blobs/${i}.png','${digest(b)}',${b.length});`);
    }
    sql(`
      INSERT INTO uploads VALUES (50,7,'READY',60),(51,7,'READY',61),(52,7,'READY',62);
      INSERT INTO media_revisions VALUES (60,50,'ACTIVE',100),(61,51,'ACTIVE',101),
        (62,52,'ACTIVE',102),(63,50,'ACTIVE',102);
      UPDATE opere SET id_upload_immagine_principale=50 WHERE id=1;
      INSERT INTO immagini_opera VALUES (80,2,52,20,'FIRMA','Firma'),
        (81,2,51,10,'DETTAGLIO','Dettaglio 🎨');
    `);
    const mediaArgs=(ids,out)=>[...metadataArgs(ids,out).slice(0,6),relations,mediaRoot,out];
    await exportArtworkMedia(mediaArgs('1,2,3',mediaOutput));
    const mediaBytes=await readFile(mediaOutput), media=JSON.parse(mediaBytes);
    assert.deepEqual(media.artworks,result.artworks);
    assert.deepEqual(media.collectors,result.collectors);
    assert.deepEqual(media.sales,result.sales);
    assert.equal(media.covers.length,1);
    assert.equal(media.covers[0].artwork_code,'PG-1');
    assert.equal(media.covers[0].sha256,digest(images[0]));
    assert.deepEqual(Buffer.from(media.covers[0].data_base64,'base64'),images[0]);
    assert.deepEqual(media.gallery.map(p=>[p.artwork_code,p.position,p.kind,p.caption]),[
      ['PG-2',1,'DETAIL','Dettaglio 🎨'],['PG-2',2,'SIGNATURE','Firma'],
    ]);
    for(const [i,p] of media.gallery.entries()) {
      assert.equal(p.sha256,digest(images[i+1]));
      assert.deepEqual(Buffer.from(p.data_base64,'base64'),images[i+1]);
    }
    assert.ok(!mediaBytes.toString('utf8').includes('blobs/'));
    await assert.rejects(exportArtworkMedia(mediaArgs('1,2,3',mediaOutput)));
    assert.deepEqual(await readFile(mediaOutput),mediaBytes);
    await exportArtworkMedia([...mediaArgs('1,2,3',batchOutput),'--batch-directory']);
    assert.deepEqual(await verifyArchiveMediaBatches(batchOutput),
      {packages:1,artworks:3,collectors:1,sales:2,photographs:3});
    const manifest=JSON.parse(await readFile(join(batchOutput,'manifest.json'),'utf8'));
    const batchBytes=await readFile(join(batchOutput,manifest.packages[0].file));
    assert.equal(digest(batchBytes),manifest.packages[0].sha256);
    const batch=JSON.parse(batchBytes);
    for(const key of ['artworks','collectors','sales','covers','gallery']) {
      const sorted=rows=>rows.map(r=>JSON.stringify(r)).sort();
      assert.deepEqual(sorted(batch[key]),sorted(media[key]));
    }

    const badMedia=join(root,'bad-media.json'),badBatch=join(root,'bad-media-batch');
    for(const ids of ['1,2,4','1,2,999','1,2','1,2,2']) {
      await assert.rejects(exportArtworkMedia(mediaArgs(ids,badMedia)));
      await missing(badMedia);
    }
    // Keep the actual PostgreSQL joins in the test: each corruption must fail
    // even though the input package and the other registered media are valid.
    for(const [corrupt,restore] of [
      ["UPDATE uploads SET id_utente_proprietario=8 WHERE id=50","UPDATE uploads SET id_utente_proprietario=7 WHERE id=50"],
      ["UPDATE uploads SET stato='PENDING' WHERE id=50","UPDATE uploads SET stato='READY' WHERE id=50"],
      ["UPDATE uploads SET current_revision_id=61 WHERE id=50","UPDATE uploads SET current_revision_id=60 WHERE id=50"],
      ["UPDATE media_revisions SET status='INACTIVE' WHERE id=60","UPDATE media_revisions SET status='ACTIVE' WHERE id=60"],
      ["UPDATE media_blobs SET id_utente_proprietario=8 WHERE id=100","UPDATE media_blobs SET id_utente_proprietario=7 WHERE id=100"],
      ["UPDATE media_blobs SET storage_provider='S3' WHERE id=100","UPDATE media_blobs SET storage_provider='LOCAL' WHERE id=100"],
      ["UPDATE media_blobs SET status='MISSING' WHERE id=100","UPDATE media_blobs SET status='AVAILABLE' WHERE id=100"],
      ["UPDATE media_blobs SET storage_key='blobs/missing.png' WHERE id=100","UPDATE media_blobs SET storage_key='blobs/0.png' WHERE id=100"],
      ["UPDATE media_blobs SET sha256=repeat('0',64) WHERE id=100",`UPDATE media_blobs SET sha256='${digest(images[0])}' WHERE id=100`],
      ["UPDATE immagini_opera SET tipo='UNKNOWN' WHERE id=81","UPDATE immagini_opera SET tipo='DETTAGLIO' WHERE id=81"],
    ]) {
      sql(corrupt);
      await assert.rejects(exportArtworkMedia(mediaArgs('1,2,3',badMedia)));
      await missing(badMedia);
      await assert.rejects(exportArtworkMedia([...mediaArgs('1,2,3',badBatch),'--batch-directory']));
      await missing(join(badBatch,'manifest.json'));
      await missing(badBatch);
      sql(restore);
    }
    // Re-export from the restored fixture proves the negative cases did not
    // accidentally poison the source or leave partial destination files.
    await exportArtworkMedia(mediaArgs('1,2,3',badMedia));
    assert.deepEqual(JSON.parse(await readFile(badMedia,'utf8')),media);
    assert.deepEqual(await readFile(mediaOutput),mediaBytes);
    assert.equal(sql('SELECT count(*) FROM media_blobs;').trim(),'3');
    // Existing package is still intact after all rejected exports.
    assert.deepEqual(JSON.parse(await readFile(relations,'utf8')),result);
    assert.equal(sql('SELECT count(*) FROM opere;').trim(),'6');
    assert.equal(sql('SELECT count(*) FROM vendite;').trim(),'4');
    // Explicit CI gate: hand the actual exporter output (not a recreated JSON
    // fixture) to native Studio while this private temporary directory exists.
    if (process.env.REPERTICA_TEST_NATIVE_ROUNDTRIP === '1') {
      const native=spawnSync('cargo',[
        'test','--manifest-path','src-tauri/Cargo.toml',
        'archive_import::batches::tests::postgres_export_native_roundtrip',
        '--','--ignored','--exact','--nocapture',
      ],{env:{...process.env,REPERTICA_TEST_BATCH_DIRECTORY:batchOutput},
        encoding:'utf8',timeout:180000,maxBuffer:4*1024*1024});
      process.stdout.write(native.stdout??'');
      process.stderr.write(native.stderr??'');
      assert.ifError(native.error);
      assert.equal(native.status,0,'native import/backup/restore test failed');
      assert.match(native.stdout,/test archive_import::batches::tests::postgres_export_native_roundtrip \.\.\. ok/);
      assert.match(native.stdout,/1 passed; 0 failed; 0 ignored/);
    }
  } finally {
    // A timed-out startup can still leave a server running. Stop the exact
    // temporary data directory before removal, and never remove live data.
    if(initialized) {
      const status=spawnSync('pg_ctl',['-D',data,'status'],{env,encoding:'utf8',timeout:5000});
      assert.ifError(status.error);
      if(status.status===0) command('pg_ctl',['-D',data,'-m','fast','-w','-t','15','stop']);
      else assert.equal(status.status,3,'could not verify temporary server stopped');
    }
    await rm(root,{recursive:true,force:true});
  }
});
