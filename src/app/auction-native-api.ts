import { invoke } from '@tauri-apps/api/core';
import type { NativeDraftPort, NativeDraftSnapshot } from './auction-preview';
import { decodeNativeDraft, newAuctionDraft, saveNativeAuctionDraft } from './auction-preview';
import { AuctionEditorSession } from './auction-editor-session';

export interface AuctionDraftSummary {
  readonly artworkId: string;
  readonly title: string;
  readonly artworkStatus: string | null;
  readonly state: 'draft' | 'pending' | 'receipt';
}
export interface AuctionDraftPage {
  readonly scope: string;
  readonly rows: readonly AuctionDraftSummary[];
  readonly nextAfter: string | null;
}

export function listNativeAuctionDrafts(expectedScope: string | null = null,
  after: string | null = null): Promise<AuctionDraftPage> {
  return invoke<AuctionDraftPage>('list_auction_drafts', { expectedScope, after });
}

/** Local IPC only; deliberately not wired to any HTTP transport or automatic send. */
export const nativeAuctionDrafts: NativeDraftPort = {
  read: artworkId => invoke<NativeDraftSnapshot>('read_auction_draft', { artworkId }),
  write: input => invoke<NativeDraftSnapshot>('write_auction_draft', { input }),
};

/** Factory for a future UI; no component mounts or network requests here. */
export function createNativeAuctionEditor(): AuctionEditorSession {
  return new AuctionEditorSession({
    read: artworkId => nativeAuctionDrafts.read(artworkId),
    decode: decodeNativeDraft,
    newDraft: newAuctionDraft,
    save: (snapshot, draft) => saveNativeAuctionDraft(nativeAuctionDrafts, snapshot, draft),
  });
}
