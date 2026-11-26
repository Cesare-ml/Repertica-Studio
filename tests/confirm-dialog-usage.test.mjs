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
