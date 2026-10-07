import { api, getToken } from '../api/client.ts'
import { getDeviceId } from './device.ts'

export type EventName =
  | 'page_view' | 'onboarding_step' | 'checkin' | 'reply_shown' | 'card_generate' | 'card_save' | 'card_share'
  | 'go_home_click' | 'game_start' | 'game_finish' | 'quiz_start' | 'quiz_complete'

interface QueuedEvent {
  name: EventName
  ts: number
  props: Record<string, string>
}

let queue: QueuedEvent[] = []
let timer: ReturnType<typeof setTimeout> | null = null

/**
 * Fire-and-forget usage events (PRD 10), batched every few seconds and flushed when the page hides.
 * Guests report too: the server counts them by a hash of the anonymous device id, never raw (admin metrics).
 */
export function track(name: EventName, props: Record<string, string> = {}) {
  queue.push({ name, ts: Date.now(), props })
  if (queue.length >= 20) void flush()
  else timer ??= setTimeout(() => void flush(), 5000)
}

export async function flush() {
  if (timer) clearTimeout(timer)
  timer = null
  if (queue.length === 0) return
  const events = queue
  queue = []
  try {
    // keepalive lets the request finish while the page is being hidden or closed
    await api('/api/events', { method: 'POST', body: { events, deviceId: getToken() ? undefined : getDeviceId() }, keepalive: true }, true)
  } catch {
    // Analytics never gets in the user's way; drop the batch.
  }
}

if (typeof document !== 'undefined') {
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') void flush()
  })
}
