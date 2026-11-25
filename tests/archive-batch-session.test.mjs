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
