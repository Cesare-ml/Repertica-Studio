import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createAuctionManager } from '../src/app/auction-manager.ts';

const auction = { auctionId: 'auc_1', slug: 'paesaggio-01', title: 'Paesaggio', state: 'open',
  startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z', startingAmount: 12000,
  minimumIncrementAmount: 1000, currentAmount: null, leadingBidId: null };
const bid = (id, at, extra = {}) => ({ bidId: id, amount: 13000, placedAt: at, bidderEmail: 'synthetic@example.test', state: 'valid', ...extra });
const prepared = { artworkId: 'art-1', idempotencyKey: 'idem-0123456789abcdef', body: JSON.stringify({ slug: 'paesaggio-01', title: 'Paesaggio' }) };

function fake(over = {}) {
  const t = { calls: [], published: false, cancelled: false,
    async publish(body, key) { t.calls.push(['publish', key, body]); if (t.failPublish) throw new Error('network'); t.published = true; return { status: t.publishStatus ?? 201 }; },
    async listAuctions() { if (t.failList) throw new Error('network'); return { items: t.published ? [auction] : [] }; },
    async listBids(id, cursor) {
      if (t.failBids) throw new Error('network');
      if (cursor === null && t.pages) return { items: [bid('bid_a', '2027-03-04T11:00:00.000Z')], nextCursor: 'p2' };
      if (cursor === 'p2') return { items: [bid('bid_b', '2027-03-04T12:00:00.000Z', t.cancelled ? { state: 'cancelled', cancelledAt: '2027-03-04T13:00:00.000Z' } : {})], nextCursor: null };
      return { items: [bid('bid_b', '2027-03-04T12:00:00.000Z', t.cancelled ? { state: 'cancelled', cancelledAt: '2027-03-04T13:00:00.000Z' } : {})], nextCursor: null };
    },
    async cancelBid(a, b, r, key) { t.calls.push(['cancel', key]); if (t.failCancel) throw new Error('network'); if (!t.refuseCancel) t.cancelled = true;
      return { status: 200, body: { bidId: b, outcome: t.refuseCancel ? 'contested' : 'cancelled' } }; },
    ...over };
  return t;
}
const manager = (t, online = true) => createAuctionManager(t, () => online);

test('offline: nothing is attempted and no operation is created', async () => {
  const t = fake();
  await assert.rejects(manager(t, false).publish(prepared), /offline/);
  await assert.rejects(manager(t, false).loadAuctions(), /offline/);
  assert.equal(t.calls.length, 0);
});

test('publication is confirmed only after the server list shows it', async () => {
  const t = fake();
  const { operation } = await manager(t).publish(prepared);
  assert.equal(operation.state, 'confirmed');
  const lost = fake({ async listAuctions() { throw new Error('network'); } });
  assert.equal((await manager(lost).publish(prepared)).operation.state, 'pending_confirmation');
});

test('a lost response is recovered by the retry with the same key and body, without duplicates', async () => {
  const t = fake();
  t.failPublish = true;
  const first = await manager(t).publish(prepared);
  assert.equal(first.operation.state, 'pending_confirmation');
  t.failPublish = false;
  const second = await manager(t).publish(prepared, first.operation);
  assert.equal(second.operation.state, 'confirmed');
  assert.equal(second.operation.attempts, 2);
  assert.deepEqual(t.calls.map(c => c[1]), [prepared.idempotencyKey, prepared.idempotencyKey]);
  await assert.rejects(manager(t).publish({ ...prepared, body: '{"slug":"paesaggio-01","title":"Altro"}' }, first.operation), /request_changed/);
});

test('refusals and expired sessions are never reported as success', async () => {
  const t = fake({ async publish() { return { status: 401 }; } });
  const out = await manager(t).publish(prepared);
  assert.equal(out.needsSignIn, true);
  assert.equal(out.operation.state, 'rejected');
  const server = fake({ async publish() { return { status: 503 }; } });
  assert.equal((await manager(server).publish(prepared)).operation.state, 'pending_confirmation');
});

test('bid history follows cursors and rejects a cursor loop', async () => {
  const t = fake(); t.pages = true;
  assert.deepEqual((await manager(t).loadBids('auc_1')).map(b => b.bidId), ['bid_a', 'bid_b']);
  const loop = fake({ async listBids() { return { items: [], nextCursor: 'same' }; } });
  await assert.rejects(manager(loop).loadBids('auc_1'), /invalid_response/);
});

test('cancellation is confirmed by the reloaded history; contested and unreachable stay honest', async () => {
  const ok = fake();
  const done = await manager(ok).cancelBid('auc_1', 'bid_b', 'sospetta', 'idem-0123456789abcdef');
  assert.equal(done.outcome.operation.state, 'confirmed');
  assert.equal(done.bids.find(b => b.bidId === 'bid_b').state, 'cancelled');

  const contested = fake(); contested.refuseCancel = true;
  assert.equal((await manager(contested).cancelBid('auc_1', 'bid_b', null, 'idem-0123456789abcdef')).outcome.operation.state, 'rejected');

  const lost = fake(); lost.failCancel = true; lost.failBids = true;
  const unknown = await manager(lost).cancelBid('auc_1', 'bid_b', null, 'idem-0123456789abcdef');
  assert.equal(unknown.outcome.operation.state, 'pending_confirmation');
  assert.equal(unknown.bids, null);

  await assert.rejects(manager(ok).cancelBid('auc_1', 'bid_b', '   ', 'idem-0123456789abcdef'), /invalid_reason/);
});

test('a cancellation answered "cancelled" but not visible in the history is not a success', async () => {
  const t = fake({ async cancelBid() { return { status: 200, body: { bidId: 'bid_b', outcome: 'cancelled' } }; } });
  const out = await manager(t).cancelBid('auc_1', 'bid_b', null, 'idem-0123456789abcdef');
  assert.equal(out.outcome.operation.state, 'pending_confirmation');
});
