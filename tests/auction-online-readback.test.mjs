import test from 'node:test';
import assert from 'node:assert/strict';
import {assertSameSnapshot,assertPreviewReadback} from '../src/app/auction-online-readback.ts';
import {decodeNativeDraft,preparePreview,saveAuctionDraft} from '../src/app/auction-preview.ts';
const draft={artworkId:'work-1',title:'Public painting',description:'',slug:'public-painting',
  startsAt:'2030-01-01T12:00:00.000Z',endsAt:'2030-01-02T12:00:00.000Z',
  startingAmount:12345,minimumIncrementAmount:100};
const scope='archive-a', attempt=preparePreview(draft,'tenant-test','key-0123456789012345',0);
const snapshot=(recordJson,revision)=>({artworkId:draft.artworkId,scope,revision,recordJson});
let raw=null;
const storage={getItem:()=>raw,setItem:(_key,value)=>{raw=value;}};
saveAuctionDraft(storage,scope,draft);
const initial=snapshot(raw,'r1');
saveAuctionDraft(storage,scope,draft,attempt);
const pending=snapshot(raw,'r2');
const receipt={tenantId:'tenant-test',auctionId:'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
  slug:'artist--public-painting',idempotencyKey:attempt.idempotencyKey};
const complete=snapshot(JSON.stringify({...JSON.parse(raw),receipt}),'r3');
const decode=s=>decodeNativeDraft(s,draft.artworkId);
const prepare={action:'prepare',revision:'r2'};
const reconcile={action:'reconcile',receipt};
test('preflight rejects changed archive, artwork, revision or raw persisted bytes',()=>{
  assert.doesNotThrow(()=>assertSameSnapshot(initial,{...initial}));
  for(const changed of [{...initial,scope:'other'},{...initial,artworkId:'other'},{...initial,revision:'r2'},
    {...initial,recordJson:'{}'},{...initial,revision:null,recordJson:null}])
    assert.throws(()=>assertSameSnapshot(initial,changed),/auction_draft_conflict/);
});
test('two explicit confirmations require exact persisted pending then exact receipt',()=>{
  assert.doesNotThrow(()=>assertPreviewReadback(initial,pending,decode(initial),decode(pending),attempt.body,prepare));
  assert.doesNotThrow(()=>assertPreviewReadback(pending,complete,decode(pending),decode(complete),attempt.body,reconcile));
  for(const after of [initial,{...pending,scope:'other'},{...pending,revision:'bad'}])
    assert.throws(()=>assertPreviewReadback(initial,after,decode(initial),decode(pending),attempt.body,prepare),/auction_draft_conflict/);
  assert.throws(()=>assertPreviewReadback(initial,pending,decode(initial),decode(pending),'{}',prepare),/auction_draft_conflict/);
  assert.throws(()=>assertPreviewReadback(pending,complete,decode(pending),decode(complete),attempt.body,
    {action:'reconcile',receipt:{...receipt,auctionId:'other'}}),/auction_draft_conflict/);
  assert.throws(()=>assertPreviewReadback(pending,{...pending,revision:'r3'},decode(pending),decode(pending),attempt.body,reconcile),/auction_draft_conflict/);
});
test('reconciliation cannot replace an immutable pending key or draft',()=>{
  const changed={...decode(complete),pending:{...decode(complete).pending,idempotencyKey:'different'}};
  assert.throws(()=>assertPreviewReadback(pending,complete,decode(pending),changed,attempt.body,reconcile),/auction_draft_conflict/);
  assert.throws(()=>assertPreviewReadback(pending,complete,decode(pending),{...decode(complete),draft:{...draft,title:'changed'}},attempt.body,reconcile),/auction_draft_conflict/);
});
