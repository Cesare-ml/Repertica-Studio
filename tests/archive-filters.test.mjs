import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildSaleIndexes, filterArtworks, filterCollectors } from '../src/app/archive-filters.ts';

const artwork = (id, overrides = {}) => ({
  id, code: id, title: id, status: 'available', year: null, technique: null,
  support: null, location: null, description: null, ...overrides
});
const collector = (id, overrides = {}) => ({
  id, name: id, email: null, phone: null, address: null, notes: null, ...overrides
});
const sale = (artworkId, collectorId, title, collectorName) => ({
  artwork_id: artworkId, collector_id: collectorId, artwork_title: title,
  collector_name: collectorName
});
const options = (overrides = {}) => ({
  query: '', status: 'all', collectorId: 'all', order: 'recent', ...overrides
});

test('filter artworks by status and actual recorded collector relationship', () => {
  const artworks = [
    artwork('a', { title: 'Alba', status: 'sold' }),
    artwork('b', { title: 'Bosco', status: 'sold' }),
    artwork('c', { title: 'Cielo' })
  ];
  const indexes = buildSaleIndexes([
    sale('a', 'buyer-1', 'Alba', 'Anna Rossi'),
    sale('b', 'buyer-2', 'Bosco', 'Bruno')
  ]);
  assert.deepEqual(filterArtworks(artworks, options({ status: 'sold', collectorId: 'buyer-1' }), indexes)
    .map(a => a.id), ['a']);
  assert.deepEqual(filterArtworks(artworks, options({ collectorId: 'missing' }), indexes), []);
  assert.deepEqual(filterArtworks(artworks, options({ status: 'available' }), indexes)
    .map(a => a.id), ['c']);
});

test('text search includes metadata and related collector, case-insensitively', () => {
  const artworks = [
    artwork('a', { title: 'Ritratto', technique: 'Olio su tela' }),
    artwork('b', { title: 'Paesaggio', support: 'Carta', location: 'MILANO', status: 'sold' })
  ];
  const indexes = buildSaleIndexes([sale('b', 'c-1', 'Paesaggio', 'Elena Verdi')]);
  // The location field is no longer part of the interface, so it is not searched.
  for (const term of ['carta', 'ELENA']) {
    assert.deepEqual(filterArtworks(artworks, options({ query: term }), indexes)
      .map(a => a.id), ['b']);
  }
  assert.deepEqual(filterArtworks(artworks, options({ query: 'OLIO' }), indexes)
    .map(a => a.id), ['a']);
  assert.deepEqual(filterArtworks(artworks, options({ query: 'milano' }), indexes), []);
});

test('title/year sorting is stable for missing years and does not mutate source array', () => {
  const artworks = [
    artwork('a', { title: 'Studio 10', year: null }),
    artwork('b', { title: 'Studio 2', year: '2017' }),
    artwork('c', { title: 'Paesaggio', year: '2024' }),
    artwork('d', { title: 'Antico', year: 'circa 1900' })
  ];
  const indexes = buildSaleIndexes([]);
  assert.deepEqual(filterArtworks(artworks, options({ order: 'title' }), indexes).map(a => a.id),
    ['d', 'c', 'b', 'a']);
  assert.deepEqual(filterArtworks(artworks, options({ order: 'year' }), indexes).map(a => a.id),
    ['c', 'b', 'd', 'a']);
  assert.deepEqual(artworks.map(a => a.id), ['a', 'b', 'c', 'd']);
});

test('collector search includes contact fields and acquired artwork title', () => {
  const contacts = [
    collector('a', { name: 'Anna', notes: 'Cliente storico' }),
    collector('b', { name: 'Luca', address: 'Roma' })
  ];
  const indexes = buildSaleIndexes([sale('work-1', 'a', 'Luce marina', 'Anna')]);
  assert.deepEqual(filterCollectors(contacts, 'marina', indexes).map(c => c.id), ['a']);
  assert.deepEqual(filterCollectors(contacts, 'ROMA', indexes).map(c => c.id), ['b']);
  assert.deepEqual(filterCollectors(contacts, 'storico', indexes).map(c => c.id), ['a']);
  assert.equal(filterCollectors(contacts, '', indexes), contacts);
});

