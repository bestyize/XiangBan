import { lazy, type ComponentType } from 'react'

const KEY = 'xb_chunk_reload'
const WINDOW_MS = 10_000

/**
 * After a deploy, a page opened earlier still asks for the previous build's code files, which no longer
 * exist (the server answers with index.html). Reloading picks up the new build. Returns false when a reload
 * already happened moments ago, so a genuinely broken file can't trap the page in a reload loop.
 */
export function reloadForNewBuild(now = Date.now()): boolean {
  try {
    if (now - Number(sessionStorage.getItem(KEY) ?? 0) < WINDOW_MS) return false
    sessionStorage.setItem(KEY, String(now))
  } catch {
    return false
  }
  location.reload()
  return true
}

/** React.lazy that recovers from a missing chunk by reloading once instead of blanking the page. */
export function lazyPage<P extends object>(load: () => Promise<{ default: ComponentType<P> }>) {
  return lazy(() => load().catch((error: unknown) => {
    // Keep Suspense showing its fallback while the page reloads.
    if (reloadForNewBuild()) return new Promise<{ default: ComponentType<P> }>(() => {})
    throw error
  }))
}
