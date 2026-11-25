// Read-only presentation of the native ledger's selected-binding projection.
// No endpoint, IPC registration, token, accounting mutation or retry is defined here.
export type SettlementSummary = Readonly<{
  sequence: number;
  status: 'fulfilled' | 'review_required';
  amount: number;
  currency: 'EUR';
  fulfilledAt: string;
  acknowledgedAt: string | null;
}>;
export type SettlementLanguage = 'it' | 'en' | 'es' | 'fr' | 'de';
const keys = ['sequence', 'status', 'amount', 'currency', 'fulfilledAt', 'acknowledgedAt'] as const;
const positive = (n: unknown): n is number => typeof n === 'number' && Number.isSafeInteger(n) && n > 0;
const timestamp = (v: unknown): v is string => typeof v === 'string' && v.length <= 40 &&
  /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/.test(v) && Number.isFinite(Date.parse(v));

export function parseSettlementSummary(raw: unknown): SettlementSummary | null {
  if (raw === null) return null;
  if (typeof raw !== 'object' || Array.isArray(raw) || raw === null) throw new Error('settlement_invalid_summary');
  const value = raw as Record<string, unknown>;
  if (Object.keys(value).length !== keys.length || keys.some(key => !Object.hasOwn(value, key)) ||
      !positive(value['sequence']) || !positive(value['amount']) || value['currency'] !== 'EUR' ||
      (value['status'] !== 'fulfilled' && value['status'] !== 'review_required') || !timestamp(value['fulfilledAt']) ||
      (value['acknowledgedAt'] !== null && !timestamp(value['acknowledgedAt']))) throw new Error('settlement_invalid_summary');
  return Object.freeze({sequence: value['sequence'], status: value['status'], amount: value['amount'], currency: value['currency'],
    fulfilledAt: value['fulfilledAt'], acknowledgedAt: value['acknowledgedAt']});
}

const labels = {
  it: ['Perfezionamento segnalato', 'Revisione manuale necessaria', 'Ricezione confermata', 'Ricezione non ancora confermata', 'Esito dell’asta: non è una registrazione di vendita o di incasso.'],
  en: ['Fulfillment reported', 'Manual review required', 'Receipt acknowledged', 'Receipt not yet acknowledged', 'Auction result: this is not a recorded sale or payment.'],
  es: ['Cumplimiento comunicado', 'Revisión manual necesaria', 'Recepción confirmada', 'Recepción aún no confirmada', 'Resultado de la subasta: no es un registro de venta ni de cobro.'],
  fr: ['Exécution signalée', 'Révision manuelle requise', 'Réception confirmée', 'Réception non encore confirmée', 'Résultat de la vente aux enchères : aucune vente ni aucun encaissement enregistré.'],
  de: ['Erfüllung gemeldet', 'Manuelle Prüfung erforderlich', 'Empfang bestätigt', 'Empfang noch nicht bestätigt', 'Auktionsergebnis: kein erfasster Verkauf oder Zahlungseingang.'],
} as const;

// Decimal formatting from integer cents, not floating-point division of potentially
// large safe integers. Intl supplies grouping and currency position.
function formatCents(amount: number, language: SettlementLanguage): string {
  const cents = BigInt(amount);
  const units = cents / 100n;
  const fraction = (cents % 100n).toString().padStart(2, '0');
  return new Intl.NumberFormat(language, {style: 'currency', currency: 'EUR', minimumFractionDigits: 2, maximumFractionDigits: 2})
    .formatToParts(units).map(part => part.type === 'fraction' ? fraction : part.value).join('');
}

export function settlementPresentation(raw: unknown, language: SettlementLanguage) {
  const summary = parseSettlementSummary(raw);
  if (!summary) return null;
  const l = labels[language] ?? labels.en;
  return Object.freeze({ sequence: summary.sequence, status: l[summary.status === 'review_required' ? 1 : 0],
    receipt: l[summary.acknowledgedAt === null ? 3 : 2], notice: l[4], amount: formatCents(summary.amount, language),
    fulfilledAt: summary.fulfilledAt, acknowledgedAt: summary.acknowledgedAt });
}

// A local selection is not an authorization. The native backend must resolve
// the published association and independently verify the settlement grant.
export type SettlementSelection = Readonly<{ artworkId: string; scope: string; revision: string }>;
export type SettlementReadRequest = Readonly<{action: 'readSettlement'; artworkId: string; scope: string}>;
export type SettlementReadState = Readonly<
  {kind: 'idle' | 'loading' | 'empty' | 'unavailable'} |
  {kind: 'ready'; summary: SettlementSummary}
>;
const sameSelection = (a: SettlementSelection | null, b: SettlementSelection | null): boolean =>
  !!a && !!b && a.artworkId === b.artworkId && a.scope === b.scope && a.revision === b.revision;

export class SettlementReader {
  private selection: SettlementSelection | null = null;
  private generation = 0;
  private state: SettlementReadState = {kind:'idle'};
  constructor(private readonly read: (request: SettlementReadRequest) => Promise<unknown>) {}

  get busy(): boolean { return this.state.kind === 'loading'; }

  clear(): void {
    this.generation++;
    this.selection = null;
    this.state = {kind:'idle'};
  }

  current(selection: SettlementSelection | null): SettlementReadState {
    return sameSelection(this.selection, selection) ? this.state : {kind:'idle'};
  }

  async load(selection: SettlementSelection): Promise<void> {
    this.clear();
    const generation = this.generation;
    const {artworkId, scope, revision} = selection;
    this.selection = {artworkId, scope, revision};
    this.state = {kind:'loading'};
    try {
      const value = parseSettlementSummary(await this.read({action:'readSettlement', artworkId, scope}));
      if (generation === this.generation) this.state = value ? {kind:'ready', summary:value} : {kind:'empty'};
    } catch {
      if (generation === this.generation) this.state = {kind:'unavailable'};
    }
  }
}
