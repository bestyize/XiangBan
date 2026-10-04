import type { Checkin } from '../api/types.ts'
import { storage } from './storage.ts'

/** Guests have no server account, so their check-in for the day is kept in this browser only. */
const KEY = 'xb_guest_checkin'

export function loadGuestCheckin(workDay: string): Checkin | null {
  const raw = storage.get(KEY)
  if (!raw) return null
  try {
    const checkin = JSON.parse(raw) as Checkin
    return checkin.workDay === workDay ? checkin : null
  } catch {
    return null
  }
}

export function saveGuestCheckin(checkin: Checkin) {
  storage.set(KEY, JSON.stringify(checkin))
}

export function clearGuestCheckin() {
  storage.remove(KEY)
}
