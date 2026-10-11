import type { Beat } from './mind.ts'

export interface Gaze { x: number; y: number }

export interface ReactionHost {
  /** Queue expression beats on the companion. */
  play(beats: Beat[]): void
  /** Drop what play() queued; only called when nothing else has taken over. */
  stop?(): void
  /** Look toward a point (-1..1 on both axes), or null to hand the eyes back. */
  lookAt?(target: Gaze | null): void
  /** True while the user (click, drag), a care bubble or dozing owns the companion. */
  busy?(): boolean
  random?(): number
  now?(): Date
}

interface Step { at: number; look: Gaze | null }
export interface Reaction {
  id: string
  beats: Beat[]
  /** Gaze changes in seconds from the start; the eyes are released when the beats end. */
  looks?: Step[]
  when?: (hour: number) => boolean
}
export interface EffectProfile {
  /** Seconds between reactions, picked at random in this range. */
  interval: [number, number]
  reactions: Reaction[]
  /** Played once when this effect is replaced or switched off. */
  ended?: Reaction
}

const isNight = (hour: number) => hour >= 19 || hour < 5
const leaf = (id: string, symbol: 'flower' | 'question', side: 1 | -1): Reaction => ({
  id,
  beats: [{ expr: { expression: 'curious', symbol }, hold: 2.2 }],
  looks: [{ at: 0, look: { x: 0.8 * side, y: -0.75 } }, { at: 1.5, look: { x: 0.3 * side, y: -0.2 } }],
})

