import { gamePublicBase } from './resultShare.ts'

export const RESULT_GAMES: Record<string, string> = {
  'falling-blocks': '俄罗斯方块', 'merge-2048': '2048', 'block-puzzle': '小伴消方块', 'water-sort': '调饮料大师',
}
export interface OtherResult { gameId: string; score: number; lines: number; maxTile: number; level: number; steps: number }
export function parseOtherResult(value: unknown): OtherResult | null {
  if (!value || typeof value !== 'object') return null
  const data = value as Record<string, unknown>
  if (typeof data.gameId !== 'string' || !Object.hasOwn(RESULT_GAMES, data.gameId)) return null
  const limits: Record<string, number> = { score: 1e9, lines: 1e6, maxTile: 1073741824, level: 1e4, steps: 1e7 }
  for (const [key, max] of Object.entries(limits)) {
    if (typeof data[key] !== 'number' || !Number.isInteger(data[key]) || data[key] < 0 || data[key] > max) return null
  }
  return { gameId: data.gameId, score: data.score as number, lines: data.lines as number, maxTile: data.maxTile as number, level: data.level as number, steps: data.steps as number }
}
export function otherResultFragment(result: OtherResult): string {
  if (!parseOtherResult(result)) throw new Error('战绩不完整，请重新玩一局。')
  return new URLSearchParams({ v: '1', s: String(result.score), l: String(result.lines), m: String(result.maxTile), n: String(result.level), p: String(result.steps) }).toString()
}
export function canShareOtherResult(result: OtherResult | null, gameId: string, round: { finished: boolean; score: number; lines: number; maxTile: number } | null): result is OtherResult {
  return Boolean(result && result.gameId === gameId && round?.finished && result.score === round.score && result.lines === round.lines && result.maxTile === round.maxTile)
}
export function readOtherResult(gameId: string, fragment: string): OtherResult | null {
  if (!fragment || fragment.length > 180) return null
  const query = new URLSearchParams(fragment.replace(/^#/, ''))
  const keys = ['v', 's', 'l', 'm', 'n', 'p']
  if (query.size !== keys.length || keys.some(key => query.getAll(key).length !== 1) || query.get('v') !== '1' || keys.slice(1).some(key => !/^\d+$/.test(query.get(key) || ''))) return null
  return parseOtherResult({ gameId, score: Number(query.get('s')), lines: Number(query.get('l')), maxTile: Number(query.get('m')), level: Number(query.get('n')), steps: Number(query.get('p')) })
}
function referral(inviteCode?: string): string { return inviteCode && /^[a-zA-Z0-9_-]{1,80}$/.test(inviteCode) ? inviteCode : '' }
export function otherResultPath(result: OtherResult, inviteCode?: string, own = true): string {
  const query = new URLSearchParams(own ? { own: '1' } : { from: 'game_share' })
  if (referral(inviteCode)) query.set('inv', referral(inviteCode))
  return `/games/${result.gameId}/share?${query}#${otherResultFragment(result)}`
}
export function otherShareLink(result: OtherResult, base: string, inviteCode?: string): string { return new URL(otherResultPath(result, inviteCode, false), gamePublicBase(base)).toString() }
export function otherChallengePath(gameId: string, inviteCode?: string): string {
  const query = new URLSearchParams({ from: 'game_share' })
  if (referral(inviteCode)) query.set('inv', referral(inviteCode))
  return `/games/${gameId}?${query}`
}
export function sharePageGameId(path: string): string | null {
  const match = /^\/games\/([a-z0-9-]+)\/share$/.exec(path)
  return match && (match[1] === 'snake-arena' || Object.hasOwn(RESULT_GAMES, match[1])) ? match[1] : null
}
export function otherResultStats(result: OtherResult): [string, string][] {
  if (result.gameId === 'water-sort') return [[String(result.level), '完成关卡'], [String(result.steps), '倒水步数']]
  if (result.gameId === 'merge-2048') return [[String(result.maxTile), '最大方块']]
  return [[String(result.lines), result.gameId === 'block-puzzle' ? '消除行列' : '消除行数']]
}
export function otherShareText(result: OtherResult): string {
  const stats = otherResultStats(result).map(([value, label]) => `${label} ${value}`).join('，')
  return `我在${RESULT_GAMES[result.gameId]}拿了 ${result.score} 分，${stats}。你能超过我吗？`
}
