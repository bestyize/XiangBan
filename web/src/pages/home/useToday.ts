import { useCallback, useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { Checkin, TodayResponse } from '../../api/types.ts'

type Status = 'loading' | 'ready' | 'error'

/** The server holds this request until the model rewrite lands, capped at 3 s (PRD 5.4). */
const POLISH_WAIT_MS = 3000

export function useToday() {
  const [status, setStatus] = useState<Status>('loading')
  const [today, setToday] = useState<TodayResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  /** True only right after tapping the button, so the stamp animation does not replay on reload. */
  const [fresh, setFresh] = useState(false)

  const awaitPolish = useCallback(async (checkinId: string) => {
    try {
      const next = await api<TodayResponse>(`/api/checkin/today?wait=${POLISH_WAIT_MS}`)
      setToday((prev) => (prev?.checkin?.id === checkinId && next.checkin?.id === checkinId ? next : prev))
    } catch {
      // Keep showing the library line.
    }
  }, [])

  const load = useCallback(async () => {
    try {
      const next = await api<TodayResponse>('/api/checkin/today')
      setToday(next)
      setStatus('ready')
      if (next.checkin?.reply.pending) void awaitPolish(next.checkin.id)
    } catch (e) {
      setError((e as Error).message)
      setStatus('error')
    }
  }, [awaitPolish])

  useEffect(() => {
    void load()
  }, [load])

  const checkIn = useCallback(async () => {
    setBusy(true)
    setError(null)
    try {
      const checkin = await api<Checkin>('/api/checkin', { method: 'POST' })
      setFresh(true)
      setToday({ workDay: checkin.workDay, checkin, greeting: null })
      if (checkin.reply.pending) void awaitPolish(checkin.id)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }, [awaitPolish])

  const undo = useCallback(async () => {
    setBusy(true)
    setError(null)
    try {
      setToday(await api<TodayResponse>('/api/checkin/today', { method: 'DELETE' }))
      setFresh(false)
    } catch (e) {
      setError((e as Error).message)
      await load()
    } finally {
      setBusy(false)
    }
  }, [load])

  return { status, today, error, busy, fresh, checkIn, undo, reload: load }
}
