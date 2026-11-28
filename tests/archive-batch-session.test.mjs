import test from 'node:test';
import assert from 'node:assert/strict';
import {BatchImportSession} from '../src/app/archive-batch-progress.ts';
const m='c'.repeat(64), a='a'.repeat(64), b='b'.repeat(64);
const plan=()=>({manifest_digest:m,packages:[{sha256:a,artworks:1},{sha256:b,artworks:2}]});
const preview=i=>({digest:i?b:a,artworks:Array.from({length:i?2:1},()=>({code:'synthetic'}))});
const transport=()=>({preflight:async()=>plan(),review:async(_d,_m,i)=>preview(i),commit:async(_d,_m,i)=>i?2:1});
test('pins folder, manifest, index and preview digest to each explicit commit',async()=>{
  const calls=[], t=transport();
  t.commit=async(...args)=>{calls.push(args);return args[2]?2:1;};
  const s=new BatchImportSession(t);
  await s.start('/chosen');
  await assert.rejects(s.commitReviewed(),/batch_phase/);
  await s.reviewNext();
  const exposed=s.preview;exposed.digest=b;
  s.state.packages[0].sha256=b;
  await s.commitReviewed();
  assert.deepEqual(calls,[['/chosen',m,0,a]]);
  assert.equal(s.state.imported,1);
  await assert.rejects(s.commitReviewed(),/batch_phase/);
  await s.reviewNext();await s.commitReviewed();
  assert.deepEqual(calls[1],['/chosen',m,1,b]);
  assert.deepEqual([s.state.phase,s.state.imported],['complete',3]);
});
test('rejects overlapping calls, cancellation and reset during an in-flight commit',async()=>{
  const t=transport();let finish;let writes=0;
  t.commit=()=>{writes++;return new Promise(resolve=>{finish=resolve;});};
  const s=new BatchImportSession(t);await s.start('/chosen');await s.reviewNext();
  const pending=s.commitReviewed();
  await assert.rejects(s.commitReviewed(),/batch_busy/);
  await assert.rejects(s.start('/other'),/batch_busy/);
  assert.throws(()=>s.stop(),/batch_busy/);
  assert.throws(()=>s.reset(true),/batch_busy/);
  finish(1);await pending;assert.equal(writes,1);assert.equal(s.state.imported,1);
});
test('failed review or inconsistent count cannot authorize a write',async()=>{
  const t=transport();t.review=async()=>({digest:a,artworks:[]});
  const s=new BatchImportSession(t);await s.start('/chosen');
  await assert.rejects(s.reviewNext(),/batch_changed/);
  assert.equal(s.preview,null);assert.equal(s.state.phase,'review');
  await assert.rejects(s.commitReviewed(),/batch_phase/);
  t.review=async()=>({digest:b,artworks:[{}]});
  await assert.rejects(s.reviewNext(),/batch_changed/);
  s.stop();s.reset();assert.equal(s.state,null);
});
test('lost reply or wrong count preserves confirmed earlier packages and forbids retry',async()=>{
  for(const fail of [async()=>{throw Error('lost');},async()=>0]){
    const t=transport();const s=new BatchImportSession(t);await s.start('/chosen');
    await s.reviewNext();await s.commitReviewed();await s.reviewNext();t.commit=fail;
    await s.commitReviewed();assert.deepEqual([s.state.phase,s.state.imported,s.state.next],['unknown',1,1]);
    await assert.rejects(s.reviewNext(),/batch_phase/);
    await assert.rejects(s.commitReviewed(),/batch_phase/);
    assert.throws(()=>s.reset(),/batch_phase/);
    s.reset(true);assert.equal(s.state,null);
  }
});
test('read-only preflight failure leaves no session; stopped review cannot write',async()=>{
  const t=transport();const s=new BatchImportSession(t);
  t.preflight=async()=>{throw Error('bad manifest');};
  await assert.rejects(s.start('/chosen'),/bad manifest/);assert.equal(s.busy,false);assert.equal(s.state,null);
  t.preflight=async()=>plan();await s.start('/chosen');await s.reviewNext();s.stop();
  assert.equal(s.preview,null);await assert.rejects(s.commitReviewed(),/batch_phase/);
});

