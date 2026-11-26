import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, EventEmitter, Input, OnInit, Output, ViewEncapsulation } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { open, save } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import { confirmed } from './confirm-dialog';
import { AppLanguage, translate } from './i18n';
import { MediaEditorComponent } from './media-editor.component';
import type {
  Artwork, CopyrightStatus, DuplicateGroup, EditOperation, MediaCollection, MediaDetail, MediaItem, MediaTag, PreviewData, SimilarPair
} from './models';

type View = 'all' | 'trash' | 'duplicates';
type Tab = 'info' | 'usages' | 'versions' | 'edit' | 'export';

/** Catalogue of every picture of the archive: search, organise, edit (with versions), export. */
@Component({
  selector: 'rp-media-center',
  standalone: true,
  imports: [CommonModule, FormsModule, MediaEditorComponent],
  encapsulation: ViewEncapsulation.None,
  styles: [`
    .mc{display:grid;gap:14px}
    .mc-head{display:flex;justify-content:space-between;align-items:center;gap:12px;flex-wrap:wrap}
    .mc-head h2{margin:0;font:500 22px Georgia,'Times New Roman',serif}
    .mc button{font:inherit;font-size:12px;padding:7px 12px;border:1px solid #d9ddd5;border-radius:8px;background:#fff;cursor:pointer;color:#33433b}
    .mc button.primary{background:#263f3c;color:#fff;border-color:#263f3c}
    .mc button.danger{color:#a3382c}
    .mc button:disabled{opacity:.55;cursor:not-allowed}
    .mc input,.mc select,.mc textarea{font:inherit;font-size:13px;padding:8px 10px;border:1px solid #d9ddd5;border-radius:8px;background:#fff;width:100%;box-sizing:border-box}
    .mc-note{margin:0;padding:9px 12px;border-radius:9px;font-size:12px}
    .mc-note.ok{background:#e3f1e6;color:#2c6a3e}.mc-note.err{background:#fbe9e6;color:#a3382c}
    .mc-shell{display:grid;grid-template-columns:200px minmax(0,1fr) auto;gap:16px;align-items:start}
    .mc-shell.picker{grid-template-columns:minmax(0,1fr) auto}
    .mc-nav{display:grid;gap:6px}
    .mc-nav h4{margin:10px 0 2px;font-size:10px;letter-spacing:1.4px;color:#8d938d;text-transform:uppercase}
    .mc-nav button{text-align:left}
    .mc-nav button.active{background:#263f3c;color:#fff;border-color:#263f3c}
    .mc-create{display:flex;gap:6px}
    .mc-tags{display:flex;flex-wrap:wrap;gap:5px}
    .mc-tags button{border-radius:99px;padding:4px 10px}
    .mc-tags button.active{background:#dfe9e2;border-color:#9fbbac}
    .mc-main{display:grid;gap:12px;min-width:0}
    .mc-tools{display:flex;gap:12px;align-items:center}
    .mc-tools span{white-space:nowrap;font-size:12px;color:#7a867f}
    .mc-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:12px}
    .mc-card{display:grid;gap:4px;text-align:left;padding:8px;border:1px solid #e3e6e0;border-radius:11px;background:#fff;cursor:pointer}
    .mc-card.selected{outline:2px solid #3f7d68}
    .mc-thumb{position:relative;aspect-ratio:1;border-radius:8px;overflow:hidden;background:#eef0ec;display:grid;place-items:center}
    .mc-thumb img{width:100%;height:100%;object-fit:cover}
    .mc-badge{position:absolute;right:5px;top:5px;background:#263f3cdd;color:#fff;font-size:11px;padding:2px 7px;border-radius:99px}
    .mc-card strong{font-size:12px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
    .mc-card small{color:#7a867f;font-size:11px}
    .mc-empty{padding:40px 20px;text-align:center;color:#7a867f;font-size:13px}
    .mc-inspector{width:min(420px,38vw);min-width:300px;display:grid;gap:10px;padding:12px;border:1px solid #e3e6e0;border-radius:12px;background:#fff;max-height:82vh;overflow:auto}
    .mc-inspector.wide{width:min(720px,52vw)}
    .mc-ihead{display:flex;justify-content:space-between;gap:8px;align-items:flex-start}
    .mc-ihead strong{font-size:13px;word-break:break-word}
    .mc-preview{width:100%;max-height:230px;object-fit:contain;background:#1f2623;border-radius:9px}
    .mc-tabs{display:flex;gap:4px;flex-wrap:wrap}
    .mc-tabs button.active{background:#dfe9e2;border-color:#9fbbac}
    .mc-form{display:grid;gap:8px}
    .mc-form label{display:grid;gap:3px;font-size:12px;font-weight:600;color:#4d5f56}
    .mc-row{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
    .mc-list{display:grid;gap:6px;margin:0;padding:0;list-style:none;font-size:12px}
    .mc-list li{display:flex;justify-content:space-between;gap:8px;align-items:center;padding:7px 9px;border:1px solid #eef0ec;border-radius:8px}
    .mc-list li.current{background:#f1f6f2;border-color:#cfe0d5}
    .mc-dl{display:grid;grid-template-columns:auto 1fr;gap:3px 12px;font-size:12px;margin:0}
    .mc-dl dt{color:#7a867f}.mc-dl dd{margin:0;word-break:break-all}
    .mc-hint{margin:0;font-size:12px;color:#66756d}
    .mc-group{border:1px solid #e3e6e0;border-radius:11px;padding:10px;background:#fff;display:grid;gap:8px}
    .mc-pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}
    .mc-dupes{display:grid;gap:12px}
    .mc-items{display:flex;gap:10px;flex-wrap:wrap}
    .mc-items .mc-card{width:150px}
    @media (max-width:1100px){.mc-shell{grid-template-columns:minmax(0,1fr)}.mc-inspector,.mc-inspector.wide{width:auto}}
  `],
  template: `
    <div class="mc">
      <div class="mc-head">
        <h2>{{ tr(pickerMode ? 'Scegli dalla libreria' : 'Libreria immagini') }}</h2>
        <button type="button" class="primary" (click)="importFiles()" [disabled]="busy">{{ tr('Importa immagini…') }}</button>
      </div>
      <p class="mc-note err" *ngIf="error" role="alert">{{ error }}</p>
      <p class="mc-note ok" *ngIf="message" role="status">{{ message }}</p>

      <div class="mc-shell" [class.picker]="pickerMode">
        <aside class="mc-nav" *ngIf="!pickerMode">
          <h4>{{ tr('Viste') }}</h4>
          <button type="button" [class.active]="view === 'all'" (click)="setView('all')">{{ tr('Tutte le immagini') }}</button>
          <button type="button" [class.active]="view === 'trash'" (click)="setView('trash')">{{ tr('Cestino') }}</button>
          <button type="button" [class.active]="view === 'duplicates'" (click)="setView('duplicates')">{{ tr('Duplicati') }}</button>
          <h4>{{ tr('Tag') }}</h4>
          <div class="mc-tags"><button type="button" *ngFor="let tag of tags" [class.active]="activeTagId === tag.id" (click)="toggleFilterTag(tag.id)">{{ tag.name }}</button></div>
          <h4>{{ tr('Raccolte') }}</h4>
          <button type="button" *ngFor="let c of collections" [class.active]="activeCollectionId === c.id" (click)="toggleCollection(c.id)">{{ c.name }} ({{ c.item_count }})</button>
          <div class="mc-create"><input [(ngModel)]="newCollection" [placeholder]="tr('Nuova raccolta')" [attr.aria-label]="tr('Nuova raccolta')" maxlength="60" (keyup.enter)="createCollection()">
            <button type="button" (click)="createCollection()" [attr.aria-label]="tr('Nuova raccolta')">+</button></div>
        </aside>

        <section class="mc-main">
          <ng-container *ngIf="view !== 'duplicates'">
            <div class="mc-tools"><input type="search" [(ngModel)]="search" (ngModelChange)="onSearch()" [placeholder]="tr('Cerca per titolo, nome file o tag…')" [attr.aria-label]="tr('Cerca')">
              <span>{{ total }} {{ tr('immagini') }}</span></div>
            <div class="mc-empty" *ngIf="loading">{{ tr('Caricamento…') }}</div>
            <div class="mc-empty" *ngIf="!loading && !items.length">
              <strong>{{ tr(view === 'trash' ? 'Il cestino è vuoto' : 'Nessuna immagine') }}</strong>
              <p *ngIf="view === 'all'">{{ tr('Le fotografie delle opere compaiono qui automaticamente. Puoi anche importare nuove immagini.') }}</p></div>
            <div class="mc-grid" *ngIf="!loading && items.length">
              <button type="button" class="mc-card" *ngFor="let item of items" [class.selected]="selected?.item?.id === item.id" (click)="select(item.id)">
                <span class="mc-thumb"><img *ngIf="thumbs[item.current_revision_id]" [src]="thumbs[item.current_revision_id]" alt="">
                  <span class="mc-badge" *ngIf="item.usage_count">{{ item.usage_count }}</span></span>
                <strong>{{ item.title || item.original_name }}</strong>
                <small>{{ item.width }} × {{ item.height }} px<ng-container *ngIf="item.revision_number > 1"> · v{{ item.revision_number }}</ng-container></small>
              </button>
            </div>
            <div class="mc-row" *ngIf="items.length < total"><button type="button" (click)="loadMore()">{{ tr('Carica altre') }}</button></div>
          </ng-container>

          <div class="mc-dupes" *ngIf="view === 'duplicates'">
            <div class="mc-row"><strong>{{ tr('File identici') }}</strong></div>
            <p class="mc-hint" *ngIf="!exactGroups.length">{{ tr('Nessun file identico trovato.') }}</p>
            <div class="mc-group" *ngFor="let group of exactGroups">
              <span class="mc-hint">{{ group.items.length }} {{ tr('file identici') }} · {{ formatSize(group.size_bytes) }}</span>
              <div class="mc-items"><button type="button" class="mc-card" *ngFor="let item of group.items" (click)="select(item.id)">
                <span class="mc-thumb"><img *ngIf="thumbs[item.current_revision_id]" [src]="thumbs[item.current_revision_id]" alt=""><span class="mc-badge" *ngIf="item.usage_count">{{ item.usage_count }}</span></span>
                <strong>{{ item.title || item.original_name }}</strong></button></div>
            </div>
            <div class="mc-row"><strong>{{ tr('Immagini simili') }}</strong>
              <button type="button" (click)="scanSimilar()" [disabled]="scanning">{{ scanning ? tr('Analisi in corso…') : tr('Cerca immagini simili') }}</button></div>
            <p class="mc-hint" *ngIf="similarScanned && !similar.length">{{ tr('Nessuna immagine simile trovata.') }}</p>
            <div class="mc-group" *ngFor="let pair of similar">
              <span class="mc-hint">{{ tr('Somiglianza') }}: {{ (pair.similarity * 100).toFixed(0) }}%</span>
              <div class="mc-pair"><button type="button" class="mc-card" *ngFor="let item of [pair.first, pair.second]" (click)="select(item.id)">
                <span class="mc-thumb"><img *ngIf="thumbs[item.current_revision_id]" [src]="thumbs[item.current_revision_id]" alt=""></span>
                <strong>{{ item.title || item.original_name }}</strong><small>{{ item.width }} × {{ item.height }} px</small></button></div>
            </div>
          </div>
        </section>

        <aside class="mc-inspector" *ngIf="selected as d" [class.wide]="tab === 'edit'">
          <div class="mc-ihead"><div><small>{{ tr('Immagine') }} · v{{ d.item.revision_number }}</small><br><strong>{{ d.item.title || d.item.original_name }}</strong></div>
            <div class="mc-row"><button type="button" class="primary" *ngIf="pickerMode" (click)="picked.emit(d.item)">{{ tr('Usa questa immagine') }}</button>
              <button type="button" (click)="closeInspector()" [attr.aria-label]="tr('Chiudi')">×</button></div></div>
          <img class="mc-preview" *ngIf="preview" [src]="preview.data_url" [alt]="d.item.alt_text || d.item.title || ''">
          <nav class="mc-tabs" *ngIf="!pickerMode">
            <button type="button" [class.active]="tab === 'info'" (click)="setTab('info')">{{ tr('Informazioni') }}</button>
            <button type="button" [class.active]="tab === 'usages'" (click)="setTab('usages')">{{ tr('Utilizzi') }} ({{ d.usages.length }})</button>
            <button type="button" [class.active]="tab === 'versions'" (click)="setTab('versions')">{{ tr('Versioni') }} ({{ d.revisions.length }})</button>
            <button type="button" [class.active]="tab === 'edit'" (click)="setTab('edit')" [disabled]="d.item.state === 'TRASHED'">{{ tr('Modifica') }}</button>
            <button type="button" [class.active]="tab === 'export'" (click)="setTab('export')">{{ tr('Esporta') }}</button>
          </nav>

          <ng-container *ngIf="!pickerMode && tab === 'info'">
            <div class="mc-form">
              <label>{{ tr('Titolo') }}<input [(ngModel)]="meta.title" maxlength="200"></label>
              <label>{{ tr('Descrizione') }}<textarea rows="3" [(ngModel)]="meta.description" maxlength="5000"></textarea></label>
              <label>{{ tr('Testo alternativo') }}<textarea rows="2" [(ngModel)]="meta.alt_text" maxlength="500"></textarea></label>
              <label>{{ tr('Crediti') }}<input [(ngModel)]="meta.credits" maxlength="300"></label>
              <label>{{ tr('Diritti') }}<select [(ngModel)]="meta.copyright_status">
                <option value="UNKNOWN">{{ tr('Non indicati') }}</option><option value="OWNED">{{ tr('Di mia proprietà') }}</option>
                <option value="LICENSED">{{ tr('Con licenza') }}</option><option value="PUBLIC_DOMAIN">{{ tr('Pubblico dominio') }}</option></select></label>
              <button type="button" class="primary" (click)="saveMeta()" [disabled]="busy">{{ tr('Salva informazioni') }}</button>
            </div>
            <div class="mc-form"><strong>{{ tr('Tag') }}</strong>
              <div class="mc-tags"><button type="button" *ngFor="let tag of tags" [class.active]="selectedTags.has(tag.id)" (click)="toggleTag(tag.id)">{{ tag.name }}</button></div>
              <div class="mc-create"><input [(ngModel)]="newTag" [placeholder]="tr('Nuovo tag')" [attr.aria-label]="tr('Nuovo tag')" maxlength="60" (keyup.enter)="createTag()"><button type="button" (click)="createTag()">{{ tr('Aggiungi') }}</button></div>
              <button type="button" (click)="saveTags()" [disabled]="busy">{{ tr('Salva tag') }}</button></div>
            <div class="mc-form" *ngIf="collections.length"><strong>{{ tr('Raccolte') }}</strong>
              <div class="mc-tags"><button type="button" *ngFor="let c of collections" [class.active]="d.collection_ids.includes(c.id)" (click)="toggleMembership(c.id)">{{ c.name }}</button></div></div>
            <dl class="mc-dl"><dt>{{ tr('Dimensioni') }}</dt><dd>{{ d.item.width }} × {{ d.item.height }} px</dd><dt>{{ tr('Peso') }}</dt><dd>{{ formatSize(d.item.size_bytes) }}</dd>
              <dt>{{ tr('File originale') }}</dt><dd>{{ d.item.original_name }}</dd></dl>
            <div class="mc-form" *ngIf="d.item.state === 'ACTIVE' && artworks.length"><strong>{{ tr('Usa in un’opera') }}</strong>
              <select [(ngModel)]="targetArtwork" [attr.aria-label]="tr('Opera')"><option value="">{{ tr('Scegli un’opera') }}</option>
                <option *ngFor="let a of artworks" [value]="a.id">{{ a.code }} — {{ a.title }}</option></select>
              <div class="mc-row"><button type="button" (click)="useAsCover()" [disabled]="busy || !targetArtwork">{{ tr('Imposta come copertina') }}</button>
                <button type="button" (click)="addToGallery()" [disabled]="busy || !targetArtwork">{{ tr('Aggiungi alla galleria') }}</button></div></div>
            <div class="mc-row">
              <button type="button" class="danger" *ngIf="d.item.state === 'ACTIVE'" (click)="trash()" [disabled]="busy">{{ tr('Sposta nel cestino') }}</button>
              <button type="button" *ngIf="d.item.state === 'TRASHED'" (click)="untrash()" [disabled]="busy">{{ tr('Ripristina dal cestino') }}</button>
              <button type="button" class="danger" *ngIf="d.item.state === 'TRASHED'" (click)="purge()" [disabled]="busy">{{ tr('Elimina definitivamente') }}</button></div>
          </ng-container>

          <ng-container *ngIf="tab === 'usages' && !pickerMode">
            <p class="mc-hint" *ngIf="!d.usages.length">{{ tr('Questa immagine non è usata in nessuna scheda.') }}</p>
            <ul class="mc-list"><li *ngFor="let u of d.usages"><span>{{ usageLabel(u.kind) }}</span><strong>{{ u.label }}</strong></li></ul>
          </ng-container>

          <ng-container *ngIf="tab === 'versions' && !pickerMode">
            <ul class="mc-list"><li *ngFor="let r of d.revisions" [class.current]="r.id === d.item.current_revision_id">
              <span><strong>{{ tr('Versione') }} {{ r.number }}</strong> · {{ r.kind === 'ORIGINAL' ? tr('Originale') : tr('Modifica') }}<br><small>{{ r.width }} × {{ r.height }} px · {{ formatDate(r.created_at) }}</small></span>
              <span *ngIf="r.id === d.item.current_revision_id">{{ tr('In uso') }}</span>
              <button type="button" *ngIf="r.id !== d.item.current_revision_id" (click)="restoreRevision(r.id)" [disabled]="busy">{{ tr('Ripristina') }}</button></li></ul>
            <p class="mc-hint">{{ tr('Ripristinare una versione la rimette in uso ovunque l’immagine sia usata; le altre versioni restano conservate.') }}</p>
          </ng-container>

          <ng-container *ngIf="tab === 'edit' && !pickerMode">
            <rp-media-editor *ngIf="editSource" [src]="editSource.data_url" [imageWidth]="d.item.width" [imageHeight]="d.item.height"
              [language]="language" [previewFn]="previewEdit" (applied)="applyEdit($event)" (cancelled)="setTab('info')"></rp-media-editor>
            <p class="mc-hint" *ngIf="!editSource">{{ tr('Caricamento…') }}</p>
          </ng-container>

          <ng-container *ngIf="tab === 'export' && !pickerMode">
            <div class="mc-form">
              <button type="button" (click)="exportAs('current')" [disabled]="busy">{{ tr('Scarica la versione attuale') }}</button>
              <button type="button" (click)="exportAs('original')" [disabled]="busy">{{ tr('Scarica il file originale') }}</button>
              <strong>{{ tr('Esportazione personalizzata') }}</strong>
              <label>{{ tr('Formato') }}<select [(ngModel)]="exportOptions.format"><option value="jpeg">JPEG</option><option value="png">PNG</option></select></label>
              <label *ngIf="exportOptions.format === 'jpeg'">{{ tr('Qualità JPEG') }} ({{ exportOptions.quality }})<input type="range" min="40" max="100" [(ngModel)]="exportOptions.quality"></label>
              <label>{{ tr('Larghezza massima (px)') }}<input type="number" min="64" max="12000" [(ngModel)]="exportOptions.max_width"></label>
              <label>{{ tr('Altezza massima (px)') }}<input type="number" min="64" max="12000" [(ngModel)]="exportOptions.max_height"></label>
              <button type="button" class="primary" (click)="exportAs('custom')" [disabled]="busy">{{ tr('Esporta') }}</button>
            </div>
          </ng-container>
        </aside>
      </div>
    </div>`
})
export class MediaCenterComponent implements OnInit {
  @Input() language: AppLanguage = 'it';
  @Input() artworks: Artwork[] = [];
  @Input() pickerMode = false;
  /** Open this item as soon as the library is shown (used by "edit" buttons elsewhere). */
  @Input() set openId(value: string | null) { if (value && value !== this.requestedId) { this.requestedId = value; void this.select(value); } }
  @Output() picked = new EventEmitter<MediaItem>();
  @Output() changed = new EventEmitter<void>();

