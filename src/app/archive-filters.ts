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

/** Normalize Latin diacritics and case for local, language-independent search. */
function foldSearch(value: string): string {
  return value.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
}

/**
 * All whitespace-separated terms must match, but they can appear in different
 * fields (for example an artwork's technique and its buyer's name).
 */
function searchTerms(query: string): string[] {
  return foldSearch(query).trim().split(/\s+/u).filter(Boolean);
}

function matchesTerms(terms: string[], fields: Array<string | null | undefined>): boolean {
  if (terms.length === 0) return true;
  const text = foldSearch(fields.map(field => field ?? '').join(' '));
  return terms.every(term => text.includes(term));
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
