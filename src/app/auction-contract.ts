/**
 * Client-side reading of the Auctions server responses (contract "draft-0": NOT verified, NOT connected).
 *
 * The server is the authority on bids, deadlines, cancellations and results. These parsers only check that
 * what arrives has the expected shape (explicit allowlist, strict UTC times, integer cents) so the interface
 * never shows invented or malformed values. Nothing here computes an official result.
 */
export const AUCTION_CONTRACT_VERSION = 'draft-0';
/** Flips only when the online project publishes and verifies a production contract. */
export const AUCTION_API_CONNECTED = false;

export type AuctionState = 'draft' | 'preview' | 'scheduled' | 'open' | 'ended' | 'awarded'
  | 'payment_pending' | 'paid' | 'fulfilled' | 'cancelled' | 'disputed' | 'unknown';
const STATES: readonly string[] = ['draft', 'preview', 'scheduled', 'open', 'ended', 'awarded',
  'payment_pending', 'paid', 'fulfilled', 'cancelled', 'disputed'];

export interface AuctionSummary {
  readonly auctionId: string;
  readonly slug: string;
  readonly title: string;
  readonly state: AuctionState;
  readonly startsAt: string;
  readonly endsAt: string;
  readonly startingAmount: number;
  readonly minimumIncrementAmount: number;
  /** Server value, never recomputed locally. */
  readonly currentAmount: number | null;
  readonly leadingBidId: string | null;
  readonly winnerBidId: string | null;
}

export interface AuctionBid {
  readonly bidId: string;
  readonly amount: number;
  readonly placedAt: string;
  /** Private: only for the authorised artist, never logged or shown publicly. */
  readonly bidderEmail: string;
  readonly state: 'valid' | 'cancelled';
  readonly cancelledAt: string | null;
  readonly cancelReason: string | null;
}

export type CancelOutcome = 'cancelled' | 'already_cancelled' | 'not_allowed' | 'contested';
export interface CancelResult { readonly bidId: string; readonly outcome: CancelOutcome; readonly reason: string | null }

const fail = (): never => { throw new Error('invalid_response'); };
const record = (value: unknown): Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : fail();
const id = (value: unknown): string =>
  typeof value === 'string' && /^[A-Za-z0-9_-]{3,96}$/.test(value) ? value : fail();
const text = (value: unknown, max: number, min = 1): string =>
  typeof value === 'string' && value.length >= min && value.length <= max ? value : fail();
const cents = (value: unknown): number =>
  typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 && value <= 1_000_000_000_000 ? value : fail();

export function utcInstant(value: unknown): string {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)) return fail();
  const time = Date.parse(value);
  return Number.isFinite(time) && new Date(time).toISOString() === value ? value : fail();
}

const optional = <T>(value: unknown, read: (v: unknown) => T): T | null =>
  value === undefined || value === null ? null : read(value);

export function parseAuction(input: unknown): AuctionSummary {
  const o = record(input);
  const state = typeof o['state'] === 'string' && STATES.includes(o['state']) ? o['state'] as AuctionState : 'unknown';
  const startsAt = utcInstant(o['startsAt']), endsAt = utcInstant(o['endsAt']);
  if (Date.parse(endsAt) <= Date.parse(startsAt)) fail();
  if (o['currency'] !== undefined && o['currency'] !== 'EUR') fail();
  return Object.freeze({
    auctionId: id(o['auctionId']), slug: text(o['slug'], 60), title: text(o['title'], 160), state,
    startsAt, endsAt, startingAmount: cents(o['startingAmount']),
    minimumIncrementAmount: cents(o['minimumIncrementAmount']),
    currentAmount: optional(o['currentAmount'], cents),
    leadingBidId: optional(o['leadingBidId'], id),
    winnerBidId: optional(o['winnerBidId'], id),
  });
}

export function parseBid(input: unknown): AuctionBid {
  const o = record(input);
  const state = o['state'];
  if (state !== 'valid' && state !== 'cancelled') fail();
  const email = text(o['bidderEmail'], 254, 3);
  if (!/^[^\s@]+@[^\s@]+$/.test(email)) fail();
  const cancelledAt = optional(o['cancelledAt'], utcInstant);
  if ((state === 'cancelled') !== (cancelledAt !== null)) fail();
  return Object.freeze({
    bidId: id(o['bidId']), amount: cents(o['amount']), placedAt: utcInstant(o['placedAt']), bidderEmail: email,
    state: state as 'valid' | 'cancelled', cancelledAt,
    cancelReason: optional(o['cancelReason'], v => text(v, 500)),
  });
}

/** One page of the private bid history; duplicates (replayed pages) collapse, order is by server time. */
export function parseBidPage(input: unknown): { bids: AuctionBid[]; nextCursor: string | null } {
  const o = record(input);
  if (!Array.isArray(o['items']) || o['items'].length > 500) fail();
  const seen = new Map<string, AuctionBid>();
  for (const item of o['items'] as unknown[]) {
    const bid = parseBid(item), prior = seen.get(bid.bidId);
    if (prior && JSON.stringify(prior) !== JSON.stringify(bid)) fail();
    seen.set(bid.bidId, bid);
  }
  const bids = [...seen.values()].sort((a, b) => a.placedAt.localeCompare(b.placedAt) || a.bidId.localeCompare(b.bidId));
  return { bids, nextCursor: optional(o['nextCursor'], v => text(v, 512)) };
}

export function parseCancelResult(input: unknown): CancelResult {
  const o = record(input);
  const outcome = o['outcome'];
  if (outcome !== 'cancelled' && outcome !== 'already_cancelled' && outcome !== 'not_allowed' && outcome !== 'contested') fail();
  return Object.freeze({ bidId: id(o['bidId']), outcome: outcome as CancelOutcome,
    reason: optional(o['reason'], v => text(v, 500)) });
}

/** The server's leading bid versus the history it returned. A mismatch is reported, never "fixed" locally. */
export function leadingConsistency(auction: AuctionSummary, bids: readonly AuctionBid[]): 'consistent' | 'mismatch' | 'unknown' {
  if (auction.leadingBidId === null) return bids.some(b => b.state === 'valid') ? 'mismatch' : 'consistent';
  const lead = bids.find(b => b.bidId === auction.leadingBidId);
  if (!lead) return 'unknown';
  return lead.state === 'valid' && (auction.currentAmount === null || auction.currentAmount === lead.amount)
    ? 'consistent' : 'mismatch';
}

/** Awarding is not a payment: the local archive never records a sale from an auction result by itself. */
export function resultNeedsExplicitSale(state: AuctionState): boolean {
  return state === 'awarded' || state === 'payment_pending' || state === 'paid' || state === 'fulfilled';
}

export function maskEmail(email: string): string {
  const at = email.indexOf('@');
  return at < 1 ? '***' : `${email[0]}***${email.slice(at)}`;
}
