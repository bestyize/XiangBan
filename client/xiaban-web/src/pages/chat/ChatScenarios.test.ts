import { describe, expect, it } from 'vitest'
import { getScenario } from './ChatScenarios.ts'

describe('ChatScenarios', () => {
  it('returns evening commute scenario between 17 and 20 on weekdays', () => {
    const s = getScenario(18, false)
    expect(s.periodName).toBe('下班通勤')
    expect(s.chips).toContain('累瘫了求安慰')
  })

  it('returns home recharge scenario between 20 and 23 on weekdays', () => {
    const s = getScenario(21, false)
    expect(s.periodName).toBe('居家回血')
    expect(s.chips).toContain('今天遇到件离谱事')
  })

  it('returns late night treehole scenario late at night', () => {
    const s1 = getScenario(23, false)
    expect(s1.periodName).toBe('深夜树洞')
    expect(s1.chips).toContain('有点睡不着')

    const s2 = getScenario(2, false)
    expect(s2.periodName).toBe('深夜树洞')
  })

  it('returns weekend daytime scenario on weekends', () => {
    const s = getScenario(14, true)
    expect(s.periodName).toBe('周末时光')
    expect(s.greeting).toContain('今天放假')
    expect(s.chips).toContain('无聊发呆中')
  })

  it('returns daytime scenario on weekdays during the day', () => {
    const s = getScenario(14, false)
    expect(s.periodName).toBe('白天时光')
    expect(s.chips).toContain('摸鱼休息一下')
  })
})
