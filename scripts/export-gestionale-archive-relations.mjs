// Opt-in exporter. Read only an explicitly selected local copy; no production access.
import { spawnSync } from 'node:child_process';
import { open } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { exportOptions, localEnvironment } from './export-gestionale-artwork-metadata.mjs';
import { convertLegacyRows } from './convert-gestionale-artwork-metadata.mjs';
import { parseLegacyJson } from './strict-legacy-json.mjs';
import { publishLocalPackage } from './publish-local-package.mjs';
import { money, date, unpaidInstallments } from './gestionale-commercial-values.mjs';

const MAX_BYTES=2*1024*1024;
function method(value) {
  const map=new Map([['bonifico','bank_transfer'],['contanti','cash'],['carta','card'],['altro','other']]);
  const result=typeof value==='string' ? map.get(value.trim().toLowerCase()) : undefined;
  if (!result) throw Error('Unknown receipt method; resolve explicitly before export');
  return result;
}
function exact(obj,keys) {
  if (!obj || typeof obj!=='object' || Array.isArray(obj) ||
      Object.keys(obj).length!==keys.length || !keys.every(k=>Object.hasOwn(obj,k)))
    throw Error('Invalid source projection');
}
function shareIndex(shares) {
  if (!Array.isArray(shares) || shares.length>1000) throw Error('Expected explicit historical shares array');
  const result=new Map();
  for(const row of shares) {
    exact(row,['artwork_code','legacy_shares_bp']);
    const {artwork_code:code,legacy_shares_bp:bp}=row;
    if(typeof code!=='string'||!code||result.has(code)||!Array.isArray(bp)||bp.length!==3||
       !bp.every(x=>Number.isInteger(x)&&x>=0&&x<=10000)||bp.reduce((a,b)=>a+b,0)!==10000)
      throw Error('Invalid or duplicate historical shares');
    result.set(code,bp);
  }
  return result;
}
function relations(rows,shares,convertMetadata) {
  if (!Array.isArray(rows)||!rows.length||rows.length>1000) throw Error('Selection rejected; nothing exported');
  const splits=shareIndex(shares), collectors=new Map(), sales=[], soldCodes=new Set();
  const source=rows.map(row=>{
    exact(row,['artwork','collector','sale']);
    const a=row.artwork;
    if(!a || a.id_vendita!==null) throw Error('Invalid source artwork');
    if(a.stato!=='VENDUTA') {
      if(row.sale!==null||row.collector!==null) throw Error('Unexpected commercial link');
      return a;
    }
    const profileFields=["birth_date","birth_time","birth_place","instagram","facebook","tiktok","website","linkedin"];
    const requiredCollectorFields=['collector_key','first_name','last_name','email','phone','address','notes'];
    if(!row.collector || typeof row.collector!=='object' || Array.isArray(row.collector) ||
      !requiredCollectorFields.every(k=>Object.hasOwn(row.collector,k)) ||
      !Object.keys(row.collector).every(k=>requiredCollectorFields.includes(k)||profileFields.includes(k)))
      throw Error('Invalid source collector projection');
    exact(row.sale,['amount','tax','shipping','sold_at','notes','receipts','installments']);
    const c=row.collector, v=row.sale, code=a.codice_archivio?.trim();
    if(typeof c.collector_key!=='string'||!/^[1-9]\d{0,9}$/.test(c.collector_key)||
       typeof c.first_name!=='string'||!c.first_name.trim()||
       !splits.has(code)||soldCodes.has(code)||!Array.isArray(v.receipts))
      throw Error('Unresolved commercial relationship or missing historical shares');
    soldCodes.add(code);
    const {collector_key:key,...details}=c;
    const normalized={key,...details};
    for(const field of profileFields) {
      const value=details[field]??null;
      if(value!==null && typeof value!=='string') throw Error('Invalid collector profile');
      normalized[field]=value;
    }
    if(normalized.birth_date!==null) normalized.birth_date=date(normalized.birth_date);
    if(normalized.birth_time!==null) {
      // A legacy time may include seconds; only exact minute precision is representable.
      if(!/^(?:[01]\d|2[0-3]):[0-5]\d(?::00(?:\.0+)?)?$/.test(normalized.birth_time))
        throw Error('Birth time cannot be represented exactly');
      normalized.birth_time=normalized.birth_time.slice(0,5);
    }
    for(const [field,limit] of Object.entries({birth_place:300,instagram:300,facebook:300,tiktok:300,website:500,linkedin:500})) {
      const value=normalized[field];
      if(value!==null && ([...value.trim()].length>limit || value.includes('\0'))) throw Error('Invalid collector profile');
    }
    const previous=collectors.get(key);
    if(previous && JSON.stringify(previous)!==JSON.stringify(normalized)) throw Error('Conflicting collector details');
    collectors.set(key,normalized);
    const amount=money(v.amount,true),tax=money(v.tax),shipping=money(v.shipping);
    if(tax+shipping>amount) throw Error('Costs exceed sale');
    const installments=unpaidInstallments(v.installments,v.receipts,amount);
    let total=0;
    const receipts=v.receipts.map(i=>{
      exact(i,['amount','received_at','method','notes']);
      const amount_cents=money(i.amount,true);
      total+=amount_cents;
      if(total>amount) throw Error('Receipts exceed sale');
      return {amount_cents,received_at:date(i.received_at),method:method(i.method),reference:null,notes:i.notes};
    });
    sales.push({artwork_code:code,collector_key:key,amount_cents:amount,currency:'EUR',sold_at:date(v.sold_at),
      notes:v.notes,tax_cents:tax,shipping_cents:shipping,legacy_shares_bp:splits.get(code),allocations:[],installments,receipts});
    return {...a,stato:'DISPONIBILE'};
  });
  if(splits.size!==soldCodes.size || sales.reduce((n,v)=>n+v.receipts.length,0)>1000 ||
     sales.reduce((n,v)=>n+v.installments.length,0)>1000)
    throw Error('Unused historical shares or too many payments or installments');
  const artworks=JSON.parse(convertMetadata(source)).artworks;
  for(const a of artworks) if(soldCodes.has(a.code)) a.status='sold';
  return {format:'repertica-studio-archive-relations-v2',artworks,collectors:[...collectors.values()],sales};
}

