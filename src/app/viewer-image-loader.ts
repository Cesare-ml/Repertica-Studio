/**
 * Load an artwork's cover and gallery for the fullscreen viewer.
 *
 * A missing/corrupt cover must not hide valid gallery photographs. Each image is
 * independently recoverable, and a partial result is deliberately not cacheable:
 * returning to the artwork should retry a temporarily unavailable file.
 *
 * This module has no Angular/Tauri dependency so the failure and navigation paths
 * can be exercised with Node tests.
 */
export interface ViewerImage {
  url: string;
  title: string;
  caption: string | null;
}

export interface ViewerGalleryReference {
  id: string;
  kind: string;
  caption: string | null;
}

export interface ViewerImageReader {
  cover: (artworkId: string) => Promise<string | null>;
  gallery: (artworkId: string) => Promise<readonly ViewerGalleryReference[]>;
  photo: (imageId: string) => Promise<string>;
}

export interface ViewerImageLabels {
  cover: string;
  photograph: string;
  kind: (kind: string) => string | null;
}

export interface ViewerImageResult {
  images: ViewerImage[];
  error: string;
  /** False when one or more reads failed, or when browsing moved elsewhere. */
  complete: boolean;
}

export async function loadViewerImages(
  artworkId: string,
  hasCover: boolean,
  reader: ViewerImageReader,
  labels: ViewerImageLabels,
  stillCurrent: () => boolean,
): Promise<ViewerImageResult> {
  const images: ViewerImage[] = [];
  let firstError = '';
  let failed = false;
  const stopped = (): ViewerImageResult => ({ images: [], error: '', complete: false });
  const recordError = (error: unknown): void => {
    failed = true;
    if (!firstError) firstError = String(error) || 'Immagine non disponibile';
  };

  if (!stillCurrent()) return stopped();
  if (hasCover) {
    try {
      const cover = await reader.cover(artworkId);
      if (!stillCurrent()) return stopped();
      if (cover) images.push({ url: cover, title: labels.cover, caption: null });
      else recordError('Copertina non disponibile');
    } catch (error) {
      if (!stillCurrent()) return stopped();
      recordError(error);
    }
  }

  let gallery: readonly ViewerGalleryReference[];
  try {
    gallery = await reader.gallery(artworkId);
  } catch (error) {
    if (!stillCurrent()) return stopped();
    recordError(error);
    return { images, error: firstError, complete: false };
  }
  if (!stillCurrent()) return stopped();

  for (const item of gallery) {
    try {
      const url = await reader.photo(item.id);
      if (!stillCurrent()) return stopped();
      if (!url) {
        recordError('Fotografia non disponibile');
        continue;
      }
      images.push({
        url,
        title: labels.kind(item.kind) ?? labels.photograph,
        caption: item.caption,
      });
    } catch (error) {
      if (!stillCurrent()) return stopped();
      recordError(error);
      // A bad gallery file must not prevent the remaining photos from loading.
    }
  }
  return { images, error: firstError, complete: !failed };
}
