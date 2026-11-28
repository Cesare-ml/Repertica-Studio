import test from 'node:test';
import assert from 'node:assert/strict';
import { loadCertificatePreview } from '../src/app/certificate-preview.ts';

const original = { id: 'a', title: 'Original title', code: 'ART-1', image_path: 'media/old.jpg' };
const load = (snapshot, read, artworkId = 'a') => loadCertificatePreview(JSON.stringify(snapshot), artworkId, read);

test('historical photo and metadata come from the issued snapshot', async () => {
  const image = 'data:image/jpeg;base64,old';
  assert.deepEqual(await load(original, async () => image), { snapshot: original, image });
});
test('an issued photo cannot silently become a placeholder', async () => {
  await assert.rejects(load(original, async () => null), /mismatch/);
  await assert.rejects(load(original, async () => { throw new Error('missing'); }), /missing/);
});
test('certificates issued without a photo may print, but unexpected photos are rejected', async () => {
  for (const snapshot of [{ ...original, image_path: null }, { id: 'a', title: 'T', code: 'C' }]) {
    assert.equal((await load(snapshot, async () => null)).image, null);
    await assert.rejects(load(snapshot, async () => 'unexpected'), /mismatch/);
  }
});
test('invalid snapshot or mismatched artwork never calls the photo reader', async () => {
  let calls = 0;
  const read = async () => { calls++; return null; };
  for (const snapshot of [null, [], {}, { ...original, image_path: 42 }, { ...original, image_path: ' ' }]) {
    await assert.rejects(load(snapshot, read), /Invalid snapshot/);
  }
  await assert.rejects(load(original, read, 'different'), /Invalid snapshot/);
  await assert.rejects(loadCertificatePreview('{', 'a', read), SyntaxError);
  assert.equal(calls, 0);
});

test('malformed printable metadata is rejected before reading any historical image', async () => {
  let calls = 0;
  const read = async () => { calls++; return 'data:image/jpeg;base64,old'; };
  for (const patch of [{ technique: 42 }, { support: {} }, { year: [] }, { height_cm: '40' }, { width_cm: -1 }, { depth_cm: {} }]) {
    await assert.rejects(load({ ...original, ...patch }, read), /Invalid snapshot/);
  }
  const overflow = JSON.stringify(original).replace(/}$/, ',"width_cm":1e309}');
  await assert.rejects(loadCertificatePreview(overflow, 'a', read), /Invalid snapshot/);
  assert.equal(calls, 0);
});
test('optional historical metadata remains compatible and is never rewritten', async () => {
  const snapshot = { ...original, image_path: null, year: '2020', technique: '  Olio  ', support: null, width_cm: 40.5, height_cm: 30, depth_cm: null };
  assert.deepEqual(await load(snapshot, async () => null), { snapshot, image: null });
});
