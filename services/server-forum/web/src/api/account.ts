/** 相伴统一账号中心的浏览器端客户端。凭证只走 Authorization 头，不用 Cookie。 */

import { siteConfig } from './forum'

export interface AccountUser {
  uid: string
  /** 唯一且不可修改的用户名，也是论坛里的展示名。 */
  username: string
  avatar_url: string | null
  identities: { kind: 'email' | 'phone'; masked: string }[]
}

export interface TokenPair {
  access_token: string
  refresh_token: string
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

async function call<T>(method: string, path: string, body?: unknown, token?: string): Promise<T> {
  const cfg = await siteConfig()
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  if (token) headers.Authorization = `Bearer ${token}`
  let res: Response
  try {
    res = await fetch(`${cfg.account_base}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  } catch {
    throw new AccountError(0, '连不上账号服务，请检查网络')
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
    call<{ cooldown_seconds: number; expires_in: number }>('POST', '/api/auth/code/send', { identifier, purpose }),
  async loginWithCode(identifier: string, code: string) {
    const { client_id } = await siteConfig()
    return call<CodeLoginResponse>('POST', '/api/auth/code/login', {
      identifier,
      code,
      client_id,
      device_name: '超级小伴网页',
    })
  },
  async loginWithPassword(identifier: string, password: string) {
    const { client_id } = await siteConfig()
    return call<LoginResponse>('POST', '/api/auth/password/login', { identifier, password, client_id, device_name: '超级小伴网页' })
  },
  async register(registrationToken: string, username: string, password: string) {
    const { client_id } = await siteConfig()
    return call<LoginResponse>('POST', '/api/auth/register', {
      registration_token: registrationToken,
      username,
      password,
      client_id,
      device_name: '超级小伴网页',
    })
  },
  resetPassword: (identifier: string, code: string, newPassword: string) =>
    call<{ ok: true }>('POST', '/api/auth/password/reset', { identifier, code, new_password: newPassword }),
  checkUsername: (username: string) =>
    call<{ available: boolean; message: string | null }>('GET', `/api/auth/username/available?username=${encodeURIComponent(username)}`),
  async refresh(refreshToken: string) {
    const { client_id } = await siteConfig()
    return call<TokenPair>('POST', '/api/auth/refresh', { refresh_token: refreshToken, client_id })
  },
  logout: (refreshToken: string) => call<unknown>('POST', '/api/auth/logout', { refresh_token: refreshToken }),
}
