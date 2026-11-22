import type { Artwork, ArtworkStatus, Collector, Sale } from './models';

export type ArtworkOrder = 'recent' | 'title' | 'year';

export interface ArtworkFilters {
  query: string;
  status: ArtworkStatus | 'all';
  collectorId: string;
  order: ArtworkOrder;
}

export interface SaleIndexes {
  byCollector: Record<string, Sale[]>;
  byArtwork: Record<string, Sale>;
}

/** Pure, in-memory indexes. Keys come from local UUIDs but must not mutate Object.prototype. */
export function buildSaleIndexes(sales: Sale[]): SaleIndexes {
  const byCollector: Record<string, Sale[]> = Object.create(null);
  const byArtwork: Record<string, Sale> = Object.create(null);
  for (const sale of sales) {
    (byCollector[sale.collector_id] ??= []).push(sale);
    byArtwork[sale.artwork_id] = sale;
  }
  return { byCollector, byArtwork };
}

/** Normalize Latin diacritics, case and whitespace for local search. */
function foldSearch(value: string): string {
  return value.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
    .replace(/\s+/gu, ' ').trim();
}

/**
 * Unquoted words are independent AND terms. Double quotes keep a sequence of
 * words together, e.g. `"olio su tela" marina` finds the phrase in one field
 * and "marina" in any other field. An unclosed quote includes the remainder.
 */
function searchTerms(query: string): string[] {
  const terms: string[] = [];
  let token = '';
  let quoted = false;
  const push = () => {
    const term = token.replace(/\s+/gu, ' ').trim();
    if (term) terms.push(term);
    token = '';
  };
  for (const character of foldSearch(query)) {
    if (character === '"') {
      push();
      quoted = !quoted;
    } else if (!quoted && /\s/u.test(character)) {
      push();
    } else {
      token += character;
    }
  }
  push();
  return terms;
}

/**
 * Phone numbers are commonly pasted without the spaces, parentheses or dashes
 * used in the saved contact. Keep this alternative strictly numeric and at
 * least five digits long: normal words, years and quoted prose remain literal.
 */
function phoneDigitsForSearch(term: string): string | null {
  if (!/^[+\d\s().\/-]+$/u.test(term)) return null;
  const digits = term.replace(/\D/gu, '');
  return digits.length >= 5 ? digits : null;
}

function matchesTerms(
  terms: string[], fields: Array<string | null | undefined>, phone?: string | null
): boolean {
  if (terms.length === 0) return true;
  // A quoted phrase cannot span multiple unrelated fields. The phone
  // alternative only compares digits against this collector's own phone.
  const normalizedFields = fields.map(field => foldSearch(field ?? ''));
  const savedPhoneDigits = (phone ?? '').replace(/\D/gu, '');
  return terms.every(term => {
    if (normalizedFields.some(field => field.includes(term))) return true;
    const digits = phoneDigitsForSearch(term);
    return digits !== null && savedPhoneDigits.includes(digits);
  });
}

export function filterArtworks(
  artworks: Artwork[],
  filters: ArtworkFilters,
  indexes: SaleIndexes
): Artwork[] {
  const terms = searchTerms(filters.query);
  const acquiredIds = filters.collectorId === 'all' ? null :
    new Set((indexes.byCollector[filters.collectorId] ?? []).map(s => s.artwork_id));
  const visible = artworks.filter(a =>
    (filters.status === 'all' || a.status === filters.status) &&
    (acquiredIds === null || acquiredIds.has(a.id)) &&
    matchesTerms(terms, [a.title, a.code, a.year, a.technique, a.support,
      a.description, indexes.byArtwork[a.id]?.collector_name]));

  if (filters.order === 'title') {
    return visible.sort((a, b) =>
      a.title.localeCompare(b.title, 'it', { numeric: true, sensitivity: 'base' }) ||
      a.code.localeCompare(b.code, 'it', { numeric: true }));
  }
  if (filters.order === 'year') {
    // Free-text years (e.g. "circa 2000") sort after numeric years.
    const year = (a: Artwork) => /^\d{1,4}$/.test(a.year?.trim() ?? '') ?
      Number(a.year) : Number.NEGATIVE_INFINITY;
    return visible.sort((a, b) => {
      const diff = year(a) === year(b) ? 0 : year(b) - year(a);
      return diff || a.title.localeCompare(b.title, 'it', { sensitivity: 'base' });
    });
  }
  return visible; // Backend delivers updated_at descending.
}

export type CollectorFilter = 'all' | 'buyers' | 'incomplete';

export function filterCollectors(
  collectors: Collector[],
  query: string,
  indexes: SaleIndexes,
  filter: CollectorFilter = 'all'
): Collector[] {
  const base = filter === 'all' ? collectors : collectors.filter(c => filter === 'buyers'
    ? (indexes.byCollector[c.id]?.length ?? 0) > 0
    : !c.email && !c.phone);
  const terms = searchTerms(query);
  if (terms.length === 0) return base;
  return base.filter(c =>
    matchesTerms(terms, [c.name, c.email, c.phone, c.address, c.notes, c.birth_place,
      c.instagram, c.facebook, c.tiktok, c.website, c.linkedin,
      ...(indexes.byCollector[c.id] ?? []).map(s => s.artwork_title)], c.phone));
}
