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
  // Artwork location remains searchable alongside technique and buyer.
  for (const term of ['carta', 'ELENA', 'milano']) {
    assert.deepEqual(filterArtworks(artworks, options({ query: term }), indexes)
      .map(a => a.id), ['b']);
  }
  assert.deepEqual(filterArtworks(artworks, options({ query: 'OLIO' }), indexes)
    .map(a => a.id), ['a']);
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
    collector('a', { name: 'Senza contatti', email: '  ', phone: '\t' }),
    collector('b', { name: 'Solo email', email: 'a@example.org', phone: ' ' }),
    collector('c', { name: 'Solo telefono', email: '\n', phone: '333 123' }),
    collector('d', { name: 'Nessuno', email: null, phone: null })
  ];
  const found = filterCollectors(people, '', buildSaleIndexes([]), 'incomplete').map(c => c.id);
  assert.deepEqual(found, ['a', 'd']);
});


test('artwork dimensions are searchable in cm with x or × and decimal commas', () => {
  const works = [
    artwork('a', { title: 'Città marina', width_cm: 30.5, height_cm: 40.25, depth_cm: 2 }),
    artwork('b', { title: 'Verticale', width_cm: 40.25, height_cm: 30.5 }),
    artwork('c', { title: 'Orizzonte', width_cm: 30, height_cm: 40 }),
    artwork('d', { title: 'Senza misure', width_cm: null, height_cm: null, depth_cm: null })
  ];
  const indexes = buildSaleIndexes([sale('a', 'buyer', 'Città marina', 'Élodie Rossi')]);
  const found = query => filterArtworks(works, options({ query }), indexes).map(a => a.id);

  assert.deepEqual(found('30,5 × 40,25'), ['a']);
  assert.deepEqual(found('30.5 x 40.25'), ['a']);
  assert.deepEqual(found('30,5 × 40,25 cm'), ['a']);
  assert.deepEqual(found('30,5 X 40,25 X 2 cm'), ['a']);
  assert.deepEqual(found('"30,5 × 40,25" ELODIE'), ['a']);
  assert.deepEqual(found('40,25x30,5'), ['b']); // Width and height keep their order.
  assert.deepEqual(found('30 × 40'), ['c']);
  assert.deepEqual(found('30x50'), []);
  assert.deepEqual(found('30,5'), ['a', 'b']); // Single dimensions remain searchable.
  assert.deepEqual(found('0x0'), []); // Missing measurements are never fabricated.
  assert.deepEqual(found('"senza misure"'), ['d']);
});


test('collector search uses the archive codes of purchased artworks, never unrelated works', () => {
  const works = [
    artwork('one', { code: 'ART-0042', title: 'Natura' }),
    artwork('two', { code: 'ML-2026-014', title: 'Natura' }),
    artwork('unsold', { code: 'UNSOLD-0001', title: 'Natura' })
  ];
  const people = [
    collector('c1', { name: 'Anna Rossi' }),
    collector('c2', { name: 'Bruno Verdi' }),
    collector('c3', { name: 'Carla' })
  ];
  const indexes = buildSaleIndexes([
    sale('one', 'c1', 'Natura', 'Anna Rossi'),
    sale('two', 'c2', 'Natura', 'Bruno Verdi')
  ]);
  const found = query => filterCollectors(people, query, indexes, 'all', works).map(c => c.id);

  assert.deepEqual(found('art-0042'), ['c1']);
  assert.deepEqual(found('"ML-2026-014"'), ['c2']);
  assert.deepEqual(found('art-0042 anna'), ['c1']);
  assert.deepEqual(found('art-0042 bruno'), []);
  assert.deepEqual(found('UNSOLD-0001'), []);
  assert.deepEqual(filterCollectors(people, 'ART-0042', indexes), []); // Existing callers remain compatible.
});

test('artwork location search combines phrases with status, collector and metadata', () => {
  const works = [
    artwork('a', { title: 'Marea', location: 'Studio di Venezia' }),
    artwork('b', { title: 'Marea', location: 'Deposito a Venezia', status: 'sold' }),
    artwork('c', { title: 'Cielo', location: null })
  ];
  const indexes = buildSaleIndexes([sale('b', 'buyer', 'Marea', 'Élodie')]);
  const found = (query, extra = {}) =>
    filterArtworks(works, options({ query, ...extra }), indexes).map(a => a.id);

  assert.deepEqual(found('venezia'), ['a', 'b']);
  assert.deepEqual(found('"studio di venezia"'), ['a']);
  assert.deepEqual(found('VENEZIA ÉLODIE'), ['b']);
  assert.deepEqual(found('venezia', { status: 'sold' }), ['b']);
  assert.deepEqual(found('venezia', { collectorId: 'buyer' }), ['b']);
  assert.deepEqual(found('deposito marea'), ['b']);
  assert.deepEqual(found('padova'), []);
});


