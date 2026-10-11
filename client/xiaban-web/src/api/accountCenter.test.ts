import { afterEach, expect, it, vi } from 'vitest'
import {
  checkUsername,
  dropAccountSession,
  loginWithCode,
  loginWithPassword,
  needsRegistration,
  register,
  resetPassword,
  sendLoginCode,
  sendResetCode,
} from './accountCenter.ts'

afterEach(() => vi.unstubAllGlobals())

function respond(status: number, body: unknown) {
  return vi.fn().mockResolvedValue({ ok: status < 400, status, json: async () => body })
}

const bodyOf = (fetcher: ReturnType<typeof vi.fn>, call = 0) => JSON.parse(fetcher.mock.calls[call][1].body)

it('asks for a login code or a reset code for the typed identifier', async () => {
  const fetcher = respond(200, { ok: true })
  vi.stubGlobal('fetch', fetcher)
  await sendLoginCode('me@example.com')
  await sendResetCode('me@example.com')
  expect(fetcher.mock.calls[0][0]).toBe('https://account.xiangban.xyz/api/auth/code/send')
  expect(bodyOf(fetcher, 0)).toEqual({ identifier: 'me@example.com', purpose: 'login' })
  expect(bodyOf(fetcher, 1)).toEqual({ identifier: 'me@example.com', purpose: 'reset_password' })
  expect(fetcher.mock.calls[0][1].credentials).toBe('omit')
})

it('tells an existing account apart from one that still has to register', async () => {
  vi.stubGlobal('fetch', respond(200, { access_token: 'a', refresh_token: 'r', is_new_user: false, user: { uid: 'u', username: 'Alice' } }))
  const existing = await loginWithCode('me@example.com', '123456')
  expect(needsRegistration(existing)).toBe(false)

  const fetcher = respond(200, { registration_required: true, registration_token: 'tok', expires_in: 900 })
  vi.stubGlobal('fetch', fetcher)
  const fresh = await loginWithCode('new@example.com', '123456')
  expect(needsRegistration(fresh)).toBe(true)
  expect(bodyOf(fetcher)).toMatchObject({ identifier: 'new@example.com', code: '123456', client_id: 'xiaban-web' })
})

it('registers with the token, username and password', async () => {
  const fetcher = respond(200, { access_token: 'a', refresh_token: 'r', is_new_user: true, user: { uid: 'u', username: 'Alice_01' } })
  vi.stubGlobal('fetch', fetcher)
  await register('tok', 'Alice_01', 'pass-word-123')
  expect(fetcher.mock.calls[0][0]).toBe('https://account.xiangban.xyz/api/auth/register')
  expect(bodyOf(fetcher)).toEqual({
    registration_token: 'tok', username: 'Alice_01', password: 'pass-word-123', client_id: 'xiaban-web', device_name: '相伴网页',
  })
})

it('logs in with a password and resets one with a code', async () => {
  const fetcher = respond(200, { access_token: 'a', refresh_token: 'r', is_new_user: false, user: { uid: 'u', username: 'Alice' } })
  vi.stubGlobal('fetch', fetcher)
  await loginWithPassword('alice', 'pass-word-123')
  expect(fetcher.mock.calls[0][0]).toBe('https://account.xiangban.xyz/api/auth/password/login')
  expect(bodyOf(fetcher)).toMatchObject({ identifier: 'alice', password: 'pass-word-123', client_id: 'xiaban-web' })

  const reset = respond(200, { ok: true })
  vi.stubGlobal('fetch', reset)
  await resetPassword('me@example.com', '123456', 'brand-new-pass-9')
  expect(bodyOf(reset)).toEqual({ identifier: 'me@example.com', code: '123456', new_password: 'brand-new-pass-9' })
})

it('checks username availability with the name URL-encoded', async () => {
  const fetcher = respond(200, { available: false, message: '这个用户名已被占用' })
  vi.stubGlobal('fetch', fetcher)
  await expect(checkUsername('a b&c')).resolves.toEqual({ available: false, message: '这个用户名已被占用' })
  expect(fetcher.mock.calls[0][0]).toBe('https://account.xiangban.xyz/api/auth/username/available?username=a%20b%26c')
  expect(fetcher.mock.calls[0][1].method).toBe('GET')
})

it('surfaces the account center error code and message', async () => {
  vi.stubGlobal('fetch', respond(409, { error: true, code: 'username_taken', message: '这个用户名已被占用，换一个吧' }))
  await expect(register('tok', 'Taken_Name', 'pass-word-123')).rejects.toMatchObject({ status: 409, code: 'username_taken', message: '这个用户名已被占用，换一个吧' })
})

it('reports a network failure in plain words', async () => {
  vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')))
  await expect(sendLoginCode('me@example.com')).rejects.toMatchObject({ status: 0, code: 'network' })
})

it('drops the account center session without failing the login when logout errors', async () => {
  const fetcher = vi.fn().mockRejectedValue(new TypeError('offline'))
  vi.stubGlobal('fetch', fetcher)
  expect(() => dropAccountSession('refresh')).not.toThrow()
  await Promise.resolve()
  expect(bodyOf(fetcher)).toEqual({ refresh_token: 'refresh' })
})
