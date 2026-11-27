// Complete management of an auction by its artist: participants, terms, e-mail blocks and the e-mail to the
// winner (contract artist-v1 "Gestione completa", Repertica-auctions docs/ARTIST_API_V1.md).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseParticipants, parseTerms, parseEmailPolicy } from '../src/app/auction-contract.ts';
import { createArtistV1Transport } from '../src/app/auction-http-transport.ts';
import { createAuctionManager } from '../src/app/auction-manager.ts';
import { winnerMailDraft } from '../src/app/auction-winner-mail.ts';

const person = (extra = {}) => ({ name: 'Anna', email: 'anna@example.test', status: 'verified', validBidCount: 2, isLeading: true,
  verifiedAt: '2027-03-04T10:00:00.000Z', ...extra });
const BLOCK_ID = '0123456789abcdef01234567';

test('participants: only the documented shape is accepted; names may be missing, addresses may not be malformed', () => {
  const { participants, contactsAvailable } = parseParticipants({ items: [person(), person({ name: null, email: 'bruno@example.test', status: 'blocked',
    validBidCount: 0, isLeading: false, verifiedAt: null })], contactsAvailable: true });
  assert.equal(contactsAvailable, true);
  assert.deepEqual(participants.map(p => [p.name, p.email, p.status, p.validBidCount, p.isLeading, p.verifiedAt]),
    [['Anna', 'anna@example.test', 'verified', 2, true, '2027-03-04T10:00:00.000Z'], [null, 'bruno@example.test', 'blocked', 0, false, null]]);
  assert.deepEqual(parseParticipants({ items: [], contactsAvailable: false }), { participants: [], contactsAvailable: false });
  for (const bad of [{ items: [person({ status: 'unknown' })], contactsAvailable: true }, { items: [person({ email: 'not an address' })], contactsAvailable: true },
    { items: [person({ validBidCount: -1 })], contactsAvailable: true }, { items: [person({ verifiedAt: '2027-03-04' })], contactsAvailable: true },
    { items: [person()] }, { items: 'x', contactsAvailable: true }, null])
    assert.throws(() => parseParticipants(bad), /invalid_response/);
});

test('terms: both winner modes, or none', () => {
  const base = { rulesVersion: '2026-11-27', showClosedPublicly: true, artistEmail: 'studio@example.test' };
  assert.deepEqual({ ...parseTerms({ terms: { ...base, winner: { mode: 'contact_privately' } } }) }.winner, { mode: 'contact_privately' });
  const custom = parseTerms({ terms: { ...base, winner: { mode: 'custom_email', subject: 'Hai vinto', body: 'Ciao {name}' } } });
  assert.deepEqual([custom.rulesVersion, custom.winner.mode, custom.winner.subject, custom.winner.body], ['2026-11-27', 'custom_email', 'Hai vinto', 'Ciao {name}']);
  assert.equal(parseTerms({ terms: null }), null);
  for (const bad of [{ terms: { ...base, winner: { mode: 'bank_transfer' } } }, { terms: { ...base, artistEmail: 'x' } },
    { terms: { ...base, winner: { mode: 'custom_email', subject: 'Hai vinto' } } }, {}])
    assert.throws(() => parseTerms(bad), /invalid_response/);
});

test('e-mail policy: masked hints only, ids of 24 hex characters, the platform list read only', () => {
  const policy = parseEmailPolicy({ blockedEmails: [{ id: BLOCK_ID, emailHint: 'b***@example.test', reason: 'suspected_abuse', blockedAt: '2027-03-04T10:00:00.000Z' },
    { id: 'f'.repeat(24), emailHint: null, reason: 'spam', blockedAt: '2027-03-05T10:00:00.000Z' }],
    disposableDomains: { count: 121, syncedAt: '2027-03-05T03:00:00.000Z' } });
  assert.deepEqual(policy.blockedEmails.map(b => [b.id, b.emailHint, b.reason]), [[BLOCK_ID, 'b***@example.test', 'suspected_abuse'], ['f'.repeat(24), null, 'spam']]);
  assert.deepEqual({ ...policy.disposableDomains }, { count: 121, syncedAt: '2027-03-05T03:00:00.000Z' });
  assert.deepEqual({ ...parseEmailPolicy({ blockedEmails: [], disposableDomains: { count: 0, syncedAt: null } }).disposableDomains }, { count: 0, syncedAt: null });
  for (const bad of [{ blockedEmails: [{ id: 'short', emailHint: null, reason: 'x', blockedAt: '2027-03-04T10:00:00.000Z' }], disposableDomains: { count: 0, syncedAt: null } },
    { blockedEmails: [], disposableDomains: { count: -1, syncedAt: null } }, { blockedEmails: [] }])
    assert.throws(() => parseEmailPolicy(bad), /invalid_response/);
});

function recordingPort(answer) {
  const calls = [];
  return { calls, port: async request => { calls.push(request); return answer(request); } };
}

