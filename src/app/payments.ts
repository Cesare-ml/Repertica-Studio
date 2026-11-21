/**
 * Payment of a sale as the artist enters it: nothing yet, paid in full, partly paid, or an
 * installment (rateale) plan with an optional advance. Pure functions, no Angular, so the
 * rules can be tested directly. Amounts are converted to exact integer cents.
 */
import type { PaymentInput } from './models';

export type PaymentMode = 'UNPAID' | 'PAID' | 'PARTIAL' | 'INSTALLMENTS';
export type ReceiptMethod = 'bank_transfer' | 'cash' | 'card' | 'other';

export interface InstallmentRow { due_date: string; amount_eur: number | null; notes: string | null; }

export interface PaymentDraft {
  mode: PaymentMode;
  received_eur: number | null;
  received_at: string;
  method: ReceiptMethod;
  reference: string | null;
  count: number;
  first_due: string;
  installments: InstallmentRow[];
}

export type PaymentResult =
  | { ok: true; input: PaymentInput | null }
  | { ok: false; error: string };

export const MIN_INSTALLMENTS = 2;
export const MAX_INSTALLMENTS = 60;

export function emptyPayment(today: string): PaymentDraft {
  return { mode: 'UNPAID', received_eur: null, received_at: today, method: 'bank_transfer', reference: null,
    count: 3, first_due: today, installments: [] };
}

export function toCents(value: number | null | undefined): number {
  return typeof value === 'number' && Number.isFinite(value) ? Math.round((value + Number.EPSILON) * 100) : NaN;
}

export function installmentsTotalCents(rows: InstallmentRow[]): number {
  return rows.reduce((sum, row) => sum + (Number.isFinite(toCents(row.amount_eur)) ? toCents(row.amount_eur) : 0), 0);
}

const validDate = (value: string) => /^\d{4}-\d{2}-\d{2}$/.test(value);

/** Rows ready for the native plan command, or an error (an interface message key). */
export function installmentRows(rows: InstallmentRow[], saleCents: number):
  { ok: true; items: { due_date: string; amount_cents: number; notes: string | null }[] } | { ok: false; error: string } {
  if (rows.length < MIN_INSTALLMENTS || rows.length > MAX_INSTALLMENTS ||
      rows.some(r => !validDate(r.due_date) || !(toCents(r.amount_eur) > 0))) {
    return { ok: false, error: 'Inserisci almeno due rate con scadenza e importo' };
  }
  if (installmentsTotalCents(rows) !== saleCents) {
    return { ok: false, error: 'La somma delle rate deve essere uguale al prezzo' };
  }
  return { ok: true, items: rows.map(r => ({ due_date: r.due_date, amount_cents: toCents(r.amount_eur), notes: r.notes?.trim() || null })) };
}

export function paymentToInput(draft: PaymentDraft, saleCents: number): PaymentResult {
  if (draft.mode === 'UNPAID') return { ok: true, input: null };
  const base = { received_at: draft.received_at || null, method: draft.method, reference: draft.reference?.trim() || null };
  if (draft.mode === 'PAID' || draft.mode === 'PARTIAL' || toCents(draft.received_eur) > 0) {
    if (!validDate(draft.received_at)) return { ok: false, error: 'Inserisci la data di incasso' };
  }
  if (draft.mode === 'PAID') {
    return { ok: true, input: { mode: 'PAID', received_cents: saleCents, ...base, installments: [] } };
  }
  if (draft.mode === 'PARTIAL') {
    const cents = toCents(draft.received_eur);
    if (!(cents > 0 && cents < saleCents)) return { ok: false, error: 'Importo incassato non valido' };
    return { ok: true, input: { mode: 'PARTIAL', received_cents: cents, ...base, installments: [] } };
  }
  const plan = installmentRows(draft.installments, saleCents);
  if (!plan.ok) return plan;
  const advance = toCents(draft.received_eur);
  if (Number.isFinite(advance) && advance !== 0 && !(advance > 0 && advance < saleCents)) {
    return { ok: false, error: 'Acconto non valido' };
  }
  return { ok: true, input: { mode: 'INSTALLMENTS', received_cents: advance > 0 ? advance : null, ...base, installments: plan.items } };
}
