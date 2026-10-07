import { describe, expect, it } from 'vitest'
import { canShareFinishedRound, gameChallengePath, gamePublicBase, gameShareLink, localResultPath, parseSnakeResult, readSharedResult, resultFragment, type SnakeResult } from './resultShare.ts'

const result: SnakeResult = { score: 1250, mass: 160, kills: 3, seconds: 245, roundType: 'endless', difficulty: 'lively' }

describe('snake result sharing', () => {
  it('round trips both game modes using only aggregate data in the fragment', () => {
    for (const roundType of ['timed', 'endless'] as const) {
      const data = { ...result, roundType }
      const url = new URL(gameShareLink(data, 'https://xiaban.xyz', 'friend-code'))
      expect(url.pathname).toBe('/games/snake-arena/share')
      expect(url.searchParams.get('from')).toBe('game_share')
      expect(url.searchParams.get('inv')).toBe('friend-code')
      expect(url.searchParams.has('score')).toBe(false)
      expect(readSharedResult(url.hash)).toEqual(data)
    }
  })

  it('drops account fields and host nonce from parsed results and links', () => {
    const parsed = parseSnakeResult({ ...result, email: 'private@example.com', token: 'secret-token', nonce: 'host-nonce' })!
    expect(parsed).toEqual(result)
    const url = gameShareLink(parsed, 'https://xiaban.xyz')
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

  it.each(['', '#v=2', '#v=1&s=1', '#' + resultFragment(result) + '&s=99', '#' + resultFragment(result) + '&token=secret', '#' + resultFragment(result).replace('s=1250', 's=1e3')])('rejects incomplete or ambiguous shared links %s', (fragment) => {
    expect(readSharedResult(fragment)).toBeNull()
  })

  it.each(['http://localhost:5173', 'http://127.0.0.1:4173', 'http://192.168.1.2', 'http://10.0.0.1', 'http://172.16.1.2', 'file:///assets', 'javascript:alert(1)', 'invalid'])('uses the public site instead of unusable preview base %s', (base) => {
    expect(gamePublicBase(base)).toBe('https://xiaban.xyz')
  })

  it('respects a public deployment origin and never uses URL credentials', () => {
    expect(gamePublicBase('https://play.example.com/subpath')).toBe('https://play.example.com')
    expect(gamePublicBase('https://user:password@example.com')).toBe('https://xiaban.xyz')
  })

  it('allows host sharing only for the completed current round with its exact score', () => {
    expect(canShareFinishedRound(result, { finished: true, score: 1250 })).toBe(true)
    expect(canShareFinishedRound(result, { finished: false, score: 1250 })).toBe(false)
    expect(canShareFinishedRound(result, { finished: true, score: 999 })).toBe(false)
    expect(canShareFinishedRound(null, { finished: true, score: 1250 })).toBe(false)
  })

  it('opens a local result page but makes challenge links retain the round type, difficulty and referral', () => {
    const local = localResultPath(result, 'friend-code')
    expect(local).toContain('?own=1&inv=friend-code#')
    expect(gameShareLink(result, 'https://xiaban.xyz')).not.toContain('own=')
    const challenge = new URL(gameChallengePath(result, 'friend-code'), 'https://xiaban.xyz')
    expect(challenge.searchParams.get('round')).toBe('endless')
    expect(challenge.searchParams.get('difficulty')).toBe('lively')
    expect(challenge.searchParams.get('inv')).toBe('friend-code')
    expect(gameShareLink(result, 'https://xiaban.xyz', 'email@example.com')).not.toContain('inv=')
  })
})
