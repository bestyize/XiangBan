import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

const bootstrap = readFileSync(new URL('../../public/theme-init.js', import.meta.url), 'utf8')

function firstPaint(saved: string | null, systemDark: boolean, storageUnavailable = false) {
  const dataset: Record<string, string> = {}
  let themeColor = ''
  runInNewContext(bootstrap, {
    localStorage: { getItem: () => {
      if (storageUnavailable) throw new Error('Storage unavailable')
      return saved
    } },
    matchMedia: () => ({ matches: systemDark }),
    document: {
      documentElement: { dataset },
      querySelector: () => ({ setAttribute: (_name: string, value: string) => { themeColor = value } }),
    },
  })
  return { theme: dataset.theme, themeColor }
}

describe('appearance before React loads', () => {
  it.each([
    ['light', true, 'light'],
    ['dark', false, 'dark'],
    ['system', true, 'dark'],
    ['system', false, 'light'],
    [null, true, 'dark'],
    [null, false, 'light'],
    ['invalid', true, 'dark'],
    ['invalid', false, 'light'],
  ])('honors saved %s with system dark=%s', (saved, systemDark, expected) => {
    expect(firstPaint(saved as string | null, systemDark as boolean)).toEqual({
      theme: expected, themeColor: expected === 'dark' ? '#141414' : '#f7f7f7',
    })
  })

  it('still opens in the system appearance when storage access is blocked', () => {
    expect(firstPaint(null, true, true)).toEqual({ theme: 'dark', themeColor: '#141414' })
    expect(firstPaint(null, false, true)).toEqual({ theme: 'light', themeColor: '#f7f7f7' })
  })
})
