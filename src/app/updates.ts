import type { UpdateReply } from './models';

/** Interval between automatic checks while the app stays open (the first one runs shortly after start). */
export const UPDATE_CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;
export const FIRST_UPDATE_CHECK_DELAY_MS = 5_000;

/** Italian interface strings (translated with `tr`) for every outcome of an update check or install. */
const MESSAGES: Record<UpdateReply['status'], string> = {
  available: 'È disponibile una nuova versione.',
  current: 'Repertica Studio è aggiornato.',
  not_configured: 'Gli aggiornamenti automatici non sono attivi in questa versione dell’app.',
  check_failed: 'Impossibile controllare gli aggiornamenti: verifica la connessione a Internet e riprova.',
  invalid_release: 'L’aggiornamento trovato non è valido e non è stato scaricato.',
  busy: 'Un’altra operazione di aggiornamento è già in corso.',
  stale: 'L’aggiornamento non è più disponibile: controlla di nuovo.',
  backup_failed: 'Il backup automatico prima dell’aggiornamento non è riuscito. Risolvi il problema, per esempio collegando l’unità di backup, oppure installa comunque.',
  download_failed: 'Download non riuscito: riprova più tardi.',
  signature_invalid: 'La firma dell’aggiornamento non è valida: non è stato installato.',
  install_failed: 'Installazione non riuscita: riprova.',
  installing: 'Installazione in corso: l’app si riavvierà da sola.',
};

export function updateMessage(status: UpdateReply['status']): string {
  return MESSAGES[status] ?? MESSAGES.check_failed;
}

/** Statuses that an automatic (silent) check should not turn into an error on screen. */
export function isQuietStatus(status: UpdateReply['status']): boolean {
  return status === 'current' || status === 'not_configured' || status === 'check_failed' || status === 'busy';
}

/** Whole percent of the download, or null while the size is unknown. */
export function progressPercent(downloaded: number, total: number | null | undefined): number | null {
  if (!total || total <= 0) return null;
  return Math.max(0, Math.min(100, Math.floor((downloaded / total) * 100)));
}

export const messageKeys = (): string[] => Object.values(MESSAGES);