  view: View = 'all';
  items: MediaItem[] = [];
  total = 0;
  loading = false;
  busy = false;
  scanning = false;
  similarScanned = false;
  search = '';
  tags: MediaTag[] = [];
  collections: MediaCollection[] = [];
  activeTagId: string | null = null;
  activeCollectionId: string | null = null;
  newCollection = '';
  newTag = '';
  thumbs: Record<string, string> = {};
  selected: MediaDetail | null = null;
  preview: PreviewData | null = null;
  editSource: PreviewData | null = null;
  tab: Tab = 'info';
  meta = { title: '' as string | null, description: '' as string | null, alt_text: '' as string | null, credits: '' as string | null, copyright_status: 'UNKNOWN' as CopyrightStatus };
  selectedTags = new Set<string>();
  targetArtwork = '';
  exactGroups: DuplicateGroup[] = [];
  similar: SimilarPair[] = [];
  exportOptions = { format: 'jpeg' as 'jpeg' | 'png', quality: 90, max_width: null as number | null, max_height: null as number | null };
  error = '';
  message = '';
  private requestedId: string | null = null;
  private searchTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(private readonly change: ChangeDetectorRef) {}

  tr(key: string): string { return translate(key, this.language); }
  async ngOnInit(): Promise<void> { await this.reloadSidebar(); await this.load(true); }

