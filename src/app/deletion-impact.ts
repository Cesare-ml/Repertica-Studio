/** What is deleted together with an artwork or a collector, as reported by the backend before the deletion. */
export interface SaleImpact { artwork_title: string; collector_name: string; amount_cents: number; sold_at: string; }
export interface DeletionImpact {
  certificates: string[];
  sales: SaleImpact[];
  receipts: number;
  attachments: number;
}

export const hasDependents = (impact: DeletionImpact): boolean => impact.certificates.length > 0 || impact.sales.length > 0;

interface Formatters { t: (key: string) => string; money: (cents: number) => string; date: (value: string) => string; }

const saleLine = (sale: SaleImpact, who: 'collector' | 'artwork', f: Formatters): string =>
  `• ${who === 'collector' ? sale.collector_name : sale.artwork_title} — ${f.money(sale.amount_cents)} (${f.date(sale.sold_at)})`;

/** The receipts and documents of the sales, in one line each (nothing when there are none). */
const paymentLines = (impact: DeletionImpact, f: Formatters): string[] => {
  const lines: string[] = [];
  if (impact.receipts > 0) lines.push(`• ${impact.receipts} ${f.t('incassi registrati')}`);
  if (impact.attachments > 0) lines.push(`• ${impact.attachments} ${f.t('documenti allegati agli incassi')}`);
  return lines;
};

/** The text of the confirmation for deleting an artwork. */
export function artworkDeletionMessage(title: string, impact: DeletionImpact, f: Formatters): string {
  const head = `${f.t('Eliminare definitivamente l’opera')} "${title}"?`;
  if (!hasDependents(impact)) return `${head}\n\n${f.t('Questa operazione non è reversibile.')}`;
  const lines: string[] = [];
  if (impact.certificates.length) lines.push(`• ${f.t('Certificati')}: ${impact.certificates.join(', ')}`);
  if (impact.sales.length) {
    lines.push(`• ${f.t('Vendita')}:`);
    lines.push(...impact.sales.map(s => '   ' + saleLine(s, 'collector', f)));
    lines.push(...paymentLines(impact, f));
  }
  return `${head}\n\n${f.t('L’opera ha dati collegati che verranno eliminati insieme a lei:')}\n${lines.join('\n')}\n\n${f.t('Questa operazione non è reversibile.')}`;
}

/** The text of the confirmation for deleting a collector. */
export function collectorDeletionMessage(name: string, impact: DeletionImpact, f: Formatters): string {
  const head = `${f.t('Eliminare il contatto')} "${name}"?`;
  if (!impact.sales.length) return head;
  const lines = [...impact.sales.map(s => saleLine(s, 'artwork', f)), ...paymentLines(impact, f)];
  return `${head}\n\n${f.t('Verranno eliminate anche le vendite annesse:')}\n${lines.join('\n')}\n\n${f.t('Le opere vendute tornano disponibili; i certificati già emessi restano.')}\n${f.t('Questa operazione non è reversibile.')}`;
}

/** The impacts of several records added together (for deleting a selection in one go). */
export function mergeImpacts(impacts: DeletionImpact[]): DeletionImpact {
  return {
    certificates: impacts.flatMap(i => i.certificates),
    sales: impacts.flatMap(i => i.sales),
    receipts: impacts.reduce((sum, i) => sum + i.receipts, 0),
    attachments: impacts.reduce((sum, i) => sum + i.attachments, 0),
  };
}

export type BulkKind = 'artworks' | 'collectors' | 'certificates';
const KIND_LABEL: Record<BulkKind, string> = { artworks: 'opere', collectors: 'contatti', certificates: 'certificati' };
const SHOWN = 8;

/** At most a few names, then how many more: the confirmation must stay readable for a long selection. */
const listed = (items: string[], f: Formatters): string[] => [
  ...items.slice(0, SHOWN).map(item => `• ${item}`),
  ...(items.length > SHOWN ? [`… ${f.t('e altri')} ${items.length - SHOWN}`] : []),
];

/**
 * The single confirmation for deleting a selection: how many and which records, and, when they have
 * dependents, what is deleted together with them (certificates, sales, receipts, attached documents).
 */
export function bulkDeletionMessage(kind: BulkKind, names: string[], impact: DeletionImpact, f: Formatters): string {
  const lines = [`${f.t('Eliminare definitivamente')} ${names.length} ${f.t(KIND_LABEL[kind])}?`, '', ...listed(names, f)];
  const related: string[] = [];
  if (kind === 'artworks' && impact.certificates.length) related.push(`• ${impact.certificates.length} ${f.t('Certificati')}`);
  if (impact.sales.length) {
    related.push(`• ${impact.sales.length} ${f.t('Vendite')}`);
    related.push(...listed(impact.sales.map(s => `${s.artwork_title} → ${s.collector_name} — ${f.money(s.amount_cents)}`), f).map(l => '   ' + l));
    related.push(...paymentLines(impact, f));
  }
  if (related.length) {
    lines.push('', f.t(kind === 'collectors' ? 'Verranno eliminate anche le vendite annesse:' : 'Verranno eliminati anche i dati collegati:'), ...related);
    if (kind === 'collectors') lines.push('', f.t('Le opere vendute tornano disponibili; i certificati già emessi restano.'));
  }
  lines.push('', f.t('Questa operazione non è reversibile.'));
  return lines.join('\n');
}
