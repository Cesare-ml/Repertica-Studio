import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges, ViewChild } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { AUCTION_CONTRACT_VERSION } from './auction-contract';
import { AuctionOnlineComponent } from './auction-online.component';
import { AuctionOnlineService } from './auction-online.service';
import type { WinnerMailDraft } from './auction-winner-mail';
import { confirmed } from './confirm-dialog';
import type { PendingPublication } from './auction-publications';
import { NOT_SENT, centsToEuro, euroToCents, loadDrafts, loadSavedBankDetails, loadWinnerDefaults, localInputToUtc, removeDraft,
  saveDraft, saveWinnerDefaults, utcToLocalInput, type KeyValueStore, type LocalAuctionDraft } from './auction-drafts';
import { newAuctionDraft, publicArtworkFrom, reviewRows, type AuctionDraft, type ReviewRow } from './auction-preview';
import { WINNER_PLACEHOLDERS, defaultWinnerEmail, type WinnerMode } from './auction-winner-email';
import type { AppLanguage } from './i18n';
import { translate } from './i18n';
import type { Artwork } from './models';

const ERRORS: Record<string, string> = {
  artwork_not_available: 'Solo un’opera disponibile può avere un’asta',
  artwork_not_selected: 'Scegli l’opera dell’asta',
  invalid_title: 'Il titolo deve avere almeno 3 caratteri',
  invalid_description: 'La descrizione è troppo lunga (massimo 1500 caratteri)',
  invalid_schedule: 'Date non valide: l’inizio deve essere futuro, la fine successiva e la durata al massimo 7 giorni',
  invalid_amount: 'Inserisci importi validi per prezzo di partenza e rilancio minimo',
  payload_too_large: 'I dati sono troppo lunghi',
  artwork_image_missing: 'Serve la fotografia principale dell’opera: aggiungila nell’archivio prima di pubblicare',
  artwork_image_invalid: 'La fotografia principale dell’opera non è leggibile',
  artwork_image_too_large: 'La fotografia principale è troppo grande anche dopo la riduzione',
  image_upload_failed: 'Caricamento della fotografia non riuscito: riprova',
  image_capacity_reached: 'Troppe fotografie in attesa di pubblicazione: riprova più tardi',
  unsupported_image_format: 'Formato della fotografia non supportato',
  invalid_image: 'La fotografia non è valida',
  image_required: 'Serve la fotografia principale dell’opera',
  image_not_found: 'La fotografia caricata è scaduta: pubblica di nuovo',
  sign_in_required: 'Sessione scaduta: accedi di nuovo.',
  network_unavailable: 'Server non raggiungibile. Controlla la connessione e riprova.',
  invalid_artist_email: 'Inserisci la tua email: riceverà i dati del vincitore',
  invalid_winner_subject: 'L’oggetto dell’email di vittoria deve avere da 3 a 160 caratteri, su una sola riga',
  invalid_winner_body: 'Il messaggio per il vincitore deve avere da 20 a 2500 caratteri',
  invalid_terms: 'Controlla la tua email e il messaggio per il vincitore',
};
/** Server refusals of a publication (artist-v1 error codes). */
const REFUSALS: Record<string, string> = {
  public_details_exist: 'Questa asta è già stata pubblicata con un titolo o una descrizione diversi: la trovi tra le aste online.',
  auction_exists: 'Questa richiesta è già stata usata per un’altra asta.',
  idempotency_conflict: 'Questa richiesta è già stata usata con dati diversi.',
  tenant_access_denied: 'Non sei autorizzato a pubblicare per questo profilo artista.',
};

/**
 * Auctions workspace: local drafts of a single artwork, review of exactly what is sent, and (only when the native
 * Auctions connection is available and the artist is signed in) publication and management through artist-v1.
 * Without the connection nothing is ever sent and nothing is presented as published.
 */
