/**
 * User-facing metadata for the nine local SQLite CSV registers.
 * Filename suggestions are stable and translated by UI messages, not by the
 * OS locale: importing these CSVs back is intentionally NOT supported.
 */
export type CsvRegisterKind = 'artworks' | 'gallery' | 'collectors' | 'certificates' | 'sales' | 'balances' | 'receipts' | 'expenses' | 'allocations' | 'installments';

export interface CsvRegisterOption {
  title: string;
  filename: string;
  success: string;
}

export const csvRegisterOptions: Record<CsvRegisterKind, CsvRegisterOption> = {
  artworks: {
    title: 'Esporta opere CSV',
    filename: 'opere',
    success: 'Registro opere esportato in CSV locale.'
  },
  gallery: {
    title: 'Esporta metadati galleria CSV',
    filename: 'galleria',
    success: 'Metadati galleria esportati in CSV locale.'
  },
  collectors: {
    title: 'Esporta collezionisti CSV',
    filename: 'collezionisti',
    success: 'Registro collezionisti esportato in CSV locale.'
  },
  certificates: {
    title: 'Esporta certificati CSV',
    filename: 'certificati',
    success: 'Registro certificati esportato in CSV locale.'
  },
  sales: {
    title: 'Esporta vendite CSV',
    filename: 'vendite',
    success: 'Registro vendite esportato in CSV locale.'
  },
  balances: {
    title: 'Esporta saldi vendite CSV',
    filename: 'saldivendite',
    success: 'Saldi vendite esportati in CSV locale.'
  },
  receipts: {
    title: 'Esporta incassi CSV',
    filename: 'incassi',
    success: 'Registro incassi esportato in CSV locale.'
  },
  installments: {
    title: 'Esporta piani rateali CSV',
    filename: 'pianirateali',
    success: 'Piani rateali esportati in CSV locale.'
  },
  allocations: {
    title: 'Esporta ripartizioni CSV',
    filename: 'ripartizioni',
    success: 'Ripartizioni esportate in CSV locale.'
  },
  expenses: {
    title: 'Esporta spese CSV',
    filename: 'spese',
    success: 'Spese esportate in CSV locale.'
  }
};
