import type { ExpressionName } from '../../companion/expressions.ts';
import type { PetMotion } from './actions.ts';

export interface MingyueRenderer {
  render(seconds: number, expression: ExpressionName, preference?: PetMotion): void;
  hit(x: number, y: number): boolean;
  poke(): void;
  setDragging(value: boolean): void;
  destroy(): void;
}
export function loadMingyue(canvas: HTMLCanvasElement, signal: AbortSignal): Promise<MingyueRenderer>;
