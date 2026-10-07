import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { actSnakeWallet, readSnakeWallet } from './economy.ts'
import { claimSnakeShareReward, rememberSnakeShareRound, snakeShareRound } from './shareReward.ts'

vi.mock('./economy.ts', () => ({ actSnakeWallet: vi.fn(), readSnakeWallet: vi.fn() }))
beforeEach(() => {
  const values = new Map<string, string>()
  vi.stubGlobal('sessionStorage', { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => values.set(key, value) })
  vi.useFakeTimers(); vi.setSystemTime(new Date('2026-10-07T15:59:50Z'))
  vi.mocked(actSnakeWallet).mockResolvedValue({ wallet: { coins: 40, shields: 0, startShieldSeconds: 5, shareAvailable: false, prices: { shield: 20, revive: 30 } }, coinsEarned: 0, protectionSeconds: 0 })
})
afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); vi.clearAllMocks() })
describe('private snake share reward context', () => {
  it('requires a matching recent owned round and expires it after one day', async () => {
    rememberSnakeShareRound('owned-session', 600)
    expect(snakeShareRound(500)).toBeNull()
    expect(snakeShareRound(600)?.sessionId).toBe('owned-session')
    vi.setSystemTime(new Date('2026-10-08T16:00:00Z'))
    expect(await claimSnakeShareReward(600)).toBe(false)
    expect(actSnakeWallet).not.toHaveBeenCalled()
  })
  it('reads current inventory instead of treating an old idempotent response as another shield', async () => {
    rememberSnakeShareRound('owned-session', 600)
    vi.mocked(readSnakeWallet).mockResolvedValue({ coins: 40, shields: 0, startShieldSeconds: 0, shareAvailable: false, prices: { shield: 20, revive: 30 } })
    expect(await claimSnakeShareReward(600)).toBe(false)
    expect(actSnakeWallet).toHaveBeenCalledWith('share_reward', expect.objectContaining({ requestId: 'share_2026-10-07_owned-session', sessionId: 'owned-session', score: 600, finished: true }))
  })
  it('keeps retries idempotent within a product day but uses another request after Shanghai midnight', async () => {
    rememberSnakeShareRound('owned-session', 600)
    vi.mocked(readSnakeWallet).mockResolvedValue({ coins: 40, shields: 0, startShieldSeconds: 5, shareAvailable: false, prices: { shield: 20, revive: 30 } })
    expect(await claimSnakeShareReward(600)).toBe(true)
    expect(await claimSnakeShareReward(600)).toBe(true)
    const first = vi.mocked(actSnakeWallet).mock.calls[0][1].requestId
    expect(vi.mocked(actSnakeWallet).mock.calls[1][1].requestId).toBe(first)
    vi.setSystemTime(new Date('2026-10-07T16:00:10Z'))
    await claimSnakeShareReward(600)
    expect(vi.mocked(actSnakeWallet).mock.calls[2][1].requestId).toBe('share_2026-10-08_owned-session')
  })
})
