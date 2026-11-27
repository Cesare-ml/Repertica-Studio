/**
 * Local auction drafts. They are marked local, hold only the public fields chosen for one artwork and are
 * never sent anywhere by this module. Persistence goes through an injectable key/value store.
 */
import type { AuctionDraft } from './auction-preview';
import { WINNER_MODES, type SavedBankDetails, type WinnerMode } from './auction-winner-email';

export interface KeyValueStore {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export interface LocalAuctionDraft extends AuctionDraft {
  readonly local: true;
  readonly updatedAt: string;
}

const KEY = 'repertica-auction-drafts-v1';
const MAX_DRAFTS = 50;

function isDraft(value: unknown): value is LocalAuctionDraft {
  if (value === null || typeof value !== 'object') return false;
  const d = value as Record<string, unknown>;
  const str = (v: unknown, max: number) => typeof v === 'string' && v.length <= max;
  const amount = (v: unknown) => v === null || (typeof v === 'number' && Number.isSafeInteger(v) && v >= 0 && v <= 1_000_000_000);
  return d['local'] === true && typeof d['artworkId'] === 'string' && d['artworkId'] !== '' && d['artworkId'].length <= 96
    && str(d['title'], 160) && str(d['description'], 1500)
    && str(d['startsAt'], 40) && str(d['endsAt'], 40) && str(d['updatedAt'], 40)
    && amount(d['startingAmount']) && amount(d['minimumIncrementAmount'])
    // Fields added later are optional in drafts saved before; older fields (page address, shipping and bank
    // transfer, before 27 November 2026) are ignored.
    && (d['reserveAmount'] === undefined || amount(d['reserveAmount']))
    && (d['artistEmail'] === undefined || str(d['artistEmail'], 254))
    && (d['winnerMode'] === undefined || WINNER_MODES.includes(d['winnerMode'] as WinnerMode))
    && (d['winnerSubject'] === undefined || str(d['winnerSubject'], 300))
    && (d['winnerBody'] === undefined || str(d['winnerBody'], 6000));
}

export function loadDrafts(store: KeyValueStore): LocalAuctionDraft[] {
  try {
    const raw = store.getItem(KEY);
    if (!raw || raw.length > 400_000) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    const seen = new Set<string>();
    const drafts: LocalAuctionDraft[] = [];
    for (const item of parsed) {
      if (!isDraft(item) || seen.has(item.artworkId)) continue;
      seen.add(item.artworkId);
      drafts.push(Object.freeze(pick(item)));
    }
    return drafts.slice(0, MAX_DRAFTS);
  } catch { return []; }
}

// Explicit allowlist: stored data never carries anything but the draft fields.
function pick(d: LocalAuctionDraft): LocalAuctionDraft {
  return { artworkId: d.artworkId, title: d.title, description: d.description,
    startsAt: d.startsAt, endsAt: d.endsAt, startingAmount: d.startingAmount,
    minimumIncrementAmount: d.minimumIncrementAmount, reserveAmount: d.reserveAmount ?? null,
    artistEmail: d.artistEmail ?? '', winnerMode: d.winnerMode ?? 'contact_privately', winnerSubject: d.winnerSubject ?? '',
    winnerBody: d.winnerBody ?? '', local: true, updatedAt: d.updatedAt };
}

function persist(store: KeyValueStore, drafts: LocalAuctionDraft[]): void {
  store.setItem(KEY, JSON.stringify(drafts.slice(0, MAX_DRAFTS)));
}

/** One draft per artwork: saving replaces the previous draft of the same work. */
export function saveDraft(store: KeyValueStore, draft: AuctionDraft, now: Date = new Date()): LocalAuctionDraft[] {
  if (!draft.artworkId) throw new Error('artwork_not_selected');
  const next = Object.freeze(pick({ ...draft, local: true, updatedAt: now.toISOString() }));
  const others = loadDrafts(store).filter(d => d.artworkId !== draft.artworkId);
  const all = [next, ...others].slice(0, MAX_DRAFTS);
  persist(store, all);
  return all;
}

const WINNER_KEY = 'repertica-auction-winner-v1';
/** Bank transfer details saved by versions before 27 November 2026; read only to fill in the e-mail template. */
const LEGACY_PAYMENT_KEY = 'repertica-auction-payment-v1';

export type WinnerDefaults = Pick<AuctionDraft, 'artistEmail' | 'winnerMode' | 'winnerSubject' | 'winnerBody'>;

/** The artist's last choices for the winner, kept on this computer only to prefill the next draft. */
export function loadWinnerDefaults(store: KeyValueStore): WinnerDefaults {
  try {
    const raw = store.getItem(WINNER_KEY);
    const d = raw && raw.length < 20_000 ? JSON.parse(raw) as Record<string, unknown> : {};
    const str = (v: unknown, max: number) => typeof v === 'string' && v.length <= max ? v : '';
    const mode = WINNER_MODES.includes(d['winnerMode'] as WinnerMode) ? d['winnerMode'] as WinnerMode : 'contact_privately';
    return { artistEmail: str(d['artistEmail'], 254), winnerMode: mode, winnerSubject: str(d['winnerSubject'], 300),
      winnerBody: str(d['winnerBody'], 6000) };
  } catch { return { artistEmail: '', winnerMode: 'contact_privately', winnerSubject: '', winnerBody: '' }; }
}

export function saveWinnerDefaults(store: KeyValueStore, draft: AuctionDraft): void {
  store.setItem(WINNER_KEY, JSON.stringify({ artistEmail: draft.artistEmail ?? '', winnerMode: draft.winnerMode ?? 'contact_privately',
    winnerSubject: draft.winnerSubject ?? '', winnerBody: draft.winnerBody ?? '' }));
}

export function loadSavedBankDetails(store: KeyValueStore): SavedBankDetails | null {
  try {
    const raw = store.getItem(LEGACY_PAYMENT_KEY);
    const d = raw && raw.length < 4000 ? JSON.parse(raw) as Record<string, unknown> : null;
    if (!d) return null;
    const str = (v: unknown) => typeof v === 'string' && v.length <= 300 && !/[\u0000-\u001f\u007f]/.test(v) ? v.trim() : '';
    const bank = { beneficiary: str(d['bankBeneficiary']), iban: str(d['bankIban']).replace(/\s+/g, '').toUpperCase(),
      bicSwift: str(d['bankBicSwift']).replace(/\s+/g, '').toUpperCase(), bankNameAndAddress: str(d['bankNameAndAddress']) };
    return bank.beneficiary || bank.iban ? bank : null;
  } catch { return null; }
}

export function removeDraft(store: KeyValueStore, artworkId: string): LocalAuctionDraft[] {
  const all = loadDrafts(store).filter(d => d.artworkId !== artworkId);
  persist(store, all);
  return all;
}


/** Fields that stay on the computer unless the artist adds them deliberately in a future, verified contract. */
export const NOT_SENT: readonly string[] = [
  'Note private e descrizione dell’archivio', 'Luogo e ubicazione', 'Immagini originali e RAW',
  'Prezzi e coefficienti interni', 'Collezionisti e vendite', 'Altre opere dell’archivio',
];

// --- conversions between form inputs and the strict contract formats ---

/** `datetime-local` value (local wall time) to an exact UTC instant; '' when invalid. */
export function localInputToUtc(value: string): string {
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(value)) return '';
  const time = new Date(value).getTime();
  return Number.isFinite(time) ? new Date(time).toISOString() : '';
}

export function utcToLocalInput(value: string): string {
  const time = Date.parse(value);
  if (!Number.isFinite(time)) return '';
  const d = new Date(time), p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** Euros typed by the artist ("120", "120,50") to cents; null when not a valid amount. */
export function euroToCents(value: string): number | null {
  const m = /^\s*(\d{1,9})(?:[.,](\d{1,2}))?\s*$/.exec(value);
  if (!m) return null;
  return Number(m[1]) * 100 + Number((m[2] ?? '').padEnd(2, '0') || '0');
}

export function centsToEuro(cents: number | null): string {
  if (cents === null) return '';
  return `${Math.floor(cents / 100)},${String(cents % 100).padStart(2, '0')}`;
}
