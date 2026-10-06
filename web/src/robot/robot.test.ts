import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { Animator } from './animator.ts'
import { EXPRESSION_NAMES, resolve, stillFrame } from './expressions.ts'
import { Mind } from './mind.ts'
import { dexArt, moodFace } from './moods.ts'
import { robotMarkup, type Frame } from './render.ts'
import { SYMBOL_NAMES } from './symbols.ts'

const run = (animator: Animator, seconds: number, onFrame?: (frame: Frame) => void) => {
  let frame = animator.step(0)
  for (let t = 0; t < seconds; t += 1 / 60) {
    frame = animator.step(1 / 60)
    onFrame?.(frame)
  }
  return frame
}

const wellFormed = (svg: string) => {
  expect(svg).not.toMatch(/NaN|undefined|Infinity/)
  expect(svg.startsWith('<svg')).toBe(true)
  expect(svg.endsWith('</svg>')).toBe(true)
}

describe('robot expressions', () => {
  it('renders every expression as clean markup', () => {
    expect(EXPRESSION_NAMES).toHaveLength(25)
    for (const name of EXPRESSION_NAMES) wellFormed(robotMarkup(stillFrame(name), 'x'))
  })

  it('renders every screen symbol', () => {
    for (const symbol of SYMBOL_NAMES) wellFormed(robotMarkup(stillFrame({ expression: 'idle', symbol, badge: symbol === 'badge' ? '100' : undefined }), 'x'))
  })

  it('has art for every unlockable expression in the server catalog', () => {
    const catalog = readFileSync(new URL('../../../server/src/main/kotlin/xyz/xiaban/server/unlocks/UnlockCatalog.kt', import.meta.url), 'utf8')
    const ids = [...catalog.matchAll(/UnlockDef\("([a-z0-9_]+)"/g)].map((m) => m[1])
    expect(ids.length).toBeGreaterThan(20)
    expect(Object.keys(dexArt).sort()).toEqual(ids.sort())
    for (const art of [...Object.values(dexArt), ...Object.values(moodFace).map((spec) => ({ spec }))]) wellFormed(robotMarkup(stillFrame(art.spec), 'x'))
  })

  it('applies per-use overrides on top of the preset', () => {
    const target = resolve({ expression: 'happy', face: { eyeL: { w: 30 } }, badge: '7' })
    expect(target.face.eyeL.w).toBe(30)
    expect(target.face.eyeR.w).toBe(22)
    expect(target.look.symbol).toBe('badge')
    expect(target.look.badge).toBe('7')
  })
})

describe('animator', () => {
  it('springs into a new expression and settles there', () => {
    const animator = new Animator('idle', { seed: 1, reducedMotion: true })
    animator.setTarget('surprised')
    const early = animator.step(1 / 60)
    expect(early.face.eyeL.w).toBeLessThan(25)
    const frame = run(animator, 2)
    expect(frame.face.eyeL.w).toBeCloseTo(26, 1)
    expect(frame.face.mouth.open).toBeCloseTo(4, 1)
    expect(frame.symbol).toBe('exclaim')
    expect(frame.symbolP).toBe(1)
  })

  it('accepts specs with face overrides as targets', () => {
    const animator = new Animator('idle', { seed: 1, reducedMotion: true })
    animator.setTarget({ expression: 'idle', face: { mouth: { open: 9 } } })
    expect(run(animator, 2).face.mouth.open).toBeCloseTo(9, 1)
  })

  it('squashes the eyes shut while swapping to a different glyph', () => {
    const animator = new Animator('idle', { seed: 1, reducedMotion: true })
    animator.setTarget('love')
    let minSquash = 1
    const frame = run(animator, 1, (f) => { minSquash = Math.min(minSquash, f.squashL) })
    expect(minSquash).toBeLessThan(0.2)
    expect(frame.glyphL).toBe('heart')
    expect(frame.squashL).toBe(1)
  })

  it('blinks on its own every few seconds, but not with reduced motion', () => {
    let blinks = 0
    let closed = false
    run(new Animator('idle', { seed: 7 }), 20, (f) => {
      if (f.squashL < 0.3 && !closed) blinks++
      closed = f.squashL < 0.3
    })
    expect(blinks).toBeGreaterThanOrEqual(3)
    let calmBlinks = 0
    run(new Animator('idle', { seed: 7, reducedMotion: true }), 20, (f) => { if (f.squashL < 0.3) calmBlinks++ })
    expect(calmBlinks).toBe(0)
  })

  it('moves its mouth while speaking', () => {
    const animator = new Animator('idle', { seed: 3, reducedMotion: true })
    animator.speaking = 1
    const opens: number[] = []
    run(animator, 1, (f) => opens.push(f.face.mouth.open))
    expect(Math.max(...opens) - Math.min(...opens)).toBeGreaterThan(3)
  })

  it('never produces NaN over a long, busy run', () => {
    const animator = new Animator('idle', { seed: 11 })
    animator.speaking = 0.6
    let i = 0
    run(animator, 30, (f) => {
      if (++i % 90 === 0) animator.setTarget(EXPRESSION_NAMES[(i / 90) % EXPRESSION_NAMES.length])
      if (i % 200 === 0) wellFormed(robotMarkup(f, 'x'))
    })
  })
})

describe('mind', () => {
  const setup = () => {
    const animator = new Animator('idle', { seed: 5, reducedMotion: true })
    const mind = new Mind(animator, { autoSleep: true, sleepyAfter: 10, sleepAfter: 20, rand: () => 0 })
    return { animator, mind }
  }

  it('dozes off when left alone and wakes up surprised', () => {
    const { mind } = setup()
    mind.tick(11)
    expect(mind.current()).toBe('sleepy')
    mind.tick(10)
    expect(mind.current()).toBe('sleeping')
    mind.activity()
    expect(mind.current()).toBe('surprised')
    mind.tick(0.5)
    mind.tick(0.8)
    expect(mind.current()).toBe('idle')
  })

  it('reacts to pokes and gets annoyed when poked too often', () => {
    const { mind } = setup()
    mind.poke()
    expect(mind.current()).toBe('surprised')
    for (let i = 0; i < 4; i++) {
      mind.tick(0.1)
      mind.poke()
    }
    expect(mind.current()).toBe('annoyed')
  })

  it('shows thinking over speaking over the base expression', () => {
    const { mind } = setup()
    mind.setBase('worried')
    mind.setFlags({ speaking: true })
    expect(mind.current()).toBe('worried')
    mind.setBase('idle')
    expect(mind.current()).toEqual({ expression: 'speaking' })
    mind.setFlags({ thinking: true })
    expect(mind.current()).toBe('thinking')
  })
})
