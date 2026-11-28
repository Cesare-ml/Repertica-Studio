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

const MAX_PRICE_CENTS = 100_000_000_000;
const valid = (n: number | null | undefined): n is number => typeof n === 'number' && Number.isFinite(n) && n > 0 && n <= 100_000;
const validFixed = (value: number | null | undefined): value is number =>
  typeof value === 'number' && Number.isSafeInteger(value) && value >= 0 && value <= MAX_PRICE_CENTS;

export function priceOf(width: number | null | undefined, height: number | null | undefined, rules: PriceRules,
  ownCoefficient: number | null | undefined, fixedPriceCents: number | null | undefined): Price {
  const own = ownCoefficient ?? null;
  const coefficient = own ?? rules.artistCoefficient;
  // A corrupt/imported fixed amount must not silently fall back to a calculated
  // estimate. Keep its selected mode visible, but refuse to display false cents.
  if (fixedPriceCents !== null && fixedPriceCents !== undefined) {
    return { cents: validFixed(fixedPriceCents) ? fixedPriceCents : null, source: 'fixed', coefficient };
  }
  const source: PriceSource = own !== null ? 'own' : 'artist';
  if (!valid(width) || !valid(height) || !valid(coefficient)
    || !Number.isSafeInteger(rules.rateCm2Cents) || rules.rateCm2Cents < 0 || rules.rateCm2Cents > 100_000) {
    return { cents: null, source, coefficient };
  }
  const cents = width * height * rules.rateCm2Cents * coefficient;
  return { cents: Number.isFinite(cents) && cents >= 0 && cents <= MAX_PRICE_CENTS ? Math.round(cents) : null, source, coefficient };
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
    return validFixed(fixedPriceCents)
      ? { artist_coefficient_override: null, fixed_price_cents: fixedPriceCents }
      : { error: 'Inserisci il prezzo fisso oppure scegli un altro modo di calcolare il prezzo' };
  }
  return { artist_coefficient_override: null, fixed_price_cents: null };
}

/** The editor's selected mode is authoritative even when its input is empty. */
export function draftPriceFor(mode: PriceMode, width: number | null | undefined, height: number | null | undefined,
  rules: PriceRules, ownCoefficient: number | null, fixedPriceCents: number | null): Price {
  const fields = priceFieldsFor(mode, ownCoefficient, fixedPriceCents);
  if ('error' in fields) {
    return { cents: null, source: mode === 'fixed' ? 'fixed' : 'own', coefficient: ownCoefficient ?? rules.artistCoefficient };
  }
  return priceOf(width, height, rules, fields.artist_coefficient_override, fields.fixed_price_cents);
}

/** Reject out-of-range euro input before rounding (including negative sub-cent amounts). */
export function fixedPriceCentsFromEuros(value: number | null | undefined): number | null {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0 || value > MAX_PRICE_CENTS / 100) return null;
  const cents = Math.round(value * 100);
  return validFixed(cents) ? cents : null;
}
