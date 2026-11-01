import { computed, ref, watch } from 'vue'
import { accents, DEFAULT_ACCENT, validAccent, paletteFor, buttonInk, accentInk, type AccentId } from './accentPalette'
export { accents } from './accentPalette'

export type ThemePreference = 'system' | 'light' | 'dark'
export const THEME_LABEL: Record<ThemePreference, string> = { system: '自动', light: '日间', dark: '夜间' }
const STORAGE_KEY = 'forum-theme'
const ACCENT_KEY = 'forum-accent'
function readAccent(): AccentId {
  try { const value = localStorage.getItem(ACCENT_KEY); return validAccent(value) ? value : DEFAULT_ACCENT } catch { return DEFAULT_ACCENT }
}
const accent = ref<AccentId>(readAccent())
const appearanceOpen = ref(false)
const accentName = computed(() => accents.find(item => item.id === accent.value)!.name)
function openAppearance() { appearanceOpen.value = true }
function setAccent(value: AccentId) {
  if (!validAccent(value)) return
  accent.value = value
  try { localStorage.setItem(ACCENT_KEY, value) } catch {}
}

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
    if (event.key === ACCENT_KEY || event.key === null) accent.value = readAccent()
  })
}

export const resolved = computed<'light' | 'dark'>(() =>
  preference.value === 'system' ? (systemDark.value ? 'dark' : 'light') : preference.value,
)

watch([resolved, accent], ([value, selected]) => {
  if (typeof document === 'undefined') return
  document.documentElement.dataset.theme = value
  document.documentElement.dataset.accent = selected
  const p = paletteFor(selected, value === 'dark')
  for (const [name, color] of Object.entries({ brand:accentInk(p, value === 'dark'), 'brand-fill':p.btn, 'brand-hover':p.btnHi, 'brand-ink':buttonInk(p.btn) })) document.documentElement.style.setProperty(`--${name}`, color)
  document.querySelector('meta[name="theme-color"]')?.setAttribute('content', value === 'dark' ? '#3b3e45' : '#ededed')
}, { immediate: true })

export function setPreference(value: ThemePreference) {
  preference.value = value
  try { localStorage.setItem(STORAGE_KEY, value) } catch {}
}

export function cycleTheme() {
  setPreference(preference.value === 'system' ? 'light' : preference.value === 'light' ? 'dark' : 'system')
}

export function useTheme() {
  return { preference, resolved, setPreference, cycleTheme, accent, accentName, setAccent, appearanceOpen, openAppearance }
}

export const THEME_OPTIONS: { value: ThemePreference; label: string }[] = [
  { value: 'system', label: '自动跟随系统' },
  { value: 'light', label: '日间模式' },
  { value: 'dark', label: '夜间模式' },
]
