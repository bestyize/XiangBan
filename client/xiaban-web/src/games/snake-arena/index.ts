import type { RobotInput } from '../../robot/expressions.ts'
import type { ResultGame } from '../common/types.ts'
import { parseSnakeResult, readSnakeResult, snakeFragment, snakeMode, survivalTime, type SnakeResult } from './share.ts'

export const snakeArena: ResultGame<SnakeResult> = {
  id: 'snake-arena',
  name: '小伴贪吃蛇',
  openDirectly: true,
  launchParams: (search) => ({
    ...(search.get('round') === 'endless' ? { round: 'endless' } : {}),
    ...(search.get('difficulty') === 'lively' ? { difficulty: 'lively' } : {}),
  }),
  parse: parseSnakeResult,
  canShare: (result, round) => Boolean(round?.finished && result.score === round.score),
  fragment: snakeFragment,
  read: readSnakeResult,
  challengeQuery: (result): Record<string, string> => result ? { round: result.roundType, difficulty: result.difficulty } : {},
  subtitle: (result) => `${snakeMode(result)} · ${result.score} 分`,
  shareText: (result) => `我在小伴贪吃蛇拿了 ${result.score} 分，存活 ${survivalTime(result.seconds)}，淘汰 ${result.kills} 条 AI 蛇。你能超过我吗？`,
  poster: (result) => {
    const [face, cheer]: [RobotInput, string] = result.kills >= 3
      ? ['proud', `一口气围住 ${result.kills} 条 AI 蛇，太猛了`]
      : result.roundType === 'endless' && result.seconds >= 180
        ? ['laughing', `撑了 ${survivalTime(result.seconds)}，我在旁边一直喊加油`]
        : ['excited', '这一局，吃得圆滚滚']
    const stats: [string, string][] = [[String(result.mass), '最终长度'], [String(result.kills), '淘汰 AI'], [survivalTime(result.seconds), '存活时长']]
    return { gameId: 'snake-arena', name: '小伴贪吃蛇', mode: snakeMode(result), cheer, face, score: result.score, stats }
  },
}
