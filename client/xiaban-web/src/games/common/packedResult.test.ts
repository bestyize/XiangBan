import { describe, expect, it } from 'vitest'
import { merge2048 } from '../merge-2048/index.ts'
import { packedFragment, parsePackedResult, readPackedResult } from './packedResult.ts'
import { challengePath, gamePublicBase, ownResultPath, resultShareLink } from './share.ts'

const result = { gameId: 'merge-2048', score: 2340, lines: 0, maxTile: 512, level: 0, steps: 0 }

describe('packed game results', () => {
  it('rejects in-progress, mismatched-game and stale results at the host', () => {
    const round = { finished: true, score: result.score, lines: result.lines, maxTile: result.maxTile }
    expect(merge2048.canShare(result, round)).toBe(true)
    expect(merge2048.canShare({ ...result, gameId: 'falling-blocks' }, round)).toBe(false)
    expect(merge2048.canShare(result, { ...round, finished: false })).toBe(false)
    expect(merge2048.canShare(result, { ...round, score: 999 })).toBe(false)
    expect(merge2048.canShare(result, { ...round, maxTile: 256 })).toBe(false)
    expect(merge2048.canShare(result, { ...round, lines: 1 })).toBe(false)
    expect(merge2048.canShare(result, null)).toBe(false)
  })

  it('rejects malformed, other-game and ambiguous results', () => {
    for (const change of [{ gameId: 'unknown' }, { gameId: 'toString' }, { gameId: 'snake-arena' }, { score: -1 }, { score: 1.1 }, { lines: Infinity }, { maxTile: '512' }, { steps: 1e8 }, { level: NaN }]) {
      expect(merge2048.parse({ ...result, ...change })).toBeNull()
    }
    expect(parsePackedResult('merge-2048', null)).toBeNull()
    const fragment = packedFragment(result)
    for (const hash of ['', fragment + '&s=1', fragment + '&token=secret', fragment.replace('v=1', 'v=2'), fragment.replace('s=2340', 's=1e3')]) {
      expect(readPackedResult('merge-2048', hash)).toBeNull()
    }
  })

  it('opens the sharer page and challenge links with public referrals only', () => {
    expect(ownResultPath(merge2048, result)).toBe(`/games/merge-2048/share?own=1#${packedFragment(result)}`)
    expect(challengePath(merge2048, result, 'friend-code')).toBe('/games/merge-2048?from=game_share&inv=friend-code')
    expect(challengePath(merge2048, null, 'private@email')).toBe('/games/merge-2048?from=game_share')
    expect(new URL(resultShareLink(merge2048, result, 'https://xiaban.xyz')).searchParams.get('from')).toBe('game_share')
  })
})

describe('public share base', () => {
  it.each(['http://localhost:5173', 'http://127.0.0.1:4173', 'http://192.168.1.2', 'http://10.0.0.1', 'http://172.16.1.2', 'file:///assets', 'javascript:alert(1)', 'invalid'])('uses the public site instead of unusable preview base %s', (base) => {
    expect(gamePublicBase(base)).toBe('https://xiaban.xyz')
  })

  it('respects a public deployment origin and never uses URL credentials', () => {
    expect(gamePublicBase('https://play.example.com/subpath')).toBe('https://play.example.com')
    expect(gamePublicBase('https://user:password@example.com')).toBe('https://xiaban.xyz')
  })
})
