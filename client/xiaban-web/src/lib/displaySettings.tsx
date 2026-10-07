import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { DEFAULT_DISPLAY_SETTINGS, DISPLAY_STORAGE_KEY, isDisplaySettings, type DisplaySettings } from './display.ts'
import { storage } from './storage.ts'

function cachedSettings(): DisplaySettings | null {
  try { const value = JSON.parse(storage.get(DISPLAY_STORAGE_KEY) || 'null'); return isDisplaySettings(value) ? value : null } catch { return null }
}

export function publishDisplaySettings(settings: DisplaySettings) {
  storage.set(DISPLAY_STORAGE_KEY, JSON.stringify(settings))
  window.dispatchEvent(new Event('xb-display-updated'))
}

const DisplayContext = createContext({ settings: DEFAULT_DISPLAY_SETTINGS, loading: true })

export function DisplaySettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState(() => cachedSettings() || DEFAULT_DISPLAY_SETTINGS)
  const [loading, setLoading] = useState(() => !cachedSettings())
  useEffect(() => {
    let alive = true
    let controller: AbortController | null = null
    async function refresh() {
      controller?.abort()
      const request = new AbortController(); controller = request
      const timeout = window.setTimeout(() => request.abort(), 4000)
      try {
        const response = await fetch('/api/display', { signal: request.signal, cache: 'no-store', credentials: 'same-origin' })
        if (!response.ok) throw new Error('Display settings unavailable')
        const value: unknown = await response.json()
        if (!isDisplaySettings(value)) throw new Error('Invalid display settings')
        if (alive && controller === request && !request.signal.aborted) { setSettings(value); storage.set(DISPLAY_STORAGE_KEY, JSON.stringify(value)) }
      } catch (error) {
        if (error instanceof Error && error.name === 'AbortError') return
        // Older backends / transient outages retain the last public settings or original catalog.
      } finally { window.clearTimeout(timeout); if (alive && controller === request) setLoading(false) }
    }
    const update = () => { const cached = cachedSettings(); if (cached) { setSettings(cached); setLoading(false) } void refresh() }
    const visibility = () => { if (!document.hidden) void refresh() }
    const stored = (event: StorageEvent) => { if (event.key === DISPLAY_STORAGE_KEY) update() }
    void refresh()
    window.addEventListener('xb-display-updated', update)
    window.addEventListener('storage', stored)
    window.addEventListener('focus', visibility)
    document.addEventListener('visibilitychange', visibility)
    return () => {
      alive = false; controller?.abort()
      window.removeEventListener('xb-display-updated', update); window.removeEventListener('storage', stored)
      window.removeEventListener('focus', visibility); document.removeEventListener('visibilitychange', visibility)
    }
  }, [])
  return <DisplayContext.Provider value={{ settings, loading }}>{children}</DisplayContext.Provider>
}

export function useDisplaySettings() { return useContext(DisplayContext) }