test('untrusted ids cannot pollute the index prototype', () => {
  const indexes = buildSaleIndexes([sale('artwork', '__proto__', 'Test', 'Buyer')]);
  assert.equal(Object.getPrototypeOf(indexes.byCollector), null);
  assert.equal(indexes.byCollector['__proto__'][0].artwork_id, 'artwork');
  assert.equal(Object.prototype.polluted, undefined);
});

test('artwork search ignores accents and requires every term across artwork and buyer fields', () => {
  const works = [
    artwork('a', { title: 'Città marina', technique: 'Olio su tela', status: 'sold' }),
    artwork('b', { title: 'Città notturna', technique: 'Tempera', status: 'sold' }),
    artwork('c', { title: 'Campagna', technique: 'Olio' })
  ];
  const indexes = buildSaleIndexes([
    sale('a', 'c-1', 'Città marina', 'Élodie D’Amico'),
    sale('b', 'c-2', 'Città notturna', 'Alessandro')
  ]);
  assert.deepEqual(filterArtworks(works, options({ query: '  ELODIE    teLa ' }), indexes)
    .map(a => a.id), ['a']);
  assert.deepEqual(filterArtworks(works, options({ query: 'citta olio' }), indexes)
    .map(a => a.id), ['a']);
  assert.deepEqual(filterArtworks(works, options({ query: 'citta acquerello' }), indexes), []);
  assert.deepEqual(filterArtworks(works, options({ query: '   ' }), indexes)
    .map(a => a.id), ['a', 'b', 'c']);
});

test('collector search combines acquired-work titles, name and address with accent folding', () => {
  const people = [
    collector('c1', { name: 'Élodie Rossi', address: 'Firenze' }),
    collector('c2', { name: 'Elena Verdi', address: 'Torino' })
  ];
  const indexes = buildSaleIndexes([sale('a', 'c1', 'Luce marina', 'Élodie Rossi')]);
  assert.deepEqual(filterCollectors(people, '  ELODIE    MARINA ', indexes).map(c => c.id), ['c1']);
  assert.deepEqual(filterCollectors(people, 'luce torino', indexes), []);
  assert.deepEqual(filterCollectors(people, 'élodie firenze', indexes).map(c => c.id), ['c1']);
  assert.equal(filterCollectors(people, '   ', indexes), people);
});

test('quoted artwork phrases match within one field and combine with independent terms', () => {
  const works = [
    artwork('a', { title: 'Città marina', technique: 'Olio su tela', status: 'sold' }),
    artwork('b', { title: 'Città di mare', description: 'marina', technique: 'Olio su tela' }),
    artwork('c', { title: 'Ritratto', technique: 'Olio', support: 'su tela' })
  ];
  const indexes = buildSaleIndexes([sale('a', 'buyer-1', 'Città marina', 'Élodie Rossi')]);
  const found = query => filterArtworks(works, options({ query }), indexes).map(a => a.id);
  assert.deepEqual(found('citta marina'), ['a', 'b']);
  assert.deepEqual(found('"CITTA MARINA"'), ['a']);
  assert.deepEqual(found('"citta   marina"'), ['a']);
  assert.deepEqual(found('"olio su tela" ÉLODIE'), ['a']);
  assert.deepEqual(found('"citta marina" "olio su tela"'), ['a']);
  assert.deepEqual(found('"olio su tela"'), ['a', 'b']);
  assert.deepEqual(found('"olio su tela" ritratto'), []);
  assert.deepEqual(found('"citta marina'), ['a']); // unfinished quote is still useful
  assert.deepEqual(found('""'), ['a', 'b', 'c']);
});

test('quoted collector searches can find acquired works but cannot span unrelated fields', () => {
  const people = [
    collector('c1', { name: 'Élodie Rossi', address: 'Firenze' }),
    collector('c2', { name: 'Anna Luce', notes: 'marina' })
  ];
  const indexes = buildSaleIndexes([sale('a', 'c1', 'Luce marina', 'Élodie Rossi')]);
  const found = query => filterCollectors(people, query, indexes).map(c => c.id);
  assert.deepEqual(found('luce marina'), ['c1', 'c2']);
  assert.deepEqual(found('"luce marina"'), ['c1']);
  assert.deepEqual(found('"LUCE MARINA" éLoDiE'), ['c1']);
  assert.deepEqual(found('"luce firenze"'), []);
  assert.deepEqual(found('luce firenze'), ['c1']);
});

