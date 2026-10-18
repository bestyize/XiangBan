import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { AccountError, accountApi, needsRegistration, type AccountUser, type LoginResponse, type TokenPair } from '@/account/api'

const STORAGE_KEY = 'xiaoban.account.session'
/** 访问令牌在到期前这么多毫秒就换新，避免请求途中过期。 */
const REFRESH_MARGIN_MS = 60_000

interface Session {
  accessToken: string
  refreshToken: string
  expiresAt: number
  user: AccountUser
}

function readSession(): Session | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    const parsed = raw ? (JSON.parse(raw) as Session) : null
    return parsed?.accessToken && parsed.refreshToken && parsed.user ? parsed : null
  } catch {
    return null
  }
}

function writeSession(session: Session | null) {
  try {
    if (session) localStorage.setItem(STORAGE_KEY, JSON.stringify(session))
    else localStorage.removeItem(STORAGE_KEY)
  } catch {
    // 隐私模式等场景下存储不可用，登录只在本次页面内有效
  }
}

export const useAccount = defineStore('account', () => {
  const session = ref<Session | null>(readSession())
  const loginOpen = ref(false)

  const user = computed(() => session.value?.user ?? null)
  const loggedIn = computed(() => session.value !== null)
  const email = computed(() => user.value?.identities.find(i => i.kind === 'email')?.masked ?? null)

  function save(next: Session | null) {
    session.value = next
    writeSession(next)
  }

  function withTokens(tokens: TokenPair, user: AccountUser): Session {
    return {
      accessToken: tokens.access_token,
      refreshToken: tokens.refresh_token,
      expiresAt: Date.now() + tokens.expires_in * 1000,
      user,
    }
  }

  let refreshing: Promise<string | null> | null = null

  /** 返回可用的访问令牌；已过期则用刷新令牌换新，刷新失败视为已退出。 */
  async function accessToken(): Promise<string | null> {
    const current = session.value
    if (!current) return null
    if (current.expiresAt - REFRESH_MARGIN_MS > Date.now()) return current.accessToken
    // 刷新令牌只能用一次，并发请求必须共用同一次刷新。
    refreshing ??= accountApi
      .refresh(current.refreshToken)
      .then(tokens => {
        save(withTokens(tokens, current.user))
        return tokens.access_token
      })
      .catch(error => {
        if (error instanceof AccountError && error.status !== 0) save(null)
        return null
      })
      .finally(() => {
        refreshing = null
      })
    return refreshing
  }

  async function sendCode(identifier: string, purpose: 'login' | 'reset_password' = 'login') {
    return accountApi.sendCode(identifier, purpose)
  }

  function signedIn(res: LoginResponse) {
    save(withTokens(res, res.user))
    return { isNewUser: res.is_new_user, user: res.user }
  }

  /** 已有账号直接登录；新的邮箱 / 手机号返回注册凭证，由调用方收集用户名和密码后调用 register。 */
  async function loginWithCode(identifier: string, code: string) {
    const res = await accountApi.loginWithCode(identifier, code)
    if (needsRegistration(res)) return { registrationToken: res.registration_token } as const
    return signedIn(res)
  }

  async function loginWithPassword(identifier: string, password: string) {
    return signedIn(await accountApi.loginWithPassword(identifier, password))
  }

  async function register(registrationToken: string, username: string, password: string) {
    return signedIn(await accountApi.register(registrationToken, username, password))
  }

  const resetPassword = accountApi.resetPassword
  const checkUsername = accountApi.checkUsername
  const signupMethods = accountApi.signupMethods

  /** 页面打开时校验登录状态并同步最新资料。 */
  async function restore() {
    const token = await accessToken()
    if (!token) return
    try {
      const fresh = await accountApi.me(token)
      if (session.value) save({ ...session.value, user: fresh })
    } catch (error) {
      if (error instanceof AccountError && (error.status === 401 || error.status === 403)) save(null)
    }
  }

  async function logout() {
    const current = session.value
    save(null)
    if (current) await accountApi.logout(current.refreshToken).catch(() => undefined)
  }

  function openLogin() {
    loginOpen.value = true
  }

  function closeLogin() {
    loginOpen.value = false
  }

  return {
    user, loggedIn, email, loginOpen, accessToken, sendCode, loginWithCode, loginWithPassword, register,
    resetPassword, checkUsername, signupMethods,
    restore, logout, openLogin, closeLogin,
  }
})
