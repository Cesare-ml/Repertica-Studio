// Read-only preflight for a completed local export directory. Not an import.
import { createHash } from 'node:crypto';
import { lstat, readdir, realpath } from 'node:fs/promises';
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { pathToFileURL } from 'node:url';
import { readRegular, PACKAGE_LIMIT, COVER_LIMIT } from './add-local-artwork-covers.mjs';
import { addArchiveImagePixels } from './archive-image-pixels.mjs';
import { parseLegacyJson } from './strict-legacy-json.mjs';
import { createLocalPackageDirectory } from './publish-local-package.mjs';
import { date, validateInstallmentNotes, validateCommercialText } from './gestionale-commercial-values.mjs';

const exact = (v, keys) => v && typeof v === 'object' && !Array.isArray(v) &&
  Object.keys(v).length === keys.length && keys.every(k => Object.hasOwn(v,k));
const decode = b => parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(b));
const sha = b => createHash('sha256').update(b).digest('hex');
const folded = s => s.replace(/[A-Z]/g,c=>c.toLowerCase());
const boundedInt = (v,min,max) => Number.isSafeInteger(v) && v>=min && v<=max;
const fields = ['file','sha256','bytes','artworks','collectors','sales','photographs'];

// Check embedded media independently of the outer package hash. This remains a
// header-only preflight; the native preview must decode every image.
function verifyPackageMedia(pkg) {
  const codes = new Set(pkg.artworks.map(a => a.code));
  const covers = new Set(), positions = new Map();
  let pixels = 0;
  function verify(photo) {
    if (!codes.has(photo?.artwork_code) || typeof photo.data_base64 !== 'string' ||
        photo.data_base64.length > Math.ceil(COVER_LIMIT / 3) * 4 ||
        typeof photo.sha256 !== 'string' || !/^[a-f0-9]{64}$/.test(photo.sha256)) throw Error('Invalid embedded photograph');
    const bytes = Buffer.from(photo.data_base64, 'base64');
    if (!bytes.length || bytes.length > COVER_LIMIT || bytes.toString('base64') !== photo.data_base64 ||
        sha(bytes) !== photo.sha256) throw Error('Embedded photograph integrity mismatch');
    pixels = addArchiveImagePixels(pixels, bytes);
  }
  for (const photo of pkg.covers) {
    if (covers.has(photo?.artwork_code)) throw Error('Repeated cover');
    verify(photo);
    covers.add(photo.artwork_code);
  }
  for (const photo of pkg.gallery) {
    if (!boundedInt(photo?.position, 1, 20)) throw Error('Invalid gallery position');
    verify(photo);
    if (!positions.has(photo.artwork_code)) positions.set(photo.artwork_code, new Set());
    const seen = positions.get(photo.artwork_code);
    if (seen.has(photo.position)) throw Error('Repeated gallery position');
    seen.add(photo.position);
  }
  for (const seen of positions.values()) {
    for (let i = 1; i <= seen.size; i++) if (!seen.has(i)) throw Error('Noncontiguous gallery positions');
  }
}

// Structural and monetary checks for exporter-produced packages. Native preview
// still validates complete records, dates, media decoding and database conflicts.
function verifyPackageRelations(pkg, seenCollectorKeys) {
  const artworks = new Map(pkg.artworks.map(a => [a.code, a.status]));
  const keys = new Set(), linked = new Set(), used = new Set();
  const int = (n, min, max) => Number.isSafeInteger(n) && n >= min && n <= max;
  const maxMoney = 100_000_000_000;
  const fail = () => { throw Error('Invalid commercial relationship, date or amount'); };
  const checkDate = value => { try { return date(value); } catch { return fail(); } };
  for (const c of pkg.collectors) {
    if (typeof c?.key !== 'string' || !/^[A-Za-z0-9_-]{1,80}$/.test(c.key) ||
        keys.has(c.key) || seenCollectorKeys.has(c.key)) fail();
    keys.add(c.key);
  }
  let receipts = 0, installments = 0;
  for (const sale of pkg.sales) {
    if (artworks.get(sale?.artwork_code) !== 'sold' || linked.has(sale.artwork_code) ||
        !keys.has(sale.collector_key) || sale.currency !== 'EUR' ||
        !int(sale.amount_cents,1,maxMoney) || !int(sale.tax_cents,0,maxMoney) ||
        !int(sale.shipping_cents,0,maxMoney) || sale.tax_cents + sale.shipping_cents > sale.amount_cents ||
        !Array.isArray(sale.legacy_shares_bp) || sale.legacy_shares_bp.length !== 3 ||
        !sale.legacy_shares_bp.every(n => int(n,0,10000)) ||
        sale.legacy_shares_bp.reduce((a,b) => a+b,0) !== 10000 ||
        !Array.isArray(sale.allocations) || sale.allocations.length !== 0 ||
        !Array.isArray(sale.receipts) || !Array.isArray(sale.installments)) fail();
    if (Object.keys(sale).some(k => !['artwork_code','collector_key','amount_cents','currency','sold_at','notes',
        'tax_cents','shipping_cents','legacy_shares_bp','allocations','installments','receipts'].includes(k))) fail();
    try { validateCommercialText(sale.notes,15000); } catch { fail(); }
    checkDate(sale.sold_at);
    linked.add(sale.artwork_code); used.add(sale.collector_key);
    receipts += sale.receipts.length; installments += sale.installments.length;
    if (receipts > 1000 || installments > 1000) fail();
    let total = 0;
    for (const receipt of sale.receipts) {
      if (!int(receipt?.amount_cents,1,maxMoney) ||
          !['bank_transfer','cash','card','other'].includes(receipt.method)) fail();
      if (Object.keys(receipt).some(k => !['amount_cents','method','received_at','reference','notes'].includes(k))) fail();
      try {
        validateCommercialText(receipt.reference,200);
        validateCommercialText(receipt.notes,2000);
      } catch { fail(); }
      checkDate(receipt.received_at);
      total += receipt.amount_cents;
      if (total > sale.amount_cents) fail();
    }
    if (sale.installments.length) {
      if (sale.receipts.length || !int(sale.installments.length,2,60)) fail();
      total = 0;
      let previous = '';
      for (const [i,item] of sale.installments.entries()) {
        if (item?.position !== i+1 || !int(item.amount_cents,1,maxMoney)) fail();
        // Match the native optional note and reject unknown installment fields.
        // An excluded package must not evade these checks during recovery.
        if (Object.keys(item).some(k => !['position','amount_cents','due_date','notes'].includes(k))) fail();
        try { validateInstallmentNotes(item.notes ?? null); } catch { fail(); }
        const due = checkDate(item.due_date);
        if (due < previous) fail();
        previous = due;
        total += item.amount_cents;
        if (total > sale.amount_cents) fail();
      }
      if (total !== sale.amount_cents) fail();
    }
  }
  for (const [code,status] of artworks) if ((status === 'sold') !== linked.has(code)) fail();
  if (keys.size !== used.size) fail();
  for (const key of keys) seenCollectorKeys.add(key);
}

