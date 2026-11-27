import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { CANCEL_REASONS, leadingConsistency, type AuctionBid, type AuctionParticipant, type AuctionState, type AuctionSummary,
  type AuctionTermsView, type CancelReason, type EmailPolicy } from './auction-contract';
import { winnerMailDraft, type WinnerMailDraft } from './auction-winner-mail';
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
  invalid_email: 'Indirizzo email non valido.',
  not_found: 'Il blocco non esiste più: aggiorna l’elenco.',
  invalid_request: 'Richiesta non valida.',
};
/** Public address of the auction pages (the service the Desktop is connected to). */
const PUBLIC_ORIGIN = 'https://auctions.repertica.com';

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
      <div class="od-hero">
        <div>
          <button type="button" class="link" (click)="closeDetail()">← {{ tr('Tutte le aste') }}</button>
          <h4>{{ a.title }}</h4>
          <p class="online-muted">{{ tr('Gestione dell’asta') }} · {{ a.slug }}</p>
        </div>
        <span class="od-status" [attr.data-state]="a.state">{{ tr(stateLabel(a.state)) }}</span>
      </div>
      <div class="online-actions">
        <button type="button" (click)="refreshDetail()" [disabled]="busy">{{ tr('Aggiorna dal server') }}</button>
        <button type="button" class="primary" *ngIf="winnerMail" (click)="writeToWinner()">{{ tr('Scrivi al vincitore') }}</button>
      </div>
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
        <div><dt>{{ tr('Partecipanti') }}</dt><dd>{{ contactsAvailable ? participants.length : '—' }}</dd></div>
        <div><dt>{{ tr('Ultima offerta') }}</dt><dd>{{ latestBid ? euro(latestBid.amount) : tr('Nessuna offerta') }}</dd></div>
        <div><dt>{{ tr('Ultimo aggiornamento') }}</dt><dd>{{ loadedAt ? when(loadedAt, 'short') : tr('Mai') }}</dd></div>
      </dl>
      <p class="online-note" *ngIf="a.state === 'awarded'">{{ tr('Aggiudicazione non significa pagamento: nessuna vendita viene registrata automaticamente nell’archivio.') }}</p>
      <p class="online-alert" *ngIf="consistency === 'mismatch'">{{ tr('Offerta guida e cronologia del server non coincidono: aggiorna dal server. Nulla viene corretto qui.') }}</p>

      <section class="od-section" aria-labelledby="od-comm">
        <div class="online-row">
          <div>
            <p class="od-eyebrow">{{ tr('Comunicazioni') }}</p>
            <h4 id="od-comm">{{ tr('Email dei partecipanti') }}</h4>
            <p class="online-muted">{{ tr('Da incollare nel campo Ccn della tua posta. Solo chi ha verificato l’email.') }}</p>
          </div>
          <button type="button" class="primary" (click)="copyEmails()" [disabled]="!emailList">{{ tr('Copia tutte') }}</button>
        </div>
        <textarea readonly rows="3" [value]="emailList" [attr.aria-label]="tr('Email dei partecipanti')" [placeholder]="tr('Nessuna email disponibile.')"></textarea>
        <div class="od-filters">
          <label class="online-check"><input type="checkbox" [(ngModel)]="excludeLeading"> {{ tr('Escludi chi è in testa') }}</label>
          <label class="online-check"><input type="checkbox" [(ngModel)]="excludeBlocked"> {{ tr('Escludi gli indirizzi bloccati') }}</label>
        </div>
        <p class="online-status" role="status" *ngIf="copyMessage">{{ copyMessage }}</p>
      </section>

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
            [attr.aria-label]="tr('Annulla offerta') + ': ' + euro(b.amount) + ' ' + b.bidderLabel">{{ tr('Annulla offerta') }}</button>
            <button type="button" class="danger" *ngIf="canCancel(b) && b.bidderEmail" (click)="askCancel(b, true)"
              [attr.aria-label]="tr('Annulla e blocca') + ': ' + euro(b.amount) + ' ' + b.bidderLabel">{{ tr('Annulla e blocca') }}</button></td>
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

      <section class="od-section" aria-labelledby="od-part">
        <h4 id="od-part">{{ tr('Partecipanti') }}</h4>
        <p class="online-muted">{{ tr('Chi ha verificato l’email per questa asta.') }}</p>
        <p class="online-alert" *ngIf="participantsError">{{ participantsError }}</p>
        <p class="online-muted" *ngIf="!contactsAvailable">{{ tr('Contatti non disponibili: il server non può leggerli.') }}</p>
        <table class="online-table" *ngIf="participants.length">
          <caption class="sr-only">{{ tr('Partecipanti') }}</caption>
          <thead><tr><th>{{ tr('Nome') }}</th><th>{{ tr('Email') }}</th><th>{{ tr('Stato') }}</th><th>{{ tr('Offerte valide') }}</th><th>{{ tr('Verifica') }}</th></tr></thead>
          <tbody><tr *ngFor="let p of participants" [class.cancelled]="p.status === 'blocked'">
            <td>{{ p.name ?? tr('Senza nome') }}</td><td>{{ p.email }}</td>
            <td>{{ tr(p.status === 'blocked' ? 'Bloccato' : 'Verificato') }}<small *ngIf="p.isLeading">{{ tr('In testa') }}</small></td>
            <td>{{ p.validBidCount }}</td><td>{{ p.verifiedAt ? when(p.verifiedAt, 'short') : '—' }}</td>
          </tr></tbody>
        </table>
        <p class="online-muted" *ngIf="contactsAvailable && !participants.length && !participantsError">{{ tr('Nessun partecipante ha ancora verificato l’email.') }}</p>
      </section>

      <section class="od-section" aria-labelledby="od-tech">
        <h4 id="od-tech">{{ tr('Dettagli e impostazioni') }}</h4>

        <details>
          <summary>{{ tr('Configurazione dell’asta') }}</summary>
          <dl class="od-grid">
            <div><dt>{{ tr('Pagina pubblica') }}</dt><dd>{{ publicAddress(a, 'it') }} <button type="button" class="link" (click)="copyText(publicAddress(a, 'it'))">{{ tr('Copia indirizzo') }}</button></dd></div>
            <div><dt>{{ tr('Pagina pubblica in inglese') }}</dt><dd>{{ publicAddress(a, 'en') }} <button type="button" class="link" (click)="copyText(publicAddress(a, 'en'))">{{ tr('Copia indirizzo') }}</button></dd></div>
            <div><dt>{{ tr('Prezzo di partenza') }}</dt><dd>{{ euro(a.startingAmount) }}</dd></div>
            <div><dt>{{ tr('Rilancio minimo') }}</dt><dd>{{ euro(a.minimumIncrementAmount) }}</dd></div>
            <div><dt>{{ tr('Prezzo di riserva') }}</dt><dd>{{ a.reserveAmount === null ? tr('Nessuno') : euro(a.reserveAmount) }}</dd></div>
            <div><dt>{{ tr('Proroga finale') }}</dt><dd>{{ tr('Un’offerta negli ultimi 2 minuti prolunga la chiusura di 2 minuti.') }}</dd></div>
            <div *ngIf="terms as t"><dt>{{ tr('Versione del regolamento') }}</dt><dd>{{ t.rulesVersion }}</dd></div>
            <div *ngIf="terms as t"><dt>{{ tr('Asta conclusa visibile al pubblico') }}</dt><dd>{{ tr(t.showClosedPublicly ? 'Sì' : 'No') }}</dd></div>
            <div><dt>{{ tr('Identificativo asta') }}</dt><dd>{{ a.auctionId }}</dd></div>
            <div><dt>{{ tr('Versione dei dati') }}</dt><dd>{{ a.version }}</dd></div>
          </dl>
        </details>

        <details>
          <summary>{{ tr('Contatto con il vincitore') }}</summary>
          <p class="online-muted" *ngIf="!terms">{{ tr('Condizioni non disponibili per questa asta.') }}</p>
          <ng-container *ngIf="terms as t">
            <dl class="od-grid">
              <div><dt>{{ tr('La tua email') }}</dt><dd>{{ t.artistEmail }}</dd></div>
              <div><dt>{{ tr('Contatto con il vincitore') }}</dt><dd>{{ tr(t.winner.mode === 'custom_email' ? 'Il vincitore riceve una mia email di vittoria' : 'Ricevo i dati del vincitore e lo contatto io') }}</dd></div>
            </dl>
            <ng-container *ngIf="customWinner(t) as w">
              <p class="od-label">{{ tr('Oggetto') }}</p><p class="od-text">{{ w.subject }}</p>
              <p class="od-label">{{ tr('Messaggio') }}</p><p class="od-text">{{ w.body }}</p>
            </ng-container>
          </ng-container>
        </details>

        <details>
          <summary>{{ tr('Email bloccate e domini temporanei') }}</summary>
          <p class="online-muted">{{ tr('Un indirizzo bloccato non può più verificare l’email nelle tue aste. Se lo sblocchi, chi era già stato escluso dovrà verificare di nuovo.') }}</p>
          <p class="online-alert" role="alert" *ngIf="policyError">{{ policyError }}</p>
          <form class="od-block" (ngSubmit)="blockAddress()">
            <label>{{ tr('Email da bloccare') }}<input type="email" name="blockEmail" autocomplete="off" [(ngModel)]="blockEmailValue"></label>
            <label>{{ tr('Motivo (facoltativo)') }}<input type="text" name="blockReason" maxlength="200" [(ngModel)]="blockReasonValue"></label>
            <button type="submit" class="danger" [disabled]="busy || !blockEmailValue.trim()">{{ tr('Blocca email') }}</button>
          </form>
          <p class="online-muted" *ngIf="policy">{{ tr('Blocchi attivi') }}: {{ policy.blockedEmails.length }}</p>
          <ul class="od-list" *ngIf="policy?.blockedEmails?.length">
            <li *ngFor="let b of policy!.blockedEmails">
              <div><strong>{{ b.emailHint ?? tr('Indirizzo non indicato') }}</strong>
                <small>{{ blockReason(b.reason) }} · {{ when(b.blockedAt, 'short') }}</small></div>
              <button type="button" (click)="removeBlock(b.id)" [disabled]="busy">{{ tr('Rimuovi blocco') }}</button>
            </li>
          </ul>
          <p class="online-muted" *ngIf="policy && !policy.blockedEmails.length">{{ tr('Nessuna email bloccata.') }}</p>
          <h5>{{ tr('Domini email temporanei') }}</h5>
          <p class="online-muted" *ngIf="policy">{{ policy.disposableDomains.count }} {{ tr('domini bloccati dalla piattaforma') }} ·
            {{ policy.disposableDomains.syncedAt ? tr('aggiornati il') + ' ' + when(policy.disposableDomains.syncedAt, 'short') : tr('lista non ancora aggiornata') }}</p>
          <p class="online-muted">{{ tr('La lista dei domini temporanei è della piattaforma, uguale per tutti gli artisti: si aggiorna ogni giorno e da qui si può solo consultare.') }}</p>
        </details>
      </section>
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
.od-hero{display:flex;justify-content:space-between;gap:12px;align-items:flex-start;flex-wrap:wrap}
.od-status{padding:4px 10px;border-radius:999px;background:#eef2ef;color:#33433b;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.06em}
.od-status[data-state="live"]{background:#e6f2e9;color:#2f6b45}.od-status[data-state="cancelled"]{background:#fbeaea;color:#8a2b2b}
.od-section{border-top:1px solid #edf0ee;padding-top:10px;display:grid;gap:8px}
.od-eyebrow{margin:0;font-size:11px;text-transform:uppercase;letter-spacing:.12em;color:#657b6e}
.od-filters{display:flex;gap:16px;flex-wrap:wrap}.od-filters label{display:flex;gap:6px;align-items:center;font-size:13px}
.od-section textarea{width:100%;box-sizing:border-box;resize:vertical}
.od-section details{border:1px solid #edf0ee;border-radius:8px;padding:8px 12px}
.od-section summary{cursor:pointer;font-size:14px;font-weight:600}
.od-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:10px;margin:10px 0 0}
.od-grid dt{font-size:11px;color:#657b6e;text-transform:uppercase}.od-grid dd{margin:0;font-size:13px;word-break:break-word}
.od-label{margin:8px 0 0;font-size:11px;color:#657b6e;text-transform:uppercase}.od-text{margin:2px 0 0;font-size:13px;white-space:pre-wrap;background:#f7f9f8;border-radius:6px;padding:8px}
.od-block{display:grid;grid-template-columns:1fr 1fr auto;gap:8px;align-items:end;margin-top:8px}
.od-block label{display:grid;gap:4px;font-size:13px}.od-block input{font:inherit;font-size:13px;padding:8px 10px;border:1px solid #d9ddd5;border-radius:8px}
.od-list{list-style:none;margin:8px 0 0;padding:0;display:grid;gap:6px}
.od-list li{display:flex;justify-content:space-between;gap:10px;align-items:center;border:1px solid #edf0ee;border-radius:8px;padding:8px 10px}
.od-list small{display:block;color:#657b6e}
.online h5{margin:12px 0 0;font-size:13px}
@media (max-width:640px){.od-block{grid-template-columns:1fr}}
.sr-only{position:absolute;clip:rect(0,0,0,0);width:1px;height:1px;overflow:hidden;white-space:nowrap}
`],
})
export class AuctionOnlineComponent implements OnInit {
  @Input() language: AppLanguage = 'en';
  /** The artist asks to write to the winner: the mail window opens with this draft (nothing is sent). */
  @Output() readonly composeToWinner = new EventEmitter<WinnerMailDraft>();
  readonly timeZone = (() => { try { return Intl.DateTimeFormat().resolvedOptions().timeZone; } catch { return 'UTC'; } })();
  auctions: AuctionSummary[] = [];
  selected: AuctionSummary | null = null;
  bids: AuctionBid[] = [];
  loaded = false;
  busy = false;
  error = '';
  notice = '';
  consistency: 'consistent' | 'mismatch' | 'unknown' = 'unknown';
  participants: AuctionParticipant[] = [];
  contactsAvailable = true;
  participantsError = '';
  terms: AuctionTermsView | null = null;
  policy: EmailPolicy | null = null;
  policyError = '';
  blockEmailValue = '';
  blockReasonValue = '';
  excludeLeading = false;
  excludeBlocked = true;
  copyMessage = '';
  /** When the detail was last read from the server (ISO), shown as "Ultimo aggiornamento". */
  loadedAt = '';
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
    this.participants = []; this.terms = null; this.policy = null;
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
    this.loadedAt = new Date().toISOString();
    await this.loadExtras(auction);
  }

  /** Participants, terms and the e-mail blocks: a failure here never hides the bids, it is shown in its own section. */
  private async loadExtras(auction: AuctionSummary): Promise<void> {
    const id = auction.auctionId, manager = this.online.manager;
    const [people, terms, policy] = await Promise.allSettled([manager.loadParticipants(id), manager.loadTerms(id), manager.loadEmailPolicy()]);
    if (this.selected?.auctionId !== id) return; // the artist moved to another auction meanwhile
    if (people.status === 'fulfilled') { this.participants = people.value.participants; this.contactsAvailable = people.value.contactsAvailable; this.participantsError = ''; }
    else { this.participants = []; this.contactsAvailable = true; this.participantsError = this.message(people.reason); }
    this.terms = terms.status === 'fulfilled' ? terms.value : null;
    if (policy.status === 'fulfilled') { this.policy = policy.value; this.policyError = ''; }
    else { this.policy = null; this.policyError = this.message(policy.reason); }
    for (const result of [people, terms, policy]) {
      if (result.status === 'rejected' && result.reason instanceof Error && result.reason.message === 'sign_in_required') await this.online.refreshStatus();
    }
  }

  closeDetail(): void {
    this.selected = null; this.bids = []; this.cancelling = null; this.participants = []; this.terms = null; this.policy = null;
    this.copyMessage = ''; this.policyError = '';
  }

  async refreshDetail(): Promise<void> { await this.run(() => this.loadAuctions()); }

  /** Participants shown in the e-mail list, after the two filters of the Gestionale. */
  get emailList(): string {
    return this.participants
      .filter(p => !(this.excludeLeading && p.isLeading) && !(this.excludeBlocked && p.status === 'blocked'))
      .map(p => p.email).join(', ');
  }

  /** The last bid received (valid ones only), by server time. */
  get latestBid(): AuctionBid | null {
    const valid = this.bids.filter(b => b.state === 'valid');
    return valid.length ? valid[valid.length - 1] : null;
  }

  get winnerBid(): AuctionBid | null {
    const winner = this.selected?.winnerBidId;
    return winner ? this.bids.find(b => b.bidId === winner) ?? null : null;
  }

  /** Offered only for an awarded auction whose winner's address the server could read. */
  get winnerMail(): WinnerMailDraft | null {
    const bid = this.winnerBid, auction = this.selected;
    if (!bid?.bidderEmail || !auction || auction.state !== 'awarded' || auction.winningAmount === null) return null;
    try { return winnerMailDraft({ email: bid.bidderEmail, name: bid.bidderName, title: auction.title, amountCents: auction.winningAmount }); }
    catch { return null; }
  }

  writeToWinner(): void {
    const draft = this.winnerMail;
    if (draft) this.composeToWinner.emit(draft);
  }

  customWinner(terms: AuctionTermsView): { subject: string; body: string } | null {
    return terms.winner.mode === 'custom_email' ? { subject: terms.winner.subject, body: terms.winner.body } : null;
  }

  publicAddress(auction: AuctionSummary, lang: 'it' | 'en'): string {
    const path = `/${lang}/${lang === 'it' ? 'aste' : 'auctions'}/${auction.slug}`;
    return this.online.status.development ? path : PUBLIC_ORIGIN + path;
  }

  blockReason(reason: string): string {
    return this.tr(REASON_LABELS[reason] ?? (reason === 'manual_block' ? 'Bloccata dall’artista' : reason));
  }

  async copyText(value: string): Promise<void> {
    try { await navigator.clipboard.writeText(value); this.copyMessage = this.tr('Copiato negli appunti.'); }
    catch { this.copyMessage = this.tr('Non è stato possibile copiare: seleziona il testo e copialo a mano.'); }
  }
  async copyEmails(): Promise<void> {
    const list = this.emailList;
    if (!list) return;
    try { await navigator.clipboard.writeText(list); this.copyMessage = this.tr('Elenco copiato negli appunti.'); }
    catch { this.copyMessage = this.tr('Non è stato possibile copiare: seleziona il testo e copialo a mano.'); }
  }

  async blockAddress(): Promise<void> {
    const email = this.blockEmailValue.trim();
    if (!email) return;
    if (!(await confirmed(this.tr('Bloccare questo indirizzo per le tue aste? Non potrà più verificare l’email.')))) return;
    await this.run(async () => {
      const outcome = await this.online.manager.blockEmail(email, this.blockReasonValue);
      if (outcome.needsSignIn) { await this.online.refreshStatus(); throw new Error('sign_in_required'); }
      if (!outcome.ok) throw new Error(outcome.code ?? 'unknown');
      this.blockEmailValue = ''; this.blockReasonValue = '';
      this.notice = this.tr('Email bloccata.');
      if (this.selected) await this.loadExtras(this.selected);
    });
  }

  async removeBlock(id: string): Promise<void> {
    if (!(await confirmed(this.tr('Togliere il blocco a questo indirizzo?')))) return;
    await this.run(async () => {
      const outcome = await this.online.manager.unblockEmail(id);
      if (outcome.needsSignIn) { await this.online.refreshStatus(); throw new Error('sign_in_required'); }
      // A block that is already gone is the state the artist asked for.
      if (!outcome.ok && outcome.code !== 'not_found') throw new Error(outcome.code ?? 'unknown');
      this.notice = this.tr('Blocco rimosso.');
      if (this.selected) await this.loadExtras(this.selected);
    });
  }

  async open(auction: AuctionSummary): Promise<void> {
    this.cancelling = null; this.copyMessage = ''; this.participants = []; this.terms = null;
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

  askCancel(bid: AuctionBid, block = false): void {
    // A pending request for the same bid is retried as it is (same key and reason).
    if (this.cancelling?.bid.bidId === bid.bidId && this.cancelling.operation) return;
    this.cancelling = { bid, reason: 'suspected_abuse', note: '', blockEmail: block && !!bid.bidderEmail, key: newIdempotencyKey().replace(/^pub-/, 'cancel-'), operation: null };
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
          await this.loadExtras(this.selected);
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
