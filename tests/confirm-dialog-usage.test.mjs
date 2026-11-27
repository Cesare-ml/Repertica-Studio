import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { confirmed } from '../src/app/confirm-dialog.ts';

const appDir = new URL('../src/app/', import.meta.url);
const sources = readdirSync(appDir, { recursive: true })
  .map(name => String(name).replaceAll('\\', '/'))
  .filter(name => name.endsWith('.ts') && name !== 'confirm-dialog.ts')
  .map(name => ({ name, text: readFileSync(new URL(name, appDir), 'utf8') }));

test('a destructive action waits for the answer and runs only on OK', async () => {
  // The shape of every call site: `if (!(await confirmed(...))) return;` followed by the action.
  let answer;
  globalThis.window = { confirm: () => new Promise(resolve => { answer = resolve; }) };
  let deleted = false;
  const remove = async () => { if (!(await confirmed('Delete?'))) return; deleted = true; };

  const cancelled = remove();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(deleted, false, 'nothing happens while the dialog is open');
  answer(false);
  await cancelled;
  assert.equal(deleted, false, 'Cancel keeps the item');

  const accepted = remove();
  answer(true);
  await accepted;
  assert.equal(deleted, true, 'OK runs the action');
  delete globalThis.window;
});

test('a dialog that cannot be shown or answers oddly counts as "no"', async () => {
  globalThis.window = { confirm: () => { throw new Error('not allowed'); } };
  assert.equal(await confirmed('Delete?'), false);
  globalThis.window = { confirm: () => Promise.resolve('Ok') };
  assert.equal(await confirmed('Delete?'), false);
  globalThis.window = { confirm: () => undefined };
  assert.equal(await confirmed('Delete?'), false);
  delete globalThis.window;
  assert.equal(await confirmed('Delete?'), false, 'no window at all');
});