test('transport: fixed paths, the selected profile, a key on every write and no other shape', async () => {
  const { calls, port } = recordingPort(request => request.method === 'GET'
    ? { status: 200, body: { items: [], contactsAvailable: true } } : { status: 201, body: { id: BLOCK_ID, blocked: true } });
  const transport = createArtistV1Transport(port, () => 'artist-a');
  await transport.getParticipants('lot-1');
  await transport.getTerms('lot-1');
  await transport.getEmailPolicy();
  const blocked = await transport.blockEmail('carla@example.test', 'spam', 'block-0123456789abcdef');
  const noReason = await transport.blockEmail('dario@example.test', null, 'block-0123456789abcdee');
  const removed = await transport.unblockEmail(BLOCK_ID, 'unblock-0123456789abcdef');
  assert.deepEqual(calls.map(c => [c.method, c.path]), [['GET', '/api/artist/auctions/lot-1/participants'], ['GET', '/api/artist/auctions/lot-1/terms'],
    ['GET', '/api/artist/email-policy?tenantId=artist-a'], ['POST', '/api/artist/email-policy/blocks'], ['POST', '/api/artist/email-policy/blocks'],
    ['POST', `/api/artist/email-policy/blocks/${BLOCK_ID}/removal`]]);
  assert.deepEqual(JSON.parse(calls[3].body), { tenantId: 'artist-a', email: 'carla@example.test', reason: 'spam' });
  assert.deepEqual(JSON.parse(calls[4].body), { tenantId: 'artist-a', email: 'dario@example.test' });
  assert.deepEqual(JSON.parse(calls[5].body), { tenantId: 'artist-a' });
  assert.ok(calls.slice(3).every(c => /^(un)?block-/.test(c.headers['Idempotency-Key'])));
  assert.deepEqual([blocked.status, removed.status], [201, 201]);
  await assert.rejects(transport.getParticipants('../x'), /invalid_auction/);
  await assert.rejects(transport.unblockEmail('not-a-block-id', 'unblock-0123456789abcdef'), /invalid_response/);
  await assert.rejects(transport.blockEmail('a@example.test', null, 'short'), /invalid_idempotency_key/);
  await assert.rejects(createArtistV1Transport(port, () => null).getEmailPolicy(), /tenant_not_selected/);
});

test('manager: reads are validated; blocking is checked before sending and the outcome is what the server said', async () => {
  const sent = [];
  const transport = {
    async getParticipants() { return { items: [person()], contactsAvailable: true }; },
    async getTerms() { return { terms: null }; },
    async getEmailPolicy() { return { blockedEmails: [], disposableDomains: { count: 0, syncedAt: null } }; },
    async blockEmail(email, reason, key) { sent.push(['block', email, reason, key]); return { status: email.startsWith('refused') ? 409 : 201, error: email.startsWith('refused') ? 'block_unavailable' : null }; },
    async unblockEmail(id, key) { sent.push(['unblock', id, key]); return { status: id === 'f'.repeat(24) ? 404 : id === 'e'.repeat(24) ? 401 : 200, error: id === 'f'.repeat(24) ? 'not_found' : null }; },
  };
  const manager = createAuctionManager(transport, () => true);
  assert.equal((await manager.loadParticipants('lot-1')).participants[0].email, 'anna@example.test');
  assert.equal(await manager.loadTerms('lot-1'), null);
  assert.deepEqual({ ...(await manager.loadEmailPolicy()).disposableDomains }, { count: 0, syncedAt: null });
  assert.deepEqual({ ...(await manager.blockEmail(' Carla@Example.test ', '  spam  ')) }, { ok: true, code: null, needsSignIn: false });
  assert.deepEqual(sent[0].slice(0, 3), ['block', 'Carla@Example.test', 'spam']);
  assert.match(sent[0][3], /^block-[0-9a-f]{36}$/);
  assert.deepEqual({ ...(await manager.blockEmail('refused@example.test', null)) }, { ok: false, code: 'block_unavailable', needsSignIn: false });
  for (const bad of ['', 'no-address', 'a@b.test, c@d.test', 'a b@c.test'])
    await assert.rejects(manager.blockEmail(bad, null), /invalid_email/, bad);
  await assert.rejects(manager.blockEmail('a@b.test', 'x'.repeat(201)), /invalid_reason/);
  assert.deepEqual({ ...(await manager.unblockEmail('f'.repeat(24))) }, { ok: false, code: 'not_found', needsSignIn: false });
  assert.equal((await manager.unblockEmail('e'.repeat(24))).needsSignIn, true);
  await assert.rejects(manager.unblockEmail('nope'), /invalid_response/);
  const offline = createAuctionManager(transport, () => false);
  await assert.rejects(offline.loadParticipants('lot-1'), /offline/);
  await assert.rejects(offline.blockEmail('a@b.test', null), /offline/);
  const broken = createAuctionManager({ ...transport, async getParticipants() { return { items: [{}] }; } }, () => true);
  await assert.rejects(broken.loadParticipants('lot-1'), /invalid_response/);
});

test('e-mail to the winner: the Gestionale template, one line subject, the address checked', () => {
  const draft = winnerMailDraft({ email: ' Anna@Example.test ', name: 'Anna\nRossi', title: 'Trinita’ di\tfichi', amountCents: 52500 });
  assert.equal(draft.to, 'Anna@Example.test');
  assert.equal(draft.subject, 'Aggiudicazione di «Trinita’ di fichi»');
  assert.match(draft.body, /^Ciao Anna Rossi,\n\ncongratulazioni! La tua offerta di 525,00\s€ si è aggiudicata l’opera «Trinita’ di fichi»\.\n\nTi scrivo per concordare i prossimi passaggi\.$/);
  assert.match(winnerMailDraft({ email: 'a@b.test', name: null, title: 'Opera', amountCents: 100 }).body, /^Ciao collezionista,/);
  for (const bad of [{ email: 'no', amountCents: 1 }, { email: 'a@b.test, c@d.test', amountCents: 1 }, { email: 'a@b.test', amountCents: -1 }, { email: 'a@b.test', amountCents: 1.5 }])
    assert.throws(() => winnerMailDraft({ name: 'A', title: 'T', ...bad }), /invalid_/);
});
