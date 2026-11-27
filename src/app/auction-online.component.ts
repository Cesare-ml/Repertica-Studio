import { CommonModule } from '@angular/common';
import { Component, Input, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { CANCEL_REASONS, leadingConsistency, type AuctionBid, type AuctionState, type AuctionSummary, type CancelReason } from './auction-contract';
import { AuctionOnlineService } from './auction-online.service';
import { confirmed } from './confirm-dialog';
import type { Operation } from './auction-operations';
import { newIdempotencyKey } from './auction-publications';
import type { AppLanguage } from './i18n';
import { translate } from './i18n';

const STATE_LABELS: Record<AuctionState, string> = {
  upcoming: 'In arrivo', live: 'In corso', ended: 'Terminata, in chiusura', awarded: 'Aggiudicata',
  not_awarded: 'Non aggiudicata', cancelled: 'Annullata', unknown: 'Stato sconosciuto',
};
const REASON_LABELS: Record<string, string> = {
  suspected_abuse: 'Sospetto abuso', obvious_test_or_troll: 'Offerta di prova o di disturbo',
  duplicate_or_technical_error: 'Doppione o errore tecnico', participant_request_confirmed: 'Ritirata su richiesta confermata dell’offerente',
  other_admin_reason: 'Altro motivo',
};
const ERROR_LABELS: Record<string, string> = {
  auctions_not_available: 'Il collegamento ad Auctions non è disponibile in questa versione.',
  browser_unavailable: 'Impossibile aprire il browser di sistema.',
  sign_in_in_progress: 'Un accesso è già in corso nel browser.',
  sign_in_cancelled: 'Accesso annullato.',
  sign_in_timeout: 'Tempo scaduto: l’accesso nel browser non è stato completato.',
  oauth_denied: 'Accesso negato nel browser.',
  oauth_exchange_failed: 'Accesso non completato: il servizio Repertica non ha confermato la sessione.',
  sign_in_required: 'Sessione scaduta: accedi di nuovo.',
  network_unavailable: 'Server non raggiungibile. Controlla la connessione e riprova.',
  service_unavailable: 'Servizio aste temporaneamente non disponibile.',
  invalid_response: 'Il server ha risposto in modo non valido: nulla è stato modificato qui.',
  offline: 'Sei offline: aste, offerte e annullamenti non sono aggiornabili.',
  invalid_reason: 'Scegli un motivo; la nota può avere al massimo 500 caratteri.',
  auction_cancelled: 'L’asta è stata annullata: le offerte non si possono più moderare.',
  block_unavailable: 'Impossibile bloccare l’email: il recapito dell’offerente non è disponibile. Annulla senza blocco.',
  auction_closed: 'L’asta è chiusa: il risultato ufficiale non può più essere modificato.',
  tenant_access_denied: 'Non sei autorizzato a gestire questo profilo o questa asta.',
};

/** Signed-in management of the artist's auctions (MVP 2). Shown only when the native connection is available. */
@Component({
  selector: 'rp-auction-online',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
<section class="online" aria-labelledby="online-title">
  <div class="online-row">
    <h3 id="online-title">{{ tr('Aste online') }}
      <span class="online-badge" *ngIf="online.status.development">{{ tr('Servizi locali di prova') }}</span></h3>
    <div class="online-actions" *ngIf="online.status.signedIn">
      <label *ngIf="online.tenants.length > 1">{{ tr('Profilo artista') }}
        <select [ngModel]="online.tenantId" (ngModelChange)="pickTenant($event)">
          <option *ngFor="let t of online.tenants" [value]="t.id">{{ t.name }}</option>
        </select></label>
      <span *ngIf="online.tenants.length === 1" class="online-muted">{{ online.tenants[0].name }}</span>
      <button type="button" (click)="reload()" [disabled]="busy">{{ tr('Aggiorna dal server') }}</button>
      <button type="button" class="link" (click)="signOut()">{{ tr('Esci') }}</button>
    </div>
  </div>

  <div class="online-alert" role="alert" *ngIf="error">{{ error }}</div>
  <p class="online-status" role="status" aria-live="polite">{{ notice }}</p>

  <div *ngIf="!online.status.signedIn" class="online-signin">
    <p>{{ tr('Per gestire le aste online accedi con il tuo account Repertica nel browser di sistema. L’archivio resta sul computer: viene inviato solo ciò che confermi.') }}</p>
    <button type="button" class="primary" (click)="signIn()" [disabled]="online.status.signingIn">{{ tr('Accedi con Repertica') }}</button>
    <ng-container *ngIf="online.status.signingIn">
      <p class="online-muted">{{ tr('Completa l’accesso nella finestra del browser, poi torna qui.') }}</p>
      <button type="button" class="link" (click)="signOut()">{{ tr('Annulla l’accesso') }}</button>
    </ng-container>
  </div>

  <ng-container *ngIf="online.status.signedIn">
    <p class="online-muted" *ngIf="online.tenants.length === 0">{{ tr('Il tuo account non gestisce ancora profili artista abilitati alle aste.') }}</p>
    <table class="online-table" *ngIf="auctions.length">
      <caption class="sr-only">{{ tr('Aste del profilo selezionato') }}</caption>
      <thead><tr><th>{{ tr('Opera') }}</th><th>{{ tr('Stato') }}</th><th>{{ tr('Offerta guida') }}</th><th>{{ tr('Offerte') }}</th><th>{{ tr('Chiusura') }}</th><th><span class="sr-only">{{ tr('Azione') }}</span></th></tr></thead>
      <tbody><tr *ngFor="let a of auctions" [class.selected]="a.auctionId === selected?.auctionId">
        <td><strong>{{ a.title }}</strong></td><td>{{ tr(stateLabel(a.state)) }}</td>
        <td>{{ a.currentAmount === null ? tr('Nessuna offerta') : euro(a.currentAmount) }}</td><td>{{ a.bidCount }}</td>
        <td>{{ when(a.endsAt, 'short') }}</td>
        <td><button type="button" (click)="open(a)" [attr.aria-label]="tr('Gestisci') + ': ' + a.title">{{ tr('Gestisci') }}</button></td>
      </tr></tbody>
    </table>
    <p class="online-muted" *ngIf="loaded && !auctions.length && online.tenants.length">{{ tr('Nessuna asta pubblicata per questo profilo.') }}</p>

    <div class="online-detail" *ngIf="selected as a">
      <h4>{{ a.title }}</h4>
      <dl>
        <div><dt>{{ tr('Stato') }}</dt><dd>{{ tr(stateLabel(a.state)) }}</dd></div>
        <div><dt>{{ tr('Offerta guida') }}</dt><dd>{{ a.currentAmount === null ? tr('Nessuna offerta') : euro(a.currentAmount) }}</dd></div>
        <div *ngIf="a.state === 'upcoming' || a.state === 'live'"><dt>{{ tr('Prossima offerta minima') }}</dt><dd>{{ euro(a.minimumNextBidAmount) }}</dd></div>
        <div><dt>{{ tr('Prezzo di riserva') }}</dt><dd>{{ a.reserveAmount === null ? tr('Nessuno') : euro(a.reserveAmount) + ' · ' + tr(a.reserveMet ? 'raggiunto' : 'non raggiunto') }}</dd></div>
        <div><dt>{{ tr('Inizio') }}</dt><dd>{{ when(a.startsAt, 'medium') }} <small>{{ timeZone }} · UTC {{ a.startsAt.slice(0, 16).replace('T', ' ') }}</small></dd></div>
        <div><dt>{{ tr('Chiusura') }}</dt><dd>{{ when(a.endsAt, 'medium') }} <small>{{ timeZone }} · UTC {{ a.endsAt.slice(0, 16).replace('T', ' ') }}</small></dd></div>
        <div *ngIf="a.maxEndsAt !== a.endsAt"><dt>{{ tr('Chiusura massima con proroghe') }}</dt><dd>{{ when(a.maxEndsAt, 'medium') }}</dd></div>
        <div><dt>{{ tr('Offerte valide') }}</dt><dd>{{ a.bidCount }}<span *ngIf="a.cancelledBidCount"> · {{ a.cancelledBidCount }} {{ tr('annullate') }}</span></dd></div>
        <div *ngIf="a.winningAmount !== null"><dt>{{ tr('Offerta vincente') }}</dt><dd>{{ euro(a.winningAmount) }}</dd></div>
      </dl>
      <p class="online-note" *ngIf="a.state === 'awarded'">{{ tr('Aggiudicazione non significa pagamento: nessuna vendita viene registrata automaticamente nell’archivio.') }}</p>
      <p class="online-alert" *ngIf="consistency === 'mismatch'">{{ tr('Offerta guida e cronologia del server non coincidono: aggiorna dal server. Nulla viene corretto qui.') }}</p>

      <h4>{{ tr('Cronologia delle offerte') }}</h4>
      <p class="online-muted">{{ tr('Le email degli offerenti sono riservate: visibili solo a te, non vengono salvate su questo computer.') }}</p>
      <p class="online-muted" *ngIf="!bids.length">{{ tr('Nessuna offerta ricevuta.') }}</p>
      <table class="online-table" *ngIf="bids.length">
        <caption class="sr-only">{{ tr('Offerte ricevute, dalla prima all’ultima') }}</caption>
        <thead><tr><th>{{ tr('Ora') }}</th><th>{{ tr('Importo') }}</th><th>{{ tr('Offerente') }}</th><th>{{ tr('Email') }}</th><th>{{ tr('Stato') }}</th><th><span class="sr-only">{{ tr('Azione') }}</span></th></tr></thead>
        <tbody><tr *ngFor="let b of bids" [class.cancelled]="b.state === 'cancelled'">
          <td>{{ when(b.placedAt, 'medium') }}</td><td class="amount">{{ euro(b.amount) }}</td><td>{{ b.bidderLabel }}<small *ngIf="b.bidderName">{{ b.bidderName }}</small></td>
          <td>{{ b.bidderEmail ?? tr('non disponibile') }}</td>
          <td>{{ b.state === 'cancelled' ? tr('Annullata') : (b.isLeading ? tr('Guida') : tr('Valida')) }}
            <small *ngIf="b.state === 'cancelled'">{{ when(b.cancelledAt, 'short') }} · {{ tr(reasonLabel(b.cancelReason)) }}<span *ngIf="b.cancelNote"> · {{ b.cancelNote }}</span></small></td>
          <td><button type="button" *ngIf="canCancel(b)" (click)="askCancel(b)"
            [attr.aria-label]="tr('Annulla offerta') + ': ' + euro(b.amount) + ' ' + b.bidderLabel">{{ tr('Annulla offerta') }}</button></td>
        </tr></tbody>
      </table>

      <form class="online-cancel" *ngIf="cancelling as c" (ngSubmit)="confirmCancel()" role="group" [attr.aria-label]="tr('Annullamento')">
        <strong>{{ tr('Annullare questa offerta?') }}</strong>
        <p>{{ euro(c.bid.amount) }} · {{ c.bid.bidderLabel }}<span *ngIf="c.bid.bidderName"> · {{ c.bid.bidderName }}</span> · {{ c.bid.bidderEmail ?? tr('non disponibile') }}</p>
        <p class="online-muted">{{ tr('L’offerta resta nella cronologia come annullata. Il server ricalcola l’offerta guida; la chiusura non cambia. Anche dopo la chiusura: in quel caso viene ricalcolato il vincitore.') }}</p>
        <label>{{ tr('Motivo') }}
          <select name="reason" [(ngModel)]="c.reason" [disabled]="!!c.operation">
            <option *ngFor="let r of reasons" [value]="r">{{ tr(reasonLabel(r)) }}</option>
          </select></label>
        <label>{{ tr('Nota riservata (facoltativa, massimo 500 caratteri)') }}
          <textarea name="note" rows="3" maxlength="500" [(ngModel)]="c.note" [readonly]="!!c.operation"></textarea></label>
        <label class="online-check"><input type="checkbox" name="blockEmail" [(ngModel)]="c.blockEmail" [disabled]="!!c.operation || !c.bid.bidderEmail">
          {{ tr('Blocca questo indirizzo email per le mie aste') }}</label>
        <div class="online-row">
          <button type="button" (click)="cancelling = null" [disabled]="busy">{{ tr('Indietro') }}</button>
          <button type="submit" class="danger" [disabled]="busy">{{ c.operation ? tr('Riprova la stessa richiesta') : tr('Annulla l’offerta') }}</button>
        </div>
      </form>
    </div>
  </ng-container>
</section>`,
  styles: [`
.online{display:grid;gap:10px}
.online h3,.online h4{margin:0}
.online-row{display:flex;gap:10px;align-items:center;justify-content:space-between;flex-wrap:wrap}
.online-actions{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
.online-actions label{display:flex;gap:6px;align-items:center;font-size:13px}
.online-muted,.online-status{margin:0;color:#657b6e;font-size:12px}
.online-note{margin:0;font-size:12px;color:#5c4a1d;background:#fbf6e8;border-radius:6px;padding:8px}
.online-alert{margin:0;padding:10px 12px;border-radius:8px;background:#fbeaea;color:#8a2b2b;font-size:13px}
.online-badge{margin-left:6px;padding:2px 7px;border-radius:999px;background:#fbf6e8;color:#5c4a1d;font-size:11px;font-weight:600}
.online-signin{display:grid;gap:8px;justify-items:start}
.online-signin p{margin:0;font-size:13px}
.online-table{border-collapse:collapse;font-size:13px;width:100%}
.online-table th{text-align:left;font-size:11px;text-transform:uppercase;letter-spacing:.06em;color:#4b5d52;padding:6px;border-bottom:1px solid #dfe5e1}
.online-table td{padding:6px;border-bottom:1px solid #edf0ee;vertical-align:top}
.online-table td:first-child,.online-table td.amount,.online-table td button{white-space:nowrap}
.online-table tr.selected td{background:#f4f7f5}
.online-table tr.cancelled td{color:#8a8f8d}.online-table tr.cancelled td.amount{text-decoration:line-through}
.online-table small{display:block;color:#657b6e}
.online-detail{border-top:1px solid #edf0ee;padding-top:10px;display:grid;gap:8px}
.online-detail dl{display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:8px;margin:0}
.online-detail dt{font-size:11px;color:#657b6e;text-transform:uppercase}.online-detail dd{margin:0;font-size:14px}
.online-detail dd small{display:block;color:#657b6e;font-size:11px}
.online-cancel{display:grid;gap:8px;border:1px solid #e8cfcf;border-radius:8px;padding:12px;background:#fffafa}
.online-cancel p{margin:0;font-size:13px}
.online-check{display:flex !important;gap:8px;align-items:center}
.online-table td small{display:block}.online-cancel label{display:grid;gap:4px;font-size:13px}
.online button{font:inherit;font-size:12px;padding:7px 12px;border:1px solid #d9ddd5;border-radius:8px;background:#fff;color:#33433b;cursor:pointer}
.online button.primary{background:#263f3c;color:#fff;border-color:#263f3c}
.online button.danger{color:#8a2b2b;border-color:#e8cfcf}
.online button.link{border:0;background:none;padding:4px 2px;text-decoration:underline}
.online button:disabled{opacity:.55;cursor:not-allowed}
.online select,.online textarea{font:inherit;font-size:13px;padding:8px 10px;border:1px solid #d9ddd5;border-radius:8px;background:#fff}
.sr-only{position:absolute;clip:rect(0,0,0,0);width:1px;height:1px;overflow:hidden;white-space:nowrap}
`],
})
export class AuctionOnlineComponent implements OnInit {
  @Input() language: AppLanguage = 'en';
  readonly timeZone = (() => { try { return Intl.DateTimeFormat().resolvedOptions().timeZone; } catch { return 'UTC'; } })();
  auctions: AuctionSummary[] = [];
  selected: AuctionSummary | null = null;
  bids: AuctionBid[] = [];
  loaded = false;
  busy = false;
  error = '';
  notice = '';
  consistency: 'consistent' | 'mismatch' | 'unknown' = 'unknown';
  cancelling: { bid: AuctionBid; reason: CancelReason; note: string; blockEmail: boolean; key: string; operation: Operation | null } | null = null;
  readonly reasons = CANCEL_REASONS;

  constructor(readonly online: AuctionOnlineService) {}

  tr(label: string): string { return translate(label, this.language); }
  stateLabel(state: AuctionState): string { return STATE_LABELS[state] ?? STATE_LABELS.unknown; }
  euro(cents: number): string {
    return new Intl.NumberFormat(this.language === 'it' ? 'it-IT' : 'en-GB', { style: 'currency', currency: 'EUR' }).format(cents / 100);
  }

  async ngOnInit(): Promise<void> {
    await this.online.refreshStatus();
    if (this.online.status.signedIn) await this.run(async () => { await this.online.loadTenants(); await this.loadAuctions(); });
  }

  private message(error: unknown): string {
    const code = error instanceof Error ? error.message : typeof error === 'string' ? error : 'unknown';
    return this.tr(ERROR_LABELS[code] ?? 'Operazione non riuscita.');
  }

  private async run(task: () => Promise<void>): Promise<void> {
    this.busy = true; this.error = '';
    try { await task(); }
    catch (error) {
      this.error = this.message(error);
      // Never log bidder data: only the error code reaches the console.
      if (error instanceof Error && error.message === 'sign_in_required') await this.online.refreshStatus();
    } finally { this.busy = false; }
  }

  async signIn(): Promise<void> {
    this.error = ''; this.notice = this.tr('Si apre il browser per l’accesso Repertica…');
    try {
      await this.online.signIn();
      this.notice = this.online.status.signedIn ? this.tr('Accesso effettuato.') : '';
      if (this.online.status.signedIn) await this.run(() => this.loadAuctions());
    } catch (error) { this.notice = ''; this.error = this.message(error); }
  }

  async signOut(): Promise<void> {
    await this.online.signOut();
    this.auctions = []; this.selected = null; this.bids = []; this.cancelling = null; this.loaded = false;
    this.notice = this.tr('Sei uscito: il collegamento è stato chiuso su questo computer.');
  }

  async pickTenant(id: string): Promise<void> {
    this.online.tenantId = id; this.selected = null; this.bids = []; this.cancelling = null;
    await this.run(() => this.loadAuctions());
  }

  async reload(): Promise<void> { await this.run(() => this.loadAuctions()); }

  private async loadAuctions(): Promise<void> {
    if (!this.online.tenantId) { this.auctions = []; this.loaded = true; return; }
    this.auctions = await this.online.manager.loadAuctions();
    this.loaded = true;
    const keep = this.selected && this.auctions.find(a => a.auctionId === this.selected!.auctionId);
    if (keep) await this.loadBids(keep); else { this.selected = null; this.bids = []; }
    this.notice = this.tr('Aggiornato dal server alle') + ' ' + new Date().toLocaleTimeString(this.language);
  }

  private async loadBids(auction: AuctionSummary): Promise<void> {
    this.selected = auction;
    this.bids = await this.online.manager.loadBids(auction.auctionId);
    this.consistency = leadingConsistency(auction, this.bids);
  }

  async open(auction: AuctionSummary): Promise<void> {
    this.cancelling = null;
    await this.run(() => this.loadBids(auction));
  }

/** Dates in the interface language (Angular's DatePipe would use its default en-US locale). */
  when(value: string | null, style: 'short' | 'medium'): string {
    if (!value) return '';
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString(this.language, { dateStyle: style, timeStyle: style });
  }

  /** Gestionale Opere: moderation is possible while the auction runs and after it closed, never once cancelled. */
  canCancel(bid: AuctionBid): boolean {
    const state = this.selected?.state;
    return bid.state === 'valid' && !!state && state !== 'cancelled' && state !== 'unknown';
  }

  reasonLabel(reason: string | null): string {
    return REASON_LABELS[reason ?? ''] ?? 'Altro motivo';
  }

  askCancel(bid: AuctionBid): void {
    // A pending request for the same bid is retried as it is (same key and reason).
    if (this.cancelling?.bid.bidId === bid.bidId && this.cancelling.operation) return;
    this.cancelling = { bid, reason: 'suspected_abuse', note: '', blockEmail: false, key: newIdempotencyKey().replace(/^pub-/, 'cancel-'), operation: null };
  }

  async confirmCancel(): Promise<void> {
    const c = this.cancelling, auction = this.selected;
    if (!c || !auction) return;
    const note = c.note.trim();
    if (!this.reasons.includes(c.reason) || note.length > 500) { this.error = this.tr(ERROR_LABELS['invalid_reason']); return; }
    if (!(await confirmed(this.tr('Confermi l’annullamento dell’offerta? L’operazione è registrata dal server.')))) return;
    await this.run(async () => {
      const { outcome, bids } = await this.online.manager.cancelBid(auction.auctionId, c.bid.bidId,
        { reason: c.reason, note, blockEmail: c.blockEmail }, c.key, c.operation ?? undefined);
      if (bids) { this.bids = bids; }
      if (outcome.needsSignIn) { await this.online.refreshStatus(); throw new Error('sign_in_required'); }
      switch (outcome.operation.state) {
        case 'confirmed':
          this.cancelling = null;
          this.auctions = await this.online.manager.loadAuctions();
          this.selected = this.auctions.find(a => a.auctionId === auction.auctionId) ?? auction;
          this.consistency = leadingConsistency(this.selected, this.bids);
          this.notice = this.tr('Annullamento confermato dal server. Offerta guida e cronologia sono state ricaricate.');
          break;
        case 'rejected':
          this.cancelling = null;
          this.error = this.message(outcome.operation.reason ?? 'unknown');
          break;
        default:
          c.operation = outcome.operation;
          this.notice = this.tr('Conferma in sospeso: la risposta del server non è arrivata. Riprova: verrà usata la stessa richiesta, senza duplicati.');
      }
    });
  }
}
