/**
 * Orchestration of publication and bid cancellation over an abstract transport.
 *
 * The contract is "artist-v1" (Repertica-auctions docs/ARTIST_API_V1.md) and auction-http-transport.ts maps it
 * over an injected HTTP port; the app does not have that native port nor a Repertica sign-in yet, so nothing is
 * connected. This module never reports success from a transport answer alone; the outcome is decided by
 * re-reading the server state, and a failed re-read leaves the operation "pending confirmation".
 *
 * NOTE: the `.ts` import suffixes let the Node test runner load this file directly. The Angular build does not
 * allow them (TS5096), so drop the suffixes (or inject the parsers) when a component first imports this module.
 */
import { parseAuction, parseBidPage, type AuctionBid, type AuctionSummary, type CancelResult } from './auction-contract.ts';
import { afterTransport, beginOperation, confirmCancellation, confirmPublication, retry, type Operation } from './auction-operations.ts';
import type { PreparedPreview } from './auction-preview.ts';

export interface AuctionTransport {
  /** Throws (or rejects) when the response is lost. */
  publish(body: string, idempotencyKey: string): Promise<{ status: number }>;
  listAuctions(): Promise<unknown>;
  listBids(auctionId: string, cursor: string | null): Promise<unknown>;
  /** `body` is the contract answer already mapped by the transport (see auction-http-transport.ts). */
  cancelBid(auctionId: string, bidId: string, reason: string, idempotencyKey: string): Promise<{ status: number; body?: CancelResult }>;
}

const CANCEL_OUTCOMES: readonly string[] = ['cancelled', 'already_cancelled', 'not_allowed', 'contested'];
function cancelAnswer(value: unknown, bidId: string): CancelResult | null {
  if (!value || typeof value !== 'object') return null;
  const v = value as Partial<CancelResult>;
  return v.bidId === bidId && typeof v.outcome === 'string' && CANCEL_OUTCOMES.includes(v.outcome) ? v as CancelResult : null;
}

export interface OperationOutcome {
  readonly operation: Operation;
  /** The request was refused because the session is missing or expired: sign in, then retry with the same key. */
  readonly needsSignIn: boolean;
}

const MAX_PAGES = 40;

export interface AuctionManager {
  loadAuctions(): Promise<AuctionSummary[]>;
  loadBids(auctionId: string): Promise<AuctionBid[]>;
  publish(prepared: PreparedPreview, previous?: Operation): Promise<OperationOutcome>;
  cancelBid(auctionId: string, bidId: string, reason: string, idempotencyKey: string,
    previous?: Operation): Promise<{ outcome: OperationOutcome; bids: AuctionBid[] | null }>;
}

export function createAuctionManager(transport: AuctionTransport, isOnline: () => boolean): AuctionManager {
  const requireOnline = (): void => { if (!isOnline()) throw new Error('offline'); };

  async function loadAuctions(): Promise<AuctionSummary[]> {
    requireOnline();
    const raw = await transport.listAuctions() as { items?: unknown };
    if (!raw || !Array.isArray(raw.items)) throw new Error('invalid_response');
    return raw.items.map(parseAuction);
  }

  /** The complete private history, read page by page; never cached beyond the caller's own lifetime. */
  async function loadBids(auctionId: string): Promise<AuctionBid[]> {
    requireOnline();
    const all = new Map<string, AuctionBid>();
    let cursor: string | null = null;
    for (let page = 0; page < MAX_PAGES; page++) {
      const { bids, nextCursor } = parseBidPage(await transport.listBids(auctionId, cursor));
      for (const bid of bids) all.set(bid.bidId, bid);
      if (nextCursor === null) {
        return [...all.values()].sort((a, b) => a.placedAt.localeCompare(b.placedAt) || a.bidId.localeCompare(b.bidId));
      }
      if (nextCursor === cursor) throw new Error('invalid_response');
      cursor = nextCursor;
    }
    throw new Error('too_many_pages');
  }

  /** Publication of an already reviewed request. Pass the same `prepared` again to retry. */
  async function publish(prepared: PreparedPreview, previous?: Operation): Promise<OperationOutcome> {
    requireOnline();
    const slug = (JSON.parse(prepared.body) as { details: { slug: string } }).details.slug;
    if (previous && (previous.body !== prepared.body || previous.idempotencyKey !== prepared.idempotencyKey)) {
      throw new Error('request_changed');
    }
    let op = previous ? retry(previous) : beginOperation('publish', slug, prepared.idempotencyKey, prepared.body);
    let status = 0;
    try {
      status = (await transport.publish(prepared.body, prepared.idempotencyKey)).status;
      op = afterTransport(op, { kind: 'response', status });
    } catch { op = afterTransport(op, { kind: 'lost' }); }
    if (op.state === 'pending_confirmation') {
      try { op = confirmPublication(op, await loadAuctions()); } catch { /* stays pending */ }
    }
    return { operation: op, needsSignIn: status === 401 };
  }

  /** Cancellation of a suspect bid; the history is reloaded to learn the official outcome. */
  async function cancelBid(auctionId: string, bidId: string, reason: string, idempotencyKey: string,
    previous?: Operation): Promise<{ outcome: OperationOutcome; bids: AuctionBid[] | null }> {
    requireOnline();
    // The server requires a private reason of 3 to 500 characters (artist-v1).
    const trimmed = typeof reason === 'string' ? reason.trim() : '';
    if (trimmed.length < 3 || trimmed.length > 500) throw new Error('invalid_reason');
    const body = JSON.stringify({ auctionId, bidId, reason: trimmed });
    if (previous && previous.body !== body) throw new Error('request_changed');
    let op = previous ? retry(previous) : beginOperation('cancel_bid', bidId, idempotencyKey, body);
    let answer: CancelResult | null = null;
    let status = 0;
    try {
      const response = await transport.cancelBid(auctionId, bidId, trimmed, idempotencyKey);
      status = response.status;
      op = afterTransport(op, { kind: 'response', status });
      answer = cancelAnswer(response.body, bidId);
      // Already cancelled (e.g. by an earlier attempt whose answer was lost): the server history decides.
      if (answer?.outcome === 'already_cancelled') op = Object.freeze({ ...op, state: 'pending_confirmation' as const, reason: null });
    } catch { op = afterTransport(op, { kind: 'lost' }); }
    let bids: AuctionBid[] | null = null;
    try { bids = await loadBids(auctionId); } catch { bids = null; }
    if (bids !== null && op.state !== 'rejected') op = confirmCancellation(op, bids, answer);
    else if (op.state === 'rejected' && answer && answer.reason) op = Object.freeze({ ...op, reason: answer.reason });
    return { outcome: { operation: op, needsSignIn: status === 401 }, bids };
  }

  return { loadAuctions, loadBids, publish, cancelBid };
}