export function checkedRelationalPackage(bytes, shares) {
  if (!(bytes instanceof Uint8Array) || bytes.byteLength>MAX_BYTES) throw Error('Source exceeds 2 MiB');
  const rows=parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
  const output=JSON.stringify(relations(rows,shares,convertLegacyRows),null,2)+'\n';
  if(Buffer.byteLength(output,'utf8')>MAX_BYTES) throw Error('Package exceeds 2 MiB');
  return output;
}
async function readShares(path) {
  const f=await open(path,'r');
  try {
    if(!(await f.stat()).isFile()) throw Error('Expected regular shares file');
    const bytes=Buffer.alloc(MAX_BYTES+1);
    let offset=0;
    while(offset<bytes.length) {
      const {bytesRead}=await f.read(bytes,offset,bytes.length-offset,null);
      if(!bytesRead) break;
      offset+=bytesRead;
    }
    if(offset>MAX_BYTES) throw Error('Shares file exceeds 2 MiB');
    return parseLegacyJson(new TextDecoder('utf-8',{fatal:true}).decode(bytes.subarray(0,offset)));
  } finally {await f.close();}
}
export async function exportRelations(args) {
  if(args.length!==8) throw Error('Usage: node scripts/export-gestionale-archive-relations.mjs --local-copy DATABASE USER PORT OWNER_ID IDS SHARES.json NEW_OUTPUT.json');
  const options=exportOptions([...args.slice(0,6),args[7]]);
  const shares=await readShares(args[6]);
  shareIndex(shares);
  options.psqlArgs[options.psqlArgs.length-1]=fileURLToPath(new URL('./gestionale-archive-relations.sql',import.meta.url));
  const result=spawnSync('psql',options.psqlArgs,{env:localEnvironment(process.env),timeout:15000,maxBuffer:MAX_BYTES+65536,windowsHide:true});
  if(result.error||result.status!==0) throw Error('Local PostgreSQL export failed; check the local copy and read-only permissions');
  await publishLocalPackage(options.output,checkedRelationalPackage(result.stdout,shares));
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href) {
  try {await exportRelations(process.argv.slice(2)); console.log('Relational package written. Review in Studio before import.');}
  catch {console.error('Relational export rejected. Check selection, unsupported fields, explicit historical shares and new output path. No existing output replaced.');process.exitCode=1;}
}
