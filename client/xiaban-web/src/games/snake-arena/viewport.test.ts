import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

interface View { physicalWidth: number; physicalHeight: number; width: number; height: number; rotation: number }
interface Insets { top: number; right: number; bottom: number; left: number }
interface Viewport {
  layout(width: number, height: number, rotation?: number): View
  transform(view: View): string
  point(view: View, x: number, y: number): { x: number; y: number }
  safeArea(view: View, insets: Insets): Insets
}
const context: { XiabanSnakeViewport?: Viewport } = {}
runInNewContext(readFileSync(new URL('../../../public/game-packs/snake-arena/viewport.js', import.meta.url), 'utf8'), context)
const viewport = context.XiabanSnakeViewport!

describe('snake arena rotated viewport', () => {
  it('preserves a native landscape viewport and its input coordinates', () => {
    const view = viewport.layout(844, 390, -1)
    expect(view).toMatchObject({ width: 844, height: 390, rotation: 0 })
    expect(viewport.transform(view)).toBe('none')
    expect(viewport.point(view, 180, 270)).toEqual({ x: 180, y: 270 })
  })

  it.each([1, -1])('swaps portrait dimensions without shrinking the playing area for direction %i', (rotation) => {
    const view = viewport.layout(390, 844, rotation)
    expect(view).toMatchObject({ width: 844, height: 390, rotation })
  })

  it('maps all physical corners into the clockwise game plane', () => {
    const view = viewport.layout(390, 844)
    expect(viewport.transform(view)).toBe('translate(390px, 0) rotate(90deg)')
    expect(viewport.point(view, 0, 0)).toEqual({ x: 0, y: 390 })
    expect(viewport.point(view, 390, 0)).toEqual({ x: 0, y: 0 })
    expect(viewport.point(view, 0, 844)).toEqual({ x: 844, y: 390 })
    expect(viewport.point(view, 390, 844)).toEqual({ x: 844, y: 0 })
  })

  it('maps all physical corners into the counterclockwise game plane', () => {
    const view = viewport.layout(390, 844, -1)
    expect(viewport.transform(view)).toBe('translate(0, 844px) rotate(-90deg)')
    expect(viewport.point(view, 0, 0)).toEqual({ x: 844, y: 0 })
    expect(viewport.point(view, 390, 0)).toEqual({ x: 844, y: 390 })
    expect(viewport.point(view, 0, 844)).toEqual({ x: 0, y: 0 })
    expect(viewport.point(view, 390, 844)).toEqual({ x: 0, y: 390 })
  })

  it('keeps notch and home indicator insets attached to their physical edges', () => {
    const physical = { top: 44, right: 8, bottom: 34, left: 4 }
    expect(viewport.safeArea(viewport.layout(390, 844), physical)).toEqual({ top: 8, right: 34, bottom: 4, left: 44 })
    expect(viewport.safeArea(viewport.layout(390, 844, -1), physical)).toEqual({ top: 4, right: 44, bottom: 8, left: 34 })
    expect(viewport.safeArea(viewport.layout(844, 390), physical)).toEqual(physical)
  })
})
