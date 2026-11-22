/**
 * Price of an artwork, as in the Gestionale Opere and in the archive's own calculation:
 *  - a fixed price set by hand always wins;
 *  - otherwise width × height × rate per cm² × coefficient, where the coefficient is the artwork's own
 *    or, when it has none, the artist's one from the settings.
 * Amounts are in cents. This mirrors `finance::price_cents` in the native code.
 */
export interface PriceRules {
  /** Rate per square centimetre, in cents (16 = 0,16 €). */
  rateCm2Cents: number;
  artistCoefficient: number;
}
export type PriceSource = 'fixed' | 'own' | 'artist';
export interface Price {
  /** null when the size is missing (and no fixed price is set). */
  cents: number | null;
  source: PriceSource;
  /** Coefficient that was (or would be) applied. */
  coefficient: number;
}
export type PriceMode = 'auto' | 'coefficient' | 'fixed';

/** Sizes used to show what a coefficient means (width × height in cm). */
export const EXAMPLE_SIZES: readonly { width: number; height: number }[] = [{ width: 30, height: 40 }, { width: 90, height: 120 }];

const valid = (n: number | null | undefined): n is number => typeof n === 'number' && Number.isFinite(n) && n > 0 && n <= 100_000;

export function priceOf(width: number | null | undefined, height: number | null | undefined, rules: PriceRules,
  ownCoefficient: number | null | undefined, fixedPriceCents: number | null | undefined): Price {
  const own = valid(ownCoefficient) ? ownCoefficient : null;
  const coefficient = own ?? rules.artistCoefficient;
  if (typeof fixedPriceCents === 'number' && Number.isFinite(fixedPriceCents) && fixedPriceCents >= 0) {
    return { cents: Math.round(fixedPriceCents), source: 'fixed', coefficient };
  }
  const source: PriceSource = own !== null ? 'own' : 'artist';
  if (!valid(width) || !valid(height)) return { cents: null, source, coefficient };
  const cents = width * height * rules.rateCm2Cents * coefficient;
  return { cents: Number.isFinite(cents) && cents <= 100_000_000_000 ? Math.round(cents) : null, source, coefficient };
}

/** Which way of setting the price an artwork currently uses. */
export function priceModeOf(ownCoefficient: number | null | undefined, fixedPriceCents: number | null | undefined): PriceMode {
  if (fixedPriceCents !== null && fixedPriceCents !== undefined) return 'fixed';
  if (ownCoefficient !== null && ownCoefficient !== undefined) return 'coefficient';
  return 'auto';
}

/** Price of a standard size for a coefficient, to show what the coefficient does. */
export function examplePriceCents(size: { width: number; height: number }, rules: PriceRules, coefficient: number): number | null {
  if (!valid(coefficient)) return null;
  return priceOf(size.width, size.height, { ...rules, artistCoefficient: coefficient }, null, null).cents;
}

export interface PriceFields { artist_coefficient_override: number | null; fixed_price_cents: number | null; }
/** Values to store for the chosen mode, or an error message (Italian interface string) when the input is incomplete. */
export function priceFieldsFor(mode: PriceMode, ownCoefficient: number | null, fixedPriceCents: number | null): PriceFields | { error: string } {
  if (mode === 'coefficient') {
    return valid(ownCoefficient) ? { artist_coefficient_override: ownCoefficient, fixed_price_cents: null }
      : { error: 'Inserisci un coefficiente maggiore di zero oppure scegli un altro modo di calcolare il prezzo' };
  }
  if (mode === 'fixed') {
    return typeof fixedPriceCents === 'number' && Number.isFinite(fixedPriceCents) && fixedPriceCents >= 0 && fixedPriceCents <= 100_000_000_000
      ? { artist_coefficient_override: null, fixed_price_cents: Math.round(fixedPriceCents) }
      : { error: 'Inserisci il prezzo fisso oppure scegli un altro modo di calcolare il prezzo' };
  }
  return { artist_coefficient_override: null, fixed_price_cents: null };
}
