import test from 'node:test';
import assert from 'node:assert/strict';
import {parseSettlementSummary, settlementPresentation} from '../src/app/auction-settlement-summary.ts';
const receipt = {sequence:2,status:'review_required',amount:25000,currency:'EUR',fulfilledAt:'2026-11-25T04:00:00.000Z',acknowledgedAt:'2026-11-25T04:02:00.000Z'};
test('native summary retains independent review and receipt states in five languages',()=>{
  for (const lang of ['it','en','es','fr','de']) {
    const view=settlementPresentation(receipt,lang);
    assert.equal(view.sequence,2);
    assert.match(view.amount,/250/);
    assert.notEqual(view.status,view.receipt);
    assert.equal(view.acknowledgedAt,receipt.acknowledgedAt);
    assert.notEqual(view.receipt,settlementPresentation({...receipt,acknowledgedAt:null},lang).receipt);
    assert.notEqual(view.status,settlementPresentation({...receipt,status:'fulfilled'},lang).status);
    assert.equal(view.notice,settlementPresentation({...receipt,status:'fulfilled',acknowledgedAt:null},lang).notice);
  }
});
test('no receipt is not a payment or an error',()=>{
  assert.equal(settlementPresentation(null,'it'),null);
  assert.equal(settlementPresentation({...receipt,acknowledgedAt:null},'en').receipt,'Receipt not yet acknowledged');
  assert.equal(settlementPresentation(receipt,'en').notice,'Auction result: this is not a recorded sale or payment.');
});
test('strict allowlist rejects private fields, invalid states, unknown currency, absent ACK and invalid numbers',()=>{
  const {acknowledgedAt,...missing}=receipt;
  for (const value of [missing, {...receipt,owner:'repertica:1'}, {...receipt,amount:0}, {...receipt,amount:1.5}, {...receipt,amount:2**53}, {...receipt,sequence:-1}, {...receipt,status:'paid'}, {...receipt,currency:'USD'}, {...receipt,fulfilledAt:'2026-11-25'}, {...receipt,acknowledgedAt:'invalid'}, [], undefined]) {
    assert.throws(()=>parseSettlementSummary(value),/settlement_invalid_summary/);
  }
});
test('exact integer cents at the upper safe-integer boundary',()=>{
  const view=settlementPresentation({...receipt,amount:Number.MAX_SAFE_INTEGER},'en');
  assert.match(view.amount,/90,071,992,547,409\.91/);
  assert.ok(Object.isFrozen(parseSettlementSummary(receipt)));
});
