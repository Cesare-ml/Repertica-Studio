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
  assert.match(source('src/app/app.component.html'), /issue\.role === 'certificate' && issue\.imageId/);
  const translations = source('src/app/i18n.ts');
  assert.equal((translations.match(/"Snapshot del certificato non valido":/g) || []).length, 4);
});

test('RAW integrity audit is streamed, read-only and explained in the desktop interface', () => {
  const rust = source('src-tauri/src/media_audit.rs');
  const html = source('src/app/app.component.html');
  const models = source('src/app/models.ts');
  const translations = source('src/app/i18n.ts');
  assert.match(rust, /FROM artwork_raw_files ORDER BY artwork_id/);
  assert.match(rust, /fn check_raw\(/);
  assert.match(rust, /Sha256::new\(\)/);
  assert.match(rust, /64 \* 1024/);
  assert.match(rust, /drop\(conn\)/);
  assert.match(rust, /fn audits_raw_size_and_sha256_and_preserves_original_records/);
  assert.match(models, /'hash_mismatch'/);
  assert.match(models, /'raw'/);
  assert.match(html, /issue\.role === 'raw' \? 'Allegato RAW'/);
  assert.match(html, /Per i file RAW controlla anche dimensione e SHA-256/);
  assert.match(html, /File verificati/);
  assert.equal((translations.match(/"File verificati":/g) || []).length, 4);
  assert.equal((translations.match(/"Allegato RAW":/g) || []).length, 4);
  assert.equal((translations.match(/"Metadati dell’allegato RAW non validi":/g) || []).length, 4);
  assert.equal((translations.match(/"Integrità RAW compromessa \(SHA-256 differente\)":/g) || []).length, 4);
  assert.equal((translations.match(/"Per i file RAW controlla anche dimensione e SHA-256, senza modificare gli allegati\.":/g) || []).length, 4);
});