/** What the companion does for each built-in effect. Custom effect packs have no entry and no reaction. */
export const EFFECT_REACTIONS: Record<string, EffectProfile> = {
  rain: {
    interval: [20, 45],
    reactions: [
      { id: 'rain-huddle', beats: [{ expr: { expression: 'worried', symbol: 'rain' }, hold: 2.4 }, { expr: 'confused', hold: 1.4 }] },
      { id: 'rain-sheepish', beats: [{ expr: { expression: 'embarrassed', symbol: 'rain' }, hold: 2.6 }] },
    ],
    ended: { id: 'rain-shake', beats: [{ expr: 'surprised', hold: 0.35 }, { expr: 'excited', hold: 1 }] },
  },
  snow: {
    interval: [30, 60],
    reactions: [
      { id: 'snow-curious', beats: [{ expr: 'curious', hold: 1.6 }, { expr: 'happy', hold: 1.2 }] },
      { id: 'snow-shiver', beats: [{ expr: 'shy', hold: 2 }] },
    ],
  },
  petals: {
    interval: [30, 60],
    reactions: [leaf('petals-right', 'flower', 1), leaf('petals-left', 'flower', -1)],
  },
  maple: {
    interval: [30, 60],
    reactions: [leaf('maple-right', 'question', 1), leaf('maple-left', 'question', -1)],
  },
  meteors: {
    interval: [25, 55],
    reactions: [
      {
        id: 'meteor-spot',
        beats: [{ expr: { expression: 'curious', symbol: 'sparkle' }, hold: 1 }, { expr: 'excited', hold: 0.9 }],
        looks: [{ at: 0, look: { x: 0.85, y: -0.85 } }, { at: 0.9, look: { x: -0.5, y: -0.5 } }],
      },
      {
        id: 'meteor-spot-left',
        beats: [{ expr: { expression: 'surprised', symbol: 'sparkle' }, hold: 0.7 }, { expr: 'happy', hold: 1.2 }],
        looks: [{ at: 0, look: { x: -0.85, y: -0.85 } }, { at: 0.8, look: { x: 0.4, y: -0.4 } }],
      },
    ],
  },
  stars: {
    interval: [30, 60],
    reactions: [
      {
        id: 'stars-gaze-night', when: isNight,
        beats: [{ expr: { expression: 'curious', symbol: 'moon' }, hold: 2.6 }],
        looks: [{ at: 0, look: { x: 0, y: -0.9 } }],
      },
      { id: 'stars-yawn', when: isNight, beats: [{ expr: 'sleepy', hold: 2.8 }], looks: [{ at: 0, look: { x: 0.2, y: -0.8 } }] },
      {
        id: 'stars-gaze-day', when: hour => !isNight(hour),
        beats: [{ expr: { expression: 'curious', symbol: 'sparkle' }, hold: 2 }],
        looks: [{ at: 0, look: { x: 0, y: -0.9 } }],
      },
      {
        id: 'stars-twinkle-day', when: hour => !isNight(hour),
        beats: [{ expr: { expression: 'happy', symbol: 'sparkle' }, hold: 1.8 }],
        looks: [{ at: 0, look: { x: -0.3, y: -0.8 } }],
      },
    ],
  },
  fireflies: {
    interval: [25, 50],
    reactions: [
      {
        id: 'fireflies-follow',
        beats: [{ expr: { expression: 'curious', symbol: 'sparkle' }, hold: 3.6 }],
        looks: [{ at: 0, look: { x: -0.7, y: -0.2 } }, { at: 1.2, look: { x: 0.6, y: -0.55 } }, { at: 2.4, look: { x: 0.2, y: 0.35 } }],
      },
      {
        id: 'fireflies-follow-back',
        beats: [{ expr: { expression: 'curious', symbol: 'sparkle' }, hold: 3.6 }],
        looks: [{ at: 0, look: { x: 0.7, y: 0.1 } }, { at: 1.2, look: { x: -0.4, y: -0.6 } }, { at: 2.4, look: { x: -0.7, y: 0.3 } }],
      },
    ],
  },
  'butterfly-dream': {
    interval: [30, 60],
    reactions: [
      {
        id: 'butterfly-follow',
        beats: [{ expr: { expression: 'curious', symbol: 'sparkle' }, hold: 1.8 }, { expr: 'happy', hold: 1.2 }],
        looks: [{ at: 0, look: { x: -0.6, y: -0.5 } }, { at: 1, look: { x: 0.2, y: -0.8 } }, { at: 2, look: { x: 0.7, y: -0.3 } }],
      },
      { id: 'butterfly-dream', beats: [{ expr: 'sleepy', hold: 2.2 }, { expr: { expression: 'thinking', symbol: 'question' }, hold: 1.8 }] },
    ],
  },
  lanterns: {
    interval: [30, 60],
    reactions: [
      {
        id: 'lanterns-watch',
        beats: [{ expr: { expression: 'curious', symbol: 'flame' }, hold: 2.4 }],
        looks: [{ at: 0, look: { x: 0.2, y: 0.2 } }, { at: 0.8, look: { x: 0.1, y: -0.5 } }, { at: 1.8, look: { x: 0, y: -0.9 } }],
      },
      { id: 'lanterns-warm', beats: [{ expr: { expression: 'happy', symbol: 'heart' }, hold: 2.2 }], looks: [{ at: 0, look: { x: -0.4, y: -0.7 } }] },
    ],
  },
  fireworks: {
    interval: [20, 45],
    reactions: [
      {
        id: 'fireworks-boom',
        beats: [{ expr: 'surprised', hold: 0.5 }, { expr: { expression: 'excited', symbol: 'sparks' }, hold: 1.4 }],
        looks: [{ at: 0, look: { x: 0.5, y: -0.8 } }],
      },
      {
        id: 'fireworks-cheer',
        beats: [{ expr: { expression: 'laughing', symbol: 'sparkle' }, hold: 1.8 }],
        looks: [{ at: 0, look: { x: -0.5, y: -0.8 } }, { at: 1, look: { x: 0.3, y: -0.7 } }],
      },
    ],
  },
  aurora: {
    interval: [30, 60],
    reactions: [
      {
        id: 'aurora-gaze',
        beats: [{ expr: { expression: 'surprised', symbol: 'sparkle' }, hold: 0.8 }, { expr: 'love', hold: 2 }],
        looks: [{ at: 0, look: { x: 0, y: -0.9 } }],
      },
      {
        id: 'aurora-sweep',
        beats: [{ expr: { expression: 'curious', symbol: 'moon' }, hold: 3 }],
        looks: [{ at: 0, look: { x: -0.8, y: -0.8 } }, { at: 1.5, look: { x: 0.8, y: -0.8 } }],
      },
    ],
  },
  galaxy: {
    interval: [30, 60],
    reactions: [
      {
        id: 'galaxy-trace',
        beats: [{ expr: { expression: 'curious', symbol: 'sparkle' }, hold: 3 }],
        looks: [{ at: 0, look: { x: -0.8, y: 0.2 } }, { at: 1.5, look: { x: 0.8, y: -0.8 } }],
      },
      { id: 'galaxy-yawn', when: isNight, beats: [{ expr: { expression: 'sleepy', symbol: 'moon' }, hold: 2.6 }], looks: [{ at: 0, look: { x: 0.3, y: -0.8 } }] },
      { id: 'galaxy-smile', when: hour => !isNight(hour), beats: [{ expr: { expression: 'happy', symbol: 'sparkle' }, hold: 2 }] },
    ],
  },
}
/** Effects where clicking the companion makes a wish: a falling star, a lantern rising. */
const WISH_EFFECTS = new Set(['meteors', 'lanterns'])

