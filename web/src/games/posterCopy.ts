import type { RobotInput } from '../robot/expressions.ts'
import { otherResultStats, RESULT_GAMES, type OtherResult } from './otherResultShare.ts'
import { survivalTime, type SnakeResult } from './resultShare.ts'

export interface GamePosterCopy {
  gameId: string
  name: string
  mode: string
  /** What 小伴 says about this round */
  cheer: string
  face: RobotInput
  score: number
  stats: [string, string][]
}

export function snakePosterCopy(result: SnakeResult): GamePosterCopy {
  const mode = `${result.roundType === 'endless' ? '无尽生存' : '三分钟竞技'} · ${result.difficulty === 'lively' ? '热闹难度' : '轻松难度'}`
  const stats: [string, string][] = [[String(result.mass), '最终长度'], [String(result.kills), '淘汰 AI'], [survivalTime(result.seconds), '存活时长']]
  const [face, cheer]: [RobotInput, string] = result.kills >= 3
    ? ['proud', `一口气围住 ${result.kills} 条 AI 蛇，太猛了`]
    : result.roundType === 'endless' && result.seconds >= 180
      ? ['laughing', `撑了 ${survivalTime(result.seconds)}，我在旁边一直喊加油`]
      : ['excited', '这一局，吃得圆滚滚']
  return { gameId: 'snake-arena', name: '小伴贪吃蛇', mode, cheer, face, score: result.score, stats }
}

export function otherPosterCopy(result: OtherResult): GamePosterCopy {
  const name = RESULT_GAMES[result.gameId] ?? '小游戏'
  const [face, cheer]: [RobotInput, string] =
    result.gameId === 'merge-2048'
      ? result.maxTile >= 2048 ? ['laughing', '合出 2048 了！我要给你鼓掌'] : ['excited', `合到了 ${result.maxTile}，下次冲 2048`]
      : result.gameId === 'water-sort'
        ? ['happy', `调完 ${result.level} 关，一滴都没洒`]
        : result.lines >= 20 ? ['laughing', `一口气消了 ${result.lines} 行，好解压`] : ['excited', '这一局，玩得漂亮']
  return { gameId: result.gameId, name, mode: '轻松玩一局', cheer, face, score: result.score, stats: otherResultStats(result) }
}
