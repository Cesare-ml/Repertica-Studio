import type { AuctionDraft } from './auction-preview';

export type AuctionEditorFields = Omit<AuctionDraft, 'artworkId' | 'startingAmount' | 'minimumIncrementAmount'> & {
  startingEuros: string;
  incrementEuros: string;
};

/** Parse decimal user input directly into cents, without floating-point rounding. */
export function parseAuctionEuros(raw: string): number | null {
  const value = raw.trim();
  if (!value) return null;
  const match = /^(\d{1,8})(?:[.,](\d{1,2}))?$/.exec(value);
  if (!match) throw new Error('invalid_amount');
  const cents = Number(match[1]) * 100 + Number((match[2] ?? '').padEnd(2, '0'));
  if (cents > 1_000_000_000) throw new Error('invalid_amount');
  return cents;
}

function euros(cents: number | null): string {
  if (cents === null) return '';
  if (!Number.isSafeInteger(cents) || cents < 0) return String(cents / 100);
  return `${Math.floor(cents / 100)}.${String(cents % 100).padStart(2, '0')}`;
}

/** Accept minute/second precision as explicitly UTC, not the computer's local time.
 * Incomplete or invalid strings stay editable in the local draft. */
export function normalizeAuctionUtc(raw: string): string {
  const value = raw.trim();
  const minute = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/;
  const second = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z?$/;
  const candidate = minute.test(value) ? value + ':00.000Z' : second.test(value) ? value.replace(/Z$/, '') + '.000Z' : value;
  const parsed = Date.parse(candidate);
  return Number.isFinite(parsed) && new Date(parsed).toISOString() === candidate ? candidate : raw;
}

export function fieldsFromAuctionDraft(draft: AuctionDraft): AuctionEditorFields {
  return { title: draft.title, description: draft.description, slug: draft.slug,
    startsAt: draft.startsAt, endsAt: draft.endsAt, startingEuros: euros(draft.startingAmount),
    incrementEuros: euros(draft.minimumIncrementAmount) };
}

export function auctionDraftEdits(fields: AuctionEditorFields): Omit<AuctionDraft, 'artworkId'> {
  return { title: fields.title, description: fields.description, slug: fields.slug,
    startsAt: normalizeAuctionUtc(fields.startsAt), endsAt: normalizeAuctionUtc(fields.endsAt),
    startingAmount: parseAuctionEuros(fields.startingEuros), minimumIncrementAmount: parseAuctionEuros(fields.incrementEuros) };
}

/** Do not display raw IPC/database errors, which can contain private paths. */
export function auctionEditorError(error: unknown): string {
  const code = error instanceof Error ? error.message : error;
  switch (code) {
    case 'auction_draft_conflict': return 'La bozza è cambiata. Conserva le modifiche e riaprila dopo averle annotate.';
    case 'artwork_not_available': return 'L’opera non è disponibile per una nuova bozza.';
    case 'invalid_amount': return 'Inserisci importi in euro con al massimo due decimali.';
    case 'invalid_stored_draft': return 'La bozza salvata non è leggibile. Nessun dato è stato sovrascritto.';
    default: return 'Operazione sulla bozza non riuscita. Nessun invio online effettuato.';
  }
}
