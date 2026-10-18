import { ApiError } from './client.ts'

/** 统一账号中心（services/server-account）的浏览器端调用：本站只用它验证身份，再把令牌交给主后端换票，不长期保存账号中心会话。 */
const CLIENT_ID = 'xiaban-web'
const BASE = ((import.meta.env.VITE_ACCOUNT_URL as string | undefined) ?? 'https://account.xiangban.xyz').replace(/\/+$/, '')
const DEVICE_NAME = '相伴网页'

export interface AccountSession {
  access_token: string
  refresh_token: string
  is_new_user: boolean
  user: { uid: string; username: string }
}

/** 验证码通过但还没有账号：带着这张凭证去设置用户名和密码。 */
export interface RegistrationRequired {
  registration_required: true
  registration_token: string
  expires_in: number
}

export type CodeLoginResult = AccountSession | RegistrationRequired

export function needsRegistration(result: CodeLoginResult): result is RegistrationRequired {
  return 'registration_required' in result && result.registration_required === true
}

async function call<T>(method: 'GET' | 'POST', path: string, body?: unknown): Promise<T> {
  let response: Response
  try {
    response = await fetch(`${BASE}${path}`, {
      method,
      ...(body === undefined ? {} : { headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }),
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

export const sendLoginCode = (identifier: string) => call('POST', '/api/auth/code/send', { identifier, purpose: 'login' })

export const sendResetCode = (identifier: string) => call('POST', '/api/auth/code/send', { identifier, purpose: 'reset_password' })

export const loginWithCode = (identifier: string, code: string) =>
  call<CodeLoginResult>('POST', '/api/auth/code/login', { identifier, code, client_id: CLIENT_ID, device_name: DEVICE_NAME })

export const loginWithPassword = (identifier: string, password: string) =>
  call<AccountSession>('POST', '/api/auth/password/login', { identifier, password, client_id: CLIENT_ID, device_name: DEVICE_NAME })

export const register = (registrationToken: string, username: string, password: string) =>
  call<AccountSession>('POST', '/api/auth/register', {
    registration_token: registrationToken,
    username,
    password,
    client_id: CLIENT_ID,
    device_name: DEVICE_NAME,
  })

export const resetPassword = (identifier: string, code: string, newPassword: string) =>
  call('POST', '/api/auth/password/reset', { identifier, code, new_password: newPassword })

export interface SignupMethods {
  email: boolean
  phone: boolean
}

/** 账号中心当前开放哪些注册方式；取不到就按只开邮箱处理。 */
export const getSignupMethods = (): Promise<SignupMethods> =>
  call<SignupMethods>('GET', '/api/auth/methods').catch(() => ({ email: true, phone: false }))

export const checkUsername = (username: string) =>
  call<{ available: boolean; message: string | null }>('GET', `/api/auth/username/available?username=${encodeURIComponent(username)}`)

/** 换票完成后立刻作废账号中心会话；本站自己的令牌接管登录状态。失败也不影响登录。 */
export function dropAccountSession(refreshToken: string): void {
  void call('POST', '/api/auth/logout', { refresh_token: refreshToken }).catch(() => undefined)
}