test('artwork search matches European ligatures in titles, locations and buyer names', () => {
  const works = [
    artwork('a', { title: 'Œuvre de Dvořák', location: 'Łódź', status: 'sold' }),
    artwork('b', { title: 'Straße di Sølv', description: 'Æther' }),
    artwork('c', { title: '日本語 · 光' })
  ];
  const indexes = buildSaleIndexes([sale('a', 'buyer', 'Œuvre de Dvořák', 'Þór Ægir')]);
  const found = query => filterArtworks(works, options({ query }), indexes).map(a => a.id);

  assert.deepEqual(found('oeuvre dvorak'), ['a']);
  assert.deepEqual(found('"oeuvre de dvorak" lodz'), ['a']);
  assert.deepEqual(found('thor aegir'), ['a']);
  assert.deepEqual(found('STRASSE SOLV'), ['b']);
  assert.deepEqual(found('aether'), ['b']);
  assert.deepEqual(found('日本語'), ['c'], 'unrelated scripts are left intact');
  assert.deepEqual(found('strasse dvorak'), [], 'AND terms cannot combine unrelated works');
});

test('collector search folds European names and only the artworks they purchased', () => {
  const people = [
    collector('buyer', { name: 'Søren Straße', address: 'Łódź', notes: 'Æther' }),
    collector('other', { name: 'Þór Ægir', notes: '日本語' })
  ];
  const works = [
    artwork('w1', { code: 'ART-0042', title: 'Œuvre de Dvořák' }),
    artwork('w2', { code: 'UNSOLD-0003', title: 'Straße' })
  ];
  const indexes = buildSaleIndexes([sale('w1', 'buyer', 'Œuvre de Dvořák', 'Søren Straße')]);
  const found = query => filterCollectors(people, query, indexes, 'all', works).map(c => c.id);

  assert.deepEqual(found('soren strasse lodz'), ['buyer']);
  assert.deepEqual(found('"oeuvre de dvorak" aether'), ['buyer']);
  assert.deepEqual(found('thor aegir'), ['other']);
  assert.deepEqual(found('日本語'), ['other']);
  assert.deepEqual(found('unsold-0003'), [], 'unrelated artwork codes stay private to their own records');
  assert.deepEqual(found('lodz thor'), [], 'terms cannot cross two collectors');
});

test('collector lookup uses current metadata only from genuinely purchased works', () => {
  const works = [
    artwork('owned', { code: 'ART-101', title: 'Fiume blu', technique: 'Acquerello',
      support: 'Carta Fabriano', year: '2023', location: 'Atelier Venezia',
      width_cm: 30, height_cm: 40 }),
    artwork('unowned', { code: 'ART-999', title: 'Ritratto segreto',
      technique: 'Tempera', location: 'Napoli' }),
    artwork('other-owned', { code: 'ART-102', title: 'Mare verde',
      technique: 'Olio', location: 'Firenze' })
  ];
  const people = [collector('a', { name: 'Anna' }), collector('b', { name: 'Bruno' })];
  const indexes = buildSaleIndexes([
    sale('owned', 'a', 'Vecchio titolo', 'Anna'),
    sale('other-owned', 'b', 'Mare verde', 'Bruno')
  ]);
  const found = query => filterCollectors(people, query, indexes, 'all', works).map(c => c.id);

  assert.deepEqual(found('fiume blu'), ['a']); // Current artwork title, not just sale text.
  assert.deepEqual(found('acquerello fabriano'), ['a']);
  assert.deepEqual(found('"atelier venezia" 2023'), ['a']);
  assert.deepEqual(found('30 x 40 cm'), ['a']);
  assert.deepEqual(found('art-101 anna'), ['a']);
  assert.deepEqual(found('firenze olio'), ['b']);
  assert.deepEqual(found('tempera'), []); // Unsold artwork cannot create a match.
  assert.deepEqual(found('napoli'), []);
  assert.deepEqual(found('acquerello bruno'), []); // No cross-buyer metadata.
  assert.deepEqual(found('vecchio titolo'), ['a']); // Historical sale name still works.
  assert.deepEqual(filterCollectors(people, 'vecchio titolo', indexes).map(c => c.id), ['a']);
});
