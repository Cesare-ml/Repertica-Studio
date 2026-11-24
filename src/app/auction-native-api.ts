import { invoke } from '@tauri-apps/api/core';
import type { NativeDraftPort, NativeDraftSnapshot } from './auction-preview';

/** Local IPC only; deliberately not wired to any HTTP transport or automatic send. */
export const nativeAuctionDrafts: NativeDraftPort = {
  read: artworkId => invoke<NativeDraftSnapshot>('read_auction_draft', { artworkId }),
  write: input => invoke<NativeDraftSnapshot>('write_auction_draft', { input }),
};
