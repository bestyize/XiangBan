import { describe, it, expect } from 'vitest'
import { readFileSync } from 'node:fs'
import vm from 'node:vm'
import { createSnakeMemoryWallet, newSnakeEconomyState, parseSnakeEconomyState, snakeCoinsForScore, snakeEconomyDay } from './economyCore.ts'

describe('snake local economy', () => {
  it('uses the Shanghai product day and score thresholds', () => {
    expect(snakeEconomyDay(new Date('2026-10-06T16:00:00Z'))).toBe('2026-10-07')
    expect([99, 100, 120, 9600].map(snakeCoinsForScore)).toEqual([0, 5, 6, 80])
  })
  it('debits inventory once per request and rejects reused request IDs', () => {
    const wallet = createSnakeMemoryWallet()
    const bought = wallet.act('buy_shield', { requestId: 'purchase' })
    expect(wallet.act('buy_shield', { requestId: 'purchase' })).toEqual(bought)
    expect(wallet.read().coins).toBe(20)
    expect(() => wallet.act('use_shield', { requestId: 'purchase', sessionId: 'round' })).toThrow('其他操作')
    expect(wallet.act('use_shield', { requestId: 'use', sessionId: 'round' }).protectionSeconds).toBe(3)
    expect(() => wallet.act('use_shield', { requestId: 'empty', sessionId: 'round' })).toThrow('保护卡')
    expect(() => wallet.act('buy_shield', { requestId: undefined as unknown as string })).toThrow('编号')
  })
  it('rejects active settlements, prevents duplicate rounds and limits each day to 300', () => {
    let day = '2026-10-07'
    const wallet = createSnakeMemoryWallet(undefined, () => day)
    expect(() => wallet.act('settle', { requestId: 'early', sessionId: 'round', score: 9600 })).toThrow('本局结束')
    for (let i = 0; i < 4; i++) {
      const reward = wallet.act('settle', { requestId: `settle-${i}`, sessionId: `round-${i}`, score: 9600, finished: true })
      expect(reward.coinsEarned).toBe(i === 3 ? 60 : 80)
    }
    expect(wallet.act('settle', { requestId: 'double', sessionId: 'round-0', score: 9600, finished: true }).coinsEarned).toBe(0)
    expect(wallet.act('settle', { requestId: 'cap', sessionId: 'capped', score: 9600, finished: true }).coinsEarned).toBe(0)
    day = '2026-10-08'
    expect(wallet.act('settle', { requestId: 'new-day', sessionId: 'tomorrow', score: 9600, finished: true }).coinsEarned).toBe(80)
    expect(wallet.read().coins).toBe(420)
  })
  it('shares once daily, consumes next-round shield once and revives once per live round', () => {
    const wallet = createSnakeMemoryWallet()
    wallet.act('share_reward', { requestId: 'share', sessionId: 'ended', finished: true })
    expect(wallet.read().shareAvailable).toBe(false)
    expect(wallet.act('consume_start_shield', { requestId: 'start', sessionId: 'live' }).protectionSeconds).toBe(5)
    wallet.act('share_reward', { requestId: 'share-again', sessionId: 'ended', finished: true })
    expect(wallet.read().startShieldSeconds).toBe(0)
    expect(wallet.act('consume_start_shield', { requestId: 'start-again', sessionId: 'live' }).protectionSeconds).toBe(0)
    const revived = wallet.act('revive', { requestId: 'revive', sessionId: 'live' })
    expect(revived.wallet.coins).toBe(10); expect(revived.protectionSeconds).toBe(5)
    expect(() => wallet.act('revive', { requestId: 'twice', sessionId: 'live' })).toThrow('一次')
    wallet.act('settle', { requestId: 'finish', sessionId: 'live', score: 9600, finished: true })
    expect(() => wallet.act('revive', { requestId: 'ended', sessionId: 'live' })).toThrow('结束')
  })
  it('persists guest balances and idempotency without granting initial coins again', () => {
    const wallet = createSnakeMemoryWallet()
    const result = wallet.act('buy_shield', { requestId: 'buy' })
    const saved = parseSnakeEconomyState(JSON.stringify(wallet.snapshot()))!
    const restored = createSnakeMemoryWallet(saved)
    expect(restored.read().coins).toBe(20)
    expect(restored.act('buy_shield', { requestId: 'buy' })).toEqual(result)
    expect(parseSnakeEconomyState(JSON.stringify({ ...newSnakeEconomyState(), coins: -1 }))).toBeNull()
  })
  it('ships a standalone async memory wallet without account, browser storage or networking', async () => {
    const context = vm.createContext({ structuredClone })
    vm.runInContext(readFileSync(new URL('../../../public/game-packs/snake-arena/economy.js', import.meta.url), 'utf8'), context)
    const standalone = context.XiabanSnakeEconomy.createMemoryWallet()
    expect((await standalone.read()).coins).toBe(40)
    expect((await standalone.act('revive', { requestId: 'revive', sessionId: 'standalone' })).protectionSeconds).toBe(5)
    expect((await standalone.read()).coins).toBe(10)
    expect((await context.XiabanSnakeEconomy.createMemoryWallet().read()).coins).toBe(40)
  })
})
