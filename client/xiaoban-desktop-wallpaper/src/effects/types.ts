/**
 * A particle effect is data: a few layers, each describing one kind of particle (how many, how big, how it
 * moves, what it looks like). The engine reads it and draws; nothing in a pack is code.
 * The format is documented in docs/effects.md.
 */
export type Range = [number, number];
export type Rgb = [number, number, number];
/** `base` is the colour at the top of a particle (or its bright core), `tip` at the bottom (or its halo). */
export interface ColorPair { base: Rgb; tip: Rgb }

export interface Vein { d: string; color: Rgb; alpha: number; width: number }
export type Shape =
  | { kind: "circle" }
  /** A soft disc of light; `core` is the radius of a small solid centre, as a fraction of the glow's radius. */
  | { kind: "glow"; core: number }
  /** An SVG path drawn in a box from -1 to 1, scaled to the particle's size. */
  | { kind: "path"; d: string; style: "fill" | "stroke" | "both"; lineWidth: number; vein: Vein | null }
  /** A line along the direction of travel with a fading tail, such as a raindrop or a meteor. */
  | { kind: "streak"; length: Range; width: Range; head: number };

/** Particles that appear somewhere, cross the screen in a short time and rest before the next turn (meteors). */
export interface Burst { duration: Range; rest: Range; x: Range; y: Range; travel: Range }

export interface Layer {
  /** Particles at density 1 on a 1920x1080 screen. */
  count: number;
  shape: Shape;
  /** Pixels on a 1080-pixel-high screen; small values are far away and large ones near. */
  size: Range;
  /** Pixels per second along `direction`; far particles are slow, near ones fast. */
  speed: Range;
  /** Degrees: 0 right, 90 down, 180 left. */
  direction: Range;
  /** How strongly the shared gusts push sideways, in pixels per second. */
  wind: number;
  sway: { amplitude: Range; frequency: Range };
  wander: { amplitude: Range; frequency: Range };
  /** Turning speed in radians per second; a negative value turns the other way. */
  spin: Range;
  /** Radians of tilt that follow the sideways sway, like a leaf swinging as it falls. */
  pendulum: number;
  /** 0 keeps every particle upright, 1 starts each at a random angle. */
  scatter: number;
  /** Turning over: the particle narrows to `min` of its width and widens again at `rate` turns per second. */
  flip: { rate: Range; min: number } | null;
  alpha: Range;
  colors: ColorPair[];
  twinkle: { rate: Range; depth: number } | null;
  burst: Burst | null;
}

export interface EffectPack {
  id: string;
  name: string;
  description: string;
  /** Two to four colours for the preview card's sky. */
  sky: string[];
  layers: Layer[];
  source: "builtin" | "user";
}

export interface EffectParams { density: number; speed: number; size: number }
export interface EffectsChoice { selected: string; params: Record<string, EffectParams> }

export const limits = {
  layers: 6, countPerLayer: 1500, totalCount: 3000, pathLength: 6000, idPattern: /^[a-z0-9][a-z0-9_-]{0,47}$/,
  density: [0.2, 2.5] as Range, speed: [0.25, 3] as Range, size: [0.5, 2] as Range,
};
export const defaultParams: EffectParams = { density: 1, speed: 1, size: 1 };
