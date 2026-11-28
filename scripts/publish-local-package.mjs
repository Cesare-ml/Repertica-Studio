// Publish only a fully written package, without replacing any existing destination.
import { link, lstat, mkdir, mkdtemp, open, rmdir, unlink } from 'node:fs/promises';
import { basename, dirname, join, resolve } from 'node:path';

const sameEntry = (a, b) => a.dev === b.dev && a.ino === b.ino;
const stat = path => lstat(path, { bigint: true });
async function ownedDirectory(path, identity) {
  const current = await stat(path);
  if (!current.isDirectory() || !sameEntry(current, identity)) throw Error('Export directory changed');
}
async function removeOwnedFile(dir, identity, path, file) {
  await ownedDirectory(dir, identity);
  const current = await stat(path);
  if (current.isFile() && sameEntry(current, file)) await unlink(path);
}

export async function publishLocalPackage(output, data) {
  const destination = resolve(output);
  // The staging directory is private and on the same filesystem as the destination.
  const staging = await mkdtemp(join(dirname(destination), '.repertica-export-'));
  const identity = await stat(staging);
  const temporary = join(staging, 'package.json');
  let written;
  try {
    const file = await open(temporary, 'wx', 0o600);
    try {
      written = await file.stat({ bigint: true });
      await file.writeFile(data, 'utf8');
      await file.sync();
    } finally {
      await file.close();
    }
    await ownedDirectory(staging, identity);
    if (!sameEntry(await stat(temporary), written)) throw Error('Staging file changed');
    // A hard link publishes atomically and fails if the destination already exists.
    await link(temporary, destination);
    return written;
  } finally {
    // Never recursively delete an export path: another process may have added files.
    if (written) await removeOwnedFile(staging, identity, temporary, written).catch(() => {});
    await ownedDirectory(staging, identity).then(() => rmdir(staging)).catch(() => {});
  }
}

// Track only files published by this invocation. These checks prevent ordinary
// replacement/cleanup mistakes, not adversarial races between filesystem calls.
export async function createLocalPackageDirectory(output) {
  const dir = resolve(output);
  await mkdir(dir, { mode: 0o700 });
  const identity = await stat(dir);
  const published = [];
  return {
    async publish(name, data) {
      if (basename(name) !== name || !/^(?:manifest|package-[0-9]{4})\.json$/.test(name)) throw Error('Invalid export filename');
      await ownedDirectory(dir, identity);
      const path = join(dir, name);
      const file = await publishLocalPackage(path, data);
      published.push({ path, file });
      await ownedDirectory(dir, identity);
    },
    async cleanup() {
      // Remove the manifest first if one was published; unknown files are untouched.
      for (const { path, file } of [...published].reverse()) {
        await removeOwnedFile(dir, identity, path, file).catch(() => {});
      }
      await ownedDirectory(dir, identity).then(() => rmdir(dir)).catch(() => {});
    },
  };
}
