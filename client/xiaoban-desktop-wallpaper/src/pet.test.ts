// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defaults } from './bridge.ts'
import { PetView } from './pet.ts'

describe('PetView animation lifecycle', () => {
  const frames = new Map<number, FrameRequestCallback>()
  let nextFrame = 0

  beforeEach(() => {
    frames.clear()
    nextFrame = 0
    vi.stubGlobal('matchMedia', () => ({ matches: false }))
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
      const id = ++nextFrame
      frames.set(id, callback)
      return id
    })
    vi.stubGlobal('cancelAnimationFrame', (id: number) => { frames.delete(id) })
  })

  afterEach(() => {
    document.body.replaceChildren()
    vi.unstubAllGlobals()
  })

  it('does not keep scheduling frames while paused or hidden', () => {
    const pet = new PetView(document.body, { ...defaults })
    expect(frames.size).toBe(1)
    pet.update({ ...defaults, paused: true })
    expect(frames.size).toBe(0)
    pet.update({ ...defaults })
    expect(frames.size).toBe(1)
    Object.defineProperty(document, 'hidden', { configurable: true, value: true })
    document.dispatchEvent(new Event('visibilitychange'))
    expect(frames.size).toBe(0)
    Object.defineProperty(document, 'hidden', { configurable: true, value: false })
    document.dispatchEvent(new Event('visibilitychange'))
    expect(frames.size).toBe(1)
    pet.destroy()
    expect(frames.size).toBe(0)
  })

  it('keeps the face nodes when two frames have identical markup', () => {
    // With reduced motion the face holds still, so two frames draw the same markup. Otherwise the gaze springs
    // keep moving and whether two frames match is down to rounding.
    vi.stubGlobal('matchMedia', () => ({ matches: true }))
    const pet = new PetView(document.body, { ...defaults })
    const run = (time: number) => {
      const [id, callback] = [...frames][0]
      frames.delete(id)
      callback(time)
    }
    run(100)
    const face = pet.host.querySelector('[data-part="face"]')!
    const firstChild = face.firstChild
    expect(firstChild).not.toBeNull()
    run(117)
    expect(face.firstChild).toBe(firstChild)
    pet.destroy()
  })
})
