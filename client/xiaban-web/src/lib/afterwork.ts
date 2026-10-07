import { storage } from './storage.ts'

export const AFTERWORK_ACTIVITIES = [
  { id: 'rest', label: '早点休息', emoji: '🌙' },
  { id: 'walk', label: '散步透气', emoji: '🚶' },
  { id: 'food', label: '吃点好的', emoji: '🍜' },
  { id: 'move', label: '动一动', emoji: '✨' },
  { id: 'entertainment', label: '看剧或玩游戏', emoji: '🎮' },
  { id: 'company', label: '陪陪家人朋友', emoji: '💛' },
  { id: 'no_plan', label: '今天不安排', emoji: '☁️' },
] as const

export type AfterworkActivityId = (typeof AFTERWORK_ACTIVITIES)[number]['id']

export interface AfterworkEntry {
  workDay: string
  activityId: AfterworkActivityId
  selectedAt: number
}

export interface AfterworkData {
  entries: AfterworkEntry[]
  preferredActivityId: AfterworkActivityId | null
  promptDismissedAt: Partial<Record<AfterworkActivityId, number>>
}

const STORAGE_KEY = 'xiaban.afterwork.v1'
const MAX_ENTRIES = 90

function emptyData(): AfterworkData {
  return { entries: [], preferredActivityId: null, promptDismissedAt: {} }
}

function isActivityId(value: unknown): value is AfterworkActivityId {
  return typeof value === 'string' && AFTERWORK_ACTIVITIES.some((activity) => activity.id === value)
}

function isWorkDay(value: unknown): value is string {
  return typeof value === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(value)
}

export function getAfterworkActivityLabel(activityId: AfterworkActivityId): string {
  return AFTERWORK_ACTIVITIES.find((activity) => activity.id === activityId)?.label ?? ''
}

export function loadAfterworkData(): AfterworkData {
  const raw = storage.get(STORAGE_KEY)
  if (!raw) return emptyData()

  try {
    const parsed: unknown = JSON.parse(raw)
    if (!parsed || typeof parsed !== 'object') return emptyData()
    const value = parsed as Record<string, unknown>
    const rawEntries: unknown[] = Array.isArray(value.entries) ? value.entries : []
    const entries: AfterworkEntry[] = []

    for (const item of rawEntries) {
      if (!item || typeof item !== 'object') continue
      const entry = item as Record<string, unknown>
      if (!isWorkDay(entry.workDay) || !isActivityId(entry.activityId) || typeof entry.selectedAt !== 'number') continue
      entries.push({ workDay: entry.workDay, activityId: entry.activityId, selectedAt: entry.selectedAt })
    }

    const promptDismissedAt: Partial<Record<AfterworkActivityId, number>> = {}
    if (value.promptDismissedAt && typeof value.promptDismissedAt === 'object') {
      const dismissed = value.promptDismissedAt as Record<string, unknown>
      for (const activity of AFTERWORK_ACTIVITIES) {
        const count = dismissed[activity.id]
        if (typeof count === 'number' && Number.isFinite(count)) promptDismissedAt[activity.id] = count
      }
    }

    return {
      entries: entries.sort((a, b) => a.workDay.localeCompare(b.workDay)).slice(-MAX_ENTRIES),
      preferredActivityId: isActivityId(value.preferredActivityId) && value.preferredActivityId !== 'no_plan'
        ? value.preferredActivityId
        : null,
      promptDismissedAt,
    }
  } catch {
    return emptyData()
  }
}

function saveData(data: AfterworkData): AfterworkData {
  storage.set(STORAGE_KEY, JSON.stringify(data))
  return data
}

export function recordAfterworkChoice(workDay: string, activityId: AfterworkActivityId): AfterworkData {
  const data = loadAfterworkData()
  const entries = data.entries.filter((entry) => entry.workDay !== workDay)
  entries.push({ workDay, activityId, selectedAt: Date.now() })
  return saveData({ ...data, entries: entries.sort((a, b) => a.workDay.localeCompare(b.workDay)).slice(-MAX_ENTRIES) })
}

export function setPreferredAfterworkActivity(activityId: AfterworkActivityId | null): AfterworkData {
  const data = loadAfterworkData()
  return saveData({
    ...data,
    preferredActivityId: activityId === 'no_plan' ? null : activityId,
  })
}

export function dismissAfterworkHabitPrompt(activityId: AfterworkActivityId, count: number): AfterworkData {
  const data = loadAfterworkData()
  return saveData({
    ...data,
    promptDismissedAt: { ...data.promptDismissedAt, [activityId]: count },
  })
}

export function clearAfterworkData(): AfterworkData {
  storage.remove(STORAGE_KEY)
  return emptyData()
}

function shiftDay(dayKey: string, delta: number): string {
  if (!isWorkDay(dayKey)) return dayKey
  const date = new Date(dayKey + 'T12:00:00.000Z')
  if (Number.isNaN(date.getTime())) return dayKey
  date.setUTCDate(date.getUTCDate() + delta)
  return date.toISOString().slice(0, 10)
}

function dayKeyInTimeZone(date: Date, timeZone: string): string {
  try {
    const parts = new Intl.DateTimeFormat('en', {
      timeZone,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    }).formatToParts(date)
    const part = (type: string) => parts.find((item) => item.type === type)?.value ?? ''
    return part('year') + '-' + part('month') + '-' + part('day')
  } catch {
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    return year + '-' + month + '-' + day
  }
}

export function getRecentAfterworkActivityCount(
  data: AfterworkData,
  activityId: AfterworkActivityId,
  endWorkDay: string,
  days = 14,
): number {
  if (!isWorkDay(endWorkDay)) return 0
  const firstWorkDay = shiftDay(endWorkDay, -(days - 1))
  return data.entries.filter((entry) =>
    entry.activityId === activityId && entry.workDay >= firstWorkDay && entry.workDay <= endWorkDay,
  ).length
}

export interface WeeklyAfterworkSummary {
  count: number
  topActivityId: AfterworkActivityId | null
  topActivityCount: number
}

export function getWeeklyAfterworkSummary(
  data: AfterworkData,
  timeZone: string,
  now = new Date(),
): WeeklyAfterworkSummary {
  const today = dayKeyInTimeZone(now, timeZone)
  const firstDay = shiftDay(today, -6)
  const entries = data.entries.filter((entry) => entry.workDay >= firstDay && entry.workDay <= today)
  const counts = new Map<AfterworkActivityId, number>()

  for (const entry of entries) counts.set(entry.activityId, (counts.get(entry.activityId) ?? 0) + 1)
  const top = [...counts.entries()].sort((a, b) => b[1] - a[1])[0]

  return {
    count: entries.length,
    topActivityId: top?.[0] ?? null,
    topActivityCount: top?.[1] ?? 0,
  }
}
