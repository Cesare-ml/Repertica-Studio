// Exact, locale-independent values for the selected local archive export.
export function money(value, positive=false) {
  if (typeof value !== 'string') throw Error('Expected exact decimal amount');
  const m=/^(\d{1,10})(?:\.(\d{1,2}))?$/.exec(value);
  if (!m) throw Error('Invalid monetary amount');
  const n=BigInt(m[1])*100n+BigInt((m[2]||'').padEnd(2,'0'));
  if (n>100000000000n || (positive && n===0n)) throw Error('Amount outside supported range');
  return Number(n);
}
export function date(value) {
  if (typeof value!=='string' || !/^\d{4}-\d{2}-\d{2}$/.test(value) ||
      !Number.isFinite(Date.parse(value)) || new Date(value).toISOString().slice(0,10)!==value)
    throw Error('Invalid date');
  return value;
}

// Shared with the batch preflight: validate the original Unicode length before
// trimming; null represents an absent note, not an invalid value.
export function validateInstallmentNotes(value) {
  if (value!==null && (typeof value!=='string' || Array.from(value).length>500 || value.includes('\0')))
    throw Error('Invalid installment notes');
}

// Legacy receipts can be assigned to particular installments; Studio instead applies
// receipts in due-date order. Until those semantics can be reconciled, accept only
// unpaid plans. Preserve source positions, do not sort or regenerate a schedule.
export function unpaidInstallments(rows, receipts, saleCents) {
  if (!Array.isArray(rows) || !Array.isArray(receipts) || !Number.isSafeInteger(saleCents) || saleCents<1 || saleCents>100000000000)
    throw Error('Invalid installment projection');
  if (!rows.length) return [];
  if (rows.length<2 || rows.length>60 || receipts.length) throw Error('Unsupported installment history');
  let previous='', total=0;
  const plan=rows.map((r,i)=>{
    if (!r || typeof r!=='object' || Array.isArray(r) || Object.keys(r).length!==4 ||
        !['position','amount','due_date','notes'].every(k=>Object.hasOwn(r,k)) || r.position!==i+1)
      throw Error('Invalid installment position or projection');
    const due_date=date(r.due_date), amount_cents=money(r.amount,true);
    if (due_date<previous) throw Error('Installment order conflicts with due dates');
    validateInstallmentNotes(r.notes);
    previous=due_date;
    total+=amount_cents;
    if (total>saleCents) throw Error('Installments exceed sale');
    return {position:r.position,due_date,amount_cents,notes:r.notes?.trim()||null};
  });
  if (total!==saleCents) throw Error('Installments must total the sale');
  return plan;
}

// Mirror native clean_relation: optional text, Unicode scalar length before
// trimming (whitespace-only values are empty), and no embedded NUL.
// Rust str::trim uses Unicode White_Space, not JavaScript's trim set.
export function validateCommercialText(value, limit) {
  if (value == null) return;
  if (typeof value !== 'string' || value.includes('\0') ||
      (!/^\p{White_Space}*$/u.test(value) && Array.from(value).length > limit))
    throw Error('Invalid commercial text');
}
