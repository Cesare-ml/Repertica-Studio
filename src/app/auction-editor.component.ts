import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, ElementRef, Input, ViewChild } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { translate, type AppLanguage } from './i18n';
import { createNativeAuctionEditor, listNativeAuctionDrafts, type AuctionDraftPage, type AuctionDraftSummary } from './auction-native-api';
import type { SelectedAuctionArtwork } from './auction-editor-session';
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
  private baseline = '';
  form: AuctionEditorFields | null = null;
  error = '';
  notice = '';
  listMode = false;
  page: AuctionDraftPage | null = null;
  private listLoading = false;

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
  get busy(): boolean { return this.editor.busy || this.listLoading; }
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

  get pendingDetails(): { tenantId: string; title: string; description: string; slug: string;
    startsAt: string; endsAt: string; startingAmount: number; minimumIncrementAmount: number } | null {
    const pending = this.view?.pending;
    // The session's native decoder validates the exact canonical body on read.
    return pending ? JSON.parse(pending.body) : null;
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
    if (this.busy || !this.form) return;
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

  close(discard = false): void {
    if (this.busy || (this.dirty && !discard)) return;
    this.editor.close(discard);
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
