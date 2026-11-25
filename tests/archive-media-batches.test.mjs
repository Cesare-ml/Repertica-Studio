import test from 'node:test';
import assert from 'node:assert/strict';
import { groupArchiveMedia, publishArchiveMediaBatches } from '../scripts/archive-media-batches.mjs';
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';

const artwork=(code,status='available')=>({code,status,title:code});
const sale=(artwork_code,collector_key)=>({artwork_code,collector_key,receipts:[],installments:[]});
const pkg={format:'repertica-studio-archive-relations-v2',artworks:[artwork('A','sold'),artwork('B'),artwork('C','sold')],
  collectors:[{key:'1',first_name:'Ada',last_name:'Example'}],sales:[sale('A','1'),sale('C','1')]};
const rows=pkg.artworks.map(a=>({code:a.code,photos:[]}));

test('keeps nonadjacent purchases and one collector in one group without dropping records',()=>{
  const groups=groupArchiveMedia(pkg,rows);
  assert.deepEqual(groups.map(g=>g.pkg.artworks.map(a=>a.code)),[['A','C'],['B']]);
  assert.deepEqual(groups[0].pkg.sales,pkg.sales);
  assert.deepEqual(groups[0].pkg.collectors,pkg.collectors);
  assert.deepEqual(groups[0].rows,[rows[0],rows[2]]);
  assert.equal(groups[1].pkg.collectors.length,0);
});
test('rejects ambiguous or incomplete links before producing any groups',()=>{
  for (const bad of [{...pkg,artworks:[...pkg.artworks,artwork('a')]},
    {...pkg,collectors:[...pkg.collectors,{key:'2',first_name:'ADA',last_name:'EXAMPLE'}]},
    {...pkg,sales:[pkg.sales[0]]},{...pkg,sales:[...pkg.sales,sale('B','1')]},
    {...pkg,collectors:[...pkg.collectors,{key:'2',first_name:'Unused'}]},
    {...pkg,gallery:[]}]) assert.throws(()=>groupArchiveMedia(bad,bad.artworks.map(a=>({code:a.code,photos:[]}))));
  assert.throws(()=>groupArchiveMedia(pkg,[rows[0],rows[0],rows[2]]));
});
// A deterministic assembler isolates the batching/publication contract from
// full media decoding; actual association and hashes have their own exporter tests.
function pngHeader(w,h,size=33) {
  const bytes=Buffer.alloc(size);
  Buffer.from([137,80,78,71,13,10,26,10]).copy(bytes);
  bytes.writeUInt32BE(13,8); bytes.write('IHDR',12);
  bytes.writeUInt32BE(w,16); bytes.writeUInt32BE(h,20);
  return bytes.toString('base64');
}
const attach=async(p,r)=>JSON.stringify({...p,covers:r.flatMap(x=>x.photos),gallery:[]});
async function fixture(t) {
  const root=await mkdtemp(join(tmpdir(),'repertica-batches-'));
  t.after(()=>rm(root,{recursive:true,force:true}));
  return root;
}
test('publishes bounded batches, exact SHA-256 manifest, and never replaces output',async t=>{
  const root=await fixture(t),dir=join(root,'new');
  const big=pngHeader(1,1,7*1024*1024);
  const projected=rows.map((r,i)=>({...r,photos:i===2?[]:[{artwork_code:r.code,data_base64:big}]}));
  const result=await publishArchiveMediaBatches(dir,pkg,projected,attach);
  assert.equal(result.length,2);
  assert.deepEqual(result.map(x=>x.artworks),[2,1]);
  assert.deepEqual(JSON.parse(await readFile(join(dir,'manifest.json'),'utf8')).packages,result);
  for (const row of result) {
    const bytes=await readFile(join(dir,row.file));
    assert.equal(bytes.length,row.bytes);
    assert.equal(createHash('sha256').update(bytes).digest('hex'),row.sha256);
    assert.ok(bytes.length<=16*1024*1024);
  }
  await assert.rejects(publishArchiveMediaBatches(dir,pkg,rows,attach));
  assert.equal((await readdir(dir)).length,3);
});
test('cleans up previous batches on late error and leaves existing directory untouched',async t=>{
  const root=await fixture(t),dir=join(root,'new');
  const big=pngHeader(1,1,7*1024*1024);
  const selected={...pkg,artworks:[...pkg.artworks,artwork('D'),artwork('E')]};
  const projected=selected.artworks.map(a=>({code:a.code,photos:[{artwork_code:a.code,data_base64:big}]}));
  // Group A/C is already too large; no output is published.
  await assert.rejects(publishArchiveMediaBatches(dir,selected,projected,attach));
  assert.deepEqual(await readdir(root),[]);
  // Force a late assembler failure after the first batch was flushed.
  const one=artwork('X'),two=artwork('Y'),three=artwork('Z');
  const simple={...pkg,artworks:[one,two,three],sales:[],collectors:[]};
  const r=simple.artworks.map(a=>({code:a.code,photos:[{artwork_code:a.code,data_base64:big}]}));
  await assert.rejects(publishArchiveMediaBatches(dir,simple,r,async(p,rr)=>{
    if (p.artworks[0].code==='Z') throw Error('missing blob');
    return attach(p,rr);
  }));
  assert.deepEqual(await readdir(root),[]);
  await writeFile(dir,'existing');
  await assert.rejects(publishArchiveMediaBatches(dir,pkg,rows,attach));
  assert.equal(await readFile(dir,'utf8'),'existing');
});


test('uses measured pixels rather than a fixed eight-photo limit',async t=>{
  const root=await fixture(t);
  const simple={...pkg,artworks:Array.from({length:9},(_,i)=>artwork('P'+i)),sales:[],collectors:[]};
  const project=(w,h)=>simple.artworks.map(a=>({code:a.code,photos:[{artwork_code:a.code,data_base64:pngHeader(w,h)}]}));
  const small=await publishArchiveMediaBatches(join(root,'small'),simple,project(200,200),attach);
  assert.deepEqual(small.map(p=>p.photographs),[9]);
  const large=await publishArchiveMediaBatches(join(root,'large'),simple,project(4000,4000),attach);
  assert.deepEqual(large.map(p=>p.photographs),[8,1]);
});
test('counts gallery pixels together with covers and never splits a collector group',async t=>{
  const root=await fixture(t),dir=join(root,'linked');
  const simple={...pkg,artworks:Array.from({length:9},(_,i)=>artwork('P'+i,'sold'))};
  simple.sales=simple.artworks.map(a=>sale(a.code,'1'));
  const projected=simple.artworks.map(a=>({code:a.code,photos:[{artwork_code:a.code,data_base64:pngHeader(4000,4000)}]}));
  const attachMixed=async(p,r)=>JSON.stringify({...p,covers:r.slice(0,1).flatMap(x=>x.photos),gallery:r.slice(1).flatMap(x=>x.photos)});
  await assert.rejects(publishArchiveMediaBatches(dir,simple,projected,attachMixed),/pixel bounds/);
  assert.deepEqual(await readdir(root),[]);
});
