/**
 * Validate and normalize the PUBLIC (never private) Tauri v2 updater key.
 *
 * Tauri expects base64( minisign public-key text ), including the untrusted
 * comment line. Accepts the CLI output, its decoded two-line form, or a bare
 * minisign public-key payload. No signing key material is generated here.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function decodeBase64(value) {
  const compact = value.replace(/\s+/g, '');
  if (!/^[A-Za-z0-9+/]+={0,2}$/.test(compact)) throw new Error('Invalid base64 encoding');
  const decoded = Buffer.from(compact, 'base64');
  if (decoded.toString('base64').replace(/=+$/, '') !== compact.replace(/=+$/, '')) {
    throw new Error('Non-canonical base64 encoding');
  }
  return decoded;
}

export function normalizeUpdaterPublicKey(value) {
  let key = String(value ?? '').trim().replace(/\r\n?/g, '\n');
  if (!key) throw new Error('Missing REPERTICA_UPDATER_PUBLIC_KEY');
  // GitHub Variables sometimes contain an escaped newline instead of a real one.
  if (key.startsWith('untrusted comment:')) key = key.replace(/\\n/g, '\n');
  if (!key.startsWith('untrusted comment:')) {
    const decoded = decodeBase64(key);
    // A generated .key.pub value is already base64 of the two-line minisign file.
    const asText = decoded.toString('utf8').trim();
    if (asText.startsWith('untrusted comment:')) {
      key = asText;
    } else if (decoded.length === 42 && decoded.subarray(0, 2).toString('ascii') === 'Ed') {
      // Some users copy just the raw RW... line from the minisign file.
      key = 'untrusted comment: minisign public key\n' + key;
    } else {
      throw new Error('Updater public key must be base64(minisign text) or a minisign Ed25519 public key');
    }
  }

  const lines = key.split('\n').map(line => line.trim()).filter(Boolean);
  if (lines.length !== 2 || !/^untrusted comment:\s*.+/.test(lines[0])) {
    throw new Error('Updater public key must include one untrusted comment and one base64 payload');
  }
  const binary = decodeBase64(lines[1]);
  if (binary.length !== 42 || binary.subarray(0, 2).toString('ascii') !== 'Ed') {
    throw new Error('Updater public key payload is not a minisign Ed25519 public key');
  }
  // The comment is intentionally untrusted; keep the real key bytes unchanged.
  return Buffer.from(lines.join('\n') + '\n', 'utf8').toString('base64');
}

export function configureReleasePublicKey(encode, configFile) {
  const value = JSON.parse(fs.readFileSync(configFile, 'utf8'));
  value.plugins ??= {};
  value.plugins.updater ??= {};
  value.plugins.updater.pubkey = encode;
  fs.writeFileSync(configFile, JSON.stringify(value, null, 2) + '\n');
}

function main() {
  const args = process.argv.slice(2);
  if (args.some(x => !['--check', '--write'].includes(x)) || (args.includes('--check') && args.includes('--write'))) {
    throw new Error('Usage: node scripts/normalize-updater-public-key.mjs [--check | --write]');
  }
  const normalized = normalizeUpdaterPublicKey(process.env.REPERTICA_UPDATER_PUBLIC_KEY || process.env.PUBLIC_KEY);
  const write = args.includes('--write');
  if (write) {
    const file = path.join(process.cwd(), 'src-tauri', 'tauri.release.conf.json');
    configureReleasePublicKey(normalized, file);
    // Make the *same* verified encoding available to Rust's option_env! macro.
    if (process.env.GITHUB_ENV) {
      fs.appendFileSync(process.env.GITHUB_ENV, 'REPERTICA_UPDATER_PUBLIC_KEY=' + normalized + '\n');
    }
  }
  // Never print key contents or the private signing secret into CI logs.
  console.log(write
    ? 'Valid public updater key normalized for Tauri configuration and Rust build.'
    : 'Public updater key format validated before compilation.');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); }
  catch (error) {
    console.error('Updater key preflight failed:', error.message);
    process.exitCode = 1;
  }
}
