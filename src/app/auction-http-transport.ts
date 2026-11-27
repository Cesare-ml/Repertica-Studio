/**
 * AuctionTransport for the Auctions contract "artist-v1" over an injected HTTP port.
 *
 * The port is the only thing that touches the network. In the app it is the native command auction_api_request
 * (auction-native.ts -> src-tauri/src/auction_online.rs), so that the Repertica access token never enters the
 * webview and the webview CSP (connect-src 'self') stays closed. The port throws or rejects when the response is
 * lost: the manager then treats the operation as uncertain.
 */
import { cancelOutcomeFromError, parseCancelResult, type CancelResult } from './auction-contract';
import type { AuctionTransport } from './auction-manager';

export interface HttpRequest {
  readonly method: 'GET' | 'POST';
  /** Path and query on the fixed Auctions origin; never a full URL chosen by the webview. */
  readonly path: string;
  readonly headers: Readonly<Record<string, string>>;
  readonly body?: string;
}
export interface HttpResponse { readonly status: number; readonly body: unknown }
export type HttpPort = (request: HttpRequest) => Promise<HttpResponse>;

const ID = /^[A-Za-z0-9_-]{1,100}$/;
const KEY = /^[A-Za-z0-9_-]{16,96}$/;
const MAX_AUCTION_PAGES = 20;

class TransportError extends Error {
  readonly status: number;
  constructor(code: string, status: number) { super(code); this.status = status; }
}

function ensureId(value: string, code: string): string {
  if (!ID.test(value)) throw new Error(code);
  return value;
}

/** Statuses that are an answer from the server (the transport did not lose it). */
function answered(response: HttpResponse): HttpResponse {
  if (!Number.isInteger(response.status) || response.status < 100 || response.status > 599) throw new Error('invalid_response');
  return response;
}

function requireOk(response: HttpResponse): unknown {
  if (response.status === 200) return response.body;
  const code = response.body !== null && typeof response.body === 'object' && !Array.isArray(response.body)
    ? String((response.body as Record<string, unknown>)['error'] ?? `http_${response.status}`) : `http_${response.status}`;
  throw new TransportError(code, response.status);
}

/** The server's error code of a refusal, when it gave a well-formed one. */
function errorCode(response: HttpResponse): string | null {
  if (response.status < 400 || response.body === null || typeof response.body !== 'object' || Array.isArray(response.body)) return null;
  const code = (response.body as Record<string, unknown>)['error'];
  return typeof code === 'string' && /^[a-z][a-z0-9_]{2,63}$/.test(code) ? code : null;
}

export function createArtistV1Transport(port: HttpPort, tenantId: () => string | null): AuctionTransport {
  const json = { 'Content-Type': 'application/json', Accept: 'application/json' };
  const get = async (path: string): Promise<unknown> =>
    requireOk(answered(await port({ method: 'GET', path, headers: { Accept: 'application/json' } })));

  return {
    async publish(body, idempotencyKey) {
      if (!KEY.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
      const response = answered(await port({ method: 'POST', path: '/api/artist/publications',
        headers: { ...json, 'Idempotency-Key': idempotencyKey }, body }));
      // A refusal carries the server's code (for example public_details_exist), shown to the artist.
      const code = response.status >= 400 && response.body !== null && typeof response.body === 'object' && !Array.isArray(response.body)
        ? (response.body as Record<string, unknown>)['error'] : null;
      return { status: response.status, error: typeof code === 'string' && /^[a-z][a-z0-9_]{2,63}$/.test(code) ? code : null };
    },

    /** All pages of the selected profile's auctions, as the manager expects: { items }. */
    async listAuctions() {
      const tenant = ensureId(tenantId() ?? '', 'tenant_not_selected');
      const items: unknown[] = [];
      for (let page = 1; page <= MAX_AUCTION_PAGES; page++) {
        const data = await get(`/api/artist/auctions?tenantId=${encodeURIComponent(tenant)}&page=${page}`) as
          { items?: unknown; hasMore?: unknown };
        if (!data || !Array.isArray(data.items) || typeof data.hasMore !== 'boolean') throw new Error('invalid_response');
        items.push(...data.items);
        if (!data.hasMore) return { items };
      }
      throw new Error('too_many_pages');
    },

    async listBids(auctionId, cursor) {
      ensureId(auctionId, 'invalid_auction');
      if (cursor !== null && !/^\d{1,16}$/.test(cursor)) throw new Error('invalid_response');
      return get(`/api/artist/auctions/${auctionId}/bids` + (cursor === null ? '' : `?cursor=${cursor}`));
    },

    async getParticipants(auctionId) {
      ensureId(auctionId, 'invalid_auction');
      return get(`/api/artist/auctions/${auctionId}/participants`);
    },

    async getTerms(auctionId) {
      ensureId(auctionId, 'invalid_auction');
      return get(`/api/artist/auctions/${auctionId}/terms`);
    },

    async getEmailPolicy() {
      const tenant = ensureId(tenantId() ?? '', 'tenant_not_selected');
      return get(`/api/artist/email-policy?tenantId=${encodeURIComponent(tenant)}`);
    },

    async blockEmail(email, reason, idempotencyKey) {
      const tenant = ensureId(tenantId() ?? '', 'tenant_not_selected');
      if (!KEY.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
      const response = answered(await port({ method: 'POST', path: '/api/artist/email-policy/blocks',
        headers: { ...json, 'Idempotency-Key': idempotencyKey }, body: JSON.stringify({ tenantId: tenant, email, ...(reason ? { reason } : {}) }) }));
      return { status: response.status, error: errorCode(response) };
    },

    async unblockEmail(blockId, idempotencyKey) {
      const tenant = ensureId(tenantId() ?? '', 'tenant_not_selected');
      if (!KEY.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
      if (!/^[a-f0-9]{24}$/.test(blockId)) throw new Error('invalid_response');
      const response = answered(await port({ method: 'POST', path: `/api/artist/email-policy/blocks/${blockId}/removal`,
        headers: { ...json, 'Idempotency-Key': idempotencyKey }, body: JSON.stringify({ tenantId: tenant }) }));
      return { status: response.status, error: errorCode(response) };
    },

    async cancelBid(auctionId, bidId, request, idempotencyKey) {
      ensureId(auctionId, 'invalid_auction');
      ensureId(bidId, 'invalid_bid');
      if (!KEY.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
      if (!request || typeof request.reason !== 'string') throw new Error('invalid_reason');
      const body = { reason: request.reason, ...(request.note ? { note: request.note } : {}), ...(request.blockEmail ? { blockEmail: true } : {}) };
      const response = answered(await port({ method: 'POST', path: `/api/artist/auctions/${auctionId}/bids/${bidId}/cancellation`,
        headers: { ...json, 'Idempotency-Key': idempotencyKey }, body: JSON.stringify(body) }));
      if (response.status === 200 || response.status === 201) {
        let result: CancelResult | undefined;
        try { result = parseCancelResult(response.body); } catch { result = undefined; }
        return { status: response.status, body: result };
      }
      return { status: response.status, body: cancelOutcomeFromError(bidId, response.status, response.body) ?? undefined };
    },
  };
}

export function isSignInRequired(error: unknown): boolean {
  return error instanceof TransportError && error.status === 401;
}
