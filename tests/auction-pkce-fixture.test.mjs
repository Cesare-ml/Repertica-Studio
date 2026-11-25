import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createPublicKey, verify } from 'node:crypto';

test('isolated OAuth fixture negatives have real signatures and differ only in intended claims', () => {
  const fixture = JSON.parse(readFileSync(new URL('../src-tauri/src/auction_pkce_http_fixture.json', import.meta.url), 'utf8'));
  const key = createPublicKey({key: fixture.jwks.keys[0], format: 'jwk'});
  const claims = {};
  for (const name of ['valid', 'wrong_scope', 'wrong_audience', 'expired']) {
    const [header, payload, signature] = fixture[name].split('.');
    assert.equal(verify('RSA-SHA256', Buffer.from(header + '.' + payload), key, Buffer.from(signature, 'base64url')), true, name);
    claims[name] = JSON.parse(Buffer.from(payload, 'base64url').toString('utf8'));
  }
  assert.equal(claims.valid.scope, 'profile');
  assert.equal(claims.valid.aud, 'repertica-auctions');
  assert.deepEqual(claims.wrong_scope, {...claims.valid, scope: 'artworks:create media:write'});
  assert.deepEqual(claims.wrong_audience, {...claims.valid, aud: 'repertica-studio-api'});
  assert.deepEqual(claims.expired, {...claims.valid, iat: 1799999100, exp: 1800000000});
});
