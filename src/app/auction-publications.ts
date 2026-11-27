/**
 * Journal of publications sent to Auctions whose outcome is not confirmed yet. It survives a restart so that a
 * retry reuses exactly the same request body and idempotency key (no duplicate auction). Only public fields are
 * stored: the body is what the artist reviewed and sent, never the token or private archive data.
 */
import type { Operation } from './auction-operations';
import type { PreparedPreview } from './auction-preview';

export interface PendingPublication {
  readonly artworkId: string;
  readonly tenantId: string;
  readonly title: string;
  readonly prepared: PreparedPreview;
  readonly operation: Operation;
}

export interface KeyValueStore { getItem(key: string): string | null; setItem(key: string, value: string): void }

const KEY = 'repertica-auction-publications-v1';
const KEY_RE = /^[A-Za-z0-9_-]{16,96}$/;

function valid(value: unknown): value is PendingPublication {
  if (!value || typeof value !== 'object') return false;
  const v = value as Record<string, unknown>;
  const p = v['prepared'] as Record<string, unknown> | undefined, op = v['operation'] as Record<string, unknown> | undefined;
  if (typeof v['artworkId'] !== 'string' || typeof v['tenantId'] !== 'string' || typeof v['title'] !== 'string'
      || !p || !op || typeof p['body'] !== 'string' || typeof p['idempotencyKey'] !== 'string' || !KEY_RE.test(p['idempotencyKey'])
      || p['auctionId'] !== `auc-${p['idempotencyKey']}` || p['artworkId'] !== v['artworkId']
      || op['kind'] !== 'publish' || op['idempotencyKey'] !== p['idempotencyKey'] || op['body'] !== p['body']
      || !['in_flight', 'pending_confirmation', 'rejected'].includes(op['state'] as string)) return false;
  try { return (JSON.parse(p['body']) as { auction?: { tenantId?: unknown } }).auction?.tenantId === v['tenantId']; }
  catch { return false; }
}

export function loadPublications(store: KeyValueStore): PendingPublication[] {
  try {
    const raw = store.getItem(KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter(valid).slice(0, 20) : [];
  } catch { return []; }
}

/** Upsert by idempotency key; confirmed operations are removed (the server is now the record). */
export function recordPublication(store: KeyValueStore, entry: PendingPublication): PendingPublication[] {
  const rest = loadPublications(store).filter(p => p.prepared.idempotencyKey !== entry.prepared.idempotencyKey);
  const next = entry.operation.state === 'confirmed' ? rest : [entry, ...rest];
  store.setItem(KEY, JSON.stringify(next));
  return next;
}

export function dropPublication(store: KeyValueStore, idempotencyKey: string): PendingPublication[] {
  const next = loadPublications(store).filter(p => p.prepared.idempotencyKey !== idempotencyKey);
  store.setItem(KEY, JSON.stringify(next));
  return next;
}

export function newIdempotencyKey(crypto: Pick<Crypto, 'getRandomValues'> = globalThis.crypto): string {
  const bytes = new Uint8Array(18);
  crypto.getRandomValues(bytes);
  return 'pub-' + Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
}
