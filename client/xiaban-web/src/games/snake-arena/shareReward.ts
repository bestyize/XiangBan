import { actSnakeWallet, readSnakeWallet } from './economy.ts'
import { snakeEconomyDay } from './economyCore.ts'
const KEY = 'xb_snake_share_round'
interface ShareRound { sessionId: string; score: number; createdAt: number }
export function rememberSnakeShareRound(sessionId: string, score: number): void {
  try { sessionStorage.setItem(KEY, JSON.stringify({ sessionId, score, createdAt: Date.now() })) } catch { /* Sharing still works without rewards. */ }
}
export function snakeShareRound(score: number): ShareRound | null {
  try {
    const value = JSON.parse(sessionStorage.getItem(KEY) || 'null') as ShareRound | null
    return value && typeof value.sessionId === 'string' && value.score === score && Number.isFinite(value.createdAt) && Date.now() - value.createdAt >= 0 && Date.now() - value.createdAt < 24 * 3600_000 ? value : null
  } catch { return null }
}
export async function claimSnakeShareReward(score: number): Promise<boolean> {
  const round = snakeShareRound(score)
  if (!round) return false
  await actSnakeWallet('share_reward', { requestId: 'share_' + snakeEconomyDay() + '_' + round.sessionId, sessionId: round.sessionId, score, finished: true })
  return (await readSnakeWallet()).startShieldSeconds === 5
}
