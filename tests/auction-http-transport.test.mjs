import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parseAuction, parseBidPage, leadingConsistency } from '../src/app/auction-contract.ts';
import { createArtistV1Transport, isSignInRequired } from '../src/app/auction-http-transport.ts';
import { createAuctionManager } from '../src/app/auction-manager.ts';

// Responses captured from the real Repertica Auctions server (artist-v1) running the synthetic preview
// (fictitious artist and @example.test bidders). They pin the Desktop parsers to the server contract.
const fixture = name => JSON.parse(readFileSync(new URL(`./fixtures/auctions-artist-v1/${name}.json`, import.meta.url), 'utf8'));
const KEY = 'idem-0123456789abcdef';

test('the Desktop parsers accept every captured server response', () => {
  const [listed] = fixture('auctions').items.map(parseAuction);
  const detail = parseAuction(fixture('auction').auction);
  assert.equal(detail.auctionId, listed.auctionId);
  assert.equal(detail.state, 'live');
  assert.equal(detail.cancelledBidCount, 1);
  const before = parseBidPage(fixture('bids')).bids;
  const after = parseBidPage(fixture('bids-after-cancel')).bids;
  assert.equal(before.filter(b => b.state === 'cancelled').length, 1);
  assert.equal(after.filter(b => b.state === 'cancelled').length, 2);
  assert.ok(after.every(b => b.bidderEmail === null || b.bidderEmail.endsWith('@example.test')));
  assert.equal(leadingConsistency(detail, before), 'consistent');
});

function replay(routes) {
  const calls = [];
  const port = async request => {
    calls.push(request);
    const route = routes.find(r => r.method === request.method && r.path.test(request.path));
    if (!route) return { status: 404, body: { error: 'not_found' } };
    if (route.lost) throw new Error('connection reset');
    return route.respond(request);
  };
  return { port, calls };
}

test('transport maps artist-v1 endpoints, headers and bodies exactly', async () => {
  const bidId = fixture('bids').items.find(b => b.state === 'valid' && !b.isLeading).bidId;
  const { port, calls } = replay([
    { method: 'GET', path: /^\/api\/artist\/auctions\?tenantId=synthetic-artist&page=1$/, respond: () => ({ status: 200, body: { ...fixture('auctions'), hasMore: false } }) },
    { method: 'GET', path: /^\/api\/artist\/auctions\/synthetic-lot\/bids$/, respond: () => ({ status: 200, body: fixture('bids') }) },
    { method: 'POST', path: /\/cancellation$/, respond: () => ({ status: 201, body: fixture('cancel-201') }) },
    { method: 'POST', path: /^\/api\/artist\/publications$/, respond: () => ({ status: 201, body: {} }) },
  ]);
  const transport = createArtistV1Transport(port, () => 'synthetic-artist');
  assert.equal((await transport.listAuctions()).items.length, 1);
  assert.equal((await transport.listBids('synthetic-lot', null)).items.length, 4);
  const answer = await transport.cancelBid('synthetic-lot', bidId, 'Offerente fittizio duplicato', KEY);
  assert.deepEqual([answer.status, answer.body.outcome], [201, 'cancelled']);
  const cancel = calls.find(c => c.method === 'POST' && c.path.endsWith('/cancellation'));
  assert.equal(cancel.path, `/api/artist/auctions/synthetic-lot/bids/${bidId}/cancellation`);
  assert.equal(cancel.headers['Idempotency-Key'], KEY);
  assert.deepEqual(JSON.parse(cancel.body), { reason: 'Offerente fittizio duplicato' });
  assert.equal((await transport.publish('{"auction":{},"details":{}}', KEY)).status, 201);
  assert.ok(calls.every(c => c.path.startsWith('/api/artist/')), 'only artist API paths, never a full URL');
  assert.ok(calls.every(c => !('Authorization' in c.headers)), 'the token is added by the native port, not the webview');
  await assert.rejects(transport.cancelBid('synthetic-lot', '../x', 'reason', KEY), /invalid_bid/);
  await assert.rejects(createArtistV1Transport(port, () => null).listAuctions(), /tenant_not_selected/);
});

test('documented refusals become cancel outcomes; 401 asks to sign in again', async () => {
  const { port } = replay([
    { method: 'POST', path: /bids\/bid-closed\/cancellation$/, respond: () => ({ status: 409, body: { error: 'auction_closed' } }) },
    { method: 'POST', path: /bids\/bid-done\/cancellation$/, respond: () => ({ status: 409, body: fixture('cancel-409-already') }) },
    { method: 'GET', path: /^\/api\/artist\/auctions\?/, respond: () => ({ status: 401, body: fixture('error-401') }) },
  ]);
  const transport = createArtistV1Transport(port, () => 'synthetic-artist');
  assert.equal((await transport.cancelBid('lot', 'bid-closed', 'reason', KEY)).body.outcome, 'not_allowed');
  assert.equal((await transport.cancelBid('lot', 'bid-done', 'reason', KEY)).body.outcome, 'already_cancelled');
  await assert.rejects(transport.listAuctions(), error => isSignInRequired(error));
});

test('manager over the transport: a lost cancellation is confirmed only by the reloaded history', async () => {
  const bidId = fixture('bids').items.find(b => b.state === 'valid' && !b.isLeading).bidId;
  let cancelled = false;
  const { port, calls } = replay([
    { method: 'POST', path: /\/cancellation$/, lost: true },
    { method: 'GET', path: /\/bids$/, respond: () => ({ status: 200, body: fixture(cancelled ? 'bids-after-cancel' : 'bids') }) },
  ]);
  const manager = createAuctionManager(createArtistV1Transport(port, () => 'synthetic-artist'), () => true);
  const pending = await manager.cancelBid('synthetic-lot', bidId, 'Offerente fittizio duplicato', KEY);
  assert.equal(pending.outcome.operation.state, 'pending_confirmation', 'lost answer, history still valid: not a success');
  cancelled = true; // the server had applied it: the retry reuses key and body and the history confirms it
  const confirmed = await manager.cancelBid('synthetic-lot', bidId, 'Offerente fittizio duplicato', KEY, pending.outcome.operation);
  assert.equal(confirmed.outcome.operation.state, 'confirmed');
  const keys = calls.filter(c => c.method === 'POST').map(c => c.headers['Idempotency-Key']);
  assert.deepEqual(keys, [KEY, KEY]);
});
