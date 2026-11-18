import type { AquariumMode, AquariumSettings } from './settings.ts';
import type { AquariumId } from '../types.ts';

export interface AquariumRenderer {
  readonly mode: AquariumMode | null;
  readonly fishCount: number;
  setMode(mode: AquariumMode): Promise<void>;
  setAquascape(enabled: boolean): void;
  setLighting(lighting: AquariumSettings['lighting']): void;
  resize(width: number, height: number, dpr: number, size: number): void;
  render(time: number): void;
  destroy(): void;
}

export function createAquariumRenderer(canvas: HTMLCanvasElement, id: AquariumId): AquariumRenderer;
