export interface MorphRect { left: number; top: number; width: number; height: number }
/** All keyframes remain inside the visible composer, including at screen edges. */
export function morphGeometry(rect: MorphRect, right: boolean) {
  const size = Math.min(48, rect.width, rect.height);
  const full = { left: rect.left, top: rect.top, width: rect.width, height: rect.height, borderRadius: '32px' };
  const ball = { left: right ? rect.left + rect.width - size : rect.left, top: rect.top + (rect.height - size) / 2, width: size, height: size, borderRadius: '50%' };
  return { full, ball };
}
export const MORPH_DURATION = 260;
