import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { EXPRESSION_NAMES, toSpec } from '../robot/expressions.ts'
import { resultShareLink } from './common/share.ts'
import { opensGameDirectly, RESULT_GAMES, resultGame, sharePageGameId } from './index.ts'
import { merge2048 } from './merge-2048/index.ts'

const packed = (gameId: string) => ({ gameId, score: 900, lines: 25, maxTile: 2048, level: 6, steps: 40 })
const packedGames = RESULT_GAMES.filter(game => game.id !== 'snake-arena')

describe('game registry', () => {
  it('registers every shipped game pack exactly once', () => {
    const catalog = JSON.parse(readFileSync(new URL('../../public/game-packs/catalog.json', import.meta.url), 'utf8')).games as string[]
    expect(RESULT_GAMES.map(game => game.id).sort()).toEqual([...catalog].sort())
    expect(new Set(RESULT_GAMES.map(game => game.id)).size).toBe(RESULT_GAMES.length)
  })

  it('round trips every packed game without leaking account fields', () => {
    for (const game of packedGames) {
      const value = packed(game.id)
      const parsed = game.parse({ ...value, token: 'secret', nonce: 'private' })!
      expect(parsed).toMatchObject(value)
      const url = new URL(resultShareLink(game, parsed, 'http://localhost:5173', 'friend-code'))
      expect(url.origin).toBe('https://xiaban.xyz')
      expect(url.pathname).toBe(`/games/${game.id}/share`)
      expect(url.searchParams.get('inv')).toBe('friend-code')
      expect(game.read(url.hash)).toMatchObject(value)
      expect(url.toString()).not.toMatch(/secret|private|own=/)
      expect(game.shareText(parsed)).toContain(game.name)
    }
  })

  it('gives every game a poster line and a real expression', () => {
    for (const game of packedGames) {
      const copy = game.poster(game.parse(packed(game.id))!)
      expect(copy.name).toBe(game.name)
      expect(copy.cheer.length).toBeGreaterThan(4)
      expect(EXPRESSION_NAMES).toContain(toSpec(copy.face).expression)
    }
    expect(merge2048.poster({ ...packed('merge-2048'), maxTile: 256 }).cheer).toContain('256')
  })

  it('only exposes known share routes and direct game entries', () => {
    expect(sharePageGameId('/games/merge-2048/share')).toBe('merge-2048')
    expect(sharePageGameId('/games/snake-arena/share')).toBe('snake-arena')
    expect(sharePageGameId('/games/toString/share')).toBeNull()
    expect(sharePageGameId('/games/unknown/share')).toBeNull()
    expect(resultGame('toString')).toBeNull()
    expect(opensGameDirectly('/games/snake-arena', '')).toBe(true)
    expect(opensGameDirectly('/games/ink-blade', '')).toBe(false)
    expect(opensGameDirectly('/games/ink-blade', '?from=game_share')).toBe(true)
    expect(opensGameDirectly('/games/unknown', '?from=game_share')).toBe(false)
    expect(opensGameDirectly('/games/ink-blade/share', '?from=game_share')).toBe(false)
  })
})
