import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');

test('the active-archive media audit uses a read-only local Tauri command', () => {
  const rust = source('src-tauri/src/media_audit.rs');
  assert.match(source('src-tauri/src/lib.rs'), /media_audit::audit_artwork_media/);
  assert.match(source('src/app/api.ts'), /invoke<MediaAudit>\('audit_artwork_media'\)/);
  assert.match(source('src/app/app.component.html'), /verifyArtworkMedia\(\)/);
  assert.match(rust, /symlink_metadata/);
  assert.match(rust, /unsafe_path/);
  assert.match(rust, /#\[test\]/);
  assert.doesNotMatch(rust, /fs::remove_file|fs::write|\.execute\(/);
});

test('the audit exposes a bounded report and distinguishes file presence from hashes', () => {
  const rust = source('src-tauri/src/media_audit.rs');
  const html = source('src/app/app.component.html');
  assert.match(rust, /MAX_VISIBLE_ISSUES: usize = 100/);
  assert.match(rust, /result\.truncated = true/);
  assert.match(html, /mediaAudit as audit/);
  assert.match(html, /non confronta hash storici/);
});
