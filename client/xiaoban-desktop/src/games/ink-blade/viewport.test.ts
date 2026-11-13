import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

type Bounds = { left: number; right: number }
type Frame = { min: number; max: number; safeMin: number; safeMax: number }
interface Viewport {
  combatFrame(player: Bounds, opponents: Bounds[], width: number, left: number, right: number, marginLeft: number, marginRight: number): Frame
}
const context: { XiabanInkViewport?: Viewport } = {}
runInNewContext(readFileSync(new URL('../../../public/game-packs/ink-blade/viewport.js', import.meta.url), 'utf8'), context)
const viewport = context.XiabanInkViewport!

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value))
}

describe('ink blade touch combat framing', () => {
  const width = 800, left = 176, right = 160, marginLeft = 12, marginRight = 20
  const frame = (player: Bounds, opponents: Bounds[]) =>
    viewport.combatFrame(player, opponents, width, left, right, marginLeft, marginRight)
  const projected = (body: Bounds, center: number) => ({ left: body.left - center + width / 2, right: body.right - center + width / 2 })

  it('shifts the camera when a nearby enemy is farther toward the buttons than the player', () => {
    const player = { left: 4975, right: 5015 }, enemy = { left: 5025, right: 5065 }
    const { safeMin, safeMax } = frame(player, [enemy])
    const center = clamp(4680, safeMin, safeMax)
    expect(projected(player, center).left).toBeGreaterThanOrEqual(left)
    expect(projected(enemy, center).right).toBeLessThanOrEqual(width - right)
  })

  it('keeps both fighters in view across the arena even when the controls cannot be avoided', () => {
    const player = { left: 1020, right: 1060 }, enemy = { left: 1680, right: 1720 }
    const { min, max, safeMin, safeMax } = frame(player, [enemy])
    const center = clamp(1360, safeMin, safeMax)
    expect(projected(player, center).left).toBeGreaterThanOrEqual(marginLeft)
    expect(projected(enemy, center).right).toBeLessThanOrEqual(width - marginRight)
    expect(projected(player, center).left < left || projected(enemy, center).right > width - right).toBe(true)
    expect(projected(enemy, min).right).toBeLessThanOrEqual(width - marginRight)
    expect(projected(player, max).left).toBeGreaterThanOrEqual(marginLeft)
  })

  it('keeps a close opponent clear when a distant one is beyond the view', () => {
    const player = { left: 1220, right: 1260 }, near = { left: 1130, right: 1170 }, far = { left: 2130, right: 2170 }
    const { safeMin, safeMax } = frame(player, [near, far])
    const center = clamp(1470, safeMin, safeMax)
    expect(projected(near, center).left).toBeGreaterThanOrEqual(left)
    expect(projected(player, center).right).toBeLessThanOrEqual(width - right)
  })
})
