import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { EXAMPLE_SIZES, draftPriceFor, examplePriceCents, fixedPriceCentsFromEuros, priceFieldsFor, priceModeOf, priceOf } from '../src/app/pricing.ts';

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
  for (const bad of [0, -2, Number.NaN, Infinity, 1e9]) {
    const price = priceOf(30, 40, rules, bad, null);
    assert.equal(price.source, 'own');
    assert.equal(price.cents, null, `invalid own coefficient ${bad} is refused`);
  }
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

test('invalid fixed cents in an imported artwork never become a plausible automatic estimate', () => {
  // The mode remains fixed even when external SQLite edits supplied invalid
  // money: silently displaying the formula price could mislead a sale.
  for (const amount of [-1, 12.5, 100_000_000_001, NaN, Infinity]) {
    assert.deepEqual(priceOf(30, 40, rules, null, amount),
      { cents: null, source: 'fixed', coefficient: 1 }, String(amount));
    assert.equal(priceModeOf(null, amount), 'fixed');
    assert.ok('error' in priceFieldsFor('fixed', null, amount),
      'the form must refuse the same invalid cents');
  }
  assert.deepEqual(priceOf(30, 40, rules, null, 0),
    { cents: 0, source: 'fixed', coefficient: 1 });
  assert.equal(priceOf(null, null, rules, null, 100_000_000_000).cents, 100_000_000_000);
});

test('malformed imported pricing settings cannot create negative estimates', () => {
  assert.equal(priceOf(30, 40, { ...rules, rateCm2Cents: -16 }, null, null).cents, null);
  assert.equal(priceOf(30, 40, { ...rules, artistCoefficient: -1 }, null, null).cents, null);
  assert.equal(priceOf(30, 40, rules, null, null).cents, 19_200,
    'valid prices are unaffected by the safety checks');
});


test('imported price parameters match native finance validation', () => {
  for (const rateCm2Cents of [-1, 0.5, 100_001, Number.MAX_SAFE_INTEGER, NaN, Infinity]) {
    assert.equal(priceOf(30, 40, { ...rules, rateCm2Cents }, null, null).cents, null);
  }
  for (const coefficient of [0, -1, 100_000.1, NaN, Infinity, -Infinity]) {
    assert.equal(priceOf(30, 40, { ...rules, artistCoefficient: coefficient }, null, null).cents, null);
    assert.equal(priceOf(30, 40, rules, coefficient, null).cents, null);
  }
  assert.equal(priceOf(30, 40, { ...rules, artistCoefficient: NaN }, 1.5, null).cents, 28_800);
  assert.equal(priceOf(30, 40, { ...rules, rateCm2Cents: 0 }, null, null).cents, 0);
  assert.equal(priceOf(null, null, { rateCm2Cents: -1, artistCoefficient: NaN }, null, 0).cents, 0);
  assert.equal(priceOf(30, 40, { ...rules, rateCm2Cents: -16, artistCoefficient: -1 }, null, null).cents, null);
});

test('cleared or invalid fixed price stays unavailable in the editor', () => {
  for (const value of [null, -1, 1.5, Infinity, NaN, 100_000_000_001]) {
    const p = draftPriceFor('fixed', 30, 40, rules, 1.5, value);
    assert.equal(p.cents, null);
    assert.equal(p.source, 'fixed');
    assert.ok('error' in priceFieldsFor('fixed', 1.5, value));
  }
});

test('cleared or invalid own coefficient stays unavailable in the editor', () => {
  for (const value of [null, 0, -1, Infinity, NaN, 100_001]) {
    const p = draftPriceFor('coefficient', 30, 40, rules, value, 500);
    assert.equal(p.cents, null);
    assert.equal(p.source, 'own');
    assert.ok('error' in priceFieldsFor('coefficient', value, 500));
  }
});

test('valid editor modes ignore inactive fields and keep zero fixed prices', () => {
  const settings = { ...rules, artistCoefficient: 2 };
  assert.equal(draftPriceFor('auto', 30, 40, settings, 1.5, 500).cents, 38_400);
  assert.equal(draftPriceFor('coefficient', 30, 40, settings, 1.5, 500).cents, 28_800);
  assert.equal(draftPriceFor('fixed', null, null, settings, 1.5, 0).cents, 0);
  assert.equal(draftPriceFor('fixed', 30, 40, settings, 1.5, 100_000_000_000).cents, 100_000_000_000);
  assert.match(get('src/app/app.component.ts'), /return draftPriceFor\(this\.priceMode, d\.width_cm, d\.height_cm, this\.priceRules, d\.artist_coefficient_override, d\.fixed_price_cents\)/);
});


test('euro input rejects invalid amounts before rounding into stored cents', () => {
  for (const value of [null, undefined, -0.001, -0.004, -1, NaN, Infinity, 1_000_000_000.001]) {
    assert.equal(fixedPriceCentsFromEuros(value), null, String(value));
  }
  assert.equal(fixedPriceCentsFromEuros(0), 0);
  assert.equal(fixedPriceCentsFromEuros(12.34), 1234);
  assert.equal(fixedPriceCentsFromEuros(12.346), 1235);
  assert.equal(fixedPriceCentsFromEuros(1_000_000_000), 100_000_000_000);
  assert.match(get('src/app/app.component.ts'), /this\.artworkDraft\.fixed_price_cents = fixedPriceCentsFromEuros\(value\)/);
});
