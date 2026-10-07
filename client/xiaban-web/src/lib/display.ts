export interface DisplayItem { id: string; visible: boolean; name: string }
export interface DisplaySettings { chatTabVisible: boolean; games: DisplayItem[]; quizzes: DisplayItem[] }
export const DISPLAY_STORAGE_KEY = 'xb_display_settings'
export const DEFAULT_DISPLAY_SETTINGS: DisplaySettings = { chatTabVisible: true, games: [], quizzes: [] }
const defaultIds = {
  games: ['falling-blocks', 'merge-2048', 'block-puzzle', 'water-sort', 'snake-arena', 'ink-blade', 'dusk-glider'],
  quizzes: ['mbti', 'mbanti', 'animal', 'afterhours'],
}

export function isDisplaySettings(value: unknown): value is DisplaySettings {
  if (!value || typeof value !== 'object') return false
  const data = value as Record<string, unknown>
  const items = (list: unknown) => Array.isArray(list) && list.length <= 100 && list.every(item =>
    item && typeof item.id === 'string' && /^[a-z0-9-]{1,64}$/.test(item.id) && typeof item.visible === 'boolean' && typeof item.name === 'string' && item.name.length <= 80,
  ) && new Set(list.map(item => item.id)).size === list.length
  return typeof data.chatTabVisible === 'boolean' && items(data.games) && items(data.quizzes)
}

/** Unknown/newly shipped catalog items keep their original order after configured items. */
export function visibleDisplayItems<T extends { id: string }>(catalog: readonly T[], settings: readonly DisplayItem[]): T[] {
  const known = new Map(catalog.map(item => [item.id, item]))
  const seen = new Set<string>(), ordered: T[] = []
  for (const item of settings) {
    const entry = known.get(item.id)
    if (!entry || seen.has(item.id)) continue
    seen.add(item.id)
    if (item.visible) ordered.push(entry)
  }
  return [...ordered, ...catalog.filter(item => !seen.has(item.id))]
}

export function moveDisplayItem(items: DisplayItem[], from: number, to: number): DisplayItem[] {
  if (!Number.isInteger(from) || !Number.isInteger(to) || from < 0 || to < 0 || from >= items.length || to >= items.length || from === to) return items
  const next = items.slice(), [item] = next.splice(from, 1)
  next.splice(to, 0, item)
  return next
}

export function defaultDisplaySettings(settings: DisplaySettings): DisplaySettings {
  const reset = (key: 'games' | 'quizzes') => settings[key].map(item => ({ ...item, visible: true })).sort((a, b) => {
    const rank = (id: string) => { const index = defaultIds[key].indexOf(id); return index < 0 ? Number.MAX_SAFE_INTEGER : index }
    return rank(a.id) - rank(b.id)
  })
  return { chatTabVisible: true, games: reset('games'), quizzes: reset('quizzes') }
}