@Component({
  selector: 'rp-auctions',
  standalone: true,
  imports: [CommonModule, FormsModule, AuctionOnlineComponent],
  template: `
<div class="auctions">
  <div class="auctions-head">
    <h2>{{ tr('Aste') }}</h2>
    <p>{{ tr('Prepara l’asta di una singola opera. L’archivio resta sul computer.') }}</p>
  </div>

  <div class="auctions-banner" role="status" *ngIf="!online.status.available">
    <strong>{{ tr('Non collegato ad Auctions') }}</strong>
    <span>{{ tr('Il servizio online non ha ancora un’interfaccia verificata per pubblicare e gestire le aste: nessun dato viene inviato e nessuna asta risulta pubblicata. Puoi preparare bozze locali.') }}</span>
  </div>

  <section class="auctions-card" aria-labelledby="auctions-published" *ngIf="!online.status.available">
    <h3 id="auctions-published">{{ tr('Aste pubblicate') }}</h3>
    <p class="auctions-muted">{{ tr('Offerte, importi e risultati vengono letti dal server dopo il collegamento. Qui non vengono mai simulati.') }}</p>
  </section>
  <section class="auctions-card" *ngIf="online.status.available"><rp-auction-online [language]="language" (composeToWinner)="composeToWinner.emit($event)"></rp-auction-online></section>

  <section class="auctions-card" aria-labelledby="auctions-pending" *ngIf="online.publications.length">
    <h3 id="auctions-pending">{{ tr('Pubblicazioni da confermare') }}</h3>
    <p class="auctions-muted">{{ tr('Il server non ha ancora confermato queste richieste. Riprova: verrà inviata la stessa richiesta, senza duplicare l’asta.') }}</p>
    <ul class="auctions-list">
      <li *ngFor="let p of online.publications">
        <div><strong>{{ p.title }}</strong>
          <span class="auctions-badge">{{ tr(p.operation.state === 'rejected' ? 'Rifiutata dal server' : 'Conferma in sospeso') }}</span>
          <small *ngIf="p.operation.reason">{{ refusal(p.operation.reason) }}</small></div>
        <div>
          <button type="button" *ngIf="p.operation.state !== 'rejected'" (click)="retryPublication(p)" [disabled]="publishing || !online.status.signedIn">{{ tr('Riprova') }}</button>
          <button type="button" class="danger" (click)="forgetPublication(p)" [disabled]="publishing">{{ tr('Rimuovi dall’elenco') }}</button>
        </div>
      </li>
    </ul>
  </section>

  <section class="auctions-card" aria-labelledby="auctions-drafts" *ngIf="!editing">
    <div class="auctions-row">
      <h3 id="auctions-drafts">{{ tr('Bozze locali') }}</h3>
      <button type="button" class="primary" (click)="startNew()" [disabled]="!available.length">{{ tr('+ Nuova bozza') }}</button>
    </div>
    <p class="auctions-muted" *ngIf="!available.length">{{ tr('Nessuna opera disponibile: solo le opere disponibili possono avere un’asta.') }}</p>
    <p class="auctions-muted" role="status" *ngIf="publishMessage">{{ publishMessage }}</p>
    <p class="auctions-muted" *ngIf="available.length && !drafts.length">{{ tr('Le bozze appariranno qui. Restano sul computer.') }}</p>
    <ul class="auctions-list" *ngIf="drafts.length">
      <li *ngFor="let d of drafts">
        <div><strong>{{ d.title }}</strong>
          <span class="auctions-badge">{{ tr('Bozza locale') }}</span>
          <small>{{ artworkTitle(d.artworkId) }} · {{ tr('aggiornata') }} {{ when(d.updatedAt, 'short') }}</small></div>
        <div>
          <button type="button" (click)="edit(d)">{{ tr('Modifica') }}</button>
          <button type="button" class="danger" (click)="discard(d)">{{ tr('Elimina') }}</button>
        </div>
      </li>
    </ul>
  </section>

  <section class="auctions-card" *ngIf="editing" aria-labelledby="auctions-editor">
    <h3 id="auctions-editor">{{ tr('Bozza di asta') }} <span class="auctions-badge">{{ tr('Bozza locale') }}</span></h3>
    <div class="auctions-alert" role="alert" *ngIf="error">{{ error }}</div>
    <div class="auctions-form">
      <label>{{ tr('Opera') }}
        <select [ngModel]="editing.artworkId" (ngModelChange)="pickArtwork($event)" [disabled]="existingDraft(editing.artworkId)">
          <option *ngFor="let a of available" [value]="a.id">{{ a.title }} ({{ a.code }})</option>
        </select></label>
      <label>{{ tr('Titolo pubblico') }}
        <input type="text" maxlength="160" [(ngModel)]="editing.title" (ngModelChange)="reviewed = false">
        <small>{{ tr('L’indirizzo della pagina lo crea il sito a partire dal titolo pubblico.') }}</small></label>
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
      <div class="auctions-two">
        <label>{{ tr('Prezzo di riserva (€, facoltativo)') }}
          <input type="text" inputmode="decimal" [(ngModel)]="reserveText" (ngModelChange)="reviewed = false"></label>
      </div>
      <p class="auctions-muted">{{ tr('Un’offerta negli ultimi 2 minuti prolunga la chiusura di 2 minuti. Sotto il prezzo di riserva l’asta non viene aggiudicata.') }}</p>
      <fieldset class="auctions-payment">
        <legend>{{ tr('Quando l’asta si chiude con un vincitore') }}</legend>
        <label>{{ tr('La tua email') }}
          <input type="email" maxlength="254" autocomplete="email" [(ngModel)]="editing.artistEmail" (ngModelChange)="reviewed = false">
          <small>{{ tr('Riceve nome ed email del vincitore. Non compare nella pagina pubblica.') }}</small></label>
        <label class="auctions-choice">
          <input type="radio" name="auction-winner-mode" value="contact_privately" [ngModel]="editing.winnerMode" (ngModelChange)="setWinnerMode($event)">
          <span><strong>{{ tr('Ricevo i dati del vincitore e lo contatto io') }}</strong>
            <small>{{ tr('Il vincitore riceve solo un avviso: lo contatterai tu per pagamento e consegna.') }}</small></span></label>
        <label class="auctions-choice">
          <input type="radio" name="auction-winner-mode" value="custom_email" [ngModel]="editing.winnerMode" (ngModelChange)="setWinnerMode($event)">
          <span><strong>{{ tr('Il vincitore riceve una mia email di vittoria') }}</strong>
            <small>{{ tr('Scrivi tu il messaggio, ad esempio con i dati per il pagamento. Rispondendo, il compratore scrive direttamente a te.') }}</small></span></label>
        <ng-container *ngIf="editing.winnerMode === 'custom_email'">
          <label>{{ tr('Oggetto') }}
            <input type="text" maxlength="160" [(ngModel)]="editing.winnerSubject" (ngModelChange)="reviewed = false"></label>
          <label>{{ tr('Messaggio') }}
            <textarea rows="12" maxlength="2500" [(ngModel)]="editing.winnerBody" (ngModelChange)="reviewed = false"></textarea></label>
          <p class="auctions-muted">{{ tr('Campi sostituiti all’invio') }}:
            <span *ngFor="let p of placeholders; let last = last"><code>{{ p.token }}</code> {{ tr(p.label) }}{{ last ? '' : ' · ' }}</span></p>
          <p class="auctions-muted">{{ tr('In fondo al messaggio Repertica aggiunge una riga che spiega che arriva da te.') }}</p>
          <div><button type="button" (click)="resetWinnerEmail()">{{ tr('Ripristina il modello') }}</button></div>
        </ng-container>
      </fieldset>
    </div>
    <div class="auctions-row">
      <button type="button" (click)="cancelEdit()">{{ tr('Annulla') }}</button>
      <button type="button" (click)="save()">{{ tr('Salva bozza') }}</button>
      <button type="button" (click)="review()">{{ tr('Rivedi i dati da inviare') }}</button>
    </div>

    <div *ngIf="reviewed" class="auctions-review" role="region" [attr.aria-label]="tr('Dati da inviare')">
      <h4>{{ tr('Dati che verrebbero inviati') }}</h4>
      <table>
        <tbody><tr *ngFor="let row of rows"><th scope="row">{{ fieldLabel(row.field) }}</th><td [class.auctions-text]="row.field === 'winnerBody'">{{ rowValue(row) }}</td></tr></tbody>
      </table>
      <h4>{{ tr('Non vengono inviati') }}</h4>
      <ul><li *ngFor="let item of notSent">{{ tr(item) }}</li></ul>
      <button type="button" class="primary" (click)="publish()" [disabled]="!online.canPublish || publishing" [attr.aria-describedby]="'auctions-publish-reason'">{{ tr('Pubblica asta') }}</button>
      <p id="auctions-publish-reason" class="auctions-muted" *ngIf="!online.canPublish">{{ tr(publishHint) }}</p>
      <p class="auctions-muted" role="status" *ngIf="publishMessage">{{ publishMessage }}</p>
    </div>
  </section>

  <p class="auctions-muted auctions-foot">{{ tr('Interfaccia Auctions') }}: {{ contractVersion }} · {{ tr(connectionLabel) }}</p>
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
.auctions-form{display:grid;gap:12px}
.auctions-form label{display:grid;gap:4px;font-size:13px}
.auctions-form small{color:#657b6e}
.auctions-two{display:grid;grid-template-columns:1fr 1fr;gap:12px}
.auctions-alert{padding:10px 12px;border-radius:8px;background:#fbeaea;color:#8a2b2b;font-size:13px}
.auctions-payment{border:1px solid #edf0ee;border-radius:8px;padding:10px 12px;display:grid;gap:10px;margin:0}
.auctions-payment legend{font-size:13px;font-weight:600;padding:0 4px}
.auctions-form .auctions-choice{display:flex;gap:8px;align-items:flex-start}
.auctions-form .auctions-choice input{width:auto;margin-top:3px}
.auctions-choice span{display:grid;gap:2px}
.auctions-payment code{font-size:11px;background:#eef2ef;border-radius:4px;padding:1px 4px}
.auctions-review td.auctions-text{white-space:pre-wrap}
.auctions-review{border-top:1px solid #edf0ee;padding-top:12px;display:grid;gap:8px}
.auctions-review table{border-collapse:collapse;font-size:13px}
.auctions-review th{text-align:left;padding:4px 12px 4px 0;color:#4b5d52;font-weight:600;white-space:nowrap;vertical-align:top}
.auctions-review td{padding:4px 0;word-break:break-word}
.auctions-review ul{margin:0;padding-left:18px;font-size:13px;color:#4b5d52}
.auctions-foot{font-size:11px}
.auctions button{font:inherit;font-size:12px;padding:7px 12px;border:1px solid #d9ddd5;border-radius:8px;background:#fff;color:#33433b;cursor:pointer}
.auctions button.primary{background:#263f3c;color:#fff;border-color:#263f3c}
.auctions button.danger{color:#8a2b2b;border-color:#e8cfcf}
.auctions button:disabled{opacity:.55;cursor:not-allowed}
.auctions-list li > div:last-child{display:flex;gap:8px;flex-wrap:wrap;justify-content:flex-end}
.auctions-list button{white-space:nowrap}
.auctions-form input,.auctions-form select,.auctions-form textarea{font:inherit;font-size:13px;padding:8px 10px;border:1px solid #d9ddd5;border-radius:8px;background:#fff;width:100%;box-sizing:border-box}
@media (max-width:640px){.auctions-two{grid-template-columns:1fr}}
`],
})
export class AuctionsComponent implements OnInit, OnChanges {
  @Input() artworks: Artwork[] = [];
  @Input() language: AppLanguage = 'en';
  /** Artwork chosen from its row ("Avvia asta"); opens a draft for it. */
  @Input() preselectedArtworkId: string | null = null;
  @Output() readonly consumed = new EventEmitter<void>();
  /** The artist wants to write to the winner of an auction: the app opens the mail window with this draft. */
  @Output() readonly composeToWinner = new EventEmitter<WinnerMailDraft>();

