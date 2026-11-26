/** Offline-only boundary for one explicitly selected artwork. No transport or credentials. */
export interface AuctionDraft {
  readonly artworkId: string;
  title: string;
  description: string;
  slug: string;
  startsAt: string;
  endsAt: string;
  startingAmount: number | null;
  minimumIncrementAmount: number | null;
  /** Optional reserve price in cents (Gestionale Opere). */
  reserveAmount?: number | null;
  /** Winner's payment terms (Gestionale Opere): shipping within Italy in cents and bank transfer details. */
  italyShippingCost?: number | null;
  bankBeneficiary?: string;
  bankIban?: string;
  bankBicSwift?: string;
  bankNameAndAddress?: string;
}

export interface PreparedPreview {
  readonly artworkId: string;
  /** Server auction id, derived from the idempotency key so that every retry names the same auction. */
  readonly auctionId: string;
  readonly idempotencyKey: string;
  /** Exact body of POST /api/artist/publications (contract artist-v1). */
  readonly body: string;
}

// Private archive description, location, images, prices and buyer information are not defaults.
export function newAuctionDraft(artwork: { id: string; title: string; status: string }): AuctionDraft {
  if (!artwork.id || artwork.status !== 'available') throw new Error('artwork_not_available');
  return { artworkId: artwork.id, title: artwork.title, description: '', slug: '',
    startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: null, reserveAmount: null,
    italyShippingCost: null, bankBeneficiary: '', bankIban: '', bankBicSwift: '', bankNameAndAddress: '' };
}

function utc(value: string): number {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)) {
    throw new Error('invalid_schedule');
  }
  const time = Date.parse(value);
  if (!Number.isFinite(time) || new Date(time).toISOString() !== value) throw new Error('invalid_schedule');
  return time;
}

function cents(value: number | null | undefined): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 1 || value > 1_000_000_000) {
    throw new Error('invalid_amount');
  }
  return value;
}

/** IBAN in electronic format with a valid ISO 13616 check (mod 97), as the server checks it. */
export function normalizeIban(value: string | undefined): string | null {
  if (typeof value !== 'string') return null;
  const iban = value.replace(/\s+/g, '').toUpperCase();
  if (!/^[A-Z]{2}\d{2}[A-Z0-9]{11,30}$/.test(iban)) return null;
  const digits = (iban.slice(4) + iban.slice(0, 4)).replace(/[A-Z]/g, c => String(c.charCodeAt(0) - 55));
  let rest = 0;
  for (const digit of digits) rest = (rest * 10 + Number(digit)) % 97;
  return rest === 1 ? iban : null;
}

const plain = (value: string | undefined, min: number, max: number): string => {
  const text = typeof value === 'string' ? value.trim() : '';
  if (text.length < min || text.length > max || /[\u0000-\u001f\u007f]/.test(text)) throw new Error('invalid_terms');
  return text;
};

/** The payment terms sent to the winner, validated exactly as the server does. */
export function paymentTerms(draft: AuctionDraft) {
  const shipping = draft.italyShippingCost;
  if (typeof shipping !== 'number' || !Number.isSafeInteger(shipping) || shipping < 0 || shipping > 100_000_000) {
    throw new Error('invalid_shipping');
  }
  const iban = normalizeIban(draft.bankIban);
  if (!iban) throw new Error('invalid_iban');
  const bic = (draft.bankBicSwift ?? '').replace(/\s+/g, '').toUpperCase();
  if (!/^[A-Z]{4}[A-Z]{2}[A-Z0-9]{2}(?:[A-Z0-9]{3})?$/.test(bic)) throw new Error('invalid_bic');
  return { italyShippingCostAmount: shipping, bankTransfer: { beneficiary: plain(draft.bankBeneficiary, 2, 140), iban,
    bicSwift: bic, bankNameAndAddress: plain(draft.bankNameAndAddress, 3, 300) } };
}

/** Call only after the user reviews the public fields and chooses a tenant.
 * Persist the returned body + key before attempting a request; retries must reuse both.
 * A changed payload is a new attempt, not a retry. Never queue automatic publication.
 */
