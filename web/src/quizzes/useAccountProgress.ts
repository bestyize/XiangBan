import { useEffect, useState } from 'react'
import { api } from '../api/client.ts'
import { useSession } from '../lib/session.tsx'
import { quizCatalog } from './catalog.ts'
import { applyRemoteAttempt, onAttemptSaved, quizAccountId, readSavedAttempt, type SavedAttempt } from './progress.ts'

interface CloudAttempt extends SavedAttempt { quizId: string }

/** One account-scoped sync worker; local saves survive failed requests and are retried on return. */
export function useAccountProgress(needed: boolean) {
  const { profile, status, isLoggedIn } = useSession()
  const owner = isLoggedIn ? profile?.userId ?? null : null
  const [engaged, setEngaged] = useState(needed)
  const [readyOwner, setReadyOwner] = useState<number | null>(null)
  const [error, setError] = useState('')
  const enabled = needed || engaged
  useEffect(() => { if (needed) setEngaged(true) }, [needed])
  useEffect(() => {
    if (!owner || !enabled) return
    const controller = new AbortController()
    const queue = new Map<string, CloudAttempt>()
    let timer: ReturnType<typeof setTimeout> | undefined
    let running = false
    const current = () => !controller.signal.aborted && quizAccountId() === owner
    const notify = () => window.dispatchEvent(new Event('xb-quiz-progress'))
    async function flush(keepalive = false) {
      if (!current() || running) return
      running = true
      try {
        while (queue.size && current()) {
          const [id, record] = queue.entries().next().value!
          const response = await api<CloudAttempt>('/api/quizzes/progress', { method: 'PUT', body: record, signal: controller.signal, keepalive }, true)
          if (!current()) return
          if (queue.get(id) === record) queue.delete(id)
          const quiz = quizCatalog.find((item) => item.id === response.quizId && item.version === response.version)
          const changed = quiz ? applyRemoteAttempt(quiz, response, owner!) : false
          setError(''); if (changed) notify()
        }
      } catch { if (current()) setError('进度已暂存此设备，账号同步暂未完成。联网后会重试。') }
      finally { running = false }
    }
    function schedule() { clearTimeout(timer); timer = setTimeout(() => { void flush() }, 500) }
    const off = onAttemptSaved((quiz, attempt, savedOwner) => {
      if (!current() || savedOwner !== owner) return
      queue.set(quiz.id, { ...attempt, quizId: quiz.id }); schedule()
    })
    async function restore() {
      try {
        const records = await api<CloudAttempt[]>('/api/quizzes/progress', { signal: controller.signal }, true)
        if (!current()) return
        for (const quiz of quizCatalog) {
          const remote = records.find((record) => record.quizId === quiz.id && record.version === quiz.version)
          if (remote) applyRemoteAttempt(quiz, remote, owner!)
          const local = readSavedAttempt(quiz, owner!)
          if (local && (!remote || local.updatedAt > remote.updatedAt)) queue.set(quiz.id, { ...local, quizId: quiz.id })
        }
        setError(''); notify(); await flush()
      } catch { if (current()) setError('暂时无法读取账号进度，可以先在此设备继续体验。') }
      finally { if (current()) setReadyOwner(owner) }
    }
    const retry = () => { if (current()) void restore() }
    const hide = () => { if (document.visibilityState === 'hidden') void flush(true) }
    void restore()
    window.addEventListener('online', retry)
    window.addEventListener('focus', retry)
    document.addEventListener('visibilitychange', hide)
    return () => { controller.abort(); clearTimeout(timer); off(); window.removeEventListener('online', retry); window.removeEventListener('focus', retry); document.removeEventListener('visibilitychange', hide) }
  }, [owner, enabled])
  return { pending: needed && ((status === 'loading' && quizAccountId() !== null) || (owner !== null && readyOwner !== owner)), error: owner ? error : '' }
}
