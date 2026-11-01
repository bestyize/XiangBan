import { resolve, type RobotInput, type Target } from './expressions'
import { cloneFace, type EyeGlyph, type Face, type FaceTree, type Look, type SymbolName } from './params'
import type { Frame } from './render'

/** Deterministic PRNG so tests (and two robots seeded alike) behave reproducibly. */
export function mulberry32(seed: number) {
  let a = seed >>> 0
  return () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

interface Spring { k: number; c: number }
const spring = (k: number, zeta: number): Spring => ({ k, c: 2 * zeta * Math.sqrt(k) })

/** Heavier body parts lag a little behind the snappy screen features. */
const SPRINGS: Record<string, Spring> = {
  eyeL: spring(320, 0.72),
  eyeR: spring(320, 0.72),
  mouth: spring(360, 0.7),
  tilt: spring(110, 0.62),
  lift: spring(140, 0.6),
  earL: spring(170, 0.5),
  earR: spring(170, 0.5),
}
const DEFAULT_SPRING = spring(200, 0.8)
const SUBSTEP = 1 / 240

function stepTree(cur: FaceTree, vel: FaceTree, tgt: FaceTree, s: Spring, dt: number) {
  for (const key of Object.keys(cur)) {
    const value = cur[key]
    if (typeof value === 'number') {
      const v = (vel[key] as number) + (((tgt[key] as number) - value) * s.k - (vel[key] as number) * s.c) * dt
      vel[key] = v
      cur[key] = value + v * dt
    } else {
      stepTree(value, vel[key] as FaceTree, tgt[key] as FaceTree, s, dt)
    }
  }
}

function zeroTree(tree: FaceTree): FaceTree {
  const out: FaceTree = {}
  for (const key of Object.keys(tree)) out[key] = typeof tree[key] === 'number' ? 0 : zeroTree(tree[key] as FaceTree)
  return out
}

const tree = (face: Face) => face as unknown as FaceTree

/** Closing then reopening, p in 0..1 → squash 1 → floor → 1. */
function closeOpen(p: number, floor: number, mid = 0.42) {
  if (p <= 0 || p >= 1) return 1
  if (p < mid) {
    const k = p / mid
    return 1 - (1 - floor) * k * k
  }
  const k = (p - mid) / (1 - mid)
  return floor + (1 - floor) * (1 - (1 - k) * (1 - k))
}

export interface AnimatorOptions {
  seed?: number
  reducedMotion?: boolean
}

/**
 * Turns discrete expression targets into continuous motion: every face parameter follows its target on a spring,
 * glyph changes go through a quick squash, and a "life" layer adds blinks, glances, breathing, ear twitches,
 * hopping and lip movement on top.
 */
export class Animator {
  /** Where the robot looks, -1..1 on each axis relative to itself; null lets it glance around on its own. */
  lookAt: { x: number; y: number } | null = null
  /** Voice amplitude 0..1 (from audio, or 1 for synthetic talking). */
  speaking = 0
  reducedMotion: boolean

  private target: Target
  private cur: Face
  private vel: FaceTree
  private shown: { glyphL: EyeGlyph; glyphR: EyeGlyph; symbol: SymbolName | null; badge: string }
  private symbolP = 1
  private swapL: number | null = null
  private swapR: number | null = null
  private t = 0
  private rand: () => number
  private blinkAt: number
  private blinkStart: number | null = null
  private blinkAgain = false
  private glance = { next: 0, x: 0, y: 0, cx: 0, cy: 0, vx: 0, vy: 0 }
  private twitch = { next: 0, l: 0, r: 0, vl: 0, vr: 0 }
  private dashPhase = 0
  private bouncePhase = 0

  constructor(initial: RobotInput = 'idle', options: AnimatorOptions = {}) {
    this.rand = mulberry32(options.seed ?? Math.floor(Math.random() * 2 ** 31))
    this.reducedMotion = options.reducedMotion ?? false
    this.target = resolve(initial)
    this.cur = cloneFace(this.target.face)
    this.vel = zeroTree(tree(this.cur))
    this.shown = { ...this.pickShown(this.target.look) }
    this.blinkAt = 1 + this.rand() * 3
    this.twitch.next = 4 + this.rand() * 6
  }

  private pickShown(look: Look) {
    return { glyphL: look.glyphL, glyphR: look.glyphR, symbol: look.symbol, badge: look.badge }
  }

  get time() {
    return this.t
  }

  get look(): Look {
    return this.target.look
  }

  setTarget(input: RobotInput | Target, instant = false) {
    this.target = typeof input === 'object' && 'look' in input ? input : resolve(input)
    if (instant) {
      this.cur = cloneFace(this.target.face)
      this.vel = zeroTree(tree(this.cur))
      this.shown = this.pickShown(this.target.look)
      this.symbolP = 1
      this.swapL = this.swapR = null
    }
  }

  /** Kicks one ear (or both) as if something brushed against it. */
  twitchEar(side: -1 | 1 | 0 = 0, strength = 1) {
    if (side <= 0) this.twitch.vl -= 300 * strength
    if (side >= 0) this.twitch.vr -= 300 * strength
  }

  blink() {
    if (this.blinkStart === null) this.blinkStart = this.t
  }

  step(dtIn: number): Frame {
    const dt = Math.min(Math.max(dtIn, 0), 1 / 15)
    this.t += dt
    const t = this.t
    const look = this.target.look
    const calm = this.reducedMotion

    for (let left = dt; left > 1e-6; left -= SUBSTEP) {
      const h = Math.min(SUBSTEP, left)
      const cur = tree(this.cur)
      const tgt = tree(this.target.face)
      for (const key of Object.keys(cur)) {
        const s = SPRINGS[key] ?? DEFAULT_SPRING
        const value = cur[key]
        if (typeof value === 'number') {
          const v = (this.vel[key] as number) + (((tgt[key] as number) - value) * s.k - (this.vel[key] as number) * s.c) * h
          this.vel[key] = v
          cur[key] = value + v * h
        } else {
          stepTree(value, this.vel[key] as FaceTree, tgt[key] as FaceTree, s, h)
        }
      }
      this.stepLife(h, look)
    }

    // Glyph swaps: squash shut, change the drawing, open again.
    const swapDuration = calm ? 0.12 : 0.24
    let squashL = 1
    let squashR = 1
    if (this.shown.glyphL !== look.glyphL && this.swapL === null) this.swapL = t
    if (this.shown.glyphR !== look.glyphR && this.swapR === null) this.swapR = t + 0.03
    if (this.swapL !== null) {
      const p = (t - this.swapL) / swapDuration
      if (p >= 0.42) this.shown.glyphL = look.glyphL
      squashL = closeOpen(p, 0.06)
      if (p >= 1) this.swapL = null
    }
    if (this.swapR !== null) {
      const p = (t - this.swapR) / swapDuration
      if (p >= 0.42) this.shown.glyphR = look.glyphR
      squashR = closeOpen(p, 0.06)
      if (p >= 1) this.swapR = null
    }

    // Symbols shrink away before the next one pops in.
    if (this.shown.symbol !== look.symbol || this.shown.badge !== look.badge) {
      this.symbolP -= dt * 8
      if (this.symbolP <= 0 || this.shown.symbol === null) {
        this.symbolP = 0
        this.shown.symbol = look.symbol
        this.shown.badge = look.badge
      }
    } else if (this.shown.symbol) {
      this.symbolP = Math.min(1, this.symbolP + dt * 3.2)
    }

    // Blinks, sometimes doubled; the right eye trails by a hair.
    let blinkL = 1
    let blinkR = 1
    if (!calm && look.blink && t >= this.blinkAt && this.blinkStart === null) {
      this.blinkStart = t
      this.blinkAgain = this.rand() < 0.2
    }
    if (this.blinkStart !== null) {
      const p = (t - this.blinkStart) / 0.17
      blinkL = closeOpen(p, 0.08)
      blinkR = closeOpen(p - 0.06, 0.08)
      if (p >= 1.1) {
        if (this.blinkAgain) {
          this.blinkAgain = false
          this.blinkStart = t + 0.06
        } else {
          this.blinkStart = null
          this.blinkAt = t + 2.2 + this.rand() * 4.2
        }
      }
    }

    const face = cloneFace(this.cur)
    const breath = calm ? 0 : Math.sin((t / 4.2) * Math.PI * 2)
    face.gazeX += this.glance.cx
    face.gazeY += this.glance.cy
    face.earL += this.twitch.l
    face.earR += this.twitch.r
    face.lift += breath * 0.8

    if (!calm) {
      this.bouncePhase += dt * Math.max(this.cur.bounceHz, 0)
      face.lift -= Math.abs(Math.sin(this.bouncePhase * Math.PI)) * Math.max(this.cur.bounce, 0)
    }
    this.dashPhase += dt * this.cur.spin

    const talk = Math.max(this.cur.talk, this.speaking)
    if (talk > 0.01) {
      const wave = 0.5 + 0.5 * (0.55 * Math.sin(t * 27) + 0.3 * Math.sin(t * 44.6 + 1) + 0.25 * Math.sin(t * 14.5 + 2))
      const amount = talk * Math.min(1, Math.max(0, wave))
      face.mouth.open = Math.max(face.mouth.open * (1 - talk * 0.5), 0) + amount * 8
      face.mouth.tongue = Math.max(face.mouth.tongue, Math.min(1, talk * 1.5))
      face.mouth.cat *= 1 - talk * 0.4
    }

    return {
      face,
      glyphL: this.shown.glyphL,
      glyphR: this.shown.glyphR,
      squashL: squashL * blinkL,
      squashR: squashR * blinkR,
      symbol: this.shown.symbol,
      symbolP: this.symbolP,
      badge: this.shown.badge,
      t,
      dash: this.dashPhase,
      breath,
    }
  }

  private stepLife(h: number, look: Look) {
    const g = this.glance
    const t = this.t
    if (this.lookAt) {
      g.x = Math.max(-1, Math.min(1, this.lookAt.x)) * 11
      g.y = Math.max(-1, Math.min(1, this.lookAt.y)) * 7
    } else if (!look.saccade || this.reducedMotion) {
      g.x = 0
      g.y = 0
    } else if (t >= g.next) {
      if (this.rand() < 0.35) {
        g.x = 0
        g.y = 0
      } else {
        g.x = (this.rand() * 2 - 1) * 7
        g.y = (this.rand() * 2 - 1) * 4
      }
      g.next = t + 0.8 + this.rand() * 2.8
    }
    const gs = spring(520, 0.86)
    g.vx += ((g.x - g.cx) * gs.k - g.vx * gs.c) * h
    g.vy += ((g.y - g.cy) * gs.k - g.vy * gs.c) * h
    g.cx += g.vx * h
    g.cy += g.vy * h

    const w = this.twitch
    if (!this.reducedMotion && t >= w.next) {
      const side = this.rand() < 0.5 ? -1 : 1
      this.twitchEar(side, 0.7 + this.rand() * 0.5)
      w.next = t + 5 + this.rand() * 9
    }
    const ts = spring(420, 0.28)
    w.vl += (-w.l * ts.k - w.vl * ts.c) * h
    w.vr += (-w.r * ts.k - w.vr * ts.c) * h
    w.l += w.vl * h
    w.r += w.vr * h
  }
}
