import { WINNER_BODY_MAX, WINNER_BODY_MIN, WINNER_MODES, WINNER_SUBJECT_MAX, type WinnerMode } from './auction-winner-email';

/**
 * Offline-only boundary for one explicitly selected artwork. No transport or credentials.
 * The page address is not part of it: the site derives it from the public title.
 */
export interface AuctionDraft {
  readonly artworkId: string;
  title: string;
  description: string;
  startsAt: string;
  endsAt: string;
  startingAmount: number | null;
  minimumIncrementAmount: number | null;
  /** Optional reserve price in cents (Gestionale Opere). */
  reserveAmount?: number | null;
  /** The artist's e-mail: it receives the winner's details, and the replies to the custom winner e-mail. */
  artistEmail?: string;
  /** How the winner is reached after closing (owner decision, 27 November 2026). */
  winnerMode?: WinnerMode;
  winnerSubject?: string;
  winnerBody?: string;
}

/** Public artwork details shown on the auction page (Gestionale Opere), taken from the archive. */
export interface PublicArtwork {
  readonly year: number | null;
  readonly technique: string | null;
  readonly support: string | null;
  readonly widthCm: number | null;
  readonly heightCm: number | null;
  readonly depthCm: number | null;
}

/** Only these archive fields become public; text that the server would refuse is left out. */
export function publicArtworkFrom(artwork: { year?: string | null; technique?: string | null; support?: string | null;
  width_cm?: number | null; height_cm?: number | null; depth_cm?: number | null } | null | undefined): PublicArtwork | null {
  if (!artwork) return null;
  const text = (v: string | null | undefined) => {
    const t = typeof v === 'string' ? v.trim() : '';
    return t.length >= 2 && t.length <= 120 && !/[\u0000-\u001f\u007f<>]/.test(t) ? t : null;
  };
  const cm = (v: number | null | undefined) => typeof v === 'number' && Number.isFinite(v) && v > 0 && v <= 100_000 ? Math.round(v * 100) / 100 : null;
  const year = typeof artwork.year === 'string' && /^\d{4}$/.test(artwork.year.trim()) ? Number(artwork.year.trim()) : null;
  return { year: year !== null && year >= 1000 && year <= 2200 ? year : null, technique: text(artwork.technique),
    support: text(artwork.support), widthCm: cm(artwork.width_cm), heightCm: cm(artwork.height_cm), depthCm: cm(artwork.depth_cm) };
}

/** What goes with the publication besides the draft: the staged main image and the public artwork details. */
export interface PublicationMedia { readonly mainImageId: string; readonly artwork: PublicArtwork | null }

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
  return { artworkId: artwork.id, title: artwork.title, description: '',
    startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: null, reserveAmount: null,
    artistEmail: '', winnerMode: 'contact_privately', winnerSubject: '', winnerBody: '' };
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

