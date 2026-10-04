import { useCallback, useEffect, useState } from 'react'
import { api, getToken } from '../../api/client.ts'
import type { Checkin, Tip, TodayResponse, Unlock } from '../../api/types.ts'
import { send } from '../../lib/bridge.ts'
import { clearGuestCheckin, loadGuestCheckin, saveGuestCheckin } from '../../lib/guestCheckin.ts'
import { browserTimezone, getDeviceId } from '../../lib/device.ts'

type Status = 'loading' | 'ready' | 'error'

/** The server holds this request until the model rewrite lands, capped at 3 s (PRD 5.4). */
const POLISH_WAIT_MS = 3000

export function useToday(active = true) {
  const [status, setStatus] = useState<Status>('loading')
  const [today, setToday] = useState<TodayResponse | null>(null)
  /** null while the practical info is still on its way */
  const [tips, setTips] = useState<Tip[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  /** True only right after tapping the button, so the stamp animation does not replay on reload. */
  const [fresh, setFresh] = useState(false)
  /** Expressions the last check-in unlocked, shown once */
  const [unlocked, setUnlocked] = useState<Unlock[]>([])
  const dismissUnlocked = useCallback(() => setUnlocked([]), [])

  /** Second request after check-in: tips (weather may be slow) and, if pending, the model rewrite. */
  const fillIn = useCallback(async (checkin: Checkin) => {
    try {
      const wait = checkin.reply.pending ? `?wait=${POLISH_WAIT_MS}` : ''
      const next = await api<TodayResponse>(`/api/checkin/today${wait}`)
      if (next.checkin?.id !== checkin.id) return
      setToday((prev) => (prev?.checkin?.id === checkin.id ? next : prev))
      setTips(next.tips)
    } catch {
      setTips([])
    }
  }, [])

  const load = useCallback(async () => {
    try {
      const tz = browserTimezone() ?? 'Asia/Shanghai'
      const next = await api<TodayResponse>(`/api/checkin/today?timezone=${encodeURIComponent(tz)}`)
      const localCheckin = getToken() ? null : loadGuestCheckin(next.workDay)
      if (localCheckin) {
        setToday({ ...next, checkin: localCheckin, greeting: null })
        setTips(next.tips)
        setStatus('ready')
        return
      }
      setToday(next)
      setError(null)
      setTips(next.tips)
      setStatus('ready')
      if (next.checkin?.reply.pending) void fillIn(next.checkin)
    } catch (e) {
      setError((e as Error).message)
      setStatus((previous) => previous === 'ready' ? previous : 'error')
    }
  }, [fillIn])

  useEffect(() => {
    if (active) void load()
  }, [active, load])

  const checkIn = useCallback(async () => {
    setBusy(true)
    setError(null)
    try {
      const tz = browserTimezone() ?? 'Asia/Shanghai'
      // deviceId lets a guest check-in count once in the admin metrics without storing anything server-side.
      const params = new URLSearchParams({ timezone: tz })
      if (!getToken()) params.set('deviceId', getDeviceId())
      const checkin = await api<Checkin>(`/api/checkin?${params}`, { method: 'POST' })
      if (!getToken()) {
        saveGuestCheckin(checkin)
      }
      setFresh(true)
      setTips(null)
      setToday({ workDay: checkin.workDay, checkin, greeting: null, tips: [] })
      // Inside the App the native cat reacts with the same face and line.
      send('checkin_done', { title: checkin.title.code, titleName: checkin.title.name, mood: checkin.mood, text: checkin.reply.text })
      // Let the title stamp land before the unlock celebration covers it.
      if (checkin.newUnlocks.length > 0) setTimeout(() => setUnlocked(checkin.newUnlocks), 900)
      if (checkin.reply.pending) void fillIn(checkin)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }, [fillIn])

  const undo = useCallback(async () => {
    setBusy(true)
    setError(null)
    try {
      if (!getToken()) {
        clearGuestCheckin()
        const tz = browserTimezone() ?? 'Asia/Shanghai'
        setToday(await api<TodayResponse>(`/api/checkin/today?timezone=${encodeURIComponent(tz)}`))
      } else {
        setToday(await api<TodayResponse>('/api/checkin/today', { method: 'DELETE' }))
      }
      setFresh(false)
    } catch (e) {
      setError((e as Error).message)
      await load()
    } finally {
      setBusy(false)
    }
  }, [load])

  return { status, today, tips, unlocked, dismissUnlocked, error, busy, fresh, checkIn, undo, reload: load }
}
