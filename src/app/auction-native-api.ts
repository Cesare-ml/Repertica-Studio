import { invoke } from '@tauri-apps/api/core';
import type { NativeDraftPort, NativeDraftSnapshot } from './auction-preview';
import { decodeNativeDraft, newAuctionDraft, saveNativeAuctionDraft } from './auction-preview';
import { AuctionEditorSession } from './auction-editor-session';

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
