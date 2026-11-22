import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, ElementRef, EventEmitter, HostListener, Input, OnChanges, OnDestroy, Output, SimpleChanges, ViewChild, ViewEncapsulation } from '@angular/core';
import { api } from './api';
import { AppLanguage, translate } from './i18n';
import { PHOTO_KINDS } from './models';
import type { Artwork } from './models';
import { zoomGeometry, type ZoomGeometry } from './viewer-zoom';

interface ViewerImage { url: string; title: string; caption: string | null; }
const STATUS: Record<string, string> = { available: 'Disponibile', reserved: 'Riservata', sold: 'Venduta', archived: 'In archivio' };
/** Pictures kept in memory while browsing (each one is a full-size data URL). */
const CACHE_LIMIT = 6;

/**
 * Full-screen viewer of the archive: browse the artworks one by one (arrow keys or buttons), see every
 * photograph of the current work with a magnifier, and read its details. Layout and behaviour follow
 * the preview of the Gestionale Opere.
 */
@Component({
  selector: 'rp-artwork-viewer',
  standalone: true,
  imports: [CommonModule],
  encapsulation: ViewEncapsulation.None,
  styles: [`
    .av-overlay{position:fixed;inset:0;z-index:1000;display:flex;align-items:stretch;justify-content:center;padding:24px;background:rgba(3,7,18,.86);backdrop-filter:blur(10px)}
    .av-shell{position:relative;width:min(1480px,100%);height:calc(100vh - 48px);border-radius:24px;overflow:auto;background:#0f172a;color:#f8fafc;box-shadow:0 24px 80px rgba(0,0,0,.45)}
    .av-layout{display:grid;grid-template-columns:minmax(0,1.55fr) minmax(320px,400px);min-height:100%}
    .av-main{position:relative;min-width:0;display:flex;flex-direction:column}
    .av-main>.av-media{flex:1}
    .av-media{display:flex;flex-direction:column;padding:28px 32px 28px;background:radial-gradient(circle at top,rgba(148,163,184,.18),transparent 40%),#020617;overflow:auto;min-width:0}
    .av-counter{align-self:flex-start;font-size:12px;letter-spacing:.12em;text-transform:uppercase;color:#94a3b8;margin-bottom:10px}
    .av-frame{position:relative;display:flex;align-items:center;justify-content:center;flex:1;min-height:52vh;padding:8px 64px}
    .av-stage{position:relative;display:flex;align-items:center;justify-content:center;max-width:100%}
    .av-image{display:block;max-width:100%;max-height:64vh;border-radius:14px;object-fit:contain;box-shadow:0 24px 60px rgba(0,0,0,.45)}
    .av-lens{position:absolute;border-radius:999px;border:1px solid rgba(255,255,255,.75);background:rgba(255,255,255,.18);box-shadow:0 8px 30px rgba(0,0,0,.35);pointer-events:none}
    .av-pane{position:absolute;top:8px;right:8px;width:340px;height:340px;border-radius:18px;border:1px solid rgba(148,163,184,.28);background-color:rgba(15,23,42,.72);background-repeat:no-repeat;pointer-events:none;z-index:5}
    .av-loading{display:flex;align-items:center;justify-content:center;min-height:50vh;color:#cbd5e1;text-align:center}
    .av-caption{margin-top:14px;color:#e5e7eb;font-size:14px}
    .av-caption small{display:block;color:#94a3b8;margin-top:2px}
    .av-thumbs{display:flex;gap:10px;margin-top:14px;padding-bottom:6px;overflow-x:auto}
    .av-thumb{flex:0 0 auto;width:68px;height:68px;padding:0;border-radius:12px;border:2px solid transparent;overflow:hidden;background:#111827;cursor:pointer}
    .av-thumb img{width:100%;height:100%;object-fit:cover}
    .av-thumb.active{border-color:#f8fafc}
    .av-side{display:flex;flex-direction:column;gap:14px;padding:32px 28px;background:linear-gradient(180deg,rgba(30,41,59,.96) 0%,#0f172a 100%);overflow:auto;min-width:0}
    .av-eyebrow{font-size:11px;letter-spacing:.18em;text-transform:uppercase;color:#94a3b8}
    .av-title{margin:0;font:500 30px/1.1 Georgia,'Times New Roman',serif;overflow-wrap:anywhere}
    .av-sub{color:#cbd5e1;font-size:13px}
    .av-facts{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
    .av-fact{padding:11px 12px;border-radius:14px;background:rgba(15,23,42,.85);border:1px solid rgba(148,163,184,.18);font-size:13px;min-width:0;overflow-wrap:anywhere}
    .av-fact.wide{grid-column:1/-1}
    .av-fact b{display:block;margin-bottom:4px;font-size:10px;font-weight:600;letter-spacing:.08em;text-transform:uppercase;color:#94a3b8}
    .av-tags{display:flex;gap:6px;flex-wrap:wrap}
    .av-tag{padding:4px 10px;border-radius:999px;background:rgba(148,163,184,.16);color:#e2e8f0;font-size:11px}
    .av-desc{margin:0;color:#e2e8f0;line-height:1.55;font-size:13px;white-space:pre-wrap}
    .av-actions{display:flex;flex-direction:column;gap:10px;margin-top:auto}
    .av-actions button{font:inherit;font-size:13px;padding:11px 14px;border-radius:12px;border:1px solid rgba(248,250,252,.55);background:rgba(15,23,42,.3);color:#f8fafc;cursor:pointer}
    .av-actions button.primary{background:#f8fafc;color:#0f172a;border-color:#f8fafc;font-weight:600}
    .av-actions button:hover{background:rgba(248,250,252,.14)}
    .av-actions button.primary:hover{background:#e2e8f0}
    .av-hint{font-size:12px;color:#94a3b8}
    .av-round{position:absolute;z-index:4;width:42px;height:42px;border-radius:50%;border:0;background:rgba(15,23,42,.72);color:#f8fafc;font-size:22px;line-height:1;cursor:pointer;display:flex;align-items:center;justify-content:center}
    .av-round:hover:not(:disabled){background:rgba(51,65,85,.92)}
    .av-round:disabled{opacity:.3;cursor:default}
    .av-round:focus-visible,.av-thumb:focus-visible,.av-actions button:focus-visible{outline:2px solid #fff;outline-offset:2px}
    .av-close{top:16px;right:16px}
    .av-prev,.av-next{top:50%;transform:translateY(-50%)}
    .av-prev{left:16px}.av-next{right:16px}
    .av-imgnav{display:flex;align-items:center;gap:10px;margin-top:12px;color:#cbd5e1;font-size:12px}
    .av-imgnav button{width:30px;height:30px;border-radius:50%;border:1px solid rgba(148,163,184,.35);background:rgba(15,23,42,.72);color:#f8fafc;cursor:pointer;font-size:16px;line-height:1}
    .av-imgnav button:disabled{opacity:.3;cursor:default}
    @media (hover:none),(pointer:coarse){.av-lens,.av-pane{display:none!important}}
    @media (max-width:1024px){
      .av-overlay{padding:0}.av-shell{height:100dvh;border-radius:0}
      .av-layout{grid-template-columns:1fr}
      .av-media{padding:64px 14px 14px}.av-frame{padding:8px 44px;min-height:40vh}.av-image{max-height:56vh}
      .av-side{padding:20px}
    }
  `],
  template: `
    <div class="av-overlay" (click)="close()">
      <div class="av-shell" role="dialog" aria-modal="true" [attr.aria-label]="tr('Visualizzatore opere')" (click)="$event.stopPropagation()">
        <button #closeButton type="button" class="av-round av-close" (click)="close()" [attr.aria-label]="tr('Chiudi')" [title]="tr('Chiudi') + ' (Esc)'">×</button>
        <div class="av-layout" *ngIf="current as a">
          <div class="av-main">
          <button type="button" class="av-round av-prev" [disabled]="index <= 0" (click)="go(-1)" [attr.aria-label]="tr('Opera precedente')" [title]="tr('Opera precedente') + ' (←)'">‹</button>
          <button type="button" class="av-round av-next" [disabled]="index >= artworks.length - 1" (click)="go(1)" [attr.aria-label]="tr('Opera successiva')" [title]="tr('Opera successiva') + ' (→)'">›</button>
          <div class="av-media">
            <div class="av-counter" aria-live="polite">{{ tr('Opera') }} {{ index + 1 }} / {{ artworks.length }}</div>
            <div class="av-frame">
              <div class="av-loading" *ngIf="loading">{{ tr('Caricamento immagini…') }}</div>
              <ng-container *ngIf="!loading && image as img">
                <div class="av-stage" (mouseenter)="zoomMove($event, pic)" (mousemove)="zoomMove($event, pic)" (mouseleave)="zoomStop()">
                  <img #pic class="av-image" [src]="img.url" [alt]="img.title" decoding="async" draggable="false">
                  <div class="av-lens" *ngIf="zoom" [ngStyle]="zoom.lens"></div>
                </div>
                <div class="av-pane" *ngIf="zoom" [ngStyle]="zoom.pane" [style.backgroundImage]="'url(&quot;' + img.url + '&quot;)'"></div>
              </ng-container>
              <div class="av-loading" *ngIf="!loading && !image">{{ error || tr('Nessuna fotografia per questa opera.') }}</div>
            </div>
            <div class="av-caption" *ngIf="image as img">{{ img.title }}<small *ngIf="img.caption">{{ img.caption }}</small></div>
            <div class="av-imgnav" *ngIf="images.length > 1">
              <button type="button" [disabled]="imageIndex === 0" (click)="goImage(-1)" [attr.aria-label]="tr('Immagine precedente')" [title]="tr('Immagine precedente') + ' (↑)'">‹</button>
              <span>{{ imageIndex + 1 }} / {{ images.length }}</span>
              <button type="button" [disabled]="imageIndex === images.length - 1" (click)="goImage(1)" [attr.aria-label]="tr('Immagine successiva')" [title]="tr('Immagine successiva') + ' (↓)'">›</button>
            </div>
            <div class="av-thumbs" *ngIf="images.length > 1">
              <button type="button" class="av-thumb" *ngFor="let t of images; let i = index" [class.active]="i === imageIndex" (click)="selectImage(i)" [attr.aria-label]="tr('Apri la fotografia') + ' ' + (i + 1)"><img [src]="t.url" alt="" loading="lazy" decoding="async"></button>
            </div>
          </div>
          </div>
          <aside class="av-side">
            <div class="av-eyebrow">{{ a.code }}</div>
            <h2 class="av-title">{{ a.title }}</h2>
            <div class="av-sub">{{ subtitle(a) }}</div>
            <div class="av-tags" *ngIf="a.signed || a.final_varnish || a.framed">
              <span class="av-tag" *ngIf="a.signed">{{ tr('Opera firmata') }}</span>
              <span class="av-tag" *ngIf="a.final_varnish">{{ tr('Vernice finale applicata') }}</span>
              <span class="av-tag" *ngIf="a.framed">{{ tr('Incorniciata') }}</span>
            </div>
            <div class="av-facts">
              <div class="av-fact"><b>{{ tr('Stato') }}</b>{{ tr(statusLabel(a)) }}</div>
              <div class="av-fact"><b>{{ tr('Anno') }}</b>{{ a.year || '—' }}</div>
              <div class="av-fact wide"><b>{{ tr('Tecnica') }} / {{ tr('Supporto') }}</b>{{ techniqueLine(a) }}</div>
              <div class="av-fact wide"><b>{{ tr('Misure') }}</b>{{ dimensions(a) }}</div>
              <div class="av-fact wide" *ngIf="prices[a.id]"><b>{{ tr('Prezzo') }}</b>{{ prices[a.id] }}</div>
              <div class="av-fact wide" *ngIf="buyers[a.id]"><b>{{ tr('Venduta a') }}</b>{{ buyers[a.id] }}</div>
            </div>
            <p class="av-desc" *ngIf="a.description">{{ a.description }}</p>
            <div class="av-actions">
              <button type="button" class="primary" (click)="edit.emit(a)">{{ tr('Modifica opera') }}</button>
              <button type="button" (click)="gallery.emit(a)">{{ tr('Gestisci le fotografie') }}</button>
            </div>
            <div class="av-hint">{{ tr('← → scorrono le opere, ↑ ↓ le fotografie, Esc chiude.') }}</div>
          </aside>
        </div>
      </div>
    </div>`
})
export class ArtworkViewerComponent implements OnChanges, OnDestroy {
  @Input() artworks: Artwork[] = [];
  @Input() startIndex = 0;
  @Input() language: AppLanguage = 'it';
  /** Name of the collector who bought each sold artwork, by artwork id. */
  @Input() buyers: Record<string, string> = {};
  /** Price of each artwork, already formatted, by artwork id. */
  @Input() prices: Record<string, string> = {};
  @Output() closed = new EventEmitter<void>();
  @Output() edit = new EventEmitter<Artwork>();
  @Output() gallery = new EventEmitter<Artwork>();
  @ViewChild('closeButton') closeButton?: ElementRef<HTMLButtonElement>;