  // ------------------------------------------------------------ helpers
  formatSize(bytes: number): string {
    return bytes >= 1048576 ? (bytes / 1048576).toFixed(1) + ' MiB' : Math.max(1, Math.round(bytes / 1024)) + ' KiB';
  }
  formatDate(value: string): string {
    const d = new Date(value);
    return Number.isNaN(d.getTime()) ? value : d.toLocaleString(this.language, { dateStyle: 'medium', timeStyle: 'short' });
  }
  usageLabel(kind: string): string {
    return this.tr({ ARTWORK_COVER: 'Copertina di', ARTWORK_GALLERY: 'Galleria di', COLLECTOR_PROFILE: 'Foto profilo di', COLLECTOR_PHOTO: 'Fotografia di' }[kind] ?? kind);
  }
  private fail(error: unknown): void { this.error = typeof error === 'string' ? error : error instanceof Error ? error.message : this.tr('Operazione non riuscita'); this.message = ''; }
  private ok(text: string): void { this.message = text; this.error = ''; }
  private async run<T>(work: () => Promise<T>): Promise<T | undefined> {
    if (this.busy) return undefined;
    this.busy = true; this.error = ''; this.message = '';
    try { return await work(); } catch (error) { this.fail(error); return undefined; }
    finally { this.busy = false; this.change.detectChanges(); }
  }