const WISH: Beat[] = [{ expr: { expression: 'happy', symbol: 'sparkle' }, hold: 2 }, { expr: 'proud', hold: 1 }]
const COOLDOWN = 15
const BUSY_RETRY = 4
const WISH_COOLDOWN = 6
const total = (beats: Beat[]) => beats.reduce((sum, beat) => sum + beat.hold, 0)

/**
 * Lets the companion answer the running desktop effect with an expression now and then. It owns no timer:
 * the host calls tick(dt) from its frame loop, so nothing runs while that loop is stopped.
 */
export class Reactions {
  private effect = ''
  private enabled = true
  private paused = false
  private destroyed = false
  private clock = 0
  private nextAt = Infinity
  private lastId = ''
  private lastFireAt = -Infinity
  private lastWishAt = -Infinity
  private activeUntil = 0
  private gazing = false
  private steps: Step[] = []
  private readonly random: () => number
  private readonly now: () => Date

  constructor(private readonly host: ReactionHost) {
    this.random = host.random?.bind(host) ?? Math.random
    this.now = host.now?.bind(host) ?? (() => new Date())
  }

  get running() {
    return this.clock < this.activeUntil
  }

  setEffect(id: string) {
    if (this.destroyed || id === this.effect) return
    const previous = Object.hasOwn(EFFECT_REACTIONS, this.effect) ? EFFECT_REACTIONS[this.effect] : undefined
    this.cancel()
    this.effect = id
    if (previous?.ended && this.enabled && !this.paused && !this.host.busy?.()) this.fire(previous.ended)
    this.schedule(0.5)
  }

  setEnabled(enabled: boolean) {
    if (this.destroyed || enabled === this.enabled) return
    this.enabled = enabled
    if (!enabled) this.cancel()
    this.schedule(0.5)
  }

  setPaused(paused: boolean) {
    if (this.destroyed || paused === this.paused) return
    this.paused = paused
    if (paused) this.cancel()
    this.schedule(0.5)
  }

  /** Clicking the companion while meteors fall or lanterns rise makes a wish. Returns false when the click should be handled as usual. */
  wish(): boolean {
    if (!this.live || !WISH_EFFECTS.has(this.effect) || this.clock - this.lastWishAt < WISH_COOLDOWN) return false
    this.cancel()
    this.lastWishAt = this.clock
    this.lastFireAt = this.clock
    this.host.play(WISH)
    this.activeUntil = this.clock + total(WISH)
    this.schedule(1)
    return true
  }

  tick(dt: number) {
    if (this.destroyed || this.paused) return
    this.clock += dt
    this.flushSteps()
    if (this.gazing && !this.running && !this.steps.length) {
      this.gazing = false
      this.host.lookAt?.(null)
    }
    if (!this.live || this.clock < this.nextAt) return
    if (this.running || this.host.busy?.()) { this.nextAt = this.clock + BUSY_RETRY; return }
    this.fire(this.pick())
    this.nextAt = this.clock + this.interval()
  }

  destroy() {
    if (this.destroyed) return
    this.cancel()
    this.destroyed = true
    this.effect = ''
    this.nextAt = Infinity
  }

  private get live() {
    return !this.destroyed && this.enabled && !this.paused && Object.hasOwn(EFFECT_REACTIONS, this.effect)
  }

  private profile() {
    return EFFECT_REACTIONS[this.effect]
  }

  private interval() {
    const [min, max] = this.profile().interval
    return min + this.random() * (max - min)
  }

  private schedule(scale: number) {
    if (!this.live) { this.nextAt = Infinity; return }
    this.nextAt = Math.max(this.clock + this.interval() * scale, this.lastFireAt + COOLDOWN)
  }

  private pick(): Reaction {
    const hour = this.now().getHours()
    const fits = this.profile().reactions.filter(reaction => !reaction.when || reaction.when(hour))
    const fresh = fits.filter(reaction => reaction.id !== this.lastId)
    const pool = fresh.length ? fresh : fits
    return pool[Math.min(pool.length - 1, Math.floor(this.random() * pool.length))]
  }

  private fire(reaction: Reaction) {
    this.lastId = reaction.id
    this.lastFireAt = this.clock
    this.host.play(reaction.beats)
    this.activeUntil = this.clock + total(reaction.beats)
    this.steps = (reaction.looks ?? []).map(step => ({ at: this.clock + step.at, look: step.look }))
    this.gazing = this.steps.length > 0
    this.flushSteps()
  }

  private flushSteps() {
    while (this.steps.length && this.steps[0].at <= this.clock) this.host.lookAt?.(this.steps.shift()!.look)
  }

  private cancel() {
    const wasRunning = this.running
    this.steps = []
    if (this.gazing) { this.gazing = false; this.host.lookAt?.(null) }
    this.activeUntil = 0
    if (wasRunning && !this.host.busy?.()) this.host.stop?.()
  }
}
