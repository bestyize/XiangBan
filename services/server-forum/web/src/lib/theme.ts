import { computed, ref, watch } from 'vue'

export type ThemePreference = 'system' | 'light' | 'dark'
export const THEME_LABEL: Record<ThemePreference, string> = { system: '自动', light: '日间', dark: '夜间' }
const STORAGE_KEY = 'forum-theme'

function readPreference(): ThemePreference {
  try {
    const value = localStorage.getItem(STORAGE_KEY)
    return value === 'light' || value === 'dark' ? value : 'system'
  } catch {
    return 'system'
  }
}

const preference = ref<ThemePreference>(readPreference())
const systemDark = ref(false)

if (typeof window !== 'undefined') {
  try { systemDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches } catch {}
  const media = window.matchMedia?.('(prefers-color-scheme: dark)')
  media?.addEventListener?.('change', event => { systemDark.value = event.matches })
  window.addEventListener('storage', event => {
    if (event.key === STORAGE_KEY || event.key === null) preference.value = readPreference()
  })
}

export const resolved = computed<'light' | 'dark'>(() =>
  preference.value === 'system' ? (systemDark.value ? 'dark' : 'light') : preference.value,
)

watch(resolved, value => {
  if (typeof document === 'undefined') return
  document.documentElement.dataset.theme = value
  document.querySelector('meta[name="theme-color"]')?.setAttribute('content', value === 'dark' ? '#201b1a' : '#f5f5f5')
}, { immediate: true })

export function setPreference(value: ThemePreference) {
  preference.value = value
  try { localStorage.setItem(STORAGE_KEY, value) } catch {}
}

export function cycleTheme() {
  setPreference(preference.value === 'system' ? 'light' : preference.value === 'light' ? 'dark' : 'system')
}

export function useTheme() {
  return { preference, resolved, setPreference, cycleTheme }
}

export const THEME_OPTIONS: { value: ThemePreference; label: string }[] = [
  { value: 'system', label: '自动跟随系统' },
  { value: 'light', label: '日间模式' },
  { value: 'dark', label: '夜间模式' },
]
