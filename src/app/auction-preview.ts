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
}

export interface PreparedPreview {
  readonly artworkId: string;
  readonly idempotencyKey: string;
  readonly body: string;
}

// Private archive description, location, images, prices and buyer information are not defaults.
export function newAuctionDraft(artwork: { id: string; title: string; status: string }): AuctionDraft {
  if (!artwork.id || artwork.status !== 'available') throw new Error('artwork_not_available');
  return { artworkId: artwork.id, title: artwork.title, description: '', slug: '',
    startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: null };
}

function utc(value: string): number {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)) {
    throw new Error('invalid_schedule');
  }
  const time = Date.parse(value);
  if (!Number.isFinite(time) || new Date(time).toISOString() !== value) throw new Error('invalid_schedule');
  return time;
}

function cents(value: number | null): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 1 || value > 1_000_000_000) {
    throw new Error('invalid_amount');
  }
  return value;
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
  // Explicit allowlist: never spread the draft or the artwork into a network payload.
  const body = JSON.stringify({ tenantId, slug: draft.slug, title: draft.title.trim(),
    description: draft.description.trim(), startsAt: draft.startsAt, endsAt: draft.endsAt,
    startingAmount: cents(draft.startingAmount), minimumIncrementAmount: cents(draft.minimumIncrementAmount),
    currency: 'EUR' });
  if (new TextEncoder().encode(body).length > 8192) throw new Error('payload_too_large');
  return Object.freeze({ artworkId: draft.artworkId, idempotencyKey, body });
}
