import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

interface TubeLayout {
  columns: number
  rows: number
  tubeWidth: number
  tubeHeight: number
  gap: number
  rowGap: number
  paddingX: number
  paddingTop: number
  paddingBottom: number
  startColumns: number[]
}
interface LayoutApi {
  getTubeLayout(count: number, width: number, height: number): TubeLayout | null
}

// Execute the same classic script used by the game, with no duplicate layout algorithm.
const context: { XiabanWaterSortLayout?: LayoutApi } = {}
runInNewContext(readFileSync(new URL('../../public/game-packs/water-sort/layout.js', import.meta.url), 'utf8'), context)
const { getTubeLayout } = context.XiabanWaterSortLayout!

function expectFits(count: number, width: number, height: number) {
  const layout = getTubeLayout(count, width, height)!
  expect(layout).not.toBeNull()
  expect(layout.startColumns).toHaveLength(count)
  expect(layout.rows).toBe(Math.ceil(count / layout.columns))
  expect(layout.tubeWidth).toBeGreaterThan(0)
  expect(layout.tubeHeight).toBeGreaterThan(0)
  // Each cup button reserves 8px horizontally and 24px for its number, gap and padding.
  const occupiedWidth = layout.columns * (layout.tubeWidth + 8) + (layout.columns - 1) * layout.gap + 2 * layout.paddingX
  const occupiedHeight = layout.rows * (layout.tubeHeight + 24) + (layout.rows - 1) * layout.rowGap + layout.paddingTop + layout.paddingBottom
  expect(occupiedWidth, `${count} cups in ${width}×${height}: width`).toBeLessThanOrEqual(width)
  expect(occupiedHeight, `${count} cups in ${width}×${height}: height`).toBeLessThanOrEqual(height)
  for (let row = 0; row < layout.rows; row += 1) {
    const starts = layout.startColumns.slice(row * layout.columns, (row + 1) * layout.columns)
    expect(starts[0]).toBeGreaterThanOrEqual(1)
    expect(starts.at(-1)! + 1).toBeLessThanOrEqual(layout.columns * 2)
    for (let index = 1; index < starts.length; index += 1) expect(starts[index] - starts[index - 1]).toBeGreaterThanOrEqual(2)
  }
  return layout
}

describe('water sort adaptive cup layout', () => {
  it('makes four cups taller than the previous 150px cap on a 390px phone', () => {
    // 390px viewport minus the game's 14px padding on each side.
    const layout = expectFits(4, 362, 650)
    expect(layout.rows).toBe(1)
    expect(layout.tubeHeight).toBeGreaterThan(150)
    expect(layout.tubeHeight).toBeGreaterThan(layout.tubeWidth * 3)
  })

  it.each([6, 10])('arranges %i cups across two balanced rows on a portrait phone', (count) => {
    const layout = expectFits(count, 362, 650)
    expect(layout.rows).toBe(2)
    expect(layout.columns).toBe(count / 2)
  })

  it.each([7, 9])('centers the shorter final row for %i cups', (count) => {
    const layout = expectFits(count, 362, 650)
    const finalStarts = layout.startColumns.slice((layout.rows - 1) * layout.columns)
    // Two tracks per cup: leading and trailing empty tracks must be equal.
    const emptyLeft = finalStarts[0] - 1
    const emptyRight = layout.columns * 2 - (finalStarts.at(-1)! + 1)
    expect(emptyLeft).toBeGreaterThan(0)
    expect(emptyLeft).toBe(emptyRight)
  })

  it.each([
    { width: 292, height: 280 }, // 320px phone with the game's horizontal padding.
    { width: 292, height: 440 },
    { width: 362, height: 620 },
    { width: 452, height: 720 },
    { width: 760, height: 230 }, // Short landscape container.
    { width: 1000, height: 330 },
  ])('fits all shipped cup counts inside $width×$height', ({ width, height }) => {
    for (const count of [4, 5, 6, 7, 8, 9, 10]) expectFits(count, width, height)
  })

  it('uses a single row when a wide container can accommodate all cups', () => {
    const layout = expectFits(10, 1000, 230)
    expect(layout.rows).toBe(1)
    expect(layout.columns).toBe(10)
  })

  it('recomputes cup sizing and row placement after a viewport change', () => {
    const portrait = expectFits(10, 362, 650)
    const landscape = expectFits(10, 1000, 230)
    expect(portrait.rows).toBe(2)
    expect(landscape.rows).toBe(1)
    expect(portrait.tubeHeight).not.toBe(landscape.tubeHeight)
    expect(getTubeLayout(10, 362, 650)).toEqual(portrait)
  })

  it('rejects unsupported cup counts and unavailable or nonfinite dimensions', () => {
    for (const count of [0, -1, 4.5, 11, NaN]) expect(getTubeLayout(count, 362, 650)).toBeNull()
    for (const [width, height] of [[0, 650], [79, 650], [362, 99], [Infinity, 650], [362, NaN]]) expect(getTubeLayout(4, width, height)).toBeNull()
  })

  it('returns no layout when padding, spacing and labels exhaust a tiny container', () => {
    // Previously returned 89×106px of occupied space for an 80×100px input.
    expect(getTubeLayout(10, 80, 100)).toBeNull()
  })
})
