import { describe, expect, it } from 'vitest'
import { canShareOtherResult, otherChallengePath, otherResultFragment, otherResultPath, otherShareLink, parseOtherResult, readOtherResult, RESULT_GAMES, sharePageGameId } from './otherResultShare.ts'

const result = { gameId: 'merge-2048', score: 2340, lines: 0, maxTile: 512, level: 0, steps: 0 }
describe('all game result sharing', () => {
  it('rejects in-progress, mismatched-game and stale results at the host', () => {
    const round = { finished: true, score: result.score, lines: result.lines, maxTile: result.maxTile }
    expect(canShareOtherResult(result, result.gameId, round)).toBe(true)
    expect(canShareOtherResult(result, 'falling-blocks', round)).toBe(false)
    expect(canShareOtherResult(result, result.gameId, { ...round, finished: false })).toBe(false)
    expect(canShareOtherResult(result, result.gameId, { ...round, score: 999 })).toBe(false)
    expect(canShareOtherResult(result, result.gameId, { ...round, maxTile: 256 })).toBe(false)
    expect(canShareOtherResult(result, result.gameId, { ...round, lines: 1 })).toBe(false)
    expect(canShareOtherResult(result, result.gameId, null)).toBe(false)
  })
  it('round trips all supported games without leaking account fields', () => {
    for (const gameId of Object.keys(RESULT_GAMES)) {
      const value = { ...result, gameId }
      const parsed = parseOtherResult({ ...value, token: 'secret', nonce: 'private' })!
      expect(parsed).toEqual(value)
      const url = new URL(otherShareLink(parsed, 'http://localhost:5173', 'friend-code'))
      expect(url.origin).toBe('https://xiaban.xyz')
      expect(url.pathname).toBe(`/games/${gameId}/share`)
      expect(url.searchParams.get('from')).toBe('game_share')
      expect(url.searchParams.get('inv')).toBe('friend-code')
      expect(readOtherResult(gameId, url.hash)).toEqual(value)
      expect(url.toString()).not.toMatch(/secret|private|own=/)
    }
  })
  it('rejects malformed, unknown and ambiguous results', () => {
    for (const change of [{ gameId: 'unknown' }, { gameId: 'toString' }, { gameId: 'snake-arena' }, { score: -1 }, { score: 1.1 }, { lines: Infinity }, { maxTile: '512' }, { steps: 1e8 }, { level: NaN }]) expect(parseOtherResult({ ...result, ...change })).toBeNull()
    const fragment = otherResultFragment(result)
    for (const hash of ['', fragment + '&s=1', fragment + '&token=secret', fragment.replace('v=1', 'v=2'), fragment.replace('s=2340', 's=1e3')]) expect(readOtherResult(result.gameId, hash)).toBeNull()
  })
  it('only exposes known share routes and preserves public referrals on challenge links', () => {
    expect(sharePageGameId('/games/merge-2048/share')).toBe('merge-2048')
    expect(sharePageGameId('/games/snake-arena/share')).toBe('snake-arena')
    expect(sharePageGameId('/games/toString/share')).toBeNull()
    expect(sharePageGameId('/games/unknown/share')).toBeNull()
    expect(otherResultPath(result)).toContain('?own=1#')
    expect(otherChallengePath(result.gameId, 'friend-code')).toBe('/games/merge-2048?from=game_share&inv=friend-code')
    expect(otherChallengePath(result.gameId, 'private@email')).not.toContain('inv=')
  })
})
