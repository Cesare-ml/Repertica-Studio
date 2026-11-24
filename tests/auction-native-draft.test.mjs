import { test } from 'node:test';
import assert from 'node:assert/strict';
import { newAuctionDraft, preparePreview, decodeNativeDraft, saveNativeAuctionDraft } from '../src/app/auction-preview.ts';

const draft = { ...newAuctionDraft({id:'a',title:'Art',status:'available'}),
  slug:'art',startsAt:'2027-01-02T10:00:00.000Z',endsAt:'2027-01-03T10:00:00.000Z',
  startingAmount:1000,minimumIncrementAmount:100 };
const pending = preparePreview(draft,'tenant-test','preview-0123456789',Date.parse(draft.startsAt));

function port() {
  let current = {artworkId:'a',scope:'archive-one',revision:null,recordJson:null};
  let counter = 0;
  return {
    read: async () => ({...current}),
    write: async input => {
      if (input.scope !== current.scope || input.expectedRevision !== current.revision) throw new Error('auction_draft_conflict');
      current = {artworkId:input.artworkId,scope:input.scope,revision:String(++counter),recordJson:input.recordJson};
      return {...current};
    }
  };
}

test('native save uses loaded revision and preserves exact pending payload', async () => {
  const native = port(), loaded = await native.read('a');
  assert.equal(decodeNativeDraft(loaded,'a'),null);
  const saved = await saveNativeAuctionDraft(native,loaded,draft,pending);
  await assert.rejects(saveNativeAuctionDraft(native,loaded,draft),/auction_draft_conflict/);
  const edited = await saveNativeAuctionDraft(native,saved,{...draft,title:'Edited'});
  assert.deepEqual(decodeNativeDraft(edited,'a').pending,pending);
  assert.equal(decodeNativeDraft(edited,'a').draft.title,'Edited');
});

test('native persistence failure never returns a saved snapshot', async () => {
  const loaded = await port().read('a');
  await assert.rejects(saveNativeAuctionDraft({write:async()=>{throw new Error('disk full')}},loaded,draft,pending),/disk full/);
  await assert.rejects(saveNativeAuctionDraft({write:async()=>loaded},loaded,draft,pending),/auction_draft_write_failed/);
});


test('an empty snapshot is bound to the selected artwork too', async () => {
  const native = port(), loaded = await native.read('a');
  assert.throws(() => decodeNativeDraft(loaded,'b'),/invalid_stored_draft/);
  await assert.rejects(saveNativeAuctionDraft(native,loaded,{...draft,artworkId:'b'}),/invalid_stored_draft/);
  assert.equal((await native.read('a')).revision,null);
});

test('native receipt is read-only in editor saves, bound to the pending tenant and key', async () => {
  const native=port(), loaded=await native.read('a');
  const saved=await saveNativeAuctionDraft(native,loaded,draft,pending);
  const receipt={tenantId:'tenant-test',idempotencyKey:pending.idempotencyKey,
    auctionId:'4c2a24d0-bf50-410a-b0f2-1c0b3c96e9a0',slug:'artist--art'};
  const record=JSON.parse(saved.recordJson);
  const confirmed=await native.write({artworkId:'a',scope:saved.scope,expectedRevision:saved.revision,
    recordJson:JSON.stringify({...record,receipt})});
  assert.deepEqual(decodeNativeDraft(confirmed,'a').receipt,receipt);
  const edited=await saveNativeAuctionDraft(native,confirmed,{...draft,title:'Local edit'});
  assert.deepEqual(decodeNativeDraft(edited,'a').receipt,receipt);
  assert.deepEqual(decodeNativeDraft(edited,'a').pending,pending);
  for (const bad of [
    {...receipt,tenantId:'another-tenant'}, {...receipt,idempotencyKey:'another-preview-key'},
    {...receipt,auctionId:'not-a-uuid'}, {...receipt,slug:'artist--other'},
    {...receipt,accessToken:'never persist'}
  ]) {
    assert.throws(()=>decodeNativeDraft({...confirmed,recordJson:JSON.stringify({...record,receipt:bad})},'a'),/invalid_stored_draft/);
  }
  assert.throws(()=>decodeNativeDraft({...confirmed,recordJson:JSON.stringify({...record,pending:null,receipt})},'a'),/invalid_stored_draft/);
});
