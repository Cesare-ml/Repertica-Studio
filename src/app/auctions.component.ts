import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { invoke } from '@tauri-apps/api/core';
import { previewLegacyDraft } from './auction-legacy-import';
import { AuctionConnection } from './auction-connection';
import { SettlementReader, settlementPresentation, type SettlementReadAction } from './auction-settlement-summary';
import { assertSameSnapshot, assertPreviewReadback } from './auction-online-readback';
import { AUCTION_API_CONNECTED, AUCTION_CONTRACT_VERSION } from './auction-contract';
import { NOT_SENT, centsToEuro, euroToCents, loadDrafts, localInputToUtc, slugify,
  utcToLocalInput, type KeyValueStore, type LocalAuctionDraft } from './auction-drafts';
import { newAuctionDraft, publicPreviewFields, reviewRows, decodeNativeDraft, saveNativeAuctionDraft, type AuctionDraft, type ReviewRow, type NativeDraftPort, type NativeDraftSnapshot } from './auction-preview';
import type { AppLanguage } from './i18n';
import { translate } from './i18n';
import type { Artwork } from './models';

interface DraftSummary {
  artworkId: string;
  title: string;
  artworkStatus: string | null;
  state: 'draft' | 'pending' | 'receipt';
}
interface DraftPage {
  scope: string;
  rows: DraftSummary[];
  nextAfter: string | null;
}

const nativeDrafts: NativeDraftPort = {
  read: artworkId => invoke<NativeDraftSnapshot>('read_auction_draft', { artworkId }),
  write: input => invoke<NativeDraftSnapshot>('write_auction_draft', { input }),
};

const ERRORS: Record<string, string> = {
  artwork_not_available: 'Solo un’opera disponibile può avere un’asta',
  artwork_not_selected: 'Scegli l’opera dell’asta',
  invalid_slug: 'L’indirizzo può contenere solo lettere minuscole, numeri e trattini',
  invalid_title: 'Il titolo deve avere almeno 3 caratteri',
  invalid_description: 'La descrizione è troppo lunga (massimo 1500 caratteri)',
  invalid_schedule: 'Date non valide: l’inizio deve essere futuro, la fine successiva e la durata al massimo 7 giorni',
  invalid_amount: 'Inserisci importi validi per prezzo di partenza e rilancio minimo',
  payload_too_large: 'I dati sono troppo lunghi',
};

/**
 * One offline-first Auctions workspace. The optional native preview bridge has no
 * renderer-supplied provider, URL, token or key. In release it is disabled; local
 * storage remains usable when the command is missing or unavailable.
 */
