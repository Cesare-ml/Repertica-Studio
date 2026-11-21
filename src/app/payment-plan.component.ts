import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, Input, ViewEncapsulation } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { api } from './api';
import { AppLanguage, translate } from './i18n';
import { installmentsTotalCents, MAX_INSTALLMENTS, MIN_INSTALLMENTS, toCents, type PaymentDraft } from './payments';

/** How a sale is paid: unpaid, paid, partly paid, or in installments. Edits `draft` in place. */
@Component({
  selector: 'rp-payment-plan',
  standalone: true,
  imports: [CommonModule, FormsModule],
  encapsulation: ViewEncapsulation.None,
  styles: [`
    .pp{display:grid;gap:10px}
    .pp-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:10px}
    .pp label{display:grid;gap:4px;font-size:12px;font-weight:600;color:#4d5f56}
    .pp input,.pp select{font:inherit;padding:8px 10px;border:1px solid #d9ddd5;border-radius:8px;background:#fff}
    .pp-rows{display:grid;gap:6px}
    .pp-row{display:grid;grid-template-columns:26px 150px 130px 1fr 34px;gap:8px;align-items:center}
    .pp-row .pp-n{font-size:12px;color:#7a867f;text-align:right}
    .pp-actions{display:flex;gap:8px;flex-wrap:wrap}
    .pp-total{font-size:12px;margin:0}
    .pp-total.bad{color:#a3382c;font-weight:700}
    .pp-sub{font-size:12px;font-weight:700;color:#4d5f56;margin:4px 0 0}
  `],
  template: `
    <div class="pp">
      <label *ngIf="!installmentsOnly"><span>{{ tr('Pagamento') }}</span>
        <select [(ngModel)]="draft.mode" (ngModelChange)="onModeChange()">
          <option value="UNPAID">{{ tr('Non incassata') }}</option>
          <option value="PAID">{{ tr('Saldata') }}</option>
          <option value="PARTIAL">{{ tr('Incasso parziale') }}</option>
          <option value="INSTALLMENTS">{{ tr('Rateale') }}</option>
        </select></label>

      <div class="pp-grid" *ngIf="draft.mode === 'PARTIAL'">
        <label><span>{{ tr('Importo incassato (€)') }}</span><input type="number" min="0.01" step="0.01" [(ngModel)]="draft.received_eur"></label>
      </div>
      <div class="pp-grid" *ngIf="draft.mode === 'PAID' || draft.mode === 'PARTIAL'">
        <label><span>{{ tr('Data incasso') }}</span><input type="date" [(ngModel)]="draft.received_at"></label>
        <label><span>{{ tr('Metodo') }}</span><select [(ngModel)]="draft.method">
          <option value="bank_transfer">{{ tr('Bonifico') }}</option><option value="cash">{{ tr('Contanti') }}</option>
          <option value="card">{{ tr('Carta') }}</option><option value="other">{{ tr('Altro') }}</option></select></label>
        <label><span>{{ tr('Riferimento') }}</span><input maxlength="200" [(ngModel)]="draft.reference"></label>
      </div>

      <ng-container *ngIf="draft.mode === 'INSTALLMENTS'">
        <div class="pp-grid">
          <label><span>{{ tr('Numero di rate') }}</span><input type="number" [min]="min" [max]="max" step="1" [(ngModel)]="draft.count"></label>
          <label><span>{{ tr('Prima scadenza') }}</span><input type="date" [(ngModel)]="draft.first_due"></label>
        </div>
        <div class="pp-actions">
          <button type="button" class="secondary small" (click)="generate()" [disabled]="!amountCents || amountCents <= 0">{{ tr('Genera rate') }}</button>
          <button type="button" class="secondary small" (click)="addRow()" [disabled]="draft.installments.length >= max">{{ tr('Aggiungi rata') }}</button>
        </div>
        <div class="pp-rows" *ngIf="draft.installments.length">
          <div class="pp-row" *ngFor="let row of draft.installments; let i = index">
            <span class="pp-n">{{ i + 1 }}</span>
            <input type="date" [(ngModel)]="row.due_date" [attr.aria-label]="tr('Scadenza')">
            <input type="number" min="0.01" step="0.01" [(ngModel)]="row.amount_eur" [attr.aria-label]="tr('Importo rata (€)')">
            <input [(ngModel)]="row.notes" maxlength="500" [attr.aria-label]="tr('Note')" [placeholder]="tr('Note')">
            <button type="button" class="small danger-text" (click)="removeRow(i)" [attr.aria-label]="tr('Rimuovi')">×</button>
          </div>
        </div>
        <p class="pp-total" [class.bad]="!totalMatches" *ngIf="draft.installments.length">
          {{ tr('Totale rate') }}: {{ formatEuro(totalCents) }} · {{ tr('Prezzo') }}: {{ formatEuro(amountCents || 0) }}</p>
        <ng-container *ngIf="!installmentsOnly">
          <p class="pp-sub">{{ tr('Acconto già incassato (facoltativo)') }}</p>
          <div class="pp-grid">
            <label><span>{{ tr('Importo incassato (€)') }}</span><input type="number" min="0" step="0.01" [(ngModel)]="draft.received_eur"></label>
            <label *ngIf="draft.received_eur"><span>{{ tr('Data incasso') }}</span><input type="date" [(ngModel)]="draft.received_at"></label>
            <label *ngIf="draft.received_eur"><span>{{ tr('Metodo') }}</span><select [(ngModel)]="draft.method">
              <option value="bank_transfer">{{ tr('Bonifico') }}</option><option value="cash">{{ tr('Contanti') }}</option>
              <option value="card">{{ tr('Carta') }}</option><option value="other">{{ tr('Altro') }}</option></select></label>
          </div>
        </ng-container>
      </ng-container>
    </div>`
})
export class PaymentPlanComponent {
  @Input({ required: true }) draft!: PaymentDraft;
  /** Sale price in cents (the plan must add up to it). */
  @Input() amountCents: number | null = null;
  @Input() language: AppLanguage = 'it';
  /** Only the installment plan is edited (used on an existing sale). */
  @Input() installmentsOnly = false;
  readonly min = MIN_INSTALLMENTS;
  readonly max = MAX_INSTALLMENTS;

