import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { AccountError, accountApi, needsRegistration, type LoginResponse, type TokenPair } from '@/api/account'
import { api, configureAuth } from '@/api/forum'
import type { SessionInfo } from '@/api/types'
import { clearInvite } from '@/lib/invite'
import { SessionScope } from '@/lib/sessionScope'

const STORAGE_KEY = 'chat.xiangban.session'
const desktopWindow = window as Window & { __xiaobanDesktopCommunity?: { ticket: string | null } }
const desktopBootstrap = desktopWindow.__xiaobanDesktopCommunity
delete desktopWindow.__xiaobanDesktopCommunity
const desktopStorageKey = `${STORAGE_KEY}.desktop`

function tokenStorage(): [Storage, string] {
  return desktopBootstrap ? [sessionStorage, desktopStorageKey] : [localStorage, STORAGE_KEY]
}

/** 访问令牌到期前这么久就换新，避免请求途中过期。 */
const REFRESH_MARGIN_MS = 60_000

interface Tokens {
  accessToken: string
  refreshToken: string
  expiresAt: number
}

function readTokens(): Tokens | null {
  try {
    const [storage, key] = tokenStorage()
    const raw = storage.getItem(key)
    const parsed = raw ? (JSON.parse(raw) as Tokens) : null
    return parsed && typeof parsed.accessToken === 'string' && parsed.accessToken && typeof parsed.refreshToken === 'string' && parsed.refreshToken
      ? { ...parsed, expiresAt:Number.isFinite(parsed.expiresAt) ? parsed.expiresAt : 0 } : null
  } catch {
    return null
  }
}

function writeTokens(tokens: Tokens | null) {
  try {
    const [storage, key] = tokenStorage()
    if (tokens) storage.setItem(key, JSON.stringify(tokens))
    else storage.removeItem(key)
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
  const loginReturnTo = ref('')
  const restoring = ref(false)
  const accountId = ref<string | null>(null)
  const scope = new SessionScope()
  let restoreSequence = 0

  const user = computed(() => info.value?.user ?? null)
  const loggedIn = computed(() => tokens.value !== null)
  const level = computed(() => user.value?.trust_level ?? -1)
  const canPost = computed(() => (info.value?.limits.daily_topics ?? 0) > 0 && user.value?.status === 'active')

  function save(next: Tokens | null) {
    tokens.value = next
    writeTokens(next)
    if (!next) info.value = null
  }

  function changeIdentity(next: Tokens | null) {
    scope.advance()
    restoring.value = false
    info.value = null
    accountId.value = null
    save(next)
  }

  async function accessToken(): Promise<string | null> {
    const current = tokens.value
    if (!current) return null
    if (current.expiresAt - REFRESH_MARGIN_MS > Date.now()) return current.accessToken
    // Refresh remains shared within one identity; an old completion cannot change a new login.
    return scope.refresh(async active => {
      const valid = () => active() && tokens.value?.refreshToken === current.refreshToken
      try {
        const pair = await accountApi.refresh(current.refreshToken)
        if (!valid()) return null
        save(fromPair(pair))
        return pair.access_token
      } catch (error) {
        if (valid() && error instanceof AccountError && error.status !== 0) changeIdentity(null)
        return null
      }
    })
  }

  configureAuth(accessToken, rejectedToken => {
    // A 401 from an earlier token must not expire a different account or a newer refresh result.
    if (rejectedToken && tokens.value?.accessToken === rejectedToken) tokens.value = { ...tokens.value, expiresAt: 0 }
  })

  async function loadSession() {
    if (!tokens.value) return
    const active = scope.profileRequest()
    try {
      const result = await api.session()
      if (!active()) return
      info.value = result
      clearInvite()
      const token = await accessToken()
      if (!token || !active()) return
      const profile = await accountApi.me(token).catch(() => null)
      if (active() && profile?.uid === result.user.uid) accountId.value = profile.account_id || null
    } catch (error) {
      if (active() && error instanceof Error && 'status' in error && (error as { status: number }).status === 401) changeIdentity(null)
    }
  }

  async function restore() {
    const generation = scope.capture(), request = ++restoreSequence
    restoring.value = true
    try {
      const ticket = desktopBootstrap?.ticket
      if (desktopBootstrap) desktopBootstrap.ticket = null
      if (ticket && !tokens.value) {
        const active = scope.loginAttempt()
        const pair = await accountApi.redeemDesktopTicket(ticket).catch(() => null)
        if (!active()) return
        if (pair) save(fromPair(pair))
      }
      await loadSession()
    } finally {
      if (scope.current(generation) && request === restoreSequence) restoring.value = false
    }
  }

  async function signedIn(res: LoginResponse, active:() => boolean) {
    if (!active()) throw new Error('登录操作已结束，请重新尝试。')
    changeIdentity(fromPair(res))
    const generation = scope.capture()
    await loadSession()
    if (!scope.current(generation)) throw new Error('登录操作已结束，请重新尝试。')
    return { isNewUser: res.is_new_user, username: res.user.username }
  }

  /** 已有账号直接登录；新的邮箱 / 手机号返回注册凭证，由调用方收集用户名和密码后调用 register。 */
  async function loginWithCode(identifier: string, code: string) {
    const active = scope.loginAttempt()
    const res = await accountApi.loginWithCode(identifier, code)
    if (!active()) throw new Error('登录操作已结束，请重新尝试。')
    if (needsRegistration(res)) return { registrationToken: res.registration_token } as const
    return signedIn(res, active)
  }

  async function loginWithPassword(identifier: string, password: string) {
    const active = scope.loginAttempt()
    return signedIn(await accountApi.loginWithPassword(identifier, password), active)
  }

  async function register(registrationToken: string, username: string, password: string) {
    const active = scope.loginAttempt()
    return signedIn(await accountApi.register(registrationToken, username, password), active)
  }

  async function logout() {
    const current = tokens.value
    changeIdentity(null)
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
    accountId,
    info,
    user,
    loggedIn,
    level,
    canPost,
    loginOpen,
    loginReturnTo,
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
