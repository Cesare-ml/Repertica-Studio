import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createNativeConnection, nativeErrorCode, parseStatus } from '../src/app/auction-native.ts';
import { createArtistV1Transport } from '../src/app/auction-http-transport.ts';
import { createAuctionManager } from '../src/app/auction-manager.ts';
import { dropPublication, loadPublications, newIdempotencyKey, recordPublication } from '../src/app/auction-publications.ts';
import { newAuctionDraft, preparePreview } from '../src/app/auction-preview.ts';

const fixture = name => JSON.parse(readFileSync(new URL(`./fixtures/auctions-artist-v1/${name}.json`, import.meta.url), 'utf8'));
const status = extra => ({ available: true, signedIn: true, expiresInSeconds: 600, development: false, signingIn: false, ...extra });

test('connection status is read strictly; unavailable means never signed in', () => {
  assert.deepEqual(parseStatus(status()), status());
  assert.equal(parseStatus(status({ available: false, signedIn: true })).signedIn, false);
  for (const bad of [null, [], status({ signedIn: 'yes' }), status({ expiresInSeconds: -1 }), status({ development: undefined })]) {
    assert.throws(() => parseStatus(bad), /invalid_response/);
  }
});

test('a missing native side degrades to "not available", never to signed in', async () => {
  const connection = createNativeConnection(async () => { throw new Error('ipc unavailable'); });
  assert.deepEqual(await connection.status(), { available: false, signedIn: false, expiresInSeconds: null, development: false, signingIn: false });
  await assert.rejects(connection.signIn(), /native_unavailable/);
  assert.equal(nativeErrorCode('auctions_not_available'), 'auctions_not_available');
  assert.equal(nativeErrorCode('Some <html> text'), 'native_unavailable');
});

test('the port hands one artist-v1 operation to the native side, without any token', async () => {
  const calls = [];
  const connection = createNativeConnection(async (command, args) => {
    calls.push([command, args]);
    if (args.path.endsWith('/cancellation')) return { status: 201, body: fixture('cancel-201') };
    return { status: 200, body: { ...fixture('auctions'), hasMore: false } };
  });
  const transport = createArtistV1Transport(connection.port, () => 'synthetic-artist');
  assert.equal((await transport.listAuctions()).items.length, 1);
  const bidId = fixture('bids').items.find(b => b.state === 'valid' && !b.isLeading).bidId;
  await transport.cancelBid('synthetic-lot', bidId, { reason: 'obvious_test_or_troll', note: 'Offerente fittizio' }, 'cancel-0123456789abcdef');
  assert.deepEqual(calls[0], ['auction_api_request', { method: 'GET', path: '/api/artist/auctions?tenantId=synthetic-artist&page=1',
    body: null, idempotencyKey: null }]);
  assert.deepEqual(calls[1][1], { method: 'POST', path: `/api/artist/auctions/synthetic-lot/bids/${bidId}/cancellation`,
    body: JSON.stringify({ reason: 'obvious_test_or_troll', note: 'Offerente fittizio' }), idempotencyKey: 'cancel-0123456789abcdef' });
  assert.ok(calls.every(([command]) => command === 'auction_api_request'));
  assert.ok(!JSON.stringify(calls).includes('Bearer'), 'the webview never handles the access token');
  const broken = createNativeConnection(async () => ({ body: {} }));
  await assert.rejects(broken.port({ method: 'GET', path: '/api/artist/tenants', headers: {} }), /invalid_response/);
});

test('native refusal before sending (no session) asks to sign in; a native error is an uncertain outcome', async () => {
  const signedOut = createNativeConnection(async () => ({ status: 401, body: { error: 'sign_in_required' } }));
  const manager = createAuctionManager(createArtistV1Transport(signedOut.port, () => 't-1'), () => true);
  const prepared = { artworkId: 'a', auctionId: 'auc-pub-0123456789abcdef', idempotencyKey: 'pub-0123456789abcdef',
    body: JSON.stringify({ auction: {}, details: { slug: 'opera' } }) };
  const out = await manager.publish(prepared);
  assert.equal(out.needsSignIn, true);
  assert.equal(out.operation.state, 'rejected');
  const lost = createNativeConnection(async () => { throw 'network_unavailable'; });
  const uncertain = await createAuctionManager(createArtistV1Transport(lost.port, () => 't-1'), () => true).publish(prepared);
  assert.equal(uncertain.operation.state, 'pending_confirmation');
});

function memory() { const m = new Map(); return { getItem: k => m.get(k) ?? null, setItem: (k, v) => m.set(k, v), m }; }

test('unconfirmed publications survive a restart with the same key and body, and only public fields', () => {
  const store = memory();
  const key = newIdempotencyKey();
  assert.match(key, /^pub-[0-9a-f]{36}$/);
  const draft = { ...newAuctionDraft({ id: 'art-1', title: 'Paesaggio', status: 'available' }), slug: 'paesaggio-01', description: 'Olio',
    startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z', startingAmount: 12000, minimumIncrementAmount: 1000, italyShippingCost: 2500, bankBeneficiary: 'Artista di prova', bankIban: 'IT60 X054 2811 1010 0000 0123 456', bankBicSwift: 'BPMOIT2S', bankNameAndAddress: 'Banca di prova, Milano' };
  const prepared = preparePreview(draft, 'tenant-a', key, Date.parse('2027-03-01T00:00:00.000Z'));
  const operation = { id: 'publish:' + key, kind: 'publish', target: 'paesaggio-01', idempotencyKey: key, body: prepared.body,
    state: 'pending_confirmation', reason: null, attempts: 1 };
  recordPublication(store, { artworkId: 'art-1', tenantId: 'tenant-a', title: 'Paesaggio', prepared, operation });
  const [restored] = loadPublications(store);
  assert.equal(restored.prepared.body, prepared.body);
  assert.equal(restored.prepared.idempotencyKey, key);
  assert.doesNotMatch(store.m.get('repertica-auction-publications-v1'), /Bearer|token|bidderEmail/i);
  // Tampered or inconsistent entries are ignored rather than sent.
  const raw = JSON.parse(store.m.get('repertica-auction-publications-v1'));
  store.setItem('repertica-auction-publications-v1', JSON.stringify([{ ...raw[0], tenantId: 'other-tenant' },
    { ...raw[0], operation: { ...raw[0].operation, body: '{}' } }, raw[0]]));
  assert.equal(loadPublications(store).length, 1);
  assert.deepEqual(recordPublication(store, { ...restored, operation: { ...operation, state: 'confirmed' } }), [], 'confirmed entries leave the journal');
  recordPublication(store, restored);
  assert.deepEqual(dropPublication(store, key), []);
  store.setItem('repertica-auction-publications-v1', 'not json');
  assert.deepEqual(loadPublications(store), []);
});
