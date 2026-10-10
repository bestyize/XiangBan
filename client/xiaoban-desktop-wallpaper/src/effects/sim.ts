import { defaultParams, type EffectPack, type EffectParams, type Layer, type Range } from "./types.ts";

const TAU = Math.PI * 2;
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const at = (range: Range, t: number) => lerp(range[0], range[1], t);
const clamp01 = (value: number) => Math.max(0, Math.min(1, value));
const wrap = (value: number) => value - Math.floor(value);
/** Which of a particle's fixed random numbers decides which trait, so traits do not move in lockstep. */
const K = { dir: 0, swayF: 1, swayA: 2, swayP: 3, wanA: 4, wanF: 5, wanP: 6, spin: 7, scatter: 8, size: 9, twR: 10, twP: 11, flipR: 12, flipP: 13, count: 14 };

export interface Particle {
  /** Position inside the padded screen, 0 to 1 on each axis (particles that loop around the screen). */
  u: number; v: number;
  /** Depth, 0 far to 1 near: it sets the size, speed and brightness. */
  d: number;
  k: number[];
  color: number;
  /** Seconds since it appeared (for a burst, since its turn began). */
  age: number;
  life: number; rest: number; sx: number; sy: number; heading: number; travel: number;
  /** What to draw this frame. */
  visible: boolean; x: number; y: number; angle: number; scaleX: number; alpha: number; size: number; dx: number; dy: number;
}
export interface LayerState { spec: Layer; pool: Particle[]; active: number; order: Particle[]; pad: number }

/** Pure simulation of one effect pack. The same code drives the desktop layer and the preview cards. */
export class ParticleSystem {
  readonly layers: LayerState[];
  width = 0; height = 0; unit = 1;
  params: EffectParams = { ...defaultParams };
  private clock = 0;
  constructor(readonly pack: EffectPack, private random: () => number = Math.random) {
    this.layers = pack.layers.map((spec) => ({ spec, pool: [], active: 0, order: [], pad: 0 }));
  }
  /** Slow gusts shared by every layer: -1 (leftwards) to 1 (rightwards). */
  wind() { return Math.sin(this.clock * 0.35) * 0.6 + Math.sin(this.clock * 0.91 + 1.7) * 0.4; }
  counts() { return this.layers.map((state) => state.active); }

  configure(width: number, height: number, params: EffectParams) {
    this.width = width; this.height = height; this.params = params;
    this.unit = Math.max(0.75, Math.min(1.8, height / 1080));
    const area = Math.max(0.25, Math.min(1.6, width * height / (1920 * 1080)));
    for (const state of this.layers) {
      const { spec } = state;
      const want = spec.count === 0 ? 0 : Math.max(1, Math.round(spec.count * params.density * area));
      while (state.pool.length < want) state.pool.push(this.spawn(spec));
      if (want !== state.active) {
        state.active = want;
        // Far particles are drawn first, so near ones cover them.
        state.order = state.pool.slice(0, want).sort((a, b) => a.d - b.d);
      }
      const reach = spec.shape.kind === "streak" ? spec.shape.length[1] + 24 : spec.size[1] * 0.75 + 8;
      state.pad = reach * this.unit * params.size;
    }
    this.advance(0);
  }

  advance(seconds: number) {
    if (this.width <= 0 || this.height <= 0) return;
    const dt = Math.max(0, Math.min(seconds, 0.1)) * this.params.speed;
    this.clock += dt;
    const wind = this.wind();
    for (const state of this.layers) for (let i = 0; i < state.active; i++) this.step(state, state.pool[i], dt, wind);
  }

  private spawn(spec: Layer): Particle {
    const r = this.random;
    const p: Particle = { u: r(), v: r(), d: Math.pow(r(), 1.4), k: Array.from({ length: K.count }, () => r()), color: Math.floor(r() * spec.colors.length),
      age: 0, life: 1, rest: 0, sx: 0, sy: 0, heading: 0, travel: 0,
      visible: true, x: 0, y: 0, angle: 0, scaleX: 1, alpha: 1, size: 1, dx: 0, dy: 1 };
    if (spec.burst) { this.roll(spec, p); p.age = r() * (p.life + p.rest); }
    return p;
  }
  private roll(spec: Layer, p: Particle) {
    const burst = spec.burst!, r = this.random;
    p.life = at(burst.duration, r()); p.rest = at(burst.rest, r());
    p.sx = at(burst.x, r()); p.sy = at(burst.y, r());
    p.heading = at(spec.direction, r()) * Math.PI / 180; p.travel = at(burst.travel, r());
  }

  private step(state: LayerState, p: Particle, dt: number, wind: number) {
    const { spec, pad } = state, { unit, params } = this;
    p.age += dt;
    p.size = at(spec.size, clamp01(p.d * 0.75 + p.k[K.size] * 0.25)) * unit * params.size;
    p.alpha = at(spec.alpha, p.d);
    if (spec.twinkle) {
      const pulse = 0.5 + 0.5 * Math.sin(TAU * (at(spec.twinkle.rate, p.k[K.twR]) * p.age + p.k[K.twP]));
      p.alpha *= 1 - spec.twinkle.depth + spec.twinkle.depth * pulse * pulse;
    }
    p.scaleX = spec.flip ? lerp(spec.flip.min, 1, Math.abs(Math.cos(TAU * (at(spec.flip.rate, p.k[K.flipR]) * p.age + p.k[K.flipP])))) : 1;
    if (spec.burst) {
      if (p.age >= p.life + p.rest) { p.age -= p.life + p.rest; this.roll(spec, p); }
      p.visible = p.age < p.life;
      if (!p.visible) return;
      const progress = p.age / p.life, travelled = p.travel * unit * progress;
      p.dx = Math.cos(p.heading); p.dy = Math.sin(p.heading);
      p.x = p.sx * this.width + p.dx * travelled; p.y = p.sy * this.height + p.dy * travelled;
      p.alpha *= Math.pow(Math.sin(Math.PI * progress), 0.6);
      p.angle = p.heading;
      return;
    }
    const W = this.width + 2 * pad, H = this.height + 2 * pad;
    const heading = at(spec.direction, p.k[K.dir]) * Math.PI / 180, speed = at(spec.speed, p.d) * unit;
    const swayPhase = TAU * (at(spec.sway.frequency, p.k[K.swayF]) * p.age + p.k[K.swayP]);
    const wander = at(spec.wander.amplitude, p.k[K.wanA]) * unit, wanderPhase = TAU * (at(spec.wander.frequency, p.k[K.wanF]) * p.age + p.k[K.wanP]);
    const vx = Math.cos(heading) * speed + at(spec.sway.amplitude, p.k[K.swayA]) * unit * Math.sin(swayPhase)
      + wind * spec.wind * (0.5 + 0.5 * p.d) * unit + wander * Math.cos(wanderPhase);
    const vy = Math.sin(heading) * speed + wander * 0.8 * Math.sin(wanderPhase * 0.73);
    p.u = wrap(p.u + vx * dt / W); p.v = wrap(p.v + vy * dt / H);
    p.x = -pad + p.u * W; p.y = -pad + p.v * H;
    const length = Math.hypot(vx, vy) || 1;
    p.dx = vx / length; p.dy = vy / length;
    p.angle = (p.k[K.scatter] * 2 - 1) * Math.PI * spec.scatter + at(spec.spin, p.k[K.spin]) * p.age + spec.pendulum * Math.sin(swayPhase);
    p.visible = true;
  }
}
