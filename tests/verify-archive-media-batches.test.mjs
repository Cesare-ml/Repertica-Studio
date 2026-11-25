import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, rm, writeFile, readFile, unlink, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { verifyArchiveMediaBatches } from '../scripts/verify-archive-media-batches.mjs';

const pkg=(code,collectors=[])=>({format:'repertica-studio-archive-relations-v2',artworks:[{code}],collectors,sales:[],covers:[],gallery:[]});
async function fixture(t,packages=[pkg('A'),pkg('B')]) {
  const root=await mkdtemp(join(tmpdir(),'repertica-verify-'));
  t.after(()=>rm(root,{recursive:true,force:true}));
  const entries=[];
  for(const [i,p] of packages.entries()) {
    const file=`package-${String(i+1).padStart(4,'0')}.json`,data=Buffer.from(JSON.stringify(p));
    await writeFile(join(root,file),data);
    entries.push({file,sha256:createHash('sha256').update(data).digest('hex'),bytes:data.length,artworks:p.artworks.length,
      collectors:p.collectors.length,sales:p.sales.length,photographs:p.covers.length+p.gallery.length});
  }
  const manifest={format:'repertica-studio-media-batches-v1',packages:entries};
  const save=()=>writeFile(join(root,'manifest.json'),JSON.stringify(manifest));
  await save(); return {root,manifest,save};
}
test('verifies complete directory and leaves bytes untouched',async t=>{
  const {root}=await fixture(t);
  const before=await readFile(join(root,'package-0001.json'));
  assert.deepEqual(await verifyArchiveMediaBatches(root),{packages:2,artworks:2,collectors:0,sales:0,photographs:0});
  assert.deepEqual(await readFile(join(root,'package-0001.json')),before);
});
test('rejects changed bytes, missing files and unlisted files',async t=>{
  const {root}=await fixture(t); const p=join(root,'package-0001.json');
  const before=await readFile(p); const changed=Buffer.from(before); changed[0]=32;
  await writeFile(p,changed); await assert.rejects(verifyArchiveMediaBatches(root),/integrity/);
  await writeFile(p,before); await writeFile(join(root,'extra.json'),'{}');
  await assert.rejects(verifyArchiveMediaBatches(root),/contents/);
  await unlink(join(root,'extra.json')); await unlink(p);
  await assert.rejects(verifyArchiveMediaBatches(root),/contents/);
});
test('rejects traversal, reordered names and false counts',async t=>{
  const {root,manifest,save}=await fixture(t); const entry=manifest.packages[0];
  entry.file='../outside.json'; await save(); await assert.rejects(verifyArchiveMediaBatches(root),/entry/);
  entry.file='package-0002.json'; await save(); await assert.rejects(verifyArchiveMediaBatches(root),/entry/);
  entry.file='package-0001.json'; entry.artworks=2; await save(); await assert.rejects(verifyArchiveMediaBatches(root),/count/);
});
test('rejects cross-batch codes and collector contacts',async t=>{
  const a=await fixture(t,[pkg('ART-1'),pkg('art-1')]);
  await assert.rejects(verifyArchiveMediaBatches(a.root),/artwork code/);
  const b=await fixture(t,[pkg('A',[{first_name:'Ada',email:'x@example.invalid'}]),pkg('B',[{first_name:'Other',email:'X@example.invalid'}])]);
  await assert.rejects(verifyArchiveMediaBatches(b.root),/collector contact/);
});
test('rejects duplicate manifest keys and symlink packages',async t=>{
  const {root,save}=await fixture(t);
  const path=join(root,'manifest.json'); const valid=await readFile(path,'utf8');
  await writeFile(path,valid.replace('{','{"format":"bad",'));
  await assert.rejects(verifyArchiveMediaBatches(root),/Duplicate JSON field/); await save();
  const p=join(root,'package-0001.json'); await unlink(p);
  try { await symlink(join(root,'package-0002.json'),p,'file'); }
  catch(e) { if(['EPERM','EACCES','ENOSYS'].includes(e.code)) {t.skip('Symlink creation unavailable');return;} throw e; }
  await assert.rejects(verifyArchiveMediaBatches(root),/Invalid local file/);
});
