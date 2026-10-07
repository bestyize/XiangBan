import { api, getToken } from '../api/client.ts'
import { storage } from '../lib/storage.ts'

export interface GameSummary { gameId: string; sessions: number; bestScore: number; bestTile: number }

/** Per-game records: from the account when logged in, otherwise from this browser. */
export async function loadGameSummaries(): Promise<GameSummary[]> {
  if (getToken()) return api<GameSummary[]>('/api/games/summary')
  try {
    const records = JSON.parse(storage.get('xb_guest_games') || '{}') as Record<string, Omit<GameSummary, 'gameId'>>
    return Object.entries(records).map(([gameId, val]) => ({ gameId, sessions: val.sessions, bestScore: val.bestScore, bestTile: val.bestTile }))
  } catch {
    return []
  }
}
