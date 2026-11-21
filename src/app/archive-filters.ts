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

interface SearchTerm {
  value: string;
  excluded: boolean;
}

/**
 * Unquoted words are independent AND terms; quoted phrases must match within
 * one field. A leading minus excludes a word or a quoted phrase, for example
 * `olio -tempera -"su tela"`. Lone '-' is ignored. Unclosed quotes still work.
 */
function searchTerms(query: string): SearchTerm[] {
  const terms: SearchTerm[] = [];
  let token = '';
  let quoted = false;
  let negateQuoted = false;
  const push = (wasQuoted = false) => {
    const term = token.replace(/\s+/gu, ' ').trim();
    if (term && term !== '-') {
      const excluded = negateQuoted || (!wasQuoted && term.startsWith('-'));
      const value = excluded && !negateQuoted ? term.slice(1) : term;
      if (value) terms.push({ value, excluded });
    }
    token = '';
    negateQuoted = false;
  };
  for (const character of foldSearch(query)) {
    if (character === '"') {
      if (quoted) {
        push(true);
        quoted = false;
      } else {
        if (token === '-') {
          token = '';
          negateQuoted = true;
        } else {
          push();
        }
        quoted = true;
      }
    } else if (!quoted && /\s/u.test(character)) {
      push();
    } else {
      token += character;
    }
  }
  push(quoted);
  return terms;
}

function matchesTerms(terms: SearchTerm[], fields: Array<string | null | undefined>): boolean {
  if (terms.length === 0) return true;
  // A quoted phrase cannot span fields. Exclusions apply to the same fields
  // as positive terms, including the linked collector or purchased artwork.
  const normalizedFields = fields.map(field => foldSearch(field ?? ''));
  return terms.every(({ value, excluded }) => {
    const found = normalizedFields.some(field => field.includes(value));
    return excluded ? !found : found;
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
    matchesTerms(terms, [a.title, a.code, a.year, a.technique, a.support, a.location,
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

export function filterCollectors(
  collectors: Collector[],
  query: string,
  indexes: SaleIndexes
): Collector[] {
  const terms = searchTerms(query);
  if (terms.length === 0) return collectors;
  return collectors.filter(c =>
    matchesTerms(terms, [c.name, c.email, c.phone, c.address, c.notes,
      ...(indexes.byCollector[c.id] ?? []).map(s => s.artwork_title)]));
}
