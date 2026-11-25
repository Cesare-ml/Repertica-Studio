// Associate verified active LOCAL blobs from an explicit offline copy.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstat, realpath } from 'node:fs/promises';
import { isAbsolute, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { exportOptions, localEnvironment } from './export-gestionale-artwork-metadata.mjs';
import { readRegular, hasImageHeader, PACKAGE_LIMIT, COVER_LIMIT } from './add-local-artwork-covers.mjs';
import { parseLegacyJson } from './strict-legacy-json.mjs';
import { publishLocalPackage } from './publish-local-package.mjs';
import { publishArchiveMediaBatches } from './archive-media-batches.mjs';

const KINDS=new Map([['DETTAGLIO','DETAIL'],['FIRMA','SIGNATURE'],['DETAIL','DETAIL'],['SIGNATURE','SIGNATURE'],
  ['DIGITAL_MOCKUP','DIGITAL_MOCKUP'],['IN_SITU','IN_SITU'],['FRAMED','FRAMED'],['WORK_IN_PROGRESS','WORK_IN_PROGRESS'],['OTHER','OTHER']]);
const decode=bytes=>parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
const exact=(value,fields)=>value && typeof value==='object' && !Array.isArray(value) &&
  Object.keys(value).length===fields.length && fields.every(k=>Object.hasOwn(value,k));
async function selectedFile(root,key) {
  if(typeof key!=='string'||!key||key.length>1024) throw Error('Invalid local blob key');
  const segments=key.split('/');
  if(segments.some(p=>!p||p==='.'||p==='..'||/[\\:\x00-\x1f<>|?*"]/.test(p)||/[. ]$/.test(p)||
      /^(?:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\.|$)/i.test(p))) throw Error('Invalid local blob key');
  let path=root;
  for(let i=0;i<segments.length;i++) {
    path=join(path,segments[i]);
    const st=await lstat(path);
    if(st.isSymbolicLink()||(i===segments.length-1?!st.isFile():!st.isDirectory())||await realpath(path)!==path)
      throw Error('Unsafe local blob path');
  }
  return path;
}
export async function attachLocalMedia(pkg,rows,mediaRoot) {
  if(!pkg||pkg.format!=='repertica-studio-archive-relations-v2'||!Array.isArray(pkg.artworks)||
      !pkg.artworks.length||pkg.artworks.length>1000||!Array.isArray(pkg.collectors)||!Array.isArray(pkg.sales)||
      Object.hasOwn(pkg,'covers')||Object.hasOwn(pkg,'gallery')) throw Error('Expected unaugmented v2 package');
  const codes=new Set();
  for(const item of pkg.artworks) {
    if(typeof item?.code!=='string'||!item.code||item.code.trim()!==item.code||codes.has(item.code))
      throw Error('Ambiguous artwork code');
    codes.add(item.code);
  }
  if(!Array.isArray(rows)||rows.length!==codes.size) throw Error('Source selection differs from package');
  if(!isAbsolute(mediaRoot)||!(await lstat(mediaRoot)).isDirectory()) throw Error('Select trusted absolute media directory');
  const root=await realpath(mediaRoot),seen=new Set(),covers=[],gallery=[],plan=[];
  let count=0,total=0;
  for(const row of rows) {
    if(!exact(row,['code','photos'])||!codes.has(row.code)||seen.has(row.code)||
        !Array.isArray(row.photos)||row.photos.length>21) throw Error('Invalid source media association');
    seen.add(row.code);
    let next=1,cover=false;
    for(const photo of row.photos) {
      if(!exact(photo,['position','kind','caption','path','sha256','size'])||
          !Number.isInteger(photo.position)||photo.position<0||photo.position>20||
          typeof photo.sha256!=='string'||!/^[a-f0-9]{64}$/.test(photo.sha256)||
          typeof photo.size!=='string'||!/^[1-9]\d{0,7}$/.test(photo.size)||Number(photo.size)>COVER_LIMIT||
          typeof photo.path!=='string'||!photo.path) throw Error('Unavailable or invalid registered local blob');
      if(photo.position===0) {
        if(cover||next!==1||photo.kind!==null||photo.caption!==null) throw Error('Invalid cover association');
        cover=true;
      } else if(photo.position!==next++||!KINDS.has(photo.kind)||
          (photo.caption!==null&&(typeof photo.caption!=='string'||photo.caption.includes('\0')||[...photo.caption].length>300)))
        throw Error('Unsupported or ambiguous gallery association');
      count++;total+=Number(photo.size);
      if(count>1000||total>PACKAGE_LIMIT) throw Error('Selected media exceeds package bounds');
      plan.push({code:row.code,photo});
    }
  }
  for(const {code,photo} of plan) {
    const bytes=await readRegular(await selectedFile(root,photo.path),COVER_LIMIT);
    if(bytes.length!==Number(photo.size)||!hasImageHeader(bytes)||createHash('sha256').update(bytes).digest('hex')!==photo.sha256)
      throw Error('Local blob does not match registered bytes');
    const entry={artwork_code:code,sha256:photo.sha256,data_base64:bytes.toString('base64')};
    if(photo.position===0) covers.push(entry);
    else gallery.push({...entry,position:photo.position,kind:KINDS.get(photo.kind),caption:photo.caption});
  }
  const output=JSON.stringify({...pkg,covers,gallery},null,2)+'\n';
  if(Buffer.byteLength(output,'utf8')>PACKAGE_LIMIT) throw Error('Combined package exceeds 16 MiB');
  return output;
}
export async function exportArtworkMedia(args) {
  const batches=args.length===10&&args[9]==='--batch-directory';
  if(args.length!==9&&!batches) throw Error('Expected local copy, selected IDs, package, media root and new output');
  const options=exportOptions([...args.slice(0,6),args[8]]);
  const pkg=decode(await readRegular(args[6],PACKAGE_LIMIT));
  options.psqlArgs[options.psqlArgs.length-1]=fileURLToPath(new URL('./gestionale-artwork-media.sql',import.meta.url));
  const result=spawnSync('psql',options.psqlArgs,{env:localEnvironment(process.env),timeout:15000,maxBuffer:2*1024*1024,windowsHide:true});
  if(result.error||result.status!==0) throw Error('Local media projection failed');
  const rows=decode(result.stdout);
  if(batches) await publishArchiveMediaBatches(options.output,pkg,rows,(p,r)=>attachLocalMedia(p,r,args[7]));
  else await publishLocalPackage(options.output,await attachLocalMedia(pkg,rows,args[7]));
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href) {
  try {
    await exportArtworkMedia(process.argv.slice(2));
    console.log('Media export written. Validate each complete package in Studio preview.');
  } catch {
    console.error('Media export rejected. Check the isolated copy, selected works, registered LOCAL blobs and new output path.');
    process.exitCode=1;
  }
}
