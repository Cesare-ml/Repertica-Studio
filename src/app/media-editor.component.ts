import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, ElementRef, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { AppLanguage, translate } from './i18n';
import {
  adjustmentFilter, adjustmentTransform, buildOperations, cropChanged, cropForRatio, defaultCrop, defaultPerspective,
  emptyEditState, moveCropCorner, perspectiveChanged, clamp01, type EditorTool, type EditState, type PerspectiveMode
} from './image-edit';
import type { EditOperation, PreviewData } from './models';

/**
 * Image editor: crop, perspective correction and light / colour adjustments on a picture.
 * It only collects what the user wants; the native engine applies it (and creates a new version).
 */
@Component({
  selector: 'rp-media-editor',
  standalone: true,
  imports: [CommonModule, FormsModule],
  encapsulation: ViewEncapsulation.None,
  styles: [`
    .me{display:grid;gap:12px}
    .me-tools{display:flex;gap:6px;flex-wrap:wrap}
    .me-tools button,.me-presets button,.me-actions button{font:inherit;font-size:12px;padding:7px 12px;border:1px solid #d9ddd5;border-radius:8px;background:#fff;cursor:pointer;color:#33433b}
    .me-tools button.active{background:#263f3c;border-color:#263f3c;color:#fff}
    .me-stage{display:grid;place-items:center;background:#1f2623;border-radius:10px;padding:12px;min-height:220px}
    .me-frame{position:relative;max-width:100%;line-height:0;user-select:none;touch-action:none}
    .me-frame img{display:block;max-width:100%;max-height:52vh;pointer-events:none}
    .me-crop{position:absolute;border:2px solid #fff;box-shadow:0 0 0 9999px rgba(0,0,0,.45);pointer-events:none}
    .me-handle{position:absolute;width:22px;height:22px;margin:-11px 0 0 -11px;border-radius:50%;border:2px solid #fff;background:#3f7d68;cursor:grab;padding:0;color:#fff;font-size:11px;font-weight:700;display:grid;place-items:center;touch-action:none}
    .me-handle:active{cursor:grabbing}
    .me-svg{position:absolute;inset:0;width:100%;height:100%;pointer-events:none}
    .me-svg polygon{fill:rgba(63,125,104,.18);stroke:#fff;stroke-width:.5;vector-effect:non-scaling-stroke}
    .me-svg line{stroke:rgba(255,255,255,.55);stroke-width:.3;vector-effect:non-scaling-stroke}
    .me-badge{position:absolute;top:8px;left:8px;background:#000b;color:#fff;font-size:12px;padding:5px 10px;border-radius:99px;display:flex;gap:8px;align-items:center;line-height:1.2}
    .me-badge button{border:0;background:transparent;color:#fff;text-decoration:underline;cursor:pointer;font-size:12px}
    .me-hint{font-size:12px;color:#66756d;margin:0}
    .me-presets{display:flex;gap:6px;flex-wrap:wrap}
    .me-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px}
    .me-grid label{display:grid;gap:4px;font-size:12px;font-weight:600;color:#4d5f56}
    .me-grid input[type=range]{width:100%}
    .me-grid select,.me-grid input[type=number]{font:inherit;padding:7px 9px;border:1px solid #d9ddd5;border-radius:8px;background:#fff}
    .me-actions{display:flex;gap:8px;justify-content:flex-end;flex-wrap:wrap}
    .me-actions .primary{background:#263f3c;color:#fff;border-color:#263f3c}
    .me-error{color:#a3382c;font-size:12px;margin:0}
  `],
  template: `
    <div class="me">
      <div class="me-tools" role="tablist">
        <button type="button" [class.active]="tool === 'crop'" (click)="setTool('crop')">{{ tr('Ritaglia') }}</button>
        <button type="button" [class.active]="tool === 'perspective'" (click)="setTool('perspective')">{{ tr('Correggi prospettiva') }}</button>
        <button type="button" [class.active]="tool === 'adjust'" (click)="setTool('adjust')">{{ tr('Luce e colore') }}</button>
      </div>

      <div class="me-stage">
        <div class="me-frame" #frame>
          <img [src]="resultPreview?.data_url || src" [alt]="tr('Anteprima')" draggable="false"
            [style.filter]="tool === 'adjust' && !resultPreview ? filter : null"
            [style.transform]="tool === 'adjust' && !resultPreview ? transform : null">
          <ng-container *ngIf="!resultPreview">
            <ng-container *ngIf="tool === 'crop'">
              <div class="me-crop" [style.left.%]="state.crop.x * 100" [style.top.%]="state.crop.y * 100"
                [style.width.%]="state.crop.width * 100" [style.height.%]="state.crop.height * 100"></div>
              <button type="button" class="me-handle" *ngFor="let corner of corners"
                [style.left.%]="(corner === 0 || corner === 3 ? state.crop.x : state.crop.x + state.crop.width) * 100"
                [style.top.%]="(corner < 2 ? state.crop.y : state.crop.y + state.crop.height) * 100"
                [attr.aria-label]="tr('Angolo di ritaglio') + ' ' + (corner + 1)"
                (pointerdown)="startDrag($event, 'crop', corner)" (pointermove)="moveDrag($event)" (pointerup)="stopDrag($event)" (pointercancel)="stopDrag($event)"></button>
            </ng-container>
            <ng-container *ngIf="tool === 'perspective'">
              <svg class="me-svg" viewBox="0 0 100 100" preserveAspectRatio="none">
                <polygon [attr.points]="polygon"></polygon>
                <line *ngFor="let l of gridLines" [attr.x1]="l.x1" [attr.y1]="l.y1" [attr.x2]="l.x2" [attr.y2]="l.y2"></line>
              </svg>
              <button type="button" class="me-handle" *ngFor="let p of state.perspective; let i = index"
                [style.left.%]="p.x * 100" [style.top.%]="p.y * 100" [attr.aria-label]="tr('Punto') + ' ' + (i + 1)"
                (pointerdown)="startDrag($event, 'perspective', i)" (pointermove)="moveDrag($event)" (pointerup)="stopDrag($event)" (pointercancel)="stopDrag($event)">{{ i + 1 }}</button>
            </ng-container>
          </ng-container>
          <div class="me-badge" *ngIf="resultPreview"><span>{{ tr('Anteprima del risultato') }}</span>
            <button type="button" (click)="resultPreview = null">{{ tr('Torna alla modifica') }}</button></div>
        </div>
      </div>

      <ng-container *ngIf="tool === 'crop'">
        <p class="me-hint">{{ tr('Trascina gli angoli per scegliere l’area da conservare.') }}</p>
        <div class="me-presets">
          <button type="button" (click)="state.crop = defaultCropBox()">{{ tr('Libero') }}</button>
          <button type="button" (click)="ratio(1)">1:1</button><button type="button" (click)="ratio(4 / 5)">4:5</button>
          <button type="button" (click)="ratio(3 / 2)">3:2</button><button type="button" (click)="ratio(16 / 9)">16:9</button>
          <button type="button" (click)="state.crop = defaultCropBox()">{{ tr('Ripristina') }}</button>
        </div>
      </ng-container>

      <ng-container *ngIf="tool === 'perspective'">
        <p class="me-hint">{{ tr('Porta i quattro punti sugli angoli dell’opera: alto a sinistra, alto a destra, basso a destra, basso a sinistra. L’immagine verrà raddrizzata.') }}</p>
        <div class="me-grid">
          <label>{{ tr('Proporzioni del risultato') }}
            <select [(ngModel)]="state.perspectiveMode">
              <option value="auto">{{ tr('Automatiche') }}</option><option value="custom">{{ tr('Misure dell’opera') }}</option>
              <option value="1:1">1:1</option><option value="4:5">4:5</option><option value="3:2">3:2</option><option value="16:9">16:9</option></select></label>
          <ng-container *ngIf="state.perspectiveMode === 'custom'">
            <label>{{ tr('Larghezza opera') }}<input type="number" min="0.1" step="0.1" [(ngModel)]="state.customWidth"></label>
            <label>{{ tr('Altezza opera') }}<input type="number" min="0.1" step="0.1" [(ngModel)]="state.customHeight"></label>
          </ng-container>
        </div>
        <div class="me-presets"><button type="button" (click)="resetPerspective()">{{ tr('Ripristina') }}</button></div>
      </ng-container>

      <div class="me-grid" *ngIf="tool === 'adjust'">
        <label>{{ tr('Rotazione') }}
          <select [(ngModel)]="state.rotation"><option [ngValue]="0">{{ tr('Nessuna') }}</option><option [ngValue]="90">90°</option><option [ngValue]="180">180°</option><option [ngValue]="270">270°</option></select></label>
        <label><span><input type="checkbox" [(ngModel)]="state.flipHorizontal"> {{ tr('Specchia in orizzontale') }}</span></label>
        <label><span><input type="checkbox" [(ngModel)]="state.flipVertical"> {{ tr('Specchia in verticale') }}</span></label>
        <label>{{ tr('Esposizione') }} ({{ state.exposure }})<input type="range" min="-80" max="100" [(ngModel)]="state.exposure"></label>
        <label>{{ tr('Contrasto') }} ({{ state.contrast }})<input type="range" min="-80" max="100" [(ngModel)]="state.contrast"></label>
        <label>{{ tr('Saturazione') }} ({{ state.saturation }})<input type="range" min="-100" max="100" [(ngModel)]="state.saturation"></label>
      </div>

      <p class="me-hint">{{ tr('L’immagine originale resta sempre conservata: ogni modifica crea una nuova versione.') }}</p>
      <p class="me-error" *ngIf="error">{{ tr(error) }}</p>
      <div class="me-actions">
        <button type="button" (click)="cancelled.emit()">{{ tr('Annulla') }}</button>
        <button type="button" (click)="preview()" [disabled]="busy || !previewFn">{{ busy ? tr('Elaborazione…') : tr('Anteprima') }}</button>
        <button type="button" class="primary" (click)="apply()" [disabled]="busy">{{ tr('Applica modifiche') }}</button>
      </div>
    </div>`
})
export class MediaEditorComponent implements OnChanges {
  /** The picture to edit, as a data URL (a reduced copy is enough). */
  @Input({ required: true }) src!: string;
  /** Pixel size of the full picture (used for aspect ratios). */
  @Input() imageWidth = 1;
  @Input() imageHeight = 1;
  @Input() language: AppLanguage = 'it';
  /** Renders the result of the edits for the preview button. */
  @Input() previewFn: ((operations: EditOperation[]) => Promise<PreviewData>) | null = null;
  @Output() applied = new EventEmitter<EditOperation[]>();
  @Output() cancelled = new EventEmitter<void>();
  @ViewChild('frame') frame?: ElementRef<HTMLElement>;

