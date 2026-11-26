import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { confirmed } from '../src/app/confirm-dialog.ts';

const appDir = new URL('../src/app/', import.meta.url);
const sources = readdirSync(appDir, { recursive: true })
  .map(name => String(name).replaceAll('\\', '/'))
  .filter(name => name.endsWith('.ts') && name !== 'confirm-dialog.ts')
  .map(name => ({ name, text: readFileSync(new URL(name, appDir), 'utf8') }));

test('a destructive action waits for the answer and runs only on OK', async () => {
  // The shape of every call site: `if (!(await confirmed(...))) return;` followed by the action.
  let answer;
  globalThis.window = { confirm: () => new Promise(resolve => { answer = resolve; }) };
  let deleted = false;
  const remove = async () => { if (!(await confirmed('Delete?'))) return; deleted = true; };

  const cancelled = remove();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(deleted, false, 'nothing happens while the dialog is open');
  answer(false);
  await cancelled;
  assert.equal(deleted, false, 'Cancel keeps the item');

  const accepted = remove();
  answer(true);
  await accepted;
  assert.equal(deleted, true, 'OK runs the action');
  delete globalThis.window;
});

test('a dialog that cannot be shown or answers oddly counts as "no"', async () => {
  globalThis.window = { confirm: () => { throw new Error('not allowed'); } };
  assert.equal(await confirmed('Delete?'), false);
  globalThis.window = { confirm: () => Promise.resolve('Ok') };
  assert.equal(await confirmed('Delete?'), false);
  globalThis.window = { confirm: () => undefined };
  assert.equal(await confirmed('Delete?'), false);
  delete globalThis.window;
  assert.equal(await confirmed('Delete?'), false, 'no window at all');
});

test('no source in src/app calls window.confirm, alert or prompt directly', () => {
  // In the Desktop app these are asynchronous (Tauri dialog plugin): `if (!confirm(...))` never stops.
  for (const { name, text } of sources) {
    assert.doesNotMatch(text, /\b(?:confirm|alert|prompt)\s*\(/, `${name}: use await confirmed(...) from ./confirm-dialog`);
  }
});

test('every confirmation in src/app is awaited', () => {
  let calls = 0;
  for (const { name, text } of sources) {
    for (const match of text.matchAll(/\b(confirmed|canLeave)\(/g)) {
      const before = text.slice(Math.max(0, match.index - 40), match.index);
      if (/\b(?:async|function)\s+$/.test(before)) continue; // the declaration itself
      calls++;
      const line = text.slice(0, match.index).split('\n').length;
      assert.match(before, /\bawait\s+(?:this\.)?$/, `${name}:${line} ${match[1]}() must be awaited`);
    }
  }
  assert.ok(calls >= 25, `expected the confirmations of the whole app, found ${calls}`);
});

test('the archive restore confirmation shows real line breaks', () => {
  const app = readFileSync(new URL('app.component.ts', appDir), 'utf8');
  assert.match(app, /await confirmed\('RIPRISTINO ARCHIVIO\\n\\nL’archivio/);
  assert.doesNotMatch(app, /RIPRISTINO ARCHIVIO\\\\n/);
});

 
// Exercise the actual component handlers in isolation, without Angular, native IPC or SMTP.
function handler(source, start, end, signature, name, api, blankComposer) {
  const first = source.indexOf(start);
  const last = end ? source.indexOf(end, first) : source.lastIndexOf('\n}');
  assert.ok(first >= 0 && last > first, name);
  const method = source.slice(first, last).replace(signature, '()').trim();
  return new Function('confirmed', 'api', 'blankComposer', 'return ({' + method + '}).' + name)(confirmed, api, blankComposer);
}

test('restore rechecks the inspected candidate and busy state after native confirmation', async () => {
  const source = readFileSync(new URL('app.component.ts', appDir), 'utf8');
  for (const scenario of ['cancel', 'reject', 'accept', 'changed', 'busy']) {
    let answer, reject;
    const calls = [];
    globalThis.window = { confirm: () => new Promise((resolve, fail) => { answer = resolve; reject = fail; }) };
    const restore = handler(source, '  async restoreSelectedBackup()', '\n  bytes(', '(): Promise<void>', 'restoreSelectedBackup',
      { restoreBackup: async path => { calls.push(path); return { previousArchive: 'previous' }; } });
    const ctx = { busy: false, restoreCandidate: { directory: 'backup-A' }, closeGallery() {},
      refresh: async () => {}, loadAllocationPlan: async () => {}, loadSchedule: async () => {},
      error: String, change: { detectChanges() {} } };
    try {
      const pending = restore.call(ctx);
      assert.deepEqual(calls, [], scenario);
      if (scenario === 'changed') ctx.restoreCandidate = { directory: 'backup-B' };
      if (scenario === 'busy') ctx.busy = true;
      if (scenario === 'reject') reject(new Error('permission denied'));
      else answer(scenario !== 'cancel');
      await pending;
      assert.deepEqual(calls, scenario === 'accept' ? ['backup-A'] : [], scenario);
      if (scenario === 'changed') assert.equal(ctx.restoreCandidate.directory, 'backup-B');
      if (scenario === 'busy') assert.equal(ctx.busy, true);
    } finally { delete globalThis.window; }
  }
});

test('mail confirmation cannot authorize a changed message or account or duplicate send', async () => {
  const source = readFileSync(new URL('business-mail.component.ts', appDir), 'utf8');
  for (const scenario of ['cancel', 'reject', 'accept', 'account', 'body', 'attachment', 'busy', 'double']) {
    let answer, reject, prompts = 0;
    const calls = [];
    const blank = () => ({ id: null, revision: null, to: '', cc: '', bcc: '', subject: '', body: '', attachments: [] });
    globalThis.window = { confirm: () => { prompts++; return new Promise((resolve, fail) => { answer = resolve; reject = fail; }); } };
    const send = handler(source, '  async send()', null, '():Promise<void>', 'send',
      { mailSend: async input => { calls.push(input); return { delivery: 'accepted', sent_copy: 'saved' }; } }, blank);
    const ctx = { busy: false, sendConfirmationPending: false, selectedId: 'account-A', composeOpen: true,
      composer: { ...blank(), to: 'recipient@example.test', body: 'original', attachments: ['original.pdf'] },
      t: value => value, operation: async fn => fn() };
    try {
      const pending = send.call(ctx);
      assert.deepEqual(calls, [], scenario);
      if (scenario === 'account') ctx.selectedId = 'account-B';
      if (scenario === 'body') ctx.composer.body = 'changed';
      if (scenario === 'attachment') ctx.composer.attachments.push('another.pdf');
      if (scenario === 'busy') ctx.busy = true;
      if (scenario === 'double') { await send.call(ctx); assert.equal(prompts, 1); }
      if (scenario === 'reject') reject(new Error('permission denied'));
      else answer(scenario !== 'cancel');
      await pending;
      assert.equal(calls.length, ['accept', 'double'].includes(scenario) ? 1 : 0, scenario);
      if (calls.length) {
        assert.equal(calls[0].account_id, 'account-A');
        assert.equal(calls[0].body, 'original');
        assert.deepEqual(calls[0].attachments, ['original.pdf']);
      }
      assert.equal(ctx.sendConfirmationPending, false, scenario);
    } finally { delete globalThis.window; }
  }
});