  readonly contractVersion = AUCTION_CONTRACT_VERSION;
  /** The signed-in panel: reloaded from the server once a publication is confirmed, so the new auction shows. */
  @ViewChild(AuctionOnlineComponent) private onlinePanel?: AuctionOnlineComponent;
  publishing = false;
  publishMessage = '';

  constructor(readonly online: AuctionOnlineService) {}

  get publishHint(): string {
    if (!this.online.status.available) return 'La pubblicazione si attiva solo con il collegamento verificato ad Auctions. Una pubblicazione richiederà sempre la tua conferma.';
    return this.online.status.signedIn ? 'Il profilo artista selezionato non è abilitato a pubblicare.' : 'Accedi con Repertica per pubblicare.';
  }
  get connectionLabel(): string {
    if (!this.online.status.available) return 'non collegata';
    return this.online.status.development ? 'servizi locali di prova' : 'collegata';
  }
  readonly notSent = NOT_SENT;
  readonly placeholders = WINNER_PLACEHOLDERS;
  readonly timeZone = (() => { try { return Intl.DateTimeFormat().resolvedOptions().timeZone; } catch { return 'UTC'; } })();
  drafts: LocalAuctionDraft[] = [];
  editing: AuctionDraft | null = null;
  startsLocal = '';
  endsLocal = '';
  startingText = '';
  incrementText = '';
  reserveText = '';
  rows: ReviewRow[] = [];
  reviewed = false;
  error = '';

