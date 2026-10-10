import { afterEach, expect, it, vi } from 'vitest'
import { dropAccountSession, loginWithCode, sendLoginCode } from './accountCenter.ts'

afterEach(() => vi.unstubAllGlobals())

function respond(status: number, body: unknown) {
  return vi.fn().mockResolvedValue({ ok: status < 400, status, json: async () => body })
}

it('asks the account center to send a login code for the typed email', async () => {
  const fetcher = respond(200, { ok: true })
  vi.stubGlobal('fetch', fetcher)
  await sendLoginCode('me@example.com')
  const [url, init] = fetcher.mock.calls[0]
  expect(url).toBe('https://account.xiangban.xyz/api/auth/code/send')
  expect(JSON.parse(init.body)).toEqual({ identifier: 'me@example.com', purpose: 'login' })
  expect(init.credentials).toBe('omit')
})

it('logs in as the xiaban-web client', async () => {
  const fetcher = respond(200, { access_token: 'a', refresh_token: 'r', is_new_user: true })
  vi.stubGlobal('fetch', fetcher)
  await expect(loginWithCode('me@example.com', '123456')).resolves.toMatchObject({ access_token: 'a', refresh_token: 'r' })
  expect(JSON.parse(fetcher.mock.calls[0][1].body)).toMatchObject({ identifier: 'me@example.com', code: '123456', client_id: 'xiaban-web' })
})

it('surfaces the account center error code and message', async () => {
  vi.stubGlobal('fetch', respond(400, { error: true, code: 'code_wrong', message: '验证码不对' }))
  await expect(loginWithCode('me@example.com', '000000')).rejects.toMatchObject({ status: 400, code: 'code_wrong', message: '验证码不对' })
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
  expect(JSON.parse(fetcher.mock.calls[0][1].body)).toEqual({ refresh_token: 'refresh' })
})
