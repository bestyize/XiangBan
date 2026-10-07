import { storage } from '../lib/storage.ts'

export class ApiError extends Error {
  readonly status: number
  readonly code: string

  constructor(status: number, code: string, message: string) {
    super(message)
    this.status = status
    this.code = code
  }
}

const TOKEN_KEY = 'xb_token'
let token: string | null = storage.get(TOKEN_KEY)
let reauthenticate: (() => Promise<boolean>) | null = null
export function onSessionRequired(_handler: (() => Promise<void>) | null) {}
export async function requireUserSession() {
  if (token) return
  throw new ApiError(401, 'session_required', '请先登录或注册账号')
}

export function setToken(value: string | null) {
  token = value
  if (value) storage.set(TOKEN_KEY, value)
  else storage.remove(TOKEN_KEY)
}

export function getToken() {
  return token
}

/** Called once when a request comes back 401, so an expired token is refreshed transparently. */
export function onUnauthorized(handler: (() => Promise<boolean>) | null) {
  reauthenticate = handler
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  body?: unknown
  signal?: AbortSignal
  keepalive?: boolean
}

export async function api<T>(path: string, options: RequestOptions = {}, retried = false): Promise<T> {
  const publicPath = path === '/api/session' || path === '/api/meta' || path === '/api/health'
    || path === '/api/account/email/status' || path === '/api/account/login/code' || path === '/api/account/login'
    || path.startsWith('/api/checkin') || path.startsWith('/api/events') || path.startsWith('/api/chat') || path.startsWith('/api/avatars/') || path.startsWith('/api/admin/')
  if (!token && !publicPath && options.method && options.method !== 'GET') await requireUserSession()
  const requestToken = token
  const headers: Record<string, string> = {}
  if (options.body !== undefined) headers['Content-Type'] = 'application/json'
  if (token) headers.Authorization = `Bearer ${token}`

  let response: Response
  try {
    response = await fetch(path, {
      method: options.method ?? 'GET',
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
      signal: options.signal,
      keepalive: options.keepalive,
      credentials: 'same-origin',
    })
  } catch (error) {
    if (error instanceof DOMException && error.name === 'AbortError') throw error
    throw new ApiError(0, 'network', '网络有点问题，检查一下再试')
  }

  if (response.status === 401 && !retried && reauthenticate) {
    // A late response from the previous account must never sign out the new account.
    if (requestToken !== token) {
      if (!options.method || options.method === 'GET') return api<T>(path, options, true)
      throw new ApiError(401, 'account_changed', '登录状态已变化，请重新确认后再操作')
    }
    const sameAccount = await reauthenticate()
    if (!sameAccount && options.method && options.method !== 'GET') throw new ApiError(401, 'account_changed', '登录状态已变化，请重新确认后再操作')
    return api<T>(path, options, true)
  }
  if (response.status === 204) return undefined as T

  const data: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    const body = (data ?? {}) as { error?: string; message?: string }
    throw new ApiError(response.status, body.error ?? 'http_error', body.message ?? '小伴走神了，稍后再试')
  }
  return data as T
}
