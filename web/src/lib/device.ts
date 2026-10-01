import { storage } from './storage.ts'

const DEVICE_KEY = 'xb_device'

function randomId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(24))
  return btoa(String.fromCharCode(...bytes)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

export function getDeviceId(): string {
  let id = storage.get(DEVICE_KEY)
  if (!id) {
    id = randomId()
    storage.set(DEVICE_KEY, id)
  }
  return id
}

export function setDeviceId(id: string) {
  storage.set(DEVICE_KEY, id)
}

export function browserTimezone(): string | undefined {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone
  } catch {
    return undefined
  }
}
