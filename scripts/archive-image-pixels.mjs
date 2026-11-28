// Header-only dimension preflight; native image decoding remains authoritative.
const invalid=()=>{throw Error('Unsupported or truncated image dimensions');};
export function archiveImagePixels(b) {
  let w,h;
  if(b.length>=33 && b.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) {
    if(b.readUInt32BE(8)!==13 || b.toString('ascii',12,16)!=='IHDR') invalid();
    w=b.readUInt32BE(16); h=b.readUInt32BE(20);
  } else if(b.length>=4 && b[0]===255 && b[1]===216) {
    let p=2;
    while(p<b.length) {
      if(b[p++]!==255) invalid();
      while(p<b.length && b[p]===255) p++;
      const marker=b[p++];
      if(marker===0 || marker===0xda || marker===0xd9 || p+2>b.length) invalid();
      const n=b.readUInt16BE(p);
      if(n<2 || p+n>b.length) invalid();
      if([0xc0,0xc1,0xc2,0xc3,0xc5,0xc6,0xc7,0xc9,0xca,0xcb,0xcd,0xce,0xcf].includes(marker)) {
        if(n<8 || n!==8+3*b[p+7] || !b[p+7]) invalid();
        h=b.readUInt16BE(p+3); w=b.readUInt16BE(p+5); break;
      }
      p+=n;
    }
  } else if(b.length>=20 && b.toString('ascii',0,4)==='RIFF' && b.toString('ascii',8,12)==='WEBP') {
    if(b.readUInt32LE(4)+8!==b.length) invalid();
    const n=b.readUInt32LE(16), kind=b.toString('ascii',12,16);
    if(20+n+(n%2)>b.length) invalid();
    if(kind==='VP8X' && n===10) {
      w=1+b.readUIntLE(24,3); h=1+b.readUIntLE(27,3);
    } else if(kind==='VP8L' && n>=5 && b[20]===0x2f && !(b[24]&0xe0)) {
      const v=b.readUInt32LE(21); w=(v&0x3fff)+1; h=((v>>>14)&0x3fff)+1;
    } else if(kind==='VP8 ' && n>=10 && !(b[20]&1) && b.subarray(23,26).equals(Buffer.from([0x9d,1,0x2a]))) {
      w=b.readUInt16LE(26)&0x3fff; h=b.readUInt16LE(28)&0x3fff;
    } else invalid();
  } else invalid();
  if(!w || !h || w>16_000_000 || h>Math.floor(16_000_000/w)) invalid();
  return w*h;
}
export function addArchiveImagePixels(total,b) {
  const pixels=archiveImagePixels(b);
  if(!Number.isSafeInteger(total) || total<0 || total+pixels>128_000_000) throw Error('Selected media exceeds package pixel bounds');
  return total+pixels;
}
