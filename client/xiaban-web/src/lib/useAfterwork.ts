import { useEffect, useState } from 'react'
import { api, getToken } from '../api/client.ts'
import { useProfile } from './session.tsx'
import { clearAfterworkData, loadAfterworkData, recordAfterworkChoice, setPreferredAfterworkActivity, type AfterworkData, type AfterworkActivityId } from './afterwork.ts'

const empty: AfterworkData = { entries: [], preferredActivityId: null, promptDismissedAt: {} }
export function useAfterwork() {
  const { userId } = useProfile()
  const [data, setData] = useState<AfterworkData>(() => (!getToken() ? loadAfterworkData() : empty))
  const [busy, setBusy] = useState(false)
  const [loading, setLoading] = useState(() => Boolean(getToken()))
  const [error, setError] = useState<string | null>(null)
  const [legacy, setLegacy] = useState(() => loadAfterworkData())
  useEffect(() => {
    if (!getToken()) {
      setData(loadAfterworkData())
      setLoading(false)
      return
    }
    const controller = new AbortController()
    setData(empty); setLoading(true); setError(null)
    api<AfterworkData>('/api/afterwork', { signal: controller.signal }).then(setData)
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [userId])
  async function mutate(path: string, method: 'PUT' | 'POST' | 'DELETE', body?: unknown) {
    if (!getToken()) {
      if (path === '/api/afterwork/choices' && body) {
        const { workDay, activityId } = body as { workDay: string; activityId: AfterworkActivityId }
        setData(recordAfterworkChoice(workDay, activityId))
      } else if (path === '/api/afterwork/preferences' && body) {
        const { preferredActivityId } = body as { preferredActivityId?: AfterworkActivityId | null }
        if (preferredActivityId !== undefined) {
          setData(setPreferredAfterworkActivity(preferredActivityId))
        }
      } else if (method === 'DELETE') {
        clearAfterworkData()
        setData(empty)
      }
      return true
    }
    setBusy(true); setError(null)
    try {
      if (method === 'DELETE') { await api(path, { method }); setData(empty) }
      else setData(await api<AfterworkData>(path, { method, body }))
      return true
    } catch (e) { setError((e as Error).message); return false } finally { setBusy(false) }
  }
  return { data, busy, loading, error, legacy,
    choose: (workDay: string, activityId: AfterworkActivityId) => mutate('/api/afterwork/choices', 'PUT', { workDay, activityId }),
    remember: (activityId: AfterworkActivityId | null) => mutate('/api/afterwork/preferences', 'PUT', { preferredActivityChanged: true, preferredActivityId: activityId }),
    dismiss: (activityId: AfterworkActivityId, count: number) => mutate('/api/afterwork/preferences', 'PUT', { dismissedActivityId: activityId, dismissedCount: count }),
    clear: () => mutate('/api/afterwork', 'DELETE'),
    importLegacy: async () => { if (await mutate('/api/afterwork/import', 'POST', legacy)) { clearAfterworkData(); setLegacy(empty) } },
  }
}
