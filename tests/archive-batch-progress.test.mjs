import test from 'node:test';
import assert from 'node:assert/strict';
import {beginBatch,acceptBatchReview,confirmBatch,settleBatch,stopBatch} from '../src/app/archive-batch-progress.ts';
const a='a'.repeat(64), b='b'.repeat(64), m='c'.repeat(64);
const plan=()=>({manifest_digest:m,packages:[{sha256:a,artworks:2},{sha256:b,artworks:3}]});
const committing=s=>confirmBatch(acceptBatchReview(s,s.next,s.packages[s.next].sha256));
test('each package requires its own review and confirmation',()=>{
  const start=beginBatch(plan());
  assert.throws(()=>confirmBatch(start),/batch_phase/);
  assert.throws(()=>acceptBatchReview(start,1,b),/batch_changed/);
  assert.throws(()=>acceptBatchReview(start,0,b),/batch_changed/);
  const first=committing(start);
  assert.throws(()=>confirmBatch(first),/batch_phase/);
  const partial=settleBatch(first,{kind:'committed',count:2});
  assert.deepEqual([partial.phase,partial.next,partial.imported],['review',1,2]);
  const done=settleBatch(committing(partial),{kind:'committed',count:3});
  assert.deepEqual([done.phase,done.next,done.imported],['complete',2,5]);
});
test('rejection and ambiguous acknowledgement retain only confirmed progress',()=>{
  const partial=settleBatch(committing(beginBatch(plan())),{kind:'committed',count:2});
  for(const [result,phase] of [[{kind:'rejected'},'stopped'],[{kind:'unknown'},'unknown'],[{kind:'committed',count:1},'unknown']]) {
    const end=settleBatch(committing(partial),result);
    assert.deepEqual([end.phase,end.next,end.imported],[phase,1,2]);
    assert.throws(()=>acceptBatchReview(end,1,b),/batch_changed/);
    assert.throws(()=>confirmBatch(end),/batch_phase/);
  }
});
test('cancel never claims a commit and cannot cancel an in-flight write',()=>{
  const start=beginBatch(plan());
  assert.equal(stopBatch(start).phase,'stopped');
  assert.equal(stopBatch(acceptBatchReview(start,0,a)).imported,0);
  assert.throws(()=>stopBatch(committing(start)),/batch_phase/);
});
test('manifest shape and counts are bounded and caller mutation cannot change plan',()=>{
  assert.throws(()=>beginBatch({manifest_digest:'bad',packages:[]}),/batch_manifest/);
  assert.throws(()=>beginBatch({manifest_digest:m,packages:[{sha256:a,artworks:1000},{sha256:b,artworks:1}]}),/batch_manifest/);
  const raw=plan(), start=beginBatch(raw); raw.packages[0].sha256=b;
  assert.equal(start.packages[0].sha256,a);
});
