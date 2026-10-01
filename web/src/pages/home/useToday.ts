import { useCallback, useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { Checkin, TodayResponse } from '../../api/types.ts'

type Status = 'loading' | 'ready' | 'error'

export function useToday() {
  const [status, setStatus] = useState<Status>('loading')
  const [today, setToday] = useState<TodayResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  /** True only right after tapping the button, so the stamp animation does not replay on reload. */
  const [fresh, setFresh] = useState(false)

  const load = useCallback(async () => {
    try {
      setToday(await api<TodayResponse>('/api/checkin/today'))
      setStatus('ready')
    } catch (e) {
      setError((e as Error).message)
      setStatus('error')
    }
  }, [])

  useEffect(() => {
    void load()
  }, [load])

  const checkIn = useCallback(async () => {
    setBusy(true)
    setError(null)
    try {
      const checkin = await api<Checkin>('/api/checkin', { method: 'POST' })
      setFresh(true)
      setToday({ workDay: checkin.workDay, checkin })
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }, [])

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
