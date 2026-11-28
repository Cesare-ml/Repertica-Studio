import type { Artwork } from './models';

/** A missing historical photograph is not the same as a certificate issued without one. */
export async function loadCertificatePreview(
  snapshotJson: string, artworkId: string, read: () => Promise<string | null>,
): Promise<{ snapshot: Artwork; image: string | null }> {
  const value: unknown = JSON.parse(snapshotJson);
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid snapshot');
  const snapshot = value as Artwork;
  if (snapshot.id !== artworkId || typeof snapshot.title !== 'string' || typeof snapshot.code !== 'string' ||
      (snapshot.image_path != null && (typeof snapshot.image_path !== 'string' || !snapshot.image_path.trim())) ||
      [snapshot.year, snapshot.technique, snapshot.support].some(v => v != null && typeof v !== 'string') ||
      [snapshot.height_cm, snapshot.width_cm, snapshot.depth_cm].some(v => v != null && (typeof v !== 'number' || !Number.isFinite(v) || v < 0))) {
    throw new Error('Invalid snapshot');
  }
  const image = await read();
  if (snapshot.image_path != null ? !image : image !== null) throw new Error('Historical photograph mismatch');
  return { snapshot, image };
}
