import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, ElementRef, Input, ViewChild } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { translate, type AppLanguage } from './i18n';
import { createNativeAuctionEditor, createNativeAuctionConnection, listNativeAuctionDrafts, type AuctionDraftPage, type AuctionDraftSummary } from './auction-native-api';
import type { SelectedAuctionArtwork } from './auction-editor-session';
import { publicPreviewFields } from './auction-preview';
import { auctionDraftEdits, auctionEditorError, fieldsFromAuctionDraft, type AuctionEditorFields } from './auction-editor-fields';

@Component({
  selector: 'rp-auction-editor',
  imports: [CommonModule, FormsModule],
  templateUrl: './auction-editor.component.html',
  styleUrls: ['./auction-editor.component.css']
})
export class AuctionEditorComponent {
  @Input() language: AppLanguage = 'en';
  @ViewChild('dialog', { static: true }) private dialog!: ElementRef<HTMLDialogElement>;
  private readonly editor = createNativeAuctionEditor();
  readonly online = createNativeAuctionConnection();
  private onlineUpdating = false;
  private baseline = '';
  form: AuctionEditorFields | null = null;
  error = '';
  notice = '';
  listMode = false;
  page: AuctionDraftPage | null = null;
  private listLoading = false;
  private reviewed: ReturnType<typeof publicPreviewFields> | null = null;
  reviewError = false;

  constructor(private readonly cdr: ChangeDetectorRef) {}

  tr(key: string): string { return translate(key, this.language); }
  get receiptLabel(): string {
    return ({
      it: 'Ricevuta locale dell’anteprima non vincolante (nessuna offerta o vendita)',
      en: 'Local non-binding preview receipt (no bid or sale)',
      es: 'Recibo local de vista previa no vinculante (sin puja ni venta)',
      fr: 'Reçu local de l’aperçu non contraignant (ni enchère ni vente)',
      de: 'Lokaler Beleg der unverbindlichen Vorschau (kein Gebot oder Verkauf)'
    })[this.language];
  }
  get view() { return this.editor.view; }
  get busy(): boolean { return this.editor.busy || this.listLoading || this.online.busy || this.onlineUpdating; }
  get dirty(): boolean {
    return !!this.form && (JSON.stringify(this.form) !== this.baseline || !!this.view?.dirty);
  }


  // Auction-specific strings kept here while core i18n #163 is under review.
  managementLabel(key: 'manage' | 'empty' | 'draft' | 'pending' | 'receipt' | 'missing' | 'next' | 'refresh' | 'original' | 'tenant'): string {
    const labels = {
      it: { manage: 'Bozze e anteprime aste', empty: 'Nessuna bozza locale.', draft: 'Bozza locale', pending: 'Da riconciliare', receipt: 'Ricevuta locale', missing: 'Opera non più presente', next: 'Pagina successiva', refresh: 'Prima pagina / aggiorna', original: 'Dati originali della richiesta conservata', tenant: 'Organizzazione' },
      en: { manage: 'Auction drafts and previews', empty: 'No local drafts.', draft: 'Local draft', pending: 'Needs reconciliation', receipt: 'Local receipt', missing: 'Artwork no longer present', next: 'Next page', refresh: 'First page / refresh', original: 'Original saved request details', tenant: 'Organization' },
      es: { manage: 'Borradores y vistas previas de subastas', empty: 'No hay borradores locales.', draft: 'Borrador local', pending: 'Por conciliar', receipt: 'Recibo local', missing: 'Obra ya no presente', next: 'Página siguiente', refresh: 'Primera página / actualizar', original: 'Datos originales de la solicitud guardada', tenant: 'Organización' },
      fr: { manage: 'Brouillons et aperçus des enchères', empty: 'Aucun brouillon local.', draft: 'Brouillon local', pending: 'À rapprocher', receipt: 'Reçu local', missing: 'Œuvre absente', next: 'Page suivante', refresh: 'Première page / actualiser', original: 'Données originales de la demande conservée', tenant: 'Organisation' },
      de: { manage: 'Auktionsentwürfe und Vorschauen', empty: 'Keine lokalen Entwürfe.', draft: 'Lokaler Entwurf', pending: 'Abgleich erforderlich', receipt: 'Lokaler Beleg', missing: 'Werk nicht mehr vorhanden', next: 'Nächste Seite', refresh: 'Erste Seite / aktualisieren', original: 'Ursprüngliche Daten der gespeicherten Anfrage', tenant: 'Organisation' }
    };
    return labels[this.language][key];
  }

