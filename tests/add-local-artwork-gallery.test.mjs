import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, writeFile, readFile, rm, symlink, access } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { addGallery } from '../scripts/add-local-artwork-gallery.mjs';
const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=','base64');
const pkg={format:'repertica-studio-archive-relations-v2',artworks:[{code:'A',title:'Synthetic',status:'available'}],collectors:[],sales:[],covers:[{artwork_code:'A',sha256:createHash('sha256').update(png).digest('hex'),data_base64:png.toString('base64')}]};
async function fixture(fn) {
  const dir=await mkdtemp(join(tmpdir(),'repertica-gallery-'));
  try {
    const source=join(dir,'source.json'),output=join(dir,'output.json'),photo=join(dir,'photo.png');
    await writeFile(source,JSON.stringify(pkg));await writeFile(photo,png);
    const item={code:'A',position:1,kind:'DETAIL',caption:'First',path:photo};
    await fn({dir,source,output,photo,item});
  } finally { await rm(dir,{recursive:true,force:true}); }
}
test('preserves explicit order, captions, kind, bytes, cover and source without paths',()=>fixture(async({source,output,photo,item})=>{
  await addGallery(source,output,[{...item,position:2,caption:'Second',kind:'SIGNATURE'},item]);
  const raw=await readFile(output,'utf8'),result=JSON.parse(raw);
  assert.equal(raw.includes(photo),false);
  assert.deepEqual(result.gallery.map(({position,caption,kind,sha256,data_base64})=>[position,caption,kind,sha256,data_base64]),
    [[2,'Second','SIGNATURE',pkg.covers[0].sha256,png.toString('base64')],[1,'First','DETAIL',pkg.covers[0].sha256,png.toString('base64')]]);
  delete result.gallery;assert.deepEqual(result,pkg);
  assert.deepEqual(JSON.parse(await readFile(source,'utf8')),pkg);
  await assert.rejects(addGallery(source,output,[item]));assert.equal(await readFile(output,'utf8'),raw);
}));
test('rejects missing, repeated or invalid positions, unknown works, types and captions before output',()=>fixture(async({source,output,item})=>{
  for (const selections of [[{...item,position:2}],[item,item],[{...item,position:21}],[{...item,code:'B'}],[{...item,kind:'INVALID'}],[{...item,caption:'x'.repeat(301)}],[{...item,caption:'bad\0value'}]])
    await assert.rejects(addGallery(source,output,selections));
  await assert.rejects(access(output));
}));
test('rejects unsupported and symlink files without publishing',()=>fixture(async({dir,source,output,photo,item})=>{
  await writeFile(photo,'not a photo');await assert.rejects(addGallery(source,output,[item]));
  const alias=join(dir,'alias');try { await symlink(photo,alias,'file'); } catch (e) { if (['EPERM','EACCES','ENOSYS'].includes(e.code)) return;throw e; }
  await writeFile(photo,png);await assert.rejects(addGallery(source,output,[{...item,path:alias}]));await assert.rejects(access(output));
}));
