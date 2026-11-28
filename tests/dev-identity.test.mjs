import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import test from 'node:test';

// Development runs must never open the owner's real archive (%LOCALAPPDATA%\it.repertica.studio):
// on 2026-11-27 a development build with a newer schema migrated it and the installed release refused to start.
const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const json = path => JSON.parse(read(path));
const RELEASE = 'it.repertica.studio';
const DEV = 'it.repertica.studio.dev';

test('the release keeps its identifier and npm run tauri:dev uses the development one', () => {
  assert.equal(json('src-tauri/tauri.conf.json').identifier, RELEASE);
  assert.equal(json('src-tauri/tauri.release.conf.json').identifier, undefined, 'the release override never changes the identifier');
  assert.equal(json('src-tauri/tauri.dev.conf.json').identifier, DEV);

  const scripts = json('package.json').scripts;
  assert.match(scripts['tauri:dev'], /^tauri dev\b.*--config src-tauri\/tauri\.dev\.conf\.json/);
  assert.doesNotMatch(scripts['tauri:build'], /tauri\.dev\.conf\.json/);

  const release = read('.github/workflows/release.yml');
  assert.match(release, /--config src-tauri\/tauri\.release\.conf\.json/);
  assert.doesNotMatch(release, /tauri\.dev\.conf\.json/);
});

test('no platform-specific config silently overrides the identifier', () => {
  // Tauri merges tauri.<platform>.conf.json automatically; build.rs only reads tauri.conf.json and --config.
  for (const name of readdirSync(new URL('../src-tauri/', import.meta.url))) {
    if (!/^tauri\.(windows|macos|linux|ios|android)\.conf\.json$/.test(name)) continue;
    assert.equal(json('src-tauri/' + name).identifier, undefined, name);
  }
});

test('names outside the data folder come from the build identifier, not from a literal', () => {
  const identity = read('src-tauri/src/identity.rs');
  assert.match(identity, new RegExp(`RELEASE_IDENTIFIER: &str = "${RELEASE.replaceAll('.', '\\.')}"`));
  assert.match(identity, /env!\("REPERTICA_APP_IDENTIFIER"\)/);
  assert.match(read('src-tauri/build.rs'), /cargo:rustc-env=REPERTICA_APP_IDENTIFIER=/);

  const lib = read('src-tauri/src/lib.rs');
  assert.match(lib, /identity::startup_check\(cfg!\(debug_assertions\)/, 'a debug build refuses the release identifier');
  assert.match(lib, /app\.config\(\)\.identifier != identity::IDENTIFIER/, 'compiled and runtime identifiers agree');

  assert.match(read('src-tauri/src/mcp.rs'), /base\.join\(crate::identity::IDENTIFIER\)/, 'MCP default folder');
  assert.match(read('src-tauri/src/mcp_client.rs'), /identity::scoped\(SERVER_NAME\)/, 'MCP entry in the assistants');
  assert.match(read('src-tauri/src/business_mail.rs'), /identity::scoped\(KEYRING_SERVICE\)/, 'mail keychain service');
  assert.match(read('src-tauri/src/backup.rs'), /identity::scoped\(ROLLBACK_FOLDER\)/, 'restore rollback folder');
  assert.match(read('src-tauri/src/updater.rs'), /identity::is_release\(\) && !PUBLIC_KEY/, 'updates only in the release');
  assert.match(read('src-tauri/src/auto_backup.rs'), /identity::backup_timer_allowed\(/, 'automatic backup timer');

  // Rust sources outside identity.rs never spell the release identifier as a string literal.
  for (const name of readdirSync(new URL('../src-tauri/src/', import.meta.url))) {
    if (!name.endsWith('.rs') || name === 'identity.rs') continue;
    assert.ok(!read('src-tauri/src/' + name).includes(`"${RELEASE}"`), name);
  }
});

test('the MCP snippets shown in Settings use the entry name of this build', () => {
  const app = read('src/app/app.component.ts');
  assert.doesNotMatch(app, /'repertica-studio'|mcp_servers\.repertica-studio|`repertica-studio --/);
  assert.match(app, /\[setup\.serverName\]/);
  assert.match(read('src/app/models.ts'), /interface McpSetup \{ serverName: string;/);
});
