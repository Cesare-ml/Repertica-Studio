import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const iconIds = ['overview', 'artworks', 'collectors', 'sales', 'certificates', 'finance', 'settings'];

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
