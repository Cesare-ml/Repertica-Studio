/**
 * Local auction drafts. They are marked local, hold only the public fields chosen for one artwork and are
 * never sent anywhere by this module. Persistence goes through an injectable key/value store.
 */
import type { AuctionDraft } from './auction-preview';

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
    && str(d['title'], 160) && str(d['description'], 1500) && str(d['slug'], 60)
    && str(d['startsAt'], 40) && str(d['endsAt'], 40) && str(d['updatedAt'], 40)
    && amount(d['startingAmount']) && amount(d['minimumIncrementAmount'])
    // Fields added with the Gestionale Opere terms: optional in drafts saved before.
    && (d['reserveAmount'] === undefined || amount(d['reserveAmount']))
    && (d['italyShippingCost'] === undefined || amount(d['italyShippingCost']))
    && ['bankBeneficiary', 'bankIban', 'bankBicSwift', 'bankNameAndAddress'].every(k => d[k] === undefined || str(d[k], 300));
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
  return { artworkId: d.artworkId, title: d.title, description: d.description, slug: d.slug,
    startsAt: d.startsAt, endsAt: d.endsAt, startingAmount: d.startingAmount,
    minimumIncrementAmount: d.minimumIncrementAmount, reserveAmount: d.reserveAmount ?? null,
    italyShippingCost: d.italyShippingCost ?? null, bankBeneficiary: d.bankBeneficiary ?? '', bankIban: d.bankIban ?? '',
    bankBicSwift: d.bankBicSwift ?? '', bankNameAndAddress: d.bankNameAndAddress ?? '', local: true, updatedAt: d.updatedAt };
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

const PAYMENT_KEY = 'repertica-auction-payment-v1';

/** The artist's last payment terms, kept on this computer only to prefill the next draft. */
export function loadPaymentDefaults(store: KeyValueStore): Pick<AuctionDraft, 'italyShippingCost' | 'bankBeneficiary' | 'bankIban' | 'bankBicSwift' | 'bankNameAndAddress'> {
  try {
    const raw = store.getItem(PAYMENT_KEY);
    const d = raw && raw.length < 4000 ? JSON.parse(raw) as Record<string, unknown> : {};
    const str = (v: unknown) => typeof v === 'string' && v.length <= 300 ? v : '';
    const shipping = d['italyShippingCost'];
    return { italyShippingCost: typeof shipping === 'number' && Number.isSafeInteger(shipping) && shipping >= 0 && shipping <= 100_000_000 ? shipping : null,
      bankBeneficiary: str(d['bankBeneficiary']), bankIban: str(d['bankIban']), bankBicSwift: str(d['bankBicSwift']),
      bankNameAndAddress: str(d['bankNameAndAddress']) };
  } catch { return { italyShippingCost: null, bankBeneficiary: '', bankIban: '', bankBicSwift: '', bankNameAndAddress: '' }; }
}

export function savePaymentDefaults(store: KeyValueStore, draft: AuctionDraft): void {
  store.setItem(PAYMENT_KEY, JSON.stringify({ italyShippingCost: draft.italyShippingCost ?? null, bankBeneficiary: draft.bankBeneficiary ?? '',
    bankIban: draft.bankIban ?? '', bankBicSwift: draft.bankBicSwift ?? '', bankNameAndAddress: draft.bankNameAndAddress ?? '' }));
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

export function slugify(title: string): string {
  return title.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase()
    .replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 60).replace(/-+$/g, '');
}
