/** Geometry of the magnifier of the full-screen viewer (ported from the Gestionale Opere preview). */
export interface ZoomGeometry {
  lens: Record<string, string>;
  pane: Record<string, string>;
}

export const LENS_SIZE = 150;
export const PANE_SIZE = 340;

/**
 * `x`/`y` are the pointer coordinates relative to the displayed picture, `width`/`height` its displayed size
 * and `naturalWidth`/`naturalHeight` the size of the file. The side pane shows a detail of the picture
 * at least four times larger than on screen (or 1.6 times the real size, whichever is bigger).
 */
export function zoomGeometry(x: number, y: number, width: number, height: number, naturalWidth: number, naturalHeight: number): ZoomGeometry | null {
  if (!(width > 0) || !(height > 0)) return null;
  const px = Math.min(Math.max(x, 0), width);
  const py = Math.min(Math.max(y, 0), height);
  const backgroundWidth = Math.max(width * 4, (naturalWidth || width) * 1.6);
  const backgroundHeight = Math.max(height * 4, (naturalHeight || height) * 1.6);
  const scaleX = backgroundWidth / width;
  const scaleY = backgroundHeight / height;
  return {
    lens: {
      left: `${px - LENS_SIZE / 2}px`, top: `${py - LENS_SIZE / 2}px`,
      width: `${LENS_SIZE}px`, height: `${LENS_SIZE}px`,
    },
    pane: {
      backgroundPosition: `${-(px * scaleX - PANE_SIZE / 2)}px ${-(py * scaleY - PANE_SIZE / 2)}px`,
      backgroundSize: `${backgroundWidth}px ${backgroundHeight}px`,
    },
  };
}
