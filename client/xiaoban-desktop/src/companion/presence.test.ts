import { describe, expect, it } from 'vitest'
import { EXPRESSION_NAMES } from './expressions.ts'
import type { Beat } from './mind.ts'
import { listenPresence, presenceBeats, presencePlay, type PresenceKind } from './presence.ts'

const KINDS: PresenceKind[] = ['welcome_back', 'first_today', 'late_night']

function harness(enabled = () => true) {
  let emit: (payload: unknown) => void = () => {}
  let stopped = false
  const played: Beat[][] = []
  const done = listenPresence(
    async (event, callback) => {
      expect(event).toBe('companion-presence')
      emit = callback
      return () => { stopped = true }
    },
    (beats) => played.push(beats),
    enabled,
  )
  return { done, played, emit: (payload: unknown) => emit(payload), stopped: () => stopped }
}

describe('companion presence', () => {
  it('maps every kind to existing expressions with positive durations', () => {
    for (const kind of KINDS) {
      const steps = presencePlay(kind)
      expect(steps.length).toBeGreaterThan(0)
      for (const step of steps) {
        expect(EXPRESSION_NAMES).toContain(step.expression)
        expect(step.seconds).toBeGreaterThan(0)
      }
    }
    expect(presencePlay('welcome_back')[0].expression).toBe('greeting')
    expect(presencePlay('first_today')[0].expression).toBe('greeting')
    expect(presencePlay('late_night')[0].expression).toBe('sleepy')
  })

  it('returns fresh copies and Mind beats', () => {
    presencePlay('late_night')[0].seconds = 99
    expect(presencePlay('late_night')[0].seconds).toBe(3.5)
    expect(presenceBeats('late_night')).toEqual([{ expr: 'sleepy', hold: 3.5 }])
  })

  it('plays the mapped beats for each event', async () => {
    const h = harness()
    await h.done
    for (const kind of KINDS) h.emit({ kind })
    expect(h.played).toEqual(KINDS.map(presenceBeats))
  })

  it('ignores events while disabled and reacts again once enabled', async () => {
    let on = false
    const h = harness(() => on)
    await h.done
    h.emit({ kind: 'first_today' })
    expect(h.played).toEqual([])
    on = true
    h.emit({ kind: 'first_today' })
    expect(h.played).toHaveLength(1)
  })

  it('ignores unknown or malformed payloads', async () => {
    const h = harness()
    await h.done
    for (const payload of [{ kind: 'dance' }, { kind: 'toString' }, { kind: 3 }, {}, null, undefined, 'late_night']) h.emit(payload)
    expect(h.played).toEqual([])
  })

  it('hands back the unsubscribe function', async () => {
    const h = harness()
    const stop = await h.done
    stop()
    expect(h.stopped()).toBe(true)
  })
})
