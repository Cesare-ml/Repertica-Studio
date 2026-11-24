import type { AuctionDraft, NativeDraftSnapshot, PreparedPreview, StoredAuctionDraft, StoredPreviewReceipt } from './auction-preview';

export type SelectedAuctionArtwork = Readonly<{ id: string; title: string; status: string }>;
export interface AuctionEditorStorage {
  read(artworkId: string): Promise<NativeDraftSnapshot>;
  decode(snapshot: NativeDraftSnapshot, artworkId: string): StoredAuctionDraft | null;
  newDraft(artwork: SelectedAuctionArtwork): AuctionDraft;
  save(snapshot: NativeDraftSnapshot, draft: AuctionDraft): Promise<NativeDraftSnapshot>;
}

interface Session {
  artwork: SelectedAuctionArtwork;
  snapshot: NativeDraftSnapshot;
  draft: AuctionDraft;
  pending: PreparedPreview | null;
  receipt?: StoredPreviewReceipt;
  dirty: boolean;
}

/** A single-work offline editor. No tenant selection, send, attempt creation or token access. */
export class AuctionEditorSession {
  private session: Session | null = null;
  private inFlight = false;
  private readonly storage: AuctionEditorStorage;

  constructor(storage: AuctionEditorStorage) { this.storage = storage; }

  get busy(): boolean { return this.inFlight; }
  get view() {
    const s = this.session;
    return s && { artwork: { ...s.artwork }, draft: { ...s.draft },
      pending: s.pending ? { ...s.pending } : null, receipt: s.receipt ? { ...s.receipt } : undefined, dirty: s.dirty,
      saved: s.snapshot.revision !== null };
  }

  private ensureIdle(): void {
    if (this.inFlight) throw new Error('auction_editor_busy');
  }

  async open(artwork: SelectedAuctionArtwork): Promise<void> {
    this.ensureIdle();
    if (this.session?.dirty) throw new Error('auction_editor_unsaved');
    const selected = { id: artwork.id, title: artwork.title, status: artwork.status };
    this.inFlight = true;
    try {
      const snapshot = await this.storage.read(selected.id);
      const existing = this.storage.decode(snapshot, selected.id);
      const draft = existing?.draft ?? this.storage.newDraft(selected);
      // Never silently truncate a source title into a public-facing title.
      // The original stays visible as artwork.title; the artist must edit the draft.
      if (!existing && draft.title.length > 160) draft.title = '';
      this.session = { artwork: selected, snapshot, draft: { ...draft },
        pending: existing?.pending ?? null, receipt: existing?.receipt, dirty: false };
    } finally { this.inFlight = false; }
  }

  edit(fields: Omit<AuctionDraft, 'artworkId'>): void {
    this.ensureIdle();
    const s = this.session;
    if (!s) throw new Error('auction_editor_closed');
    // Only editor fields; do not accept an artwork ID, pending request or other record fields.
    const next: AuctionDraft = { artworkId: s.artwork.id, title: fields.title,
      description: fields.description, slug: fields.slug, startsAt: fields.startsAt,
      endsAt: fields.endsAt, startingAmount: fields.startingAmount,
      minimumIncrementAmount: fields.minimumIncrementAmount };
    s.dirty = s.dirty || JSON.stringify(next) !== JSON.stringify(s.draft);
    s.draft = next;
  }

  async save(): Promise<void> {
    this.ensureIdle();
    const s = this.session;
    if (!s) throw new Error('auction_editor_closed');
    this.inFlight = true;
    try {
      // Use the original loaded revision, never silently reload and retry a conflicting save.
      const saved = await this.storage.save(s.snapshot, { ...s.draft });
      const decoded = this.storage.decode(saved, s.artwork.id);
      if (!decoded || saved.scope !== s.snapshot.scope || !saved.revision ||
          decoded.pending?.body !== s.pending?.body ||
          decoded.pending?.idempotencyKey !== s.pending?.idempotencyKey ||
          JSON.stringify(decoded.receipt) !== JSON.stringify(s.receipt)) {
        throw new Error('auction_draft_write_failed');
      }
      this.session = { ...s, snapshot: saved, draft: { ...decoded.draft },
        pending: decoded.pending, receipt: decoded.receipt, dirty: false };
    } finally { this.inFlight = false; }
  }

  /** Discard is explicit and affects only unsaved editor fields, never the database. */
  close(discardUnsaved = false): void {
    this.ensureIdle();
    if (this.session?.dirty && !discardUnsaved) throw new Error('auction_editor_unsaved');
    this.session = null;
  }
}
