import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, readdir, rm, writeFile, symlink, lstat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { publishLocalPackage } from '../scripts/publish-local-package.mjs';

async function withDirectory(run) {
  const dir = await mkdtemp(join(tmpdir(), 'repertica-publish-test-'));
  try { await run(dir); }
  finally { await rm(dir, { recursive: true, force: true }); }
}

test('publishes exact UTF-8 bytes and removes private staging', async () => withDirectory(async dir => {
  const output = join(dir, 'export.json');
  const data = '{"title":"Étude"}\n';
  await publishLocalPackage(output, data);
  assert.deepEqual(await readFile(output), Buffer.from(data));
  assert.deepEqual(await readdir(dir), ['export.json']);
  if (process.platform !== 'win32') assert.equal((await lstat(output)).mode & 0o077, 0);
}));

test('existing file and failed writes leave destination unchanged and no staging', async () => withDirectory(async dir => {
  const output = join(dir, 'export.json');
  await writeFile(output, 'original');
  await assert.rejects(publishLocalPackage(output, 'replacement'), { code: 'EEXIST' });
  assert.equal(await readFile(output, 'utf8'), 'original');
  const invalid = join(dir, 'invalid.json');
  await assert.rejects(publishLocalPackage(invalid, null), { code: 'ERR_INVALID_ARG_TYPE' });
  assert.deepEqual(await readdir(dir), ['export.json']);
}));

test('concurrent publishers cannot overwrite each other or expose mixed bytes', async () => withDirectory(async dir => {
  const output = join(dir, 'export.json');
  const packages = ['a'.repeat(128000), 'b'.repeat(128000)];
  const results = await Promise.allSettled(packages.map(data => publishLocalPackage(output, data)));
  assert.equal(results.filter(result => result.status === 'fulfilled').length, 1);
  const winner = results.findIndex(result => result.status === 'fulfilled');
  assert.equal(results[1 - winner].reason.code, 'EEXIST');
  assert.equal(await readFile(output, 'utf8'), packages[winner]);
  assert.deepEqual(await readdir(dir), ['export.json']);
}));

test('never follows or replaces an existing destination symlink', { skip: process.platform === 'win32' }, async () => withDirectory(async dir => {
  const target = join(dir, 'target');
  const output = join(dir, 'export.json');
  await writeFile(target, 'private');
  await symlink(target, output);
  await assert.rejects(publishLocalPackage(output, 'replacement'), { code: 'EEXIST' });
  assert.equal(await readFile(target, 'utf8'), 'private');
  assert.ok((await lstat(output)).isSymbolicLink());
  assert.deepEqual((await readdir(dir)).sort(), ['export.json', 'target']);
}));
