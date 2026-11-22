import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { EXAMPLE_SIZES, examplePriceCents, priceFieldsFor, priceModeOf, priceOf } from '../src/app/pricing.ts';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const rules = { rateCm2Cents: 16, artistCoefficient: 1 };

test('a fixed price wins, then the own coefficient, then the artist coefficient', () => {
  assert.deepEqual(priceOf(30, 40, rules, null, null), { cents: 19_200, source: 'artist', coefficient: 1 });
  assert.equal(priceOf(30, 40, { ...rules, artistCoefficient: 2 }, null, null).cents, 38_400);
  assert.deepEqual(priceOf(30, 40, { ...rules, artistCoefficient: 2 }, 1.5, null), { cents: 28_800, source: 'own', coefficient: 1.5 });
  assert.deepEqual(priceOf(30, 40, rules, 1.5, 50_000), { cents: 50_000, source: 'fixed', coefficient: 1.5 });
  assert.equal(priceOf(null, null, rules, null, 50_000).cents, 50_000, 'a fixed price needs no size');
  assert.equal(priceOf(null, 40, rules, null, null).cents, null);
  assert.equal(priceOf(30, 40, rules, null, 0).cents, 0, 'a fixed price of zero is a price');
  for (const bad of [0, -2, Number.NaN, Infinity, 1e9]) assert.equal(priceOf(30, 40, rules, bad, null).source, 'artist', `invalid own coefficient ${bad} is ignored`);
});

test('the interface formula gives the same cents as the native one', () => {
  // The same fixtures as finance::price_cents in the native tests.
  assert.equal(priceOf(30, 40, rules, null, null).cents, 19_200);
  assert.equal(priceOf(90, 120, { rateCm2Cents: 16, artistCoefficient: 1.25 }, null, null).cents, 216_000);
  assert.equal(priceOf(30, 40, { rateCm2Cents: 16, artistCoefficient: 2 }, 1.5, null).cents, 28_800);
});

test('the first-start examples show what a coefficient does for 30x40 and 90x120', () => {
  assert.deepEqual(EXAMPLE_SIZES.map(s => [s.width, s.height]), [[30, 40], [90, 120]]);
  assert.deepEqual(EXAMPLE_SIZES.map(size => examplePriceCents(size, rules, 1)), [19_200, 172_800]);
  assert.deepEqual(EXAMPLE_SIZES.map(size => examplePriceCents(size, rules, 1.5)), [28_800, 259_200]);
  assert.equal(examplePriceCents(EXAMPLE_SIZES[0], rules, 0), null, 'an invalid coefficient shows no price');
});

test('the price mode follows what is stored and only complete input can be saved', () => {
  assert.equal(priceModeOf(null, null), 'auto');
  assert.equal(priceModeOf(1.2, null), 'coefficient');
  assert.equal(priceModeOf(null, 100_000), 'fixed');
  assert.equal(priceModeOf(1.2, 0), 'fixed', 'a fixed price (even zero) wins');
  assert.deepEqual(priceFieldsFor('auto', 1.5, 5000), { artist_coefficient_override: null, fixed_price_cents: null }, 'automatic clears both');
  assert.deepEqual(priceFieldsFor('coefficient', 1.5, 5000), { artist_coefficient_override: 1.5, fixed_price_cents: null });
  assert.deepEqual(priceFieldsFor('fixed', 1.5, 5000), { artist_coefficient_override: null, fixed_price_cents: 5000 });
  assert.ok('error' in priceFieldsFor('coefficient', null, null));
  assert.ok('error' in priceFieldsFor('coefficient', 0, null));
  assert.ok('error' in priceFieldsFor('fixed', null, null));
  assert.ok('error' in priceFieldsFor('fixed', null, -1));
});

test('price rules are stored per artwork, shown in the archive and asked at the first start', () => {
  assert.match(get('src-tauri/src/db.rs'), /ALTER TABLE artworks ADD COLUMN artist_coefficient_override/);
  assert.match(get('src-tauri/src/db.rs'), /ALTER TABLE artworks ADD COLUMN fixed_price_cents/);
  assert.match(get('src-tauri/src/finance.rs'), /pub fn price_cents/);
  assert.doesNotMatch(get('src-tauri/src/lib.rs'), /price_preview/);
  const html = get('src/app/app.component.html');
  assert.match(html, /name="price-mode"/);
  assert.match(html, /priceLabel\(a\)/);
  assert.match(html, /saveArtistProfile\(profileDraft\.name, profileDraft\.alias, profileDraft\.coefficient\)/);
  assert.match(html, /exampleCents\(size, profileDraft\.coefficient\)/);
  assert.match(get('src/app/app.component.ts'), /EXAMPLE_SIZES/);
  assert.match(get('src/app/brochures.component.ts'), /artist: this\.artistName/);
  assert.match(get('src/app/certificate-sheet.component.ts'), /c\.declarant_name \|\| c\.artist_name/);
});