  // ------------------------------------------------------------ listing
  private async reloadSidebar(): Promise<void> {
    try { [this.tags, this.collections] = await Promise.all([api.mediaTags(), api.mediaCollections()]); } catch (e) { this.fail(e); }
  }
  async load(reset: boolean): Promise<void> {
    this.loading = reset; this.error = '';
    try {
      const page = await api.mediaList({ query: this.search, tagId: this.activeTagId, collectionId: this.activeCollectionId,
        state: this.view === 'trash' ? 'TRASHED' : 'ACTIVE', offset: reset ? 0 : this.items.length, limit: 60 });
      this.items = reset ? page.items : [...this.items, ...page.items];
      this.total = page.total;
    } catch (e) { this.fail(e); }
    this.loading = false;
    this.change.detectChanges();
    await this.loadThumbs(this.items);
  }
  async loadMore(): Promise<void> { await this.load(false); }
  private async loadThumbs(items: MediaItem[]): Promise<void> {
    const wanted = items.filter(i => !this.thumbs[i.current_revision_id]);
    for (let start = 0; start < wanted.length; start += 6) {
      await Promise.all(wanted.slice(start, start + 6).map(async item => {
        try { this.thumbs[item.current_revision_id] = await api.mediaThumbnail(item.id); } catch { /* shown without a thumbnail */ }
      }));
      this.change.detectChanges();
    }
  }
  onSearch(): void {
    if (this.searchTimer) clearTimeout(this.searchTimer);
    this.searchTimer = setTimeout(() => void this.load(true), 250);
  }
  async setView(view: View): Promise<void> {
    this.view = view; this.selected = null; this.preview = null;
    if (view === 'duplicates') { await this.loadExact(); return; }
    await this.load(true);
  }
  async toggleFilterTag(id: string): Promise<void> { this.activeTagId = this.activeTagId === id ? null : id; await this.load(true); }
  async toggleCollection(id: string): Promise<void> { this.activeCollectionId = this.activeCollectionId === id ? null : id; await this.load(true); }
  async createCollection(): Promise<void> {
    const name = this.newCollection.trim();
    if (!name) return;
    if (await this.run(() => api.mediaCreateCollection(name))) { this.newCollection = ''; await this.reloadSidebar(); }
  }
  async importFiles(): Promise<void> {
    if (this.busy) return;
    try {
      const selected = await open({ directory: false, multiple: true, title: this.tr('Importa immagini…'),
        filters: [{ name: 'JPEG / PNG / WebP', extensions: ['jpg', 'jpeg', 'png', 'webp'] }] });
      const paths = Array.isArray(selected) ? selected : typeof selected === 'string' ? [selected] : [];
      if (!paths.length) return;
      const result = await this.run(() => api.mediaImport(paths));
      if (!result) return;
      if (result.failed.length) this.error = result.failed.map(f => `${f.path}: ${f.message}`).join(' · ');
      else this.ok(this.tr('Immagini importate') + ': ' + result.imported.length);
      this.view = 'all';
      await this.load(true);
      this.changed.emit();
    } catch (e) { this.fail(e); }
  }