  reviewLabel(key: 'review' | 'invalid' | 'localOnly'): string {
    return ({
      it: { review: 'Rivedi i dati pubblici', invalid: 'Completa titolo, identificativo, date future UTC e importi positivi. La durata massima è 7 giorni.', localOnly: 'Solo revisione locale. Nessuna organizzazione selezionata, nessuna richiesta preparata o inviata. La connessione non è ancora abilitata.' },
      en: { review: 'Review public details', invalid: 'Complete the title, URL identifier, future UTC dates and positive amounts. Maximum duration is 7 days.', localOnly: 'Local review only. No organization selected, no request prepared or sent. Connection is not enabled yet.' },
      es: { review: 'Revisar datos públicos', invalid: 'Completa título, identificador, fechas futuras UTC e importes positivos. Duración máxima: 7 días.', localOnly: 'Solo revisión local. Ninguna organización seleccionada ni solicitud preparada o enviada. La conexión aún no está habilitada.' },
      fr: { review: 'Vérifier les données publiques', invalid: 'Complétez le titre, l’identifiant, les dates UTC futures et les montants positifs. Durée maximale : 7 jours.', localOnly: 'Vérification locale uniquement. Aucune organisation sélectionnée, aucune demande préparée ou envoyée. La connexion n’est pas encore activée.' },
      de: { review: 'Öffentliche Angaben prüfen', invalid: 'Titel, URL-Kennung, zukünftige UTC-Daten und positive Beträge ausfüllen. Höchstdauer: 7 Tage.', localOnly: 'Nur lokale Prüfung. Keine Organisation ausgewählt, keine Anfrage vorbereitet oder gesendet. Verbindung noch nicht aktiviert.' }
    })[this.language][key];
  }

  get canReview(): boolean {
    const v = this.view;
    return !!v?.saved && !v.pending && v.artwork.status === 'available' && !this.dirty && !this.busy;
  }
  get reviewedDetails() { return this.canReview ? this.reviewed : null; }

  reviewPublicDetails(): void {
    if (!this.canReview) return;
    this.reviewed = null;
    this.reviewError = false;
    try { this.reviewed = publicPreviewFields(this.view!.draft); }
    catch { this.reviewError = true; }
  }

  invalidateReview(): void {
    this.reviewed = null;
    this.reviewError = false;
  }

