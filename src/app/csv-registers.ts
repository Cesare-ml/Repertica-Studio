/**
 * User-facing metadata for the five local SQLite CSV registers.
 * Filename suggestions are stable and translated by UI messages, not by the
 * OS locale: importing these CSVs back is intentionally NOT supported.
 */
export type CsvRegisterKind = 'artworks' | 'collectors' | 'certificates' | 'sales' | 'expenses';

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
  expenses: {
    title: 'Esporta spese CSV',
    filename: 'spese',
    success: 'Spese esportate in CSV locale.'
  }
};
