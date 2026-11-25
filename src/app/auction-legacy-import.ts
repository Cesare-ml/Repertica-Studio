import type { LocalAuctionDraft } from './auction-drafts';
import type { AuctionDraft, NativeDraftSnapshot } from './auction-preview';

/** Preview only: legacy storage has no archive identity. The artist must review
 * the selected local artwork and explicitly save using this same empty snapshot.
 * Neither the original legacy record nor an existing SQLite draft is changed. */
export function previewLegacyDraft(legacy: LocalAuctionDraft,
  artwork: { id: string; status: string }, snapshot: NativeDraftSnapshot,
  expectedScope: string): AuctionDraft {
  if (!expectedScope || snapshot.scope !== expectedScope || snapshot.artworkId !== artwork.id ||
      snapshot.revision !== null || snapshot.recordJson !== null) throw new Error('auction_draft_conflict');
  if (artwork.status !== 'available' || legacy.artworkId !== artwork.id || legacy.local !== true) {
    throw new Error('artwork_not_available');
  }
  return { artworkId: artwork.id, title: legacy.title, description: legacy.description,
    slug: legacy.slug, startsAt: legacy.startsAt, endsAt: legacy.endsAt,
    startingAmount: legacy.startingAmount, minimumIncrementAmount: legacy.minimumIncrementAmount };
}
