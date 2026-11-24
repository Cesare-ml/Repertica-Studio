import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, ElementRef, Input, ViewChild } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { translate, type AppLanguage } from './i18n';
import { createNativeAuctionEditor } from './auction-native-api';
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
  get busy(): boolean { return this.editor.busy; }
  get dirty(): boolean {
    return !!this.form && (JSON.stringify(this.form) !== this.baseline || !!this.view?.dirty);
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
