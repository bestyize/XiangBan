import { test } from 'node:test'
import assert from 'node:assert/strict'
import { aiBucketLabel, formatAiCredits } from '../src/lib/aiCredits.ts'

test('converts integer gateway quota to 小伴点 at $10 = 1,000 credits', () => {
  for (const [quota, expected] of [
    [5_000_000, '1,000 点'], [500_000, '100 点'], [250_000, '50 点'],
    [62_500, '12.5 点'], [500, '0.1 点'], [1, '0.0002 点'],
    [1_234_567, '246.9134 点'], [75_000_000, '15,000 点'],
    [0, '0 点'], [-500, '−0.1 点'],
  ] as const) assert.equal(formatAiCredits(quota), expected)
})

test('respects configured quota units, defaults old servers and keeps tiny charges visible', () => {
  assert.equal(formatAiCredits(10_000_000, 1_000_000), '1,000 点')
  for (const unit of [undefined, 0, -1, NaN, Infinity]) {
    assert.equal(formatAiCredits(5_000_000, unit), '1,000 点')
  }
  assert.equal(formatAiCredits(1, 2_000_000), '<0.0001 点')
})

test('labels AI buckets separately from forum community points', () => {
  assert.equal(aiBucketLabel('trial'), '试用点数')
  assert.equal(aiBucketLabel('monthly'), '月卡')
  assert.equal(aiBucketLabel('topup'), '加油包')
  assert.equal(aiBucketLabel('grant'), '点数')
})