@Component({
  selector: 'rp-auctions',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
<div class="auctions">
  <div class="auctions-head">
    <h2>{{ tr('Aste') }}</h2>
    <p>{{ tr('Prepara l’asta di una singola opera. L’archivio resta sul computer.') }}</p>
  </div>

  <div class="auctions-banner" role="status" *ngIf="!connected">
    <strong>{{ tr('Non collegato ad Auctions') }}</strong>
    <span>{{ tr('Il servizio online non ha ancora un’interfaccia verificata per pubblicare e gestire le aste: nessun dato viene inviato e nessuna asta risulta pubblicata. Puoi preparare bozze locali.') }}</span>
  </div>

  <section class="auctions-card" aria-labelledby="auctions-published">
    <h3 id="auctions-published">{{ tr('Aste pubblicate') }}</h3>
    <p class="auctions-muted">{{ tr('Offerte, importi e risultati vengono letti dal server dopo il collegamento. Qui non vengono mai simulati.') }}</p>
  </section>

  <section class="auctions-card" aria-labelledby="auctions-online">
    <h3 id="auctions-online">{{ previewHeading }}</h3>
    <p role="status" class="auctions-muted">{{ onlineLabel(online.state) }}</p>
    <div class="auctions-row">
      <button type="button" (click)="checkConnection()" [disabled]="busy || !!online.review">{{ onlineLabel('check') }}</button>
      <button type="button" (click)="loadOrganizations()" [disabled]="busy || !!online.review || online.state !== 'connected'">{{ onlineLabel('organizations') }}</button>
      <button type="button" (click)="disconnectOnline()" [disabled]="busy || online.state !== 'connected'">{{ onlineLabel('disconnect') }}</button>
    </div>
    <select *ngIf="online.organizations.length" [disabled]="busy || !!online.review"
      [ngModel]="online.selected?.tenantId ?? ''" (ngModelChange)="chooseOrganization($event)"
      [attr.aria-label]="onlineLabel('organizations')">
      <option value="" disabled>—</option>
      <option *ngFor="let organization of online.organizations" [value]="organization.tenantId"
        [disabled]="organization.status !== 'active'">{{ organization.artistName ?? organization.tenantId }} · {{ organization.role }}</option>
    </select>
    <ul class="auctions-list" *ngIf="online.selected && online.previews.length">
      <li *ngFor="let preview of online.previews">
        <div><strong>{{ preview.title }}</strong><small>{{ preview.startsAt }} – {{ preview.endsAt }}</small></div>
        <span class="auctions-badge">{{ preview.status }}</span>
      </li>
    </ul>
    <p role="status" *ngIf="onlineNotice">{{ onlineNotice }}</p>
    <p role="alert" class="auctions-alert" *ngIf="onlineError">{{ onlineError }}</p>
  </section>

  <section class="auctions-card" aria-labelledby="auctions-drafts" *ngIf="!editing">
    <div class="auctions-row">
      <h3 id="auctions-drafts">{{ tr('Bozze locali') }}</h3>
      <button type="button" (click)="reloadList()" [disabled]="busy">{{ tr('Ricarica bozze') }}</button>
      <select [(ngModel)]="selectedArtworkId" [disabled]="busy || !scope" [attr.aria-label]="tr('Opera')">
        <option value="">{{ tr('Scegli l’opera dell’asta') }}</option>
        <option *ngFor="let a of available" [value]="a.id">{{ a.title }} ({{ a.code }})</option>
      </select>
      <button type="button" (click)="startNew()" [disabled]="busy || !scope || !selectedArtworkId">{{ tr('+ Nuova bozza') }}</button>
    </div>
    <div class="auctions-alert" role="alert" *ngIf="error">{{ error }}</div>
    <p class="auctions-muted" role="status" *ngIf="busy">{{ tr('Caricamento bozze…') }}</p>
    <p class="auctions-muted" *ngIf="!available.length">{{ tr('Nessuna opera disponibile: solo le opere disponibili possono avere un’asta.') }}</p>
    <p class="auctions-muted" *ngIf="available.length && !drafts.length">{{ tr('Le bozze appariranno qui. Restano sul computer.') }}</p>
    <ul class="auctions-list" *ngIf="drafts.length">
      <li *ngFor="let d of drafts">
        <div><strong>{{ d.title }}</strong>
          <span class="auctions-badge">{{ tr(d.state === 'draft' ? 'Bozza locale' : d.state === 'pending' ? 'Conferma in sospeso' : 'Ricevuta locale') }}</span>
          <small>{{ artworkTitle(d.artworkId) }}</small></div>
        <div>
          <button type="button" (click)="edit(d)" [disabled]="busy">{{ tr('Modifica') }}</button>
          <button type="button" class="danger" (click)="discard(d)" [disabled]="busy || d.state !== 'draft'">{{ tr('Elimina') }}</button>
        </div>
      </li>
    </ul>
    <section *ngIf="legacyDrafts.length" aria-labelledby="legacy-auctions">
      <h4 id="legacy-auctions">{{ tr('Bozze precedenti') }}</h4>
      <p class="auctions-muted">{{ tr('Le vecchie bozze non sono associate a un archivio. Verifica una sola opera e salva esplicitamente nell’archivio corrente. Gli originali restano intatti.') }}</p>
      <ul class="auctions-list">
        <li *ngFor="let d of legacyDrafts">
          <div><strong>{{ d.title }}</strong><small>{{ artworkTitle(d.artworkId) }}</small></div>
          <button type="button" [disabled]="busy || !canImport(d)" (click)="reviewLegacy(d)">{{ tr('Rivedi vecchia bozza') }}</button>
        </li>
      </ul>
    </section>
  </section>

  <section class="auctions-card" *ngIf="editing" aria-labelledby="auctions-editor">
    <h3 id="auctions-editor">{{ tr('Bozza di asta') }} <span class="auctions-badge">{{ tr(editorState === 'draft' ? 'Bozza locale' : editorState === 'pending' ? 'Conferma in sospeso' : 'Ricevuta locale') }}</span></h3>
    <p class="auctions-muted" *ngIf="editorState !== 'draft'">{{ tr('I dati della richiesta precedente restano intatti. Le modifiche alla bozza non inviano né sostituiscono una richiesta.') }}</p>
    <div class="auctions-alert" role="alert" *ngIf="error">{{ error }}</div>
    <p class="auctions-muted" *ngIf="importingLegacy">{{ tr('Importazione locale da confermare: controlla opera e dati prima di salvare.') }}</p>
    <fieldset class="auctions-form" [disabled]="busy || !!online.review">
      <label>{{ tr('Opera') }}
        <span>{{ artworkTitle(editing.artworkId) }}</span></label>
      <label>{{ tr('Titolo pubblico') }}
        <input type="text" maxlength="160" [(ngModel)]="editing.title" (ngModelChange)="reviewed = false"></label>
      <label>{{ tr('Indirizzo della pagina') }}
        <input type="text" maxlength="60" [(ngModel)]="editing.slug" (ngModelChange)="reviewed = false">
        <small>{{ tr('Solo lettere minuscole, numeri e trattini') }}</small></label>
      <label>{{ tr('Descrizione pubblica') }}
        <textarea rows="4" maxlength="1500" [(ngModel)]="editing.description" (ngModelChange)="reviewed = false"></textarea>
        <small>{{ tr('Scrivila apposta: la descrizione privata dell’archivio non viene usata.') }}</small></label>
      <div class="auctions-two">
        <label>{{ tr('Inizio') }}
          <input type="datetime-local" [ngModel]="startsLocal" (ngModelChange)="startsLocal = $event; reviewed = false"></label>
        <label>{{ tr('Fine') }}
          <input type="datetime-local" [ngModel]="endsLocal" (ngModelChange)="endsLocal = $event; reviewed = false"></label>
      </div>
      <p class="auctions-muted">{{ tr('Gli orari sono nel tuo fuso') }}: {{ timeZone }}. {{ tr('Il server li registra in UTC e decide scadenza e chiusura.') }}</p>
      <div class="auctions-two">
        <label>{{ tr('Prezzo di partenza (€)') }}
          <input type="text" inputmode="decimal" [(ngModel)]="startingText" (ngModelChange)="reviewed = false"></label>
        <label>{{ tr('Rilancio minimo (€)') }}
          <input type="text" inputmode="decimal" [(ngModel)]="incrementText" (ngModelChange)="reviewed = false"></label>
      </div>
      <p class="auctions-muted">{{ tr('Le immagini non sono ancora incluse: l’invio delle foto sarà disponibile con l’interfaccia online verificata.') }}</p>
    </fieldset>
    <div class="auctions-row">
      <button type="button" (click)="cancelEdit()" [disabled]="busy || !!online.review">{{ tr('Annulla') }}</button>
      <button type="button" (click)="save()" [disabled]="busy || !!online.review">{{ tr('Salva bozza') }}</button>
      <button type="button" (click)="review()" [disabled]="busy || !!online.review">{{ tr('Rivedi i dati da inviare') }}</button>
    </div>

    <div *ngIf="reviewed" class="auctions-review" role="region" [attr.aria-label]="tr('Dati da inviare')">
      <h4>{{ tr('Dati che verrebbero inviati') }}</h4>
      <table>
        <tbody><tr *ngFor="let row of rows"><th scope="row">{{ fieldLabel(row.field) }}</th><td>{{ row.value }}</td></tr></tbody>
      </table>
      <h4>{{ tr('Non vengono inviati') }}</h4>
      <ul><li *ngFor="let item of notSent">{{ tr(item) }}</li></ul>
      <button type="button" disabled [attr.aria-describedby]="'auctions-publish-reason'">{{ tr('Pubblica asta') }}</button>
      <p id="auctions-publish-reason" class="auctions-muted">{{ tr('La pubblicazione si attiva solo con il collegamento verificato ad Auctions. Una pubblicazione richiederà sempre la tua conferma.') }}</p>
    </div>
    <div class="auctions-review" *ngIf="online.state === 'connected'">
      <button type="button" (click)="reviewOnline()" [disabled]="!canReviewOnline">{{ onlineLabel('review') }}</button>
      <div *ngIf="online.review as nativeReview">
        <table><tbody><tr *ngFor="let row of onlineReviewRows"><th scope="row">{{ fieldLabel(row.field) }}</th><td>{{ row.value }}</td></tr></tbody></table>
        <div class="auctions-row">
          <button type="button" (click)="cancelOnlineReview()" [disabled]="busy">{{ onlineLabel('cancel') }}</button>
          <button type="button" (click)="confirmOnline()" [disabled]="busy || !canConfirmOnline">{{ onlineLabel(nativeReview.action === 'prepare' ? 'prepare' : 'reconcile') }}</button>
        </div>
      </div>
      <div class="auctions-row">
        <button type="button" (click)="readSettlement()" [disabled]="busy || !!online.review || !savedReference">{{ settlementLabel('read') }}</button>
        <button type="button" (click)="readSettlement('pollSettlement')" [disabled]="busy || !!online.review || !savedReference">{{ settlementLabel('poll') }}</button>
        <button type="button" (click)="readSettlement('historySettlement')" [disabled]="busy || !!online.review || !savedReference">{{ settlementLabel('history') }}</button>
      </div>
      <p role="status" *ngIf="settlementState.kind === 'loading'">{{ settlementLabel('loading') }}</p>
      <p role="status" *ngIf="!busy && settlementState.kind === 'empty'">{{ settlementLabel('empty') }}</p>
      <p role="alert" *ngIf="!busy && settlementState.kind === 'unavailable'">{{ settlementLabel('unavailable') }}</p>
      <div *ngIf="settlementDetails as result">
        <strong>{{ result.status }} · {{ result.amount }}</strong>
        <p>{{ result.receipt }} · {{ result.fulfilledAt }}</p>
        <p *ngIf="result.acknowledgedAt">{{ result.acknowledgedAt }}</p>
        <p class="auctions-muted">{{ result.notice }}</p>
        <button type="button" *ngIf="!result.acknowledgedAt" (click)="acknowledgeSettlement()"
          [disabled]="busy || !!online.review || !savedReference">{{ settlementLabel('ack') }} #{{ result.sequence }}</button>
      </div>
    </div>
  </section>

  <p class="auctions-muted auctions-foot">{{ tr('Interfaccia Auctions') }}: {{ contractVersion }} · {{ tr('non verificata') }}</p>
</div>`,
  styles: [`
.auctions{max-width:880px;display:grid;gap:16px}
.auctions-head h2{margin:0 0 4px}.auctions-head p{margin:0;color:#657b6e}
.auctions-banner{display:grid;gap:4px;padding:12px 14px;border:1px solid #d9c9a3;background:#fbf6e8;color:#5c4a1d;border-radius:8px;font-size:13px}
.auctions-card{border:1px solid #dfe5e1;border-radius:10px;padding:16px;background:#fff;display:grid;gap:10px}
.auctions-card h3,.auctions-card h4{margin:0}
.auctions-row{display:flex;gap:10px;align-items:center;justify-content:space-between;flex-wrap:wrap}
.auctions-muted{margin:0;color:#657b6e;font-size:12px}
.auctions-list{list-style:none;margin:0;padding:0;display:grid;gap:8px}
.auctions-list li{display:flex;justify-content:space-between;gap:10px;align-items:center;padding:10px;border:1px solid #edf0ee;border-radius:8px}
.auctions-list small{display:block;color:#657b6e}
.auctions-badge{margin-left:6px;padding:2px 7px;border-radius:999px;background:#eef2ef;color:#4b5d52;font-size:11px;font-weight:600}
.auctions-form{display:grid;gap:12px;border:0;padding:0;margin:0;min-width:0}
.auctions-form label{display:grid;gap:4px;font-size:13px}
.auctions-form small{color:#657b6e}
.auctions-two{display:grid;grid-template-columns:1fr 1fr;gap:12px}
.auctions-alert{padding:10px 12px;border-radius:8px;background:#fbeaea;color:#8a2b2b;font-size:13px}
.auctions-review{border-top:1px solid #edf0ee;padding-top:12px;display:grid;gap:8px}
.auctions-review table{border-collapse:collapse;font-size:13px}
.auctions-review th{text-align:left;padding:4px 12px 4px 0;color:#4b5d52;font-weight:600;white-space:nowrap;vertical-align:top}
.auctions-review td{padding:4px 0;word-break:break-word}
.auctions-review ul{margin:0;padding-left:18px;font-size:13px;color:#4b5d52}
.auctions-foot{font-size:11px}
.danger{color:#8a2b2b}
@media (max-width:640px){.auctions-two{grid-template-columns:1fr}}
`],
})
export class AuctionsComponent implements OnInit, OnChanges {
  @Input() artworks: Artwork[] = [];
  @Input() language: AppLanguage = 'en';
  /** Artwork chosen from its row ("Avvia asta"); opens a draft for it. */
  @Input() preselectedArtworkId: string | null = null;
  @Output() readonly consumed = new EventEmitter<void>();

  readonly connected = AUCTION_API_CONNECTED;
  readonly contractVersion = AUCTION_CONTRACT_VERSION;
  readonly notSent = NOT_SENT;
  readonly timeZone = (() => { try { return Intl.DateTimeFormat().resolvedOptions().timeZone; } catch { return 'UTC'; } })();
  drafts: DraftSummary[] = [];
  legacyDrafts: LocalAuctionDraft[] = [];
  scope: string | null = null;
  busy = false;
  importingLegacy = false;
  editorState: DraftSummary['state'] = 'draft';
  selectedArtworkId = '';
  private initialized = false;
  private snapshot: NativeDraftSnapshot | null = null;
  private reviewSnapshot: NativeDraftSnapshot | null = null;
  readonly online = new AuctionConnection(request => invoke<unknown>('auction_preview_command', { request }));
  readonly settlement = new SettlementReader(request => invoke<unknown>('auction_preview_command', { request }));
  onlineError = '';
  onlineNotice = '';
  editing: AuctionDraft | null = null;
  startsLocal = '';
  endsLocal = '';
  startingText = '';
  incrementText = '';
  rows: ReviewRow[] = [];
  reviewed = false;
  error = '';

  tr(label: string): string { return translate(label, this.language); }
  private get store(): KeyValueStore | null {
    try { return typeof localStorage === 'undefined' ? null : localStorage; } catch { return null; }
  }
  get available(): Artwork[] { return this.artworks.filter(a => a.status === 'available'); }

  ngOnInit(): void {
    this.initialized = true;
    void this.reloadList();
  }
  ngOnChanges(changes: SimpleChanges): void {
    if (changes['artworks']) {
      this.settlement.clear();
      this.reviewSnapshot = null;
      this.onlineNotice = '';
    }
    if (this.initialized && changes['preselectedArtworkId'] && !this.busy && !this.editing) {
      void this.openPreselected();
    }
  }

  private async refresh(): Promise<void> {
    const rows: DraftSummary[] = [];
    let scope: string | null = null, after: string | null = null;
    const seen = new Set<string>();
    for (let n = 0; n < 200; n++) {
      const page: DraftPage = await invoke<DraftPage>('list_auction_drafts', { expectedScope: scope, after });
      if (!page.scope || (scope !== null && page.scope !== scope)) throw new Error('auction_draft_conflict');
      scope = page.scope;
      for (const row of page.rows) {
        if (seen.has(row.artworkId)) throw new Error('auction_draft_conflict');
        seen.add(row.artworkId);
        rows.push(row);
      }
      if (page.nextAfter === null) {
        this.scope = scope;
        this.drafts = rows;
        const store = this.store;
        this.legacyDrafts = store ? loadDrafts(store) : [];
        return;
      }
      if (page.nextAfter === after || !seen.has(page.nextAfter)) throw new Error('auction_draft_conflict');
      after = page.nextAfter;
    }
    throw new Error('auction_draft_conflict');
  }

  async reloadList(): Promise<void> {
    if (this.busy || this.editing) return;
    this.busy = true;
    this.error = '';
    try { await this.refresh(); }
    catch { this.scope = null; this.drafts = []; this.error = this.tr('Impossibile salvare la bozza su questo computer'); }
    finally { this.busy = false; }
    if (this.scope) await this.openPreselected();
  }

  private async openPreselected(): Promise<void> {
    const id = this.preselectedArtworkId;
    if (!id || !this.scope || this.busy || this.editing) return;
    this.consumed.emit();
    await this.open(id);
  }

  artworkTitle(id: string): string { return this.artworks.find(a => a.id === id)?.title ?? '—'; }
  canImport(d: LocalAuctionDraft): boolean {
    return !!this.scope && this.available.some(a => a.id === d.artworkId) &&
      !this.drafts.some(x => x.artworkId === d.artworkId);
  }

  async startNew(): Promise<void> {
    if (this.available.some(a => a.id === this.selectedArtworkId)) await this.open(this.selectedArtworkId);
  }
  async edit(d: DraftSummary): Promise<void> { await this.open(d.artworkId, true); }
  async reviewLegacy(d: LocalAuctionDraft): Promise<void> {
    if (this.canImport(d)) await this.open(d.artworkId, false, d);
  }
  private async open(id: string, requireExisting = false, legacy?: LocalAuctionDraft): Promise<void> {
    if (this.busy || this.editing || !this.scope) return;
    const expected = this.scope;
    this.busy = true; this.error = '';
    try {
      const snapshot = await nativeDrafts.read(id);
      if (snapshot.scope !== expected) throw new Error('auction_draft_conflict');
      const saved = decodeNativeDraft(snapshot, id);
      if ((requireExisting && !saved) || (legacy && saved)) throw new Error('auction_draft_conflict');
      const artwork = this.artworks.find(a => a.id === id);
      if (!saved && (!artwork || artwork.status !== 'available')) throw new Error('artwork_not_available');
      const draft = legacy ? previewLegacyDraft(legacy, artwork!, snapshot, expected)
        : saved?.draft ?? newAuctionDraft(artwork!);
      if (!legacy && !saved && draft.title.length > 160) draft.title = '';
      this.snapshot = snapshot;
      this.importingLegacy = !!legacy;
      this.editorState = saved?.receipt ? 'receipt' : saved?.pending ? 'pending' : 'draft';
      this.begin(draft);
    } catch (e) {
      this.error = this.draftError(e);
    } finally { this.busy = false; }
  }
  private begin(draft: AuctionDraft): void {
    this.settlement.clear();
    this.reviewSnapshot = null;
    this.editing = { ...draft };
    this.startsLocal = utcToLocalInput(draft.startsAt);
    this.endsLocal = utcToLocalInput(draft.endsAt);
    this.startingText = centsToEuro(draft.startingAmount);
    this.incrementText = centsToEuro(draft.minimumIncrementAmount);
    this.reviewed = false; this.error = ''; this.rows = [];
  }
  cancelEdit(): void {
    if (this.busy || this.online.review) return;
    this.settlement.clear();
    this.reviewSnapshot = null;
    this.editing = null; this.snapshot = null; this.importingLegacy = false;
    this.reviewed = false; this.error = '';
  }

  private draftError(error: unknown): string {
    const code = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
    if (code === 'artwork_not_available') return this.tr(ERRORS['artwork_not_available']);
    if (code === 'auction_draft_conflict') return this.tr('Bozza o archivio cambiati. Ricarica le bozze prima di riprovare.');
    return this.tr('Impossibile salvare la bozza su questo computer');
  }

  private collect(): AuctionDraft | null {
    const e = this.editing;
    if (!e) return null;
    if (!this.importingLegacy && !e.slug.trim() && e.title.trim()) e.slug = slugify(e.title);
    return { artworkId: e.artworkId, title: e.title, description: e.description, slug: e.slug.trim(),
      startsAt: this.startsLocal === utcToLocalInput(e.startsAt) ? e.startsAt : localInputToUtc(this.startsLocal),
      endsAt: this.endsLocal === utcToLocalInput(e.endsAt) ? e.endsAt : localInputToUtc(this.endsLocal),
      startingAmount: this.startingText === centsToEuro(e.startingAmount) ? e.startingAmount : euroToCents(this.startingText),
      minimumIncrementAmount: this.incrementText === centsToEuro(e.minimumIncrementAmount) ? e.minimumIncrementAmount : euroToCents(this.incrementText) };
  }

  async save(): Promise<void> {
    if (this.busy || this.online.review || !this.snapshot) return;
    this.settlement.clear();
    this.reviewSnapshot = null;
    const draft = this.collect(), snapshot = this.snapshot;
    if (!draft) return;
    this.busy = true; this.error = '';
    try {
      await saveNativeAuctionDraft(nativeDrafts, snapshot, draft);
      this.editing = null; this.snapshot = null; this.importingLegacy = false; this.reviewed = false;
      // Saved only after exact native readback. A failed refresh cannot silently
      // turn the persisted draft into a new draft or cause an automatic retry.
      try { await this.refresh(); }
      catch { this.scope = null; this.drafts = []; this.error = this.tr('Bozza salvata. Impossibile ricaricare l’elenco.'); }
    } catch (e) {
      this.error = this.draftError(e);
    } finally { this.busy = false; }
  }
  async discard(d: DraftSummary): Promise<void> {
    if (this.busy || !this.scope || d.state !== 'draft') return;
    this.busy = true; this.error = '';
    try {
      const snapshot = await nativeDrafts.read(d.artworkId);
      if (snapshot.scope !== this.scope || !snapshot.revision) throw new Error('auction_draft_conflict');
      const current = decodeNativeDraft(snapshot, d.artworkId);
      if (!current || current.pending || current.receipt) throw new Error('pending_preview_unresolved');
      if (!confirm(this.tr('Eliminare questa bozza? L’opera non verrà eliminata.') + '\n' + current.draft.title)) return;
      await invoke('remove_auction_draft', { artworkId: d.artworkId, expectedScope: snapshot.scope, expectedRevision: snapshot.revision });
      try { await this.refresh(); }
      catch { this.scope = null; this.drafts = []; this.error = this.tr('Bozza eliminata. Impossibile ricaricare l’elenco.'); }
    } catch (e) { this.scope = null; this.drafts = []; this.error = this.draftError(e); }
    finally { this.busy = false; }
  }
  review(): void {
    if (this.busy) return;
    const draft = this.collect();
    if (!draft) return;
    try { this.rows = reviewRows(draft); this.reviewed = true; this.error = ''; }
    catch (e) {
      this.reviewed = false;
      const code = e instanceof Error ? e.message : '';
      this.error = this.tr(ERRORS[code] ?? 'Dati non validi');
    }
  }

  get previewHeading(): string {
    return ({it:'Anteprime non vincolanti',en:'Non-binding previews',es:'Vistas previas no vinculantes',
      fr:'Aperçus non contraignants',de:'Unverbindliche Vorschauen'})[this.language];
  }

  onlineLabel(key: 'unknown' | 'disabled' | 'disconnected' | 'connected' | 'check' | 'organizations' | 'disconnect' | 'readOnly' | 'review' | 'prepare' | 'reconcile' | 'cancel' | 'prepared' | 'received' | 'failed'): string {
    const labels = {
      it: { unknown:'Connessione non verificata', disabled:'Connessione non abilitata. Le bozze restano locali.', disconnected:'Non connesso. Le bozze restano locali.', connected:'Sessione nativa attiva', check:'Verifica stato connessione', organizations:'Carica organizzazioni', disconnect:'Disconnetti', readOnly:'Sola consultazione', review:'Rivedi richiesta con organizzazione', prepare:'Conferma preparazione locale', reconcile:'Conferma invio / riconciliazione anteprima', cancel:'Annulla conferma', prepared:'Richiesta salvata. Nessun invio effettuato.', received:'Ricevuta anteprima salvata. Nessuna vendita.', failed:'Operazione non confermata. Verifica lo stato locale prima di riprovare.' },
      en: { unknown:'Connection not checked', disabled:'Connection not enabled. Drafts remain local.', disconnected:'Not connected. Drafts remain local.', connected:'Native session active', check:'Check connection status', organizations:'Load organizations', disconnect:'Disconnect', readOnly:'Read only', review:'Review request with organization', prepare:'Confirm local preparation', reconcile:'Confirm preview send / reconciliation', cancel:'Cancel confirmation', prepared:'Request saved. Nothing sent.', received:'Preview receipt saved. No sale.', failed:'Operation not confirmed. Check local state before retrying.' },
      es: { unknown:'Conexión no verificada', disabled:'Conexión no habilitada. Los borradores siguen locales.', disconnected:'Sin conexión. Los borradores siguen locales.', connected:'Sesión nativa activa', check:'Verificar estado de conexión', organizations:'Cargar organizaciones', disconnect:'Desconectar', readOnly:'Solo lectura', review:'Revisar solicitud con organización', prepare:'Confirmar preparación local', reconcile:'Confirmar envío / conciliación de vista previa', cancel:'Cancelar confirmación', prepared:'Solicitud guardada. No se ha enviado.', received:'Recibo guardado. Sin venta.', failed:'Operación no confirmada. Verifica el estado local antes de reintentar.' },
      fr: { unknown:'Connexion non vérifiée', disabled:'Connexion non activée. Les brouillons restent locaux.', disconnected:'Non connecté. Les brouillons restent locaux.', connected:'Session native active', check:'Vérifier la connexion', organizations:'Charger les organisations', disconnect:'Déconnecter', readOnly:'Lecture seule', review:'Vérifier la demande avec organisation', prepare:'Confirmer la préparation locale', reconcile:'Confirmer envoi / rapprochement de l’aperçu', cancel:'Annuler la confirmation', prepared:'Demande enregistrée. Aucun envoi.', received:'Reçu enregistré. Aucune vente.', failed:'Opération non confirmée. Vérifiez l’état local avant de réessayer.' },
      de: { unknown:'Verbindung nicht geprüft', disabled:'Verbindung nicht aktiviert. Entwürfe bleiben lokal.', disconnected:'Nicht verbunden. Entwürfe bleiben lokal.', connected:'Native Sitzung aktiv', check:'Verbindungsstatus prüfen', organizations:'Organisationen laden', disconnect:'Trennen', readOnly:'Nur lesen', review:'Anfrage mit Organisation prüfen', prepare:'Lokale Vorbereitung bestätigen', reconcile:'Vorschauversand / Abgleich bestätigen', cancel:'Bestätigung abbrechen', prepared:'Anfrage gespeichert. Nichts gesendet.', received:'Vorschaubeleg gespeichert. Kein Verkauf.', failed:'Vorgang nicht bestätigt. Vor erneutem Versuch lokalen Status prüfen.' }
    };
    return labels[this.language][key];
  }

  settlementLabel(key: 'read' | 'poll' | 'history' | 'ack' | 'empty' | 'unavailable' | 'loading'): string {
    return ({
      it: {read:'Leggi esito locale dell’asta',poll:'Aggiorna esito',history:'Recupera prossima pagina dello storico',ack:'Conferma solo ricezione esito',empty:'Nessun esito ricevuto per l’associazione verificata.',unavailable:'Operazione non confermata. Controlla l’esito locale prima di riprovare.',loading:'Operazione sull’esito…'},
      en: {read:'Read local auction result',poll:'Refresh result',history:'Recover next history page',ack:'Confirm receipt only',empty:'No result received for the verified association.',unavailable:'Operation not confirmed. Check the local result before retrying.',loading:'Processing result…'},
      es: {read:'Leer resultado local de la subasta',poll:'Actualizar resultado',history:'Recuperar siguiente página del historial',ack:'Confirmar solo recepción',empty:'No hay resultado para la asociación verificada.',unavailable:'Operación no confirmada. Comprueba el resultado local antes de reintentar.',loading:'Procesando resultado…'},
      fr: {read:'Lire le résultat local de l’enchère',poll:'Actualiser le résultat',history:'Récupérer la page suivante de l’historique',ack:'Confirmer uniquement la réception',empty:'Aucun résultat reçu pour l’association vérifiée.',unavailable:'Opération non confirmée. Vérifiez le résultat local avant de réessayer.',loading:'Traitement du résultat…'},
      de: {read:'Lokales Auktionsergebnis lesen',poll:'Ergebnis aktualisieren',history:'Nächste Verlaufsseite abrufen',ack:'Nur Empfang bestätigen',empty:'Kein Ergebnis für die verifizierte Zuordnung erhalten.',unavailable:'Vorgang nicht bestätigt. Vor erneutem Versuch das lokale Ergebnis prüfen.',loading:'Ergebnis wird verarbeitet…'}
    })[this.language][key];
  }


  private get cleanSaved() {
    const snapshot = this.snapshot, draft = this.editing;
    if (!snapshot?.revision || !draft || snapshot.scope !== this.scope || snapshot.artworkId !== draft.artworkId) return null;
    try {
      const stored = decodeNativeDraft(snapshot,draft.artworkId);
      if (!stored || JSON.stringify(draft) !== JSON.stringify(stored.draft) ||
          this.startsLocal !== utcToLocalInput(stored.draft.startsAt) ||
          this.endsLocal !== utcToLocalInput(stored.draft.endsAt) ||
          this.startingText !== centsToEuro(stored.draft.startingAmount) ||
          this.incrementText !== centsToEuro(stored.draft.minimumIncrementAmount)) return null;
      return {snapshot,stored,reference:{artworkId:snapshot.artworkId,scope:snapshot.scope,revision:snapshot.revision}};
    } catch { return null; }
  }
  get savedReference() { return this.cleanSaved?.reference ?? null; }
  get canReviewOnline(): boolean {
    const clean = this.cleanSaved, selected = this.online.selected;
    return !this.busy && this.online.state === 'connected' && !this.online.review &&
      !!clean && !clean.stored.receipt && !!selected?.publicationEligible &&
      (clean.stored.pending ? JSON.parse(clean.stored.pending.body)['tenantId'] === selected.tenantId :
        this.available.some(a => a.id === clean.snapshot.artworkId));
  }
  get canConfirmOnline(): boolean {
    const clean = this.cleanSaved;
    if (!clean || !this.reviewSnapshot || !this.online.review || this.online.state !== 'connected') return false;
    try { assertSameSnapshot(this.reviewSnapshot,clean.snapshot); return true; } catch { return false; }
  }
  get onlineReviewRows(): ReviewRow[] {
    return this.online.review ? Object.entries(JSON.parse(this.online.review.publicBody) as Record<string,unknown>)
      .map(([field,value]) => ({field,value:String(value)})) : [];
  }
  private async onlineAction(work: () => Promise<void>, preserveSettlement = false): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.onlineError = ''; this.onlineNotice = '';
    if (!preserveSettlement) this.settlement.clear();
    try { await work(); }
    catch { this.settlement.clear(); this.onlineError = this.onlineLabel('failed'); }
    finally { this.busy = false; }
  }
  checkConnection(): Promise<void> { return this.onlineAction(() => this.online.refreshStatus()); }
  loadOrganizations(): Promise<void> { return this.onlineAction(() => this.online.loadOrganizations()); }
  chooseOrganization(id: string): Promise<void> { return this.onlineAction(() => this.online.selectOrganization(id)); }
  disconnectOnline(): Promise<void> {
    return this.onlineAction(async () => { this.reviewSnapshot = null; await this.online.disconnect(); });
  }
  cancelOnlineReview(): Promise<void> {
    return this.onlineAction(async () => { this.reviewSnapshot = null; await this.online.cancelReview(); });
  }
  reviewOnline(): Promise<void> {
    if (!this.canReviewOnline) return Promise.resolve();
    return this.onlineAction(async () => {
      const clean = this.cleanSaved, selected = this.online.selected;
      if (!clean || !selected) throw new Error('auction_draft_conflict');
      const {snapshot,stored,reference} = clean;
      assertSameSnapshot(snapshot,await nativeDrafts.read(snapshot.artworkId));
      const body = stored.pending?.body ?? JSON.stringify({tenantId:selected.tenantId,...publicPreviewFields(stored.draft)});
      this.reviewSnapshot = snapshot;
      try { await this.online.reviewDraft(reference,body,!!stored.pending); }
      catch (error) { this.reviewSnapshot = null; throw error; }
    });
  }
  confirmOnline(): Promise<void> {
    if (this.busy || !this.canConfirmOnline) return Promise.resolve();
    return this.onlineAction(async () => {
      const clean = this.cleanSaved, review = this.online.review, pinned = this.reviewSnapshot;
      this.reviewSnapshot = null;
      if (!clean || !review || !pinned) throw new Error('auction_draft_conflict');
      assertSameSnapshot(pinned,clean.snapshot);
      // After confirmation is attempted, an uncertain outcome requires reopening
      // the persisted draft. Never reuse a stale revision or retry automatically.
      this.snapshot = null;
      assertSameSnapshot(pinned,await nativeDrafts.read(pinned.artworkId));
      const result = await this.online.confirm();
      const after = await nativeDrafts.read(pinned.artworkId);
      const saved = decodeNativeDraft(after,pinned.artworkId);
      assertPreviewReadback(pinned,after,clean.stored,saved,review.publicBody,result);
      this.snapshot = after;
      this.editorState = saved?.receipt ? 'receipt' : 'pending';
      this.onlineNotice = this.onlineLabel(result.action === 'prepare' ? 'prepared' : 'received');
    });
  }
  get settlementState() {
    return this.settlement.current(this.online.state === 'connected' ? this.savedReference : null);
  }
  get settlementDetails() {
    const state = this.settlementState;
    return !this.busy && state.kind === 'ready' ? settlementPresentation(state.summary,this.language) : null;
  }
  readSettlement(action: SettlementReadAction = 'readSettlement'): Promise<void> {
    return this.settlementAction(action);
  }
  acknowledgeSettlement(): Promise<void> {
    const state = this.settlementState;
    if (state.kind !== 'ready' || state.summary.acknowledgedAt !== null) return Promise.resolve();
    return this.settlementAction('ackSettlement');
  }
  private settlementAction(action: SettlementReadAction | 'ackSettlement'): Promise<void> {
    if (this.busy || this.online.state !== 'connected' || this.online.review || !this.savedReference) return Promise.resolve();
    const reviewed = this.settlementState;
    return this.onlineAction(async () => {
      const clean = this.cleanSaved;
      if (!clean) throw new Error('auction_draft_conflict');
      assertSameSnapshot(clean.snapshot,await nativeDrafts.read(clean.reference.artworkId));
      const current = this.cleanSaved;
      if (!current) throw new Error('auction_draft_conflict');
      assertSameSnapshot(clean.snapshot,current.snapshot);
      if (action === 'ackSettlement') {
        // A cleared/changed selection cannot confirm an older result.
        if (this.settlementState !== reviewed) throw new Error('auction_draft_conflict');
        await this.settlement.acknowledge(clean.reference);
      } else {
        await this.settlement.load(clean.reference,action);
      }
      try { assertSameSnapshot(clean.snapshot,await nativeDrafts.read(clean.reference.artworkId)); }
      catch (error) { this.settlement.clear(); throw error; }
    }, true);
  }

  fieldLabel(field: string): string {
    const labels: Record<string, string> = { slug: 'Indirizzo della pagina', title: 'Titolo pubblico', description: 'Descrizione pubblica',
      startsAt: 'Inizio (UTC)', endsAt: 'Fine (UTC)', startingAmount: 'Prezzo di partenza (centesimi)',
      minimumIncrementAmount: 'Rilancio minimo (centesimi)', currency: 'Valuta' };
    return this.tr(labels[field] ?? field);
  }
}
