import { describe, expect, it } from 'vitest'
import { buildBodyGeometry, buildPatch, POT, rand, soilHeight, surfaceZ, wallRadius } from './pot.ts'

const finite = (array: ArrayLike<number>) => [...(array as number[])].every(Number.isFinite)

describe('pot body', () => {
  const geometry = buildBodyGeometry()
  geometry.computeBoundingBox()

  it('is a clean mesh that stands on the ground, about 1.1 high and 1.4 across at the rim', () => {
    expect(finite(geometry.getAttribute('position').array)).toBe(true)
    expect(finite(geometry.getAttribute('normal').array)).toBe(true)
    const box = geometry.boundingBox!
    expect(box.min.y).toBeCloseTo(0, 6)
    expect(box.max.y).toBeGreaterThan(1.08)
    expect(box.max.y).toBeLessThan(1.14)
    expect(box.max.x - box.min.x).toBeGreaterThan(1.3)
    expect(box.max.x - box.min.x).toBeLessThan(1.5)
    expect(box.max.z - box.min.z).toBeLessThan(box.max.x - box.min.x)
  })

  it('carries a baked shade that darkens under the lip and never goes below black or above white', () => {
    const colors = geometry.getAttribute('color').array as Float32Array
    expect(Math.min(...colors)).toBeGreaterThan(0.4)
    expect(Math.max(...colors)).toBeLessThanOrEqual(1)
    expect(Math.min(...colors)).toBeLessThan(0.85)
  })
})

describe('the wall the screen sits on', () => {
  it('flares outward going up, and curves away from the front toward the sides', () => {
    expect(wallRadius(0.7)).toBeGreaterThan(wallRadius(0.2))
    expect(surfaceZ(0, 0.7)).toBeGreaterThan(surfaceZ(0, 0.2))
    expect(surfaceZ(0.3, 0.46)).toBeLessThan(surfaceZ(0, 0.46))
    expect(surfaceZ(0.2, 0.46)).toBeLessThan(surfaceZ(0, 0.46))
    expect(surfaceZ(5, 0.46)).toBeGreaterThanOrEqual(0)
  })

  it('has a front depth of the shape ratio times the half-width', () => {
    expect(surfaceZ(0, 0.46)).toBeCloseTo(POT.depth * wallRadius(0.46), 9)
  })
})

describe('inlays on the wall', () => {
  const lift = (d: number) => 0.02 * d
  const patch = buildPatch({ cy: POT.screenY, halfW: POT.screenHalfW, halfH: POT.screenHalfH, exponent: 5, lift, segments: 24 })

  it('follows the wall and stays in the box asked for', () => {
    const position = patch.getAttribute('position').array as Float32Array
    expect(finite(position)).toBe(true)
    for (let k = 0; k < position.length; k += 3) {
      expect(Math.abs(position[k])).toBeLessThanOrEqual(POT.screenHalfW + 1e-6)
      expect(Math.abs(position[k + 1] - POT.screenY)).toBeLessThanOrEqual(POT.screenHalfH + 1e-6)
      expect(position[k + 2]).toBeGreaterThanOrEqual(surfaceZ(position[k], position[k + 1]) - 1e-5)
    }
  })

  it('covers the texture exactly once', () => {
    const uv = patch.getAttribute('uv').array as Float32Array
    expect(Math.min(...uv)).toBeGreaterThanOrEqual(0)
    expect(Math.max(...uv)).toBeLessThanOrEqual(1)
    expect(Math.max(...uv)).toBeCloseTo(1, 6)
  })

  it('faces the front', () => {
    const normal = patch.getAttribute('normal').array as Float32Array
    let sum = 0
    for (let k = 2; k < normal.length; k += 3) sum += normal[k]
    expect(sum).toBeGreaterThan(0)
  })
})

describe('soil and random numbers', () => {
  it('lies a little below the rim, lumpy but level', () => {
    for (const [x, z] of [[0, 0], [0.3, 0.1], [-0.4, -0.2], [0.5, 0]]) {
      expect(soilHeight(x, z)).toBeGreaterThan(POT.soilY - 0.05)
      expect(soilHeight(x, z)).toBeLessThan(POT.soilY + 0.08)
    }
  })

  it('gives the same sequence for the same seed', () => {
    const a = rand(42)
    const b = rand(42)
    expect([a(), a(), a()]).toEqual([b(), b(), b()])
    expect(rand(1)()).not.toBe(rand(2)())
  })
})
