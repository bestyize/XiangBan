import type { RobotInput } from '../../robot/expressions.ts'
import { packedFragment, parsePackedResult, readPackedResult, type PackedResult } from '../common/packedResult.ts'
import type { ResultGame } from '../common/types.ts'

/**
 * Ink blade packs kills into `lines`, best combo into `maxTile`, parries into `steps` and a clear flag into `level`.
 * Version 2 adds the 幅 (`stage`, a level code such as 103 for 卷一第三幅, 100 for the whole 长卷), the seal bitmask
 * (1 过, 2 全, 4 技) and the grade (1–4 能 / 妙 / 神 / 逸品). Version 1 links come from the single-scroll release
 * and read as 长卷 without seals or grade.
 */
export interface InkBladeResult extends PackedResult { stage: number; seals: number; grade: number }

const NAME = '水墨剑客'
const GRADES = ['', '能品', '妙品', '神品', '逸品']
const STAGES: Record<number, { name: string; sub: string }> = {
  100: { name: '长卷', sub: '竹林卷 · 长卷' },
  101: { name: '山门', sub: '竹林卷 · 第一幅' },
  102: { name: '竹林', sub: '竹林卷 · 第二幅' },
  103: { name: '鸦岭', sub: '竹林卷 · 第三幅' },
  104: { name: '断桥', sub: '竹林卷 · 第四幅' },
  105: { name: '夜雨', sub: '竹林卷 · 第五幅' },
  106: { name: '无常', sub: '竹林卷 · 终幅' },
  200: { name: '长卷', sub: '雪江卷 · 长卷' },
  201: { name: '寒峰', sub: '雪江卷 · 第一幅' },
  202: { name: '古渡', sub: '雪江卷 · 第二幅' },
  203: { name: '冰泽', sub: '雪江卷 · 第三幅' },
  204: { name: '芦荡', sub: '雪江卷 · 第四幅' },
  205: { name: '夜雪', sub: '雪江卷 · 第五幅' },
  206: { name: '独钓', sub: '雪江卷 · 终幅' },
}
const SEAL_NAMES = ['', '一印', '一印', '二印', '一印', '二印', '二印', '三印']

function extras(value: Record<string, unknown>): { stage: number; seals: number; grade: number } | null {
  const { stage = 100, seals = 0, grade = 0 } = value
  if (typeof stage !== 'number' || !Object.hasOwn(STAGES, stage)) return null
  if (typeof seals !== 'number' || !Number.isInteger(seals) || seals < 0 || seals > 7) return null
  if (typeof grade !== 'number' || !Number.isInteger(grade) || grade < 0 || grade > 4) return null
  return { stage, seals, grade }
}

export function parseInkBladeResult(value: unknown): InkBladeResult | null {
  const base = parsePackedResult('ink-blade', value)
  const rest = base && extras(value as Record<string, unknown>)
  return base && rest ? { ...base, ...rest } : null
}

function fragment(result: InkBladeResult): string {
  if (!parseInkBladeResult(result)) throw new Error('战绩不完整，请重新玩一局。')
  const query = new URLSearchParams(packedFragment(result))
  query.set('v', '2')
  query.set('t', String(result.stage)); query.set('e', String(result.seals)); query.set('q', String(result.grade))
  return query.toString()
}

function read(hash: string): InkBladeResult | null {
  if (!hash || hash.length > 200) return null
  const query = new URLSearchParams(hash.replace(/^#/, ''))
  if (query.getAll('v').length !== 1) return null
  if (query.get('v') === '1') {
    const legacy = readPackedResult('ink-blade', hash)
    return legacy ? { ...legacy, stage: 100, seals: 0, grade: 0 } : null
  }
  if (query.get('v') !== '2') return null
  const added = ['t', 'e', 'q']
  if (added.some((key) => query.getAll(key).length !== 1 || !/^\d+$/.test(query.get(key) || ''))) return null
  const [stage, seals, grade] = added.map((key) => Number(query.get(key)))
  for (const key of added) query.delete(key)
  query.set('v', '1')
  const base = readPackedResult('ink-blade', query.toString())
  return base ? parseInkBladeResult({ ...base, stage, seals, grade }) : null
}

function stageName(result: InkBladeResult) { return STAGES[result.stage].name }
function cleared(result: InkBladeResult) { return result.level > 0 }

function cheer(result: InkBladeResult): [RobotInput, string] {
  const where = stageName(result)
  if (!cleared(result)) return result.lines >= 10 ? ['excited', `斩了 ${result.lines} 个墨影，差一点就到头了`] : ['happy', '提剑再入，画卷还在等你']
  if (result.seals === 7) return ['proud', `「${where}」三印齐落，一笔不差`]
  if (result.stage === 206 || result.stage === 200) return result.steps >= 10 ? ['proud', `弹反 ${result.steps} 次，钓叟也教冰魄折断`] : ['laughing', '雪夜一剑，钓叟也称快哉']
  if (result.stage === 106 || result.stage === 100) return result.steps >= 10 ? ['proud', `弹反 ${result.steps} 次，一剑破无常`] : ['laughing', '月下一剑，无常也拦不住你']
  return result.grade >= 3 ? ['laughing', `「${where}」画出了${GRADES[result.grade]}`] : ['excited', `过了「${where}」，下一幅等你`]
}

export const inkBlade: ResultGame<InkBladeResult> = {
  id: 'ink-blade',
  name: NAME,
  launchParams: (search) => {
    const stage = Number(search.get('stage')), target = Number(search.get('target'))
    if (!Object.hasOwn(STAGES, stage)) return {}
    return { stage: String(stage), ...(Number.isInteger(target) && target > 0 && target <= 1e9 ? { target: String(target) } : {}) }
  },
  parse: parseInkBladeResult,
  canShare: (result, round) => Boolean(round?.finished && result.score === round.score && result.lines === round.lines && result.maxTile === round.maxTile),
  fragment,
  read,
  challengeQuery: (result): Record<string, string> => result ? { stage: String(result.stage), target: String(result.score) } : {},
  subtitle: (result) => `${STAGES[result.stage].sub} · ${result.grade ? GRADES[result.grade] + ' · ' : ''}${result.score} 分`,
  shareText: (result) => {
    const where = result.stage === 100 ? '' : `「${stageName(result)}」`
    const lead = cleared(result) && (result.stage === 106 || result.stage === 100) ? '一剑破无常，' : ''
    const grade = result.grade ? `画出${GRADES[result.grade]}${result.seals ? `（${SEAL_NAMES[result.seals]}）` : ''}，` : ''
    return `我在${NAME}${where}${lead}${grade}拿了 ${result.score} 分，击败 ${result.lines}，最高连击 ${result.maxTile}，弹反 ${result.steps}。你能超过我吗？`
  },
  poster: (result) => {
    const [face, line] = cheer(result)
    const mode = `${STAGES[result.stage].sub}${result.grade ? ' · ' + GRADES[result.grade] : cleared(result) ? ' · 一卷终了' : ' · 画卷未完'}`
    return {
      gameId: 'ink-blade', name: NAME, mode, cheer: line, face, score: result.score,
      stats: [[String(result.lines), '击败'], [String(result.maxTile), '最高连击'], [String(result.steps), '弹反']],
    }
  },
}
