import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, rm, writeFile, readFile, unlink, symlink, mkdir, readdir, lstat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { verifyArchiveMediaBatches, prepareRemainingBatches } from '../scripts/verify-archive-media-batches.mjs';

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

const manifestDigest=async root=>createHash('sha256').update(await readFile(join(root,'manifest.json'))).digest('hex');
test('manual recovery renumbers explicitly absent packages without changing any payload or source file',async t=>{
  const {root}=await fixture(t,[pkg('A'),pkg('B'),pkg('C')]);
  const parent=await mkdtemp(join(tmpdir(),'repertica-recovery-'));
  t.after(()=>rm(parent,{recursive:true,force:true}));
  const dest=join(parent,'remaining');
  const before=await Promise.all((await readdir(root)).sort().map(async name=>[name,await readFile(join(root,name))]));
  const digest=await manifestDigest(root);
  assert.deepEqual(await prepareRemainingBatches(root,dest,digest,[2,3]),{packages:2,artworks:2,collectors:0,sales:0,photographs:0});
  assert.deepEqual(await readFile(join(dest,'package-0001.json')),await readFile(join(root,'package-0002.json')));
  assert.deepEqual(await readFile(join(dest,'package-0002.json')),await readFile(join(root,'package-0003.json')));
  assert.deepEqual(await verifyArchiveMediaBatches(dest),{packages:2,artworks:2,collectors:0,sales:0,photographs:0});
  for(const [name,bytes] of before) assert.deepEqual(await readFile(join(root,name)),bytes);
});
test('recovery refuses ambiguous or unpinned selections without creating output',async t=>{
  const {root}=await fixture(t,[pkg('A'),pkg('B'),pkg('C')]);
  const parent=await mkdtemp(join(tmpdir(),'repertica-recovery-'));
  t.after(()=>rm(parent,{recursive:true,force:true}));
  const dest=join(parent,'remaining'),digest=await manifestDigest(root);
  for(const indexes of [[],[1,2,3],[2,2],[3,2],[0],[4],[1.5],['2']]) {
    await assert.rejects(prepareRemainingBatches(root,dest,digest,indexes),/subset/);
    await assert.rejects(lstat(dest),{code:'ENOENT'});
  }
  await assert.rejects(prepareRemainingBatches(root,dest,'0'.repeat(64),[2]),/manifest changed/);
  await assert.rejects(lstat(dest),{code:'ENOENT'});
  await assert.rejects(prepareRemainingBatches(root,join(root,'nested'),digest,[2]),/outside/);
  await assert.rejects(lstat(join(root,'nested')),{code:'ENOENT'});
  await mkdir(dest);await writeFile(join(dest,'keep'),'existing');
  await assert.rejects(prepareRemainingBatches(root,dest,digest,[2]),{code:'EEXIST'});
  assert.equal(await readFile(join(dest,'keep'),'utf8'),'existing');
});
test('recovery refuses a damaged original even when the damaged package is not selected',async t=>{
  const {root}=await fixture(t);
  const parent=await mkdtemp(join(tmpdir(),'repertica-recovery-'));
  t.after(()=>rm(parent,{recursive:true,force:true}));
  const dest=join(parent,'remaining'),digest=await manifestDigest(root);
  await writeFile(join(root,'package-0001.json'),'tampered');
  await assert.rejects(prepareRemainingBatches(root,dest,digest,[2]),/integrity/);
  await assert.rejects(lstat(dest),{code:'ENOENT'});
});
