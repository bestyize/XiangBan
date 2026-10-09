import type { Animator } from './animator'
import { toSpec, type RobotInput } from './expressions'

export interface Beat {
  expr: RobotInput
  /** Seconds to hold this expression. */
  hold: number
}

export interface MindOptions {
  /** Doze off when nobody interacts for a while. */
  autoSleep?: boolean
  sleepyAfter?: number
  sleepAfter?: number
  rand?: () => number
}

export interface MindFlags {
  speaking: boolean
  thinking: boolean
  listening: boolean
}

const REACTIONS: Beat[][] = [
  [{ expr: 'surprised', hold: 0.35 }, { expr: 'laughing', hold: 1.1 }],
  [{ expr: 'shy', hold: 1.4 }],
  [{ expr: 'love', hold: 1.3 }],
  [{ expr: 'excited', hold: 1 }],
  [{ expr: 'happy', hold: 0.6 }, { expr: 'proud', hold: 0.9 }],
]
const TOO_MANY_POKES: Beat[] = [{ expr: 'annoyed', hold: 0.9 }, { expr: 'angry', hold: 1.2 }]
const WAKE_UP: Beat[] = [{ expr: 'surprised', hold: 0.45 }, { expr: 'happy', hold: 0.7 }]

const key = (input: RobotInput) => JSON.stringify(toSpec(input))

/**
 * Picks what the robot should be showing right now. Priority: a playing performance, then thinking /
 * listening / speaking, then dozing off, then the base expression the page asked for.
 */
export class Mind {
  private base: RobotInput = 'idle'
  private flags: MindFlags = { speaking: false, thinking: false, listening: false }
  private queue: Beat[] = []
  private beatLeft = 0
  private idle = 0
  private pokes: number[] = []
  private clock = 0
  private shownKey = ''
  private readonly options: Required<MindOptions>
  private readonly animator: Animator

  constructor(animator: Animator, options: MindOptions = {}) {
    this.animator = animator
    this.options = { autoSleep: false, sleepyAfter: 40, sleepAfter: 90, rand: Math.random, ...options }
  }

  setBase(input: RobotInput) {
    this.base = input
  }

  setFlags(flags: Partial<MindFlags>) {
    this.flags = { ...this.flags, ...flags }
    if (flags.speaking || flags.thinking || flags.listening) this.activity()
  }

  get dozing() {
    return this.options.autoSleep && this.idle >= this.options.sleepyAfter
  }

  /** Something happened near the robot: reset the doze timer and wake it if needed. */
  activity() {
    const wasDozing = this.dozing
    this.idle = 0
    if (wasDozing && this.queue.length === 0) this.play(WAKE_UP)
  }

  play(beats: Beat[]) {
    this.queue = [...beats]
    this.beatLeft = this.queue[0]?.hold ?? 0
  }

  poke() {
    const wasDozing = this.dozing
    this.idle = 0
    this.pokes = [...this.pokes.filter((at) => this.clock - at < 3), this.clock]
    this.animator.twitchEar(0, 0.8)
    if (wasDozing) return this.play(WAKE_UP)
    if (this.pokes.length >= 5) {
      this.pokes = []
      return this.play(TOO_MANY_POKES)
    }
    const pick = REACTIONS[Math.floor(this.options.rand() * REACTIONS.length)] ?? REACTIONS[0]
    this.play(pick)
  }

  /** The expression that should be on screen now. */
  current(): RobotInput {
    if (this.queue.length) return this.queue[0].expr
    if (this.flags.thinking) return 'thinking'
    if (this.flags.listening) return 'listening'
    if (this.flags.speaking) {
      const spec = toSpec(this.base)
      return spec.expression === 'idle' ? { ...spec, expression: 'speaking' } : this.base
    }
    if (this.options.autoSleep) {
      if (this.idle >= this.options.sleepAfter) return 'sleeping'
      if (this.idle >= this.options.sleepyAfter) return 'sleepy'
    }
    return this.base
  }

  tick(dt: number) {
    this.clock += dt
    this.idle += dt
    if (this.queue.length) {
      this.beatLeft -= dt
      if (this.beatLeft <= 0) {
        this.queue.shift()
        this.beatLeft = this.queue[0]?.hold ?? 0
      }
    }
    const next = this.current()
    const nextKey = key(next)
    if (nextKey !== this.shownKey) {
      this.shownKey = nextKey
      this.animator.setTarget(next)
    }
  }
}
