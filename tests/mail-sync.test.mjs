import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { badgeText, filterMessages, folderLabel, inboxStamp, messageDate, notificationContent, senderName, splitFolders, splitInbox } from '../src/app/mail-sync.ts';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const t = key => `[${key}]`;

test('the menu badge shows the unread count, nothing for zero and 99+ for a lot', () => {
  assert.equal(badgeText(0), '');
  assert.equal(badgeText(-3), '');
  assert.equal(badgeText(Number.NaN), '');
  assert.equal(badgeText(1), '1');
  assert.equal(badgeText(99), '99');
  assert.equal(badgeText(100), '99+');
});

test('folders get normal names, never INBOX or technical prefixes', () => {
  assert.equal(folderLabel({ role: 'inbox', display: 'INBOX' }, t), '[Inbox]');
  assert.equal(folderLabel({ role: 'sent', display: 'Sent' }, t), '[Sent]');
  assert.equal(folderLabel({ role: 'trash', display: 'Corbeille' }, t), '[Trash]');
  assert.equal(folderLabel({ role: 'junk', display: 'Junk' }, t), '[Spam]');
  assert.equal(folderLabel({ role: 'archive', display: 'Archiv' }, t), '[Archive]');
  assert.equal(folderLabel({ role: 'other', display: 'Clienti / Roma' }, t), 'Clienti / Roma', 'other folders show their own readable name');
  const { main, other } = splitFolders([
    { name: 'INBOX', role: 'inbox' }, { name: 'X', role: 'drafts' }, { name: 'Y', role: 'other' }, { name: 'S', role: 'sent' }]);
  assert.deepEqual(main.map(f => f.role), ['inbox', 'sent']);
  assert.deepEqual(other.map(f => f.role), ['drafts', 'other']);
});

test('messages are split into still-to-read and the rest, and can be filtered by sender or subject', () => {
  const rows = [
    { uid: 1, seen: false, from: 'Ada <ada@x.org>', subject: 'Preventivo' },
    { uid: 2, seen: true, from: 'Bob', subject: 'Fattura di novembre' },
    { uid: 3, seen: false, from: 'Carla', subject: 'Invito' }];
  const { unread, rest } = splitInbox(rows);
  assert.deepEqual(unread.map(r => r.uid), [1, 3]);
  assert.deepEqual(rest.map(r => r.uid), [2]);
  assert.deepEqual(filterMessages(rows, 'ADA').map(r => r.uid), [1]);
  assert.deepEqual(filterMessages(rows, 'fattura').map(r => r.uid), [2]);
  assert.equal(filterMessages(rows, '  ').length, 3);
  assert.equal(filterMessages(rows, 'nessuno').length, 0);
});

test('sender names and dates are shown for people', () => {
  assert.equal(senderName('"Ada Lovelace" <ada@x.org>'), 'Ada Lovelace');
  assert.equal(senderName('<ada@x.org>'), 'ada@x.org');
  assert.equal(senderName('ada@x.org'), 'ada@x.org');
  assert.equal(senderName(''), '');
  assert.ok(messageDate({ date: 'Sun, 22 Nov 2026 10:00:00 +0100' }) instanceof Date);
  assert.equal(messageDate({ date: 'non è una data' }), null);
});

test('the inbox stamp changes when mail arrives or is read, so the list refreshes by itself', () => {
  const state = unread => ({ unread_total: unread, checked_at: null, fresh: [], accounts: [{ account_id: 'a', email: '', unread, exists: 5, uid_validity: 7, uid_next: 10, error: false }] });
  assert.equal(inboxStamp(null, 'a'), '');
  assert.equal(inboxStamp(state(2), 'zzz'), '');
  assert.notEqual(inboxStamp(state(2), 'a'), inboxStamp(state(1), 'a'));
  assert.equal(inboxStamp(state(2), 'a'), inboxStamp(state(2), 'a'));
});

test('the system notification names the sender and subject, or summarises several messages', () => {
  const mail = (sender, subject) => ({ account_id: 'a', account_email: 'me@x.org', uid: 1, sender, subject });
  assert.equal(notificationContent([], t), null);
  assert.deepEqual(notificationContent([mail('Ada', 'Preventivo')], t), { title: 'Ada', body: 'Preventivo' });
  assert.deepEqual(notificationContent([mail('', '')], t), { title: 'me@x.org', body: '[(No subject)]' });
  assert.deepEqual(notificationContent([mail('Ada', 'a'), mail('Bob', 'b'), mail('Ada', 'c')], t), { title: '3 [new emails]', body: 'Ada, Bob' });
});

