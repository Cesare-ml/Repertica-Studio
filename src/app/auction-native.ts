/**
 * Bridge to the native Auctions connection (src-tauri/src/auction_online.rs).
 *
 * The webview never sees the Repertica access token, never chooses the server origin and never talks to the
 * network itself (CSP connect-src 'self'): it asks the native side for one artist-v1 operation at a time.
 * `invoke` is injected so the bridge is testable without Tauri.
 */
import type { HttpPort, HttpRequest, HttpResponse } from './auction-http-transport';

export type Invoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;

export interface ConnectionStatus {
  /** The service can be reached from this build (off until Auctions and the Studio sign-in are live). */
  readonly available: boolean;
  readonly signedIn: boolean;
  readonly expiresInSeconds: number | null;
  /** Debug build pointed at local synthetic services. */
  readonly development: boolean;
  readonly signingIn: boolean;
}

const OFFLINE: ConnectionStatus = Object.freeze({ available: false, signedIn: false, expiresInSeconds: null, development: false, signingIn: false });

export function parseStatus(value: unknown): ConnectionStatus {
  const v = value !== null && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : null;
  if (!v || typeof v['available'] !== 'boolean' || typeof v['signedIn'] !== 'boolean' || typeof v['development'] !== 'boolean'
      || typeof v['signingIn'] !== 'boolean') throw new Error('invalid_response');
  const seconds = v['expiresInSeconds'];
  if (seconds !== null && (typeof seconds !== 'number' || !Number.isSafeInteger(seconds) || seconds < 0)) throw new Error('invalid_response');
  return Object.freeze({ available: v['available'], signedIn: v['available'] && v['signedIn'], expiresInSeconds: seconds as number | null,
    development: v['development'], signingIn: v['signingIn'] });
}

/** Errors the native side reports instead of an HTTP answer. */
export function nativeErrorCode(error: unknown): string {
  if (typeof error === 'string' && /^[a-z_]{3,40}$/.test(error)) return error;
  if (error instanceof Error && /^[a-z_]{3,40}$/.test(error.message)) return error.message;
  return 'native_unavailable';
}

export function createNativeConnection(invoke: Invoke) {
  return Object.freeze({
    async status(): Promise<ConnectionStatus> {
      try { return parseStatus(await invoke('auction_connection_status')); } catch { return OFFLINE; }
    },
    /** Opens the system browser on the Repertica sign-in; resolves when the browser comes back (max 5 minutes). */
    async signIn(): Promise<ConnectionStatus> {
      try { return parseStatus(await invoke('auction_sign_in')); }
      catch (error) { throw new Error(nativeErrorCode(error)); }
    },
    async signOut(): Promise<ConnectionStatus> {
      try { return parseStatus(await invoke('auction_sign_out')); } catch { return OFFLINE; }
    },
    /** HttpPort for createArtistV1Transport. A native error means the answer is lost or never sent. */
    port: (async (request: HttpRequest): Promise<HttpResponse> => {
      const result = await invoke('auction_api_request', {
        method: request.method, path: request.path, body: request.body ?? null,
        idempotencyKey: request.headers['Idempotency-Key'] ?? null,
      }) as { status?: unknown; body?: unknown } | null;
      if (!result || typeof result.status !== 'number' || !Number.isInteger(result.status)) throw new Error('invalid_response');
      return { status: result.status, body: result.body ?? null };
    }) satisfies HttpPort,
  });
}
export type NativeConnection = ReturnType<typeof createNativeConnection>;
