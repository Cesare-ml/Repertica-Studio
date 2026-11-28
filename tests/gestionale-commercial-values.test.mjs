import test from 'node:test';
import assert from 'node:assert/strict';
import {money,date,unpaidInstallments} from '../scripts/gestionale-commercial-values.mjs';
const plan=[{position:1,due_date:'2024-02-29',amount:'40.29',notes:' first '},
  {position:2,due_date:'2024-03-31',amount:'59.71',notes:null}];
const convert=(rows=plan,receipts=[],amount=10000)=>unpaidInstallments(rows,receipts,amount);
test('exact decimal and canonical date conversion',()=>{
  assert.equal(money('0.29',true),29);assert.equal(money('1000000000.00'),100000000000);
  for(const v of ['0','-1','1.001','1000000000.01','1e2',1]) assert.throws(()=>money(v,true));
  assert.equal(date('2024-02-29'),'2024-02-29');
  for(const d of ['2023-02-29','2024-04-31','2024-1-01']) assert.throws(()=>date(d));
});
test('unpaid plans preserve exact dates, positions, amounts and notes',()=>{
  assert.deepEqual(convert(),[{position:1,due_date:'2024-02-29',amount_cents:4029,notes:'first'},
    {position:2,due_date:'2024-03-31',amount_cents:5971,notes:null}]);
  assert.deepEqual(convert([],[{amount:'1'}]),[]);
});
test('refuses ambiguity, receipts with a plan, and plans that do not total the sale',()=>{
  assert.throws(()=>convert(plan,[{amount:'1'}]));assert.throws(()=>convert(plan,[],9999));assert.throws(()=>convert(plan,[],10001));
  for(const rows of [null,[plan[0]],[plan[1],plan[0]],[plan[0],{...plan[1],position:3}],
    [plan[0],{...plan[1],due_date:'2024-01-01'}],[plan[0],{...plan[1],amount:'0'}],
    [plan[0],{...plan[1],notes:'x'.repeat(501)}],[plan[0],{...plan[1],notes:'x\0y'}],
    [plan[0],{...plan[1],notes:7}],[plan[0],{...plan[1],unmapped:true}],
    Array.from({length:61},(_,i)=>({...plan[0],position:i+1}))]) assert.throws(()=>convert(rows));
});