test('no source in src/app calls window.confirm, alert or prompt directly', () => {
  // In the Desktop app these are asynchronous (Tauri dialog plugin): `if (!confirm(...))` never stops.
  for (const { name, text } of sources) {
    assert.doesNotMatch(text, /\b(?:confirm|alert|prompt)\s*\(/, `${name}: use await confirmed(...) from ./confirm-dialog`);
  }
});

test('every confirmation in src/app is awaited', () => {
  let calls = 0;
  for (const { name, text } of sources) {
    for (const match of text.matchAll(/\b(confirmed|canLeave)\(/g)) {
      const before = text.slice(Math.max(0, match.index - 40), match.index);
      if (/\b(?:async|function)\s+$/.test(before)) continue; // the declaration itself
      calls++;
      const line = text.slice(0, match.index).split('\n').length;
      assert.match(before, /\bawait\s+(?:this\.)?$/, `${name}:${line} ${match[1]}() must be awaited`);
    }
  }
  assert.ok(calls >= 25, `expected the confirmations of the whole app, found ${calls}`);
});

test('the archive restore confirmation shows real line breaks', () => {
  const app = readFileSync(new URL('app.component.ts', appDir), 'utf8');
  assert.match(app, /await confirmed\('RIPRISTINO ARCHIVIO\\n\\nL’archivio/);
  assert.doesNotMatch(app, /RIPRISTINO ARCHIVIO\\\\n/);
});

 
// Exercise the actual component handlers in isolation, without Angular, native IPC or SMTP.
function handler(source, start, end, signature, name, api, blankComposer) {
  const first = source.indexOf(start);
  const last = end ? source.indexOf(end, first) : source.lastIndexOf('\n}');
  assert.ok(first >= 0 && last > first, name);
  const method = source.slice(first, last).replace(signature, '()').trim();
  return new Function('confirmed', 'api', 'blankComposer', 'return ({' + method + '}).' + name)(confirmed, api, blankComposer);
}

test('restore rechecks the inspected candidate and busy state after native confirmation', async () => {
  const source = readFileSync(new URL('app.component.ts', appDir), 'utf8');
  for (const scenario of ['cancel', 'reject', 'accept', 'changed', 'busy']) {
    let answer, reject;
    const calls = [];
    globalThis.window = { confirm: () => new Promise((resolve, fail) => { answer = resolve; reject = fail; }) };
    const restore = handler(source, '  async restoreSelectedBackup()', '\n  bytes(', '(): Promise<void>', 'restoreSelectedBackup',
      { restoreBackup: async path => { calls.push(path); return { previousArchive: 'previous' }; } });
    const ctx = { busy: false, restoreCandidate: { directory: 'backup-A' }, closeGallery() {},
      refresh: async () => {}, loadAllocationPlan: async () => {}, loadSchedule: async () => {},
      error: String, change: { detectChanges() {} } };
    try {
      const pending = restore.call(ctx);
      assert.deepEqual(calls, [], scenario);
      if (scenario === 'changed') ctx.restoreCandidate = { directory: 'backup-B' };
      if (scenario === 'busy') ctx.busy = true;
      if (scenario === 'reject') reject(new Error('permission denied'));
      else answer(scenario !== 'cancel');
      await pending;
      assert.deepEqual(calls, scenario === 'accept' ? ['backup-A'] : [], scenario);
      if (scenario === 'changed') assert.equal(ctx.restoreCandidate.directory, 'backup-B');
      if (scenario === 'busy') assert.equal(ctx.busy, true);
    } finally { delete globalThis.window; }
  }
});

test('mail confirmation cannot authorize a changed message or account or duplicate send', async () => {
  const source = readFileSync(new URL('business-mail.component.ts', appDir), 'utf8');
  for (const scenario of ['cancel', 'reject', 'accept', 'account', 'body', 'attachment', 'busy', 'double']) {
    let answer, reject, prompts = 0;
    const calls = [];
    const blank = () => ({ id: null, revision: null, to: '', cc: '', bcc: '', subject: '', body: '', attachments: [] });
    globalThis.window = { confirm: () => { prompts++; return new Promise((resolve, fail) => { answer = resolve; reject = fail; }); } };
    const send = handler(source, '  async send()', null, '():Promise<void>', 'send',
      { mailSend: async input => { calls.push(input); return { delivery: 'accepted', sent_copy: 'saved' }; } }, blank);
    const ctx = { busy: false, sendConfirmationPending: false, selectedId: 'account-A', composeOpen: true,
      composer: { ...blank(), to: 'recipient@example.test', body: 'original', attachments: ['original.pdf'] },
      t: value => value, operation: async fn => fn() };
    try {
      const pending = send.call(ctx);
      assert.deepEqual(calls, [], scenario);
      if (scenario === 'account') ctx.selectedId = 'account-B';
      if (scenario === 'body') ctx.composer.body = 'changed';
      if (scenario === 'attachment') ctx.composer.attachments.push('another.pdf');
      if (scenario === 'busy') ctx.busy = true;
      if (scenario === 'double') { await send.call(ctx); assert.equal(prompts, 1); }
      if (scenario === 'reject') reject(new Error('permission denied'));
      else answer(scenario !== 'cancel');
      await pending;
      assert.equal(calls.length, ['accept', 'double'].includes(scenario) ? 1 : 0, scenario);
      if (calls.length) {
        assert.equal(calls[0].account_id, 'account-A');
        assert.equal(calls[0].body, 'original');
        assert.deepEqual(calls[0].attachments, ['original.pdf']);
      }
      assert.equal(ctx.sendConfirmationPending, false, scenario);
    } finally { delete globalThis.window; }
  }
});


function mailDiscardGuard(source) {
  const first = source.indexOf('  private async confirmDiscard(');
  const last = source.indexOf('\n  // ---- views', first);
  assert.ok(first >= 0 && last > first);
  const method = source.slice(first, last).replace('private async ', 'async ')
    .replace('(message:string, ask=true):Promise<boolean>', '(message, ask=true)').trim();
  return new Function('confirmed', 'return ({' + method + '}).confirmDiscard')(confirmed);
}

test('discard confirmation is bound to the original editor and account', async () => {
  const source = readFileSync(new URL('business-mail.component.ts', appDir), 'utf8');
  const blank = () => ({id:null,revision:null,to:'',cc:'',bcc:'',subject:'',body:'',attachments:[]});
  const guard = mailDiscardGuard(source);
  const leave = handler(source, 'async canLeave()', '\n  /** Consent', '():Promise<boolean>', 'canLeave');
  const close = handler(source, '  async closeCompose()', '\n  minimizeCompose', '():Promise<void>', 'closeCompose', {}, blank);
  const discard = handler(source, '  async discardCompose()', '\n  async send()', '():Promise<void>', 'discardCompose', {}, blank);
  for (const action of [close, discard]) {
    for (const scenario of ['accept','cancel','reject','body','replacement','account','busy','destroyed','double']) {
      let answer, reject, prompts=0;
      globalThis.window = { confirm: () => { prompts++; return new Promise((resolve,fail)=>{answer=resolve;reject=fail;}); } };
      const ctx = { busy:false, destroyed:false, discardConfirmationPending:false, sendConfirmationPending:false,
        selectedId:'account-A', composeOpen:true, composer:{...blank(),body:'original'}, drafts:[],
        hasUnsavedComposer:true, t:x=>x, confirmDiscard:guard, canLeave:leave };
      try {
        const pending=action.call(ctx);
        assert.equal(ctx.composeOpen,true);
        if(scenario==='body')ctx.composer.body='new edits';
        if(scenario==='replacement')ctx.composer={...ctx.composer};
        if(scenario==='account')ctx.selectedId='account-B';
        if(scenario==='busy')ctx.busy=true;
        if(scenario==='destroyed')ctx.destroyed=true;
        if(scenario==='double'){await action.call(ctx);assert.equal(prompts,1);}
        if(scenario==='reject')reject(new Error('not allowed'));else answer(scenario!=='cancel');
        await pending;
        assert.equal(ctx.composeOpen,!['accept','double'].includes(scenario),scenario);
        if(scenario==='body')assert.equal(ctx.composer.body,'new edits');
        assert.equal(ctx.discardConfirmationPending,false);
      } finally {delete globalThis.window;}
    }
  }
});

test('deleting a saved draft uses its original account and preserves subsequent editor changes', async () => {
  const source = readFileSync(new URL('business-mail.component.ts', appDir), 'utf8');
  const blank = () => ({id:null,revision:null,to:'',cc:'',bcc:'',subject:'',body:'',attachments:[]});
  const first=source.indexOf('  async removeDraft('), last=source.indexOf('\n  /** Throw the open',first);
  assert.ok(first>=0&&last>first);
  const method=source.slice(first,last).replace('(draft:MailDraft):Promise<void>','(draft)').trim();
  for(const scenario of ['accept','cancel','account','foreign','editing-during-confirm','editing-during-ipc']){
    let answer;
    const calls=[];
    const draft={id:'draft-A',account_id:'account-A',revision:4};
    const ctx={busy:false,destroyed:false,discardConfirmationPending:false,sendConfirmationPending:false,
      selectedId:'account-A',composeOpen:true,composer:{...blank(),id:draft.id,revision:4,body:'original'},
      drafts:[draft],t:x=>x,confirmDiscard:mailDiscardGuard(source),operation:async fn=>fn()};
    const api={mailRemoveDraft:async(...args)=>{calls.push(args);if(scenario==='editing-during-ipc')ctx.composer.body='new edits';}};
    const remove=new Function('api','blankComposer','return ({'+method+'}).removeDraft')(api,blank);
    globalThis.window={confirm:()=>new Promise(resolve=>{answer=resolve;})};
    try{
      if(scenario==='foreign')draft.account_id='account-B';
      const pending=remove.call(ctx,draft);
      if(scenario==='account')ctx.selectedId='account-B';
      if(scenario==='editing-during-confirm')ctx.composer.body='new edits';
      if(scenario!=='foreign')answer(scenario!=='cancel');
      await pending;
      assert.deepEqual(calls,['accept','editing-during-ipc'].includes(scenario)?[['account-A','draft-A',4]]:[],scenario);
      assert.equal(ctx.composeOpen,scenario!=='accept',scenario);
      if(scenario.startsWith('editing'))assert.equal(ctx.composer.body,'new edits');
      if(scenario==='editing-during-ipc'){assert.equal(ctx.composer.id,null);assert.equal(ctx.composer.revision,null);}
    }finally{delete globalThis.window;}
  }
});


async function exercisePurge(source, confirmed, scenario, assert) {
  const first = source.indexOf('  private async run<T>(');
  const last = source.indexOf('\n  // ------------------------------------------------------------ listing', first);
  const runText = source.slice(first, last).trim().replace('private async run<T>(work: () => Promise<T>): Promise<T | undefined>', 'async run(work)');
  const calls = [];
  let answer, reject, prompts = 0, closed = 0, reloaded = 0, success = 0, failed = 0;
  const original = {item:{id:'image-A',state:'TRASHED'}};
  const other = {item:{id:'image-B',state:'TRASHED'}};
  const ctx = {selected:original,busy:false,purgeConfirmationPending:false,tr:x=>x,
    run:new Function('return ({'+runText+'}).run')(),
    closeInspector(){closed++;this.selected=null;},ok(){success++;},fail(){failed++;},
    load:async reset=>{assert.equal(reset,true);reloaded++;},change:{detectChanges(){}}};
  const api={mediaPurge:async id=>{
    calls.push(id);
    if(scenario==='ipc-error')throw new Error('permission denied');
    if(scenario==='changed-during-ipc')ctx.selected=other;
    // Tauri returns void on success.
  }};
  const guard = () => new Promise((resolve,fail)=>{prompts++;answer=resolve;reject=fail;});
  const start=source.indexOf('  async purge()'),end=source.indexOf('\n  // ------------------------------------------------------------ duplicates',start);
  assert.ok(start>=0&&end>start);
  const method=source.slice(start,end).trim().replace('(): Promise<void>','()');
  const purge=new Function('confirmed','api','return ({'+method+'}).purge')(
    message=>confirmed(message,guard),api);
  if(scenario==='active')original.item.state='ACTIVE';
  if(scenario==='initial-busy')ctx.busy=true;
  const pending=purge.call(ctx);
  assert.deepEqual(calls,[]);
  if(scenario==='active'||scenario==='initial-busy'){
    assert.equal(prompts,0);
  }else{
    assert.equal(prompts,1);
    if(scenario==='changed')ctx.selected=other;
    if(scenario==='closed')ctx.selected=null;
    if(scenario==='restored')original.item.state='ACTIVE';
    if(scenario==='busy')ctx.busy=true;
    if(scenario==='double'){await purge.call(ctx);assert.equal(prompts,1);}
    if(scenario==='reject')reject(new Error('dialog unavailable'));else answer(scenario!=='cancel');
  }
  await pending;
  const invoked=['accept','double','ipc-error','changed-during-ipc'].includes(scenario);
  const succeeded=invoked&&scenario!=='ipc-error';
  assert.deepEqual(calls,invoked?['image-A']:[]);
  assert.equal(reloaded,succeeded?1:0);
  assert.equal(success,succeeded?1:0);
  assert.equal(failed,scenario==='ipc-error'?1:0);
  assert.equal(closed,['accept','double'].includes(scenario)?1:0);
  if(scenario==='changed'||scenario==='changed-during-ipc')assert.equal(ctx.selected,other);
  assert.equal(ctx.purgeConfirmationPending,false);
}

for (const scenario of ["accept","cancel","reject","changed","closed","restored","busy","double","active","initial-busy","ipc-error","changed-during-ipc"]) {
  test('media purge: '+scenario, async () => {
    const source=readFileSync(new URL('media-center.component.ts',appDir),'utf8');
    try {
      await exercisePurge(source, (message,guard)=>{
        globalThis.window={confirm:guard};
        return confirmed(message);
      }, scenario, assert);
    } finally {delete globalThis.window;}
  });
}


// Native consent belongs to the exact photo and editor, not a later selection.
async function exercisePhotoConfirmation(source, confirmed, action, scenario, assert) {
  const get = (start, end, signature, replacement, name, api) => {
    const first=source.indexOf(start), last=source.indexOf(end,first);
    assert.ok(first>=0&&last>first,name);
    const method=source.slice(first,last).trim().replace(signature,replacement);
    return new Function('confirmed','api','return ({'+method+'}).'+name)(confirmed,api);
  };
  const collector=action==='deleteCollectorImage';
  const artwork={id:'art-A'}, otherArtwork={id:'art-B'}, draft={id:'collector-A'};
  const image={id:'photo-A',artwork_id:artwork.id,collector_id:draft.id};
  const otherImage={id:'photo-B',artwork_id:otherArtwork.id,collector_id:'collector-B'};
  const calls=[], reads=[];
  let answer, reject, prompts=0;
  const switchEditor=()=>{
    ctx.galleryArtwork=otherArtwork; ctx.galleryImages=[otherImage]; ctx.galleryPreviews={'photo-B':'other'};
    ctx.collectorDraft={id:'collector-B'}; ctx.collectorImages=[otherImage]; ctx.collectorPreviews={'photo-B':'other'};
  };
  const mutate=async (...args)=>{
    calls.push(args);
    if(scenario==='ipc-error')throw new Error('write failed');
    if(scenario==='switched-during-ipc')switchEditor();
  };
  const api={
    promoteArtworkGalleryImage:mutate,deleteArtworkGalleryImage:mutate,deleteCollectorImage:mutate,
    listArtworkGallery:async id=>{reads.push(id);if(scenario==='switched-during-read')switchEditor();return [];},
    listCollectorImages:async id=>{reads.push(id);if(scenario==='switched-during-read')switchEditor();return [];},
    collectorImageData:async()=> 'preview'
  };
  const ctx={
    busy:false,galleryArtwork:artwork,galleryImages:[image],galleryPreviews:{'photo-A':'original'},
    collectorDialog:true,collectorDraft:draft,collectorImages:[image],collectorPreviews:{'photo-A':'original'},
    artworks:[artwork],thumbnails:{'art-A':'cover'},editingGalleryCaptionId:image.id,
    change:{detectChanges(){}},tr:x=>x,error:String,cancelGalleryCaptionEdit(){this.editingGalleryCaptionId=null;},
    loadGalleryPreview:async()=>{},
    run:async function(fn){if(this.busy)return false;this.busy=true;try{await fn();return true;}catch{return false;}finally{this.busy=false;}}
  };
  if(source.includes('private async confirmPhotoChange(')){
    ctx.confirmPhotoChange=get('  private async confirmPhotoChange(', '\n  async makeGalleryPrimary(', 'private async confirmPhotoChange(message: string, isCurrent: () => boolean): Promise<boolean>', 'async confirmPhotoChange(message, isCurrent)', 'confirmPhotoChange',api);
  }
  ctx.loadCollectorImages=get('  private async loadCollectorImages(', '\n  async saveCollector(', 'private async loadCollectorImages(collectorId: string): Promise<void>', 'async loadCollectorImages(collectorId)', 'loadCollectorImages',api);
  const end={makeGalleryPrimary:'\n  async removeGalleryPhoto(',removeGalleryPhoto:'\n  initials(c:',deleteCollectorImage:'\n  async deleteCollector(c:'}[action];
  const method=get('  async '+action+'(',end,'(image: '+(collector?'CollectorImage':'GalleryImage')+'): Promise<void>','(image)',action,api);
  globalThis.window={confirm:()=>{prompts++;return new Promise((resolve,fail)=>{answer=resolve;reject=fail;});}};
  try{
    if(scenario==='foreign'){image.artwork_id='foreign';image.collector_id='foreign';}
    if(scenario==='initial-busy')ctx.busy=true;
    const pending=method.call(ctx,image);
    assert.deepEqual(calls,[]);
    if(['foreign','initial-busy'].includes(scenario)){
      assert.equal(prompts,0,scenario);
    }else{
      assert.equal(prompts,1,scenario);
      assert.equal(ctx.busy,true,'archive lock before native dialog');
      if(scenario==='closed'){ctx.galleryArtwork=null;ctx.collectorDialog=false;}
      if(scenario==='switched')switchEditor();
      if(scenario==='reopened'){ctx.galleryArtwork={...artwork};ctx.collectorDraft={...draft};}
      if(scenario==='replaced'){ctx.galleryImages=[{...image}];ctx.collectorImages=[{...image}];}
      if(scenario==='double'){await method.call(ctx,image);assert.equal(prompts,1);}
      if(scenario==='reject')reject(new Error('native dialog unavailable'));else answer(scenario!=='cancel');
    }
    await pending;
    const invoked=['accept','double','switched-during-ipc','switched-during-read','ipc-error'].includes(scenario);
    assert.deepEqual(calls,invoked?[action==='makeGalleryPrimary'?['art-A','photo-A']:['photo-A']]:[],scenario);
    assert.deepEqual(reads,['accept','double','switched-during-read'].includes(scenario)?[collector?'collector-A':'art-A']:[],scenario);
    if(scenario.startsWith('switched')){
      assert.equal(ctx.galleryArtwork,otherArtwork);
      assert.deepEqual(ctx.galleryImages,[otherImage]);
      assert.deepEqual(ctx.collectorImages,[otherImage]);
      assert.deepEqual(ctx.galleryPreviews,{'photo-B':'other'});
      assert.deepEqual(ctx.collectorPreviews,{'photo-B':'other'});
    }
    assert.equal(ctx.busy,scenario==='initial-busy',scenario);
  }finally{if(answer)answer(false);delete globalThis.window;}
}
for(const action of ['makeGalleryPrimary','removeGalleryPhoto','deleteCollectorImage']){
  for(const scenario of ['accept','cancel','reject','closed','switched','reopened','replaced','foreign','initial-busy','double','switched-during-ipc','switched-during-read','ipc-error']){
    test('photo consent: '+action+' / '+scenario,async()=>{
      await exercisePhotoConfirmation(readFileSync(new URL('app.component.ts',appDir),'utf8'),confirmed,action,scenario,assert);
    });
  }
}
