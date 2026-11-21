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
  const implementation = rust.split('#[cfg(test)]')[0];
  assert.doesNotMatch(implementation, /fs::remove_file|fs::write|\.execute\(/);
});

test('the audit exposes a bounded report and distinguishes file presence from hashes', () => {
  const rust = source('src-tauri/src/media_audit.rs');
  const html = source('src/app/app.component.html');
  assert.match(rust, /MAX_VISIBLE_ISSUES: usize = 100/);
  assert.match(rust, /result\.truncated = true/);
  assert.match(html, /mediaAudit as audit/);
  assert.match(html, /non confronta hash storici/);
});

test('the active audit includes historic certificate images with their own visible role', () => {
  const rust = source('src-tauri/src/media_audit.rs');
  assert.match(rust, /SELECT artwork_id, id, snapshot_json FROM certificates/);
  assert.match(rust, /role: "certificate"\.into\(\)/);
  assert.match(source('src/app/models.ts'), /'primary' \| 'gallery' \| 'certificate'/);
  assert.match(source('src/app/app.component.html'), /Fotografia del certificato/);
  assert.match(source('src/app/i18n.ts'), /"Fotografia del certificato": "Certificate photograph"/);
});

test('historic certificate snapshots with invalid JSON or image paths are surfaced without exposing their contents', () => {
  const rust = source('src-tauri/src/media_audit.rs');
  assert.match(rust, /fn malformed_certificate_snapshots_are_reported_without_exposing_or_changing_json/);
  assert.match(rust, /problem: "invalid_snapshot"/);
  assert.match(rust, /path: "snapshot_json"\.into\(\)/);
  assert.match(source('src/app/models.ts'), /'invalid_snapshot'/);
  assert.match(source('src/app/app.component.ts'), /invalid_snapshot: 'Snapshot del certificato non valido'/);
  const translations = source('src/app/i18n.ts');
  assert.equal((translations.match(/"Snapshot del certificato non valido":/g) || []).length, 4);
});
