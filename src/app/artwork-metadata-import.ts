// Only explicit file selection; no native filesystem paths, no remote access.
export const MAX_IMPORT_BYTES = 16 * 1024 * 1024;
export interface ArtworkMetadataPreview {
  digest: string;
  covers: {artwork_code: string; sha256: string; size_bytes: number; position: number; caption: string | null; kind: string}[];
  artworks: {code: string; title: string; year: string | null; status: string}[];
  collectors: {key: string; first_name: string; last_name: string | null; email: string | null; birth_date: string | null; birth_time: string | null; birth_place: string | null; instagram: string | null; facebook: string | null; tiktok: string | null; website: string | null; linkedin: string | null}[];
  sales: {artwork_code: string; collector_key: string; sold_at: string; amount_cents: number; currency: string; tax_cents: number; shipping_cents: number; installments: {position: number; due_date: string; amount_cents: number; notes: string | null}[]; receipts: {received_at: string; amount_cents: number; method: string}[]}[];
}
export async function readImportFile(file: Pick<File, 'size' | 'arrayBuffer'>): Promise<string> {
  if (file.size > MAX_IMPORT_BYTES) throw new Error('size');
  const bytes = await file.arrayBuffer();
  if (bytes.byteLength > MAX_IMPORT_BYTES) throw new Error('size');
  try { return new TextDecoder('utf-8', {fatal:true}).decode(bytes); }
  catch { throw new Error('format'); }
}
export function importErrorParts(error: unknown): {key: string; row: string} {
  const raw = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  const [key, row = ''] = raw.split(':');
  return {
    key: ['size','format','count','invalid_row','duplicate','changed','database','relations','collector_conflict'].includes(key) ? key : 'database',
    row: /^[1-9]\d{0,3}$/.test(row) ? row : ''
  };
}
