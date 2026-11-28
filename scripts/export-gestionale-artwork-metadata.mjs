// Opt-in source-side exporter for a LOCAL COPY; no database access on app startup.
import { spawnSync } from 'node:child_process';
import { publishLocalPackage } from './publish-local-package.mjs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { convertLegacyRows } from './convert-gestionale-artwork-metadata.mjs';
import { parseLegacyJson } from './strict-legacy-json.mjs';

const query = fileURLToPath(new URL('./gestionale-artwork-export.sql', import.meta.url));
const MAX_BYTES = 2 * 1024 * 1024;
const positiveId = value => /^[1-9]\d{0,9}$/.test(value) && Number(value) <= 2147483647;

export function exportOptions(args) {
  const [ack, database, user, port, owner, selection, output] = args;
  const ids = selection?.split(',') ?? [];
  if (args.length !== 7 || ack !== '--local-copy' ||
      !/^[A-Za-z0-9_][A-Za-z0-9_-]{0,62}$/.test(database ?? '') || !/^[A-Za-z0-9_][A-Za-z0-9_-]{0,62}$/.test(user ?? '') ||
      !positiveId(port) || Number(port) > 65535 || !positiveId(owner) ||
      !ids.length || ids.length > 1000 || !ids.every(positiveId) || new Set(ids).size !== ids.length || !output) {
    throw Error('Usage: node scripts/export-gestionale-artwork-metadata.mjs --local-copy DATABASE USER PORT OWNER_ID ARTWORK_ID[,ARTWORK_ID...] NEW_OUTPUT.json');
  }
  return { output, psqlArgs: ['-X', '-w', '-qAt', '-h', '127.0.0.1', '-p', port, '-d', database, '-U', user,
    '-v', 'ON_ERROR_STOP=1', '-v', `owner_id=${owner}`, '-v', `artwork_ids={${ids.join(',')}}`, '-f', query] };
}

// Avoid inherited libpq service/host overrides, remote targets or session options.
export function localEnvironment(source) {
  const env = Object.fromEntries(Object.entries(source).filter(([key]) => !key.toUpperCase().startsWith('PG')));
  return { ...env, PGCONNECT_TIMEOUT: '5', PGCLIENTENCODING: 'UTF8' };
}

export function checkedPackage(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.byteLength > MAX_BYTES) throw Error('Source export exceeds 2 MiB; select fewer works.');
  const rows = parseLegacyJson(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  if (Array.isArray(rows) && rows.length === 0) throw Error('Selection rejected: missing/ineligible work, historical sale or unresolved metadata. Nothing exported.');
  return convertLegacyRows(rows);
}

export async function exportLocalCopy(args) {
  const options = exportOptions(args);
  // No shell, no remote host, no password argument, no connection-string parsing.
  const result = spawnSync('psql', options.psqlArgs, { env: localEnvironment(process.env), timeout: 15000, maxBuffer: MAX_BYTES + 65536, windowsHide: true });
  if (result.error || result.status !== 0) throw Error('Local PostgreSQL export failed; check psql, the local copy, role permissions and local password file.');
  const data = checkedPackage(result.stdout);
  await publishLocalPackage(options.output, data);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await exportLocalCopy(process.argv.slice(2));
    console.log('Local metadata package written. Review it in Studio before importing.');
  } catch (error) {
    console.error(error.code ? 'File operation failed; choose a new output file.' : error instanceof SyntaxError ? 'Invalid source response.' : error.message);
    process.exitCode = 1;
  }
}
