import { afterEach, expect, it, vi } from 'vitest'
import { api, getToken, onUnauthorized, setToken } from './client.ts'
afterEach(() => { vi.unstubAllGlobals(); onUnauthorized(null); setToken(null) })
it('does not sign out a new account when an old mutation answers 401', async () => {
  let finish!: (response: unknown) => void
  vi.stubGlobal('fetch', vi.fn(() => new Promise((resolve) => { finish = resolve })))
  const renew = vi.fn().mockResolvedValue(true)
  onUnauthorized(renew); setToken('old-account-token')
  const pending = api('/api/profile', { method: 'PUT', body: { nickname: 'Alice' } })
  setToken('new-account-token')
  finish({ status: 401, ok: false })
  await expect(pending).rejects.toMatchObject({ code: 'account_changed' })
  expect(renew).not.toHaveBeenCalled(); expect(getToken()).toBe('new-account-token')
})
it('does not replay a write into a different guest account after reauthentication', async () => {
  const fetcher = vi.fn().mockResolvedValue({ status: 401, ok: false })
  vi.stubGlobal('fetch', fetcher); setToken('expired-user-token')
  onUnauthorized(async () => { setToken('new-guest-token'); return false })
  await expect(api('/api/profile', { method: 'PUT', body: { nickname: 'Alice' } })).rejects.toMatchObject({ code: 'account_changed' })
  expect(fetcher).toHaveBeenCalledTimes(1)
})
