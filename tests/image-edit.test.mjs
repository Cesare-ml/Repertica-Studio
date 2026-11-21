import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  adjustmentFilter, adjustmentTransform, buildOperations, cropChanged, cropForRatio, defaultCrop, emptyEditState,
  moveCropCorner, perspectiveChanged, perspectiveRatio
} from '../src/app/image-edit.ts';

test('an untouched editor has nothing to apply', () => {
  const result = buildOperations(emptyEditState());
  assert.equal(result.ok, false);
  assert.match(result.error, /Nessuna modifica/);
});

test('crop, rotation, mirroring and adjustments become native operations in the right order', () => {
  const state = emptyEditState();
  state.crop = { x: 0.2, y: 0.1, width: 0.5, height: 0.6 };
  state.rotation = 90;
  state.flipHorizontal = true;
  state.exposure = 50; state.contrast = -20; state.saturation = 10;
  const result = buildOperations(state);
  assert.equal(result.ok, true);
  assert.deepEqual(result.operations.map(o => o.type), ['crop', 'rotate', 'flip', 'adjust']);
  assert.deepEqual(result.operations[0], { type: 'crop', x: 0.2, y: 0.1, width: 0.5, height: 0.6 });
  assert.deepEqual(result.operations[3], { type: 'adjust', exposure: 0.5, contrast: -0.2, saturation: 0.1 });
});

test('a perspective correction replaces the crop, as in the Gestionale', () => {
  const state = emptyEditState();
  state.crop = { x: 0.3, y: 0.3, width: 0.3, height: 0.3 };
  state.perspective = [{ x: 0.05, y: 0.12 }, { x: 0.93, y: 0.08 }, { x: 0.9, y: 0.9 }, { x: 0.1, y: 0.95 }];
  const result = buildOperations(state);
  assert.deepEqual(result.operations.map(o => o.type), ['perspective']);
  assert.equal(result.operations[0].outputAspectRatio, null);
  state.perspectiveMode = '3:2';
  assert.equal(buildOperations(state).operations[0].outputAspectRatio, 1.5);
});

test('a fixed output ratio alone already requests the perspective step, custom measures must be complete', () => {
  const state = emptyEditState();
  state.perspectiveMode = '16:9';
  assert.deepEqual(buildOperations(state).operations.map(o => o.type), ['perspective']);
  state.perspectiveMode = 'custom';
  assert.equal(buildOperations(state).ok, false);
  state.customWidth = 60; state.customHeight = 40;
  assert.equal(perspectiveRatio(state), 1.5);
  assert.equal(buildOperations(state).ok, true);
});

test('crop presets fit the largest centred box inside the picture', () => {
  const wide = cropForRatio(1, 200, 100);   // square inside a 2:1 picture
  assert.deepEqual([wide.x, wide.y, wide.width, wide.height], [0.25, 0, 0.5, 1]);
  const tall = cropForRatio(16 / 9, 100, 200);
  assert.ok(Math.abs(tall.width - 1) < 1e-9 && tall.height < 1 && Math.abs(tall.y - (1 - tall.height) / 2) < 1e-9);
  assert.deepEqual(cropForRatio(0, 100, 100), defaultCrop());
});

test('dragging a crop corner keeps a minimum size and stays inside the image', () => {
  const box = { x: 0.2, y: 0.2, width: 0.5, height: 0.5 };
  assert.deepEqual(moveCropCorner(box, 2, { x: 0.9, y: 0.95 }), { x: 0.2, y: 0.2, width: 0.7, height: 0.75 });
  const crushed = moveCropCorner(box, 0, { x: 5, y: 5 });
  assert.ok(crushed.width >= 0.03 - 1e-9 && crushed.height >= 0.03 - 1e-9 && crushed.x + crushed.width <= 0.7 + 1e-9);
  const outside = moveCropCorner(box, 2, { x: 4, y: -2 });
  assert.ok(outside.x + outside.width <= 1 + 1e-9 && outside.height >= 0.03 - 1e-9);
  assert.equal(cropChanged(box), true);
  assert.equal(cropChanged(defaultCrop()), false);
  assert.equal(perspectiveChanged(emptyEditState().perspective), false);
});

test('live previews use CSS filters that match the sliders', () => {
  const state = emptyEditState();
  assert.equal(adjustmentFilter(state), 'brightness(1.000) contrast(1.000) saturate(1.000)');
  state.exposure = 100; state.contrast = 50; state.saturation = -100;
  assert.equal(adjustmentFilter(state), 'brightness(2.000) contrast(1.500) saturate(0.000)');
  state.rotation = 90; state.flipVertical = true;
  assert.equal(adjustmentTransform(state), 'rotate(90deg) scale(1, -1)');
});