  get pendingDetails(): { tenantId: string; title: string; description: string; slug: string;
    startsAt: string; endsAt: string; startingAmount: number; minimumIncrementAmount: number } | null {
    const pending = this.view?.pending;
    // The session's native decoder validates the exact canonical body on read.
    return pending ? JSON.parse(pending.body) : null;
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

  private async onlineAction(work: () => Promise<void>): Promise<void> {
    if (this.busy) return;
    this.onlineUpdating = true;
    this.error = ''; this.notice = '';
    try { await work(); }
    catch { this.error = this.onlineLabel('failed'); }
    finally { this.onlineUpdating = false; this.cdr.markForCheck(); }
  }

  checkConnection(): Promise<void> { return this.onlineAction(() => this.online.refreshStatus()); }
  loadOrganizations(): Promise<void> { return this.onlineAction(() => this.online.loadOrganizations()); }
  chooseOrganization(id: string): Promise<void> { return this.onlineAction(() => this.online.selectOrganization(id)); }
  disconnectOnline(): Promise<void> { return this.onlineAction(() => this.online.disconnect()); }
  cancelOnlineReview(): Promise<void> { return this.onlineAction(() => this.online.cancelReview()); }

  get canReviewOnline(): boolean {
    const v = this.view;
    return !!v?.saved && !v.receipt && !this.dirty && !this.busy && this.online.state === 'connected' &&
      !!this.online.selected?.publicationEligible && !this.online.review &&
      (v.pending ? this.pendingDetails?.tenantId === this.online.selected.tenantId : v.artwork.status === 'available');
  }

  reviewOnline(): Promise<void> {
    if (!this.canReviewOnline) return Promise.resolve();
    return this.onlineAction(async () => {
      const v = this.view, ref = this.editor.savedReference, selected = this.online.selected;
      if (!v || !ref || !selected) throw new Error('auction_draft_conflict');
      const body = v.pending?.body ?? JSON.stringify({tenantId:selected.tenantId, ...publicPreviewFields(v.draft)});
      await this.online.reviewDraft(ref, body, !!v.pending);
    });
  }

  get onlineReviewDetails(): ReturnType<AuctionEditorComponent['readOnlineReview']> {
    return this.readOnlineReview();
  }

  private readOnlineReview(): { tenantId: string; title: string; description: string; slug: string;
    startsAt: string; endsAt: string; startingAmount: number; minimumIncrementAmount: number } | null {
    return this.online.review ? JSON.parse(this.online.review.publicBody) : null;
  }

  confirmOnline(): Promise<void> {
    if (this.dirty || !this.online.review || !this.editor.savedReference) return Promise.resolve();
    return this.onlineAction(async () => {
      const reviewedBody = this.online.review!.publicBody;
      const result = await this.online.confirm();
      await this.editor.reloadSaved(false, result.action === 'prepare' ? result.revision : undefined);
      const v = this.view;
      // No success shown until the persisted record matches the exact reviewed
      // request and (for reconciliation) the native receipt. Never retry here.
      if (!v?.pending || v.pending.body !== reviewedBody ||
          (result.action === 'prepare' && !!v.receipt) ||
          (result.action === 'reconcile' && (!v.receipt || v.receipt.auctionId !== result.receipt.auctionId ||
            v.receipt.tenantId !== result.receipt.tenantId || v.receipt.slug !== result.receipt.slug))) {
        this.refreshForm();
        throw new Error('auction_draft_conflict');
      }
      this.refreshForm();
      this.notice = this.onlineLabel(result.action === 'prepare' ? 'prepared' : 'received');
    });
  }

  async showList(): Promise<void> {
    if (this.dialog.nativeElement.open || this.busy) return;
    this.error = '';
    this.notice = '';
    this.listMode = true;
    this.page = null;
    this.dialog.nativeElement.showModal();
    await this.loadPage(false);
  }

  async loadPage(next: boolean): Promise<void> {
    if (this.busy || !this.listMode || (next && !this.page?.nextAfter)) return;
    this.listLoading = true;
    this.error = '';
    try {
      this.page = await listNativeAuctionDrafts(next ? this.page!.scope : null, next ? this.page!.nextAfter : null);
    } catch (error) {
      // Do not keep a stale page after an archive replacement or failed read.
      this.page = null;
      this.error = auctionEditorError(error);
    } finally { this.listLoading = false; this.cdr.markForCheck(); }
  }

  async openListed(row: AuctionDraftSummary): Promise<void> {
    if (this.busy || !this.listMode || !this.page || !this.page.rows.includes(row)) return;
    this.error = '';
    try {
      await this.editor.open({ id: row.artworkId, title: row.title, status: row.artworkStatus ?? 'missing' }, this.page.scope);
      this.refreshForm();
      this.listMode = false;
      this.page = null;
    } catch (error) { this.page = null; this.error = auctionEditorError(error); }
    finally { this.cdr.markForCheck(); }
  }

  private refreshForm(): void {
    this.invalidateReview();
    const view = this.editor.view;
    this.form = view ? fieldsFromAuctionDraft(view.draft) : null;
    this.baseline = this.form ? JSON.stringify(this.form) : '';
  }

  async show(artwork: SelectedAuctionArtwork): Promise<void> {
    if (this.dialog.nativeElement.open || this.busy) return;
    this.error = '';
    this.notice = '';
    this.form = null;
    this.baseline = '';
    this.listMode = false;
    this.page = null;
    this.dialog.nativeElement.showModal();
    try {
      // Only the selected work's id/title/status enter the editor.
      await this.editor.open({ id: artwork.id, title: artwork.title, status: artwork.status });
      this.refreshForm();
    } catch (error) { this.error = auctionEditorError(error); }
    finally { this.cdr.markForCheck(); }
  }

  async saveDraft(): Promise<void> {
    if (this.busy || !this.form || this.online.review) return;
    this.error = '';
    this.notice = '';
    try {
      this.editor.edit(auctionDraftEdits(this.form));
      await this.editor.save();
      this.refreshForm();
      this.notice = 'Bozza salvata solo nell’archivio locale.';
    } catch (error) { this.error = auctionEditorError(error); }
    finally { this.cdr.markForCheck(); }
  }

  reloadLabel(discard: boolean): string {
    const labels = {
      it: ['Rileggi bozza salvata', 'Scarta modifiche locali e rileggi bozza'],
      en: ['Reload saved draft', 'Discard local edits and reload draft'],
      es: ['Recargar borrador guardado', 'Descartar cambios locales y recargar borrador'],
      fr: ['Relire le brouillon enregistré', 'Ignorer les modifications locales et relire le brouillon'],
      de: ['Gespeicherten Entwurf neu laden', 'Lokale Änderungen verwerfen und Entwurf neu laden']
    };
    return labels[this.language][discard ? 1 : 0];
  }

  async reloadDraft(discard = false): Promise<void> {
    if (this.busy || this.online.review || !this.view?.saved || (this.dirty && !discard)) return;
    this.invalidateReview();
    this.error = '';
    this.notice = '';
    try {
      await this.editor.reloadSaved(discard);
      this.refreshForm();
    } catch (error) { this.error = auctionEditorError(error); }
    finally { this.cdr.markForCheck(); }
  }

  async close(discard = false): Promise<void> {
    if (this.busy || (this.dirty && !discard)) return;
    if (this.online.state === 'connected' || this.online.review) {
      // No token survives an explicit UI logout; if the IPC fails the
      // controller resets to unknown, and the offline dialog can still close.
      try { await this.online.disconnect(); } catch { /* local editor remains usable */ }
    }
    this.editor.close(discard);
    this.invalidateReview();
    this.dialog.nativeElement.close();
    this.listMode = false;
    this.page = null;
    this.form = null;
    this.baseline = '';
    this.error = '';
    this.notice = '';
  }

  cancel(event: Event): void {
    event.preventDefault();
    this.close();
  }
}
