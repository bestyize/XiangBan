import { beforeEach, describe, expect, it, vi } from 'vitest'

const api = vi.fn()
vi.mock('../../api/client.ts', () => ({ api: (...args: unknown[]) => api(...args) }))
const store = new Map<string, string>()
vi.mock('../../lib/storage.ts', () => ({
  storage: { get: (key: string) => store.get(key) ?? null, set: (key: string, value: string) => { store.set(key, value) }, remove: (key: string) => { store.delete(key) } },
}))

const { loadGameSave, readLocalSave, SAVE_MAX_BYTES, storeGameSave, validSave, writeLocalSave } = await import('./saves.ts')

beforeEach(() => { api.mockReset(); store.clear() })

describe('game saves', () => {
  it('accepts only plain JSON objects within the size limit', () => {
    expect(validSave({ levels: {} })).toBe(true)
    for (const bad of [null, [], 'save', 7, { big: 'x'.repeat(SAVE_MAX_BYTES) }]) expect(validSave(bad)).toBe(false)
    const cyclic: Record<string, unknown> = {}
    cyclic.self = cyclic
    expect(validSave(cyclic)).toBe(false)
  })

  it('keeps guest progress in this browser per game', async () => {
    await storeGameSave('ink-blade', false, { v: 1 })
    writeLocalSave('other', { v: 2 })
    expect(await loadGameSave('ink-blade', false)).toEqual({ v: 1 })
    expect(readLocalSave('other')).toEqual({ v: 2 })
    expect(api).not.toHaveBeenCalled()
  })

  it('stores registered progress on the server', async () => {
    api.mockResolvedValueOnce({ data: { v: 3 }, updatedAt: 1 })
    expect(await loadGameSave('ink-blade', true)).toEqual({ v: 3 })
    expect(api).toHaveBeenCalledWith('/api/games/saves/ink-blade')
    api.mockResolvedValueOnce({})
    await storeGameSave('ink-blade', true, { v: 4 }, true)
    expect(api).toHaveBeenLastCalledWith('/api/games/saves/ink-blade', expect.objectContaining({ method: 'PUT', keepalive: true, body: expect.objectContaining({ data: { v: 4 } }) }))
  })

  it('moves guest progress into an account that has no save yet', async () => {
    writeLocalSave('ink-blade', { v: 1, levels: { '1-1': {} } })
    api.mockResolvedValueOnce({ data: null, updatedAt: null }).mockResolvedValueOnce({})
    expect(await loadGameSave('ink-blade', true)).toEqual({ v: 1, levels: { '1-1': {} } })
    expect(api).toHaveBeenLastCalledWith('/api/games/saves/ink-blade', expect.objectContaining({ method: 'PUT' }))
    expect(readLocalSave('ink-blade')).toBeNull()
  })

  it('prefers the account save and leaves guest progress alone when both exist', async () => {
    writeLocalSave('ink-blade', { v: 1 })
    api.mockResolvedValueOnce({ data: { v: 9 }, updatedAt: 5 })
    expect(await loadGameSave('ink-blade', true)).toEqual({ v: 9 })
    expect(api).toHaveBeenCalledTimes(1)
  })

  it('surfaces a failed server read so the host does not start on an empty save', async () => {
    api.mockRejectedValueOnce(new Error('offline'))
    await expect(loadGameSave('ink-blade', true)).rejects.toThrow('offline')
  })
})
