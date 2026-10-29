export type Point = { x: number; y: number };
export type Rect = Point & { w: number; h: number };
export const clamp = (n: number, min: number, max: number) => Math.max(min, Math.min(n, Math.max(min, max)));
export function selectionBetween(a: Point, b: Point): Rect {
  return { x: Math.min(a.x,b.x), y: Math.min(a.y,b.y), w: Math.abs(a.x-b.x), h: Math.abs(a.y-b.y) };
}
export function resizeSelection(rect: Rect, edge: string, delta: Point, screen: Point): Rect {
  let { x,y,w,h } = rect; let right=x+w, bottom=y+h;
  if(edge.includes('w')) x=clamp(x+delta.x,0,right-12);
  if(edge.includes('e')) right=clamp(right+delta.x,x+12,screen.x);
  if(edge.includes('n')) y=clamp(y+delta.y,0,bottom-12);
  if(edge.includes('s')) bottom=clamp(bottom+delta.y,y+12,screen.y);
  return { x,y,w:right-x,h:bottom-y };
}
export function toolbarPosition(rect: Rect, width: number, height: number, screen: Point) {
  let top=rect.y+rect.h+14;
  if(top+height+52>screen.y-12) top=rect.y-height-14;
  return { left:clamp(rect.x+rect.w-width,10,screen.x-width-10), top:clamp(top,10,screen.y-height-60) };
}
/** Snapshot pixels can differ from CSS pixels on Retina and mixed-DPI displays. */
export function sourceRect(rect: Rect, viewport: Point, image: Point): Rect {
  const sx=image.x/viewport.x, sy=image.y/viewport.y;
  return { x:rect.x*sx,y:rect.y*sy,w:rect.w*sx,h:rect.h*sy };
}
