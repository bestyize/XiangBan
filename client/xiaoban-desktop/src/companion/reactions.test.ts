import { describe, expect, it, vi } from 'vitest'
import { EFFECT_REACTIONS, Reactions, type ReactionHost } from './reactions.ts'
import { EXPRESSIONS, toSpec } from './expressions.ts'
import type { Beat } from './mind.ts'

const BUILTIN = ['rain', 'snow', 'fireflies', 'petals', 'maple', 'meteors', 'stars', 'butterfly-dream', 'lanterns', 'fireworks', 'aurora', 'galaxy']

function setup(options: { random?: () => number; hour?: number; busy?: boolean } = {}) {
  const state = { busy: options.busy ?? false }
  const played: Beat[][] = []
  const host = {
    play: vi.fn((beats: Beat[]) => { played.push(beats) }),
    stop: vi.fn(),
    lookAt: vi.fn(),
    busy: () => state.busy,
    random: options.random ?? (() => 0.5),
    now: () => new Date(2026, 9, 10, options.hour ?? 22),
  } satisfies ReactionHost
  const reactions = new Reactions(host)
  const run = (seconds: number) => { for (let i = 0; i < seconds; i++) reactions.tick(1) }
  return { reactions, host, played, state, run }
}

describe('companion reactions', () => {
  it('has at least two reactions per built-in effect, using real expressions', () => {
    for (const id of BUILTIN) {
      const profile = EFFECT_REACTIONS[id]
      expect(profile.reactions.length, id).toBeGreaterThanOrEqual(2)
      expect(profile.interval[0], id).toBeGreaterThanOrEqual(20)
      expect(profile.interval[1], id).toBeLessThanOrEqual(60)
      for (const reaction of [...profile.reactions, ...(profile.ended ? [profile.ended] : [])])
        for (const beat of reaction.beats) expect(EXPRESSIONS[toSpec(beat.expr).expression], `${id}/${reaction.id}`).toBeDefined()
    }
  })

  it('reacts at least once to every built-in effect, and not before the first interval', () => {
    for (const id of BUILTIN) {
      const { reactions, played, run } = setup()
      reactions.setEffect(id)
      run(5)
      expect(played, id).toHaveLength(0)
      run(60)
      expect(played.length, id).toBeGreaterThanOrEqual(1)
    }
  })

  it('does nothing for no effect, an unknown id or a custom pack', () => {
    for (const id of ['', 'glitter', 'toString', 'constructor']) {
      const { reactions, played, run } = setup()
      reactions.setEffect(id)
      run(600)
      expect(played, id).toHaveLength(0)
    }
  })

  it('waits a random interval inside the effect range between reactions', () => {
    const low = setup({ random: () => 0 })
    low.reactions.setEffect('snow')
    low.run(15)
    expect(low.played).toHaveLength(1)
    low.run(29)
    expect(low.played).toHaveLength(1)
    low.run(2)
    expect(low.played).toHaveLength(2)
    const high = setup({ random: () => 0.999 })
    high.reactions.setEffect('snow')
    high.run(29)
    expect(high.played).toHaveLength(0)
    high.run(2)
    expect(high.played).toHaveLength(1)
    high.run(58)
    expect(high.played).toHaveLength(1)
    high.run(3)
    expect(high.played).toHaveLength(2)
  })

  it('never repeats the same action twice in a row', () => {
    const { reactions, played, run } = setup({ random: () => 0 })
    reactions.setEffect('rain')
    run(1200)
    expect(played.length).toBeGreaterThan(10)
    played.slice(1).forEach((beats, i) => expect(beats).not.toBe(played[i]))
    const ids = new Set(played)
    expect(ids.size).toBe(EFFECT_REACTIONS.rain.reactions.length)
  })

  it('keeps a cooldown when the effect is switched back and forth', () => {
    const { reactions, played, run } = setup({ random: () => 0 })
    reactions.setEffect('snow')
    run(15)
    expect(played).toHaveLength(1)
    for (const id of ['petals', 'snow', 'maple', 'snow']) { reactions.setEffect(id); run(2) }
    expect(played).toHaveLength(1)
    run(20)
    expect(played.length).toBeLessThanOrEqual(2)
  })

  it('stops at once when the effect changes, and starts a fresh schedule', () => {
    const { reactions, host, played, run } = setup({ random: () => 0 })
    reactions.setEffect('fireflies')
    run(13)
    expect(played).toHaveLength(1)
    reactions.setEffect('')
    expect(host.stop).toHaveBeenCalledTimes(1)
    expect(host.lookAt).toHaveBeenLastCalledWith(null)
    run(300)
    expect(played).toHaveLength(1)
  })

  it('releases the gaze after the beats end', () => {
    const { reactions, host, run } = setup({ random: () => 0 })
    reactions.setEffect('petals')
    run(15)
    expect(host.lookAt).toHaveBeenCalledWith(expect.objectContaining({ y: expect.any(Number) }))
    run(4)
    expect(host.lookAt).toHaveBeenLastCalledWith(null)
  })

  it('shakes off once when rain is switched away, but not when everything is turned off', () => {
    const away = setup()
    away.reactions.setEffect('rain')
    away.reactions.setEffect('snow')
    expect(away.played).toEqual([EFFECT_REACTIONS.rain.ended!.beats])
    const off = setup()
    off.reactions.setEffect('rain')
    off.reactions.setEnabled(false)
    off.reactions.setEffect('')
    expect(off.played).toHaveLength(0)
    const paused = setup()
    paused.reactions.setEffect('rain')
    paused.reactions.setPaused(true)
    paused.reactions.setEffect('snow')
    expect(paused.played).toHaveLength(0)
  })

  it('stops and stays quiet while disabled, and resumes when enabled again', () => {
    const { reactions, host, played, run } = setup({ random: () => 0 })
    reactions.setEffect('stars')
    run(16)
    expect(played).toHaveLength(1)
    reactions.setEnabled(false)
    expect(host.stop).toHaveBeenCalledTimes(1)
    run(600)
    expect(played).toHaveLength(1)
    reactions.setEnabled(true)
    run(600)
    expect(played.length).toBeGreaterThan(1)
  })

  it('does not trigger while paused and stops what is playing', () => {
    const { reactions, host, played, run } = setup({ random: () => 0 })
    reactions.setEffect('maple')
    run(16)
    expect(played).toHaveLength(1)
    reactions.setPaused(true)
    expect(host.stop).toHaveBeenCalledTimes(1)
    run(600)
    expect(played).toHaveLength(1)
    reactions.setPaused(false)
    run(600)
    expect(played.length).toBeGreaterThan(1)
  })

  it('stops and clears everything on destroy', () => {
    const { reactions, host, played, run } = setup({ random: () => 0 })
    reactions.setEffect('fireflies')
    run(13)
    reactions.destroy()
    expect(host.stop).toHaveBeenCalledTimes(1)
    run(600)
    reactions.setEffect('rain')
    reactions.setEnabled(true)
    run(600)
    expect(played).toHaveLength(1)
    expect(reactions.wish()).toBe(false)
  })

  it('defers to direct interaction and does not wipe the interaction it yields to', () => {
    const { reactions, host, played, state, run } = setup({ random: () => 0 })
    reactions.setEffect('snow')
    state.busy = true
    run(120)
    expect(played).toHaveLength(0)
    state.busy = false
    run(5)
    expect(played).toHaveLength(1)
    state.busy = true
    reactions.setEffect('rain')
    expect(host.stop).not.toHaveBeenCalled()
  })

  it('picks night or daytime stargazing from the clock', () => {
    const night = setup({ random: () => 0, hour: 23 })
    night.reactions.setEffect('stars')
    night.run(1500)
    const nightIds = new Set(night.played)
    const day = setup({ random: () => 0, hour: 13 })
    day.reactions.setEffect('stars')
    day.run(1500)
    const dayBeats = EFFECT_REACTIONS.stars.reactions.filter(reaction => reaction.id.endsWith('-day')).map(reaction => reaction.beats)
    const nightBeats = EFFECT_REACTIONS.stars.reactions.filter(reaction => !reaction.id.endsWith('-day')).map(reaction => reaction.beats)
    expect([...nightIds].every(beats => nightBeats.includes(beats))).toBe(true)
    expect(nightIds.size).toBe(2)
    expect(day.played.every(beats => dayBeats.includes(beats))).toBe(true)
  })

  it('turns a click into a wish only while meteors fall', () => {
    const { reactions, host, played, run } = setup()
    expect(reactions.wish()).toBe(false)
    reactions.setEffect('snow')
    expect(reactions.wish()).toBe(false)
    reactions.setEffect('meteors')
    expect(reactions.wish()).toBe(true)
    expect(played).toHaveLength(1)
    expect(toSpec(played[0][0].expr).symbol).toBe('sparkle')
    expect(reactions.wish()).toBe(false)
    run(7)
    expect(reactions.wish()).toBe(true)
    reactions.setEnabled(false)
    run(7)
    expect(reactions.wish()).toBe(false)
    reactions.setEnabled(true)
    reactions.setPaused(true)
    expect(reactions.wish()).toBe(false)
    expect(host.play).toHaveBeenCalledTimes(2)
  })

  it('also makes a wish on a rising lantern', () => {
    const { reactions } = setup()
    reactions.setEffect('fireworks')
    expect(reactions.wish()).toBe(false)
    reactions.setEffect('lanterns')
    expect(reactions.wish()).toBe(true)
  })
})
