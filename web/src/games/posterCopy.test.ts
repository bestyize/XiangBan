import { describe, expect, it } from 'vitest'
import { EXPRESSION_NAMES, toSpec } from '../robot/expressions.ts'
import { otherPosterCopy, snakePosterCopy } from './posterCopy.ts'
import { RESULT_GAMES } from './otherResultShare.ts'

const snake = { score: 1860, mass: 42, kills: 0, seconds: 95, roundType: 'timed', difficulty: 'cozy' } as const

describe('game poster copy', () => {
  it('lets 小伴 react to how the snake round went', () => {
    expect(snakePosterCopy(snake).face).toBe('excited')
    const hunter = snakePosterCopy({ ...snake, kills: 4 })
    expect(hunter.face).toBe('proud')
    expect(hunter.cheer).toContain('4')
    expect(snakePosterCopy({ ...snake, roundType: 'endless', seconds: 200 }).cheer).toContain('3:20')
    expect(snakePosterCopy(snake).stats.map(([, label]) => label)).toEqual(['最终长度', '淘汰 AI', '存活时长'])
  })

  it('has a line and a real expression for every other game', () => {
    for (const gameId of Object.keys(RESULT_GAMES)) {
      const copy = otherPosterCopy({ gameId, score: 900, lines: 25, maxTile: 2048, level: 6, steps: 40 })
      expect(copy.name).toBe(RESULT_GAMES[gameId])
      expect(copy.cheer.length).toBeGreaterThan(4)
      expect(EXPRESSION_NAMES).toContain(toSpec(copy.face).expression)
    }
    expect(otherPosterCopy({ gameId: 'merge-2048', score: 1, lines: 0, maxTile: 256, level: 0, steps: 0 }).cheer).toContain('256')
  })
})
