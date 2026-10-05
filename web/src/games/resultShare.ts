export const GAME_SHARE_PATH = '/games/snake-arena/share'
const PUBLIC_BASE = 'https://xiaban.xyz'

export interface SnakeResult {
  score: number
  mass: number
  kills: number
  seconds: number
  roundType: 'timed' | 'endless'
  difficulty: 'cozy' | 'lively'
}

export function parseSnakeResult(value: unknown): SnakeResult | null {
  if (!value || typeof value !== 'object') return null
  const data = value as Record<string, unknown>
  const integer = (key: string, min: number, max: number) => typeof data[key] === 'number' && Number.isInteger(data[key]) && data[key] >= min && data[key] <= max
  if (!integer('score', 0, 1e9) || !integer('mass', 32, 1000) || !integer('kills', 0, 1e6) || !integer('seconds', 0, 1e7) ||
    (data.roundType !== 'timed' && data.roundType !== 'endless') || (data.difficulty !== 'cozy' && data.difficulty !== 'lively')) return null
  return { score: data.score as number, mass: data.mass as number, kills: data.kills as number, seconds: data.seconds as number, roundType: data.roundType as SnakeResult['roundType'], difficulty: data.difficulty as SnakeResult['difficulty'] }
}

export function resultFragment(result: SnakeResult): string {
  if (!parseSnakeResult(result)) throw new Error('战绩不完整，请重新玩一局。')
  return new URLSearchParams({ v: '1', s: String(result.score), l: String(result.mass), k: String(result.kills), t: String(result.seconds), r: result.roundType, d: result.difficulty }).toString()
}

export function readSharedResult(fragment: string): SnakeResult | null {
  if (!fragment || fragment.length > 180) return null
  const params = new URLSearchParams(fragment.replace(/^#/, ''))
  const keys = ['v', 's', 'l', 'k', 't', 'r', 'd']
  if (params.size !== keys.length || keys.some(key => params.getAll(key).length !== 1) || params.get('v') !== '1') return null
  if (['s', 'l', 'k', 't'].some(key => !/^\d+$/.test(params.get(key) || ''))) return null
  return parseSnakeResult({ score: Number(params.get('s')), mass: Number(params.get('l')), kills: Number(params.get('k')), seconds: Number(params.get('t')), roundType: params.get('r'), difficulty: params.get('d') })
}

function invite(value?: string): string { return value && /^[a-zA-Z0-9_-]{1,80}$/.test(value) ? value : '' }

/** Local preview addresses must never become unusable invitation links on shared cards. */
export function gamePublicBase(base: string): string {
  try {
    const url = new URL(base)
    const host = url.hostname.toLowerCase()
    const local = /^(localhost|127\.|0\.|10\.|192\.168\.|172\.(1[6-9]|2\d|3[01])\.)/.test(host) || host === '[::1]' || host.endsWith('.local')
    return !local && ['http:', 'https:'].includes(url.protocol) && !url.username && !url.password ? url.origin : PUBLIC_BASE
  } catch { return PUBLIC_BASE }
}

export function gameShareLink(result: SnakeResult, base: string, inviteCode?: string): string {
  const url = new URL(GAME_SHARE_PATH, gamePublicBase(base))
  url.searchParams.set('from', 'game_share')
  if (invite(inviteCode)) url.searchParams.set('inv', invite(inviteCode))
  url.hash = resultFragment(result)
  return url.toString()
}

export function localResultPath(result: SnakeResult, inviteCode?: string): string {
  const query = new URLSearchParams({ own: '1' })
  if (invite(inviteCode)) query.set('inv', invite(inviteCode))
  return GAME_SHARE_PATH + '?' + query + '#' + resultFragment(result)
}

export function canShareFinishedRound(result: SnakeResult | null, round: { finished: boolean; score: number } | null): result is SnakeResult {
  return Boolean(result && round?.finished && result.score === round.score)
}

export function gameChallengePath(result: SnakeResult | null, inviteCode?: string): string {
  const query = new URLSearchParams({ from: 'game_share' })
  if (invite(inviteCode)) query.set('inv', invite(inviteCode))
  if (result) { query.set('round', result.roundType); query.set('difficulty', result.difficulty) }
  return '/games/snake-arena?' + query
}

export function survivalTime(seconds: number): string { return Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2, '0') }

export function gameShareText(result: SnakeResult): string {
  return `我在小伴贪吃蛇拿了 ${result.score} 分，存活 ${survivalTime(result.seconds)}，淘汰 ${result.kills} 条 AI 蛇。你能超过我吗？`
}
