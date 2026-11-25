/**
 * Journal of auction operations that may have an uncertain outcome (lost response, restart, timeout).
 * An operation is "confirmed" only when the server state read afterwards shows it; a transport answer alone,
 * or silence, never counts as success. Retries reuse the same idempotency key and the same body.
 */
import type { AuctionBid, AuctionSummary, CancelResult } from './auction-contract';

export type OperationKind = 'publish' | 'cancel_bid';
export type OperationState = 'in_flight' | 'pending_confirmation' | 'confirmed' | 'rejected';

export interface Operation {
  readonly id: string;
  readonly kind: OperationKind;
  /** Slug for a publication, bid id for a cancellation. */
  readonly target: string;
  readonly idempotencyKey: string;
  readonly body: string;
  readonly state: OperationState;
  /** Why it is rejected, as reported by the server. */
  readonly reason: string | null;
  readonly attempts: number;
}

export type TransportResult =
  | { kind: 'response'; status: number }
  | { kind: 'lost' };

export function beginOperation(kind: OperationKind, target: string, idempotencyKey: string, body: string): Operation {
  if (!/^[A-Za-z0-9_-]{16,96}$/.test(idempotencyKey)) throw new Error('invalid_idempotency_key');
  return Object.freeze({ id: `${kind}:${target}:${idempotencyKey}`, kind, target, idempotencyKey, body,
    state: 'in_flight' as const, reason: null, attempts: 1 });
}

/** What the transport said. Never "confirmed": only a refresh of the server state can confirm. */
export function afterTransport(op: Operation, result: TransportResult): Operation {
  if (op.state === 'confirmed' || op.state === 'rejected') return op;
  if (result.kind === 'response' && result.status >= 400 && result.status < 500 && result.status !== 408 && result.status !== 429) {
    return Object.freeze({ ...op, state: 'rejected' as const, reason: `http_${result.status}` });
  }
  return Object.freeze({ ...op, state: 'pending_confirmation' as const });
}

/** Retry of an uncertain operation: same key, same body, never a new request identity. */
export function retry(op: Operation): Operation {
  if (op.state !== 'pending_confirmation') throw new Error('not_retryable');
  return Object.freeze({ ...op, state: 'in_flight' as const, attempts: op.attempts + 1 });
}

export function confirmPublication(op: Operation, serverAuctions: readonly AuctionSummary[]): Operation {
  if (op.kind !== 'publish') throw new Error('wrong_kind');
  if (serverAuctions.some(a => a.slug === op.target)) return Object.freeze({ ...op, state: 'confirmed' as const });
  return op.state === 'rejected' ? op : Object.freeze({ ...op, state: 'pending_confirmation' as const });
}

export function confirmCancellation(op: Operation, serverBids: readonly AuctionBid[], answer: CancelResult | null): Operation {
  if (op.kind !== 'cancel_bid') throw new Error('wrong_kind');
  const bid = serverBids.find(b => b.bidId === op.target);
  if (bid?.state === 'cancelled') return Object.freeze({ ...op, state: 'confirmed' as const });
  if (answer && (answer.outcome === 'not_allowed' || answer.outcome === 'contested')) {
    return Object.freeze({ ...op, state: 'rejected' as const, reason: answer.outcome });
  }
  // Still valid on the server, or not found: not confirmed.
  return Object.freeze({ ...op, state: 'pending_confirmation' as const });
}

/** Plain-language state for the interface (Italian source keys of the UI dictionary). */
export function operationLabel(op: Operation): string {
  switch (op.state) {
    case 'in_flight': return 'Operazione in corso';
    case 'pending_confirmation': return 'Conferma in sospeso: verifica lo stato sul server';
    case 'confirmed': return 'Confermata dal server';
    case 'rejected': return 'Rifiutata dal server';
  }
}