  tr(label: string): string { return translate(label, this.language); }
  private get store(): KeyValueStore | null {
    try { return typeof localStorage === 'undefined' ? null : localStorage; } catch { return null; }
  }
  get available(): Artwork[] { return this.artworks.filter(a => a.status === 'available'); }

  ngOnInit(): void { this.refresh(); this.openPreselected(); void this.online.refreshStatus(); }
  ngOnChanges(changes: SimpleChanges): void { if (changes['preselectedArtworkId']) this.openPreselected(); }

  private refresh(): void { const s = this.store; this.drafts = s ? loadDrafts(s) : []; }
  private openPreselected(): void {
    const id = this.preselectedArtworkId;
    if (!id) return;
    const artwork = this.artworks.find(a => a.id === id);
    this.consumed.emit();
    if (!artwork) return;
    const existing = this.drafts.find(d => d.artworkId === id);
    if (existing) this.edit(existing);
    else if (artwork.status === 'available') this.begin(newAuctionDraft(artwork));
    else this.error = this.tr(ERRORS['artwork_not_available']);
  }

  artworkTitle(id: string): string { return this.artworks.find(a => a.id === id)?.title ?? '—'; }
  existingDraft(id: string): boolean { return this.drafts.some(d => d.artworkId === id) && this.editing?.artworkId === id; }

