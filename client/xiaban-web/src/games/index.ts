import { blockPuzzle } from './block-puzzle/index.ts'
import type { ResultGame } from './common/types.ts'
import { fallingBlocks } from './falling-blocks/index.ts'
import { inkBlade } from './ink-blade/index.ts'
import { merge2048 } from './merge-2048/index.ts'
import { snakeArena } from './snake-arena/index.ts'
import { waterSort } from './water-sort/index.ts'

/** Every game whose results the website can share. Add a new game's descriptor here. */
export const RESULT_GAMES: readonly ResultGame[] = [fallingBlocks, merge2048, blockPuzzle, waterSort, snakeArena, inkBlade]

const byId = new Map(RESULT_GAMES.map(game => [game.id, game]))

export function resultGame(id: string): ResultGame | null { return byId.get(id) ?? null }

export function sharePageGameId(path: string): string | null {
  const match = /^\/games\/([a-z0-9-]+)\/share$/.exec(path)
  return match && byId.has(match[1]) ? match[1] : null
}

/** Game pages that skip onboarding: games marked `openDirectly`, and challenge links from a shared result. */
export function opensGameDirectly(path: string, search: string): boolean {
  const match = /^\/games\/([a-z0-9-]+)$/.exec(path)
  const game = match ? byId.get(match[1]) : undefined
  return Boolean(game && (game.openDirectly || ['game_share', 'game-share'].includes(new URLSearchParams(search).get('from') || '')))
}
