import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/app/app.component.ts', import.meta.url), 'utf8');
function handler(name, end, open, api, confirmed) {
  const start = source.indexOf('  async ' + name + '()');
  const stop = source.indexOf(end, start);
  assert.ok(start >= 0 && stop > start);
  const method = source.slice(start, stop).replace('(): Promise<void>', '()').trim();
  return new Function('open', 'api', 'confirmed', 'return ({' + method + '}).' + name)(open, api, confirmed);
}
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return {promise, resolve, reject};
}
for (const scenario of ['cancel', 'picker-error', 'inspect-error', 'success', 'double-picker', 'restore-during-picker', 'restore-during-inspect', 'already-busy', 'not-initialized']) {
  test('restore folder lock: ' + scenario, async () => {
    const picker = deferred(), inspection = deferred();
    const original = {directory:'backup-old'}, candidate = {directory:'backup-new'};
    const ctx = {busy:scenario === 'already-busy', initialized:scenario !== 'not-initialized',
      restoreCandidate:original, errorMessage:'old error', error:String, change:{detectChanges() {}}};
    let picks = 0, prompts = 0;
    const inspected = [];
    const api = {inspectBackup(path) {inspected.push(path); return inspection.promise;}};
    const choose = handler('chooseRestoreFolder', '\n  async restoreSelectedBackup()', () => {picks++; return picker.promise;}, api);
    const restore = handler('restoreSelectedBackup', '\n  bytes(', null, api, async () => {prompts++; return false;});
    const pending = choose.call(ctx);
    if (scenario === 'already-busy' || scenario === 'not-initialized') {
      await pending;
      assert.equal(picks, 0);
      assert.equal(ctx.busy, scenario === 'already-busy');
      assert.equal(ctx.restoreCandidate, original);
      return;
    }
    assert.equal(ctx.busy, true, 'lock before native picker');
    assert.equal(picks, 1);
    if (scenario === 'double-picker') {await choose.call(ctx); assert.equal(picks, 1);}
    if (scenario === 'restore-during-picker') {await restore.call(ctx); assert.equal(prompts, 0);}
    if (scenario === 'picker-error') picker.reject(new Error('picker failed'));
    else if (scenario === 'cancel') picker.resolve(null);
    else {
      picker.resolve('backup-new');
      await Promise.resolve();
      assert.deepEqual(inspected, ['backup-new']);
      assert.equal(ctx.busy, true, 'lock through inspection');
      if (scenario === 'restore-during-inspect') {await restore.call(ctx); assert.equal(prompts, 0);}
      if (scenario === 'inspect-error') inspection.reject(new Error('inspection failed'));
      else inspection.resolve(candidate);
    }
    await pending;
    assert.equal(ctx.busy, false, 'release lock on every exit');
    assert.equal(ctx.restoreCandidate, scenario === 'cancel' ? original :
      ['picker-error','inspect-error'].includes(scenario) ? null : candidate);
    if (scenario === 'cancel') assert.equal(ctx.errorMessage, 'old error');
    if (scenario.endsWith('error')) assert.ok(ctx.errorMessage.includes('failed'));
  });
}
