import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseAuction, parseBid, parseBidPage, parseCancelResult, cancelOutcomeFromError, leadingConsistency,
  resultNeedsExplicitSale, maskEmail, AUCTION_API_CONNECTED, AUCTION_CONTRACT_VERSION } from '../src/app/auction-contract.ts';
import { beginOperation, afterTransport, retry, confirmPublication, confirmCancellation } from '../src/app/auction-operations.ts';
import { loadDrafts, saveDraft, removeDraft, localInputToUtc, euroToCents, centsToEuro, loadSavedBankDetails, loadWinnerDefaults,
  saveWinnerDefaults } from '../src/app/auction-drafts.ts';
import { defaultWinnerEmail, fillWinnerTemplate } from '../src/app/auction-winner-email.ts';
import { newAuctionDraft, reviewRows } from '../src/app/auction-preview.ts';

// Shapes of contract artist-v1 (see tests/fixtures/auctions-artist-v1, captured from the real server).
const auction = (extra = {}) => ({ auctionId: 'auc_1', tenantId: 'tenant-a', slug: 'paesaggio-01', title: 'Paesaggio', state: 'live',
  startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z', maxEndsAt: '2027-03-06T10:00:00.000Z', currency: 'EUR',
  startingAmount: 12000, minimumIncrementAmount: 1000, reserveAmount: null, reserveMet: true, currentAmount: 14000,
  leadingBidId: 'bid_2', minimumNextBidAmount: 15000, bidCount: 2, cancelledBidCount: 0, winnerBidId: null, winningAmount: null,
  paymentRecorded: false, studioArtworkId: null, version: 2, internalNote: 'must not survive', ...extra });
const bid = (id, amount, at, extra = {}) => ({ bidId: id, amount, placedAt: at, bidderLabel: '#1', bidderEmail: 'synthetic@example.test',
  state: 'valid', isLeading: false, cancelledAt: null, cancelReason: null, ...extra });
const memory = () => { const m = new Map(); return { getItem: k => m.get(k) ?? null, setItem: (k, v) => m.set(k, v), m }; };

test('the client is connected to the deployed services (mirrors ONLINE_ENABLED)', () => {
  assert.equal(AUCTION_API_CONNECTED, true);
  assert.equal(AUCTION_CONTRACT_VERSION, 'artist-v1');
});

test('auction parser keeps an allowlist and rejects malformed values', () => {
  const parsed = parseAuction(auction());
  assert.equal('internalNote' in parsed, false);
  assert.equal(parsed.currentAmount, 14000);
  assert.equal(parseAuction(auction({ state: 'something_new' })).state, 'unknown');
  for (const bad of [{ startsAt: '2027-03-04T10:00:00Z' }, { endsAt: '2027-03-01T00:00:00.000Z' },
    { startingAmount: 1.5 }, { currency: 'USD' }, { slug: '' }, { auctionId: '../x' }, { paymentRecorded: true },
    { maxEndsAt: '2027-03-05T00:00:00.000Z' }, { bidCount: -1 }, { reserveMet: 'yes' }, { leadingBidId: undefined }]) {
    assert.throws(() => parseAuction(auction(bad)), /invalid_response/);
  }
  assert.throws(() => parseAuction(null), /invalid_response/);
});

test('bids: cancelled needs a time, pages dedupe and sort by server time', () => {
  assert.throws(() => parseBid(bid('bid_a', 100, '2027-03-04T11:00:00.000Z', { state: 'cancelled' })), /invalid_response/);
  assert.throws(() => parseBid(bid('bid_a', 100, '2027-03-04T11:00:00.000Z', { bidderEmail: 'not-an-email' })), /invalid_response/);
  assert.throws(() => parseBid(bid('bid_a', 100, '2027-03-04T11:00:00.000Z', { bidderLabel: 'alice' })), /invalid_response/);
  assert.equal(parseBid(bid('bid_a', 100, '2027-03-04T11:00:00.000Z', { bidderEmail: null })).bidderEmail, null);
  const page = parseBidPage({ items: [bid('bid_b', 14000, '2027-03-04T12:00:00.000Z'), bid('bid_a', 13000, '2027-03-04T11:00:00.000Z'),
    bid('bid_b', 14000, '2027-03-04T12:00:00.000Z')], nextCursor: null });
  assert.deepEqual(page.bids.map(b => b.bidId), ['bid_a', 'bid_b']);
  assert.equal(maskEmail('synthetic@example.test'), 's***@example.test');
});

