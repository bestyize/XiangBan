import { ApiError } from './client.ts'

/** 统一账号中心（services/server-account）的浏览器端调用：只发验证码并换取令牌，不保存账号中心会话。 */
const CLIENT_ID = 'xiaban-web'
const BASE = ((import.meta.env.VITE_ACCOUNT_URL as string | undefined) ?? 'https://account.xiangban.xyz').replace(/\/+$/, '')

export interface AccountCenterLogin {
  access_token: string
  refresh_token: string
  is_new_user: boolean
}

async function call<T>(path: string, body: unknown): Promise<T> {
  let response: Response
  try {
    response = await fetch(`${BASE}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      credentials: 'omit',
    })
  } catch {
    throw new ApiError(0, 'network', '连不上账号中心，检查一下网络再试')
  }
  const data: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    const error = (data ?? {}) as { code?: string; message?: string }
    throw new ApiError(response.status, error.code ?? 'http_error', error.message ?? '账号中心走神了，稍后再试')
  }
  return data as T
}

export function sendLoginCode(email: string): Promise<unknown> {
  return call('/api/auth/code/send', { identifier: email, purpose: 'login' })
}

export function loginWithCode(email: string, code: string, deviceName?: string): Promise<AccountCenterLogin> {
  return call('/api/auth/code/login', { identifier: email, code, client_id: CLIENT_ID, device_name: deviceName })
}

/** 换票完成后立刻作废账号中心会话；本站自己的令牌接管登录状态。失败也不影响登录。 */
export function dropAccountSession(refreshToken: string): void {
  void call('/api/auth/logout', { refresh_token: refreshToken }).catch(() => undefined)
}
