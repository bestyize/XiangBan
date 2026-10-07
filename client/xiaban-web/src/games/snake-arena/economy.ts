import { api, getToken } from '../../api/client.ts'
import { storage } from '../../lib/storage.ts'
import { createSnakeMemoryWallet, parseSnakeEconomyState, type SnakeWallet, type SnakeWalletAction, type SnakeWalletResult, type SnakeActionOptions } from './economyCore.ts'

export type { SnakeWallet, SnakeWalletAction, SnakeWalletResult, SnakeActionOptions } from './economyCore.ts'
const KEY = 'xb_guest_snake_wallet_v1'
const URL = '/api/games/snake-arena/wallet'
let queue: Promise<unknown> = Promise.resolve()

function local<T>(operation: (wallet: ReturnType<typeof createSnakeMemoryWallet>) => T): Promise<T> {
  const run = () => {
    const wallet = createSnakeMemoryWallet(parseSnakeEconomyState(storage.get(KEY)) ?? undefined)
    const result = operation(wallet)
    storage.set(KEY, JSON.stringify(wallet.snapshot()))
    return result
  }
  // Serialize same-tab writes and, where supported, coordinate other tabs on this origin.
  const next = queue.then(() => typeof navigator !== 'undefined' && navigator.locks
    ? navigator.locks.request(KEY, run) : run())
  queue = next.catch(() => undefined)
  return next
}

export async function readSnakeWallet(): Promise<SnakeWallet> {
  if (getToken()) return api<SnakeWallet>(URL)
  return local((wallet) => wallet.read())
}
export async function actSnakeWallet(action: SnakeWalletAction, options: SnakeActionOptions): Promise<SnakeWalletResult> {
  if (getToken()) return api<SnakeWalletResult>(URL + '/actions', {
    method: 'POST', body: { action, requestId: options.requestId, sessionId: options.sessionId },
  })
  return local((wallet) => wallet.act(action, options))
}
