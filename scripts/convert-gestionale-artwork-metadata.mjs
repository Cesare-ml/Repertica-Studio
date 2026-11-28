// Explicit, local conversion of a user-selected legacy metadata JSON file.
// No database connection, HTTP, credentials, or copying media paths.
import { open } from 'node:fs/promises';
import { publishLocalPackage } from './publish-local-package.mjs';
import { pathToFileURL } from 'node:url';
import { parseLegacyJson } from './strict-legacy-json.mjs';

const MAX_BYTES = 2 * 1024 * 1024;
const MAX_ROWS = 1000;
const FORMAT = 'repertica-studio-artwork-metadata-v1';
const fields = new Set(['codice_archivio', 'titolo', 'anno_realizzazione', 'tecnica', 'supporto',
  'larghezza_cm', 'altezza_cm', 'profondita_cm', 'descrizione', 'stato', 'firma_presente',
  'cornice_presente', 'vernice_finale', 'coefficiente_artista_override', 'prezzo_personalizzato', 'id_vendita']);

function text(value, max, required = false) {
  if (value == null && !required) return null;
  if (typeof value !== 'string') throw Error('invalid text');
  const result = value.trim();
  if ([...result].length > max || result.includes('\0') || (required && (!result || /\p{Cc}/u.test(result)))) throw Error('invalid text');
  return result || null;
}
function positive(value) {
  if (value == null) return null;
  if (typeof value !== 'number' && (typeof value !== 'string' || !/^\d+(?:\.\d+)?$/.test(value))) throw Error('invalid measurement or coefficient');
  const number = Number(value);
  if (!Number.isFinite(number) || number <= 0 || number > 100000) throw Error('invalid measurement or coefficient');
  return number;
}
function cents(value) {
  if (value == null) return null;
  if (typeof value !== 'string' && typeof value !== 'number') throw Error('invalid price');
  const match = /^(\d{1,10})(?:\.(\d{1,2}))?$/.exec(String(value));
  if (!match) throw Error('price must be non-negative EUR with at most two decimals');
  const amount = BigInt(match[1]) * 100n + BigInt((match[2] || '').padEnd(2, '0'));
  if (amount > 100000000000n) throw Error('price exceeds limit');
  return Number(amount);
}
function flag(value) {
  if (typeof value !== 'boolean') throw Error('resolve unknown signature/frame facts explicitly');
  return value;
}
function varnish(value) {
  if (value === false) return false; // explicit no, confirmed in the selected source file
  if (['OPACA', 'LUCIDA', 'SATINATA'].includes(value)) return true;
  throw Error('resolve unknown varnish explicitly (false if not applied)');
}
function year(value) {
  if (value == null) return null;
  if (!/^-?\d{1,6}$/.test(String(value)) || !['string', 'number'].includes(typeof value)) throw Error('invalid year');
  return String(value);
}
export function convertLegacyRows(rows) {
  if (!Array.isArray(rows) || rows.length < 1 || rows.length > MAX_ROWS) throw Error('select 1–1000 rows');
  const seen = new Set();
  const artworks = rows.map((row, index) => {
    try {
      if (!row || typeof row !== 'object' || Array.isArray(row) || Object.keys(row).some(key => !fields.has(key))) throw Error('unexpected field; use the explicit metadata projection');
      // Never convert commercial or ambiguous states into available artworks.
      const status = row.stato === 'DISPONIBILE' ? 'available' : row.stato === 'RISERVATA' ? 'reserved' : null;
      if (!status || row.id_vendita !== null) throw Error('commercial or unsupported state; needs manual migration');
      const code = text(row.codice_archivio, 80, true);
      const suffix = /\d+$/.exec(code)?.[0];
      if (suffix && BigInt(suffix) === 9223372036854775807n) throw Error('archive code sequence exhausted');
      const folded = code.replace(/[A-Z]/g, c => c.toLowerCase());
      if (seen.has(folded)) throw Error('duplicate archive code');
      seen.add(folded);
      return { code, title: text(row.titolo, 240, true), year: year(row.anno_realizzazione),
        technique: text(row.tecnica, 300), support: text(row.supporto, 300),
        width_cm: positive(row.larghezza_cm), height_cm: positive(row.altezza_cm), depth_cm: positive(row.profondita_cm),
        description: text(row.descrizione, 40000), status, signed: flag(row.firma_presente),
        framed: flag(row.cornice_presente), final_varnish: varnish(row.vernice_finale),
        artist_coefficient_override: positive(row.coefficiente_artista_override), fixed_price_cents: cents(row.prezzo_personalizzato) };
    } catch (error) { throw Error(`Row ${index + 1}: ${error.message}`); }
  });
  const result = JSON.stringify({ format: FORMAT, artworks }, null, 2) + '\n';
  if (Buffer.byteLength(result, 'utf8') > MAX_BYTES) throw Error('converted package exceeds 2 MiB; select fewer rows');
  return result;
}
export async function convertFile(input, output) {
  const file = await open(input, 'r');
  let source;
  try {
    const stat = await file.stat();
    if (!stat.isFile() || stat.size > MAX_BYTES) throw Error('input must be a regular file up to 2 MiB');
    // Bound the read even if the file changes after stat().
    const bytes = Buffer.alloc(MAX_BYTES + 1);
    let offset = 0;
    while (offset < bytes.length) {
      const { bytesRead } = await file.read(bytes, offset, bytes.length - offset, null);
      if (!bytesRead) break;
      offset += bytesRead;
    }
    if (offset > MAX_BYTES) throw Error('input exceeds 2 MiB');
    source = new TextDecoder('utf-8', { fatal: true }).decode(bytes.subarray(0, offset));
  } finally { await file.close(); }
  const result = convertLegacyRows(parseLegacyJson(source));
  // Explicit new destination; never replace an existing export or source file.
  await publishLocalPackage(output, result);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.argv.length !== 4) throw Error('Usage: node scripts/convert-gestionale-artwork-metadata.mjs selected.json import.json');
    await convertFile(process.argv[2], process.argv[3]);
    console.log('Metadata package written. Review it in Repertica Studio before importing.');
  } catch (error) {
    // Do not print raw JSON, absolute paths or exception details from the filesystem.
    console.error(error.code ? 'File operation failed; check input and choose a new output file.' : error instanceof SyntaxError ? 'Invalid JSON.' : error.message);
    process.exitCode = 1;
  }
}
