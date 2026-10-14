import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { AccountError, accountApi, needsRegistration, type LoginResponse, type TokenPair } from '@/api/account'
import { api, configureAuth } from '@/api/forum'
import type { SessionInfo } from '@/api/types'
import { clearInvite } from '@/lib/invite'

const STORAGE_KEY = 'chat.xiangban.session'
/** 访问令牌到期前这么久就换新，避免请求途中过期。 */
const REFRESH_MARGIN_MS = 60_000

interface Tokens {
  accessToken: string
  refreshToken: string
  expiresAt: number
}

function readTokens(): Tokens | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    const parsed = raw ? (JSON.parse(raw) as Tokens) : null
    return parsed?.accessToken && parsed.refreshToken ? parsed : null
  } catch {
    return null
  }
}

function writeTokens(tokens: Tokens | null) {
  try {
    if (tokens) localStorage.setItem(STORAGE_KEY, JSON.stringify(tokens))
    else localStorage.removeItem(STORAGE_KEY)
  } catch {
    // 隐私模式下存储不可用，登录只在本页有效
  }
}

function fromPair(pair: TokenPair): Tokens {
  return { accessToken: pair.access_token, refreshToken: pair.refresh_token, expiresAt: Date.now() + pair.expires_in * 1000 }
}

export const useSession = defineStore('session', () => {
  const tokens = ref<Tokens | null>(readTokens())
  const info = ref<SessionInfo | null>(null)
  const loginOpen = ref(false)
  const restoring = ref(false)

  const user = computed(() => info.value?.user ?? null)
  const loggedIn = computed(() => tokens.value !== null)
  const level = computed(() => user.value?.trust_level ?? -1)
  const canPost = computed(() => (info.value?.limits.daily_topics ?? 0) > 0 && user.value?.status === 'active')

  function save(next: Tokens | null) {
    tokens.value = next
    writeTokens(next)
    if (!next) info.value = null
  }

  let refreshing: Promise<string | null> | null = null

  async function accessToken(): Promise<string | null> {
    const current = tokens.value
    if (!current) return null
    if (current.expiresAt - REFRESH_MARGIN_MS > Date.now()) return current.accessToken
    // 刷新令牌只能用一次，并发请求共用同一次刷新。
    refreshing ??= accountApi
      .refresh(current.refreshToken)
      .then(pair => {
        save(fromPair(pair))
        return pair.access_token
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

  configureAuth(accessToken, () => {
    // 令牌被服务端拒绝：强制下次刷新，刷新也失败就退出。
    if (tokens.value) tokens.value = { ...tokens.value, expiresAt: 0 }
  })

  async function loadSession() {
    if (!tokens.value) return
    try {
      info.value = await api.session()
      clearInvite()
    } catch (error) {
      if (error instanceof Error && 'status' in error && (error as { status: number }).status === 401) save(null)
    }
  }

  async function restore() {
    restoring.value = true
    try {
      await loadSession()
    } finally {
      restoring.value = false
    }
  }

  async function signedIn(res: LoginResponse) {
    save(fromPair(res))
    await loadSession()
    return { isNewUser: res.is_new_user, username: res.user.username }
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

  async function logout() {
    const current = tokens.value
    save(null)
    if (current) await accountApi.logout(current.refreshToken).catch(() => undefined)
  }

  /** 需要登录的操作先调用它：未登录就弹出登录框并返回 false。 */
  function requireLogin(): boolean {
    if (loggedIn.value) return true
    loginOpen.value = true
    return false
  }

  return {
    tokens,
    info,
    user,
    loggedIn,
    level,
    canPost,
    loginOpen,
    restoring,
    accessToken,
    restore,
    loadSession,
    loginWithCode,
    loginWithPassword,
    register,
    logout,
    requireLogin,
  }
})
