// In-memory progress for explicitly confirmed independent archive packages.
// No persistence, automatic retry, or inference of success from a lost response.
export type BatchEntry = {readonly sha256: string; readonly artworks: number};
export type BatchPlan = {readonly manifest_digest: string; readonly packages: readonly BatchEntry[]};
export type BatchPhase = 'review' | 'confirm' | 'committing' | 'complete' | 'stopped' | 'unknown';
export type BatchProgress = Readonly<{
  manifest: string; packages: readonly BatchEntry[]; next: number;
  imported: number; phase: BatchPhase;
}>;
const hash = (s: string) => /^[0-9a-f]{64}$/.test(s);
export function beginBatch(plan: BatchPlan): BatchProgress {
  if (!hash(plan.manifest_digest) || plan.packages.length < 1 || plan.packages.length > 1000 ||
      plan.packages.some(p => !hash(p.sha256) || !Number.isSafeInteger(p.artworks) || p.artworks < 1 || p.artworks > 1000) ||
      plan.packages.reduce((sum,p) => sum+p.artworks,0) > 1000) throw Error('batch_manifest');
  return {manifest: plan.manifest_digest, packages: plan.packages.map(p => ({sha256:p.sha256, artworks:p.artworks})), next:0, imported:0, phase:'review'};
}
export function acceptBatchReview(state: BatchProgress, index: number, digest: string): BatchProgress {
  if (state.phase !== 'review' || index !== state.next || digest !== state.packages[index]?.sha256) throw Error('batch_changed');
  return {...state, phase:'confirm'};
}
export function confirmBatch(state: BatchProgress): BatchProgress {
  if (state.phase !== 'confirm') throw Error('batch_phase');
  return {...state, phase:'committing'};
}
export function settleBatch(state: BatchProgress, result: {kind:'committed'; count:number} | {kind:'rejected'} | {kind:'unknown'}): BatchProgress {
  if (state.phase !== 'committing') throw Error('batch_phase');
  if (result.kind === 'rejected') return {...state, phase:'stopped'};
  if (result.kind === 'unknown' || result.count !== state.packages[state.next]?.artworks) return {...state, phase:'unknown'};
  const next = state.next+1;
  return {...state, next, imported:state.imported+result.count, phase:next===state.packages.length?'complete':'review'};
}
export function stopBatch(state: BatchProgress): BatchProgress {
  if (state.phase !== 'review' && state.phase !== 'confirm') throw Error('batch_phase');
  return {...state, phase:'stopped'};
}
