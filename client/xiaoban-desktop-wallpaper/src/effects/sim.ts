import { defaultParams, type Burst, type EffectPack, type EffectParams, type Layer, type Range } from "./types.ts";

const TAU = Math.PI * 2;
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const at = (range: Range, t: number) => lerp(range[0], range[1], t);
const clamp01 = (value: number) => Math.max(0, Math.min(1, value));
const wrap = (value: number) => value - Math.floor(value);
/** Which of a particle's fixed random numbers decides which trait, so traits do not move in lockstep. */
const K = { dir: 0, swayF: 1, swayA: 2, swayP: 3, wanA: 4, wanF: 5, wanP: 6, spin: 7, scatter: 8, size: 9, twR: 10, twP: 11, flipR: 12, flipP: 13, lift: 14, mix: 15, count: 16 };

export interface Particle {
  /** Position inside the padded screen, 0 to 1 on each axis (particles that loop around the screen). */
  u: number; v: number;
  /** Depth, 0 far to 1 near: it sets the size, speed and brightness. */
  d: number;
  k: number[];
  color: number;
  /** Seconds since it appeared (for a burst, since its turn began). */
  age: number;
  life: number; rest: number; sx: number; sy: number; heading: number; travel: number; launch: number; tilt: number; flat: number;
  /** A burst's flash: brightness 0 to 1, where and how big. Only a group's first particle carries it. */
  flash: number; fx: number; fy: number; fr: number;
  /** Place across a band, -1 to 1, and the sway that has moved it off that place, in pixels. */
  w: number; ox: number; oy: number;
  /** What to draw this frame. */
  visible: boolean; x: number; y: number; angle: number; scaleX: number; alpha: number; size: number; dx: number; dy: number;
  /** Share of its full length a streak shows, so a spark that slows down leaves a shorter tail. */
  stretch: number;
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
      const group = spec.burst?.group ?? 1;
      // A group is never split, so a firework always has all its sparks.
      const want = spec.count === 0 ? 0 : Math.max(1, Math.round(spec.count * params.density * area / group)) * group;
      while (state.pool.length < want) state.pool.push(this.spawn(spec));
      if (want !== state.active) {
        state.active = want;
        // Far particles are drawn first, so near ones cover them.
        state.order = state.pool.slice(0, want).sort((a, b) => a.d - b.d);
      }
      const { shape } = spec;
      const reach = shape.kind === "streak" ? shape.length[1] + 24 : shape.kind === "beam" ? shape.length[1] * 0.6 + 8
        : spec.size[1] * 0.75 * (shape.kind === "glow" ? Math.max(1, shape.aspect) : 1) + 8;
      state.pad = reach * this.unit * params.size;
    }
    this.advance(0);
  }

  advance(seconds: number) {
    if (this.width <= 0 || this.height <= 0) return;
    const dt = Math.max(0, Math.min(seconds, 0.1)) * this.params.speed;
    this.clock += dt;
    const wind = this.wind();
    for (const state of this.layers) {
      const group = state.spec.burst?.group;
      if (group) for (let i = 0; i < state.active; i += group) this.burst(state, i, Math.min(group, state.active - i), dt);
      else for (let i = 0; i < state.active; i++) this.step(state, state.pool[i], dt, wind);
    }
  }

  private spawn(spec: Layer): Particle {
    const r = this.random;
    const p: Particle = { u: r(), v: r(), d: Math.pow(r(), 1.4), k: Array.from({ length: K.count }, () => r()), color: Math.floor(r() * spec.colors.length),
      age: 0, life: 1, rest: 0, sx: 0, sy: 0, heading: 0, travel: 0, launch: 0, tilt: 0, flat: 1, flash: 0, fx: 0, fy: 0, fr: 0, w: 0, ox: 0, oy: 0,
      visible: true, x: 0, y: 0, angle: 0, scaleX: 1, alpha: 1, size: 1, dx: 0, dy: 1, stretch: 1 };
    if (spec.burst) { this.roll(spec, p); p.age = r() * cycle(spec.burst, p); }
    if (spec.band) p.w = spec.band.falloff === "gauss" ? (r() + r() + r()) / 1.5 - 1 : r() * 2 - 1;
    return p;
  }
  private roll(spec: Layer, p: Particle) {
    const burst = spec.burst!, r = this.random;
    p.life = at(burst.duration, r()); p.rest = at(burst.rest, r());
    p.sx = at(burst.x, r()); p.sy = at(burst.y, r());
    p.heading = at(spec.direction, r()) * Math.PI / 180; p.travel = at(burst.travel, r());
    p.launch = burst.launch ? at(burst.launch, r()) : 0;
    p.tilt = r() * Math.PI; p.flat = at(burst.flatten, r());
    if (burst.group > 1) p.color = Math.floor(r() * spec.colors.length);
  }

  /** Size, brightness and turning over, which every kind of particle has. */
  private look(spec: Layer, p: Particle) {
    p.size = at(spec.size, clamp01(p.d * 0.75 + p.k[K.size] * 0.25)) * this.unit * this.params.size;
    p.alpha = at(spec.alpha, p.d);
    if (spec.twinkle) {
      const pulse = 0.5 + 0.5 * Math.sin(TAU * (at(spec.twinkle.rate, p.k[K.twR]) * p.age + p.k[K.twP]));
      p.alpha *= 1 - spec.twinkle.depth + spec.twinkle.depth * pulse * pulse;
    }
    p.scaleX = spec.flip ? lerp(spec.flip.min, 1, Math.abs(Math.cos(TAU * (at(spec.flip.rate, p.k[K.flipR]) * p.age + p.k[K.flipP])))) : 1;
  }

  /** One group of a burst layer: the first particle keeps the turn's timing and place for all of them. */
  private burst(state: LayerState, first: number, n: number, dt: number) {
    const { spec } = state, b = spec.burst!, lead = state.pool[first], { unit } = this;
    for (let j = 0; j < n; j++) state.pool[first + j].age += dt;
    const turn = cycle(b, lead);
    if (lead.age >= turn) {
      lead.age -= turn; this.roll(spec, lead);
      // Each turn draws its sparks afresh, so no two bursts of a group look the same.
      if (n > 1) for (let j = 0; j < n; j++) { const k = state.pool[first + j].k; for (let i = 0; i < K.count; i++) k[i] = this.random(); }
    }
    const fall = b.gravity * unit, cosT = Math.cos(lead.tilt), sinT = Math.sin(lead.tilt), pace = b.ease === "out" ? 3 : 1;
    const ox = lead.sx * this.width, oy = lead.sy * this.height, reach0 = lead.travel * unit;
    const since = lead.age - lead.launch;
    lead.flash = b.flash > 0 && since >= 0 && since < b.flash ? (1 - since / b.flash) ** 2 : 0;
    lead.fx = ox; lead.fy = oy; lead.fr = Math.max(24 * unit, reach0 * 0.5);
    for (let j = 0; j < n; j++) {
      const p = state.pool[first + j];
      this.look(spec, p);
      p.stretch = 1;
      if (n > 1) p.color = b.mix > 0 && p.k[K.mix] < b.mix ? (lead.color + 1) % spec.colors.length : lead.color;
      if (j === 0 && lead.age < lead.launch) {
        // The rocket: up from below the screen to where the group bursts, slowing as it climbs.
        const climb = lead.age / lead.launch;
        p.visible = true; p.x = ox; p.y = lerp(this.height * 1.04, oy, 1 - (1 - climb) ** 2);
        p.dx = 0; p.dy = -1; p.angle = -Math.PI / 2; p.alpha *= 0.9 * Math.min(1, climb / 0.15); p.stretch = lerp(1, 0.4, climb);
        continue;
      }
      const t = since - j * b.stagger;
      p.visible = t >= 0 && t < lead.life;
      if (!p.visible) continue;
      const progress = t / lead.life;
      const eased = b.ease === "out" ? 1 - (1 - progress) ** 3 : progress, rate = b.ease === "out" ? 3 * (1 - progress) ** 2 : 1;
      const heading = n > 1 && b.spread > 0 ? lead.heading + b.spread * Math.PI / 180 * (j + (p.k[K.dir] - 0.5) * 0.5) / n : lead.heading;
      // A shell is a sphere of sparks seen from the front: the rim stays crowded and the middle fills in.
      const z = p.k[K.lift] * 2 - 1, jitter = p.k[K.swayA];
      const radial = n > 1 ? lerp(lerp(0.7, 1, jitter), Math.sqrt(1 - z * z) * lerp(0.9, 1, jitter), b.fill) : 1;
      let ux = Math.cos(heading) * radial, uy = Math.sin(heading) * radial;
      if (n > 1 && lead.flat < 1) { uy *= lead.flat; [ux, uy] = [ux * cosT - uy * sinT, ux * sinT + uy * cosT]; }
      p.x = ox + ux * reach0 * eased; p.y = oy + uy * reach0 * eased + 0.5 * fall * t * t;
      const vx = ux * reach0 * rate / lead.life, vy = uy * reach0 * rate / lead.life + fall * t, speed = Math.hypot(vx, vy);
      p.dx = speed > 1e-6 ? vx / speed : Math.cos(heading); p.dy = speed > 1e-6 ? vy / speed : Math.sin(heading);
      p.stretch = Math.max(0.15, Math.min(1, speed / (reach0 * pace / lead.life || 1)));
      p.alpha *= b.fade === "out" ? Math.min(1, progress / 0.04) * (1 - progress * progress) : Math.pow(Math.sin(Math.PI * progress), 0.6);
      p.size *= lerp(b.grow[0], b.grow[1], eased);
      p.angle = spec.orient === "motion" ? Math.atan2(p.dy, p.dx) : heading;
    }
  }

  private step(state: LayerState, p: Particle, dt: number, wind: number) {
    const { spec, pad } = state, { unit } = this;
    p.age += dt;
    this.look(spec, p);
    const heading = at(spec.direction, p.k[K.dir]) * Math.PI / 180, speed = at(spec.speed, p.d) * unit;
    const swayPhase = TAU * (at(spec.sway.frequency, p.k[K.swayF]) * p.age + p.k[K.swayP]);
    const wander = at(spec.wander.amplitude, p.k[K.wanA]) * unit, wanderPhase = TAU * (at(spec.wander.frequency, p.k[K.wanF]) * p.age + p.k[K.wanP]);
    const bx = Math.cos(heading) * speed + wind * spec.wind * (0.5 + 0.5 * p.d) * unit, by = Math.sin(heading) * speed;
    const sx = at(spec.sway.amplitude, p.k[K.swayA]) * unit * Math.sin(swayPhase) + wander * Math.cos(wanderPhase);
    const sy = wander * 0.8 * Math.sin(wanderPhase * 0.73);
    const vx = bx + sx, vy = by + sy;
    let tilt = 0;
    if (spec.band) {
      // Along the band the particle travels and loops; across it, it keeps its place and only sways around it.
      const band = spec.band, turn = band.angle * Math.PI / 180, ca = Math.cos(turn), sa = Math.sin(turn);
      const length = this.width * Math.abs(ca) + this.height * Math.abs(sa) + 2 * pad;
      p.u = wrap(p.u + (bx * ca + by * sa) * dt / length);
      p.ox += sx * dt; p.oy += sy * dt;
      const across = p.w * band.width * this.height / 2 + band.wave.amplitude * this.height * Math.sin(TAU * (band.wave.count * p.u + band.wave.speed * this.clock));
      const along = (p.u - 0.5) * length, mx = this.width / 2 - sa * band.offset * this.height, my = this.height / 2 + ca * band.offset * this.height;
      p.x = mx + ca * along - sa * across + p.ox; p.y = my + sa * along + ca * across + p.oy;
      tilt = turn;
    } else {
      const W = this.width + 2 * pad, H = this.height + 2 * pad;
      p.u = wrap(p.u + vx * dt / W); p.v = wrap(p.v + vy * dt / H);
      p.x = -pad + p.u * W; p.y = -pad + p.v * H;
    }
    const length = Math.hypot(vx, vy) || 1;
    p.dx = vx / length; p.dy = vy / length;
    p.angle = spec.orient === "motion" ? Math.atan2(p.dy, p.dx) + spec.pendulum * Math.sin(swayPhase)
      : tilt + (p.k[K.scatter] * 2 - 1) * Math.PI * spec.scatter + at(spec.spin, p.k[K.spin]) * p.age + spec.pendulum * Math.sin(swayPhase);
    p.visible = true;
  }
}

/** Seconds from one turn of a burst to the next: the climb, each particle's start, the burst itself, the rest. */
const cycle = (burst: Burst, p: Particle) => p.launch + burst.stagger * (burst.group - 1) + p.life + p.rest;
