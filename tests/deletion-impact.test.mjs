import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { confirmed } from '../src/app/confirm-dialog.ts';
import { artworkDeletionMessage, bulkDeletionMessage, collectorDeletionMessage, hasDependents, mergeImpacts } from '../src/app/deletion-impact.ts';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const f = { t: key => key, money: cents => `€${(cents / 100).toFixed(2)}`, date: value => value };
const empty = { certificates: [], sales: [], receipts: 0, attachments: 0 };
const sale = { artwork_title: 'Mare', collector_name: 'Anna Rossi', amount_cents: 50000, sold_at: '2026-11-01' };

test('an artwork without dependents only asks the plain question', () => {
  assert.equal(hasDependents(empty), false);
  const text = artworkDeletionMessage('Mare', empty, f);
  assert.match(text, /Eliminare definitivamente l’opera "Mare"\?/);
  assert.doesNotMatch(text, /Certificati|Vendita/);
});

test('an artwork with a certificate and a sale lists what goes with it', () => {
  const impact = { certificates: ['RS-2026-000001', 'RS-2026-000002'], sales: [sale], receipts: 2, attachments: 1 };
  assert.equal(hasDependents(impact), true);
  const text = artworkDeletionMessage('Mare', impact, f);
  assert.match(text, /Certificati: RS-2026-000001, RS-2026-000002/);
  assert.match(text, /Vendita:/);
  assert.match(text, /Anna Rossi — €500.00 \(2026-11-01\)/);
  assert.match(text, /2 incassi registrati/);
  assert.match(text, /1 documenti allegati agli incassi/);
  assert.match(text, /non è reversibile/);
});

test('a certificate alone is enough to need the cascade', () => {
  const impact = { ...empty, certificates: ['RS-1'] };
  assert.equal(hasDependents(impact), true);
  assert.doesNotMatch(artworkDeletionMessage('Mare', impact, f), /Vendita:/);
});

test('a collector without sales is deleted with the plain question', () => {
  assert.equal(collectorDeletionMessage('Anna', empty, f), 'Eliminare il contatto "Anna"?');
});

test('a collector with sales is warned that the sales are deleted too', () => {
  const text = collectorDeletionMessage('Anna Rossi', { ...empty, sales: [sale], receipts: 1 }, f);
  assert.match(text, /Verranno eliminate anche le vendite annesse:/);
  assert.match(text, /• Mare — €500.00/);
  assert.match(text, /1 incassi registrati/);
  assert.match(text, /i certificati già emessi restano/);
});

