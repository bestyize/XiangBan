// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defaults } from './bridge.ts'
import { PetView } from './pet.ts'
import type { PetDefinition } from './pets/model.ts'

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

  it('pokes a full-body companion and only hit-tests its visible pose', () => {
    const definition: PetDefinition = {
      id: 'actor', name: '溟月', description: '全身伙伴', kind: 'character',
      poses: { idle: 'standing', happy: 'smile' },
      parts: ['standing', 'smile'].map(id => ({
        id, name: id, pivot: [160, 280], motion: 'none',
        shapes: [{ type: 'rect', x: 100, y: 100, width: 60, height: 100, fill: '#123456' }],
      })),
    }
    vi.spyOn(Math, 'random').mockReturnValue(0.99) // happy, then proud
    const pet = new PetView(document.body, { ...defaults }, undefined, definition)
    const active = () => pet.host.querySelector<SVGElement>('[data-pet-part]:not([display])')!
    const standing = active()
    expect(standing.dataset.petPart).toBe('standing')
    pet.reactions.setEnabled(false)
    try {
      pet.poke()
      const [id, frame] = [...frames][0]
      frames.delete(id)
      frame(100)
      expect(active().dataset.petPart).toBe('smile')
      expect(standing.getAttribute('display')).toBe('none')
      const art = pet.host.querySelector<HTMLElement>('.pet-art')!
      vi.spyOn(art, 'getBoundingClientRect').mockReturnValue(new DOMRect(0, 0, 320, 320))
      const hidden = standing.querySelector('rect')!
      const visible = active().querySelector('rect')!
      Object.defineProperty(hidden, 'getScreenCTM', { value: () => { throw Error('hidden pose was hit-tested') } })
      Object.defineProperty(visible, 'getScreenCTM', { value: () => null })
      expect(pet.hit(130, 150)).toBe(false)
    } finally {
      pet.destroy()
      vi.restoreAllMocks()
    }
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