  index = 0;
  images: ViewerImage[] = [];
  imageIndex = 0;
  loading = false;
  error = '';
  zoom: ZoomGeometry | null = null;
  private cache = new Map<string, ViewerImage[]>();
  private token = 0;
  private opener: Element | null = typeof document !== 'undefined' ? document.activeElement : null;

  constructor(private readonly change: ChangeDetectorRef) {
    // The page behind the viewer must not scroll.
    if (typeof document !== 'undefined') document.body.style.overflow = 'hidden';
  }

  get current(): Artwork | null { return this.artworks[this.index] ?? null; }
  get image(): ViewerImage | null { return this.images[this.imageIndex] ?? null; }
  tr(text: string): string { return translate(text, this.language); }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['startIndex'] || changes['artworks']) {
      const wanted = Math.min(Math.max(this.startIndex, 0), Math.max(this.artworks.length - 1, 0));
      if (changes['startIndex'] || this.index >= this.artworks.length) this.index = wanted;
      void this.load();
      queueMicrotask(() => this.closeButton?.nativeElement.focus());
    }
  }
  ngOnDestroy(): void {
    this.token++;
    if (typeof document !== 'undefined') document.body.style.overflow = '';
    (this.opener as HTMLElement | null)?.focus?.();
  }

  close(): void { this.closed.emit(); }
  go(delta: number): void {
    const next = this.index + delta;
    if (next < 0 || next >= this.artworks.length) return;
    this.index = next;
    this.imageIndex = 0;
    this.zoom = null;
    void this.load();
  }
  goImage(delta: number): void { this.selectImage(this.imageIndex + delta); }
  selectImage(i: number): void {
    if (i < 0 || i >= this.images.length) return;
    this.imageIndex = i;
    this.zoom = null;
  }

  zoomMove(event: MouseEvent, element: HTMLImageElement): void {
    const rect = element.getBoundingClientRect();
    this.zoom = zoomGeometry(event.clientX - rect.left, event.clientY - rect.top, rect.width, rect.height, element.naturalWidth, element.naturalHeight);
  }
  zoomStop(): void { this.zoom = null; }

  private async load(): Promise<void> {
    const artwork = this.current;
    const token = ++this.token;
    this.error = '';
    if (!artwork) { this.images = []; return; }
    const cached = this.cache.get(artwork.id);
    if (cached) {
      this.images = cached; this.loading = false;
      this.change.detectChanges();
      return;
    }
    this.images = []; this.loading = true;
    this.change.detectChanges();
    const found: ViewerImage[] = [];
    try {
      if (artwork.image_path) {
        const cover = await api.artworkImage(artwork.id);
        if (cover) found.push({ url: cover, title: this.tr('Copertina'), caption: null });
      }
      if (token !== this.token) return;
      for (const item of await api.listArtworkGallery(artwork.id)) {
        if (token !== this.token) return;
        try {
          const kind = PHOTO_KINDS.find(k => k.code === item.kind);
          found.push({ url: await api.artworkGalleryImageData(item.id), title: kind ? this.tr(kind.label) : this.tr('Fotografia'), caption: item.caption });
        } catch { /* a missing or oversized file must not hide the others */ }
      }
    } catch (error) {
      if (token === this.token) this.error = String(error);
    }
    if (token !== this.token) return;
    this.cache.set(artwork.id, found);
    while (this.cache.size > CACHE_LIMIT) this.cache.delete(this.cache.keys().next().value as string);
    this.images = found; this.imageIndex = 0; this.loading = false;
    this.change.detectChanges();
  }

  statusLabel(a: Artwork): string { return STATUS[a.status] ?? a.status; }
  subtitle(a: Artwork): string { return [a.year, this.techniqueLine(a) !== '—' ? this.techniqueLine(a) : ''].filter(Boolean).join(' · '); }
  techniqueLine(a: Artwork): string {
    const technique = a.technique ? this.tr(a.technique) : '';
    const support = a.support ? this.tr(a.support) : '';
    return [technique, support].filter(Boolean).join(' · ') || '—';
  }
  dimensions(a: Artwork): string {
    const parts = [a.height_cm, a.width_cm, a.depth_cm].filter(v => v !== null);
    return parts.length ? parts.join(' × ') + ' cm' : '—';
  }

  @HostListener('window:keydown', ['$event'])
  onKey(event: KeyboardEvent): void {
    const target = event.target as HTMLElement | null;
    if (target && ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) return;
    const actions: Record<string, () => void> = {
      Escape: () => this.close(), ArrowLeft: () => this.go(-1), ArrowRight: () => this.go(1),
      ArrowUp: () => this.goImage(-1), ArrowDown: () => this.goImage(1),
    };
    const action = actions[event.key];
    if (action) { event.preventDefault(); action(); this.change.detectChanges(); }
  }
}