const ADDRESS = /^[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]{1,64}@[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+$/;

/** How the winner is reached, validated exactly as the server does (contract artist-v1, "terms"). */
export function winnerTerms(draft: AuctionDraft) {
  const email = (draft.artistEmail ?? '').trim();
  if (email.length > 254 || !ADDRESS.test(email)) throw new Error('invalid_artist_email');
  const mode = draft.winnerMode ?? 'contact_privately';
  if (!WINNER_MODES.includes(mode)) throw new Error('invalid_terms');
  if (mode === 'contact_privately') return { artistEmail: email.toLowerCase(), winner: { mode } };
  const subject = (draft.winnerSubject ?? '').trim();
  if (subject.length < 3 || subject.length > WINNER_SUBJECT_MAX || /[\u0000-\u001f\u007f]/.test(subject)) throw new Error('invalid_winner_subject');
  const body = (draft.winnerBody ?? '').replace(/\r\n?/g, '\n').trim();
  if (body.length < WINNER_BODY_MIN || body.length > WINNER_BODY_MAX || /[\u0000-\u0008\u000b-\u001f\u007f]/.test(body)) {
    throw new Error('invalid_winner_body');
  }
  return { artistEmail: email.toLowerCase(), winner: { mode, subject, body } };
}

/** Call only after the user reviews the public fields and chooses a tenant.
 * Persist the returned body + key before attempting a request; retries must reuse both.
 * A changed payload is a new attempt, not a retry. Never queue automatic publication.
 */
export function preparePreview(draft: AuctionDraft, tenantId: string, idempotencyKey: string,
  now: number = Date.now(), media?: PublicationMedia): PreparedPreview {
  if (!draft.artworkId) throw new Error('artwork_not_selected');
  if (!/^[A-Za-z0-9_-]{3,80}$/.test(tenantId)) throw new Error('invalid_tenant');
  if (!/^[A-Za-z0-9_-]{16,96}$/.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
  if (draft.title.trim().length < 3 || draft.title.length > 160) throw new Error('invalid_title');
  if (draft.description.length > 1500) throw new Error('invalid_description');
  const start = utc(draft.startsAt), end = utc(draft.endsAt);
  if (!Number.isFinite(now) || start < now || end <= start || end - start > 7 * 86_400_000) {
    throw new Error('invalid_schedule');
  }
  const startingAmount = cents(draft.startingAmount);
  const reserve = draft.reserveAmount === null || draft.reserveAmount === undefined ? null : cents(draft.reserveAmount);
  const terms = winnerTerms(draft);
  if (media && !/^[a-f0-9]{64}$/.test(media.mainImageId)) throw new Error('image_required');
  const artwork = media?.artwork ? Object.fromEntries(Object.entries(media.artwork).filter(([, v]) => v !== null)) : null;
  const auctionId = `auc-${idempotencyKey}`;
  // Explicit allowlist: never spread the draft or the artwork into a network payload.
  // In this endpoint the server expects epoch milliseconds (UTC) and cents; currency is always EUR.
  // Anti-sniping (2 + 2 minutes) is applied by the server.
  const body = JSON.stringify({
    auction: { id: auctionId, tenantId, startsAt: start, endsAt: end,
      startingAmount, incrementAmount: cents(draft.minimumIncrementAmount), ...(reserve === null ? {} : { reserveAmount: reserve }) },
    details: { title: draft.title.trim(), description: draft.description.trim() },
    terms,
    ...(media ? { images: { main: media.mainImageId }, ...(artwork && Object.keys(artwork).length ? { artwork } : {}) } : {}),
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
export function reviewRows(draft: AuctionDraft, now: number = Date.now(), artwork: PublicArtwork | null = null): ReviewRow[] {
  const prepared = preparePreview(draft, 'review-offline', 'review-offline-0123456789', now);
  const body = JSON.parse(prepared.body) as { auction: Record<string, number | string>; details: Record<string, string>;
    terms: { artistEmail: string; winner: { mode: WinnerMode; subject?: string; body?: string } } };
  // Exactly the fields of the request; tenant and generated id are assigned at sign-in / send time.
  return [
    { field: 'title', value: body.details['title'] },
    { field: 'description', value: body.details['description'] },
    { field: 'startsAt', value: new Date(body.auction['startsAt'] as number).toISOString() },
    { field: 'endsAt', value: new Date(body.auction['endsAt'] as number).toISOString() },
    { field: 'startingAmount', value: String(body.auction['startingAmount']) },
    { field: 'minimumIncrementAmount', value: String(body.auction['incrementAmount']) },
    ...(body.auction['reserveAmount'] === undefined ? [] : [{ field: 'reserveAmount', value: String(body.auction['reserveAmount']) }]),
    { field: 'currency', value: 'EUR' },
    { field: 'artistEmail', value: body.terms.artistEmail },
    { field: 'winnerMode', value: body.terms.winner.mode },
    ...(body.terms.winner.mode === 'custom_email' ? [{ field: 'winnerSubject', value: body.terms.winner.subject ?? '' },
      { field: 'winnerBody', value: body.terms.winner.body ?? '' }] : []),
    // Gestionale Opere: the main image is required; it is re-encoded on this computer when publishing.
    { field: 'mainImage', value: 'main' },
    ...(artwork?.year ? [{ field: 'year', value: String(artwork.year) }] : []),
    ...(artwork?.technique ? [{ field: 'technique', value: artwork.technique }] : []),
    ...(artwork?.support ? [{ field: 'support', value: artwork.support }] : []),
    ...(artwork && (artwork.widthCm || artwork.heightCm || artwork.depthCm)
      ? [{ field: 'dimensions', value: [artwork.widthCm, artwork.heightCm, artwork.depthCm].filter(v => v !== null).join(' × ') + ' cm' }] : []),
  ];
}
