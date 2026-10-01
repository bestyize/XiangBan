import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react'
import { api, onUnauthorized, setToken } from '../api/client.ts'
import type { Profile, ProfileUpdate, SessionResponse } from '../api/types.ts'
import { browserTimezone, getDeviceId, setDeviceId } from './device.ts'

type Status = 'loading' | 'ready' | 'error'

interface SessionValue {
  status: Status
  error: string | null
  profile: Profile | null
  retry: () => void
  updateProfile: (patch: ProfileUpdate) => Promise<Profile>
}

const SessionContext = createContext<SessionValue | null>(null)

let inflight: Promise<SessionResponse> | null = null

/** One request per page load even when React mounts effects twice. */
function openSession(): Promise<SessionResponse> {
  // The session endpoint never answers 401, so skip the re-auth hook (it would wait on itself).
  inflight ??= api<SessionResponse>(
    '/api/session',
    { method: 'POST', body: { deviceId: getDeviceId(), timezone: browserTimezone() } },
    true,
  ).then((res) => {
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
  const [attempt, setAttempt] = useState(0)

  useEffect(() => {
    let alive = true
    onUnauthorized(async () => {
      setToken(null)
      const res = await openSession()
      if (alive) setProfile(res.profile)
    })
    openSession()
      .then((res) => {
        if (!alive) return
        setProfile(res.profile)
        setStatus('ready')
      })
      .catch((e: Error) => {
        if (!alive) return
        setError(e.message)
        setStatus('error')
      })
    return () => {
      alive = false
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

  const value = useMemo(
    () => ({ status, error, profile, retry, updateProfile }),
    [status, error, profile, retry, updateProfile],
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
