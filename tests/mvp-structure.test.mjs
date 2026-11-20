import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('no cloud or Sites dependency in runtime', () => {
  const cargo = get('src-tauri/Cargo.toml');
  const api = get('src/app/api.ts');
  assert.doesNotMatch(api, /fetch\(|axios|http:\/\/|https:\/\//);
  assert.doesNotMatch(cargo, /reqwest|sqlx|postgres|aws-sdk/);
});
test('four MVP modules available via typed native IPC', () => {
  const native = get('src-tauri/src/lib.rs');
  for (const name of ['list_artworks','list_collectors','list_sales','list_certificates','issue_certificate']) {
    assert.ok(native.includes('commands::' + name));
  }
});
test('CPAL license attribution correctly names Repertica', () => {
  const license = get('LICENSE');
  assert.match(license, /Common Public Attribution License Version 1\.0/);
  assert.match(license, /Powered by Repertica/);
  assert.doesNotMatch(license, /Powered by VirgoStars/);
});
