import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

const code = readFileSync(new URL('../../../public/game-packs/result-share.js', import.meta.url), 'utf8')
function runtime(embedded: boolean) {
  const events: Record<string, (event?: unknown) => void> = {}
  const messages: unknown[] = []
  const button = { hidden: true, addEventListener: (type: string, callback: () => void) => { events[type] = callback } }
  const location = { search: '?hostNonce=demo', href: '' }
  const parent = { postMessage: (value: unknown) => messages.push(value) }
  const window = { parent: embedded ? parent : undefined, addEventListener: (type: string, callback: () => void) => { events[type] = callback } } as unknown as { parent: unknown; XiabanResultShare: { finish: (value: unknown) => void; reset: () => void } }
  if (!embedded) window.parent = window
  runInNewContext(code, { window, location, document: { getElementById: () => button }, URLSearchParams })
  const init = (source = parent, nonce = 'demo') => events.message({ source, data: { channel: 'xiaban.game-host', protocol: 1, type: 'host.init', nonce } })
  return { window, button, events, messages, location, init }
}
describe('shared pack result button', () => {
  it('requires verified host initialization and clears prior results on restart', () => {
    const game = runtime(true)
    game.window.XiabanResultShare.finish({ gameId: 'block-puzzle', score: 234, lines: 5 })
    expect(game.button.hidden).toBe(false)
    game.init(undefined, 'wrong'); game.events.click()
    expect(game.messages).toHaveLength(0)
    game.init(); game.events.click()
    expect(game.messages[0]).toMatchObject({ type: 'game.share', nonce: 'demo', result: { gameId: 'block-puzzle', score: 234, lines: 5, maxTile: 0 } })
    game.window.XiabanResultShare.reset(); game.events.click()
    expect(game.button.hidden).toBe(true)
    expect(game.messages).toHaveLength(1)
  })
  it('opens the same share page when a pack is played directly', () => {
    const game = runtime(false)
    game.window.XiabanResultShare.finish({ gameId: 'water-sort', score: 300, level: 2, steps: 9 })
    game.events.click()
    expect(game.location.href).toBe('/games/water-sort/share?own=1#v=1&s=300&l=0&m=0&n=2&p=9')
  })
})