test('mail is synchronised automatically: no connect or refresh buttons, a background watcher, a menu badge and notifications', () => {
  const html = get('src/app/business-mail.component.html');
  assert.doesNotMatch(html, /Connect \/ refresh IMAP|t\('Refresh'\)|Refresh is manual/, 'no manual synchronisation buttons');
  const ts = get('src/app/business-mail.component.ts');
  assert.match(ts, /REFRESH_MS/);
  assert.match(ts, /ngOnChanges/);
  assert.match(ts, /mailSetSeen/, 'opening a message marks it as read');
  const rust = get('src-tauri/src/mail_watch.rs');
  assert.match(rust, /POLL_SECONDS: u64 = 60/);
  assert.match(rust, /emit\("mail-status"/);
  const lib = get('src-tauri/src/lib.rs');
  for (const name of ['mail_watch::start', 'mail_watch::mail_sync_state', 'mail_watch::mail_sync_now', 'tauri_plugin_notification::init', 'business_mail::mail_set_seen']) {
    assert.ok(lib.includes(name), name);
  }
  assert.match(get('src-tauri/capabilities/default.json'), /notification:default/);
  const app = get('src/app/app.component.html');
  assert.match(app, /class="nav-badge"/);
  assert.match(app, /item\.id === 'mail' && mailUnread > 0/);
  const component = get('src/app/app.component.ts');
  assert.match(component, /listen<MailSyncState>\('mail-status'/);
  assert.match(component, /sendNotification\(content\)/);
  assert.match(component, /if \(!this\.settings\.mail_notifications/, 'notifications can be switched off');
  assert.match(get('src-tauri/src/finance.rs'), /fn notifications_on\(\) -> bool \{ true \}/, 'notifications are on by default');
  assert.match(get('src-tauri/src/db.rs'), /mail_notifications INTEGER NOT NULL DEFAULT 1/);
});

test('the mail section has the layout of the Gestionale Opere with the look of Repertica Studio', () => {
  const css = get('src/app/business-mail.component.css');
  for (const token of ['.inbox-section-heading', '.compose-window', '.compose-titlebar', '.folders', '.reader']) assert.ok(css.includes(token), token);
  // ... drawn with the colours of Repertica Studio, not the Gestionale's blue.
  assert.match(css, /--accent: #285343/);
  assert.doesNotMatch(css, /#0759b4|#0b57d0|#c2e7ff/);
  const html = get('src/app/business-mail.component.html');
  for (const part of ['class="mail-bar', 'class="folders"', 'class="inbox-row', 'class="reader"', 'class="compose-window"']) assert.ok(html.includes(part), part);
});

test('only one mailbox can be connected', () => {
  const rust = get('src-tauri/src/business_mail.rs');
  assert.match(rust, /one_mailbox_only\(accounts\(&db\)\?\.len\(\),input\.id\.is_none\(\)\)/);
  const html = get('src/app/business-mail.component.html');
  assert.match(html, /!accountForm && !accounts\.length/, 'no add button once a mailbox is connected');
  assert.doesNotMatch(html, /account-select|selectAccount/, 'no mailbox selector');
});

test('the mail section is fast: one kept IMAP session, one command for folders and messages, remembered lists', () => {
  const rust = get('src-tauri/src/business_mail.rs');
  assert.match(rust, /fn kept_sessions\(\)/, 'a logged-in session is kept per mailbox');
  assert.match(rust, /const IDLE_LIMIT:Duration=Duration::from_secs\(4\*60\)/);
  assert.match(rust, /CONNECT_TIMEOUT/);
  assert.match(rust, /IO_TIMEOUT/);
  assert.doesNotMatch(rust, /let _=session\.logout\(\);\s*result/, 'sessions are no longer closed after every command');
  assert.match(rust, /forget_session\(&id\)/, 'a changed password or server drops the kept session');
  assert.match(rust, /forget_session\(&account_id\)/, 'a removed mailbox drops its session');
  assert.match(get('src-tauri/src/lib.rs'), /business_mail::mail_overview/);
  assert.match(get('src/app/api.ts'), /'mail_overview'/);
  const ts = get('src/app/business-mail.component.ts');
  assert.match(ts, /api\.mailOverview\(/);
  assert.doesNotMatch(ts, /api\.mailListFolders\(/, 'folders and messages come together');
  assert.match(ts, /const mailMemory = new Map/, 'the last lists are shown at once');
  assert.match(ts, /this\.messageLoading=true/, 'the reader opens before the body arrives');
});
