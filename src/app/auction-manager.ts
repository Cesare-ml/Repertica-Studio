/**
 * Orchestration of publication and bid cancellation over an abstract transport.
 *
 * The contract is "artist-v1" (Repertica-auctions docs/ARTIST_API_V1.md); auction-http-transport.ts maps it over
 * the native port of auction-native.ts. This module never reports success from a transport answer alone; the
 * outcome is decided by re-reading the server state, and a failed re-read leaves the operation "pending
 * confirmation". Tests load it through tests/ts-resolve.mjs (extensionless imports, as the Angular build needs).
 */
import { parseAuction, parseBidPage, type AuctionBid, type AuctionSummary, type CancelResult, CANCEL_REASONS, type CancelRequest } from './auction-contract';
import { afterTransport, beginOperation, confirmCancellation, confirmPublication, retry, type Operation } from './auction-operations';
import type { PreparedPreview } from './auction-preview';

export interface AuctionTransport {
  /** Throws (or rejects) when the response is lost. */
  publish(body: string, idempotencyKey: string): Promise<{ status: number; error?: string | null }>;
  listAuctions(): Promise<unknown>;
  listBids(auctionId: string, cursor: string | null): Promise<unknown>;
  /** `body` is the contract answer already mapped by the transport (see auction-http-transport.ts). */
  cancelBid(auctionId: string, bidId: string, request: CancelRequest, idempotencyKey: string): Promise<{ status: number; body?: CancelResult }>;
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
  cancelBid(auctionId: string, bidId: string, request: CancelRequest, idempotencyKey: string,
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
        // The server lists bids in the order it accepted them; a stable sort keeps that order for equal times.
        return [...all.values()].sort((a, b) => a.placedAt.localeCompare(b.placedAt));
      }
      if (nextCursor === cursor) throw new Error('invalid_response');
      cursor = nextCursor;
    }
    throw new Error('too_many_pages');
  }

  /** Publication of an already reviewed request. Pass the same `prepared` again to retry. */
  async function publish(prepared: PreparedPreview, previous?: Operation): Promise<OperationOutcome> {
    requireOnline();
    // The site chooses the page address, so a publication is recognised by the auction id the request carries.
    const auctionId = (JSON.parse(prepared.body) as { auction: { id: string } }).auction.id;
    if (previous && (previous.body !== prepared.body || previous.idempotencyKey !== prepared.idempotencyKey)) {
      throw new Error('request_changed');
    }
    let op = previous ? retry(previous) : beginOperation('publish', auctionId, prepared.idempotencyKey, prepared.body);
    let status = 0;
    try {
      const answer = await transport.publish(prepared.body, prepared.idempotencyKey);
      status = answer.status;
      op = afterTransport(op, { kind: 'response', status, error: answer.error ?? null });
    } catch { op = afterTransport(op, { kind: 'lost' }); }
    if (op.state === 'pending_confirmation') {
      try { op = confirmPublication(op, await loadAuctions()); } catch { /* stays pending */ }
    }
    return { operation: op, needsSignIn: status === 401 };
  }

  /** Cancellation of a suspect bid; the history is reloaded to learn the official outcome. */
  async function cancelBid(auctionId: string, bidId: string, request: CancelRequest, idempotencyKey: string,
    previous?: Operation): Promise<{ outcome: OperationOutcome; bids: AuctionBid[] | null }> {
    requireOnline();
    // Gestionale Opere: one of five reasons, an optional private note of up to 500 characters, an optional block.
    const note = typeof request?.note === 'string' ? request.note.trim() : '';
    if (!request || !(CANCEL_REASONS as readonly string[]).includes(request.reason) || note.length > 500
      || (request.blockEmail !== undefined && typeof request.blockEmail !== 'boolean')) throw new Error('invalid_reason');
    const sent: CancelRequest = { reason: request.reason, ...(note ? { note } : {}), ...(request.blockEmail ? { blockEmail: true } : {}) };
    const body = JSON.stringify({ auctionId, bidId, ...sent });
    if (previous && previous.body !== body) throw new Error('request_changed');
    let op = previous ? retry(previous) : beginOperation('cancel_bid', bidId, idempotencyKey, body);
    let answer: CancelResult | null = null;
    let status = 0;
    try {
      const response = await transport.cancelBid(auctionId, bidId, sent, idempotencyKey);
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
