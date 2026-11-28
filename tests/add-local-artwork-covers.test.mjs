import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, writeFile, readFile, rm, symlink, access } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { addCovers, COVER_LIMIT, PACKAGE_LIMIT } from '../scripts/add-local-artwork-covers.mjs';
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=','base64');
const pkg = {format:'repertica-studio-archive-relations-v2',artworks:[{code:'A',title:'Synthetic',status:'available'}],collectors:[],sales:[]};
async function fixture(fn) {
  const dir=await mkdtemp(join(tmpdir(),'repertica-cover-'));
  try {
    const source=join(dir,'source.json'), output=join(dir,'output.json'), photo=join(dir,'photo.png');
    await writeFile(source,JSON.stringify(pkg)); await writeFile(photo,png);
    await fn({dir,source,output,photo});
  } finally { await rm(dir,{recursive:true,force:true}); }
}
test('preserves selected bytes, hash and relations without exposing paths',()=>fixture(async({source,output,photo})=>{
  await addCovers(source,output,['A',photo]);
  const raw=await readFile(output,'utf8'), result=JSON.parse(raw);
  assert.equal(raw.includes(photo),false);
  assert.deepEqual(result.covers,[{artwork_code:'A',sha256:createHash('sha256').update(png).digest('hex'),data_base64:png.toString('base64')}]);
  delete result.covers; assert.deepEqual(result,pkg);
  assert.deepEqual(JSON.parse(await readFile(source,'utf8')),pkg);
  await assert.rejects(addCovers(source,output,['A',photo]));
  assert.equal(await readFile(output,'utf8'),raw);
}));
test('rejects unknown and repeated selections and malformed packages',()=>fixture(async({source,output,photo})=>{
  for(const pairs of [['B',photo],['A',photo,'A',photo],['A']]) await assert.rejects(addCovers(source,output,pairs));
  await writeFile(source,JSON.stringify({...pkg,covers:[]}));
  await assert.rejects(addCovers(source,output,['A',photo]));
  await writeFile(source,JSON.stringify(pkg).replace('"code":"A"','"code":"B","code":"A"'));
  await assert.rejects(addCovers(source,output,['A',photo]));
  await assert.rejects(access(output));
}));
test('rejects unsupported files and oversized combined packages',()=>fixture(async({source,output,photo})=>{
  await writeFile(photo,'not an image'); await assert.rejects(addCovers(source,output,['A',photo]));
  await writeFile(photo,Buffer.alloc(COVER_LIMIT+1)); await assert.rejects(addCovers(source,output,['A',photo]));
  const large=Buffer.alloc(COVER_LIMIT);png.copy(large);await writeFile(photo,large);
  await writeFile(source,JSON.stringify({...pkg,artworks:[{...pkg.artworks[0],description:'x'.repeat(PACKAGE_LIMIT - COVER_LIMIT)}]}));
  await assert.rejects(addCovers(source,output,['A',photo])); await assert.rejects(access(output));
}));
test('rejects symlink input when the platform permits creating it',()=>fixture(async({dir,source,output,photo})=>{
  const alias=join(dir,'alias');
  try { await symlink(photo,alias,'file'); } catch (e) { if (['EPERM','EACCES','ENOSYS'].includes(e.code)) return; throw e; }
  await assert.rejects(addCovers(source,output,['A',alias]));await assert.rejects(access(output));
}));
