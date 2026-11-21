import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const iconIds = ['overview', 'artworks', 'media', 'collectors', 'sales', 'certificates', 'brochures', 'finance', 'settings'];

test('sidebar uses valid, color-adaptive SVG icons for all sections', () => {
  const template = read('src/app/app.component.html');
  const styles = read('src/app/app.component.css');
  const config = JSON.parse(read('angular.json'));
  const assets = config.projects['repertica-studio'].architect.build.options.assets;

  assert.ok(template.includes(`[ngClass]="'nav-icon--' + item.id"`));
  assert.ok(!template.includes('{{ item.icon }}'));
  assert.ok(styles.includes('background-color:currentColor'));
  assert.ok(assets.some((asset) => asset.input === 'src/assets/icons' && asset.output === 'assets/icons'));

  for (const id of iconIds) {
    const svg = read(`src/assets/icons/${id}.svg`);
    assert.match(svg, /^<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg"/);
    assert.ok(svg.includes('viewBox="0 0 24 24"'), `${id}: expected 24px viewBox`);
    assert.ok(!/<script\b/i.test(svg), `${id}: unexpected script`);
    assert.ok(styles.includes(`.nav-icon--${id}{`), `${id}: missing navigation class`);
    assert.ok(styles.includes(`/assets/icons/${id}.svg`), `${id}: missing SVG asset URL`);
  }
});


const actionIds = ['export', 'edit', 'photo', 'gallery', 'delete', 'print', 'confirm', 'folder', 'copy', 'shield', 'restore'];
const reusedActionIds = ['artworks', 'collectors', 'sales', 'certificates', 'finance'];

test('contextual button icons are bundled, consistently styled, and decorative', () => {
  const template = read('src/app/app.component.html');
  const styles = read('src/app/app.component.css');
  const assetIds = [...template.matchAll(/action-glyph--([a-z]+)/g)].map((match) => match[1]);

  assert.ok(assetIds.length >= 35, 'expected artwork, export, archive and settings actions to use SVG');
  assert.ok(styles.includes('button.action-glyph::before'), 'icons must not replace accessible button text');
  assert.ok(styles.includes('background:currentColor'), 'glyph must inherit the button text color');
  assert.ok(template.includes(`[attr.aria-current]="section === item.id ? 'page' : null"`));

  for (const id of new Set(assetIds)) {
    assert.ok([...actionIds, ...reusedActionIds].includes(id), `unrecognized glyph ${id}`);
    const path = actionIds.includes(id) ? `actions/${id}.svg` : `${id}.svg`;
    const icon = read(`src/assets/icons/${path}`);

    assert.match(icon, /^<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg"/);
    assert.match(icon, /viewBox="0 0 24 24"/);
    assert.ok(icon.includes('</svg>'), `${id}: unfinished SVG`);
    assert.ok(!/<script\b|<foreignObject\b|<image\b/i.test(icon), `${id}: unsafe external content`);
    assert.ok(styles.includes(`.action-glyph--${id}::before{`), `${id}: missing mask class`);
    assert.ok(styles.includes(`/assets/icons/${path}`), `${id}: missing bundled asset URL`);
  }

  for (const id of actionIds) {
    assert.ok(assetIds.includes(id), `${id}: SVG exists but has no UI use`);
  }
  assert.ok(styles.includes('nav button:focus-visible'), 'keyboard users need a visible navigation focus');
});