  startNew(): void {
    const free = this.available.find(a => !this.drafts.some(d => d.artworkId === a.id)) ?? this.available[0];
    if (!free) return;
    // The last choices for the winner made on this computer prefill the new draft.
    this.begin({ ...newAuctionDraft(free), ...(this.store ? loadWinnerDefaults(this.store) : {}) });
  }
  edit(d: LocalAuctionDraft): void { this.publishMessage = ''; this.begin({ ...d }); }
  private begin(draft: AuctionDraft): void {
    this.editing = { ...draft };
    this.startsLocal = utcToLocalInput(draft.startsAt);
    this.endsLocal = utcToLocalInput(draft.endsAt);
    this.startingText = centsToEuro(draft.startingAmount);
    this.incrementText = centsToEuro(draft.minimumIncrementAmount);
    this.reserveText = centsToEuro(draft.reserveAmount ?? null);
    if (!this.editing.winnerMode) this.editing.winnerMode = 'contact_privately';
    this.reviewed = false; this.error = ''; this.rows = [];
  }
  pickArtwork(id: string): void {
    const a = this.artworks.find(x => x.id === id);
    if (!a || !this.editing) return;
    this.begin({ ...newAuctionDraft(a), ...this.drafts.find(d => d.artworkId === id) });
  }
  cancelEdit(): void { this.editing = null; this.reviewed = false; this.error = ''; }

