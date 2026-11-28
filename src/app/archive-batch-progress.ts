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

// Transport is native IPC supplied by the desktop host; never accepts a path
// from a manifest. Preview and commit always use the explicitly chosen folder.
export interface BatchTransport<P extends {digest: string; artworks: readonly unknown[]}> {
  preflight(directory: string): Promise<BatchPlan>;
  review(directory: string, expectedManifest: string, index: number): Promise<P>;
  commit(directory: string, expectedManifest: string, index: number, expectedPreview: string): Promise<number>;
}
export class BatchImportSession<P extends {digest: string; artworks: readonly unknown[]}> {
  private directory = '';
  private progress: BatchProgress | null = null;
  private candidate: P | null = null;
  private pending = false;
  private readonly transport: BatchTransport<P>;
  constructor(transport: BatchTransport<P>) { this.transport = transport; }
  get busy(): boolean { return this.pending; }
  get state(): BatchProgress | null { return structuredClone(this.progress); }
  get preview(): P | null { return structuredClone(this.candidate); }
  private idle(): void { if (this.pending) throw Error('batch_busy'); }
  async start(directory: string): Promise<void> {
    this.idle();
    if (this.progress || !directory) throw Error('batch_phase');
    this.pending = true;
    try {
      this.progress = beginBatch(await this.transport.preflight(directory));
      this.directory = directory;
    } finally { this.pending = false; }
  }
  async reviewNext(): Promise<void> {
    this.idle();
    const state = this.progress;
    if (!state || state.phase !== 'review') throw Error('batch_phase');
    this.pending = true;
    this.candidate = null;
    try {
      const preview = structuredClone(await this.transport.review(this.directory,state.manifest,state.next));
      if (preview.artworks.length !== state.packages[state.next].artworks) throw Error('batch_changed');
      const next = acceptBatchReview(state,state.next,preview.digest);
      this.candidate = preview;
      this.progress = next;
    } finally { this.pending = false; }
  }
  async commitReviewed(): Promise<void> {
    this.idle();
    const state = this.progress;
    if (!state || !this.candidate) throw Error('batch_phase');
    const committing = confirmBatch(state);
    const digest = this.candidate.digest;
    this.pending = true;
    this.progress = committing;
    this.candidate = null;
    try {
      const count = await this.transport.commit(this.directory,state.manifest,state.next,digest);
      this.progress = settleBatch(committing,{kind:'committed',count});
    } catch {
      // IPC rejection can mean a lost response after a successful write.
      // Never infer rollback, never retry and never count unconfirmed rows.
      this.progress = settleBatch(committing,{kind:'unknown'});
    } finally { this.pending = false; }
  }
  stop(): void {
    this.idle();
    if (!this.progress) throw Error('batch_phase');
    this.progress = stopBatch(this.progress);
    this.candidate = null;
  }
  reset(archiveManuallyChecked = false): void {
    this.idle();
    const phase = this.progress?.phase;
    if (phase && phase !== 'complete' && phase !== 'stopped' && !(phase === 'unknown' && archiveManuallyChecked)) {
      throw Error('batch_phase');
    }
    this.progress = null;
    this.candidate = null;
    this.directory = '';
  }
}
