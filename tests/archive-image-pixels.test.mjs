import test from 'node:test';
import assert from 'node:assert/strict';
import {archiveImagePixels,addArchiveImagePixels} from '../scripts/archive-image-pixels.mjs';
function png(w,h) {
  const b=Buffer.alloc(33); Buffer.from([137,80,78,71,13,10,26,10]).copy(b);
  b.writeUInt32BE(13,8); b.write('IHDR',12); b.writeUInt32BE(w,16); b.writeUInt32BE(h,20); return b;
}
function jpeg(w,h) {
  const b=Buffer.from([255,216,255,225,0,4,0,0,255,194,0,11,8,0,0,0,0,1,1,0x11,0]);
  b.writeUInt16BE(h,13); b.writeUInt16BE(w,15); return b;
}
function webp(kind,payload) {
  const b=Buffer.alloc(20+payload.length+(payload.length%2));
  b.write('RIFF'); b.writeUInt32LE(b.length-8,4); b.write('WEBP',8); b.write(kind,12); b.writeUInt32LE(payload.length,16); payload.copy(b,20); return b;
}
test('reads bounded PNG and JPEG headers, including JPEG APP segments',()=>{
  assert.equal(archiveImagePixels(png(4000,4000)),16_000_000);
  assert.equal(archiveImagePixels(jpeg(600,800)),480_000);
  for(const b of [png(0,2),png(4001,4000),png(0xffffffff,0xffffffff),jpeg(65535,65535),jpeg(1,1).subarray(0,18),png(1,1).subarray(0,16)]) assert.throws(()=>archiveImagePixels(b));
});
test('reads three WebP bitstream headers and rejects malformed lengths',()=>{
  const x=Buffer.alloc(10); x.writeUIntLE(299,4,3); x.writeUIntLE(199,7,3);
  const l=Buffer.alloc(5); l[0]=0x2f; l.writeUInt32LE(299|(199<<14),1);
  const p=Buffer.alloc(10); Buffer.from([0x9d,1,0x2a]).copy(p,3); p.writeUInt16LE(300,6); p.writeUInt16LE(200,8);
  for(const [kind,data] of [['VP8X',x],['VP8L',l],['VP8 ',p]]) {
    const b=webp(kind,data); assert.equal(archiveImagePixels(b),60_000); assert.throws(()=>archiveImagePixels(b.subarray(0,-1)));
  }
  x.writeUIntLE(0xffffff,4,3); assert.throws(()=>archiveImagePixels(webp('VP8X',x)));
  l[4]|=0x20; assert.throws(()=>archiveImagePixels(webp('VP8L',l)));
});
test('aggregate pixel budget matches native limits, independently of compressed size',()=>{
  let total=0;
  for(let i=0;i<8;i++) total=addArchiveImagePixels(total,png(4000,4000));
  assert.equal(total,128_000_000);
  assert.throws(()=>addArchiveImagePixels(total,png(1,1)));
  assert.throws(()=>addArchiveImagePixels(-1,png(1,1)));
});
