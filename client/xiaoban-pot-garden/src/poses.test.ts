import { describe, expect, it } from 'vitest'
import { EXPRESSIONS, isExpression, LABELS, mixPose, poseFor } from './poses.ts'

describe('poses', () => {
  it('has a label and a pose for every expression', () => {
    expect(EXPRESSIONS).toHaveLength(8)
    for (const expression of EXPRESSIONS) {
      expect(LABELS[expression], expression).toBeTruthy()
      expect(Object.values(poseFor(expression)).every(value => typeof value === 'boolean' || Number.isFinite(value)), expression).toBe(true)
    }
  })

  it('lifts the leaves for joy and lets them droop for sadness', () => {
    expect(poseFor('excited').leftLift).toBeGreaterThan(poseFor('happy').leftLift)
    expect(poseFor('happy').leftLift).toBeGreaterThan(poseFor('idle').leftLift)
    expect(poseFor('sad').leftLift).toBeLessThan(poseFor('sleepy').leftLift + 0.2)
    expect(poseFor('sad').leftLift).toBeLessThan(0)
    expect(poseFor('sad').leftBend).toBeGreaterThan(poseFor('happy').leftBend)
  })

  it('shows the burst lines only when pleased', () => {
    const showing = EXPRESSIONS.filter(expression => poseFor(expression).sparkles).sort()
    expect(showing).toEqual(['excited', 'happy', 'love'])
  })

  it('blends between two poses and ends exactly on the target', () => {
    const from = poseFor('sad')
    const to = poseFor('excited')
    expect(mixPose(from, to, 0).leftLift).toBeCloseTo(from.leftLift, 9)
    expect(mixPose(from, to, 1).leftLift).toBeCloseTo(to.leftLift, 9)
    expect(mixPose(from, to, 0.5).leftLift).toBeCloseTo((from.leftLift + to.leftLift) / 2, 9)
    expect(mixPose(from, to, 0.5).sparkles).toBe(to.sparkles)
  })

  it('recognises only its own expression names', () => {
    expect(isExpression('happy')).toBe(true)
    expect(isExpression('angry')).toBe(false)
    expect(isExpression(null)).toBe(false)
  })
})
