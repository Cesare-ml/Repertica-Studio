import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { confirmed } from '../src/app/confirm-dialog.ts';

test('confirmed() waits for the asynchronous dialog of the Desktop app', async () => {
  // Tauri's dialog plugin: window.confirm returns a Promise, which is truthy before the person answers.
  globalThis.window = { confirm: () => Promise.resolve(false) };
  assert.equal(await confirmed('Delete?'), false);
  globalThis.window = { confirm: () => Promise.resolve(true) };
  assert.equal(await confirmed('Delete?'), true);
  // A browser's synchronous confirm.
  globalThis.window = { confirm: () => false };
  assert.equal(await confirmed('Delete?'), false);
  globalThis.window = { confirm: () => true };
  assert.equal(await confirmed('Delete?'), true);
  // Without the dialog permission the plugin rejects: nothing was asked, so nothing is confirmed.
  globalThis.window = { confirm: () => Promise.reject(new Error('dialog.message not allowed')) };
  assert.equal(await confirmed('Delete?'), false);
  delete globalThis.window;
});

test('the Desktop may show the confirmation dialog', () => {
  const capability = JSON.parse(readFileSync(new URL('../src-tauri/capabilities/default.json', import.meta.url), 'utf8'));
  assert.ok(capability.permissions.includes('dialog:allow-message'), 'window.confirm/alert need dialog:allow-message in Tauri');
});

test('the Auctions section never uses an unawaited confirm()', () => {
  const dir = new URL('../src/app/', import.meta.url);
  for (const file of readdirSync(dir).filter(name => /^auction.*\.ts$/.test(name))) {
    assert.doesNotMatch(readFileSync(new URL(file, dir), 'utf8'), /\bconfirm\(/, file);
  }
});