export function preparePreview(draft: AuctionDraft, tenantId: string, idempotencyKey: string,
  now: number = Date.now()): PreparedPreview {
  if (!draft.artworkId) throw new Error('artwork_not_selected');
  if (!/^[A-Za-z0-9_-]{3,80}$/.test(tenantId)) throw new Error('invalid_tenant');
  if (!/^[A-Za-z0-9_-]{16,96}$/.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(draft.slug) || draft.slug.length > 60) throw new Error('invalid_slug');
  if (draft.title.trim().length < 3 || draft.title.length > 160) throw new Error('invalid_title');
  if (draft.description.length > 1500) throw new Error('invalid_description');
  const start = utc(draft.startsAt), end = utc(draft.endsAt);
  if (!Number.isFinite(now) || start < now || end <= start || end - start > 7 * 86_400_000) {
    throw new Error('invalid_schedule');
  }
  const startingAmount = cents(draft.startingAmount);
  const reserve = draft.reserveAmount === null || draft.reserveAmount === undefined ? null : cents(draft.reserveAmount);
  const terms = paymentTerms(draft);
  const auctionId = `auc-${idempotencyKey}`;
  // Explicit allowlist: never spread the draft or the artwork into a network payload.
  // In this endpoint the server expects epoch milliseconds (UTC) and cents; currency is always EUR.
  // Anti-sniping (2 + 2 minutes) is applied by the server.
  const body = JSON.stringify({
    auction: { id: auctionId, tenantId, startsAt: start, endsAt: end,
      startingAmount, incrementAmount: cents(draft.minimumIncrementAmount), ...(reserve === null ? {} : { reserveAmount: reserve }) },
    details: { slug: draft.slug, title: draft.title.trim(), description: draft.description.trim() },
    terms,
  });
  if (new TextEncoder().encode(body).length > 8192) throw new Error('payload_too_large');
  return Object.freeze({ artworkId: draft.artworkId, auctionId, idempotencyKey, body });
}

export interface ReviewRow { readonly field: string; readonly value: string }

/**
 * Validates the draft exactly as the future request would and lists the fields that would be sent.
 * The tenant is a placeholder (assigned by the server after sign-in) and is not listed.
 * Nothing is persisted or transmitted. Throws the validation error code of preparePreview.
 */
export function reviewRows(draft: AuctionDraft, now: number = Date.now()): ReviewRow[] {
  const prepared = preparePreview(draft, 'review-offline', 'review-offline-0123456789', now);
  const body = JSON.parse(prepared.body) as { auction: Record<string, number | string>; details: Record<string, string>;
    terms: { italyShippingCostAmount: number; bankTransfer: Record<string, string> } };
  // Exactly the fields of the request; tenant and generated id are assigned at sign-in / send time.
  return [
    { field: 'title', value: body.details['title'] },
    { field: 'slug', value: body.details['slug'] },
    { field: 'description', value: body.details['description'] },
    { field: 'startsAt', value: new Date(body.auction['startsAt'] as number).toISOString() },
    { field: 'endsAt', value: new Date(body.auction['endsAt'] as number).toISOString() },
    { field: 'startingAmount', value: String(body.auction['startingAmount']) },
    { field: 'minimumIncrementAmount', value: String(body.auction['incrementAmount']) },
    ...(body.auction['reserveAmount'] === undefined ? [] : [{ field: 'reserveAmount', value: String(body.auction['reserveAmount']) }]),
    { field: 'currency', value: 'EUR' },
    { field: 'italyShippingCost', value: String(body.terms.italyShippingCostAmount) },
    { field: 'bankBeneficiary', value: body.terms.bankTransfer['beneficiary'] },
    { field: 'bankIban', value: body.terms.bankTransfer['iban'] },
    { field: 'bankBicSwift', value: body.terms.bankTransfer['bicSwift'] },
    { field: 'bankNameAndAddress', value: body.terms.bankTransfer['bankNameAndAddress'] },
  ];
}