test('collector filters separate buyers and incomplete records and search social profiles', () => {
  const collectors = [
    collector('a', { name: 'Anna', email: 'anna@example.org' }),
    collector('b', { name: 'Luca', phone: '333', instagram: '@luca_art' }),
    collector('c', { name: 'Chiara' })
  ];
  const indexes = buildSaleIndexes([sale('w1', 'a', 'Notte', 'Anna')]);
  const ids = list => list.map(c => c.id);
  assert.deepEqual(ids(filterCollectors(collectors, '', indexes)), ['a', 'b', 'c']);
  assert.deepEqual(ids(filterCollectors(collectors, '', indexes, 'buyers')), ['a']);
  assert.deepEqual(ids(filterCollectors(collectors, '', indexes, 'incomplete')), ['c']);
  assert.deepEqual(ids(filterCollectors(collectors, 'luca_art', indexes)), ['b']);
  assert.deepEqual(ids(filterCollectors(collectors, 'notte', indexes, 'incomplete')), []);
});


test('collector search matches phone numbers across formatting without weakening other terms', () => {
  const people = [
    collector('a', { name: 'Luca', phone: '+39 (333) 123-4567', notes: 'Opera 2026' }),
    collector('b', { name: 'Marta', phone: '555.000.9999' }),
    collector('c', { name: 'Carla', phone: null, notes: '3331234567' })
  ];
  const indexes = buildSaleIndexes([]);
  const found = query => filterCollectors(people, query, indexes).map(c => c.id);

  assert.deepEqual(found('3331234567'), ['a', 'c']);
  assert.deepEqual(found('"333 123 4567"'), ['a']);
  assert.deepEqual(found('Luca 333.123.4567'), ['a']);
  assert.deepEqual(found('5550009999'), ['b']);
  assert.deepEqual(found('Marta 3331234567'), []);
  assert.deepEqual(found('3331239999'), []);
  assert.deepEqual(found('2026'), ['a']); // No digit-only fuzzy matching for years.
});


test('typographic apostrophes, quotes and dashes search both artworks and collectors', () => {
  const works = [
    artwork('a', { title: 'L’Amico — ritratto', technique: 'Olio su tela' }),
    artwork('b', { title: 'Paesaggio–notturno', technique: 'Tempera' })
  ];
  const indexes = buildSaleIndexes([sale('a', 'c1', 'L’Amico — ritratto', 'D’Amico')]);
  const ids = query => filterArtworks(works, options({ query }), indexes).map(w => w.id);

  assert.deepEqual(ids("l'amico - ritratto"), ['a']);
  assert.deepEqual(ids('paesaggio-notturno'), ['b']);
  assert.deepEqual(ids('“olio su tela” d’amico'), ['a']);
  assert.deepEqual(ids('“olio su” ritratto'), ['a']);
  assert.deepEqual(ids('“olio su” notturno'), []);

  const people = [collector('c1', { name: 'D’Amico', notes: 'Collezione – privata' })];
  assert.deepEqual(filterCollectors(people, "d'amico", indexes).map(c => c.id), ['c1']);
  assert.deepEqual(filterCollectors(people, 'collezione - privata', indexes).map(c => c.id), ['c1']);
});

test('incomplete contacts exclude blank or whitespace-only email and phone values', () => {
  const people = [
    collector('a', { name: 'Senza contatti', email: '  ', phone: '\\t' }),
    collector('b', { name: 'Solo email', email: 'a@example.org', phone: ' ' }),
    collector('c', { name: 'Solo telefono', email: '\\n', phone: '333 123' }),
    collector('d', { name: 'Nessuno', email: null, phone: null })
  ];
  const found = filterCollectors(people, '', buildSaleIndexes([]), 'incomplete').map(c => c.id);
  assert.deepEqual(found, ['a', 'd']);
});
