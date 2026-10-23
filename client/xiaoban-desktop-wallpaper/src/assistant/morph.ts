/**
 * The ball and the capsule live in one window that never moves while one turns into the other. Two stand-ins carry
 * the change: the glass (the capsule's frosted surface) grows out of the ball, and a copy of the brand mark travels
 * from the ball to the capsule's voice button. At either end each stand-in sits exactly on the real thing, so
 * swapping them in and out is not seen.
 */
export interface Box { left: number; top: number; width: number; height: number }

export const MORPH_DURATION = 300;
export const MORPH_EASING = 'cubic-bezier(.32,.72,0,1)';
/** The ball's disc, drawn in the middle of its square. */
export const BALL_DIAMETER = 44;
/** The travelling mark is drawn at this size and scaled. */
export const ORB_SIZE = 44;

/** The ball's disc in the window, from the stage layout's square. */
export function ballDisc(ball: { x: number; y: number; size: number }): Box {
  const inset = (ball.size - BALL_DIAMETER) / 2;
  return { left: ball.x + inset, top: ball.y + inset, width: BALL_DIAMETER, height: BALL_DIAMETER };
}

export function boxOf(rect: DOMRect | Box): Box {
  return { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
}

/** The glass covering `box`; a disc's radius makes it round. */
export function glassFrame(box: Box, radius: number, opacity: number): Keyframe {
  return { left: `${box.left}px`, top: `${box.top}px`, width: `${box.width}px`, height: `${box.height}px`, borderRadius: `${radius}px`, opacity };
}

/** Puts the travelling mark (an ORB_SIZE square at the window's corner) over the disc in `box`. */
export function orbFrame(box: Box): Keyframe {
  const scale = Math.min(box.width, box.height) / ORB_SIZE;
  const x = box.left + box.width / 2 - ORB_SIZE / 2, y = box.top + box.height / 2 - ORB_SIZE / 2;
  return { transform: `translate(${round(x)}px, ${round(y)}px) scale(${round(scale, 4)})` };
}

const round = (value: number, places = 2) => Math.round(value * 10 ** places) / 10 ** places;

/**
 * Hover intent: the pointer has to rest on the ball, not just cross it. It counts as resting while it stays within
 * `slop` pixels of where it settled; moving further starts the wait again.
 */
export class HoverIntent {
  private timer: ReturnType<typeof setTimeout> | undefined;
  private at: { x: number; y: number } | null = null;
  constructor(private fire: () => void, private wait = 280, private slop = 6) {}
  enter(x: number, y: number) { this.at = { x, y }; this.restart(); }
  move(x: number, y: number) {
    if (!this.at) return this.enter(x, y);
    if (Math.hypot(x - this.at.x, y - this.at.y) > this.slop) this.enter(x, y);
  }
  cancel() { clearTimeout(this.timer); this.timer = undefined; this.at = null; }
  get waiting() { return this.timer !== undefined; }
  private restart() {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => { this.timer = undefined; this.at = null; this.fire(); }, this.wait);
  }
}
