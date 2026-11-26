import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { LENS_SIZE, PANE_SIZE, zoomGeometry } from '../src/app/viewer-zoom.ts';
import { loadViewerImages } from '../src/app/viewer-image-loader.ts';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('the magnifier follows the pointer and stays inside the picture', () => {
  const inside = zoomGeometry(100, 50, 400, 200, 4000, 2000);
  assert.equal(inside.lens.left, `${100 - LENS_SIZE / 2}px`);
  assert.equal(inside.lens.top, `${50 - LENS_SIZE / 2}px`);
  // The side pane is at least 4 times the displayed size, and at least 1.6 times the real size.
  assert.equal(inside.pane.backgroundSize, `${4000 * 1.6}px ${2000 * 1.6}px`);
  const outside = zoomGeometry(-50, 999, 400, 200, 400, 200);
  assert.equal(outside.lens.left, `${0 - LENS_SIZE / 2}px`, 'x is clamped to the picture');
  assert.equal(outside.lens.top, `${200 - LENS_SIZE / 2}px`, 'y is clamped to the picture');
  const small = zoomGeometry(0, 0, 400, 200, 100, 50);
  assert.equal(small.pane.backgroundSize, '1600px 800px');
  assert.equal(small.pane.backgroundPosition, `${PANE_SIZE / 2}px ${PANE_SIZE / 2}px`);
  assert.equal(zoomGeometry(1, 1, 0, 0, 10, 10), null, 'a picture that is not laid out yet gives no magnifier');
});

test('the archive opens artworks full screen and browses them with the keyboard', () => {
  const viewer = get('src/app/artwork-viewer.component.ts');
  for (const key of ['Escape', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown']) assert.ok(viewer.includes(key), key);
  assert.match(viewer, /aria-modal="true"/);
  assert.match(viewer, /window:keydown/);
  assert.match(viewer, /cover:\s*api\.artworkImage/);
  assert.match(viewer, /gallery:\s*api\.listArtworkGallery/);
  const html = get('src/app/app.component.html');
  assert.match(html, /<rp-artwork-viewer/);
  assert.match(html, /openViewer\(a\)/);
  assert.match(get('src/app/app.component.ts'), /ArtworkViewerComponent/);
});

const labels = { cover: 'Copertina', photograph: 'Fotografia', kind: code => code === 'DETAIL' ? 'Dettaglio' : null };
const reader = (overrides = {}) => ({
  cover: async () => 'data:cover',
  gallery: async () => [
    { id: 'broken', kind: 'PHOTO', caption: null },
    { id: 'healthy', kind: 'DETAIL', caption: 'Firma' },
  ],
  photo: async id => 'data:' + id,
  ...overrides,
});

test('a broken cover does not hide the remaining gallery photographs', async () => {
  const result = await loadViewerImages('work', true, reader({
    cover: async () => { throw Error('Cover file missing'); },
  }), labels, () => true);
  assert.deepEqual(result.images, [
    { url: 'data:broken', title: 'Fotografia', caption: null },
    { url: 'data:healthy', title: 'Dettaglio', caption: 'Firma' },
  ]);
  assert.match(result.error, /Cover file missing/);
  assert.equal(result.complete, false, 'incomplete results must not enter the viewer cache');
});

test('one corrupt gallery image does not prevent loading the others', async () => {
  const result = await loadViewerImages('work', true, reader({
    photo: async id => {
      if (id === 'broken') throw Error('Unreadable gallery file');
      return 'data:' + id;
    },
  }), labels, () => true);
  assert.deepEqual(result.images.map(image => image.url), ['data:cover', 'data:healthy']);
  assert.match(result.error, /Unreadable gallery file/);
  assert.equal(result.complete, false);
});

test('the cover remains visible if the gallery index is unavailable', async () => {
  const result = await loadViewerImages('work', true, reader({
    gallery: async () => { throw Error('SQLite unavailable'); },
  }), labels, () => true);
  assert.deepEqual(result.images.map(image => image.url), ['data:cover']);
  assert.match(result.error, /SQLite unavailable/);
  assert.equal(result.complete, false);
});

test('a healthy artwork with no cover is cacheable, and navigation cancels stale results', async () => {
  const good = await loadViewerImages('work', false, reader(), labels, () => true);
  assert.equal(good.complete, true);
  assert.deepEqual(good.images.map(image => image.url), ['data:broken', 'data:healthy']);

  let current = true;
  const interrupted = await loadViewerImages('work', true, reader({
    cover: async () => { current = false; return 'data:old-cover'; },
    gallery: async () => { throw Error('Should not query old gallery'); },
  }), labels, () => current);
  assert.deepEqual(interrupted, { images: [], error: '', complete: false });
});

test('a missing cover with an empty gallery is not silently cached as healthy', async () => {
  const result = await loadViewerImages('work', true, reader({
    cover: async () => null, gallery: async () => [],
  }), labels, () => true);
  assert.deepEqual(result.images, []);
  assert.equal(result.complete, false);
  assert.match(result.error, /Copertina non disponibile/);
});

test('a stale navigation never starts native reads and cancellation during a photo stops later reads', async () => {
  let calls = 0;
  const unused = reader({
    cover: async () => { calls++; return 'data:cover'; },
    gallery: async () => { calls++; return []; },
  });
  assert.deepEqual(await loadViewerImages('work', true, unused, labels, () => false), { images: [], error: '', complete: false });
  assert.equal(calls, 0);
  let current = true;
  const requested = [];
  const result = await loadViewerImages('work', false, reader({
    photo: async id => { requested.push(id); current = false; return 'data:' + id; },
  }), labels, () => current);
  assert.deepEqual(requested, ['broken']);
  assert.deepEqual(result, { images: [], error: '', complete: false });
});

test('an empty native rejection still marks the result incomplete', async () => {
  const result = await loadViewerImages('work', true, reader({
    cover: async () => { throw ''; },
  }), labels, () => true);
  assert.equal(result.complete, false);
  assert.ok(result.error);
  assert.equal(result.images.length, 2);
});

test('viewer invalidates stale image labels and exposes partial results accessibly', () => {
  const viewer = get('src/app/artwork-viewer.component.ts');
  assert.match(viewer, /if \(changes\['artworks'\] \|\| changes\['language'\]\) this\.cache\.clear\(\)/);
  assert.match(viewer, /role="status" \*ngIf="!loading && image && error"/);
  assert.match(viewer, /if \(!artwork\) \{ this\.images = \[\]; this\.loading = false;/);
});
