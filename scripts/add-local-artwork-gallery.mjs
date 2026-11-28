// Explicit selections in a trusted local copy; never resolve paths from a package.
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { parseLegacyJson } from './strict-legacy-json.mjs';
import { publishLocalPackage } from './publish-local-package.mjs';
import { readRegular, hasImageHeader, PACKAGE_LIMIT, COVER_LIMIT } from './add-local-artwork-covers.mjs';

const KINDS = new Set(['DETAIL','SIGNATURE','DIGITAL_MOCKUP','IN_SITU','OTHER','FRAMED','WORK_IN_PROGRESS']);
export async function addGallery(input, output, selections) {
  if (!Array.isArray(selections) || !selections.length || selections.length > 1000) throw Error('Select gallery entries');
  const pkg = parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(await readRegular(input,PACKAGE_LIMIT)));
  if (pkg?.format !== 'repertica-studio-archive-relations-v2' || !Array.isArray(pkg.artworks) || !Array.isArray(pkg.collectors) || !Array.isArray(pkg.sales) || Object.hasOwn(pkg,'gallery'))
    throw Error('Expected v2 without gallery');
  if (Object.hasOwn(pkg,'covers') && !Array.isArray(pkg.covers)) throw Error('Invalid covers');
  if ((pkg.covers?.length ?? 0) + selections.length > 1000) throw Error('Too many photographs');
  const codes = new Set();
  for (const row of pkg.artworks) {
    const code = row?.code;
    if (typeof code !== 'string' || !code || code.trim() !== code || codes.has(code)) throw Error('Ambiguous artwork code');
    codes.add(code);
  }
  const positions = new Map();
  for (const item of selections) {
    if (!item || !codes.has(item.code) || !Number.isInteger(item.position) || item.position < 1 || item.position > 20 ||
        !KINDS.has(item.kind) || typeof item.path !== 'string' || typeof item.caption !== 'string' ||
        item.caption.includes('\0') || [...item.caption].length > 300) throw Error('Invalid gallery selection');
    if (!positions.has(item.code)) positions.set(item.code,new Set());
    const seen = positions.get(item.code);
    if (seen.has(item.position)) throw Error('Repeated gallery position');
    seen.add(item.position);
  }
  for (const seen of positions.values()) for (let i=1;i<=seen.size;i++) if (!seen.has(i)) throw Error('Noncontiguous gallery positions');
  let total = 0;
  pkg.gallery = [];
  for (const item of selections) {
    const bytes = await readRegular(item.path,COVER_LIMIT);
    total += bytes.length;
    if (total > PACKAGE_LIMIT || !hasImageHeader(bytes)) throw Error('Unsupported photograph');
    pkg.gallery.push({artwork_code:item.code,position:item.position,kind:item.kind,caption:item.caption.trim() || null,
      sha256:createHash('sha256').update(bytes).digest('hex'),data_base64:bytes.toString('base64')});
  }
  const result=JSON.stringify(pkg,null,2)+'\n';
  if (Buffer.byteLength(result,'utf8') > PACKAGE_LIMIT) throw Error('Combined package too large');
  await publishLocalPackage(output,result);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const [input,output,...args]=process.argv.slice(2);
    if (args.length % 5) throw Error('Use code position kind caption path groups');
    const selections=[];
    for(let i=0;i<args.length;i+=5) {
      const [code,position,kind,caption,path]=args.slice(i,i+5);
      if (!/^(?:[1-9]|1[0-9]|20)$/.test(position)) throw Error('Invalid position');
      selections.push({code,position:Number(position),kind,caption,path});
    }
    await addGallery(input,output,selections);
    console.log('Package written. Validate the full images in Studio preview before import.');
  } catch { console.error('Gallery package rejected. Check explicit selections, position, size limits and new output path.'); process.exitCode=1; }
}
