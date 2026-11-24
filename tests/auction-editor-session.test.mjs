import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AuctionEditorSession } from '../src/app/auction-editor-session.ts';

const artwork = { id: 'a', title: 'Art', status: 'available' };
const pending = { artworkId: 'a', idempotencyKey: 'preview-0123456789', body: '{"synthetic":true}' };
function storage() {
  const records = new Map();
  let counter = 0;
  return {
    read: async artworkId => records.get(artworkId) ?? { artworkId, scope: 'archive', revision: null, recordJson: null },
    decode: (snapshot, artworkId) => {
      assert.equal(snapshot.artworkId, artworkId);
      return snapshot.recordJson === null ? null : JSON.parse(snapshot.recordJson);
    },
    newDraft: work => {
      if (work.status !== 'available') throw new Error('artwork_not_available');
      return { artworkId: work.id, title: work.title, description: '', slug: '', startsAt: '', endsAt: '',
        startingAmount: null, minimumIncrementAmount: null };
    },
    save: async (snapshot, draft) => {
      const previous = records.get(snapshot.artworkId);
      if ((previous?.revision ?? null) !== snapshot.revision) throw new Error('auction_draft_conflict');
      const saved = { ...snapshot, revision: String(++counter), recordJson: JSON.stringify({ draft,
        pending: previous ? JSON.parse(previous.recordJson).pending : null }) };
      records.set(snapshot.artworkId, saved);
      return saved;
    },
    records,
  };
}

test('one artwork, explicit save and no silent loss when switching', async () => {
  const port = storage(), editor = new AuctionEditorSession(port);
  await editor.open(artwork);
  const view = editor.view;
  view.draft.title = 'not an edit';
  assert.equal(editor.view.draft.title, 'Art');
  editor.edit({ ...editor.view.draft, title: 'Edited', artworkId: 'b', token: 'not persisted' });
  assert.equal(editor.view.draft.artworkId, 'a');
  assert.equal('token' in editor.view.draft, false);
  assert.throws(() => editor.close(), /auction_editor_unsaved/);
  await assert.rejects(editor.open({ ...artwork, id: 'b' }), /auction_editor_unsaved/);
  assert.equal(port.records.size, 0);
  await editor.save();
  assert.equal(editor.view.saved, true);
  assert.equal(editor.view.dirty, false);
  editor.close();
  await editor.open(artwork);
  assert.equal(editor.view.draft.title, 'Edited');
  assert.equal(port.records.size, 1);
});

test('a failed CAS keeps unsaved edits and never silently retries', async () => {
  const port = storage(), first = new AuctionEditorSession(port), second = new AuctionEditorSession(port);
  await first.open(artwork);
  await second.open(artwork);
  first.edit({ ...first.view.draft, title: 'First' });
  second.edit({ ...second.view.draft, title: 'Second' });
  await first.save();
  await assert.rejects(second.save(), /auction_draft_conflict/);
  assert.equal(second.view.dirty, true);
  assert.equal(second.view.draft.title, 'Second');
  assert.equal(second.busy, false);
  second.close(true);
  await second.open(artwork);
  assert.equal(second.view.draft.title, 'First');
});

test('an in-flight read or save blocks switching, closing and editing', async () => {
  const port = storage(), editor = new AuctionEditorSession(port);
  const read = editor.open(artwork);
  assert.throws(() => editor.close(true), /auction_editor_busy/);
  await assert.rejects(editor.open({ ...artwork, id: 'b' }), /auction_editor_busy/);
  await read;
  const save = editor.save();
  assert.throws(() => editor.edit(editor.view.draft), /auction_editor_busy/);
  await assert.rejects(editor.open({ ...artwork, id: 'b' }), /auction_editor_busy/);
  await save;
  assert.equal(editor.view.artwork.id, 'a');
});

test('a retained pending request stays immutable; sold work cannot start a new draft', async () => {
  const port = storage(), editor = new AuctionEditorSession(port);
  await editor.open(artwork);
  await editor.save();
  const saved = port.records.get('a');
  const record = JSON.parse(saved.recordJson);
  port.records.set('a', { ...saved, recordJson: JSON.stringify({ ...record, pending }) });
  editor.close();
  await editor.open({ ...artwork, status: 'sold' });
  editor.view.pending.body = 'changed';
  editor.edit({ ...editor.view.draft, title: 'Local correction' });
  await editor.save();
  assert.deepEqual(editor.view.pending, pending);
  await assert.rejects(editor.open({ ...artwork, id: 'b', status: 'sold' }), /artwork_not_available/);
  assert.equal(editor.view.artwork.id, 'a');
});

test('overlong source titles are not truncated; draft stays local and incomplete', async () => {
  const editor = new AuctionEditorSession(storage());
  const longTitle = '🎨'.repeat(81);
  await editor.open({ ...artwork, title: longTitle });
  assert.equal(editor.view.artwork.title, longTitle);
  assert.equal(editor.view.draft.title, '');
  await editor.save();
  assert.equal(editor.view.saved, true);
});

test('a management-list selection cannot reopen a different archive after restore', async () => {
  const port = storage(), editor = new AuctionEditorSession(port);
  await assert.rejects(editor.open(artwork, 'previous-archive'), /auction_draft_conflict/);
  assert.equal(editor.view, null);
  assert.equal(editor.busy, false);
  await editor.open(artwork, 'archive');
  assert.equal(editor.view.artwork.id, 'a');
  assert.equal(port.records.size, 0);
});
