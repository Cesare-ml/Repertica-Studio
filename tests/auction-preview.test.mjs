import { test } from 'node:test';
import assert from 'node:assert/strict';
import { newAuctionDraft, preparePreview } from '../src/app/auction-preview.ts';

const now = Date.parse('2027-03-01T00:00:00.000Z');
const key = 'preview-0123456789';
const artwork = { id: 'local-private-id', title: 'Paesaggio', status: 'available',
  description: 'private notes', location: 'private address', image_path: 'media/private.jpg',
  collector_id: 'private-buyer', fixed_price_cents: 999 };
const draft = () => ({ ...newAuctionDraft(artwork), slug: 'paesaggio-01', description: 'Olio su tela',
  startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z',
  startingAmount: 12000, minimumIncrementAmount: 1000 });
const prepare = d => preparePreview(d, 'tenant-test', key, now);

test('offline draft selects exactly one available work and excludes private defaults', () => {
  assert.deepEqual(newAuctionDraft(artwork), { artworkId: artwork.id, title: artwork.title,
    description: '', slug: '', startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: null });
  for (const status of ['sold', 'reserved', 'archived']) {
    assert.throws(() => newAuctionDraft({ ...artwork, status }), /artwork_not_available/);
  }
});

test('request body uses only the nine documented public fields even with extra draft data', () => {
  const prepared = prepare({ ...artwork, ...draft(), accessToken: 'private-token' });
  assert.deepEqual(JSON.parse(prepared.body), { tenantId: 'tenant-test', slug: 'paesaggio-01', title: 'Paesaggio',
    description: 'Olio su tela', startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z',
    startingAmount: 12000, minimumIncrementAmount: 1000, currency: 'EUR' });
  assert.equal(prepared.artworkId, artwork.id);
  assert.doesNotMatch(prepared.body, /private|artworkId|accessToken/);
});

test('frozen request can be serialized locally and replayed unchanged after edits or restart', () => {
  const d = draft();
  const prepared = prepare(d);
  const saved = JSON.stringify(prepared);
  d.title = 'Edited later';
  const recovered = JSON.parse(saved);
  assert.equal(recovered.body, prepared.body);
  assert.equal(recovered.idempotencyKey, key);
  assert.equal(JSON.parse(recovered.body).title, 'Paesaggio');
  assert.ok(Object.isFrozen(prepared));
});

test('invalid tenant, key and incomplete public fields are rejected', () => {
  assert.throws(() => preparePreview(draft(), 'a', key, now), /invalid_tenant/);
  assert.throws(() => preparePreview(draft(), 'tenant-test', 'short', now), /invalid_idempotency_key/);
  assert.throws(() => prepare({ ...draft(), slug: 'Bad slug' }), /invalid_slug/);
  assert.throws(() => prepare({ ...draft(), title: '  a  ' }), /invalid_title/);
  assert.throws(() => prepare({ ...draft(), description: 'x'.repeat(1501) }), /invalid_description/);
});

test('rejects ambiguous UTC timestamps, rollover dates, past starts and excessive duration', () => {
  for (const startsAt of ['2027-03-04T10:00:00', '2027-02-30T10:00:00.000Z', '2027-02-28T00:00:00.000Z']) {
    assert.throws(() => prepare({ ...draft(), startsAt }), /invalid_schedule/);
  }
  for (const endsAt of ['2027-03-04T10:00:00.000Z', '2027-03-11T10:00:00.001Z']) {
    assert.throws(() => prepare({ ...draft(), endsAt }), /invalid_schedule/);
  }
  assert.doesNotThrow(() => prepare({ ...draft(), endsAt: '2027-03-11T10:00:00.000Z' }));
});

test('amounts are explicit positive integer cents within the beta bounds', () => {
  for (const value of [null, 0, -1, 1.5, Infinity, 1_000_000_001]) {
    assert.throws(() => prepare({ ...draft(), startingAmount: value }), /invalid_amount/);
    assert.throws(() => prepare({ ...draft(), minimumIncrementAmount: value }), /invalid_amount/);
  }
  assert.doesNotThrow(() => prepare({ ...draft(), startingAmount: 1_000_000_000, minimumIncrementAmount: 1 }));
});

test('checks the encoded JSON size, not just the description character count', () => {
  assert.throws(() => prepare({ ...draft(), description: '\u0000'.repeat(1500) }), /payload_too_large/);
  assert.doesNotThrow(() => prepare({ ...draft(), description: '作品'.repeat(500) }));
});