  /** The template is offered the first time the artist chooses the winner e-mail. */
  setWinnerMode(mode: WinnerMode): void {
    if (!this.editing) return;
    this.editing.winnerMode = mode;
    this.reviewed = false;
    if (mode === 'custom_email' && !(this.editing.winnerBody ?? '').trim()) this.fillWinnerTemplate();
  }
  async resetWinnerEmail(): Promise<void> {
    if (!this.editing) return;
    if ((this.editing.winnerBody ?? '').trim() && !(await confirmed(this.tr('Sostituire oggetto e messaggio con il modello?')))) return;
    this.fillWinnerTemplate();
  }
  private fillWinnerTemplate(): void {
    if (!this.editing) return;
    const template = defaultWinnerEmail(this.language, this.store ? loadSavedBankDetails(this.store) : null);
    this.editing.winnerSubject = template.subject;
    this.editing.winnerBody = template.body;
    this.reviewed = false;
  }

  private collect(): AuctionDraft | null {
    const e = this.editing;
    if (!e) return null;
    return { artworkId: e.artworkId, title: e.title, description: e.description,
      startsAt: localInputToUtc(this.startsLocal), endsAt: localInputToUtc(this.endsLocal),
      startingAmount: euroToCents(this.startingText), minimumIncrementAmount: euroToCents(this.incrementText),
      reserveAmount: this.reserveText.trim() ? euroToCents(this.reserveText) ?? -1 : null,
      artistEmail: e.artistEmail ?? '', winnerMode: e.winnerMode ?? 'contact_privately', winnerSubject: e.winnerSubject ?? '',
      winnerBody: e.winnerBody ?? '' };
  }

  save(): void {
    const draft = this.collect(), store = this.store;
    if (!draft) return;
    if (!store) { this.error = this.tr('Impossibile salvare la bozza su questo computer'); return; }
    try { this.drafts = saveDraft(store, draft); saveWinnerDefaults(store, draft); this.error = ''; this.editing = null; this.reviewed = false; }
    catch { this.error = this.tr('Impossibile salvare la bozza su questo computer'); }
  }
  async discard(d: LocalAuctionDraft): Promise<void> {
    const store = this.store;
    if (!store || !(await confirmed(this.tr('Eliminare questa bozza? L’opera non verrà eliminata.')))) return;
    try { this.drafts = removeDraft(store, d.artworkId); } catch { this.error = this.tr('Impossibile salvare la bozza su questo computer'); }
  }
  review(): void {
    const draft = this.collect();
    if (!draft) return;
    const artwork = this.artworks.find(a => a.id === draft.artworkId);
    if (!artwork?.image_path) { this.reviewed = false; this.error = this.tr(ERRORS['artwork_image_missing']); return; }
    try { this.rows = reviewRows(draft, Date.now(), publicArtworkFrom(artwork)); this.reviewed = true; this.error = ''; }
    catch (e) {
      this.reviewed = false;
      const code = e instanceof Error ? e.message : '';
      this.error = this.tr(ERRORS[code] ?? 'Dati non validi');
    }
  }
  private outcomeMessage(state: string, reason: string | null): string {
    if (state === 'confirmed') return this.tr('Asta pubblicata: confermata dal server. La bozza locale è chiusa: l’asta è tra le aste online.');
    if (state === 'rejected') return this.tr('Pubblicazione rifiutata dal server') + (reason ? ': ' + this.refusal(reason) : '.');
    return this.tr('Conferma in sospeso: la risposta del server non è arrivata. Riprova dall’elenco delle pubblicazioni da confermare.');
  }

  refusal(reason: string): string { return REFUSALS[reason] ? this.tr(REFUSALS[reason]) : reason; }

