import { describe, expect, it } from 'vitest'
import { challengePath, ownResultPath, resultShareLink } from '../common/share.ts'
import { snakeArena } from './index.ts'
import { parseSnakeResult, readSnakeResult, snakeFragment, type SnakeResult } from './share.ts'

const result: SnakeResult = { score: 1250, mass: 160, kills: 3, seconds: 245, roundType: 'endless', difficulty: 'lively' }

describe('snake result sharing', () => {
  it('round trips both game modes using only aggregate data in the fragment', () => {
    for (const roundType of ['timed', 'endless'] as const) {
      const data = { ...result, roundType }
      const url = new URL(resultShareLink(snakeArena, data, 'https://xiaban.xyz', 'friend-code'))
      expect(url.pathname).toBe('/games/snake-arena/share')
      expect(url.searchParams.get('from')).toBe('game_share')
      expect(url.searchParams.get('inv')).toBe('friend-code')
      expect(url.searchParams.has('score')).toBe(false)
      expect(readSnakeResult(url.hash)).toEqual(data)
    }
  })

  it('drops account fields and host nonce from parsed results and links', () => {
    const parsed = parseSnakeResult({ ...result, email: 'private@example.com', token: 'secret-token', nonce: 'host-nonce' })!
    expect(parsed).toEqual(result)
    const url = resultShareLink(snakeArena, parsed, 'https://xiaban.xyz')
    expect(url).not.toContain('private')
    expect(url).not.toContain('secret-token')
    expect(url).not.toContain('host-nonce')
  })

  it.each([
    { score: -1 }, { score: 1.5 }, { score: Infinity }, { score: 1e9 + 1 }, { score: '1250' },
    { mass: 0 }, { mass: 1001 }, { kills: -1 }, { seconds: NaN }, { roundType: 'unknown' }, { difficulty: 'unknown' }, { roundType: ['timed'] }, { difficulty: ['cozy'] },
  ])('rejects malformed or out-of-range payload %j', (change) => {
    expect(parseSnakeResult({ ...result, ...change })).toBeNull()
  })

  it.each(['', '#v=2', '#v=1&s=1', '#' + snakeFragment(result) + '&s=99', '#' + snakeFragment(result) + '&token=secret', '#' + snakeFragment(result).replace('s=1250', 's=1e3')])('rejects incomplete or ambiguous shared links %s', (fragment) => {
    expect(readSnakeResult(fragment)).toBeNull()
  })

  it('allows host sharing only for the completed current round with its exact score', () => {
    const round = { finished: true, score: 1250, lines: 0, maxTile: 0 }
    expect(snakeArena.canShare(result, round)).toBe(true)
    expect(snakeArena.canShare(result, { ...round, finished: false })).toBe(false)
    expect(snakeArena.canShare(result, { ...round, score: 999 })).toBe(false)
    expect(snakeArena.canShare(result, null)).toBe(false)
  })

  it('opens a local result page but makes challenge links retain the round type, difficulty and referral', () => {
    expect(ownResultPath(snakeArena, result, 'friend-code')).toContain('?own=1&inv=friend-code#')
    expect(resultShareLink(snakeArena, result, 'https://xiaban.xyz')).not.toContain('own=')
    const challenge = new URL(challengePath(snakeArena, result, 'friend-code'), 'https://xiaban.xyz')
    expect(challenge.pathname).toBe('/games/snake-arena')
    expect(challenge.searchParams.get('round')).toBe('endless')
    expect(challenge.searchParams.get('difficulty')).toBe('lively')
    expect(challenge.searchParams.get('inv')).toBe('friend-code')
    expect(resultShareLink(snakeArena, result, 'https://xiaban.xyz', 'email@example.com')).not.toContain('inv=')
  })

  it('forwards only supported challenge options to the pack', () => {
    expect(snakeArena.launchParams!(new URLSearchParams('round=endless&difficulty=lively'))).toEqual({ round: 'endless', difficulty: 'lively' })
    expect(snakeArena.launchParams!(new URLSearchParams('round=unknown&difficulty=unknown&hostNonce=x'))).toEqual({})
  })

  it('lets 小伴 react to how the round went', () => {
    const snake = { ...result, mass: 42, kills: 0, seconds: 95, roundType: 'timed', difficulty: 'cozy' } as const
    expect(snakeArena.poster(snake).face).toBe('excited')
    const hunter = snakeArena.poster({ ...snake, kills: 4 })
    expect(hunter.face).toBe('proud')
    expect(hunter.cheer).toContain('4')
    expect(snakeArena.poster({ ...snake, roundType: 'endless', seconds: 200 }).cheer).toContain('3:20')
    expect(snakeArena.poster(snake).stats.map(([, label]) => label)).toEqual(['最终长度', '淘汰 AI', '存活时长'])
    expect(snakeArena.subtitle(result)).toBe('无尽生存 · 热闹难度 · 1250 分')
  })
})
