import { describe, expect, it } from 'vitest'
import { createLeafGeometry, defaultLeaf, fillLeaf, LENGTH_SEGMENTS, outline, vertexCount, WIDTH_SEGMENTS, type LeafShape } from './leaf.ts'

const blade = (shape: Partial<LeafShape> = {}) => {
  const out = new Float32Array(vertexCount * 3)
  fillLeaf({ ...defaultLeaf, ...shape }, out)
  return out
}
const at = (out: Float32Array, i: number, j: number) => {
  const k = (i * (WIDTH_SEGMENTS + 1) + j) * 3
  return { x: out[k], y: out[k + 1], z: out[k + 2] }
}

describe('leaf outline', () => {
  it('is closed at both ends and widest about a third of the way along', () => {
    expect(outline(0)).toBeCloseTo(0, 6)
    expect(outline(1)).toBeCloseTo(0, 6)
    let widest = 0
    for (let u = 0; u <= 1; u += 0.01) if (outline(u) > outline(widest)) widest = u
    expect(widest).toBeGreaterThan(0.28)
    expect(widest).toBeLessThan(0.5)
  })
})

describe('leaf blade', () => {
  it('stays finite however far it is bent, rolled or twisted', () => {
    for (const shape of [{}, { bend: -2, curlA: 1.5, curlB: 1.5 }, { bend: 2.5, twist: 1.4, fold: 0.9, wave: 0.3 }, { pitch: -1, curlA: 0.001 }]) {
      expect([...blade(shape)].every(Number.isFinite), JSON.stringify(shape)).toBe(true)
    }
  })

  it('starts at the stalk and closes to a point at the tip', () => {
    const out = blade()
    const base = at(out, 0, WIDTH_SEGMENTS / 2)
    expect(Math.hypot(base.x, base.y, base.z)).toBeLessThan(1e-6)
    const left = at(out, LENGTH_SEGMENTS, 0)
    const right = at(out, LENGTH_SEGMENTS, WIDTH_SEGMENTS)
    expect(Math.hypot(left.x - right.x, left.y - right.y, left.z - right.z)).toBeLessThan(1e-6)
  })

  it('is about as long as asked when it is not bent', () => {
    const tip = at(blade({ bend: 0, pitch: 0, length: 0.8 }), LENGTH_SEGMENTS, WIDTH_SEGMENTS / 2)
    expect(tip.x).toBeCloseTo(0.8, 2)
    expect(tip.y).toBeCloseTo(0, 6)
  })

  it('droops at the tip when bent', () => {
    const flat = at(blade({ bend: 0 }), LENGTH_SEGMENTS, WIDTH_SEGMENTS / 2)
    const bent = at(blade({ bend: 1.2 }), LENGTH_SEGMENTS, WIDTH_SEGMENTS / 2)
    expect(bent.y).toBeLessThan(flat.y)
  })

  it('rolls the curled edge up and back over the blade, and leaves the other edge alone', () => {
    const flat = blade({ fold: 0.2, wave: 0 })
    const curled = blade({ fold: 0.2, wave: 0, curlA: 1.2 })
    const mid = Math.round(LENGTH_SEGMENTS * 0.45)
    const before = at(flat, mid, 0)
    const after = at(curled, mid, 0)
    expect(Math.abs(after.z)).toBeLessThan(Math.abs(before.z))
    expect(after.y).toBeGreaterThan(before.y)
    expect(at(curled, mid, WIDTH_SEGMENTS)).toEqual(at(flat, mid, WIDTH_SEGMENTS))
  })

  it('builds a closed grid with one normal per vertex', () => {
    const geometry = createLeafGeometry()
    expect(geometry.getAttribute('position').count).toBe(vertexCount)
    expect(geometry.getIndex()!.count).toBe(LENGTH_SEGMENTS * WIDTH_SEGMENTS * 6)
    expect([...(geometry.getAttribute('normal').array as Float32Array)].every(Number.isFinite)).toBe(true)
  })
})