  /** Once the server shows the auction, the local draft is closed: publishing it again would only be refused. */
  private async published(artworkId: string): Promise<void> {
    const store = this.store;
    if (store) { try { this.drafts = removeDraft(store, artworkId); } catch { /* the draft simply stays */ } }
    if (this.editing?.artworkId === artworkId) { this.editing = null; this.reviewed = false; }
    // Earlier refused requests for the same artwork are now moot.
    for (const entry of this.online.publications.filter(p => p.artworkId === artworkId && p.operation.state === 'rejected')) {
      this.online.discardPublication(entry);
    }
    await this.onlinePanel?.reload();
  }

  /** Sends exactly the reviewed request, after an explicit confirmation. */
  async publish(): Promise<void> {
    const draft = this.collect();
    if (!draft || !this.reviewed || !this.online.canPublish) return;
    if (!(await confirmed(this.tr('Pubblicare l’asta su Repertica Auctions? Verranno inviati solo i dati mostrati nella revisione.')))) return;
    this.publishing = true; this.publishMessage = this.tr('Invio in corso…');
    try {
      const { operation, needsSignIn } = await this.online.publish(draft, draft.title.trim(),
        this.artworks.find(a => a.id === draft.artworkId) ?? null);
      if (operation.state === 'confirmed') await this.published(draft.artworkId);
      this.publishMessage = needsSignIn ? this.tr('Sessione scaduta: accedi di nuovo.') : this.outcomeMessage(operation.state, operation.reason);
    } catch (e) {
      // Native errors arrive as strings (no image, network), service errors as Error codes.
      const code = typeof e === 'string' ? e : e instanceof Error ? e.message : '';
      this.publishMessage = this.tr(ERRORS[code] ?? 'Pubblicazione non avviata.');
    } finally { this.publishing = false; }
  }

  async retryPublication(entry: PendingPublication): Promise<void> {
    this.publishing = true;
    try {
      const { operation } = await this.online.retryPublication(entry);
      if (operation.state === 'confirmed') await this.published(entry.artworkId);
      this.publishMessage = this.outcomeMessage(operation.state, operation.reason);
    } catch { this.publishMessage = this.tr('Riprova non riuscita: controlla il profilo artista selezionato e la connessione.'); }
    finally { this.publishing = false; }
  }

  async forgetPublication(entry: PendingPublication): Promise<void> {
    if (await confirmed(this.tr('Rimuovere questa richiesta dall’elenco? Se il server l’aveva ricevuta, l’asta resta pubblicata e la trovi tra le aste online.'))) {
      this.online.discardPublication(entry);
    }
  }

/** Dates in the interface language (Angular's DatePipe would use its default en-US locale). */
  when(value: string | null, style: 'short' | 'medium'): string {
    if (!value) return '';
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString(this.language, { dateStyle: style, timeStyle: style });
  }

  fieldLabel(field: string): string {
    const labels: Record<string, string> = { title: 'Titolo pubblico', description: 'Descrizione pubblica',
      startsAt: 'Inizio (UTC)', endsAt: 'Fine (UTC)', startingAmount: 'Prezzo di partenza (centesimi)',
      minimumIncrementAmount: 'Rilancio minimo (centesimi)', currency: 'Valuta', reserveAmount: 'Prezzo di riserva (centesimi)',
      artistEmail: 'La tua email', winnerMode: 'Contatto con il vincitore', winnerSubject: 'Oggetto dell’email di vittoria',
      winnerBody: 'Messaggio per il vincitore', mainImage: 'Fotografia principale', year: 'Anno', technique: 'Tecnica',
      support: 'Supporto', dimensions: 'Misure' };
    return this.tr(labels[field] ?? field);
  }

  rowValue(row: ReviewRow): string {
    if (row.field === 'mainImage') return this.tr('Dall’archivio, ridotta a 2400 px e senza metadati (EXIF, GPS)');
    if (row.field === 'winnerMode') {
      return this.tr(row.value === 'custom_email' ? 'Il vincitore riceve una mia email di vittoria' : 'Ricevo i dati del vincitore e lo contatto io');
    }
    return row.value;
  }
}
