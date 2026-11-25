import test from 'node:test';
import assert from 'node:assert/strict';
import {SettlementReader} from '../src/app/auction-settlement-summary.ts';
const selected = {artworkId:'local-work',scope:'archive',revision:'r1'};
const confirmationId = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
const result = {sequence:3,status:'review_required',amount:25000,currency:'EUR',
  fulfilledAt:'2026-11-25T04:00:00.000Z',acknowledgedAt:null,ackPending:1,ackScanned:0,ackConfirmed:0};

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

test('poll and one history page are explicit and never send renderer cursors',async()=>{
  const calls=[];
  const reader=new SettlementReader(async request=>{calls.push(request);return result;});
  await reader.load({...selected,afterSequence:99},'pollSettlement');
  await reader.load({...selected,afterSequence:99},'historySettlement');
  assert.deepEqual(calls,[
    {action:'pollSettlement',artworkId:'local-work',scope:'archive'},
    {action:'historySettlement',artworkId:'local-work',scope:'archive'},
  ]);
});
test('ACK requires the reviewed current result and never sends private or remote identifiers',async()=>{
  const calls=[];
  const acknowledged={...result,acknowledgedAt:'2026-11-25T04:02:00.000Z',ackPending:0,ackConfirmed:1};
  const reader=new SettlementReader(async request=>{calls.push(request);return request.action==='ackSettlement'?acknowledged:request.action==='reviewSettlementAck'?{confirmationId,summary:result}:result;});
  await reader.acknowledge(selected);
  await reader.load(selected);
  await reader.acknowledge({...selected,revision:'r2'});
  await reader.acknowledge({...selected,token:'secret',resultId:'remote'});
  await reader.acknowledge(selected);
  assert.deepEqual(calls,[
    {action:'readSettlement',artworkId:'local-work',scope:'archive'},
    {action:'reviewSettlementAck',artworkId:'local-work',scope:'archive',expectedSequence:3},
    {action:'ackSettlement',artworkId:'local-work',scope:'archive',expectedSequence:3,confirmationId},
  ]);
  assert.deepEqual(reader.current(selected),{kind:'ready',summary:acknowledged});
});
test('uncertain or mismatched ACK is not success and is never retried automatically',async()=>{
  for(const response of [null,{...result,sequence:4,acknowledgedAt:'2026-11-25T04:02:00.000Z'},result]){
    let calls=0;
    const reader=new SettlementReader(async()=>++calls===1?result:calls===2?{confirmationId,summary:result}:response);
    await reader.load(selected);
    await reader.acknowledge(selected);
    assert.equal(reader.current(selected).kind,'unavailable');
    await reader.acknowledge(selected);
    assert.equal(calls,3);
  }
});
test('clear invalidates in-flight ACK and prevents duplicate request while busy',async()=>{
  let finish;let calls=0;
  const reader=new SettlementReader(async()=>++calls===1?result:new Promise(resolve=>{finish=resolve;}));
  await reader.load(selected);
  const pending=reader.acknowledge(selected);
  await reader.acknowledge(selected);
  reader.clear();
  finish({confirmationId,summary:result});
  await pending;
  assert.equal(calls,2);
  assert.equal(reader.current(selected).kind,'idle');
});

test('native review mismatch or invalid capability never sends ACK',async()=>{
  for(const review of [{confirmationId,summary:{...result,amount:1}},{confirmationId,summary:{...result,ackPending:0}},{confirmationId:'bad',summary:result}]) {
    const calls=[];
    const reader=new SettlementReader(async request=>{calls.push(request);return calls.length===1?result:review;});
    await reader.load(selected);
    await reader.acknowledge(selected);
    assert.equal(reader.current(selected).kind,'unavailable');
    assert.equal(calls.length,2);
  }
});
