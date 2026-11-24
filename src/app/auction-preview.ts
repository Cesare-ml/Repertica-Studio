/** Offline-only boundary for one explicitly selected artwork. No transport or credentials. */
export interface AuctionDraft {
  readonly artworkId: string;
  title: string;
  description: string;
  slug: string;
  startsAt: string;
  endsAt: string;
  startingAmount: number | null;
  minimumIncrementAmount: number | null;
}

export interface PreparedPreview {
  readonly artworkId: string;
  readonly idempotencyKey: string;
  readonly body: string;
}

// Private archive description, location, images, prices and buyer information are not defaults.
export function newAuctionDraft(artwork: { id: string; title: string; status: string }): AuctionDraft {
  if (!artwork.id || artwork.status !== 'available') throw new Error('artwork_not_available');
  return { artworkId: artwork.id, title: artwork.title, description: '', slug: '',
    startsAt: '', endsAt: '', startingAmount: null, minimumIncrementAmount: null };
}

function utc(value: string): number {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)) {
    throw new Error('invalid_schedule');
  }
  const time = Date.parse(value);
  if (!Number.isFinite(time) || new Date(time).toISOString() !== value) throw new Error('invalid_schedule');
  return time;
}

function cents(value: number | null): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 1 || value > 1_000_000_000) {
    throw new Error('invalid_amount');
  }
  return value;
}

/** Call only after the user reviews the public fields and chooses a tenant.
 * Persist the returned body + key before attempting a request; retries must reuse both.
 * A changed payload is a new attempt, not a retry. Never queue automatic publication.
 */
export function preparePreview(draft: AuctionDraft, tenantId: string, idempotencyKey: string,
  now: number = Date.now()): PreparedPreview {
  if (!draft.artworkId) throw new Error('artwork_not_selected');
  if (!/^[A-Za-z0-9_-]{3,80}$/.test(tenantId)) throw new Error('invalid_tenant');
  if (!/^[A-Za-z0-9_-]{16,96}$/.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(draft.slug) || draft.slug.length > 60) throw new Error('invalid_slug');
  if (draft.title.trim().length < 3 || draft.title.length > 160) throw new Error('invalid_title');
  if (draft.description.length > 1500) throw new Error('invalid_description');
  const start = utc(draft.startsAt), end = utc(draft.endsAt);
  if (!Number.isFinite(now) || start < now || end <= start || end - start > 7 * 86_400_000) {
    throw new Error('invalid_schedule');
  }
  // Explicit allowlist: never spread the draft or the artwork into a network payload.
  const body = JSON.stringify({ tenantId, slug: draft.slug, title: draft.title.trim(),
    description: draft.description.trim(), startsAt: draft.startsAt, endsAt: draft.endsAt,
    startingAmount: cents(draft.startingAmount), minimumIncrementAmount: cents(draft.minimumIncrementAmount),
    currency: 'EUR' });
  if (new TextEncoder().encode(body).length > 8192) throw new Error('payload_too_large');
  return Object.freeze({ artworkId: draft.artworkId, idempotencyKey, body });
}

/** Explicit offline storage boundary. The caller must supply a stable local archive scope
 * and a single-writer storage adapter. No default browser storage, network or credentials.
 * Until wired to native archive storage this is not part of SQLite backups. */
export interface AuctionDraftStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}
export interface StoredAuctionDraft {
  readonly draft: AuctionDraft;
  readonly pending: PreparedPreview | null;
}

function draftStorageKey(scope: string, artworkId: string): string {
  if (!scope || scope.length > 256 || !artworkId || artworkId.length > 256) {
    throw new Error('invalid_draft_scope');
  }
  return 'repertica.auction-draft.v1:' + JSON.stringify([scope, artworkId]);
}

function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('invalid_stored_draft');
  return value as Record<string, unknown>;
}
function stringField(value: unknown, max: number): string {
  if (typeof value !== 'string' || value.length > max) throw new Error('invalid_stored_draft');
  return value;
}
function draftAmount(value: unknown): number | null {
  if (value === null) return null;
  if (typeof value !== 'number' || !Number.isFinite(value)) throw new Error('invalid_stored_draft');
  return value;
}
function projectDraft(value: unknown): AuctionDraft {
  const d = object(value);
  // An unfinished draft need not be publishable yet. Persist only editor fields.
  return { artworkId: stringField(d['artworkId'], 256), title: stringField(d['title'], 160),
    description: stringField(d['description'], 1500), slug: stringField(d['slug'], 60),
    startsAt: stringField(d['startsAt'], 64), endsAt: stringField(d['endsAt'], 64),
    startingAmount: draftAmount(d['startingAmount']), minimumIncrementAmount: draftAmount(d['minimumIncrementAmount']) };
}
function restorePending(value: unknown, artworkId: string): PreparedPreview {
  const p = object(value);
  const id = stringField(p['artworkId'], 256), key = stringField(p['idempotencyKey'], 96);
  const body = stringField(p['body'], 8192);
  if (id !== artworkId) throw new Error('invalid_stored_draft');
  const payload = object(JSON.parse(body));
  const draft = projectDraft({ ...payload, artworkId });
  // Validate against the original start, not today's clock: an already sent request
  // may need reconciliation after its scheduled start. Never regenerate the key/body.
  const expected = preparePreview(draft, stringField(payload['tenantId'], 80), key, utc(draft.startsAt));
  if (expected.body !== body) throw new Error('invalid_stored_draft');
  return Object.freeze({ artworkId, idempotencyKey: key, body });
}

