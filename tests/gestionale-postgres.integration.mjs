// Explicit integration gate: real PostgreSQL, a fresh temporary cluster only.
// Run as a non-root Linux user with initdb, pg_ctl, psql and Node 22 on PATH.
// No database URL, existing cluster, production data or credentials are accepted.
import assert from 'node:assert/strict';
import test from 'node:test';
import { spawnSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm, stat } from 'node:fs/promises';
import { createServer } from 'node:net';
import { join } from 'node:path';
import { exportLocalCopy, localEnvironment } from '../scripts/export-gestionale-artwork-metadata.mjs';
import { exportRelations } from '../scripts/export-gestionale-archive-relations.mjs';

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
GRANT USAGE ON SCHEMA public TO fixture_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO fixture_reader;
`;

test('real PostgreSQL local exporters preserve explicit finance and reject unsafe selections atomically',async()=>{
  assert.notEqual(process.getuid?.(),0,'initdb must run as a non-root user');
  const root=await mkdtemp('/tmp/repertica-pg-');
  const data=join(root,'data'),socket=join(root,'socket'),log=join(root,'postgres.log');
  let started=false;
  try {
    await mkdir(socket,{mode:0o700});
    const port=await freePort();
    // Only the disposable cluster listens on loopback; use its private Unix socket
    // for all setup and mutations. A failed bind aborts before the exporters run.
    command('initdb',['-D',data,'-U','fixture_admin','--auth-local=trust','--auth-host=trust','--encoding=UTF8','--no-locale']);
    command('pg_ctl',['-D',data,'-l',log,'-o','-h 127.0.0.1 -p '+port+' -k '+socket,'-w','-t','15','start']);
    started=true;
    const admin=['-X','-w','-qAt','-h',socket,'-p',port,'-U','fixture_admin','-d','postgres','-v','ON_ERROR_STOP=1'];
    const sql=input=>command('psql',admin,input);
    sql(schema);
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
    // Existing package is still intact after all rejected exports.
    assert.deepEqual(JSON.parse(await readFile(relations,'utf8')),result);
    assert.equal(sql('SELECT count(*) FROM opere;').trim(),'6');
    assert.equal(sql('SELECT count(*) FROM vendite;').trim(),'4');
  } finally {
    try {
      if(started) command('pg_ctl',['-D',data,'-m','fast','-w','-t','15','stop']);
    } finally { await rm(root,{recursive:true,force:true}); }
  }
});
