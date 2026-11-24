import { test } from 'node:test';
import assert from 'node:assert/strict';
import { newAuctionDraft, preparePreview, loadAuctionDraft, saveAuctionDraft } from '../src/app/auction-preview.ts';

const scope = 'archive-one';
const artwork = { id: 'work-one', title: 'Paesaggio', status: 'available' };
const blank = () => newAuctionDraft(artwork);
const complete = () => ({ ...blank(), slug: 'paesaggio', startsAt: '2025-01-02T10:00:00.000Z',
  endsAt: '2025-01-03T10:00:00.000Z', startingAmount: 10000, minimumIncrementAmount: 500 });
const prepare = (draft = complete(), key = 'preview-0123456789') =>
  preparePreview(draft, 'tenant-test', key, Date.parse(draft.startsAt));
function memory() {
  const records = new Map();
  return { records, getItem: key => records.get(key) ?? null, setItem: (key, value) => records.set(key, value) };
}

test('unfinished drafts round-trip offline and are isolated by archive and artwork', () => {
  const storage = memory();
  const draft = { ...blank(), description: 'Bozza', accessToken: 'never persist', collector_id: 'private' };
  saveAuctionDraft(storage, scope, draft);
  assert.deepEqual(loadAuctionDraft(storage, scope, artwork.id), { draft: { ...blank(), description: 'Bozza' }, pending: null });
  assert.equal(loadAuctionDraft(storage, 'archive-two', artwork.id), null);
  assert.equal(loadAuctionDraft(storage, scope, 'work-two'), null);
  assert.doesNotMatch([...storage.records.values()][0], /accessToken|collector_id|never persist/);
});

test('pending body and key survive restart, past schedule and subsequent draft edits', () => {
  const storage = memory(), draft = complete(), attempt = prepare(draft);
  saveAuctionDraft(storage, scope, draft, attempt);
  const restarted = memory();
  restarted.records.set(...[...storage.records.entries()][0]);
  saveAuctionDraft(restarted, scope, { ...draft, title: 'Nuovo titolo' });
  const recovered = loadAuctionDraft(restarted, scope, artwork.id);
  assert.equal(recovered.draft.title, 'Nuovo titolo');
  assert.deepEqual(recovered.pending, attempt);
  assert.ok(Object.isFrozen(recovered.pending));
});

test('unresolved attempts cannot be replaced and first attempt must match reviewed draft', () => {
  const storage = memory(), draft = complete(), attempt = prepare(draft);
  const edited = { ...draft, title: 'Nuovo titolo' };
  assert.throws(() => saveAuctionDraft(storage, scope, edited, attempt), /preview_draft_changed/);
  assert.equal(storage.records.size, 0);
  saveAuctionDraft(storage, scope, draft, attempt);
  const before = [...storage.records.values()][0];
  assert.throws(() => saveAuctionDraft(storage, scope, edited, prepare(edited, 'preview-9876543210')), /pending_preview_unresolved/);
  assert.equal([...storage.records.values()][0], before);
  assert.doesNotThrow(() => saveAuctionDraft(storage, scope, edited, attempt));
});

test('corrupt stored attempts fail closed without replacing the original bytes', () => {
  const storage = memory(), draft = complete();
  saveAuctionDraft(storage, scope, draft, prepare(draft));
  const [key, raw] = [...storage.records.entries()][0];
  for (const corrupt of [
    { ...JSON.parse(raw), version: 2 },
    { ...JSON.parse(raw), scope: 'other' },
    { ...JSON.parse(raw), pending: { ...prepare(draft), artworkId: 'other' } },
    { ...JSON.parse(raw), pending: { ...prepare(draft), body: JSON.stringify({ ...JSON.parse(prepare(draft).body), accessToken: 'secret' }) } }
  ]) {
    const value = JSON.stringify(corrupt);
    storage.records.set(key, value);
    assert.throws(() => loadAuctionDraft(storage, scope, artwork.id));
    assert.throws(() => saveAuctionDraft(storage, scope, draft));
    assert.equal(storage.records.get(key), value);
  }
});

test('storage errors are propagated and empty archive scopes are rejected', () => {
  const storage = { getItem: () => null, setItem: () => { throw new Error('disk full'); } };
  assert.throws(() => saveAuctionDraft(storage, scope, complete(), prepare()), /disk full/);
  assert.throws(() => loadAuctionDraft(storage, '', artwork.id), /invalid_draft_scope/);
});
