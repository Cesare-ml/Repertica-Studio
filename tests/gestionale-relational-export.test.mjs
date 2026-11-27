import test from 'node:test';
import assert from 'node:assert/strict';
import { checkedRelationalPackage } from '../scripts/export-gestionale-archive-relations.mjs';
const artwork={codice_archivio:'TEST-1',titolo:'Synthetic',anno_realizzazione:2024,tecnica:null,supporto:null,
  larghezza_cm:null,altezza_cm:null,profondita_cm:null,descrizione:null,stato:'VENDUTA',
  firma_presente:true,cornice_presente:false,vernice_finale:'OPACA',
  coefficiente_artista_override:null,prezzo_personalizzato:null,id_vendita:null};
const collector={collector_key:'7',first_name:'Synthetic',last_name:'Collector',email:null,phone:null,address:null,notes:null};
const sale={amount:'100.25',tax:'10.00',shipping:'5.25',sold_at:'2024-02-29',notes:null,installments:[],
  receipts:[{amount:'0.29',received_at:'2024-03-01',method:'Bonifico',notes:null}]};
const shares=[{artwork_code:'TEST-1',legacy_shares_bp:[5000,3000,2000]}];
const pack=(rows,s=shares)=>JSON.parse(checkedRelationalPackage(Buffer.from(JSON.stringify(rows)),s));
test('keeps sale status, collector relation, explicit shares and exact cents',()=>{
  const p=pack([{artwork,collector,sale}]);
  assert.equal(p.artworks[0].status,'sold');
  assert.equal(p.sales[0].collector_key,p.collectors[0].key);
  assert.equal(p.sales[0].amount_cents,10025);
  assert.equal(p.sales[0].receipts[0].amount_cents,29);
  assert.equal(p.sales[0].receipts[0].method,'bank_transfer');
  assert.deepEqual(p.sales[0].legacy_shares_bp,[5000,3000,2000]);
});
test('rejects unsupported, ambiguous and missing historical financial data',()=>{
  const row={artwork,collector,sale};
  for(const s of [[],[...shares,...shares],[{...shares[0],legacy_shares_bp:[5000,3000,1999]}],[...shares,{artwork_code:'other',legacy_shares_bp:[5000,3000,2000]}]]) assert.throws(()=>pack([row],s));
  for(const bad of [{...sale,amount:'10.001'},{...sale,sold_at:'2023-02-29'},{...sale,receipts:[{...sale.receipts[0],method:'unknown'}]},{...sale,receipts:[{...sale.receipts[0],amount:'101'}]},{...sale,installments:[{}]}]) assert.throws(()=>pack([{...row,sale:bad}]));
  assert.throws(()=>pack([row,{...row,artwork:{...artwork,codice_archivio:'TEST-2'}}]));
});
test('unsold work stays unsold and accepts no invented finance data',()=>{
  const p=pack([{artwork:{...artwork,stato:'RISERVATA'},collector:null,sale:null}],[]);
  assert.equal(p.artworks[0].status,'reserved');assert.deepEqual(p.sales,[]);assert.deepEqual(p.collectors,[]);
  assert.throws(()=>pack([{artwork:{...artwork,stato:'RISERVATA'},collector,sale}],[]));
});

test('export includes unpaid plans, and does not reinterpret historical receipts',()=>{
  const installments=[{position:1,due_date:'2024-03-31',amount:'40.25',notes:'initial'},
    {position:2,due_date:'2024-04-30',amount:'60.00',notes:null}];
  const p=pack([{artwork,collector,sale:{...sale,receipts:[],installments}}]);
  assert.deepEqual(p.sales[0].installments,[{position:1,due_date:'2024-03-31',amount_cents:4025,notes:'initial'},
    {position:2,due_date:'2024-04-30',amount_cents:6000,notes:null}]);
  assert.deepEqual(p.sales[0].receipts,[]);
  assert.throws(()=>pack([{artwork,collector,sale:{...sale,installments}}]));
  assert.throws(()=>pack([{artwork,collector,sale:{...sale,receipts:[],installments:[...installments].reverse()}}]));
});


test('collector profile preserves source fields and only exact minute birth times',()=>{
  const profile={birth_date:'1980-02-29',birth_time:'07:45:00',birth_place:'Example city',instagram:'@example',facebook:'example',tiktok:'@example',website:'https://example.invalid',linkedin:'profile'};
  const p=pack([{artwork,collector:{...collector,...profile},sale}]);
  for(const [key,value] of Object.entries(profile)) assert.equal(p.collectors[0][key],key==='birth_time'?'07:45':value);
  for(const bad of [{birth_time:'07:45:01'},{birth_time:'24:00:00'},{birth_date:'1981-02-29'},{website:'x'.repeat(501)},{instagram:7},{unknown:'x'}])
    assert.throws(()=>pack([{artwork,collector:{...collector,...profile,...bad},sale}]));
});

test('export validates optional sale and receipt notes before publishing a package',()=>{
  for(const [field,limit] of [['sale',15000],['receipt',2000]]) {
    const make=notes=>field==='sale'?{...sale,notes}:{...sale,receipts:[{...sale.receipts[0],notes}]};
    for(const notes of [null,'','  valid  ','🎨'.repeat(limit),' '.repeat(limit+1)]) {
      const p=pack([{artwork,collector,sale:make(notes)}]);
      assert.equal(field==='sale'?p.sales[0].notes:p.sales[0].receipts[0].notes,notes);
    }
    for(const notes of [false,42,[],{},'a\0b','🎨'.repeat(limit+1),'x'+' '.repeat(limit)]) {
      assert.throws(()=>pack([{artwork,collector,sale:make(notes)}]),/Invalid commercial text/);
    }
  }
});

test('export rejects invalid collector details before producing an import package',()=>{
  for(const bad of [{first_name:'a\nb'},{first_name:'🎨'.repeat(241)},{last_name:7},
    {email:'not-an-email'},{email:'a @b'},{phone:'🎨'.repeat(121)},{address:'a\0b'},
    {notes:'x'.repeat(15001)},{instagram:'x'+' '.repeat(300)}]) {
    assert.throws(()=>pack([{artwork,collector:{...collector,...bad},sale}]));
  }
  const valid={phone:'🎨'.repeat(120),notes:' '.repeat(15001),email:'  a@example.invalid  '};
  const p=pack([{artwork,collector:{...collector,...valid},sale}]);
  for(const [k,v] of Object.entries(valid))assert.equal(p.collectors[0][k],v);
});
