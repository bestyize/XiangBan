import type { RobotInput } from '../../robot/expressions.ts'
import type { ResultGame } from './types.ts'

/**
 * The result shape shared by `result-share.js` packs: five nonnegative integers whose meaning each game decides
 * (for example ink blade puts kills in `lines` and parries in `steps`).
 */
export interface PackedResult { gameId: string; score: number; lines: number; maxTile: number; level: number; steps: number }

const LIMITS = { score: 1e9, lines: 1e6, maxTile: 1073741824, level: 1e4, steps: 1e7 } as const
const KEYS = ['v', 's', 'l', 'm', 'n', 'p']

export function parsePackedResult(gameId: string, value: unknown): PackedResult | null {
  if (!value || typeof value !== 'object') return null
  const data = value as Record<string, unknown>
  if (data.gameId !== gameId) return null
  for (const [key, max] of Object.entries(LIMITS)) {
    if (typeof data[key] !== 'number' || !Number.isInteger(data[key]) || data[key] < 0 || data[key] > max) return null
  }
  return { gameId, score: data.score as number, lines: data.lines as number, maxTile: data.maxTile as number, level: data.level as number, steps: data.steps as number }
}

export function packedFragment(result: PackedResult): string {
  if (!parsePackedResult(result.gameId, result)) throw new Error('战绩不完整，请重新玩一局。')
  return new URLSearchParams({ v: '1', s: String(result.score), l: String(result.lines), m: String(result.maxTile), n: String(result.level), p: String(result.steps) }).toString()
}

export function readPackedResult(gameId: string, fragment: string): PackedResult | null {
  if (!fragment || fragment.length > 180) return null
  const query = new URLSearchParams(fragment.replace(/^#/, ''))
  if (query.size !== KEYS.length || KEYS.some(key => query.getAll(key).length !== 1) || query.get('v') !== '1' || KEYS.slice(1).some(key => !/^\d+$/.test(query.get(key) || ''))) return null
  return parsePackedResult(gameId, { gameId, score: Number(query.get('s')), lines: Number(query.get('l')), maxTile: Number(query.get('m')), level: Number(query.get('n')), steps: Number(query.get('p')) })
}

export interface PackedGameOptions {
  id: string
  name: string
  /** The figures shown on the poster and in the share text, as [value, label]. */
  stats(result: PackedResult): [string, string][]
  /** 小伴's expression and line for this round. */
  cheer(result: PackedResult): [RobotInput, string]
  /** The small label under the game name on the poster. */
  mode?(result: PackedResult): string
  /** Inserted after the game name in the share text, e.g. "一剑破无常，". */
  shareLead?(result: PackedResult): string
}

/** Builds the website description of a game that reports results through `result-share.js`. */
export function packedResultGame(options: PackedGameOptions): ResultGame<PackedResult> {
  const { id, name } = options
  return {
    id, name,
    parse: value => parsePackedResult(id, value),
    canShare: (result, round) => Boolean(result.gameId === id && round?.finished && result.score === round.score && result.lines === round.lines && result.maxTile === round.maxTile),
    fragment: packedFragment,
    read: fragment => readPackedResult(id, fragment),
    subtitle: result => `${name} · ${result.score} 分`,
    shareText: (result) => {
      const stats = options.stats(result).map(([value, label]) => `${label} ${value}`).join('，')
      return `我在${name}${options.shareLead?.(result) ?? ''}拿了 ${result.score} 分，${stats}。你能超过我吗？`
    },
    poster: (result) => {
      const [face, cheer] = options.cheer(result)
      return { gameId: id, name, mode: options.mode?.(result) ?? '轻松玩一局', cheer, face, score: result.score, stats: options.stats(result) }
    },
  }
}
