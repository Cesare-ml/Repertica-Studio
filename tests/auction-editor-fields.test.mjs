import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseAuctionEuros, normalizeAuctionUtc, fieldsFromAuctionDraft, auctionDraftEdits, auctionEditorError } from '../src/app/auction-editor-fields.ts';

test('euro input is exact, bounded and supports decimal comma', () => {
  for (const [value, cents] of [['', null], ['  ', null], ['0', 0], ['0,01', 1], ['1.10', 110], ['12,3', 1230], ['10000000', 1000000000]]) {
    assert.equal(parseAuctionEuros(value), cents);
  }
  for (const value of ['-1', '1e2', '0.001', '1,000.00', '10000000.01', 'Infinity']) {
    assert.throws(() => parseAuctionEuros(value), /invalid_amount/);
  }
});

test('UTC date input is deterministic and invalid unfinished values stay intact', () => {
  assert.equal(normalizeAuctionUtc('2027-03-04T10:15'), '2027-03-04T10:15:00.000Z');
  assert.equal(normalizeAuctionUtc('2027-03-04T10:15:20Z'), '2027-03-04T10:15:20.000Z');
  for (const value of ['', '2027-02-30T10:15', '2027-03-04T10:15:00+01:00', 'unfinished']) assert.equal(normalizeAuctionUtc(value), value);
});

test('editor projection preserves empty values and excludes private fields', () => {
  const draft = { artworkId: 'one', title: 'Art', description: '', slug: '', startsAt: '', endsAt: '', startingAmount: 110, minimumIncrementAmount: null, buyer: 'private' };
  const fields = fieldsFromAuctionDraft(draft);
  assert.equal(fields.startingEuros, '1.10');
  assert.equal(fields.incrementEuros, '');
  assert.equal('buyer' in fields, false);
  const edits = auctionDraftEdits({ ...fields, artworkId: 'other', token: 'private' });
  const { artworkId, buyer, ...expected } = draft;
  assert.deepEqual(edits, expected);
});

test('only safe error labels are exposed', () => {
  assert.match(auctionEditorError('auction_draft_conflict'), /cambiata/);
  assert.match(auctionEditorError(new Error('invalid_amount')), /decimali/);
  assert.doesNotMatch(auctionEditorError(new Error('C:/private/token')), /private|token/);
});
