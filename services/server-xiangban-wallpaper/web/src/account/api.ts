/** 相伴统一账号中心（server-account）的浏览器端客户端。凭证只走 Authorization 头，不用 Cookie。 */

export const CLIENT_ID = 'xiangban-wallpaper-web'
const FALLBACK_ACCOUNT_URL = 'https://account.xiangban.xyz'

export interface AccountIdentity {
  kind: 'email' | 'phone'
  masked: string
}

export interface AccountUser {
  uid: string
  /** 唯一且不可修改的用户名，也是对外展示的名字。 */
  username: string
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

/** 验证码通过但还没有账号：带着这张凭证去设置用户名和密码。 */
export interface RegistrationRequired {
  registration_required: true
  registration_token: string
  expires_in: number
}

export type CodeLoginResponse = LoginResponse | RegistrationRequired

export function needsRegistration(result: CodeLoginResponse): result is RegistrationRequired {
  return 'registration_required' in result && result.registration_required === true
}

export class AccountError extends Error {
  constructor(
    public status: number,
    message: string,
    public retryAfter?: number,
    /** 账号中心的机读错误码，如 username_taken、registration_expired、identity_exists。 */
    public code?: string,
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
    throw new AccountError(res.status, data?.message ?? `请求失败（${res.status}）`, retry, typeof data?.code === 'string' ? data.code : undefined)
  }
  return data as T
}

export const accountApi = {
  sendCode: (identifier: string, purpose: 'login' | 'reset_password' = 'login') =>
    call<{ ok: true; cooldown_seconds: number; expires_in: number }>('POST', '/api/auth/code/send', { identifier, purpose }),
  loginWithCode: (identifier: string, code: string) =>
    call<CodeLoginResponse>('POST', '/api/auth/code/login', {
      identifier,
      code,
      client_id: CLIENT_ID,
      device_name: '小伴壁纸网站',
    }),
  loginWithPassword: (identifier: string, password: string) =>
    call<LoginResponse>('POST', '/api/auth/password/login', {
      identifier,
      password,
      client_id: CLIENT_ID,
      device_name: '小伴壁纸网站',
    }),
  register: (registrationToken: string, username: string, password: string) =>
    call<LoginResponse>('POST', '/api/auth/register', {
      registration_token: registrationToken,
      username,
      password,
      client_id: CLIENT_ID,
      device_name: '小伴壁纸网站',
    }),
  resetPassword: (identifier: string, code: string, newPassword: string) =>
    call<{ ok: true }>('POST', '/api/auth/password/reset', { identifier, code, new_password: newPassword }),
  checkUsername: (username: string) =>
    call<{ available: boolean; message: string | null }>('GET', `/api/auth/username/available?username=${encodeURIComponent(username)}`),
  refresh: (refreshToken: string) =>
    call<TokenPair>('POST', '/api/auth/refresh', { refresh_token: refreshToken, client_id: CLIENT_ID }),
  logout: (refreshToken: string) => call<{ ok: true }>('POST', '/api/auth/logout', { refresh_token: refreshToken }),
  me: (accessToken: string) => call<AccountUser>('GET', '/api/me', undefined, accessToken),
}
