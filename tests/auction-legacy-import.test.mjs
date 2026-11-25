import { test } from 'node:test';
import assert from 'node:assert/strict';
import { previewLegacyDraft } from '../src/app/auction-legacy-import.ts';

const work = { id: 'art-1', status: 'available' };
const snapshot = { artworkId: 'art-1', scope: 'archive-a', revision: null, recordJson: null };
const legacy = Object.freeze({ artworkId: 'art-1', local: true, updatedAt: '2026-11-25T10:00:00.000Z',
  title: 'Legacy art', description: 'Public description', slug: '', startsAt: '', endsAt: '',
  startingAmount: null, minimumIncrementAmount: 100, token: 'synthetic private field' });

test('one explicit legacy review projects only draft fields and preserves incomplete data', () => {
  const preview = previewLegacyDraft(legacy, work, snapshot, 'archive-a');
  assert.deepEqual(preview, { artworkId: 'art-1', title: 'Legacy art', description: 'Public description',
    slug: '', startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: 100 });
  preview.title = 'Edited';
  assert.equal(legacy.title, 'Legacy art');
  assert.equal(snapshot.recordJson, null);
});

test('a different archive, existing draft or inconsistent snapshot cannot be imported', () => {
  for (const changed of [{ scope: 'archive-b' }, { revision: 'saved', recordJson: '{}' },
    { revision: 'saved' }, { recordJson: '{}' }, { artworkId: 'art-2' }]) {
    assert.throws(() => previewLegacyDraft(legacy, work, { ...snapshot, ...changed }, 'archive-a'), /auction_draft_conflict/);
  }
});

test('missing, sold or different work cannot inherit an unscoped legacy draft', () => {
  for (const changed of [{ status: 'sold' }, { status: 'missing' }]) {
    assert.throws(() => previewLegacyDraft(legacy, { ...work, ...changed }, snapshot, 'archive-a'), /artwork_not_available/);
  }
  assert.throws(() => previewLegacyDraft({ ...legacy, artworkId: 'art-2' }, work, snapshot, 'archive-a'), /artwork_not_available/);
  assert.throws(() => previewLegacyDraft({ ...legacy, local: false }, work, snapshot, 'archive-a'), /artwork_not_available/);
});