test('cancel answers: success shape, and refusals mapped from the documented error codes', () => {
  assert.equal(parseCancelResult({ bidId: 'bid_2', outcome: 'cancelled', cancelledAt: '2027-03-04T13:00:00.000Z' }).outcome, 'cancelled');
  assert.throws(() => parseCancelResult({ bidId: 'bid_2', outcome: 'maybe', cancelledAt: '2027-03-04T13:00:00.000Z' }), /invalid_response/);
  assert.throws(() => parseCancelResult({ bidId: 'bid_2', outcome: 'cancelled' }), /invalid_response/);
  assert.equal(cancelOutcomeFromError('bid_2', 409, { error: 'bid_already_cancelled' }).outcome, 'already_cancelled');
  assert.equal(cancelOutcomeFromError('bid_2', 409, { error: 'auction_closed' }).outcome, 'not_allowed');
  assert.equal(cancelOutcomeFromError('bid_2', 403, { error: 'tenant_access_denied' }).outcome, 'not_allowed');
  assert.equal(cancelOutcomeFromError('bid_2', 400, { error: 'invalid_reason' }), null);
});

test('leading bid is checked against the server history, never recomputed', () => {
  const a = parseAuction(auction());
  const ok = [bid('bid_1', 13000, '2027-03-04T11:00:00.000Z'), bid('bid_2', 14000, '2027-03-04T12:00:00.000Z', { isLeading: true })].map(parseBid);
  assert.equal(leadingConsistency(a, ok), 'consistent');
  const cancelledLead = [ok[0], parseBid(bid('bid_2', 14000, '2027-03-04T12:00:00.000Z', { state: 'cancelled', cancelledAt: '2027-03-04T13:00:00.000Z' }))];
  assert.equal(leadingConsistency(a, cancelledLead), 'mismatch');
  assert.equal(leadingConsistency(a, [ok[0]]), 'unknown');
});

test('award is not payment', () => {
  assert.equal(resultNeedsExplicitSale('awarded'), true);
  assert.equal(resultNeedsExplicitSale('live'), false);
  assert.equal(resultNeedsExplicitSale('not_awarded'), false);
});

test('a lost response is pending, not success; retry keeps key and body; only the server state confirms', () => {
  const key = 'idem-0123456789abcdef';
  // A publication is recognised by its auction id: the site chooses the page address.
  let op = beginOperation('publish', 'auc_1', key, '{"auction":{"id":"auc_1"}}');
  op = afterTransport(op, { kind: 'lost' });
  assert.equal(op.state, 'pending_confirmation');
  const again = retry(op);
  assert.equal(again.idempotencyKey, key);
  assert.equal(again.body, op.body);
  assert.equal(again.attempts, 2);
  assert.equal(confirmPublication(again, []).state, 'pending_confirmation');
  assert.equal(confirmPublication(again, [parseAuction(auction({ slug: 'paesaggio-2' }))]).state, 'confirmed');
  assert.equal(confirmPublication(again, [parseAuction(auction({ auctionId: 'auc_2' }))]).state, 'pending_confirmation');
  // Requests journaled by versions that named the page address are still confirmed by it.
  const journaled = beginOperation('publish', 'paesaggio-01', key, '{"details":{"slug":"paesaggio-01"}}');
  assert.equal(confirmPublication(journaled, [parseAuction(auction())]).state, 'confirmed');
  assert.equal(afterTransport(beginOperation('publish', 's', key, '{}'), { kind: 'response', status: 201 }).state, 'pending_confirmation');
  assert.equal(afterTransport(beginOperation('publish', 's', key, '{}'), { kind: 'response', status: 403 }).state, 'rejected');
  assert.equal(afterTransport(beginOperation('publish', 's', key, '{}'), { kind: 'response', status: 503 }).state, 'pending_confirmation');
  assert.throws(() => retry(beginOperation('publish', 's', key, '{}')), /not_retryable/);
  assert.throws(() => beginOperation('publish', 's', 'short', '{}'), /invalid_idempotency_key/);
});

