import { ref } from 'vue'

export type AdminTheme = 'light' | 'dark'
export type AdminAccent = 'red' | 'green' | 'blue' | 'purple' | 'amber' | 'graphite'

export const ADMIN_ACCENTS: { id: AdminAccent; label: string; color: string }[] = [
  { id: 'red', label: '小伴红', color: '#D93A44' },
  { id: 'green', label: '薄荷绿', color: '#2E8B5E' },
  { id: 'blue', label: '海湾蓝', color: '#2F6FDB' },
  { id: 'purple', label: '暮紫', color: '#7152D8' },
  { id: 'amber', label: '琥珀', color: '#B46B0E' },
  { id: 'graphite', label: '石墨', color: '#4A4A52' },
]

const THEME_KEY = 'xb-wallpaper-admin:theme'
const ACCENT_KEY = 'xb-wallpaper-admin:accent'

function readTheme(): AdminTheme {
  try {
    const v = localStorage.getItem(THEME_KEY)
    if (v === 'light' || v === 'dark') return v
  } catch { /* ignore */ }
  return 'light'
}

function readAccent(): AdminAccent {
  try {
    const v = localStorage.getItem(ACCENT_KEY) as AdminAccent | null
    if (v && ADMIN_ACCENTS.some((a) => a.id === v)) return v
  } catch { /* ignore */ }
  return 'red'
}

export const adminTheme = ref<AdminTheme>(readTheme())
export const adminAccent = ref<AdminAccent>(readAccent())

export function applyAdminRootAttrs(active: boolean) {
  const el = document.documentElement
  if (active) {
    el.dataset.adminTheme = adminTheme.value
    el.dataset.adminAccent = adminAccent.value
  } else {
    delete el.dataset.adminTheme
    delete el.dataset.adminAccent
  }
}

export function setAdminTheme(next: AdminTheme) {
  adminTheme.value = next
  try { localStorage.setItem(THEME_KEY, next) } catch { /* ignore */ }
  if (document.documentElement.dataset.adminTheme !== undefined) {
    document.documentElement.dataset.adminTheme = next
  }
}

export function toggleAdminTheme() {
  setAdminTheme(adminTheme.value === 'light' ? 'dark' : 'light')
}

export function setAdminAccent(next: AdminAccent) {
  adminAccent.value = next
  try { localStorage.setItem(ACCENT_KEY, next) } catch { /* ignore */ }
  if (document.documentElement.dataset.adminAccent !== undefined) {
    document.documentElement.dataset.adminAccent = next
  }
}
