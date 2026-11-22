import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { LENS_SIZE, PANE_SIZE, zoomGeometry } from '../src/app/viewer-zoom.ts';

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
  assert.match(viewer, /api\.artworkImage\(/);
  assert.match(viewer, /api\.listArtworkGallery\(/);
  const html = get('src/app/app.component.html');
  assert.match(html, /<rp-artwork-viewer/);
  assert.match(html, /openViewer\(a\)/);
  assert.match(get('src/app/app.component.ts'), /ArtworkViewerComponent/);
});
