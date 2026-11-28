import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, readdir, rm, writeFile, rename, mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createLocalPackageDirectory, publishLocalPackage } from '../scripts/publish-local-package.mjs';

async function fixture(t) {
  const root = await mkdtemp(join(tmpdir(), 'repertica-owned-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  return root;
}

test('cleanup removes only published files and leaves unrelated data intact', async t => {
  const root = await fixture(t), dir = join(root, 'new');
  const output = await createLocalPackageDirectory(dir);
  await output.publish('package-0001.json', '{}');
  await output.publish('manifest.json', '{}');
  await writeFile(join(dir, 'notes.txt'), 'keep');
  await mkdir(join(dir, 'other'));
  await writeFile(join(dir, 'other', 'private.txt'), 'private');
  await output.cleanup();
  assert.deepEqual((await readdir(dir)).sort(), ['notes.txt', 'other']);
  assert.equal(await readFile(join(dir, 'other', 'private.txt'), 'utf8'), 'private');
});

test('cleanup and further publication refuse a replaced directory', async t => {
  const root = await fixture(t), dir = join(root, 'new'), original = join(root, 'original');
  const output = await createLocalPackageDirectory(dir);
  await output.publish('package-0001.json', '{}');
  await rename(dir, original);
  await mkdir(dir);
  await writeFile(join(dir, 'package-0001.json'), 'keep');
  await assert.rejects(output.publish('manifest.json', '{}'), /directory changed/);
  await output.cleanup();
  assert.equal(await readFile(join(dir, 'package-0001.json'), 'utf8'), 'keep');
  assert.equal(await readFile(join(original, 'package-0001.json'), 'utf8'), '{}');
});

test('cleanup preserves a replaced package and removes untouched owned files', async t => {
  const root = await fixture(t), dir = join(root, 'new');
  const output = await createLocalPackageDirectory(dir);
  await output.publish('package-0001.json', '{}');
  await output.publish('package-0002.json', '{}');
  await rename(join(dir, 'package-0001.json'), join(root, 'original.json'));
  await writeFile(join(dir, 'package-0001.json'), 'keep');
  await output.cleanup();
  assert.deepEqual(await readdir(dir), ['package-0001.json']);
  assert.equal(await readFile(join(dir, 'package-0001.json'), 'utf8'), 'keep');
});

test('ordinary failure cleanup removes an empty destination and never overwrites', async t => {
  const root = await fixture(t), dir = join(root, 'new');
  const output = await createLocalPackageDirectory(dir);
  await output.publish('package-0001.json', '{}');
  await assert.rejects(output.publish('package-0001.json', 'replacement'), { code: 'EEXIST' });
  await assert.rejects(output.publish('../manifest.json', '{}'), /filename/);
  await output.cleanup();
  assert.deepEqual(await readdir(root), []);
  const path = join(root, 'invalid.json');
  await assert.rejects(publishLocalPackage(path, null), { code: 'ERR_INVALID_ARG_TYPE' });
  assert.deepEqual(await readdir(root), []);
});
