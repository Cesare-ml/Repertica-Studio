import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { isQuietStatus, messageKeys, progressPercent, updateMessage, UPDATE_CHECK_INTERVAL_MS } from '../src/app/updates.ts';

test('every update outcome has a message and silent checks stay quiet', () => {
  const statuses = ['available', 'current', 'not_configured', 'check_failed', 'invalid_release', 'busy', 'stale',
    'backup_failed', 'download_failed', 'signature_invalid', 'install_failed', 'installing'];
  for (const status of statuses) assert.ok(updateMessage(status).length > 10, status);
  for (const status of ['current', 'not_configured', 'check_failed', 'busy']) assert.ok(isQuietStatus(status), status);
  for (const status of ['available', 'signature_invalid', 'backup_failed', 'install_failed']) assert.ok(!isQuietStatus(status), status);
  assert.equal(UPDATE_CHECK_INTERVAL_MS, 6 * 3600 * 1000);
});

test('download progress is a whole percent and unknown sizes give no percentage', () => {
  assert.equal(progressPercent(0, 1000), 0);
  assert.equal(progressPercent(500, 1000), 50);
  assert.equal(progressPercent(999, 1000), 99);
  assert.equal(progressPercent(1500, 1000), 100);
  assert.equal(progressPercent(10, null), null);
  assert.equal(progressPercent(10, 0), null);
});

test('update messages are translated in every language', () => {
  const dictionary = readFileSync(new URL('../src/app/i18n.ts', import.meta.url), 'utf8');
  for (const message of messageKeys()) assert.ok(dictionary.includes(message), message);
});
