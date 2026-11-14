import type { AquariumMode, AquariumSettings } from './settings.ts';

export interface ArowanaRenderer {
  readonly mode: AquariumMode | null;
  readonly fishCount: number;
  setMode(mode: AquariumMode): Promise<void>;
  setLighting(lighting: AquariumSettings['lighting']): void;
  resize(width: number, height: number, dpr: number, size: number): void;
  render(time: number): void;
  destroy(): void;
}

export function createArowanaRenderer(canvas: HTMLCanvasElement): ArowanaRenderer;
