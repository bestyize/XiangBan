import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { storage } from './storage.ts'

describe('storage fallback', () => {
  const persisted = new Map<string, string>()
  let writable = true
  const keys = ['fallback-answers', 'fallback-other']

  beforeEach(() => {
    persisted.clear()
    writable = true
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => persisted.get(key) ?? null,
      setItem: (key: string, value: string) => {
        if (!writable) throw new Error('Storage write blocked')
        persisted.set(key, value)
      },
      removeItem: (key: string) => {
        if (!writable) throw new Error('Storage delete blocked')
        persisted.delete(key)
      },
    })
    keys.forEach((key) => storage.remove(key))
  })

  afterEach(() => {
    writable = true
    keys.forEach((key) => storage.remove(key))
    vi.unstubAllGlobals()
  })

  it('keeps new answers readable when writes are blocked but reads still work', () => {
    writable = false
    storage.set(keys[0], 'new answers')
    expect(storage.get(keys[0])).toBe('new answers')
  })

  it('prefers the latest answers over stale persisted answers after a failed write', () => {
    storage.set(keys[0], 'old answers')
    storage.set(keys[1], 'another quiz')
    writable = false
    storage.set(keys[0], 'updated answers')
    expect(storage.get(keys[0])).toBe('updated answers')
    expect(storage.get(keys[1])).toBe('another quiz')
  })

  it('keeps a restarted attempt cleared even when persisted deletion is blocked', () => {
    storage.set(keys[0], 'completed answers')
    writable = false
    storage.remove(keys[0])
    expect(storage.get(keys[0])).toBeNull()
    storage.set(keys[0], 'restarted answers')
    expect(storage.get(keys[0])).toBe('restarted answers')
  })

  it('returns to persisted storage when a later write succeeds', () => {
    writable = false
    storage.set(keys[0], 'temporary answers')
    writable = true
    storage.set(keys[0], 'saved answers')
    persisted.set(keys[0], 'another tab updated the answers')
    expect(storage.get(keys[0])).toBe('another tab updated the answers')
  })
})