  // ------------------------------------------------------------ inspector
  async select(id: string): Promise<void> {
    try {
      const detail = await api.mediaDetail(id);
      this.selected = detail; this.preview = null; this.editSource = null; this.tab = 'info'; this.targetArtwork = '';
      this.syncForm(detail);
      this.change.detectChanges();
      this.preview = await api.mediaPreview(id, 900);
    } catch (e) { this.fail(e); }
    this.change.detectChanges();
  }
  private syncForm(detail: MediaDetail): void {
    const i = detail.item;
    this.meta = { title: i.title ?? '', description: i.description ?? '', alt_text: i.alt_text ?? '', credits: i.credits ?? '', copyright_status: i.copyright_status };
    this.selectedTags = new Set(i.tags.map(t => t.id));
  }
  private async replace(detail: MediaDetail, refreshPreview = false): Promise<void> {
    this.selected = detail; this.syncForm(detail);
    const index = this.items.findIndex(i => i.id === detail.item.id);
    if (index >= 0) this.items[index] = detail.item;
    if (refreshPreview) { this.preview = await api.mediaPreview(detail.item.id, 900); this.editSource = null; }
    await this.loadThumbs([detail.item]);
    this.change.detectChanges();
  }
  closeInspector(): void { this.selected = null; this.preview = null; this.editSource = null; }
  async setTab(tab: Tab): Promise<void> {
    this.tab = tab; this.error = '';
    if (tab === 'edit' && this.selected && !this.editSource) {
      try { this.editSource = await api.mediaPreview(this.selected.item.id, 1400); } catch (e) { this.fail(e); }
      this.change.detectChanges();
    }
  }
  async saveMeta(): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => api.mediaUpdate(d.item.id, { title: this.meta.title || null, description: this.meta.description || null,
      alt_text: this.meta.alt_text || null, credits: this.meta.credits || null, copyright_status: this.meta.copyright_status }));
    if (detail) { await this.replace(detail); this.ok(this.tr('Informazioni salvate')); }
  }
  toggleTag(id: string): void { if (!this.selectedTags.delete(id)) this.selectedTags.add(id); this.selectedTags = new Set(this.selectedTags); }
  async createTag(): Promise<void> {
    const name = this.newTag.trim(); if (!name) return;
    const tag = await this.run(() => api.mediaCreateTag(name));
    if (tag) { this.newTag = ''; await this.reloadSidebar(); this.selectedTags = new Set(this.selectedTags).add(tag.id); }
  }
  async saveTags(): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => api.mediaSetTags(d.item.id, [...this.selectedTags]));
    if (detail) { await this.replace(detail); this.ok(this.tr('Tag salvati')); }
  }
  async toggleMembership(collectionId: string): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => d.collection_ids.includes(collectionId) ? api.mediaCollectionRemove(collectionId, d.item.id) : api.mediaCollectionAdd(collectionId, d.item.id));
    if (detail) { await this.replace(detail); await this.reloadSidebar(); }
  }
  async useAsCover(): Promise<void> {
    const d = this.selected; if (!d || !this.targetArtwork) return;
    if (await this.run(() => api.artworkSetCoverFromMedia(this.targetArtwork, d.item.id)) !== undefined) {
      this.ok(this.tr('Copertina aggiornata')); this.changed.emit(); await this.select(d.item.id);
    }
  }
  async addToGallery(): Promise<void> {
    const d = this.selected; if (!d || !this.targetArtwork) return;
    if (await this.run(() => api.artworkAddGalleryFromMedia(this.targetArtwork, d.item.id, null)) !== undefined) {
      this.ok(this.tr('Fotografia aggiunta alla galleria')); this.changed.emit(); await this.select(d.item.id);
    }
  }

  // ------------------------------------------------------------ editing and versions
  readonly previewEdit = (operations: EditOperation[]): Promise<PreviewData> => api.mediaPreview(this.selected!.item.id, 1400, operations);
  async applyEdit(operations: EditOperation[]): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => api.mediaApplyEdit(d.item.id, operations));
    if (detail) {
      await this.replace(detail, true); this.tab = 'versions'; this.ok(this.tr('Nuova versione creata')); this.changed.emit();
    }
  }
  async restoreRevision(revisionId: string): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => api.mediaRestoreRevision(d.item.id, revisionId));
    if (detail) { await this.replace(detail, true); this.ok(this.tr('Versione ripristinata')); this.changed.emit(); }
  }

  // ------------------------------------------------------------ export, trash
  async exportAs(kind: 'current' | 'original' | 'custom'): Promise<void> {
    const d = this.selected; if (!d || this.busy) return;
    const extension = kind === 'custom' ? (this.exportOptions.format === 'png' ? 'png' : 'jpg') : (kind === 'original' ? d.revisions[d.revisions.length - 1].image_path : d.item.image_path).split('.').pop() ?? 'jpg';
    try {
      const base = (d.item.title || d.item.original_name).replace(/\.[^.]+$/, '').replace(/[\\/:*?"<>|]/g, '_');
      const destination = await save({ title: this.tr('Esporta'), defaultPath: `${base}${kind === 'original' ? '-originale' : ''}.${extension}`,
        filters: [{ name: extension.toUpperCase(), extensions: [extension] }] });
      if (typeof destination !== 'string') return;
      const options = kind === 'custom' ? { format: this.exportOptions.format, quality: Number(this.exportOptions.quality),
        max_width: this.exportOptions.max_width ? Number(this.exportOptions.max_width) : null,
        max_height: this.exportOptions.max_height ? Number(this.exportOptions.max_height) : null } : null;
      if (await this.run(() => api.mediaExport(d.item.id, kind, options, destination)) !== undefined) this.ok(this.tr('Immagine esportata'));
    } catch (e) { this.fail(e); }
  }
  async trash(): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => api.mediaTrash(d.item.id));
    if (detail) { this.closeInspector(); this.ok(this.tr('Immagine spostata nel cestino')); await this.load(true); }
  }
  async untrash(): Promise<void> {
    const d = this.selected; if (!d) return;
    const detail = await this.run(() => api.mediaUntrash(d.item.id));
    if (detail) { this.closeInspector(); this.ok(this.tr('Immagine ripristinata')); await this.load(true); }
  }
  async purge(): Promise<void> {
    const d = this.selected; if (!d) return;
    if (!(await confirmed(this.tr('Eliminare definitivamente questa immagine e tutte le sue versioni? L’operazione non è reversibile.')))) return;
    if (await this.run(() => api.mediaPurge(d.item.id)) !== undefined) { this.closeInspector(); this.ok(this.tr('Immagine eliminata')); await this.load(true); }
  }

  // ------------------------------------------------------------ duplicates
  async loadExact(): Promise<void> {
    this.loading = true;
    try { this.exactGroups = await api.mediaExactDuplicates(); await this.loadThumbs(this.exactGroups.flatMap(g => g.items)); } catch (e) { this.fail(e); }
    this.loading = false; this.change.detectChanges();
  }
  async scanSimilar(): Promise<void> {
    if (this.scanning) return;
    this.scanning = true; this.error = '';
    try { this.similar = await api.mediaSimilar(); this.similarScanned = true; await this.loadThumbs(this.similar.flatMap(p => [p.first, p.second])); }
    catch (e) { this.fail(e); }
    this.scanning = false; this.change.detectChanges();
  }
}
