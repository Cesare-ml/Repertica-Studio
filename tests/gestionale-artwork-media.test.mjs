import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp,mkdir,writeFile,rm,symlink,realpath,readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { attachLocalMedia } from '../scripts/export-gestionale-artwork-media.mjs';
import { publishArchiveMediaBatches } from '../scripts/archive-media-batches.mjs';
const pkg={format:'repertica-studio-archive-relations-v2',artworks:[{code:'TEST-1'},{code:'TEST-2'}],collectors:[],sales:[]};
// Header-only fixture; full image decoding is the native preview's responsibility.
const bytes=Buffer.from([137,80,78,71,13,10,26,10,1,2,3]);
const sha256=createHash('sha256').update(bytes).digest('hex');
const cover={position:0,kind:null,caption:null,path:'blobs/a.png',sha256,size:String(bytes.length)};
const gallery={...cover,position:1,kind:'DETTAGLIO',caption:'Detail'};
const rows=(photos=[cover,gallery])=>[{code:'TEST-1',photos},{code:'TEST-2',photos:[]}];
async function fixture(t) {
  const root=await realpath(await mkdtemp(join(tmpdir(),'repertica-media-')));
  t.after(()=>rm(root,{recursive:true,force:true}));
  await mkdir(join(root,'blobs'));
  await writeFile(join(root,'blobs','a.png'),bytes);
  return root;
}
test('maps selected current bytes to covers and gallery without source paths',async t=>{
  const root=await fixture(t),result=await attachLocalMedia(pkg,rows(),root),out=JSON.parse(result);
  assert.equal(out.covers[0].artwork_code,'TEST-1');
  assert.equal(out.gallery[0].kind,'DETAIL');
  assert.equal(out.gallery[0].caption,'Detail');
  assert.equal(out.gallery[0].position,1);
  assert.equal(out.gallery[0].sha256,sha256);
  assert.deepEqual(Buffer.from(out.covers[0].data_base64,'base64'),bytes);
  assert.ok(!result.includes('blobs/a.png'));
  assert.ok(!Object.hasOwn(pkg,'covers'));
});
test('refuses missing or foreign artwork and ambiguous/unsupported gallery',async t=>{
  const root=await fixture(t);
  for(const bad of [[],[rows()[0],rows()[0]],[rows()[0],{code:'FOREIGN',photos:[]}],
    rows([{...gallery,position:2}]),rows([gallery,gallery]),rows([{...gallery,kind:'UNKNOWN'}])])
    await assert.rejects(attachLocalMedia(pkg,bad,root));
  await assert.rejects(attachLocalMedia({...pkg,covers:[]},rows(),root));
});
test('refuses unavailable blob, changed hash/length and unsafe paths',async t=>{
  const root=await fixture(t);
  for(const bad of [{...cover,path:null,sha256:null,size:null},{...cover,sha256:'0'.repeat(64)},
    {...cover,size:'9'},{...cover,path:'../a.png'},{...cover,path:'/a.png'},
    {...cover,path:'C:/a.png'},{...cover,path:'blobs\\a.png'},{...cover,path:'blobs/missing.png'}])
    await assert.rejects(attachLocalMedia(pkg,rows([bad]),root));
});
test('refuses symlinked blob or ancestor',{skip:process.platform==='win32'},async t=>{
  const root=await fixture(t);
  await symlink(join(root,'blobs','a.png'),join(root,'alias.png'));
  await symlink(join(root,'blobs'),join(root,'alias-dir'));
  for(const path of ['alias.png','alias-dir/a.png']) await assert.rejects(attachLocalMedia(pkg,rows([{...cover,path}]),root));
});

test('batch mode uses real local media association and keeps files in independent v2 packages',async t=>{
  const root=await fixture(t),output=join(root,'batch');
  const artworks=Array.from({length:9},(_,i)=>({code:'WORK-'+i,title:'Example',status:'available'}));
  const selection={...pkg,artworks};
  const projected=artworks.map(a=>({code:a.code,photos:[cover]}));
  const result=await publishArchiveMediaBatches(output,selection,projected,(p,r)=>attachLocalMedia(p,r,root));
  assert.deepEqual(result.map(x=>x.photographs),[8,1]);
  const imported=[];
  for(const row of result) {
    const data=await readFile(join(output,row.file));
    assert.equal(createHash('sha256').update(data).digest('hex'),row.sha256);
    const p=JSON.parse(data);
    for(const c of p.covers) assert.deepEqual(Buffer.from(c.data_base64,'base64'),bytes);
    imported.push(...p.artworks.map(a=>a.code));
  }
  assert.deepEqual(imported,artworks.map(a=>a.code));
});
