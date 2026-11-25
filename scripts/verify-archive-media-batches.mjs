// Read-only preflight for a completed local export directory. Not an import.
import { createHash } from 'node:crypto';
import { lstat, readdir, realpath } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { readRegular, PACKAGE_LIMIT } from './add-local-artwork-covers.mjs';
import { parseLegacyJson } from './strict-legacy-json.mjs';

const exact = (v, keys) => v && typeof v === 'object' && !Array.isArray(v) &&
  Object.keys(v).length === keys.length && keys.every(k => Object.hasOwn(v,k));
const decode = b => parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(b));
const sha = b => createHash('sha256').update(b).digest('hex');
const folded = s => s.replace(/[A-Z]/g,c=>c.toLowerCase());
const boundedInt = (v,min,max) => Number.isSafeInteger(v) && v>=min && v<=max;
const fields = ['file','sha256','bytes','artworks','collectors','sales','photographs'];

export async function verifyArchiveMediaBatches(directory) {
  const selected=resolve(directory);
  if (!(await lstat(selected)).isDirectory()) throw Error('Unsafe export directory');
  const dir=await realpath(selected);
  const manifestPath=join(dir,'manifest.json');
  const manifestBytes=await readRegular(manifestPath,1024*1024);
  const manifest=decode(manifestBytes);
  if (!exact(manifest,['format','packages']) || manifest.format!=='repertica-studio-media-batches-v1' ||
      !Array.isArray(manifest.packages) || !boundedInt(manifest.packages.length,1,1000)) throw Error('Invalid manifest');
  const expected=['manifest.json'];
  const totals={packages:manifest.packages.length,artworks:0,collectors:0,sales:0,photographs:0};
  for (const [i,row] of manifest.packages.entries()) {
    if (!exact(row,fields) || row.file!==`package-${String(i+1).padStart(4,'0')}.json` ||
        typeof row.sha256!=='string' || !/^[0-9a-f]{64}$/.test(row.sha256) ||
        !boundedInt(row.bytes,1,PACKAGE_LIMIT) || !boundedInt(row.artworks,1,1000) ||
        !['collectors','sales','photographs'].every(k=>boundedInt(row[k],0,1000))) throw Error('Invalid manifest entry');
    expected.push(row.file);
    for(const k of ['artworks','collectors','sales','photographs']) totals[k]+=row[k];
  }
  if(totals.artworks>1000 || totals.collectors>1000 || totals.sales>1000 || totals.photographs>21000) throw Error('Export selection exceeds bounds');
  const checkEntries=async()=>{
    const actual=(await readdir(dir)).sort();
    if(actual.length!==expected.length || actual.some((v,i)=>v!==expected[i])) throw Error('Incomplete or unexpected directory contents');
  };
  expected.sort();
  await checkEntries();
  const codes=new Set(), names=new Set(), emails=new Set();
  for(const row of manifest.packages) {
    const bytes=await readRegular(join(dir,row.file),PACKAGE_LIMIT);
    if(bytes.length!==row.bytes || sha(bytes)!==row.sha256) throw Error('Package integrity mismatch');
    const pkg=decode(bytes);
    if(!exact(pkg,['format','artworks','collectors','sales','covers','gallery']) ||
        pkg.format!=='repertica-studio-archive-relations-v2' ||
        !['artworks','collectors','sales','covers','gallery'].every(k=>Array.isArray(pkg[k])) ||
        pkg.artworks.length!==row.artworks || pkg.collectors.length!==row.collectors || pkg.sales.length!==row.sales ||
        pkg.covers.length+pkg.gallery.length!==row.photographs) throw Error('Package count mismatch');
    for(const a of pkg.artworks) {
      if(typeof a?.code!=='string' || !a.code || a.code.trim()!==a.code || codes.has(folded(a.code))) throw Error('Repeated or invalid artwork code');
      codes.add(folded(a.code));
    }
    for(const c of pkg.collectors) {
      if(typeof c?.first_name!=='string' || !c.first_name.trim() ||
          (c.last_name!=null && typeof c.last_name!=='string') || (c.email!=null && typeof c.email!=='string')) throw Error('Invalid collector');
      const name=folded([c.first_name.trim(),c.last_name?.trim()].filter(Boolean).join(' '));
      const email=c.email?.trim() ? folded(c.email.trim()) : null;
      if(names.has(name) || (email && emails.has(email))) throw Error('Repeated collector contact');
      names.add(name); if(email) emails.add(email);
    }
  }
  await checkEntries();
  if(!(await readRegular(manifestPath,1024*1024)).equals(manifestBytes)) throw Error('Manifest changed');
  return totals;
}
if(process.argv[1] && import.meta.url===pathToFileURL(process.argv[1]).href) {
  try {
    if(process.argv.length!==3) throw Error('Select one completed export directory');
    const totals=await verifyArchiveMediaBatches(process.argv[2]);
    console.log(JSON.stringify(totals));
    console.log('Directory integrity verified. Preview every package in Studio before confirming; this is not semantic validation or an import.');
  } catch {
    console.error('Batch verification rejected. Check the complete export directory and manifest; do not import any package.');
    process.exitCode=1;
  }
}
