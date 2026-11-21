import assert from 'node:assert/strict';
import { test } from 'node:test';
import { normalizeUpdaterPublicKey } from '../scripts/normalize-updater-public-key.mjs';

// Synthetic *public* minisign payload; not an actual release key pair.
const raw = Buffer.concat([Buffer.from('Ed', 'ascii'), Buffer.alloc(40, 0x01)]).toString('base64');
const minisign = 'untrusted comment: minisign public key: TEST\n' + raw + '\n';
const normalized = Buffer.from(minisign).toString('base64');

test('Tauri-generated encoded public key stays a usable encoded minisign key', () => {
  assert.equal(normalizeUpdaterPublicKey(normalized), normalized);
});
test('raw minisign public key text gets base64-wrapped', () => {
  assert.equal(normalizeUpdaterPublicKey(minisign), normalized);
});
test('bare minisign payload is safely given an untrusted comment and encoded', () => {
  const value = normalizeUpdaterPublicKey(raw);
  assert.equal(Buffer.from(value, 'base64').toString('utf8').split('\n')[1], raw);
});
test('escaped newlines from GitHub variables are supported', () => {
  assert.equal(normalizeUpdaterPublicKey(minisign.replace(/\n/g, '\\n')), normalized);
});
test('a malformed key fails early instead of failing after a 7 minute build', () => {
  for (const val of ['', 'abc', 'YWJjZA==', 'not a public key', 'untrusted comment: bad\nabc']) {
    assert.throws(() => normalizeUpdaterPublicKey(val));
  }
});
test('a mismatched or invalid payload is never reconstructed into a new key', () => {
  const bogus = Buffer.alloc(42, 0xff).toString('base64');
  assert.throws(() => normalizeUpdaterPublicKey(bogus));
});