test('bid cancellation is confirmed only when the reloaded history shows it', () => {
  const key = 'idem-0123456789abcdef';
  const op = afterTransport(beginOperation('cancel_bid', 'bid_2', key, '{}'), { kind: 'response', status: 200 });
  const valid = [parseBid(bid('bid_2', 14000, '2027-03-04T12:00:00.000Z'))];
  const cancelled = [parseBid(bid('bid_2', 14000, '2027-03-04T12:00:00.000Z', { state: 'cancelled', cancelledAt: '2027-03-04T13:00:00.000Z' }))];
  assert.equal(confirmCancellation(op, valid, { bidId: 'bid_2', outcome: 'cancelled', reason: null }).state, 'pending_confirmation');
  assert.equal(confirmCancellation(op, cancelled, null).state, 'confirmed');
  const refused = confirmCancellation(op, valid, { bidId: 'bid_2', outcome: 'contested', reason: null });
  assert.equal(refused.state, 'rejected');
  assert.equal(refused.reason, 'contested');
});

test('local drafts: one per artwork, validated on load, no extra fields kept', () => {
  const store = memory();
  const a = { id: 'art-1', title: 'Paesaggio', status: 'available' };
  const d = { ...newAuctionDraft(a), description: 'Olio', secret: 'private', buyer: 'x', artistEmail: 'studio@example.test',
    winnerMode: 'custom_email', winnerSubject: 'Hai vinto', winnerBody: 'Ciao {name}, grazie per la tua offerta.' };
  saveDraft(store, d, new Date('2027-01-01T00:00:00Z'));
  saveDraft(store, { ...d, title: 'Paesaggio II' }, new Date('2027-01-02T00:00:00Z'));
  const loaded = loadDrafts(store);
  assert.equal(loaded.length, 1);
  assert.equal(loaded[0].title, 'Paesaggio II');
  assert.equal(loaded[0].local, true);
  assert.equal('secret' in loaded[0], false);
  assert.deepEqual([loaded[0].artistEmail, loaded[0].winnerMode, loaded[0].winnerSubject], ['studio@example.test', 'custom_email', 'Hai vinto']);
  // A draft saved by an earlier version (page address, shipping and bank transfer) still loads, without those fields.
  store.setItem('repertica-auction-drafts-v1', JSON.stringify([{ artworkId: 'art-1', title: 'Paesaggio', description: 'Olio',
    startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: null, slug: 'paesaggio', italyShippingCost: 2500, bankIban: 'IT60X0542811101000000123456', local: true,
    updatedAt: '2027-01-01T00:00:00.000Z' }]));
  const old = loadDrafts(store)[0];
  assert.equal(old.winnerMode, 'contact_privately');
  assert.equal(['slug', 'italyShippingCost', 'bankIban'].some(key => key in old), false);
  saveDraft(store, d, new Date('2027-01-02T00:00:00Z'));
  assert.equal(JSON.stringify(store.m.get('repertica-auction-drafts-v1')).includes('private'), false);
  store.setItem('repertica-auction-drafts-v1', 'not json');
  assert.deepEqual(loadDrafts(store), []);
  store.setItem('repertica-auction-drafts-v1', JSON.stringify([{ artworkId: 'x', title: 5 }]));
  assert.deepEqual(loadDrafts(store), []);
  saveDraft(store, d);
  assert.deepEqual(removeDraft(store, 'art-1'), []);
});

