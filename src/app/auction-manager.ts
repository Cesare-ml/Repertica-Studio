/**
 * Orchestration of publication and bid cancellation over an abstract transport.
 *
 * No transport is implemented in the app yet (the online contract is not verified): this module is developed
 * and tested against a simulated one. It never reports success from a transport answer alone; the outcome is
 * decided by re-reading the server state, and a failed re-read leaves the operation "pending confirmation".
 *
 * NOTE: the `.ts` import suffixes let the Node test runner load this file directly. The Angular build does not
 * allow them (TS5096), so drop the suffixes (or inject the parsers) when a component first imports this module.
 */
import { parseAuction, parseBidPage, parseCancelResult, type AuctionBid, type AuctionSummary, type CancelResult } from './auction-contract.ts';
import { afterTransport, beginOperation, confirmCancellation, confirmPublication, retry, type Operation } from './auction-operations.ts';
import type { PreparedPreview } from './auction-preview.ts';

export interface AuctionTransport {
  /** Throws (or rejects) when the response is lost. */
  publish(body: string, idempotencyKey: string): Promise<{ status: number; auctionId?: string }>;
  listAuctions(): Promise<unknown>;
  listBids(auctionId: string, cursor: string | null): Promise<unknown>;
  cancelBid(auctionId: string, bidId: string, reason: string | null, idempotencyKey: string): Promise<{ status: number; body?: unknown }>;
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
  cancelBid(auctionId: string, bidId: string, reason: string | null, idempotencyKey: string,
    previous?: Operation): Promise<{ outcome: OperationOutcome; bids: AuctionBid[] | null }>;
}

export function createAuctionManager(transport: AuctionTransport, isOnline: () => boolean): AuctionManager {
  const requireOnline = (): void => { if (!isOnline()) throw new Error('offline'); };

  async function loadAuctions(): Promise<AuctionSummary[]> {
    requireOnline();
    const raw = await transport.listAuctions() as { items?: unknown };
    requireOnline();
    if (!raw || !Array.isArray(raw.items) || raw.items.length > 500) throw new Error('invalid_response');
    const items = raw.items.map(parseAuction);
    if (new Set(items.map(a => a.auctionId)).size !== items.length) throw new Error('invalid_response');
    return items;
  }

  /** The complete private history, read page by page; never cached beyond the caller's own lifetime. */
  async function loadBids(auctionId: string): Promise<AuctionBid[]> {
    requireOnline();
    const all = new Map<string, AuctionBid>();
    let cursor: string | null = null;
    const seenCursors = new Set<string>();
    for (let page = 0; page < MAX_PAGES; page++) {
      requireOnline();
      const { bids, nextCursor } = parseBidPage(await transport.listBids(auctionId, cursor));
      requireOnline();
      for (const bid of bids) {
        const prior = all.get(bid.bidId);
        // An unstable history must be read again explicitly, never silently
        // combine contradictory versions of a private bid.
        if (prior && JSON.stringify(prior) !== JSON.stringify(bid)) throw new Error('invalid_response');
        all.set(bid.bidId, bid);
      }
      if (nextCursor === null) {
        return [...all.values()].sort((a, b) => a.placedAt.localeCompare(b.placedAt) || a.bidId.localeCompare(b.bidId));
      }
      if (seenCursors.has(nextCursor)) throw new Error('invalid_response');
      seenCursors.add(nextCursor);
      cursor = nextCursor;
    }
    throw new Error('too_many_pages');
  }

  /** Publication of an already reviewed request. Pass the same `prepared` again to retry. */
  async function publish(prepared: PreparedPreview, previous?: Operation): Promise<OperationOutcome> {
    requireOnline();
    const slug = (JSON.parse(prepared.body) as { slug: string }).slug;
    if (previous && (previous.kind !== 'publish' || previous.target !== slug || previous.body !== prepared.body || previous.idempotencyKey !== prepared.idempotencyKey)) {
      throw new Error('request_changed');
    }
    let op = previous ? retry(previous) : beginOperation('publish', slug, prepared.idempotencyKey, prepared.body);
    let status = 0;
    let receiptId: string | null = null;
    try {
      const response = await transport.publish(op.body, op.idempotencyKey);
      status = response.status;
      op = afterTransport(op, { kind: 'response', status });
      // This is a transport-port receipt, not a new HTTP response contract.
      // A lost or malformed receipt cannot be attributed by slug alone.
      if ((status === 200 || status === 201) && typeof response.auctionId === 'string'
          && /^[A-Za-z0-9_-]{3,96}$/.test(response.auctionId)) receiptId = response.auctionId;
    } catch { op = afterTransport(op, { kind: 'lost' }); }
    if (op.state === 'pending_confirmation' && receiptId !== null) {
      try { op = confirmPublication(op, await loadAuctions(), receiptId); } catch { /* stays pending */ }
    }
    return { operation: op, needsSignIn: status === 401 };
  }

  /** Cancellation of a suspect bid; the history is reloaded to learn the official outcome. */
  async function cancelBid(auctionId: string, bidId: string, reason: string | null, idempotencyKey: string,
    previous?: Operation): Promise<{ outcome: OperationOutcome; bids: AuctionBid[] | null }> {
    requireOnline();
    if (reason !== null && (reason.trim().length === 0 || reason.length > 500)) throw new Error('invalid_reason');
    const body = JSON.stringify({ auctionId, bidId, reason });
    if (previous && (previous.kind !== 'cancel_bid' || previous.target !== bidId || previous.body !== body || previous.idempotencyKey !== idempotencyKey)) throw new Error('request_changed');
    let op = previous ? retry(previous) : beginOperation('cancel_bid', bidId, idempotencyKey, body);
    let answer: CancelResult | null = null;
    let status = 0;
    try {
      const response = await transport.cancelBid(auctionId, bidId, reason, op.idempotencyKey);
      status = response.status;
      op = afterTransport(op, { kind: 'response', status });
      if (response.body !== undefined) { try { answer = parseCancelResult(response.body); } catch { answer = null; } }
    } catch { op = afterTransport(op, { kind: 'lost' }); }
    let bids: AuctionBid[] | null = null;
    if (op.state !== 'rejected') {
      try { bids = await loadBids(auctionId); } catch { bids = null; }
    }
    if (bids !== null && op.state !== 'rejected') op = confirmCancellation(op, bids, answer);
    return { outcome: { operation: op, needsSignIn: status === 401 }, bids };
  }

  return { loadAuctions, loadBids, publish, cancelBid };
}
