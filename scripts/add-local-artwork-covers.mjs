// Explicit local selections only; never resolve a path from the input package.
import { createHash } from 'node:crypto';
import { open, lstat } from 'node:fs/promises';
import { constants } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { parseLegacyJson } from './strict-legacy-json.mjs';
import { publishLocalPackage } from './publish-local-package.mjs';

export const PACKAGE_LIMIT = 16 * 1024 * 1024;
export const COVER_LIMIT = 8 * 1024 * 1024;
export async function readRegular(path, limit) {
  const before = await lstat(path);
  if (!before.isFile() || before.size < 1 || before.size > limit) throw Error('Invalid local file');
  const f = await open(path, constants.O_RDONLY | (constants.O_NOFOLLOW || 0));
  try {
    const meta = await f.stat();
    if (!meta.isFile() || meta.dev !== before.dev || meta.ino !== before.ino || meta.size !== before.size)
      throw Error('Selected file changed');
    const buf = Buffer.alloc(meta.size + 1);
    let offset = 0;
    while (offset < buf.length) {
      const { bytesRead } = await f.read(buf, offset, buf.length - offset, null);
      if (!bytesRead) break;
      offset += bytesRead;
    }
    const after = await f.stat();
    if (offset !== meta.size || after.size !== meta.size || after.mtimeMs !== meta.mtimeMs || after.ctimeMs !== meta.ctimeMs)
      throw Error('Selected file changed');
    return buf.subarray(0, offset);
  } finally { await f.close(); }
}
export function hasImageHeader(bytes) {
  return bytes.subarray(0,3).equals(Buffer.from([255,216,255])) ||
    bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])) ||
    (bytes.length >= 12 && bytes.toString('ascii',0,4)==='RIFF' && bytes.toString('ascii',8,12)==='WEBP');
}
export async function addCovers(input, output, pairs) {
  if (!Array.isArray(pairs) || pairs.length < 2 || pairs.length % 2 || pairs.length > 2000) throw Error('Select code/path pairs');
  const pkg = parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(await readRegular(input,PACKAGE_LIMIT)));
  if (pkg?.format !== 'repertica-studio-archive-relations-v2' || !Array.isArray(pkg.artworks) || !Array.isArray(pkg.collectors) || !Array.isArray(pkg.sales) || Object.hasOwn(pkg,'covers'))
    throw Error('Expected v2 without covers');
  if (Object.hasOwn(pkg,'gallery') && !Array.isArray(pkg.gallery)) throw Error('Invalid gallery');
  if ((pkg.gallery?.length ?? 0) + pairs.length / 2 > 1000) throw Error('Too many photographs');
  const codes = new Set();
  for (const row of pkg.artworks) {
    const code = row?.code;
    if (typeof code !== 'string' || !code || code.trim()!==code || codes.has(code)) throw Error('Ambiguous artwork code');
    codes.add(code);
  }
  const used = new Set();
  let total = 0;
  pkg.covers = [];
  for (let i=0;i<pairs.length;i+=2) {
    const [code,path] = [pairs[i],pairs[i+1]];
    if (!codes.has(code) || used.has(code)) throw Error('Unknown or repeated artwork code');
    used.add(code);
    const bytes = await readRegular(path,COVER_LIMIT);
    total += bytes.length;
    if (total > PACKAGE_LIMIT || !hasImageHeader(bytes)) throw Error('Unsupported cover');
    pkg.covers.push({artwork_code:code,sha256:createHash('sha256').update(bytes).digest('hex'),data_base64:bytes.toString('base64')});
  }
  const result = JSON.stringify(pkg,null,2)+'\n';
  if (Buffer.byteLength(result,'utf8') > PACKAGE_LIMIT) throw Error('Combined package too large');
  await publishLocalPackage(output,result);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const [input,output,...pairs]=process.argv.slice(2);
    await addCovers(input,output,pairs);
    console.log('Package written. Validate the full images in Studio preview before import.');
  } catch { console.error('Cover package rejected. Check explicit selections, size limits and new output path.'); process.exitCode=1; }
}
