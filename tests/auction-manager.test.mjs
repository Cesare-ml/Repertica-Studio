import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createAuctionManager } from '../src/app/auction-manager.ts';

// Shapes of contract artist-v1 (fixtures captured from the real server are exercised in auction-http-transport.test.mjs).
const auction = { auctionId: 'auc_1', tenantId: 'tenant-a', slug: 'paesaggio-01', title: 'Paesaggio', state: 'live',
  startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z', maxEndsAt: '2027-03-06T10:00:00.000Z',
  currency: 'EUR', startingAmount: 12000, minimumIncrementAmount: 1000, reserveAmount: null, reserveMet: false,
  currentAmount: null, leadingBidId: null, minimumNextBidAmount: 12000, bidCount: 0, cancelledBidCount: 0,
  winnerBidId: null, winningAmount: null, paymentRecorded: false, studioArtworkId: null, version: 0 };
const bid = (id, at, extra = {}) => ({ bidId: id, amount: 13000, placedAt: at, bidderLabel: '#1',
  bidderEmail: 'synthetic@example.test', state: 'valid', isLeading: false, cancelledAt: null, cancelReason: null, ...extra });
const cancelledBid = (id, at) => bid(id, at, { state: 'cancelled', cancelledAt: '2027-03-04T13:00:00.000Z', cancelReason: 'sospetta' });
const prepared = { artworkId: 'art-1', auctionId: 'auc-idem-0123456789abcdef', idempotencyKey: 'idem-0123456789abcdef',
  // The site chooses the page address: the publication is recognised by the auction id it carries.
  body: JSON.stringify({ auction: { id: 'auc_1' }, details: { title: 'Paesaggio' } }) };
const KEY = 'idem-0123456789abcdef';

function fake(over = {}) {
  const t = { calls: [], published: false, cancelled: false,
    async publish(body, key) { t.calls.push(['publish', key, body]); if (t.failPublish) throw new Error('network'); t.published = true; return { status: t.publishStatus ?? 201 }; },
    async listAuctions() { if (t.failList) throw new Error('network'); return { items: t.published ? [auction] : [] }; },
    async listBids(id, cursor) {
      if (t.failBids) throw new Error('network');
      const b = t.cancelled ? cancelledBid('bid_b', '2027-03-04T12:00:00.000Z') : bid('bid_b', '2027-03-04T12:00:00.000Z');
      if (cursor === null && t.pages) return { items: [bid('bid_a', '2027-03-04T11:00:00.000Z')], nextCursor: '7' };
      return { items: [b], nextCursor: null };
    },
    async cancelBid(a, b, r, key) { t.calls.push(['cancel', key, r]); if (t.failCancel) throw new Error('network');
      if (t.refuseCancel) return { status: 409, body: { bidId: b, outcome: 'not_allowed', reason: 'auction_closed' } };
      t.cancelled = true; return { status: 201, body: { bidId: b, outcome: 'cancelled', reason: null } }; },
    ...over };
  return t;
}
const manager = (t, online = true) => createAuctionManager(t, () => online);

test('offline: nothing is attempted and no operation is created', async () => {
  const t = fake();
  await assert.rejects(manager(t, false).publish(prepared), /offline/);
  await assert.rejects(manager(t, false).loadAuctions(), /offline/);
  await assert.rejects(manager(t, false).cancelBid('auc_1', 'bid_b', { reason: 'suspected_abuse' }, KEY), /offline/);
  assert.equal(t.calls.length, 0);
});

test('publication is confirmed only after the server list shows it', async () => {
  const t = fake();
  const { operation } = await manager(t).publish(prepared);
  assert.equal(operation.state, 'confirmed');
  assert.equal(operation.target, 'auc_1', 'the auction id, not the page address');
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
  const changed = { ...prepared, body: JSON.stringify({ auction: { id: 'auc_1' }, details: { title: 'Altro' } }) };
  await assert.rejects(manager(t).publish(changed, first.operation), /request_changed/);
});

