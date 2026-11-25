// Publish only a fully written package, without replacing any existing destination.
import { link, mkdtemp, open, rm } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';

export async function publishLocalPackage(output, data) {
  const destination = resolve(output);
  // The staging directory is private and on the same filesystem as the destination.
  const staging = await mkdtemp(join(dirname(destination), '.repertica-export-'));
  const temporary = join(staging, 'package.json');
  try {
    const file = await open(temporary, 'wx', 0o600);
    try {
      await file.writeFile(data, 'utf8');
      await file.sync();
    } finally {
      await file.close();
    }
    // A hard link publishes atomically and fails if the destination already exists,
    // including a symlink. Do not fall back to an overwrite-capable rename or copy.
    await link(temporary, destination);
  } finally {
    // A failed cleanup must not misreport a successful publication as a failed one.
    // A crash or filesystem failure can leave a private staging directory behind.
    await rm(staging, { recursive: true, force: true }).catch(() => {});
  }
}
