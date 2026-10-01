import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react'
import { api, onUnauthorized, setToken } from '../api/client.ts'
import type { Features, Profile, ProfileUpdate, SessionResponse } from '../api/types.ts'
import { inApp, on, send, waitForAuth } from './bridge.ts'
import { browserTimezone, getDeviceId, setDeviceId } from './device.ts'
import { captureReferral } from './env.ts'

type Status = 'loading' | 'ready' | 'error'

interface SessionValue {
  status: Status
  error: string | null
  profile: Profile | null
  features: Features
  retry: () => void
  updateProfile: (patch: ProfileUpdate) => Promise<Profile>
  /** After binding an email, possibly into another account: adopt its token and profile. */
  adopt: (token: string, profile: Profile) => void
  setProfile: (profile: Profile) => void
}

const SessionContext = createContext<SessionValue | null>(null)

let inflight: Promise<SessionResponse> | null = null
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
function openSession(): Promise<SessionResponse> {
  // The session endpoint never answers 401, so skip the re-auth hook (it would wait on itself).
  inflight ??= appToken().then(() => api<SessionResponse>(
    '/api/session',
    { method: 'POST', body: { deviceId: getDeviceId(), timezone: browserTimezone(), ...captureReferral() } },
    true,
  )).then((res) => {
    setToken(res.token)
    setDeviceId(res.deviceId)
    return res
  }).finally(() => {
    inflight = null
  })
  return inflight
}

export function SessionProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<Status>('loading')
  const [error, setError] = useState<string | null>(null)
  const [profile, setProfile] = useState<Profile | null>(null)
  const [features, setFeatures] = useState<Features>({ goHome: false, appDownloadUrl: null })
  const [attempt, setAttempt] = useState(0)

  useEffect(() => {
    let alive = true
    const apply = (res: SessionResponse) => {
      if (!alive) return
      setProfile(res.profile)
      setFeatures(res.features)
    }
    onUnauthorized(async () => {
      setToken(null)
      apply(await openSession())
    })
    // The App may also hand over a token later, e.g. after the user logs in natively.
    const offAuth = on('auth', (p) => {
      if (typeof p.token !== 'string') return
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
    }
  }, [attempt])

  const retry = useCallback(() => {
    setStatus('loading')
    setError(null)
    setAttempt((n) => n + 1)
  }, [])

  const updateProfile = useCallback(async (patch: ProfileUpdate) => {
    const next = await api<Profile>('/api/profile', { method: 'PUT', body: patch })
    setProfile(next)
    return next
  }, [])

  const adopt = useCallback((token: string, next: Profile) => {
    setToken(token)
    setProfile(next)
  }, [])

  const value = useMemo(
    () => ({ status, error, profile, features, retry, updateProfile, adopt, setProfile }),
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
