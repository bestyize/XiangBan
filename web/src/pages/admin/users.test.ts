import { describe, expect, it } from 'vitest'
import { csvCell, userCsv } from './users.ts'

describe('admin CSV exports', () => {
  it('escapes delimiters and spreadsheet formulas supplied through user fields', () => {
    expect(csvCell('a,"b"')).toBe('"a,""b"""')
    expect(csvCell('=HYPERLINK("x")')).toBe('"\'=HYPERLINK(""x"")"')
    expect(csvCell('  +1')).toBe('"\'  +1"')
    expect(csvCell('@someone')).toBe('"\'@someone"')
    expect(csvCell(null)).toBe('""')
  })
  it('exports just the supplied page with readable types, verification dates and UTF-8 BOM', () => {
    const csv = userCsv([{ id: 12, email: null, emailVerifiedAt: null, nickname: '=1+1', companionName: '小伴',
      createdAt: 1, onboardedAt: null, lastActiveAt: 1, deviceCount: 2, checkinCount: 0, hasNote: false }])
    expect(csv.startsWith('\uFEFF')).toBe(true)
    expect(csv.split('\r\n')).toHaveLength(2)
    expect(csv).toContain('"游客账号"')
    expect(csv).toContain('"\'=1+1"')
    expect(csv).toContain('"未完成"')
  })
})
