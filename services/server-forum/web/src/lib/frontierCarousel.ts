export function carouselIndex(current: number, step: number, count: number): number {
  return count > 0 ? ((current + step) % count + count) % count : 0
}
export function swipeStep(dx: number, dy: number): number {
  return Math.abs(dx) >= 40 && Math.abs(dx) > Math.abs(dy) * 1.3 ? (dx < 0 ? 1 : -1) : 0
}