export function loadAuctionDraft(storage: AuctionDraftStorage, scope: string, artworkId: string): StoredAuctionDraft | null {
  const raw = storage.getItem(draftStorageKey(scope, artworkId));
  if (raw === null) return null;
  if (raw.length > 64_000) throw new Error('invalid_stored_draft');
  const record = object(JSON.parse(raw));
  if (record['version'] !== 1 || record['scope'] !== scope) throw new Error('invalid_stored_draft');
  const draft = projectDraft(record['draft']);
  if (draft.artworkId !== artworkId) throw new Error('invalid_stored_draft');
  const pending = record['pending'] === null ? null : restorePending(record['pending'], artworkId);
  return { draft, pending };
}

/** A saved pending request survives editor changes and cannot be replaced before
 * reconciliation. The caller must not send if setItem throws. No send is performed here. */
export function saveAuctionDraft(storage: AuctionDraftStorage, scope: string, input: AuctionDraft,
  attempt?: PreparedPreview): StoredAuctionDraft {
  const draft = projectDraft(input);
  const key = draftStorageKey(scope, draft.artworkId);
  const previous = loadAuctionDraft(storage, scope, draft.artworkId);
  let pending = previous?.pending ?? null;
  if (attempt !== undefined) {
    const candidate = restorePending(attempt, draft.artworkId);
    if (pending && (pending.body !== candidate.body || pending.idempotencyKey !== candidate.idempotencyKey)) {
      throw new Error('pending_preview_unresolved');
    }
    if (!pending) {
      const payload = object(JSON.parse(candidate.body));
      const reviewed = preparePreview(draft, stringField(payload['tenantId'], 80), candidate.idempotencyKey,
        utc(draft.startsAt));
      if (reviewed.body !== candidate.body) throw new Error('preview_draft_changed');
    }
    pending = candidate;
  }
  storage.setItem(key, JSON.stringify({ version: 1, scope, draft, pending }));
  return { draft, pending };
}


/** Async native bridge: the read provides both archive identity and CAS revision.
 * No browser storage and no retry of a stale write. A successful save is NOT publication. */
export interface NativeDraftSnapshot {
  readonly scope: string;
  readonly revision: string | null;
  readonly recordJson: string | null;
}
export interface NativeDraftPort {
  read(artworkId: string): Promise<NativeDraftSnapshot>;
  write(input: { artworkId: string; scope: string; expectedRevision: string | null; recordJson: string }): Promise<NativeDraftSnapshot>;
}

export function decodeNativeDraft(snapshot: NativeDraftSnapshot, artworkId: string): StoredAuctionDraft | null {
  if ((snapshot.revision === null) !== (snapshot.recordJson === null)) throw new Error('invalid_stored_draft');
  return loadAuctionDraft({ getItem: () => snapshot.recordJson, setItem: () => { throw new Error('read_only'); } },
    snapshot.scope, artworkId);
}

/** Use the snapshot from when the editor loaded, not a newly fetched revision.
 * The caller must reload/review after auction_draft_conflict, never silently overwrite. */
export async function saveNativeAuctionDraft(port: NativeDraftPort, snapshot: NativeDraftSnapshot,
  input: AuctionDraft, attempt?: PreparedPreview): Promise<NativeDraftSnapshot> {
  decodeNativeDraft(snapshot, input.artworkId);
  let recordJson = snapshot.recordJson;
  saveAuctionDraft({ getItem: () => recordJson, setItem: (_key, value) => { recordJson = value; } },
    snapshot.scope, input, attempt);
  if (recordJson === null) throw new Error('invalid_stored_draft');
  const saved = await port.write({ artworkId: input.artworkId, scope: snapshot.scope,
    expectedRevision: snapshot.revision, recordJson });
  if (saved.scope !== snapshot.scope || saved.recordJson !== recordJson || !saved.revision) {
    throw new Error('auction_draft_write_failed');
  }
  return saved;
}