async function inspectArchiveMediaBatches(directory) {
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
  const codes=new Set(), names=new Set(), emails=new Set(), collectorKeys=new Set();
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
    verifyPackageMedia(pkg);
    for(const c of pkg.collectors) {
      if(typeof c?.first_name!=='string' || !c.first_name.trim() ||
          (c.last_name!=null && typeof c.last_name!=='string') || (c.email!=null && typeof c.email!=='string')) throw Error('Invalid collector');
      const name=folded([c.first_name.trim(),c.last_name?.trim()].filter(Boolean).join(' '));
      const email=c.email?.trim() ? folded(c.email.trim()) : null;
      if(names.has(name) || (email && emails.has(email))) throw Error('Repeated collector contact');
      names.add(name); if(email) emails.add(email);
    }
    verifyPackageRelations(pkg,collectorKeys);
  }
  await checkEntries();
  if(!(await readRegular(manifestPath,1024*1024)).equals(manifestBytes)) throw Error('Manifest changed');
  return {dir, manifestBytes, manifest, manifest_digest:sha(manifestBytes), totals};
}

export async function verifyArchiveMediaBatches(directory) {
  return (await inspectArchiveMediaBatches(directory)).totals;
}

// Manual recovery only: the operator has checked the active archive and explicitly
// selected whole packages known to be absent. This cannot establish commit status.
export async function prepareRemainingBatches(directory, output, expectedDigest, indexes) {
  const verified = await inspectArchiveMediaBatches(directory);
  if (expectedDigest !== verified.manifest_digest) throw Error('Original manifest changed');
  const entries = verified.manifest.packages;
  if (!Array.isArray(indexes) || indexes.length < 1 || indexes.length >= entries.length ||
      indexes.some((i,n) => !boundedInt(i,1,entries.length) || (n>0 && i<=indexes[n-1]))) {
    throw Error('Select a strict, ascending subset of whole absent packages');
  }
  const selected = resolve(output);
  const parent = await realpath(dirname(selected));
  const rel = relative(verified.dir,parent);
  if (!isAbsolute(rel) && (rel==='' || (rel!=='..' && !rel.startsWith('..'+sep)))) {
    throw Error('Recovery destination must be outside the original export');
  }
  const dest = join(parent,basename(selected));
  const destination=await createLocalPackageDirectory(dest);
  try {
    const packages=[];
    for (const index of indexes) {
      const entry=entries[index-1];
      const bytes=await readRegular(join(verified.dir,entry.file),PACKAGE_LIMIT);
      if(bytes.length!==entry.bytes || sha(bytes)!==entry.sha256) throw Error('Package integrity mismatch');
      const file=`package-${String(packages.length+1).padStart(4,'0')}.json`;
      await destination.publish(file,bytes);
      packages.push({...entry,file});
    }
    if (!(await readRegular(join(verified.dir,'manifest.json'),1024*1024)).equals(verified.manifestBytes)) {
      throw Error('Original manifest changed');
    }
    await destination.publish('manifest.json',JSON.stringify({format:verified.manifest.format,packages},null,2)+'\n');
    return await verifyArchiveMediaBatches(dest);
  } catch (error) {
    await destination.cleanup();
    throw error;
  }
}
if(process.argv[1] && import.meta.url===pathToFileURL(process.argv[1]).href) {
  try {
    const args=process.argv.slice(2);
    if(args[0]==='--prepare-remaining') {
      if(args.length!==6 || args[4]!=='--confirmed-absent' || !/^[1-9][0-9]*(?:,[1-9][0-9]*)*$/.test(args[5])) {
        throw Error('Explicit confirmed-absent package numbers required');
      }
      const totals=await prepareRemainingBatches(args[1],args[2],args[3],args[5].split(',').map(Number));
      console.log(JSON.stringify(totals));
      console.log('New recovery folder prepared. This is not proof of absence; run native preflight and confirm each package in Studio.');
    } else {
      if(args.length!==1) throw Error('Select one completed export directory');
      const result=await inspectArchiveMediaBatches(args[0]);
      console.log(JSON.stringify(result.totals));
      console.log('Manifest SHA-256: '+result.manifest_digest);
      console.log('Directory integrity verified. Preview every package in Studio before confirming; this is not semantic validation or an import.');
    }
  } catch {
    console.error('Batch verification or recovery rejected. Check the original export, explicit selection and new destination; do not import an unverified package.');
    process.exitCode=1;
  }
}
