import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, Input, OnChanges, SimpleChanges, ViewEncapsulation, inject } from '@angular/core';
import { translate } from './i18n';
import { certificateFingerprint, certificateTexts, dimensionsText, formatIssueDate, localizedValue } from './certificate-text';
import type { CertificateTexts } from './certificate-text';
import type { Artwork, Certificate } from './models';

/**
 * The printed certificate of authenticity: an A4 sheet with the same layout as the Gestionale Opere
 * (artist name and title at the top, large picture, details, declaration, place/date/signature and
 * verification box). It is drawn from the frozen snapshot of the issued certificate.
 */
@Component({
  selector: 'rp-certificate-sheet',
  standalone: true,
  imports: [CommonModule],
  encapsulation: ViewEncapsulation.None,
  styles: [`
    .cs-page{box-sizing:border-box;width:210mm;height:297mm;padding:15mm 17mm;background:#fffdf8;color:#25231f;font-family:'Palatino Linotype',Georgia,serif;display:flex;flex-direction:column;overflow:hidden;text-align:left}
    .cs-gold{color:#9a762d}
    .cs-head{display:flex;justify-content:space-between;align-items:flex-end;gap:8mm;border-bottom:1px solid #d8c9a6;padding-bottom:2mm}
    .cs-brand{font:24px Georgia,serif;overflow-wrap:anywhere}
    .cs-title{padding-bottom:1mm;text-align:right;font:12px/1.35 Arial,sans-serif;letter-spacing:1.5px;text-transform:uppercase;flex:none}
    .cs-number{margin:6mm 0;font:11px Arial,sans-serif;letter-spacing:1px}
    .cs-image{flex:1 1 90mm;min-height:60mm;box-sizing:border-box;padding:4mm 6mm;border:1px solid #ddd5c4;display:flex;align-items:center;justify-content:center;background:#f6f1e8}
    .cs-image img{max-height:100%;max-width:100%;object-fit:contain}
    .cs-placeholder{color:#766;font:11px Arial,sans-serif}
    .cs-info{display:grid;grid-template-columns:1fr 1fr;gap:8mm;margin-top:6mm}
    .cs-name{font:26px/1.1 'Palatino Linotype',Georgia,serif;overflow-wrap:anywhere}
    .cs-meta{font:11px/1.65 Arial,sans-serif;overflow-wrap:anywhere}
    .cs-decl{margin-top:5mm;font:11px/1.55 'Palatino Linotype',Georgia,serif}
    .cs-decl p{margin:0 0 3mm}
    .cs-notes{margin:0;font:10px/1.5 Arial,sans-serif;color:#4a463e;white-space:pre-wrap}
    .cs-foot{margin-top:6mm;display:grid;grid-template-columns:1fr 1fr;gap:12mm;font:10px Arial,sans-serif}
    .cs-sign{border-top:1px solid #555;padding-top:3mm;margin-top:12mm}
    .cs-verify{border-left:2px solid #a8873c;padding-left:5mm}
    .cs-hash{color:#777;font-size:8px;margin-top:3mm;overflow-wrap:anywhere}
    @media print{
      .print-only .cs-page{margin:0;height:296mm;break-inside:avoid;-webkit-print-color-adjust:exact;print-color-adjust:exact}
    }
  `],
  template: `
    <article class="cs-page" *ngIf="texts as t">
      <header class="cs-head">
        <div class="cs-brand">{{ certificate.artist_name }}</div>
        <div class="cs-title"><span class="cs-gold"><ng-container *ngFor="let line of t.titleLines; let last = last">{{ line }}<br *ngIf="!last"></ng-container></span></div>
      </header>
      <div class="cs-number">{{ certificate.serial }}</div>
      <div class="cs-image">
        <img *ngIf="image; else missing" [src]="image" [alt]="snapshot.title">
        <ng-template #missing><span class="cs-placeholder">{{ t.labels.missingImage }}</span></ng-template>
      </div>
      <section class="cs-info">
        <div><div class="cs-name">{{ snapshot.title }}</div><div class="cs-meta">{{ identity }}</div></div>
        <div class="cs-meta">
          <b>{{ t.labels.artist }}</b><br>{{ certificate.artist_name }}<br>
          <b>{{ t.labels.technique }}</b><br>{{ technique(t) }}<br>
          <b>{{ t.labels.dimensions }}</b><br>{{ dimensions }}
          <ng-container *ngIf="certificate.collector_name"><br><b>{{ t.labels.recipient }}</b><br>{{ certificate.collector_name }}</ng-container>
        </div>
      </section>
      <div class="cs-decl"><p *ngFor="let paragraph of t.declarations">{{ paragraph }}</p></div>
      <p class="cs-notes" *ngIf="certificate.notes">{{ certificate.notes }}</p>
      <footer class="cs-foot">
        <div>{{ certificate.issued_place }}{{ certificate.issued_place && date ? ' · ' : '' }}{{ date }}<div class="cs-sign">{{ t.labels.signature }}</div></div>
        <div class="cs-verify"><b>{{ t.labels.verification }}</b><br>{{ certificate.serial }}<div class="cs-hash" *ngIf="hash">SHA-256 {{ hash }}</div></div>
      </footer>
    </article>`
})
export class CertificateSheetComponent implements OnChanges {
  @Input({ required: true }) certificate!: Certificate;
  @Input({ required: true }) snapshot!: Artwork;
  /** Data URL of the artwork picture frozen in the certificate (null: a placeholder is printed). */
  @Input() image: string | null = null;

  private readonly change = inject(ChangeDetectorRef);
  texts: CertificateTexts | null = null;
  date = '';
  identity = '';
  dimensions = '';
  hash = '';

  ngOnChanges(changes: SimpleChanges): void {
    if (!this.certificate || !this.snapshot) return;
    if (changes['certificate'] || changes['snapshot']) {
      const c = this.certificate;
      // The header and the artist line show the (stage) name; the declaration is signed with the full name.
      this.texts = certificateTexts(c.language, c.declarant_name || c.artist_name);
      this.date = formatIssueDate(c.issued_at, c.language);
      this.identity = [this.snapshot.year, this.snapshot.code].filter(Boolean).join(' · ');
      this.dimensions = dimensionsText(this.snapshot);
      this.hash = '';
      const token = c.id;
      void certificateFingerprint(c.serial, c.snapshot_json).then(hex => { if (this.certificate?.id === token) { this.hash = hex; this.change.detectChanges(); } });
    }
  }

  technique(t: CertificateTexts): string {
    const english = (text: string) => translate(text, 'en');
    const technique = localizedValue(this.snapshot.technique, this.certificate.language, english);
    const support = localizedValue(this.snapshot.support, this.certificate.language, english);
    return technique ? technique + (support ? t.labels.support + support : '') : support || '—';
  }
}
