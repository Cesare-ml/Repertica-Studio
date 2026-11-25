import test from 'node:test';
import assert from 'node:assert/strict';
import {SettlementReader} from '../src/app/auction-settlement-summary.ts';
const selected = {artworkId:'local-work',scope:'archive',revision:'r1'};
const result = {sequence:3,status:'review_required',amount:25000,currency:'EUR',
  fulfilledAt:'2026-11-25T04:00:00.000Z',acknowledgedAt:null};

test('read sends only selected local artwork and archive; no authority or remote IDs',async()=>{
  const calls=[];
  const reader=new SettlementReader(async request=>{calls.push(request);return result;});
  assert.equal(reader.current(selected).kind,'idle');
  await reader.load({...selected,tenantId:'other',token:'private'});
  assert.deepEqual(calls,[{action:'readSettlement',artworkId:'local-work',scope:'archive'}]);
  assert.deepEqual(reader.current(selected),{kind:'ready',summary:result});
  for(const changed of [{...selected,scope:'other'},{...selected,artworkId:'other'},{...selected,revision:'r2'},null])
    assert.equal(reader.current(changed).kind,'idle');
});

test('no binding or denial is distinct from verified empty result; no auto retry',async()=>{
  let calls=0;
  const reader=new SettlementReader(async()=>{calls++; if(calls===1) return null; throw Error('settlement_binding_unavailable');});
  await reader.load(selected);
  assert.equal(reader.current(selected).kind,'empty');
  await reader.load(selected);
  assert.equal(reader.current(selected).kind,'unavailable');
  assert.equal(calls,2);
});

test('clear and subsequent read invalidate outstanding responses',async()=>{
  let finish;
  let count=0;
  const reader=new SettlementReader(async()=>++count===1?new Promise(resolve=>{finish=resolve;}):null);
  const pending=reader.load(selected);
  assert.equal(reader.busy,true);
  reader.clear();
  await reader.load({...selected,scope:'other'});
  finish(result);
  await pending;
  assert.equal(reader.current(selected).kind,'idle');
  assert.equal(reader.current({...selected,scope:'other'}).kind,'empty');
  assert.equal(reader.busy,false);
});

test('malformed summary or private fields clear a previously visible result',async()=>{
  let calls=0;
  const reader=new SettlementReader(async()=>++calls===1?result:{...result,ownerSubject:'private'});
  await reader.load(selected);
  assert.equal(reader.current(selected).kind,'ready');
  await reader.load(selected);
  assert.equal(reader.current(selected).kind,'unavailable');
});
