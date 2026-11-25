import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const get = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

const TABS = {
  profile: ['Artista e certificati', 'Lingua dell’app', 'Guida di primo avvio'],
  pricing: ['Prezzi delle opere', 'Ripartizione vendite', 'Tasse e tesoreria'],
  archive: ['Cartella dei dati', 'Codice archivio', 'Diagnostica fotografie'],
  backup: ['Backup automatico', 'Backup del tuo archivio', 'Ripristina o trasferisci un archivio'],
  export: ['Esportazioni'],
  ai: ['Assistenti IA locali (MCP)'],
  about: ['Aggiornamenti', 'Informazioni'],
};

function settingsMarkup() {
  const html = get('src/app/app.component.html');
  const start = html.indexOf("section === 'settings'");
  return html.slice(start, html.indexOf('</section>', start));
}

test('the settings are grouped in seven categories, each with its own cards', () => {
  const markup = settingsMarkup();
  for (const [id, headings] of Object.entries(TABS)) {
    const from = markup.indexOf(`*ngIf="settingsTab === '${id}'"`);
    assert.ok(from > 0, `tab ${id}`);
    const next = markup.indexOf('class="settings-tab"', from + 10);
    const body = markup.slice(from, next > 0 ? next : undefined);
    for (const heading of headings) assert.ok(body.includes(`tr('${heading}`), `${heading} is in ${id}`);
  }
});

test('every setting appears exactly once: nothing was lost or duplicated by the reorganisation', () => {
  const markup = settingsMarkup();
  const all = Object.values(TABS).flat();
  for (const heading of all) {
    const count = markup.split(`<h3>{{ tr('${heading}`).length - 1;
    assert.equal(count, 1, `${heading} appears ${count} times as a card title`);
  }
  for (const action of ['saveArtistProfile', 'selectLanguage', 'reopenGuide', 'saveEconomySettings', 'saveAllocationPlan', 'saveCodePolicy', 'verifyArtworkMedia',
    'saveBackupSchedule', 'makeBackup', 'restoreSelectedBackup', 'checkUpdates', 'connectClient', 'setAutoUpdate']) {
    assert.ok(markup.includes(action), action);
  }
  assert.doesNotMatch(markup, /settings-panel/, 'the old single panel is gone');
});

test('the page has an accessible category list and other screens open the right category', () => {
  const markup = settingsMarkup();
  assert.match(markup, /role="tablist"/);
  assert.match(markup, /role="tabpanel"/);
  assert.match(markup, /\[attr\.aria-selected\]="settingsTab === item\.id"/);
  const html = get('src/app/app.component.html');
  assert.match(html, /goToSettings\('backup'\)/, 'the first-start guide opens Backup');
  assert.match(html, /goToSettings\('profile'\)/, 'the certificate dialog opens Profile');
  assert.match(html, /goToSettings\('about'\)/, 'the update banner opens Updates');
  const ts = get('src/app/app.component.ts');
  for (const id of Object.keys(TABS)) assert.ok(ts.includes(`id: '${id}'`), id);
  assert.match(ts, /goToSettings\(tab: SettingsTab\)/);
});
