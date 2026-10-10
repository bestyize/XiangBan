import { createContext, useContext, useEffect, useLayoutEffect, useState, type ReactNode } from 'react'
import { storage } from './storage.ts'

export type ThemePreference = 'system' | 'light' | 'dark'
const THEME_KEY = 'xb_theme'
export const THEME_OPTIONS: { value: ThemePreference; label: string }[] = [
  { value: 'system', label: '跟随系统' },
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
]

function readPreference(): ThemePreference {
  const saved = storage.get(THEME_KEY)
  return saved === 'light' || saved === 'dark' ? saved : 'system'
}

const ThemeContext = createContext<{
  preference: ThemePreference
  resolved: 'light' | 'dark'
  setPreference: (preference: ThemePreference) => void
}>({ preference: 'system', resolved: 'light', setPreference: () => {} })

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [preference, setPreference] = useState(readPreference)
  const [systemDark, setSystemDark] = useState(() => window.matchMedia('(prefers-color-scheme: dark)').matches)
  const resolved = preference === 'system' ? (systemDark ? 'dark' : 'light') : preference

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const changed = () => setSystemDark(media.matches)
    const stored = (event: StorageEvent) => {
      if (event.key === THEME_KEY || event.key === null) setPreference(readPreference())
    }
    changed()
    media.addEventListener('change', changed)
    window.addEventListener('storage', stored)
    return () => {
      media.removeEventListener('change', changed)
      window.removeEventListener('storage', stored)
    }
  }, [])

  useLayoutEffect(() => {
    document.documentElement.dataset.theme = resolved
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', resolved === 'dark' ? '#141414' : '#f7f7f7')
  }, [resolved])

  return <ThemeContext.Provider value={{ preference, resolved, setPreference: next => {
    storage.set(THEME_KEY, next)
    setPreference(next)
  } }}>{children}</ThemeContext.Provider>
}

export function useTheme() { return useContext(ThemeContext) }
