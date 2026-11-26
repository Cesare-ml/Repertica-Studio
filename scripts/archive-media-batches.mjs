// Split a selected v2 archive without splitting a collector's purchase history.
import { createHash } from 'node:crypto';
import { createLocalPackageDirectory } from './publish-local-package.mjs';
import { addArchiveImagePixels } from './archive-image-pixels.mjs';

const FORMAT = 'repertica-studio-archive-relations-v2';
const LIMIT = 16 * 1024 * 1024;
const PIXEL_LIMIT = 128_000_000;
const encode = pkg => JSON.stringify(pkg, null, 2) + '\n';
const asciiFold = s => s.replace(/[A-Z]/g, c => c.toLowerCase());
const blank = () => ({format:FORMAT,artworks:[],collectors:[],sales:[]});
const record = x => x && typeof x === 'object' && !Array.isArray(x);

export function groupArchiveMedia(pkg, rows) {
  if (!record(pkg) || pkg.format !== FORMAT || Object.keys(pkg).length !== 4 ||
      !['artworks','collectors','sales'].every(k => Array.isArray(pkg[k]) && pkg[k].length <= 1000) ||
      !pkg.artworks.length || !Array.isArray(rows) || rows.length !== pkg.artworks.length)
    throw Error('Expected unaugmented v2 selection');
  const artworks = new Map(), codes = new Set(), collectors = new Map(), names = new Set(), emails = new Set();
  const sales = new Map(), media = new Map(), groups = new Map();
  for (const a of pkg.artworks) {
    if (!record(a) || typeof a.code !== 'string' || !a.code || a.code.trim() !== a.code || codes.has(asciiFold(a.code)))
      throw Error('Ambiguous artwork code');
    codes.add(asciiFold(a.code)); artworks.set(a.code,a);
  }
  for (const c of pkg.collectors) {
    if (!record(c) || typeof c.key !== 'string' || !c.key || collectors.has(c.key) ||
        typeof c.first_name !== 'string' || !c.first_name.trim() ||
        (c.last_name != null && typeof c.last_name !== 'string') ||
        (c.email != null && typeof c.email !== 'string')) throw Error('Invalid collector');
    const name=asciiFold([c.first_name.trim(),c.last_name?.trim()].filter(Boolean).join(' '));
    const email=c.email?.trim() ? asciiFold(c.email.trim()) : null;
    if (names.has(name) || (email && emails.has(email))) throw Error('Ambiguous collector contact');
    names.add(name); if (email) emails.add(email); collectors.set(c.key,c);
  }
  for (const s of pkg.sales) {
    if (!record(s) || artworks.get(s.artwork_code)?.status !== 'sold' || !collectors.has(s.collector_key) ||
        sales.has(s.artwork_code) || !Array.isArray(s.receipts) || !Array.isArray(s.installments))
      throw Error('Unresolved sale');
    sales.set(s.artwork_code,s);
  }
  for (const row of rows) {
    if (!record(row) || !artworks.has(row.code) || media.has(row.code) || !Array.isArray(row.photos))
      throw Error('Source selection differs from package');
    media.set(row.code,row);
  }
  const used = new Set();
  for (const a of pkg.artworks) {
    const s=sales.get(a.code);
    if ((a.status === 'sold') !== Boolean(s)) throw Error('Unresolved sold artwork');
    const key=s ? 'collector:'+s.collector_key : 'artwork:'+a.code;
    if (!groups.has(key)) groups.set(key,{pkg:blank(),rows:[]});
    const g=groups.get(key);
    g.pkg.artworks.push(a); g.rows.push(media.get(a.code));
    if (s) {
      g.pkg.sales.push(s);
      if (!used.has(s.collector_key)) {
        used.add(s.collector_key); g.pkg.collectors.push(collectors.get(s.collector_key));
      }
    }
  }
  if (used.size !== collectors.size) throw Error('Unlinked collector');
  return [...groups.values()];
}

function combine(a,b) {
  return {format:FORMAT,...Object.fromEntries(['artworks','collectors','sales','covers','gallery'].map(k=>[k,[...(a[k]??[]),...(b[k]??[])]]))};
}
const imageCount = p => p.covers.length + p.gallery.length;
// Measure each assembled group once, including both covers and gallery entries.
// The native preview still performs full decoding and hash verification.
function groupPixels(p) {
  let total=0;
  for (const photo of [...p.covers,...p.gallery]) {
    total=addArchiveImagePixels(total,Buffer.from(photo.data_base64,'base64'));
  }
  return total;
}
const bounded = p => p.artworks.length <= 1000 && p.collectors.length <= 1000 && p.sales.length <= 1000 &&
  p.sales.reduce((n,s)=>n+s.receipts.length,0) <= 1000 && p.sales.reduce((n,s)=>n+s.installments.length,0) <= 1000 &&
  imageCount(p) <= 1000 && Buffer.byteLength(encode(p),'utf8') <= LIMIT;

// The caller supplies the existing verified local-media assembler. Each group is
// independently assembled; only one batch and one group are retained in memory.
export async function publishArchiveMediaBatches(output, pkg, rows, attach) {
  const groups=groupArchiveMedia(pkg,rows);
  const destination=await createLocalPackageDirectory(output);
  const packages=[];
  let batch=null, batchPixels=0;
  async function flush() {
    if (!batch) return;
    const data=encode(batch), name=`package-${String(packages.length+1).padStart(4,'0')}.json`;
    await destination.publish(name,data);
    packages.push({file:name,sha256:createHash('sha256').update(data,'utf8').digest('hex'),
      bytes:Buffer.byteLength(data,'utf8'),artworks:batch.artworks.length,collectors:batch.collectors.length,
      sales:batch.sales.length,photographs:imageCount(batch)});
    batch=null; batchPixels=0;
  }
  try {
    for (const group of groups) {
      const part=JSON.parse(await attach(group.pkg,group.rows));
      if (!bounded(part)) throw Error('One linked group exceeds package limits');
      const partPixels=groupPixels(part);
      if (batch && (batch.artworks.length+part.artworks.length>100 || batchPixels+partPixels>PIXEL_LIMIT)) await flush();
      if (batch) {
        const candidate=combine(batch,part);
        if (bounded(candidate)) {batch=candidate;batchPixels+=partPixels;continue;}
        await flush();
      }
      batch=part; batchPixels=partPixels;
    }
    await flush();
    // Written last: no manifest means incomplete export, never import it.
    await destination.publish('manifest.json',encode({format:'repertica-studio-media-batches-v1',packages}));
    return packages;
  } catch (error) {
    await destination.cleanup();
    throw error;
  }
}
