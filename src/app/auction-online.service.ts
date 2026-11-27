/**
 * Shared state of the optional Auctions connection: sign-in status, selected artist profile, the operation
 * manager over the native transport, and the journal of unconfirmed publications.
 * Every figure comes from the server; nothing here computes leaders, minimums or results.
 */
import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { parseTenants, type ArtistTenant } from './auction-contract';
import { createArtistV1Transport } from './auction-http-transport';
import { createAuctionManager, type AuctionManager, type OperationOutcome } from './auction-manager';
import { createNativeConnection, type ConnectionStatus, type NativeConnection } from './auction-native';
import { preparePreview, publicArtworkFrom, type AuctionDraft } from './auction-preview';
import { dropPublication, loadPublications, newIdempotencyKey, recordPublication, type KeyValueStore,
  type PendingPublication } from './auction-publications';

const OFFLINE: ConnectionStatus = { available: false, signedIn: false, expiresInSeconds: null, development: false, signingIn: false };

@Injectable({ providedIn: 'root' })
export class AuctionOnlineService {
  private readonly native: NativeConnection = createNativeConnection((command, args) => invoke(command, args));
  readonly manager: AuctionManager;
  status: ConnectionStatus = OFFLINE;
  tenants: ArtistTenant[] = [];
  tenantId: string | null = null;
  publications: PendingPublication[] = [];

  constructor() {
    const transport = createArtistV1Transport(this.native.port, () => this.tenantId);
    this.manager = createAuctionManager(transport, () => typeof navigator === 'undefined' || navigator.onLine);
    const store = this.store;
    this.publications = store ? loadPublications(store) : [];
  }

  private get store(): KeyValueStore | null {
    try { return typeof localStorage === 'undefined' ? null : localStorage; } catch { return null; }
  }

  get selectedTenant(): ArtistTenant | null { return this.tenants.find(t => t.id === this.tenantId) ?? null; }
  get canPublish(): boolean { return this.status.signedIn && this.selectedTenant?.canPublish === true; }

  async refreshStatus(): Promise<ConnectionStatus> {
    this.status = await this.native.status();
    if (!this.status.signedIn) { this.tenants = []; this.tenantId = null; }
    return this.status;
  }

  async signIn(): Promise<void> {
    this.status = { ...this.status, signingIn: true };
    try { this.status = await this.native.signIn(); }
    finally { if (this.status.signingIn) this.status = await this.native.status(); }
    if (this.status.signedIn) await this.loadTenants();
  }

  async signOut(): Promise<void> {
    this.status = await this.native.signOut();
    this.tenants = []; this.tenantId = null;
  }

  async loadTenants(): Promise<ArtistTenant[]> {
    const response = await this.native.port({ method: 'GET', path: '/api/artist/tenants', headers: { Accept: 'application/json' } });
    if (response.status === 401) { await this.refreshStatus(); throw new Error('sign_in_required'); }
    if (response.status !== 200) throw new Error('service_unavailable');
    this.tenants = parseTenants(response.body);
    if (!this.tenants.some(t => t.id === this.tenantId)) this.tenantId = this.tenants[0]?.id ?? null;
    return this.tenants;
  }

  /**
   * Freezes the reviewed draft into the exact request, journals it, then sends it. The journal entry is kept
   * until the server list shows the auction, so a later retry (even after a restart) reuses key and body.
   */
  async publish(draft: AuctionDraft, title: string, artwork: Parameters<typeof publicArtworkFrom>[0] = null): Promise<OperationOutcome> {
    const tenantId = this.tenantId;
    if (!this.canPublish || !tenantId) throw new Error('tenant_not_selected');
    // Gestionale Opere: a main image is required. It is uploaded first (re-encoded natively, staged 24 hours);
    // the publication then names it, and a retry of the same publication reuses the same request.
    const upload = await this.native.uploadArtworkImage(draft.artworkId);
    if (upload.status === 401) { await this.refreshStatus(); throw new Error('sign_in_required'); }
    if (!upload.imageId) throw new Error(upload.error ?? 'image_upload_failed');
    const prepared = preparePreview(draft, tenantId, newIdempotencyKey(), Date.now(),
      { mainImageId: upload.imageId, artwork: publicArtworkFrom(artwork) });
    return this.send({ artworkId: draft.artworkId, tenantId, title, prepared, operation: null });
  }

  async retryPublication(entry: PendingPublication): Promise<OperationOutcome> {
    if (entry.tenantId !== this.tenantId) throw new Error('tenant_changed');
    // A refusal is final for this request: the artist corrects the draft and publishes anew.
    if (entry.operation.state === 'rejected') throw new Error('not_retryable');
    return this.send(entry);
  }

  discardPublication(entry: PendingPublication): void {
    const store = this.store;
    if (store) this.publications = dropPublication(store, entry.prepared.idempotencyKey);
  }

  private async send(entry: Omit<PendingPublication, 'operation'> & { operation: PendingPublication['operation'] | null }): Promise<OperationOutcome> {
    const store = this.store;
    if (!store) throw new Error('storage_unavailable');
    // Journal before sending: if the app stops now, the next attempt reuses the same identity.
    const journaled = { ...entry, operation: entry.operation ?? { id: `publish:${entry.prepared.idempotencyKey}`, kind: 'publish' as const,
      target: (JSON.parse(entry.prepared.body) as { auction: { id: string } }).auction.id, idempotencyKey: entry.prepared.idempotencyKey,
      body: entry.prepared.body, state: 'pending_confirmation' as const, reason: null, attempts: 0 } };
    this.publications = recordPublication(store, journaled);
    const outcome = await this.manager.publish(entry.prepared, journaled.operation.attempts > 0 ? journaled.operation : undefined);
    this.publications = recordPublication(store, { ...entry, operation: outcome.operation });
    if (outcome.needsSignIn) await this.refreshStatus();
    return outcome;
  }
}
