import { test } from 'node:test';
import assert from 'node:assert/strict';
import { emptyPayment, installmentRows, installmentsTotalCents, paymentToInput, toCents } from '../src/app/payments.ts';

const draft = (overrides = {}) => ({ ...emptyPayment('2026-11-21'), ...overrides });
const rows = (...pairs) => pairs.map(([due_date, amount_eur]) => ({ due_date, amount_eur, notes: null }));

test('money is converted to exact cents without floating point drift', () => {
  assert.equal(toCents(0.1 + 0.2), 30);
  assert.equal(toCents(1234.56), 123456);
  assert.equal(toCents(19.99), 1999);
  assert.ok(Number.isNaN(toCents(null)));
  assert.ok(Number.isNaN(toCents(Number.POSITIVE_INFINITY)));
});

test('an unpaid sale sends no payment at all', () => {
  assert.deepEqual(paymentToInput(draft(), 10_000), { ok: true, input: null });
});

test('paid in full records the whole price on the chosen date', () => {
  const result = paymentToInput(draft({ mode: 'PAID', method: 'cash', reference: ' ric. 7 ' }), 12_345);
  assert.equal(result.ok, true);
  assert.deepEqual(result.input, { mode: 'PAID', received_cents: 12_345, received_at: '2026-11-21', method: 'cash', reference: 'ric. 7', installments: [] });
  assert.equal(paymentToInput(draft({ mode: 'PAID', received_at: '' }), 100).ok, false);
});

test('a partial payment must be above zero and below the price', () => {
  assert.equal(paymentToInput(draft({ mode: 'PARTIAL', received_eur: 40 }), 10_000).input.received_cents, 4_000);
  for (const bad of [0, -5, 100, 150, null]) {
    assert.equal(paymentToInput(draft({ mode: 'PARTIAL', received_eur: bad }), 10_000).ok, false, String(bad));
  }
});

test('an installment plan must have at least two rows adding up exactly to the price', () => {
  const plan = rows(['2026-12-21', 33.34], ['2027-01-21', 33.33], ['2027-02-21', 33.33]);
  assert.equal(installmentsTotalCents(plan), 10_000);
  const ok = paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan }), 10_000);
  assert.equal(ok.ok, true);
  assert.deepEqual(ok.input.installments.map(i => i.amount_cents), [3334, 3333, 3333]);
  assert.equal(ok.input.received_cents, null);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan }), 10_001).ok, false);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan.slice(0, 1) }), 3334).ok, false);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: rows(['', 50], ['2027-01-21', 50]) }), 10_000).ok, false);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: rows(['2026-12-21', 0], ['2027-01-21', 100]) }), 10_000).ok, false);
});

test('an advance on an installment plan is optional and must be below the price', () => {
  const plan = rows(['2026-12-21', 60], ['2027-01-21', 40]);
  const withAdvance = paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan, received_eur: 25 }), 10_000);
  assert.equal(withAdvance.input.received_cents, 2_500);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan, received_eur: 100 }), 10_000).ok, false);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan, received_eur: 25, received_at: '' }), 10_000).ok, false);
});

test('the plan editor of an existing sale uses the same validation', () => {
  assert.equal(installmentRows(rows(['2026-12-21', 50], ['2027-01-21', 50]), 10_000).ok, true);
  assert.equal(installmentRows(rows(['2026-12-21', 50], ['2027-01-21', 49.99]), 10_000).ok, false);
});

test('the frontend uses real Gregorian dates, matching native Rust validation', () => {
  assert.equal(paymentToInput(draft({ mode: 'PAID', received_at: '2028-02-29' }), 10_000).ok, true);
  for (const invalid of ['2026-02-29', '2026-04-31', '2026-13-01', '2026-00-01', '2026-11-31', '2026-1-01', '2026-11-21x']) {
    assert.equal(paymentToInput(draft({ mode: 'PAID', received_at: invalid }), 10_000).ok, false, invalid);
    const plan = rows([invalid, 50], ['2027-01-21', 50]);
    assert.equal(installmentRows(plan, 10_000).ok, false, invalid);
  }
  assert.equal(installmentRows(rows(['2028-02-29', 50], ['2028-03-31', 50]), 10_000).ok, true);
});

test('amounts outside the native integer ceiling are rejected before IPC', () => {
  assert.equal(toCents(1_000_000_000), 100_000_000_000);
  for (const tooLarge of [1_000_000_000.01, 1e20, -1e20]) {
    assert.ok(Number.isNaN(toCents(tooLarge)), String(tooLarge));
    assert.equal(installmentRows(rows(['2026-12-21', tooLarge], ['2027-01-21', 50]), 10_000).ok, false);
  }
});

test('a malformed optional advance is not silently treated as unpaid', () => {
  const plan = rows(['2026-12-21', 60], ['2027-01-21', 40]);
  for (const bad of [NaN, Infinity, -Infinity, -0.001, 1e20]) {
    assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan, received_eur: bad }), 10_000).ok, false, String(bad));
  }
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan, received_eur: null }), 10_000).input.received_cents, null);
  assert.equal(paymentToInput(draft({ mode: 'INSTALLMENTS', installments: plan, received_eur: 0 }), 10_000).input.received_cents, null);
});
