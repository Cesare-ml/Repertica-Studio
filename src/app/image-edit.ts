/**
 * Pure rules of the image editor: default selections, dragging of crop and perspective handles,
 * and the conversion of what the user set into the list of operations the native engine applies.
 * Coordinates are relative to the image (0..1).
 */
import type { EditOperation } from './models';

export interface Box { x: number; y: number; width: number; height: number; }
export interface Pt { x: number; y: number; }
export type EditorTool = 'crop' | 'perspective' | 'adjust';
export type PerspectiveMode = 'auto' | 'custom' | '1:1' | '4:5' | '3:2' | '16:9';

export interface EditState {
  crop: Box;
  perspective: Pt[];
  perspectiveMode: PerspectiveMode;
  customWidth: number | null;
  customHeight: number | null;
  rotation: number;
  flipHorizontal: boolean;
  flipVertical: boolean;
  exposure: number;
  contrast: number;
  saturation: number;
}

export const MIN_CROP = 0.03;
export const clamp01 = (value: number): number => Math.min(1, Math.max(0, Number.isFinite(value) ? value : 0));

export const defaultCrop = (): Box => ({ x: 0.1, y: 0.1, width: 0.8, height: 0.8 });
export const defaultPerspective = (): Pt[] => [{ x: 0.1, y: 0.1 }, { x: 0.9, y: 0.1 }, { x: 0.9, y: 0.9 }, { x: 0.1, y: 0.9 }];

export function emptyEditState(): EditState {
  return { crop: defaultCrop(), perspective: defaultPerspective(), perspectiveMode: 'auto', customWidth: null, customHeight: null,
    rotation: 0, flipHorizontal: false, flipVertical: false, exposure: 0, contrast: 0, saturation: 0 };
}

export const cropChanged = (box: Box): boolean => {
  const d = defaultCrop();
  return (['x', 'y', 'width', 'height'] as const).some(k => Math.abs(box[k] - d[k]) > 1e-6);
};
export const perspectiveChanged = (points: Pt[]): boolean =>
  defaultPerspective().some((p, i) => Math.abs(p.x - points[i].x) > 1e-6 || Math.abs(p.y - points[i].y) > 1e-6);

/** The largest centred box with the given pixel aspect ratio (width / height). */
export function cropForRatio(ratio: number, imageWidth: number, imageHeight: number): Box {
  if (!(ratio > 0) || !(imageWidth > 0) || !(imageHeight > 0)) return defaultCrop();
  const widthPerHeight = (ratio * imageHeight) / imageWidth;   // normalised width / normalised height
  const width = widthPerHeight >= 1 ? 1 : widthPerHeight;
  const height = widthPerHeight >= 1 ? 1 / widthPerHeight : 1;
  return { x: (1 - width) / 2, y: (1 - height) / 2, width, height };
}

/** Drag one corner (0 top-left, 1 top-right, 2 bottom-right, 3 bottom-left) keeping the box inside the image. */
export function moveCropCorner(box: Box, corner: number, point: Pt): Box {
  const p = { x: clamp01(point.x), y: clamp01(point.y) };
  const right = box.x + box.width;
  const bottom = box.y + box.height;
  const left = Math.min(p.x, right - MIN_CROP);
  const top = Math.min(p.y, bottom - MIN_CROP);
  const newRight = Math.max(MIN_CROP + box.x, p.x);
  const newBottom = Math.max(MIN_CROP + box.y, p.y);
  switch (corner) {
    case 0: return { x: left, y: top, width: right - left, height: bottom - top };
    case 1: return { x: box.x, y: top, width: newRight - box.x, height: bottom - top };
    case 2: return { x: box.x, y: box.y, width: newRight - box.x, height: newBottom - box.y };
    default: return { x: left, y: box.y, width: right - left, height: newBottom - box.y };
  }
}

export function perspectiveRatio(state: EditState): number | null {
  const fixed: Record<string, number> = { '1:1': 1, '4:5': 4 / 5, '3:2': 3 / 2, '16:9': 16 / 9 };
  if (state.perspectiveMode === 'auto') return null;
  if (state.perspectiveMode !== 'custom') return fixed[state.perspectiveMode];
  const w = Number(state.customWidth);
  const h = Number(state.customHeight);
  return w > 0 && h > 0 ? w / h : null;
}

export type BuildResult = { ok: true; operations: EditOperation[] } | { ok: false; error: string };

export function buildOperations(state: EditState): BuildResult {
  const ratio = perspectiveRatio(state);
  if (state.perspectiveMode === 'custom' && !ratio) return { ok: false, error: 'Inserisci larghezza e altezza dell’opera' };
  const operations: EditOperation[] = [];
  if (perspectiveChanged(state.perspective) || ratio) {
    operations.push({ type: 'perspective', points: state.perspective.map(p => ({ x: clamp01(p.x), y: clamp01(p.y) })), outputAspectRatio: ratio });
  } else if (cropChanged(state.crop)) {
    operations.push({ type: 'crop', ...state.crop });
  }
  if (state.rotation) operations.push({ type: 'rotate', degrees: state.rotation });
  if (state.flipHorizontal || state.flipVertical) operations.push({ type: 'flip', horizontal: state.flipHorizontal, vertical: state.flipVertical });
  if (state.exposure || state.contrast || state.saturation) {
    operations.push({ type: 'adjust', exposure: state.exposure / 100, contrast: state.contrast / 100, saturation: state.saturation / 100 });
  }
  return operations.length ? { ok: true, operations } : { ok: false, error: 'Nessuna modifica da applicare' };
}

/** CSS used to preview light and colour changes live, before the real edit is rendered. */
export function adjustmentFilter(state: EditState): string {
  return `brightness(${Math.pow(2, state.exposure / 100).toFixed(3)}) contrast(${(1 + state.contrast / 100).toFixed(3)}) saturate(${(1 + state.saturation / 100).toFixed(3)})`;
}
export function adjustmentTransform(state: EditState): string {
  return `rotate(${state.rotation}deg) scale(${state.flipHorizontal ? -1 : 1}, ${state.flipVertical ? -1 : 1})`;
}

/** The artwork-shaped output of a perspective correction, summarised for the interface. */
export function describeOperations(operations: EditOperation[]): string[] {
  return operations.map(op => op.type);
}