  constructor(private readonly change: ChangeDetectorRef) {}

  tr(key: string): string { return translate(key, this.language); }
  formatEuro(cents: number): string {
    return new Intl.NumberFormat(this.language, { style: 'currency', currency: 'EUR' }).format(cents / 100);
  }
  get totalCents(): number { return installmentsTotalCents(this.draft.installments); }
  get totalMatches(): boolean { return this.totalCents === (this.amountCents ?? -1); }

  onModeChange(): void {
    if (this.draft.mode === 'PAID' && this.amountCents) this.draft.received_eur = this.amountCents / 100;
    if (this.draft.mode === 'INSTALLMENTS' || this.draft.mode === 'UNPAID') this.draft.received_eur = null;
    if (this.draft.mode === 'INSTALLMENTS' && !this.draft.installments.length) void this.generate();
  }

  async generate(): Promise<void> {
    const cents = this.amountCents;
    if (!cents || cents <= 0) return;
    const count = Math.trunc(Number(this.draft.count));
    if (!Number.isFinite(count) || count < MIN_INSTALLMENTS || count > MAX_INSTALLMENTS || !this.draft.first_due) return;
    try {
      const plan = await api.generateInstallmentPlan(cents, count, this.draft.first_due);
      this.draft.installments = plan.map(line => ({ due_date: line.due_date, amount_eur: line.amount_cents / 100, notes: null }));
    } catch { /* the totals line and the save validation explain what is missing */ }
    this.change.detectChanges();
  }

  addRow(): void {
    const last = this.draft.installments[this.draft.installments.length - 1];
    this.draft.installments = [...this.draft.installments, { due_date: last?.due_date ?? this.draft.first_due, amount_eur: null, notes: null }];
  }
  removeRow(index: number): void { this.draft.installments = this.draft.installments.filter((_, i) => i !== index); }
  protected readonly toCents = toCents;
}