test('review lists only the public fields and reports validation codes', () => {
  const now = Date.parse('2027-03-01T00:00:00.000Z');
  const d = { ...newAuctionDraft({ id: 'art-1', title: 'Paesaggio', status: 'available' }),
    description: 'Olio su tela', startsAt: '2027-03-04T10:00:00.000Z', endsAt: '2027-03-06T10:00:00.000Z',
    startingAmount: 12000, minimumIncrementAmount: 1000, artistEmail: 'studio@example.test' };
  const rows = reviewRows(d, now);
  assert.deepEqual(rows.map(r => r.field), ['title', 'description', 'startsAt', 'endsAt',
    'startingAmount', 'minimumIncrementAmount', 'currency', 'artistEmail', 'winnerMode', 'mainImage']);
  const custom = reviewRows({ ...d, winnerMode: 'custom_email', winnerSubject: 'Hai vinto', winnerBody: 'Ciao {name},\nbonifico a IT60…' }, now);
  assert.deepEqual(custom.filter(r => r.field.startsWith('winner')).map(r => [r.field, r.value]),
    [['winnerMode', 'custom_email'], ['winnerSubject', 'Hai vinto'], ['winnerBody', 'Ciao {name},\nbonifico a IT60…']]);
  assert.equal(rows.find(r => r.field === 'startsAt').value, '2027-03-04T10:00:00.000Z');
  assert.throws(() => reviewRows({ ...d, endsAt: d.startsAt }, now), /invalid_schedule/);
  assert.throws(() => reviewRows({ ...d, startingAmount: null }, now), /invalid_amount/);
});

test('form conversions are strict', () => {
  assert.equal(euroToCents('120'), 12000);
  assert.equal(euroToCents('120,5'), 12050);
  assert.equal(euroToCents('120.05'), 12005);
  assert.equal(euroToCents('-1'), null);
  assert.equal(euroToCents('1e3'), null);
  assert.equal(centsToEuro(12050), '120,50');
  assert.equal(localInputToUtc('nonsense'), '');
  assert.match(localInputToUtc('2027-03-04T10:00'), /^2027-03-0[34]T\d\d:00:00\.000Z$/);
});

test('winner choices are kept on this computer; the template uses saved bank details and the interface language', () => {
  const store = memory();
  assert.deepEqual(loadWinnerDefaults(store), { artistEmail: '', winnerMode: 'contact_privately', winnerSubject: '', winnerBody: '' });
  saveWinnerDefaults(store, { artistEmail: 'studio@example.test', winnerMode: 'custom_email', winnerSubject: 'Hai vinto', winnerBody: 'Testo' });
  assert.deepEqual(loadWinnerDefaults(store), { artistEmail: 'studio@example.test', winnerMode: 'custom_email', winnerSubject: 'Hai vinto', winnerBody: 'Testo' });
  assert.equal(loadSavedBankDetails(store), null);
  store.setItem('repertica-auction-payment-v1', JSON.stringify({ bankBeneficiary: 'Artista di prova', bankIban: 'it60 x054 2811 1010 0000 0123 456',
    bankBicSwift: 'bpmo it2s', bankNameAndAddress: 'Banca di prova, Milano', italyShippingCost: 2500 }));
  const bank = loadSavedBankDetails(store);
  assert.deepEqual(bank, { beneficiary: 'Artista di prova', iban: 'IT60X0542811101000000123456', bicSwift: 'BPMOIT2S', bankNameAndAddress: 'Banca di prova, Milano' });
  const it = defaultWinnerEmail('it', bank);
  assert.equal(it.subject, 'Hai vinto l’asta: {title}');
  assert.match(it.body, /^Ciao \{name\},[\s\S]*\{amount\}[\s\S]*Beneficiario: Artista di prova\nIBAN: IT60X0542811101000000123456\nBIC\/SWIFT: BPMOIT2S[\s\S]*\{artist\}$/);
  assert.match(defaultWinnerEmail('en').body, /^Hello \{name\},[\s\S]*IBAN: …/);
  for (const language of ['it', 'en', 'es', 'fr', 'de']) {
    const t = defaultWinnerEmail(language, bank);
    assert.ok(t.subject.length >= 3 && t.subject.length <= 160 && !/\n/.test(t.subject), language);
    assert.ok(t.body.length >= 20 && t.body.length <= 2500 && /\{name\}/.test(t.body) && /\{amount\}/.test(t.body), language);
  }
  assert.equal(fillWinnerTemplate('Ciao {name}, {amount} per {title} {altro}', { name: 'Anna', amount: '160,00 €', title: 'Opera' }),
    'Ciao Anna, 160,00 € per Opera {altro}');
});
