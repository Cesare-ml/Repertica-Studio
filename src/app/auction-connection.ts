/** Optional native IPC only. No URL, token, key, network or persistence adapter. */
export type AuctionConnectionStatus = 'unknown' | 'disabled' | 'disconnected' | 'connected';
export interface DraftReference { artworkId: string; scope: string; revision: string }
export interface AuctionOrganization { tenantId: string; artistName: string | null; role: string;
  status: string; publicationEligible: boolean; publishedThisMonth: number }
export interface AuctionPreviewSummary { auctionId: string; slug: string; title: string;
  status: string; startsAt: string; endsAt: string }
export interface AuctionReview { confirmationId: string; publicBody: string; action: 'prepare' | 'reconcile' }
export interface AuctionReceipt { tenantId: string; auctionId: string; slug: string }
export type AuctionNativeRequest = { action: 'status' | 'disconnect' | 'organizations' | 'clearSelection' | 'cancelReview' }
  | { action: 'selectOrganization'; tenantId: string }
  | ({ action: 'reviewNew'; publicBody: string } & DraftReference)
  | ({ action: 'reviewPending' } & DraftReference)
  | { action: 'confirmPrepare' | 'confirmReconcile'; confirmationId: string };
export type AuctionNativePort = (request: AuctionNativeRequest) => Promise<unknown>;

function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('auction_invalid_response');
  return value as Record<string, unknown>;
}
function text(value: unknown): string {
  if (typeof value !== 'string' || !value) throw new Error('auction_invalid_response');
  return value;
}
function status(value: unknown): AuctionConnectionStatus {
  const s = object(value)['status'];
  if (s !== 'disabled' && s !== 'disconnected' && s !== 'connected') throw new Error('auction_invalid_response');
  return s;
}
function organization(value: unknown): AuctionOrganization {
  const v = object(value), count = v['publishedThisMonth'], eligible = v['publicationEligible'];
  const role = text(v['role']), state = text(v['status']);
  if (typeof count !== 'number' || !Number.isInteger(count) || count < 0 || count > 12 || typeof eligible !== 'boolean' ||
      !['owner','editor','viewer'].includes(role) || !['active','pending','suspended'].includes(state) ||
      (eligible && (state !== 'active' || role === 'viewer'))) throw new Error('auction_invalid_response');
  return { tenantId: text(v['tenantId']), artistName: v['artistName'] === null ? null : text(v['artistName']),
    role, status: state, publicationEligible: eligible, publishedThisMonth: count };
}
function preview(value: unknown): AuctionPreviewSummary {
  const v = object(value);
  return { auctionId: text(v['auctionId']), slug: text(v['slug']), title: text(v['title']), status: text(v['status']),
    startsAt: text(v['startsAt']), endsAt: text(v['endsAt']) };
}
function list<T>(value: unknown, decode: (item: unknown) => T): T[] {
  if (!Array.isArray(value) || value.length > 100) throw new Error('auction_invalid_response');
  return value.map(decode);
}

/** Serializes the optional UI's explicit actions. Status is read locally from native;
 * the distributed provider reports disabled. Errors never trigger automatic retries. */
export class AuctionConnection {
  state: AuctionConnectionStatus = 'unknown';
  organizations: readonly AuctionOrganization[] = [];
  selected: AuctionOrganization | null = null;
  previews: readonly AuctionPreviewSummary[] = [];
  review: AuctionReview | null = null;
  busy = false;
  private readonly call: AuctionNativePort;
  constructor(call: AuctionNativePort) { this.call = call; }

  private resetSelection(): void { this.selected = null; this.previews = []; this.review = null; }
  private reset(): void { this.organizations = []; this.resetSelection(); }
  private idle(): void { if (this.busy) throw new Error('auction_editor_busy'); }
  private connected(): void { this.idle(); if (this.state !== 'connected') throw new Error('auction_not_connected'); }
  private async run<T>(work: () => Promise<T>): Promise<T> {
    this.idle(); this.busy = true;
    try { return await work(); }
    catch (error) { this.review = null; this.state = 'unknown'; this.reset(); throw error; }
    finally { this.busy = false; }
  }

  async refreshStatus(): Promise<void> {
    this.idle(); this.reset(); this.state = 'unknown';
    await this.run(async () => { this.state = status(await this.call({action:'status'})); });
  }
  async disconnect(): Promise<void> {
    this.idle(); this.reset(); this.state = 'unknown';
    await this.run(async () => {
      const next = status(await this.call({action:'disconnect'}));
      if (next === 'connected') throw new Error('auction_invalid_response');
      this.state = next;
    });
  }
  async loadOrganizations(): Promise<void> {
    this.connected(); this.reset();
    await this.run(async () => { this.organizations = list(await this.call({action:'organizations'}), organization); });
  }
  async selectOrganization(tenantId: string): Promise<void> {
    this.connected(); this.resetSelection();
    if (!this.organizations.some(o => o.tenantId === tenantId && o.status === 'active')) throw new Error('auction_access_denied');
    await this.run(async () => {
      const data = object(await this.call({action:'selectOrganization',tenantId}));
      const selected = organization(data['organization']);
      if (selected.tenantId !== tenantId || selected.status !== 'active') throw new Error('auction_invalid_response');
      const previews = list(data['previews'], preview);
      this.selected = selected; this.previews = previews;
    });
  }
  async cancelReview(): Promise<void> {
    this.idle(); const hadReview = this.review !== null; this.review = null;
    if (hadReview && this.state === 'connected') await this.run(async () => { await this.call({action:'cancelReview'}); });
  }
  async reviewDraft(ref: DraftReference, publicBody: string, pending: boolean): Promise<void> {
    this.connected(); this.review = null;
    if (!this.selected?.publicationEligible || object(JSON.parse(publicBody))['tenantId'] !== this.selected.tenantId) {
      throw new Error('auction_access_denied');
    }
    const {artworkId,scope,revision} = ref;
    const action = pending ? 'reconcile' : 'prepare';
    await this.run(async () => {
      const data = object(await this.call(pending ? {action:'reviewPending',artworkId,scope,revision}
        : {action:'reviewNew',artworkId,scope,revision,publicBody}));
      if (data['publicBody'] !== publicBody || data['action'] !== action) throw new Error('auction_invalid_response');
      this.review = { confirmationId:text(data['confirmationId']), publicBody, action };
    });
  }
  async confirm(): Promise<{action:'prepare'; revision:string} | {action:'reconcile'; receipt:AuctionReceipt}> {
    this.connected(); const review = this.review; this.review = null;
    if (!review || !this.selected?.publicationEligible) throw new Error('auction_review_required');
    const tenantId = this.selected.tenantId;
    return this.run(async () => {
      const value = object(await this.call({action:review.action === 'prepare' ? 'confirmPrepare' : 'confirmReconcile', confirmationId:review.confirmationId}));
      if (review.action === 'prepare') return {action:'prepare',revision:text(value['revision'])};
      if (value['tenantId'] !== tenantId) throw new Error('auction_invalid_response');
      return {action:'reconcile',receipt:{tenantId,auctionId:text(value['auctionId']),slug:text(value['slug'])}};
    });
  }
}
