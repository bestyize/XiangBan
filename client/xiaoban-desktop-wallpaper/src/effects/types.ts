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
/** A soft light behind a path shape, in the shape's own -1 to 1 box: a lantern's glow. */
export interface Halo { scale: number; y: number; color: Rgb; alpha: number }
export type Shape =
  | { kind: "circle" }
  /** A soft disc of light; `core` is the radius of a small solid centre, as a fraction of the glow's radius. */
  | { kind: "glow"; core: number; aspect: number }
  /** An SVG path drawn in a box from -1 to 1, scaled to the particle's size. */
  | { kind: "path"; d: string; style: "fill" | "stroke" | "both"; lineWidth: number; vein: Vein | null; halo: Halo | null }
  /** An outline of a circle, flattened to `aspect` of its width (a ripple seen from the side). */
  | { kind: "ring"; width: Range; aspect: number }
  /** A tall soft ray of light, brightest near its lower end; `size` is its width (an aurora's curtain). */
  | { kind: "beam"; length: Range; focus: number }
  /** A line along the direction of travel with a fading tail, such as a raindrop or a meteor. */
  | { kind: "streak"; length: Range; width: Range; head: number };

/**
 * Particles that appear somewhere, cross the screen in a short time and rest before the next turn (meteors).
 * With `group` above 1 that many particles share one start, one turn and one colour: a firework's sparks, or the
 * rings of one ripple. `spread` fans their directions out, `stagger` starts them one after another, and `launch`
 * first sends the group's first particle up from the bottom edge to where the others then burst out.
 */
export interface Burst {
  duration: Range; rest: Range; x: Range; y: Range; travel: Range;
  group: number; spread: number; stagger: number; launch: Range | null;
  /** Pixels per second squared, pulling downwards. */
  gravity: number;
  /** `out` slows down towards the end instead of moving evenly. */
  ease: "linear" | "out";
  /** `in-out` fades in and out; `out` appears at once and fades away. */
  fade: "in-out" | "out";
  /** Size from the start of a turn to its end, as multiples. */
  grow: Range;
  /** 0 puts a group's particles on a ring; 1 spreads them like a sphere of sparks seen from the front. */
  fill: number;
  /** Squeezes the group's pattern to this share of its height, at a random tilt each turn (a ring seen at an angle). */
  flatten: Range;
  /** Seconds of a bright flash where the group bursts. */
  flash: number;
  /** Share of a group's particles that take the next colour of the palette (two-colour shells). */
  mix: number;
}

/** Particles kept along a band across the screen instead of all over it (the Milky Way, an aurora). */
export interface Band {
  /** Degrees the band is turned from horizontal. */
  angle: number;
  /** Where its middle line sits, in screen heights from the centre (negative is up). */
  offset: number;
  /** How thick it is, in screen heights. */
  width: number;
  /** `gauss` gathers particles towards the middle line, `even` spreads them evenly. */
  falloff: "gauss" | "even";
  /** The middle line waves: amplitude in screen heights, waves along the band, waves per second. */
  wave: { amplitude: number; count: number; speed: number };
}

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
  band: Band | null;
  /** `add` makes overlapping light brighter, for glows, fireworks and auroras. */
  blend: "normal" | "add";
  /** `motion` turns each particle to face the way it is moving (a fish); `fixed` uses spin and scatter. */
  orient: "fixed" | "motion";
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
