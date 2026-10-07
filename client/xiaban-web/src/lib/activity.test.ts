import { describe, expect, it } from 'vitest'
import { layHomeUnderActivity } from './activity.ts'

/** Like window.location + history: the location object follows whatever the history now points at. */
function fakeBrowser(start: string, state: unknown = null) {
  const loc = { pathname: '', search: '', hash: '' }
  const go = (url: string) => {
    const u = new URL(url, 'https://x.test')
    Object.assign(loc, { pathname: u.pathname, search: u.search, hash: u.hash })
  }
  go(start)
  const entries: string[] = [start]
  const h = {
    state,
    entries,
    replaceState(data: unknown, _: string, url = '') { h.state = data; entries[entries.length - 1] = url; go(url) },
    pushState(data: unknown, _: string, url = '') { h.state = data; entries.push(url); go(url) },
  }
  return { loc, h }
}

describe('layHomeUnderActivity', () => {
  it('puts the home page under a directly opened campaign link, keeping the query', () => {
    const { loc, h } = fakeBrowser('/xiaban?from=card&inv=abc')
    expect(layHomeUnderActivity(loc, h)).toBe(true)
    expect(h.entries).toEqual(['/', '/xiaban?from=card&inv=abc'])
  })

  it('sends old card links that pointed at the home page to the campaign', () => {
    const { loc, h } = fakeBrowser('/?from=card')
    layHomeUnderActivity(loc, h)
    expect(h.entries).toEqual(['/', '/xiaban?from=card'])
  })

  it('leaves other pages and reloads alone', () => {
    const other = fakeBrowser('/games')
    expect(layHomeUnderActivity(other.loc, other.h)).toBe(false)
    expect(other.h.entries).toEqual(['/games'])
    const reloaded = fakeBrowser('/xiaban', { xbActivity: true })
    expect(layHomeUnderActivity(reloaded.loc, reloaded.h)).toBe(false)
  })
})