  tool: EditorTool = 'crop';
  state: EditState = emptyEditState();
  readonly corners = [0, 1, 2, 3];
  resultPreview: PreviewData | null = null;
  error = '';
  busy = false;
  private drag: { kind: 'crop' | 'perspective'; index: number } | null = null;

  constructor(private readonly change: ChangeDetectorRef) {}

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['src']) { this.state = emptyEditState(); this.resultPreview = null; this.error = ''; this.tool = 'crop'; }
  }

  tr(key: string): string { return translate(key, this.language); }
  defaultCropBox() { return defaultCrop(); }
  get filter(): string { return adjustmentFilter(this.state); }
  get transform(): string { return adjustmentTransform(this.state); }
  setTool(tool: EditorTool): void { this.tool = tool; this.resultPreview = null; }
  ratio(value: number): void { this.state.crop = cropForRatio(value, this.imageWidth, this.imageHeight); }
  resetPerspective(): void { this.state.perspective = defaultPerspective(); this.state.perspectiveMode = 'auto'; }

  get polygon(): string { return this.state.perspective.map(p => `${p.x * 100},${p.y * 100}`).join(' '); }
  get gridLines(): { x1: number; y1: number; x2: number; y2: number }[] {
    const [a, b, c, d] = this.state.perspective;
    const mix = (p: { x: number; y: number }, q: { x: number; y: number }, t: number) => ({ x: (p.x + (q.x - p.x) * t) * 100, y: (p.y + (q.y - p.y) * t) * 100 });
    return [1 / 3, 2 / 3].flatMap(t => {
      const top = mix(a, b, t), bottom = mix(d, c, t), left = mix(a, d, t), right = mix(b, c, t);
      return [{ x1: top.x, y1: top.y, x2: bottom.x, y2: bottom.y }, { x1: left.x, y1: left.y, x2: right.x, y2: right.y }];
    });
  }

  startDrag(event: PointerEvent, kind: 'crop' | 'perspective', index: number): void {
    (event.target as HTMLElement).setPointerCapture(event.pointerId);
    this.drag = { kind, index };
    event.preventDefault();
  }
  moveDrag(event: PointerEvent): void {
    if (!this.drag || !this.frame) return;
    const rect = this.frame.nativeElement.getBoundingClientRect();
    if (!rect.width || !rect.height) return;
    const point = { x: clamp01((event.clientX - rect.left) / rect.width), y: clamp01((event.clientY - rect.top) / rect.height) };
    if (this.drag.kind === 'crop') this.state.crop = moveCropCorner(this.state.crop, this.drag.index, point);
    else this.state.perspective = this.state.perspective.map((p, i) => i === this.drag!.index ? point : p);
  }
  stopDrag(event: PointerEvent): void {
    if (!this.drag) return;
    try { (event.target as HTMLElement).releasePointerCapture(event.pointerId); } catch { /* already released */ }
    this.drag = null;
  }

  async preview(): Promise<void> {
    const built = buildOperations(this.state);
    if (!built.ok) { this.error = built.error; return; }
    if (!this.previewFn) return;
    this.error = ''; this.busy = true;
    try { this.resultPreview = await this.previewFn(built.operations); }
    catch (e) { this.error = typeof e === 'string' ? e : 'Operazione non riuscita'; }
    this.busy = false;
    this.change.detectChanges();
  }
  apply(): void {
    const built = buildOperations(this.state);
    if (!built.ok) { this.error = built.error; return; }
    this.error = '';
    this.applied.emit(built.operations);
  }
  protected readonly cropChanged = cropChanged;
  protected readonly perspectiveChanged = perspectiveChanged;
  protected readonly modes: PerspectiveMode[] = ['auto', 'custom', '1:1', '4:5', '3:2', '16:9'];
}
