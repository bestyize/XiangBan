/**
 * JS Bridge with the 相伴 App (PRD 8.2–8.3). The App's WebView injects `window.XiabanBridge.postMessage`
 * and calls `window.xiabanWeb.receive(json)` to talk back. Every message is
 * `{ "type": string, "payload": object, "version": 1 }`.
 *
 * App → web: auth { token }, env { platform, appVersion, safeAreaTop, safeAreaBottom }
 * Web → App: ready {}, checkin_done { title, titleName, mood, text }, save_image { base64, mime },
 *            share { base64?, mime?, text, url }, open_native { page }
 */
export const BRIDGE_VERSION = 1

export interface BridgeMessage<T = Record<string, unknown>> {
  type: string
  payload: T
  version: number
}

declare global {
  interface Window {
    XiabanBridge?: { postMessage: (message: string) => void }
    xiabanWeb?: { receive: (message: string) => void }
  }
}

type Handler = (payload: Record<string, unknown>) => void
const handlers = new Map<string, Set<Handler>>()
const buffered: BridgeMessage[] = []

export function inApp(): boolean {
  return typeof window !== 'undefined' && typeof window.XiabanBridge?.postMessage === 'function'
}

export function send(type: string, payload: Record<string, unknown> = {}) {
  if (!inApp()) return
  const message: BridgeMessage = { type, payload, version: BRIDGE_VERSION }
  window.XiabanBridge!.postMessage(JSON.stringify(message))
}

/** Subscribes to an App message; messages that arrived before anyone listened are replayed once. */
export function on(type: string, handler: Handler): () => void {
  const set = handlers.get(type) ?? new Set<Handler>()
  set.add(handler)
  handlers.set(type, set)
  for (const message of buffered.filter((m) => m.type === type)) handler(message.payload)
  return () => set.delete(handler)
}

export function receive(raw: string) {
  let message: BridgeMessage
  try {
    message = JSON.parse(raw) as BridgeMessage
  } catch {
    return
  }
  if (typeof message?.type !== 'string' || (message.version ?? 1) > BRIDGE_VERSION) return
  const payload = message.payload ?? {}
  const set = handlers.get(message.type)
  if (set && set.size > 0) set.forEach((h) => h(payload))
  else buffered.push({ ...message, payload })
}

/** Resolves with the App's token, or null when it does not arrive in time. */
export function waitForAuth(timeoutMs: number): Promise<string | null> {
  return new Promise((resolve) => {
    let settled = false
    let off: (() => void) | null = null
    const finish = (token: string | null) => {
      if (settled) return
      settled = true
      clearTimeout(timer)
      off?.()
      resolve(token)
    }
    const timer = setTimeout(() => finish(null), timeoutMs)
    // A buffered auth replays synchronously inside on(), before `off` is assigned.
    off = on('auth', (p) => finish(typeof p.token === 'string' ? p.token : null))
    if (settled) off()
  })
}

if (typeof window !== 'undefined') {
  window.xiabanWeb = { receive }
  on('env', (p) => {
    const root = document.documentElement.style
    if (typeof p.safeAreaTop === 'number') root.setProperty('--safe-top', `${p.safeAreaTop}px`)
    if (typeof p.safeAreaBottom === 'number') root.setProperty('--safe-bottom', `${p.safeAreaBottom}px`)
  })
}

/** Blob → base64 without the data: prefix, for save_image / share. */
export function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result).split(',')[1] ?? '')
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(blob)
  })
}
