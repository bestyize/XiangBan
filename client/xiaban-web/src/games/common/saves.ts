import { api } from '../../api/client.ts'
import { storage } from '../../lib/storage.ts'

/** A game's own progress. The website stores it but never interprets it. */
export type GameSave = Record<string, unknown>

export const SAVE_MAX_BYTES = 32 * 1024
const LOCAL_KEY = 'xb_game_saves'

/** Accepts only a plain JSON object small enough to store; anything else from a pack is dropped. */
export function validSave(value: unknown): value is GameSave {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false
  try { return new TextEncoder().encode(JSON.stringify(value)).length <= SAVE_MAX_BYTES } catch { return false }
}

function localSaves(): Record<string, GameSave> {
  try {
    const parsed: unknown = JSON.parse(storage.get(LOCAL_KEY) || '{}')
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed as Record<string, GameSave> : {}
  } catch { return {} }
}

export function readLocalSave(gameId: string): GameSave | null {
  const save = localSaves()[gameId]
  return validSave(save) ? save : null
}

export function writeLocalSave(gameId: string, data: GameSave) {
  storage.set(LOCAL_KEY, JSON.stringify({ ...localSaves(), [gameId]: data }))
}

export function clearLocalSave(gameId: string) {
  const saves = localSaves()
  delete saves[gameId]
  storage.set(LOCAL_KEY, JSON.stringify(saves))
}

interface RemoteSave { data: GameSave | null; updatedAt: number | null }

/**
 * Guests keep progress in this browser; registered accounts keep it on the server. The first time an account
 * has no server save, progress made here as a guest is uploaded so logging in never loses it.
 */
export async function loadGameSave(gameId: string, registered: boolean): Promise<GameSave | null> {
  if (!registered) return readLocalSave(gameId)
  const remote = await api<RemoteSave>(`/api/games/saves/${encodeURIComponent(gameId)}`)
  if (remote.data && validSave(remote.data)) return remote.data
  const local = readLocalSave(gameId)
  if (local) {
    await storeGameSave(gameId, true, local)
    clearLocalSave(gameId)
  }
  return local
}

export async function storeGameSave(gameId: string, registered: boolean, data: GameSave, keepalive = false) {
  if (!registered) { writeLocalSave(gameId, data); return }
  await api(`/api/games/saves/${encodeURIComponent(gameId)}`, { method: 'PUT', keepalive, body: { data, updatedAt: Date.now() } })
}