test('refusals and expired sessions are never reported as success', async () => {
  const t = fake({ async publish() { return { status: 401 }; } });
  const out = await manager(t).publish(prepared);
  assert.equal(out.needsSignIn, true);
  assert.equal(out.operation.state, 'rejected');
  const server = fake({ async publish() { return { status: 503 }; } });
  assert.equal((await manager(server).publish(prepared)).operation.state, 'pending_confirmation');
  // Seen in the Desktop window: the server's own code is kept as the reason (not just the HTTP status).
  const taken = fake({ async publish() { return { status: 409, error: 'public_details_exist' }; } });
  assert.deepEqual([(await manager(taken).publish(prepared)).operation.state, (await manager(taken).publish(prepared)).operation.reason],
    ['rejected', 'public_details_exist']);
  const bare = fake({ async publish() { return { status: 409 }; } });
  assert.equal((await manager(bare).publish(prepared)).operation.reason, 'http_409');
});

test('bid history follows cursors and rejects a cursor loop', async () => {
  const t = fake(); t.pages = true;
  assert.deepEqual((await manager(t).loadBids('auc_1')).map(b => b.bidId), ['bid_a', 'bid_b']);
  const loop = fake({ async listBids() { return { items: [], nextCursor: 'same' }; } });
  await assert.rejects(manager(loop).loadBids('auc_1'), /invalid_response/);
});

test('bids placed at the same instant keep the order in which the server accepted them', async () => {
  // Seen in the Desktop window: equal timestamps were reordered by their random ids.
  const at = '2027-03-04T12:00:00.000Z';
  const t = fake({ async listBids() {
    return { items: [bid('f-first', at), bid('a-second', at), bid('c-third', '2027-03-04T11:59:59.000Z')], nextCursor: null }; } });
  assert.deepEqual((await manager(t).loadBids('auc_1')).map(b => b.bidId), ['c-third', 'f-first', 'a-second']);
});

test('cancellation is confirmed by the reloaded history; refusals and unreachable stay honest', async () => {
  const ok = fake();
  const done = await manager(ok).cancelBid('auc_1', 'bid_b', { reason: 'suspected_abuse', note: '  sospetta  ' }, KEY);
  assert.equal(done.outcome.operation.state, 'confirmed');
  assert.equal(done.bids.find(b => b.bidId === 'bid_b').state, 'cancelled');
  assert.deepEqual(ok.calls.at(-1)[2], { reason: 'suspected_abuse', note: 'sospetta' }, 'note sent trimmed');

  const closed = fake(); closed.refuseCancel = true;
  const refused = await manager(closed).cancelBid('auc_1', 'bid_b', { reason: 'suspected_abuse' }, KEY);
  assert.equal(refused.outcome.operation.state, 'rejected');
  assert.equal(refused.outcome.operation.reason, 'auction_closed');

  const lost = fake(); lost.failCancel = true; lost.failBids = true;
  const unknown = await manager(lost).cancelBid('auc_1', 'bid_b', { reason: 'suspected_abuse' }, KEY);
  assert.equal(unknown.outcome.operation.state, 'pending_confirmation');
  assert.equal(unknown.bids, null);

  // Gestionale Opere: a reason code, an optional note of up to 500 characters, an optional block.
  for (const request of [{ reason: 'sospetta' }, { reason: 'suspected_abuse', note: 'x'.repeat(501) },
    { reason: 'suspected_abuse', blockEmail: 'yes' }, null, 'suspected_abuse']) {
    await assert.rejects(manager(ok).cancelBid('auc_1', 'bid_b', request, KEY), /invalid_reason/);
  }
  await manager(ok).cancelBid('auc_1', 'bid_b', { reason: 'other_admin_reason', blockEmail: true }, 'idem-abcdef0123456789');
  assert.deepEqual(ok.calls.at(-1)[2], { reason: 'other_admin_reason', blockEmail: true });
});

test('already cancelled by an earlier attempt: the server history confirms it', async () => {
  const t = fake({ async cancelBid(a, b) { t.cancelled = true; return { status: 409, body: { bidId: b, outcome: 'already_cancelled', reason: null } }; } });
  const out = await manager(t).cancelBid('auc_1', 'bid_b', { reason: 'suspected_abuse' }, KEY);
  assert.equal(out.outcome.operation.state, 'confirmed');
});

test('a cancellation answered "cancelled" but not visible in the history is not a success', async () => {
  const t = fake({ async cancelBid(a, b) { return { status: 201, body: { bidId: b, outcome: 'cancelled', reason: null } }; } });
  const out = await manager(t).cancelBid('auc_1', 'bid_b', { reason: 'suspected_abuse' }, KEY);
  assert.equal(out.outcome.operation.state, 'pending_confirmation');
});
