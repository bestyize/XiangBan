import { afterEach, describe, expect, it, vi } from 'vitest'

describe('bridge', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    vi.resetModules()
  })

  async function load(withApp: boolean) {
    const posted: string[] = []
    const style = { setProperty: vi.fn() }
    vi.stubGlobal('document', { documentElement: { style } })
    vi.stubGlobal('window', withApp ? { XiabanBridge: { postMessage: (m: string) => posted.push(m) } } : {})
    const bridge = await import('./bridge.ts')
    return { bridge, posted, style }
  }

  it('sends versioned JSON envelopes only inside the App', async () => {
    const { bridge, posted } = await load(true)
    expect(bridge.inApp()).toBe(true)
    bridge.send('checkin_done', { title: 'overtime_hero' })
    expect(JSON.parse(posted[0])).toEqual({ type: 'checkin_done', payload: { title: 'overtime_hero' }, version: 1 })

    const outside = await (async () => { vi.resetModules(); return load(false) })()
    expect(outside.bridge.inApp()).toBe(false)
    outside.bridge.send('share', {})
    expect(outside.posted).toHaveLength(0)
  })

  it('delivers App messages, replays early ones and ignores newer protocol versions', async () => {
    const { bridge, style } = await load(true)
    bridge.receive(JSON.stringify({ type: 'auth', payload: { token: 't1' }, version: 1 }))
    await expect(bridge.waitForAuth(100)).resolves.toBe('t1')

    bridge.receive(JSON.stringify({ type: 'env', payload: { safeAreaTop: 44 }, version: 1 }))
    expect(style.setProperty).toHaveBeenCalledWith('--safe-top', '44px')

    const seen: unknown[] = []
    bridge.on('custom', (p) => seen.push(p))
    bridge.receive(JSON.stringify({ type: 'custom', payload: { a: 1 }, version: 2 }))
    bridge.receive('not json')
    expect(seen).toEqual([])
  })
})