test('the interface asks for the impact first and only cascades what was shown', () => {
  const app = get('src/app/app.component.ts');
  assert.match(app, /api\.artworkDeletionImpact\(a\.id\)[\s\S]*await confirmed\(artworkDeletionMessage[\s\S]*api\.deleteArtwork\(a\.id, hasDependents\(impact\)\)/);
  assert.match(app, /api\.collectorDeletionImpact\(c\.id\)[\s\S]*await confirmed\(collectorDeletionMessage[\s\S]*api\.deleteCollector\(c\.id, impact\.sales\.length > 0\)/);
  assert.match(app, /api\.deleteCertificate\(c\.id\)/);
  assert.match(get('src/app/app.component.html'), /\(click\)="deleteCertificate\(c\)"/);
});

test('every new text has the four translations', () => {
  const i18n = get('src/app/i18n.ts');
  for (const key of ['Verranno eliminate anche le vendite annesse:', 'Eliminare il certificato', 'Certificato eliminato', 'incassi registrati']) {
    assert.ok(i18n.split(`"${key}"`).length - 1 >= 4 || i18n.split(`'${key}'`).length - 1 >= 4, key);
  }
});

test('impacts of several records are added together', () => {
  const merged = mergeImpacts([
    { certificates: ['RS-1'], sales: [sale], receipts: 2, attachments: 1 },
    { certificates: ['RS-2'], sales: [], receipts: 0, attachments: 0 },
  ]);
  assert.deepEqual(merged.certificates, ['RS-1', 'RS-2']);
  assert.equal(merged.sales.length, 1);
  assert.equal(merged.receipts, 2);
  assert.equal(merged.attachments, 1);
});

test('deleting a selection asks once, with the count, the names and what goes along', () => {
  const impact = { certificates: ['RS-1'], sales: [sale], receipts: 1, attachments: 0 };
  const text = bulkDeletionMessage('artworks', ['Mare', 'Monte'], impact, f);
  assert.match(text, /^Eliminare definitivamente 2 opere\?/);
  assert.ok(text.includes('• Mare\n• Monte'));
  assert.match(text, /Verranno eliminati anche i dati collegati:/);
  assert.match(text, /1 Certificati/);
  assert.match(text, /Mare → Anna Rossi — €500.00/);
  assert.match(text, /non è reversibile/);
  const plain = bulkDeletionMessage('certificates', ['RS-1'], empty, f);
  assert.doesNotMatch(plain, /Verranno/);
  assert.match(plain, /1 certificati\?/);
});

test('a long selection is shortened and a collector selection warns about the sales', () => {
  const names = Array.from({ length: 12 }, (_, i) => `Nome ${i + 1}`);
  const text = bulkDeletionMessage('collectors', names, { ...empty, sales: [sale] }, f);
  assert.match(text, /12 contatti\?/);
  assert.match(text, /… e altri 4/);
  assert.doesNotMatch(text, /Nome 9/);
  assert.match(text, /Verranno eliminate anche le vendite annesse:/);
  assert.match(text, /i certificati già emessi restano/);
});

test('the lists have tick boxes, a select-all box and one bulk delete that only counts listed records', () => {
  const html = get('src/app/app.component.html');
  for (const set of ['selectedArtworkIds', 'selectedCollectorIds', 'selectedCertificateIds']) {
    assert.ok(html.includes(`toggleId(${set},`), set);
    assert.ok(html.includes(`.clear()`), 'deselect');
  }
  for (const method of ['deleteSelectedArtworks', 'deleteSelectedCollectors', 'deleteSelectedCertificates']) assert.ok(html.includes(`(click)="${method}()"`), method);
  const app = get('src/app/app.component.ts');
  assert.match(app, /pickedArtworks[\s\S]*this\.visibleArtworks\.filter/);
  assert.match(app, /pickedCollectors[\s\S]*this\.visibleCollectors\.filter/);
  assert.match(app, /await confirmed\(bulkDeletionMessage\('artworks'/);
  assert.match(app, /await confirmed\(bulkDeletionMessage\('collectors'/);
  assert.match(app, /await confirmed\(bulkDeletionMessage\('certificates'/);
});


// Exercise the actual four cascade handlers and their shared lock, with synthetic IPC.
async function cascadeScenario(source, action, scenario, askNative) {
  const extract = (start, end, signature, replacement, name, ...deps) => {
    const first = source.indexOf(start), last = source.indexOf(end, first);
    assert.ok(first >= 0 && last > first, name);
    const raw = source.slice(first, last).trim().replace(signature, replacement);
    return new Function('api', 'confirmed', 'artworkDeletionMessage', 'collectorDeletionMessage',
      'bulkDeletionMessage', 'hasDependents', 'mergeImpacts', 'return ({' + raw + '}).' + name)(...deps);
  };
  const bulk = action.startsWith('deleteSelected'), artwork = action.endsWith('Artwork') || action.endsWith('Artworks');
  const a = { id: 'A', title: 'A', name: 'A' }, b = { id: 'B', title: 'B', name: 'B' };
  let answer, reject, prompts = 0, reads = 0, refreshes = 0, changed = false, releaseRead;
  const calls = [];
  const ctx = { busy: scenario === 'busy', artworks: [a,b], collectors: [a,b], pickedArtworks: [a,b], pickedCollectors: [a,b],
    selectedArtworkIds: new Set(['A','B']), selectedCollectorIds: new Set(['A','B']), thumbnails: {A:'a',B:'b'},
    change: { detectChanges() {} }, error: String, tr: x=>x, formatters: ()=>f, refresh: async()=>{refreshes++;} };
  const read = async id => {
    reads++;
    if (scenario === 'read-error') throw Error('read error');
    if (scenario === 'delayed-read' && reads === 1) await new Promise(resolve=>{releaseRead=resolve;});
    if (scenario === 'recheck-error' && changed) throw Error('recheck error');
    return { certificates: artwork ? ['RS-'+id] : [], sales: artwork ? [] : [sale], receipts: changed && scenario === 'impact' ? 1 : 0, attachments: 0 };
  };
  const remove = async (id, cascade) => {
    calls.push([id,cascade]);
    if (scenario === 'write-error' && (id === 'B' || !bulk)) throw Error('write error');
  };
  const api = { artworkDeletionImpact:read,collectorDeletionImpact:read,deleteArtwork:remove,deleteCollector:remove };
  const dialog = () => { prompts++; return new Promise((resolve,fail)=>{answer=resolve;reject=fail;}); };
  const deps = [api, message=>askNative(message,dialog), artworkDeletionMessage,collectorDeletionMessage,bulkDeletionMessage,hasDependents,mergeImpacts];
  if (source.includes('private async confirmImpactDeletion(')) {
    ctx.confirmImpactDeletion = extract('  private async confirmImpactDeletion(', '\n  async deleteArtwork(', /private async confirmImpactDeletion\([\s\S]*?\): Promise<boolean>/, 'async confirmImpactDeletion(isCurrent, read, ask, remove, success)', 'confirmImpactDeletion', ...deps);
  }
  const end = {deleteArtwork:'\n  /** One confirmation',deleteSelectedArtworks:'\n  async deleteSelectedCollectors(',deleteSelectedCollectors:'\n  async deleteSelectedCertificates(',deleteCollector:'\n  recalculateDefaultSaleTax()'}[action];
  const run = extract('  async '+action+'(',end, /\([^)]*\): Promise<void>/, bulk?'()':artwork?'(a)':'(c)',action,...deps);
  const pending = run.call(ctx,a);
  assert.deepEqual(calls,[]);
  assert.equal(ctx.busy,true,'lock before reading impact');
  if (scenario === 'busy' || scenario === 'read-error') {
    await pending;
    assert.equal(prompts,0);
  } else {
    if (scenario === 'delayed-read') {
      await run.call(ctx,a);
      assert.equal(prompts,0);
      releaseRead();
    }
    for (let i=0;i<20&&!answer;i++) await Promise.resolve();
    assert.equal(prompts,1);
    if (scenario === 'double') { await run.call(ctx,a); assert.equal(prompts,1); }
    if (scenario === 'selection') {
      if (bulk) {ctx.pickedArtworks=[b];ctx.pickedCollectors=[b];}
      else {ctx.artworks=[{...a},b];ctx.collectors=[{...a},b];}
    }
    changed=true;
    if (scenario === 'reject') reject(Error('dialog unavailable')); else answer(scenario !== 'cancel');
    await pending;
  }
  const invoked = ['accept','double','delayed-read','write-error'].includes(scenario);
  assert.deepEqual(calls,invoked?(bulk?[['A',true],['B',true]]:[['A',true]]):[]);
  assert.equal(ctx.busy,scenario==='busy');
  assert.equal(refreshes,invoked?1:0);
  if (scenario.endsWith('error')) assert.ok(ctx.errorMessage);
  if (invoked && scenario!=='write-error') {
    assert.ok(ctx.notice);
    if (bulk) assert.equal((artwork?ctx.selectedArtworkIds:ctx.selectedCollectorIds).size,0);
  }
}
for (const action of ['deleteArtwork','deleteCollector','deleteSelectedArtworks','deleteSelectedCollectors']) {
  for (const scenario of ['accept','cancel','reject','selection','impact','busy','double','delayed-read','read-error','recheck-error','write-error']) {
    test('cascade consent: '+action+' / '+scenario, async () => {
      try { await cascadeScenario(get('src/app/app.component.ts'),action,scenario,(message,dialog)=>{
        globalThis.window={confirm:dialog};
        return confirmed(message);
      }); } finally { delete globalThis.window; }
    });
  }
}
