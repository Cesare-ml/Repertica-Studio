import type { MailFolder, MailHeader, MailSyncState } from './models';

/** Translator for the mail texts (English keys). */
export type Translate = (key: string) => string;

/** Number in the menu badge: nothing for zero, "99+" above 99. */
export function badgeText(unread: number): string {
  if (!Number.isFinite(unread) || unread <= 0) return '';
  return unread > 99 ? '99+' : String(Math.floor(unread));
}

const ROLE_LABELS: Record<string, string> = {
  inbox: 'Inbox', sent: 'Sent', drafts: 'Drafts on server', trash: 'Trash', junk: 'Spam', archive: 'Archive',
};
/** Readable name of a folder: well-known roles get a translated name, any other folder shows its own name. */
export function folderLabel(folder: Pick<MailFolder, 'role' | 'display'>, t: Translate): string {
  const key = ROLE_LABELS[folder.role];
  return key ? t(key) : folder.display;
}
/** Folders shown in the main list (the others go under "Other folders"). */
export const MAIN_ROLES: readonly string[] = ['inbox', 'sent', 'archive', 'junk', 'trash'];
export function splitFolders(folders: readonly MailFolder[]): { main: MailFolder[]; other: MailFolder[] } {
  return {
    main: folders.filter(f => MAIN_ROLES.includes(f.role)),
    other: folders.filter(f => !MAIN_ROLES.includes(f.role)),
  };
}

/** Display name of a sender: `"Ada Lovelace" <ada@x.org>` gives `Ada Lovelace`. */
export function senderName(from: string): string {
  const raw = (from ?? '').trim();
  const name = raw.split('<')[0].trim().replace(/^"+|"+$/g, '').trim();
  return name || raw.replace(/^<|>$/g, '').trim();
}
/** Date of a message from its header, or null when it cannot be read. */
export function messageDate(header: Pick<MailHeader, 'date'>): Date | null {
  const date = new Date(header.date);
  return Number.isNaN(date.getTime()) ? null : date;
}
/** Messages of the inbox split like the Gestionale does: still to read, and everything else. */
export function splitInbox<T extends { seen: boolean }>(items: readonly T[]): { unread: T[]; rest: T[] } {
  return { unread: items.filter(i => !i.seen), rest: items.filter(i => i.seen) };
}
/** Case-insensitive filter on sender and subject of the messages that are loaded. */
export function filterMessages<T extends Pick<MailHeader, 'from' | 'subject'>>(items: readonly T[], query: string): T[] {
  const needle = query.trim().toLowerCase();
  return needle ? items.filter(i => (i.from + ' ' + i.subject).toLowerCase().includes(needle)) : [...items];
}

/** What changes in an account's inbox: used to reload the list only when something happened. */
export function inboxStamp(state: MailSyncState | null, accountId: string): string {
  const account = state?.accounts.find(a => a.account_id === accountId);
  return account ? `${account.uid_validity}:${account.uid_next}:${account.exists}:${account.unread}` : '';
}

export interface NotificationContent { title: string; body: string }
/** Text of the system notification for new mail (one message: sender and subject; several: a summary). */
export function notificationContent(fresh: MailSyncState['fresh'], t: Translate): NotificationContent | null {
  if (!fresh.length) return null;
  if (fresh.length === 1) {
    const [only] = fresh;
    return { title: only.sender || only.account_email, body: only.subject || t('(No subject)') };
  }
  const senders = [...new Set(fresh.map(m => m.sender).filter(Boolean))].slice(0, 3).join(', ');
  return { title: `${fresh.length} ${t('new emails')}`, body: senders };
}
