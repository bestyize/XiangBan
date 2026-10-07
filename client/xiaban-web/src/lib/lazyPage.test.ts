import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { reloadForNewBuild } from './lazyPage.ts'

describe('reloadForNewBuild', () => {
  const store = new Map<string, string>()
  const reload = vi.fn()

  beforeEach(() => {
    store.clear()
    reload.mockClear()
    vi.stubGlobal('sessionStorage', {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => { store.set(k, v) },
    })
    vi.stubGlobal('location', { reload })
  })
  afterEach(() => { vi.unstubAllGlobals() })

  it('reloads once, then refuses again within ten seconds so a broken file cannot loop', () => {
    expect(reloadForNewBuild(1_000_000)).toBe(true)
    expect(reload).toHaveBeenCalledTimes(1)
    expect(reloadForNewBuild(1_005_000)).toBe(false)
    expect(reload).toHaveBeenCalledTimes(1)
  })

  it('reloads again for a later deploy', () => {
    reloadForNewBuild(1_000_000)
    expect(reloadForNewBuild(1_020_000)).toBe(true)
    expect(reload).toHaveBeenCalledTimes(2)
  })

  it('does not reload when storage is unavailable, since the loop guard would not work', () => {
    vi.stubGlobal('sessionStorage', { getItem: () => { throw new Error('blocked') }, setItem: () => {} })
    expect(reloadForNewBuild()).toBe(false)
    expect(reload).not.toHaveBeenCalled()
  })
})
