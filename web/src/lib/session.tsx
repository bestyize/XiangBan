import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react'
import { api, getToken, onUnauthorized, setToken } from '../api/client.ts'
import type { Features, Profile, ProfileUpdate, SessionResponse } from '../api/types.ts'
import { inApp, on, send, waitForAuth } from './bridge.ts'
import { browserTimezone, getDeviceId, setDeviceId } from './device.ts'
import { captureReferral } from './env.ts'
import { storage } from './storage.ts'

type Status = 'loading' | 'ready' | 'error'

const GUEST_PROFILE_KEY = 'xb_guest_profile'

function loadGuestProfile(base: Profile): Profile {
  const raw = storage.get(GUEST_PROFILE_KEY)
  if (!raw) return base
  try {
    const saved = JSON.parse(raw) as Partial<Profile>
    return { ...base, ...saved, userId: 0, accountType: 'guest', email: null }
  } catch {
    return base
  }
}

interface SessionValue {
  status: Status
  isLoggedIn: boolean
  error: string | null
  profile: Profile | null
  features: Features
  retry: () => void
  refresh: () => Promise<void>
  updateProfile: (patch: ProfileUpdate) => Promise<Profile>
  /** After binding an email, possibly into another account: adopt its token and profile. */
  adopt: (token: string, profile: Profile) => void
  setProfile: (profile: Profile) => void
}

const SessionContext = createContext<SessionValue | null>(null)

type SessionResult = SessionResponse & { epoch: number }
let inflight: Promise<SessionResult> | null = null
let epoch = 0
let activeUserId: number | null = null
let appAuthChecked = false

/**
 * Inside the 相伴 App the first session waits briefly for the App's token (bridge `auth`), so the page
 * opens as the App's account instead of creating an anonymous one.
 */
async function appToken(): Promise<void> {
  if (appAuthChecked || !inApp()) return
  appAuthChecked = true
  send('ready')
  const token = await waitForAuth(1500)
  if (token) setToken(token)
}

/** One request per page load even when React mounts effects twice. */
function openSession(): Promise<SessionResult> {
  // The session endpoint never answers 401, so skip the re-auth hook (it would wait on itself).
  if (inflight) return inflight
  const generation = epoch
  const request: Promise<SessionResult> = appToken().then(() => api<SessionResponse>(
    '/api/session',
    { method: 'POST', body: { deviceId: getDeviceId(), timezone: browserTimezone(), ...captureReferral() } },
    true,
  )).then((res) => {
    if (generation === epoch) {
      if (res.token) setToken(res.token)
      else setToken(null)
      setDeviceId(res.deviceId)
      activeUserId = res.profile.userId
    }
    const profile = res.token ? res.profile : loadGuestProfile(res.profile)
    return { ...res, profile, epoch: generation }
  }).finally(() => {
    if (inflight === request) inflight = null
  })
  inflight = request
  return request
}

export function SessionProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<Status>('loading')
  const [error, setError] = useState<string | null>(null)
  const [profile, setProfile] = useState<Profile | null>(null)
  const [features, setFeatures] = useState<Features>({ goHome: false, appDownloadUrl: null })
  const [attempt, setAttempt] = useState(0)

  useEffect(() => {
    let alive = true
    const apply = (res: SessionResult) => {
      if (!alive || res.epoch !== epoch) return
      setProfile(res.profile)
      setFeatures(res.features)
    }
    onUnauthorized(async () => {
      const previousId = activeUserId
      epoch += 1
      inflight = null
      setToken(null)
      const renewed = await openSession()
      apply(renewed)
      return renewed.profile.userId === previousId
    })
    // The App may also hand over a token later, e.g. after the user logs in natively.
    const offAuth = on('auth', (p) => {
      if (typeof p.token !== 'string') return
      epoch += 1
      inflight = null
      setToken(p.token)
      openSession().then(apply).catch(() => undefined)
    })
    openSession()
      .then((res) => {
        apply(res)
        if (alive) setStatus('ready')
      })
      .catch((e: Error) => {
        if (!alive) return
        setError(e.message)
        setStatus('error')
      })
    return () => {
      alive = false
      offAuth()
      onUnauthorized(null)
    }
  }, [attempt])

  const retry = useCallback(() => {
    setStatus('loading')
    setError(null)
    setAttempt((n) => n + 1)
  }, [])

  const updateProfile = useCallback(async (patch: ProfileUpdate) => {
    if (!getToken()) {
      const current = profile ?? {
        companionName: '小伴', personality: 'toxic', nickname: null, cityCode: null, cityName: null,
        timezone: browserTimezone() ?? 'Asia/Shanghai', workStart: '09:00', workEnd: '18:00', payday: null,
        showCityOnCard: false, inviteCode: '', email: null, recallOptIn: false, termsVersion: null,
        onboarded: false, createdAt: Date.now(), userId: 0, accountType: 'guest', avatarUrl: null,
      }
      const next: Profile = {
        ...current,
        companionName: patch.companionName ?? current.companionName,
        personality: (patch.personality as any) ?? current.personality,
        nickname: patch.nickname !== undefined ? (patch.nickname || null) : current.nickname,
        cityCode: patch.cityCode !== undefined ? (patch.cityCode || null) : current.cityCode,
        workStart: patch.workStart ?? current.workStart,
        workEnd: patch.workEnd ?? current.workEnd,
        payday: patch.payday !== undefined ? (patch.payday || null) : current.payday,
        showCityOnCard: patch.showCityOnCard ?? current.showCityOnCard,
        onboarded: patch.onboarded ?? current.onboarded,
        termsVersion: patch.termsVersion ?? current.termsVersion,
      }
      storage.set(GUEST_PROFILE_KEY, JSON.stringify(next))
      setProfile(next)
      return next
    }
    const next = await api<Profile>('/api/profile', { method: 'PUT', body: patch })
    setProfile(next)
    return next
  }, [profile])

  const adopt = useCallback((token: string, next: Profile) => {
    epoch += 1
    inflight = null
    activeUserId = next.userId
    setToken(token)
    setProfile(next)
    storage.remove(GUEST_PROFILE_KEY)
  }, [])

  const refresh = useCallback(async () => {
    const result = await openSession()
    if (result.epoch !== epoch) return
    setProfile(result.profile)
    setFeatures(result.features)
  }, [])
  useEffect(() => {
    const check = () => { if (document.visibilityState === 'visible') void refresh().catch(() => undefined) }
    window.addEventListener('focus', check)
    document.addEventListener('visibilitychange', check)
    return () => { window.removeEventListener('focus', check); document.removeEventListener('visibilitychange', check) }
  }, [refresh])

  const value = useMemo(
    () => ({ status, isLoggedIn: status === 'ready' && profile?.accountType === 'registered', error, profile, features, retry, refresh, updateProfile, adopt, setProfile }),
    [status, error, profile, features, retry, updateProfile, adopt],
  )
  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>
}

export function useSession(): SessionValue {
  const value = useContext(SessionContext)
  if (!value) throw new Error('useSession must be used inside SessionProvider')
  return value
}

/** For screens that only render once the session is ready. */
export function useProfile(): Profile {
  const { profile } = useSession()
  if (!profile) throw new Error('profile not loaded')
  return profile
}
