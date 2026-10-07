/** Snake keeps its own landscape result payload: mass, AI eliminations, survival time, round type and difficulty. */
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
  if (!integer('score', 0, 1e9) || !integer('mass', 32, 1e7) || !integer('kills', 0, 1e6) || !integer('seconds', 0, 1e7) ||
    (data.roundType !== 'timed' && data.roundType !== 'endless') || (data.difficulty !== 'cozy' && data.difficulty !== 'lively')) return null
  return { score: data.score as number, mass: data.mass as number, kills: data.kills as number, seconds: data.seconds as number, roundType: data.roundType as SnakeResult['roundType'], difficulty: data.difficulty as SnakeResult['difficulty'] }
}

export function snakeFragment(result: SnakeResult): string {
  if (!parseSnakeResult(result)) throw new Error('战绩不完整，请重新玩一局。')
  return new URLSearchParams({ v: '1', s: String(result.score), l: String(result.mass), k: String(result.kills), t: String(result.seconds), r: result.roundType, d: result.difficulty }).toString()
}

export function readSnakeResult(fragment: string): SnakeResult | null {
  if (!fragment || fragment.length > 180) return null
  const params = new URLSearchParams(fragment.replace(/^#/, ''))
  const keys = ['v', 's', 'l', 'k', 't', 'r', 'd']
  if (params.size !== keys.length || keys.some(key => params.getAll(key).length !== 1) || params.get('v') !== '1') return null
  if (['s', 'l', 'k', 't'].some(key => !/^\d+$/.test(params.get(key) || ''))) return null
  return parseSnakeResult({ score: Number(params.get('s')), mass: Number(params.get('l')), kills: Number(params.get('k')), seconds: Number(params.get('t')), roundType: params.get('r'), difficulty: params.get('d') })
}

export function survivalTime(seconds: number): string { return Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2, '0') }

export function snakeMode(result: Pick<SnakeResult, 'roundType' | 'difficulty'>): string {
  return result.roundType === 'endless' ? '无尽成长' : '三分钟冲分'
}
