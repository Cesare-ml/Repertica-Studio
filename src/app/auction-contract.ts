/**
 * Client-side reading of the Auctions server responses, contract "artist-v1"
 * (Repertica-auctions docs/ARTIST_API_V1.md; fixtures in tests/fixtures/auctions-artist-v1 were captured from the
 * real server running with synthetic data).
 *
 * The server is the authority on bids, deadlines, cancellations and results. These parsers only check that
 * what arrives has the expected shape (explicit allowlist, strict UTC times, integer cents) so the interface
 * never shows invented or malformed values. Nothing here computes an official result.
 */
export const AUCTION_CONTRACT_VERSION = 'artist-v1';
/**
 * Production switch, mirrored by ONLINE_ENABLED in src-tauri/src/auction_online.rs (the source of truth): the
 * interface reads the real availability from the native side. Off until Auctions and the Repertica sign-in are live.
 */
export const AUCTION_API_CONNECTED = false;

export type AuctionState = 'upcoming' | 'live' | 'ended' | 'awarded' | 'not_awarded' | 'cancelled' | 'unknown';
const STATES: readonly string[] = ['upcoming', 'live', 'ended', 'awarded', 'not_awarded', 'cancelled'];

export interface AuctionSummary {
  readonly auctionId: string;
  readonly tenantId: string;
  readonly slug: string;
  readonly title: string;
  readonly state: AuctionState;
  readonly startsAt: string;
  /** Current close, including anti-sniping extensions already granted by the server. */
  readonly endsAt: string;
  /** Latest possible close if every extension is used. */
  readonly maxEndsAt: string;
  readonly startingAmount: number;
  readonly minimumIncrementAmount: number;
  readonly reserveAmount: number | null;
  readonly reserveMet: boolean;
  /** Server values, never recomputed locally. */
  readonly currentAmount: number | null;
  readonly leadingBidId: string | null;
  readonly minimumNextBidAmount: number;
  readonly bidCount: number;
  readonly cancelledBidCount: number;
  readonly winnerBidId: string | null;
  readonly winningAmount: number | null;
  readonly studioArtworkId: number | null;
  readonly version: number;
}

export interface AuctionBid {
  readonly bidId: string;
  readonly amount: number;
  readonly placedAt: string;
  /** Stable per auction (#1, #2…), never an identity. */
  readonly bidderLabel: string;
  /** Private: only for the authorised artist, never logged, cached on disk or shown publicly. Null when the
   * server cannot decrypt the contact. */
  readonly bidderEmail: string | null;
  readonly state: 'valid' | 'cancelled';
  readonly isLeading: boolean;
  readonly cancelledAt: string | null;
  readonly cancelReason: string | null;
}

export type CancelOutcome = 'cancelled' | 'already_cancelled' | 'not_allowed' | 'contested';
export interface CancelResult { readonly bidId: string; readonly outcome: CancelOutcome; readonly reason: string | null }

const fail = (): never => { throw new Error('invalid_response'); };
const record = (value: unknown): Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : fail();
const id = (value: unknown): string =>
  typeof value === 'string' && /^[A-Za-z0-9_-]{1,100}$/.test(value) ? value : fail();
const text = (value: unknown, max: number, min = 1): string =>
  typeof value === 'string' && value.length >= min && value.length <= max ? value : fail();
const cents = (value: unknown): number =>
  typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 && value <= 1_000_000_000_000 ? value : fail();
const count = (value: unknown): number =>
  typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 ? value : fail();
const bool = (value: unknown): boolean => typeof value === 'boolean' ? value : fail();

export function utcInstant(value: unknown): string {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)) return fail();
  const time = Date.parse(value);
  return Number.isFinite(time) && new Date(time).toISOString() === value ? value : fail();
}

/** null only when the field is explicitly null; a missing field is a malformed answer. */
const nullable = <T>(value: unknown, read: (v: unknown) => T): T | null => value === null ? null : read(value);

export function parseAuction(input: unknown): AuctionSummary {
  const o = record(input);
  const state = typeof o['state'] === 'string' && STATES.includes(o['state']) ? o['state'] as AuctionState : 'unknown';
  const startsAt = utcInstant(o['startsAt']), endsAt = utcInstant(o['endsAt']), maxEndsAt = utcInstant(o['maxEndsAt']);
  if (Date.parse(endsAt) <= Date.parse(startsAt) || Date.parse(maxEndsAt) < Date.parse(endsAt)) fail();
  if (o['currency'] !== 'EUR') fail();
  // An award is never a payment: the contract states it explicitly.
  if (o['paymentRecorded'] !== false) fail();
  const studioArtworkId = nullable(o['studioArtworkId'], v =>
    typeof v === 'number' && Number.isSafeInteger(v) && v > 0 ? v : fail());
  return Object.freeze({
    auctionId: id(o['auctionId']), tenantId: id(o['tenantId']), slug: text(o['slug'], 100), title: text(o['title'], 160),
    state, startsAt, endsAt, maxEndsAt,
    startingAmount: cents(o['startingAmount']), minimumIncrementAmount: cents(o['minimumIncrementAmount']),
    reserveAmount: nullable(o['reserveAmount'], cents), reserveMet: bool(o['reserveMet']),
    currentAmount: nullable(o['currentAmount'], cents), leadingBidId: nullable(o['leadingBidId'], id),
    minimumNextBidAmount: cents(o['minimumNextBidAmount']),
    bidCount: count(o['bidCount']), cancelledBidCount: count(o['cancelledBidCount']),
    winnerBidId: nullable(o['winnerBidId'], id), winningAmount: nullable(o['winningAmount'], cents),
    studioArtworkId, version: count(o['version']),
  });
}

