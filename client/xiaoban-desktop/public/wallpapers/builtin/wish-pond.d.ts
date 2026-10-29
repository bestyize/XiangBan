interface Wish { x: number; y: number }
interface Context { canvas: HTMLCanvasElement; dpr?: number; settings?: Record<string, unknown> }
export interface PondRenderer {
  update(settings: Record<string, unknown>): void;
  resize(size: { dpr: number }): void;
  setWishes(wishes: Wish[]): void;
  ripple(x: number, y: number, time: number): void;
  render(frame: { time: number }): void;
  destroy(): void;
}
export default function create(context: Context): PondRenderer;
