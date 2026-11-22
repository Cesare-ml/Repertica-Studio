import { CommonModule } from '@angular/common';
import { Component, Input, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { save } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import type { AppLanguage } from './i18n';
import { translate } from './i18n';
import type { Artwork, Brochure, BrochureItem, GalleryImage } from './models';

/**
 * Offline publication workspace, adapted from Gestionale Opere's editor.
 * The editable document lives in SQLite; all PDF rendering happens in Rust.
 */
@Component({
  selector: 'rp-brochures',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './brochures.component.html',
  styleUrls: ['./brochures.component.css'],
})
export class BrochuresComponent implements OnInit {
  @Input() artworks: Artwork[] = [];
  @Input() language: AppLanguage = 'en';
  /** Name proposed as the artist of a new brochure (stage name or full name from the settings). */
  @Input() artistName = '';

  documents: Brochure[] = [];
  draft: Brochure | null = null;
  addedArtworkId = '';
  busy = false;
  errorMessage = '';
  notice = '';
  previews: Record<string, string> = {};
  /** Path of the PDF just exported, so its folder can be opened. */
  exportedPath: string | null = null;
  galleries: Record<string, GalleryImage[]> = {};
  private loadGeneration = 0;

  tr(label: string): string { return translate(label, this.language); }
  get available(): Artwork[] {
    const chosen = new Set(this.draft?.items.map(item => item.artwork_id) ?? []);
    return this.artworks.filter(artwork => !chosen.has(artwork.id));
  }
  artwork(id: string): Artwork | undefined { return this.artworks.find(artwork => artwork.id === id); }
  get pageCount(): number {
    return (this.draft?.items.length ?? 0) + 2 + (this.draft?.introduction.trim() ? 1 : 0);
  }

  async ngOnInit(): Promise<void> { await this.reload(); }
  async reload(): Promise<void> {
    try {
      this.documents = await api.listBrochures();
    } catch (error) { this.showError(error); }
  }
  create(): void {
    this.draft = {
      id: '', title: '', subtitle: '', artist: this.artistName, introduction: '',
      contact_email: '', contact_website: '', contact_instagram: '', language: this.language,
      items: [], created_at: '', updated_at: ''
    };
    this.resetEditor();
  }
  edit(document: Brochure): void {
    this.draft = { ...document, items: document.items.map(item => ({...item})) };
    this.resetEditor();
    void this.loadAllPreviews();
  }
  close(): void { this.draft = null; this.resetEditor(); }
  private resetEditor(): void {
    this.exportedPath = null;
    this.errorMessage = '';
    this.notice = '';
    this.addedArtworkId = '';
    this.previews = {};
    this.galleries = {};
    ++this.loadGeneration;
  }
  private async loadAllPreviews(): Promise<void> {
    const items = this.draft?.items ?? [];
    await Promise.all(items.map(item => this.loadArtworkMedia(item)));
  }
  private async loadArtworkMedia(item: BrochureItem): Promise<void> {
    const generation = this.loadGeneration;
    try {
      const [gallery, preview] = await Promise.all([
        api.listArtworkGallery(item.artwork_id),
        item.gallery_image_id ? api.artworkGalleryImageData(item.gallery_image_id) : api.artworkImage(item.artwork_id)
      ]);
      if (generation !== this.loadGeneration) return;
      this.galleries[item.artwork_id] = gallery;
      if (preview) this.previews[item.artwork_id] = preview;
      else delete this.previews[item.artwork_id];
    } catch {
      if (generation === this.loadGeneration) delete this.previews[item.artwork_id];
    }
  }
  async addArtwork(): Promise<void> {
    if (!this.draft || !this.addedArtworkId || this.draft.items.some(item => item.artwork_id === this.addedArtworkId)) return;
    const item: BrochureItem = {
      artwork_id: this.addedArtworkId,
      template: 'classic', gallery_image_id: null,
      title_override: '', description_override: '', include_description: true
    };
    this.draft.items.push(item);
    this.addedArtworkId = '';
    await this.loadArtworkMedia(item);
  }
  remove(index: number): void {
    const removed = this.draft?.items.splice(index, 1)[0];
    if (removed) {
      delete this.previews[removed.artwork_id];
      delete this.galleries[removed.artwork_id];
    }
  }
  move(index: number, direction: -1 | 1): void {
    if (!this.draft) return;
    const next = index + direction;
    if (next < 0 || next >= this.draft.items.length) return;
    const [item] = this.draft.items.splice(index, 1);
    this.draft.items.splice(next, 0, item);
  }
  async changeImage(item: BrochureItem, selected: string | null): Promise<void> {
    item.gallery_image_id = selected || null;
    const generation = this.loadGeneration;
    try {
      const preview = item.gallery_image_id
        ? await api.artworkGalleryImageData(item.gallery_image_id)
        : await api.artworkImage(item.artwork_id);
      if (generation === this.loadGeneration) {
        if (preview) this.previews[item.artwork_id] = preview;
        else delete this.previews[item.artwork_id];
      }
    } catch (error) { this.showError(error); }
  }
  private showError(error: unknown): void {
    this.errorMessage = typeof error === 'string' ? error
      : error instanceof Error ? error.message : this.tr('Operazione non riuscita');
    this.notice = '';
  }
  private async persist(): Promise<boolean> {
    if (!this.draft || this.busy) return false;
    this.exportedPath = null;
    if (!this.draft.title.trim()) {
      this.errorMessage = this.tr('Inserisci un titolo per la brochure');
      return false;
    }
    this.busy = true;
    this.errorMessage = '';
    try {
      this.draft = await api.saveBrochure(this.draft);
      await this.reload();
      this.notice = this.tr('Brochure salvata nel tuo archivio');
      return true;
    } catch (error) {
      this.showError(error);
      return false;
    } finally {
      this.busy = false;
    }
  }
  async saveDraft(): Promise<void> { await this.persist(); }
  async duplicate(document: Brochure): Promise<void> {
    const copy: Brochure = {
      ...document, id: '', created_at: '', updated_at: '',
      title: document.title + ' (' + this.tr('Copia') + ')',
      items: document.items.map(item => ({...item}))
    };
    this.busy = true;
    try {
      const saved = await api.saveBrochure(copy);
      await this.reload();
      this.busy = false;
      this.edit(saved);
      this.notice = this.tr('Brochure duplicata');
    } catch (error) { this.busy = false; this.showError(error); }
  }
  async delete(document: Brochure): Promise<void> {
    if (!window.confirm(this.tr('Eliminare questa brochure? Le opere non verranno eliminate.'))) return;
    this.busy = true;
    try {
      await api.deleteBrochure(document.id);
      if (this.draft?.id === document.id) this.close();
      await this.reload();
      this.notice = this.tr('Brochure eliminata');
    } catch (error) { this.showError(error); }
    finally { this.busy = false; }
  }
  /** Show the exported PDF in the file manager. */
  async openExportFolder(): Promise<void> {
    if (!this.exportedPath) return;
    try { await api.revealInFolder(this.exportedPath); }
    catch (error) { this.showError(error); }
  }
  // ---- Preview: the same texts and geometry as the exported PDF -----------------------------
  /** Running header of the inner pages: the artist, or the title when there is none. */
  runningHeader(b: Brochure): string {
    return (b.artist.trim() || b.title.trim()).toUpperCase().slice(0, 70);
  }
  coverKicker(language: string): string {
    return ({ en: 'SELECTED WORKS', es: 'OBRAS SELECCIONADAS', fr: 'ŒUVRES CHOISIES', de: 'AUSGEWÄHLTE WERKE' } as Record<string, string>)[language] ?? 'OPERE SELEZIONATE';
  }
  contactsHeading(language: string): string {
    return ({ en: 'CONTACTS', es: 'CONTACTOS', fr: 'CONTACTS', de: 'KONTAKT' } as Record<string, string>)[language] ?? 'CONTATTI';
  }
  introHeading(language: string): string {
    return ({ en: 'Introduction', es: 'Introducción', fr: 'Introduction', de: 'Einleitung' } as Record<string, string>)[language] ?? 'Introduzione';
  }
  contactEntries(b: Brochure): { label: string; value: string }[] {
    const website = ({ it: 'SITO WEB', es: 'SITIO WEB', fr: 'SITE WEB' } as Record<string, string>)[b.language] ?? 'WEBSITE';
    return [
      { label: 'EMAIL', value: b.contact_email }, { label: website, value: b.contact_website }, { label: 'INSTAGRAM', value: b.contact_instagram },
    ].filter(entry => entry.value.trim());
  }
  /** Code, year, technique / support and size, as printed under the picture. */
  technicalLine(item: BrochureItem): string {
    const art = this.artwork(item.artwork_id);
    if (!art) return '';
    const material = [art.technique ?? '', art.support ?? ''].filter(Boolean).join(' / ');
    const size = art.height_cm != null && art.width_cm != null ? `${art.height_cm} × ${art.width_cm} cm` : '';
    return [art.code, art.year ?? '', material, size].join('  ·  ');
  }
  /** Page number of the n-th artwork (cover and introduction come first; continuation pages are not simulated). */
  pageNumber(index: number): number {
    return 2 + (this.draft?.introduction.trim() ? 1 : 0) + index;
  }
  async exportPdf(): Promise<void> {
    if (!this.draft || this.busy) return;
    if (!this.draft.items.length) {
      this.errorMessage = this.tr('Aggiungi almeno un’opera prima dell’esportazione');
      return;
    }
    const saved = await this.persist();
    if (!saved || !this.draft) return;
    const name = this.draft.title.normalize('NFKD').replace(/[\u0300-\u036f]/g, '')
      .replace(/[^a-zA-Z0-9 _-]/g, '').trim().replace(/\s+/g, '-').slice(0, 75) || 'repertica-brochure';
    try {
      const destination = await save({
        title: this.tr('Esporta brochure PDF'),
        defaultPath: name + '.pdf',
        filters: [{ name: 'PDF', extensions: ['pdf'] }]
      });
      if (typeof destination !== 'string') return;
      this.busy = true;
      const path = await api.exportBrochurePdf(this.draft.id, destination);
      this.notice = this.tr('PDF salvato in') + ' ' + path;
      this.exportedPath = path;
      this.errorMessage = '';
    } catch (error) { this.showError(error); }
    finally { this.busy = false; }
  }
}
