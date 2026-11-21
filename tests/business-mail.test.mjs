import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('mailbox remains independent from newsletter platform and server-side credentials', () => {
  const rust = read('src-tauri/src/business_mail.rs');
  const app = read('src/app/app.component.html');
  const lib = read('src-tauri/src/lib.rs');
  const api = read('src/app/api.ts');
  assert.match(app, /<rp-business-mail/);
  for (const cmd of [
    'mail_list_accounts', 'mail_save_account', 'mail_remove_account', 'mail_test_account',
    'mail_list_folders', 'mail_list_messages', 'mail_read_message',
    'mail_list_drafts', 'mail_save_draft', 'mail_remove_draft', 'mail_send'
  ]) {
    assert.ok(lib.includes('business_mail::' + cmd), cmd);
    assert.ok(api.includes("'" + cmd + "'"), cmd);
  }
  assert.match(rust, /KEYRING_SERVICE/);
  assert.match(rust, /keyring::/);
  assert.match(rust, /imap::connect/);
  assert.match(rust, /SmtpTransport::starttls_relay/);
  assert.match(rust, /SmtpTransport::relay/);
  assert.match(rust, /BODY.PEEK/);
  assert.match(rust, /examine\(&folder\)/);
  assert.doesNotMatch(rust, /http:\/\//);
});
test('migration stores account configuration and local drafts but never a password column', () => {
  const db = read('src-tauri/src/db.rs');
  const rust = read('src-tauri/src/business_mail.rs');
  assert.match(db, /SCHEMA_VERSION: i64 = 16/);
  assert.match(db, /CREATE TABLE IF NOT EXISTS mail_accounts/);
  assert.match(db, /CREATE TABLE IF NOT EXISTS mail_drafts/);
  assert.match(db, /references mail_accounts/i);
  assert.doesNotMatch(db.slice(db.indexOf('if version < 16 {'),db.indexOf('// Certificates are append-only')), /password|secret/i);
  const accountBody = rust.slice(rust.indexOf('pub struct MailAccount {'),rust.indexOf('pub struct MailAccountInput'));
  assert.doesNotMatch(accountBody,/password|secret/i);
  assert.match(rust, /pub struct MailAccountInput/);
  assert.match(rust,/set_password/);
  assert.match(rust,/get_password/);
});
test('every new email label has an English fallback and translations', () => {
  const src = read('src/app/business-mail.component.ts');
  const html = read('src/app/business-mail.component.html');
  const dict = read('src/app/mail-i18n.ts');
  const keys = [...new Set(
    [...src.matchAll(/\bt\('([^']+)'\)/g),...html.matchAll(/\bt\('([^']+)'\)/g)]
      .map(m => m[1])
  )];
  for (const k of keys) {
    assert.ok(dict.includes(JSON.stringify(k)+':') ||
      dict.includes("'" + k + "':"), 'Missing email translation: ' + k);
  }
  const main = read('src/app/i18n.ts');
  for (const label of ['Business email','Correo profesional','Courrier professionnel','Geschäftliche E-Mails'])
    assert.ok(main.includes(label),label);
});
test('mail UI requires explicit send confirmation and never embeds remote HTML', () => {
  const src = read('src/app/business-mail.component.ts');
  const html = read('src/app/business-mail.component.html');
  assert.match(src,/window\.confirm\(this\.t\('Send this email now/);
  assert.match(src,/confirm:true/);
  assert.match(html,/type="password"/);
  assert.match(html,/\{\{msg\.body\}\}/);
  assert.doesNotMatch(html,/\[innerHTML\]/);
  assert.match(src,/mailSaveDraft/);
  assert.match(src,/hasUnsavedComposer/);
  assert.match(src,/mailReadMessage/);
});
