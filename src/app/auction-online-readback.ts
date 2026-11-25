import type { NativeDraftSnapshot, StoredAuctionDraft } from './auction-preview';
import type { AuctionReceipt } from './auction-connection';

type Confirmed = {action:'prepare';revision:string} | {action:'reconcile';receipt:AuctionReceipt};
const conflict = (): never => { throw new Error('auction_draft_conflict'); };

export function assertSameSnapshot(expected: NativeDraftSnapshot, actual: NativeDraftSnapshot): void {
  if (!expected.revision || expected.recordJson === null ||
      expected.artworkId !== actual.artworkId || expected.scope !== actual.scope ||
      expected.revision !== actual.revision || expected.recordJson !== actual.recordJson) conflict();
}

/** An IPC success is not evidence of durable preparation/receipt. The native
 * record must be read again, scoped to the same selected work and archive.
 * This validates only local readback; it never authorizes or retries a send. */
export function assertPreviewReadback(before: NativeDraftSnapshot, after: NativeDraftSnapshot,
  previous: StoredAuctionDraft, saved: StoredAuctionDraft | null,
  reviewedBody: string, result: Confirmed): void {
  if (!before.revision || !after.revision || before.revision === after.revision ||
      before.artworkId !== after.artworkId || before.scope !== after.scope ||
      after.recordJson === null || after.recordJson === before.recordJson ||
      previous.receipt || !saved?.pending || saved.pending.body !== reviewedBody ||
      saved.draft.artworkId !== before.artworkId ||
      JSON.stringify(previous.draft) !== JSON.stringify(saved.draft)) conflict();
  if (!saved || !saved.pending) return conflict();
  if (result.action === 'prepare') {
    if (previous.pending || saved.receipt || result.revision !== after.revision) conflict();
  } else {
    if (!previous.pending || previous.pending.body !== reviewedBody ||
        previous.pending.idempotencyKey !== saved.pending.idempotencyKey ||
        !saved.receipt || saved.receipt.idempotencyKey !== saved.pending.idempotencyKey ||
        saved.receipt.tenantId !== result.receipt.tenantId ||
        saved.receipt.auctionId !== result.receipt.auctionId ||
        saved.receipt.slug !== result.receipt.slug) conflict();
  }
}
