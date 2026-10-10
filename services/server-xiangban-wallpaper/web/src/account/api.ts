/** 相伴统一账号中心（server-account）的浏览器端客户端。凭证只走 Authorization 头，不用 Cookie。 */

export const CLIENT_ID = 'xiangban-wallpaper-web'
const FALLBACK_ACCOUNT_URL = 'https://account.xiangban.xyz'

export interface AccountIdentity {
  kind: 'email' | 'phone'
  masked: string
}

export interface AccountUser {
  uid: string
  nickname: string
  avatar_url: string | null
  has_password: boolean
  identities: AccountIdentity[]
  created_at: string
}

export interface TokenPair {
  access_token: string
  refresh_token: string
  token_type: 'Bearer'
  expires_in: number
}

export interface LoginResponse extends TokenPair {
  is_new_user: boolean
  user: AccountUser
}

export class AccountError extends Error {
  constructor(
    public status: number,
    message: string,
    public retryAfter?: number,
  ) {
    super(message)
  }
}

let baseUrl: Promise<string> | null = null

/** 账号中心地址取自壁纸服务的 /auth/config（即令牌签发者），取不到时用正式域名。 */
function accountBase(): Promise<string> {
  baseUrl ??= fetch('/api/v1/auth/config')
    .then(res => (res.ok ? res.json() : null))
    .then(body => (typeof body?.issuer === 'string' && body.issuer ? body.issuer : FALLBACK_ACCOUNT_URL))
    .catch(() => FALLBACK_ACCOUNT_URL)
    .then(url => url.replace(/\/+$/, ''))
  return baseUrl
}

async function call<T>(method: string, path: string, body?: unknown, token?: string): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  if (token) headers.Authorization = `Bearer ${token}`

  let res: Response
  try {
    res = await fetch(`${await accountBase()}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  } catch {
    throw new AccountError(0, '无法连接账号服务，请检查网络')
  }
  const data = await res.json().catch(() => null)
  if (!res.ok) {
    const retry = typeof data?.retry_after === 'number' ? data.retry_after : undefined
    throw new AccountError(res.status, data?.message ?? `请求失败（${res.status}）`, retry)
  }
  return data as T
}

export const accountApi = {
  sendCode: (identifier: string) =>
    call<{ ok: true; cooldown_seconds: number; expires_in: number }>('POST', '/api/auth/code/send', {
      identifier,
      purpose: 'login',
    }),
  loginWithCode: (identifier: string, code: string) =>
    call<LoginResponse>('POST', '/api/auth/code/login', {
      identifier,
      code,
      client_id: CLIENT_ID,
      device_name: '小伴壁纸网站',
    }),
  refresh: (refreshToken: string) =>
    call<TokenPair>('POST', '/api/auth/refresh', { refresh_token: refreshToken, client_id: CLIENT_ID }),
  logout: (refreshToken: string) => call<{ ok: true }>('POST', '/api/auth/logout', { refresh_token: refreshToken }),
  me: (accessToken: string) => call<AccountUser>('GET', '/api/me', undefined, accessToken),
}