test('desktop batch import wires explicit native selection, review and confirmation', async () => {
  const {readFileSync} = await import('node:fs');
  const source = path => readFileSync(new URL('../'+path, import.meta.url), 'utf8');
  const host = source('src-tauri/src/lib.rs');
  const api = source('src/app/api.ts');
  const ui = source('src/app/app.component.ts');
  const html = source('src/app/app.component.html');
  const i18n = source('src/app/i18n.ts');
  for (const command of ['preflight_archive_batches','review_archive_batch','commit_archive_batch']) {
    assert.ok(host.includes('archive_import::batches::'+command));
    assert.ok(api.includes("'"+command+"'"));
  }
  assert.ok(ui.includes('directory:true, multiple:false'));
  assert.ok(ui.includes('await this.batchSession.commitReviewed()'));
  assert.ok(ui.includes("phase === 'unknown' && !(await confirmed"));
  assert.ok(ui.includes('this.batchState || !this.restoreCandidate'));
  assert.ok(html.includes('batchPreview || metadataImport'));
  assert.ok(html.includes("(click)=\"confirmReviewedBatch()\""));
  for (const key of ["Scegli cartella di migrazione","Ogni pacchetto richiede una conferma separata. I pacchetti già importati restano nell’archivio anche se interrompi la procedura.","Pacchetti confermati:","Pacchetto corrente:","Importazione della cartella completata.","Importazione interrotta. Le opere confermate restano nell’archivio.","Esito incerto: il pacchetto potrebbe essere già stato importato. Controlla l’archivio prima di proseguire.","Verifica il prossimo pacchetto","Interrompi importazione","Chiudi sessione di importazione","Importa pacchetto selezionato","Verifica del pacchetto non riuscita. Controlla la cartella e l’archivio.","Importare solo il pacchetto in anteprima? I pacchetti già importati non saranno annullati.","Ho controllato manualmente l’archivio e non ripeterò alla cieca il pacchetto incerto."]) {
    assert.equal(i18n.split(JSON.stringify(key)+':').length-1,4, key);
  }
});

test('import confirmation handlers wait, refuse cancellation and reject stale previews', async () => {
  const {readFileSync} = await import('node:fs');
  const source = readFileSync(new URL('../src/app/app.component.ts', import.meta.url), 'utf8');
  const methods = ['confirmReviewedBatch','resetBatchImport','confirmMetadataImport','restoreSelectedBackup'];
  for (const name of methods) {
    const start = source.indexOf('  async '+name+'(): Promise<void> {');
    assert.ok(start >= 0, name);
    const end = source.indexOf('\n  }', start);
    const code = source.slice(start, end+4).replace('(): Promise<void>', '()');
    for (const scenario of ['cancel','stale','accept']) {
      let answer, writes=0, cleared=0;
      const confirmed=()=>new Promise(resolve=>{answer=resolve;});
      const api={importArtworkMetadata:async()=>{writes++;return 1;},restoreBackup:async()=>{writes++;return {previousArchive:'synthetic'};}};
      const state = {phase:name==='resetBatchImport'?'unknown':'confirm'};
      const host = {busy:false, initialized:true, section:'settings', batchState:name.includes('Batch')?state:null,
        batchPreview:{digest:'a'}, metadataImport:{digest:'b'}, metadataSource:'source', restoreCandidate:{directory:'synthetic'},
        tr:x=>x, batchAction:async f=>f(), batchSession:{commitReviewed:async()=>{writes++;},reset:()=>{writes++;}},
        syncBatch(){}, refresh:async()=>{}, loadAllocationPlan:async()=>{}, loadSchedule:async()=>{}, closeGallery(){},
        clearMetadataImport(){cleared++;this.metadataImport=null;this.metadataSource=null;},
        change:{detectChanges(){}}, showMetadataImportError(e){throw e;}, error:e=>String(e)};
      const handler = new Function('confirmed','api','return ({'+code+'})')['call'](null,confirmed,api)[name];
      const pending=handler.call(host);
      assert.equal(writes,0,name+' before answer');
      assert.equal(cleared,0,name+' before answer');
      if(scenario==='stale') {
        if(name.includes('Batch')) host.batchState={...state};
        else if(name==='restoreSelectedBackup') host.batchState={phase:'review'};
        else host.metadataImport={digest:'new'};
      }
      answer(scenario!=='cancel');await pending;
      assert.equal(writes,scenario==='accept'?1:0,name+' '+scenario);
      assert.equal(cleared,scenario==='accept'&&!name.includes('Batch')?1:0,name+' clear '+scenario);
    }
  }
});
