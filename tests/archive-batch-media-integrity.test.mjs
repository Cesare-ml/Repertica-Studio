import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, readdir, rm, lstat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createLocalPackageDirectory } from '../scripts/publish-local-package.mjs';
import { verifyArchiveMediaBatches, prepareRemainingBatches } from '../scripts/verify-archive-media-batches.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
// Small PNG fixture; only the header is inspected by this verifier.
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jR1sAAAAASUVORK5CYII=', 'base64');
const photo = (code, bytes = png) => ({artwork_code:code, sha256:sha(bytes), data_base64:bytes.toString('base64')});
const gallery = (code, position, bytes = png) => ({...photo(code,bytes), position, kind:'DETAIL', caption:null});
const pkg = code => ({format:'repertica-studio-archive-relations-v2', artworks:[{code,title:code,status:'available'}], collectors:[], sales:[], covers:[], gallery:[]});

async function fixture(t, packages) {
  const root = await mkdtemp(join(tmpdir(),'repertica-batch-media-'));
  t.after(() => rm(root,{recursive:true,force:true}));
  const original = join(root,'original'), dest = join(root,'recovery');
  const output = await createLocalPackageDirectory(original);
  const entries = [];
  for (const [i,p] of packages.entries()) {
    const file = `package-${String(i+1).padStart(4,'0')}.json`, data = Buffer.from(JSON.stringify(p));
    await output.publish(file,data);
    entries.push({file,sha256:sha(data),bytes:data.length,artworks:p.artworks.length,collectors:p.collectors.length,sales:p.sales.length,photographs:p.covers.length+p.gallery.length});
  }
  const manifest = JSON.stringify({format:'repertica-studio-media-batches-v1',packages:entries});
  await output.publish('manifest.json',manifest);
  return {root,original,dest,digest:sha(manifest)};
}

test('verified cover and ordered gallery survive recovery with identical package bytes',async t => {
  const a = pkg('A'), b = pkg('B');
  b.covers.push(photo('B')); b.gallery.push(gallery('B',1),gallery('B',2));
  const {original,dest,digest} = await fixture(t,[a,b]);
  assert.equal((await verifyArchiveMediaBatches(original)).photographs,3);
  const before = await readFile(join(original,'package-0002.json'));
  assert.equal((await prepareRemainingBatches(original,dest,digest,[2])).photographs,3);
  assert.deepEqual(await readFile(join(dest,'package-0001.json')),before);
  assert.deepEqual(await readFile(join(original,'package-0002.json')),before);
});

test('recomputed outer hashes cannot hide an incorrect embedded digest or noncanonical base64',async t => {
  for (const mutate of [p => {p.sha256='0'.repeat(64);},p => {p.data_base64+='\n';}]) {
    const a=pkg('A'); a.covers.push(photo('A')); mutate(a.covers[0]);
    const {original}=await fixture(t,[a]);
    await assert.rejects(verifyArchiveMediaBatches(original),/Embedded photograph integrity/);
  }
});

test('rejects orphan media, duplicate covers, and duplicate or gapped gallery positions',async t => {
  for (const [mutate,pattern] of [
    [p=>p.covers.push(photo('B')), /Invalid embedded photograph/],
    [p=>p.covers.push(photo('A'),photo('A')), /Repeated cover/],
    [p=>p.gallery.push(gallery('A',1),gallery('A',1)), /Repeated gallery position/],
    [p=>p.gallery.push(gallery('A',2)), /Noncontiguous gallery positions/],
    [p=>p.gallery.push(gallery('A',21)), /Invalid gallery position/],
  ]) {
    const a=pkg('A'); mutate(a);
    const {original}=await fixture(t,[a,pkg('B')]);
    await assert.rejects(verifyArchiveMediaBatches(original),pattern);
  }
});

test('rejects invalid headers and combined pixel budget even with correct inner and outer hashes',async t => {
  const bad=pkg('A'); bad.covers.push(photo('A',Buffer.from('not an image')));
  const f=await fixture(t,[bad]);
  await assert.rejects(verifyArchiveMediaBatches(f.original),/image dimensions/);
  // Synthetic dimensions only: full pixel decoding is native, not part of this test.
  const header=Buffer.from(png); header.writeUInt32BE(4000,16); header.writeUInt32BE(4000,20);
  const big=pkg('B'); big.covers.push(photo('B',header));
  for(let i=1;i<=7;i++) big.gallery.push(gallery('B',i,header));
  const good=await fixture(t,[big]);
  assert.equal((await verifyArchiveMediaBatches(good.original)).photographs,8);
  big.gallery.push(gallery('B',8,header));
  const over=await fixture(t,[big]);
  await assert.rejects(verifyArchiveMediaBatches(over.original),/pixel bounds/);
});

test('recovery refuses damaged media in an excluded package before creating any destination',async t => {
  const a=pkg('A'); a.covers.push({...photo('A'),sha256:'0'.repeat(64)});
  const {root,original,dest,digest}=await fixture(t,[a,pkg('B')]);
  await assert.rejects(prepareRemainingBatches(original,dest,digest,[2]),/Embedded photograph integrity/);
  await assert.rejects(lstat(dest),{code:'ENOENT'});
  assert.deepEqual(await readdir(root),['original']);
});

test('preflight accepts native cover defaults and gallery metadata at Unicode limits',async t => {
  const a=pkg('A');
  a.covers.push({...photo('A'),position:0,caption:null,kind:null});
  a.gallery.push({...photo('A'),position:1},
    {...gallery('A',2),caption:'🎨'.repeat(300)},
    {...gallery('A',3),caption:' '.repeat(301),kind:null});
  const {original,dest,digest}=await fixture(t,[pkg('B'),a]);
  assert.equal((await verifyArchiveMediaBatches(original)).photographs,4);
  assert.equal((await prepareRemainingBatches(original,dest,digest,[2])).photographs,4);
  assert.deepEqual(await readFile(join(dest,'package-0001.json')),
    await readFile(join(original,'package-0002.json')));
});

test('recovery rejects invalid cover and gallery metadata in excluded packages with valid hashes',async t => {
  for(const [isGallery,patch] of [
    [false,{position:null}],[false,{position:1}],[false,{caption:''}],
    [false,{kind:'OTHER'}],[false,{path:'private.jpg'}],
    [true,{kind:'WRONG'}],[true,{kind:1}],[true,{caption:42}],
    [true,{caption:false}],[true,{caption:[]}],[true,{caption:'x\0y'}],
    [true,{caption:'🎨'.repeat(301)}],[true,{caption:'x'+' '.repeat(300)}],
    [true,{url:'https://example.invalid'}],
  ]) {
    const a=pkg('A');
    if(isGallery) a.gallery.push({...gallery('A',1),...patch});
    else a.covers.push({...photo('A'),...patch});
    const {root,original,dest,digest}=await fixture(t,[a,pkg('B')]);
    await assert.rejects(verifyArchiveMediaBatches(original),/Invalid photograph metadata/);
    await assert.rejects(prepareRemainingBatches(original,dest,digest,[2]),/Invalid photograph metadata/);
    await assert.rejects(lstat(dest),{code:'ENOENT'});
    assert.deepEqual(await readdir(root),['original']);
  }
});