export function parseBid(input: unknown): AuctionBid {
  const o = record(input);
  const state = o['state'];
  if (state !== 'valid' && state !== 'cancelled') fail();
  const bidderEmail = nullable(o['bidderEmail'], v => {
    const email = text(v, 254, 3);
    return /^[^\s@]+@[^\s@]+$/.test(email) ? email : fail();
  });
  const cancelledAt = nullable(o['cancelledAt'], utcInstant);
  if ((state === 'cancelled') !== (cancelledAt !== null)) fail();
  const label = text(o['bidderLabel'], 8);
  if (!/^#\d{1,6}$/.test(label)) fail();
  return Object.freeze({
    bidId: id(o['bidId']), amount: cents(o['amount']), placedAt: utcInstant(o['placedAt']), bidderLabel: label, bidderEmail,
    state: state as 'valid' | 'cancelled', isLeading: bool(o['isLeading']), cancelledAt,
    cancelReason: nullable(o['cancelReason'], v => text(v, 500)),
  });
}

/** One page of the private bid history; duplicates (replayed pages) collapse, order is by server time and, for equal
 * times, the order in which the server accepted the bids (a stable sort keeps the page order). */
export function parseBidPage(input: unknown): { bids: AuctionBid[]; nextCursor: string | null } {
  const o = record(input);
  if (!Array.isArray(o['items']) || o['items'].length > 500) fail();
  const seen = new Map<string, AuctionBid>();
  for (const item of o['items'] as unknown[]) { const bid = parseBid(item); seen.set(bid.bidId, bid); }
  const bids = [...seen.values()].sort((a, b) => a.placedAt.localeCompare(b.placedAt));
  return { bids, nextCursor: nullable(o['nextCursor'], v => text(v, 512)) };
}

/** Successful cancellation answer (201 first time, 200 idempotent replay). */
export function parseCancelResult(input: unknown): CancelResult {
  const o = record(input);
  if (o['outcome'] !== 'cancelled') fail();
  utcInstant(o['cancelledAt']);
  return Object.freeze({ bidId: id(o['bidId']), outcome: 'cancelled' as const, reason: null });
}

/** Refusals of the contract mapped to what the interface explains; anything else is not a cancel outcome. */
export function cancelOutcomeFromError(bidId: string, status: number, body: unknown): CancelResult | null {
  const code = body !== null && typeof body === 'object' && !Array.isArray(body)
    ? (body as Record<string, unknown>)['error'] : undefined;
  if (status === 409 && code === 'bid_already_cancelled') return Object.freeze({ bidId, outcome: 'already_cancelled' as const, reason: null });
  if (status === 409 && code === 'auction_closed') return Object.freeze({ bidId, outcome: 'not_allowed' as const, reason: 'auction_closed' });
  if (status === 403) return Object.freeze({ bidId, outcome: 'not_allowed' as const, reason: 'tenant_access_denied' });
  return null;
}

export interface ArtistTenant { readonly id: string; readonly name: string; readonly role: string; readonly canPublish: boolean }

/** GET /api/artist/tenants: the artist profiles the signed-in account manages. */
export function parseTenants(input: unknown): ArtistTenant[] {
  const o = record(input);
  if (!Array.isArray(o['tenants']) || o['tenants'].length > 100) fail();
  return (o['tenants'] as unknown[]).map(item => {
    const t = record(item);
    const role = t['role'];
    if (role !== 'owner' && role !== 'editor' && role !== 'viewer') fail();
    const tenantId = id(t['id']);
    return Object.freeze({ id: tenantId, name: t['name'] === null ? tenantId : text(t['name'], 160), role: role as string,
      canPublish: bool(t['canPublish']) });
  });
}

/** The server's leading bid versus the history it returned. A mismatch is reported, never "fixed" locally. */
export function leadingConsistency(auction: AuctionSummary, bids: readonly AuctionBid[]): 'consistent' | 'mismatch' | 'unknown' {
  if (auction.leadingBidId === null) return bids.some(b => b.state === 'valid') ? 'mismatch' : 'consistent';
  const lead = bids.find(b => b.bidId === auction.leadingBidId);
  if (!lead) return 'unknown';
  return lead.state === 'valid' && lead.isLeading && (auction.currentAmount === null || auction.currentAmount === lead.amount)
    ? 'consistent' : 'mismatch';
}

/** Awarding is not a payment: the local archive never records a sale from an auction result by itself. */
export function resultNeedsExplicitSale(state: AuctionState): boolean {
  return state === 'awarded';
}

export function maskEmail(email: string): string {
  const at = email.indexOf('@');
  return at < 1 ? '***' : `${email[0]}***${email.slice(at)}`;
}
