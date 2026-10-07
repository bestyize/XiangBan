import { readFileSync } from 'node:fs'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { gameEntryUrl, loadGameManifest } from './packLoader.ts'

afterEach(() => vi.unstubAllGlobals())

describe('game pack orientation', () => {
  const manifest = JSON.parse(readFileSync(new URL('../../../public/game-packs/snake-arena/manifest.json', import.meta.url), 'utf8'))
  function serve(data: unknown) { vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => data })) }

  it('loads the new landscape manifest without changing existing portrait packs', async () => {
    serve(manifest)
    expect((await loadGameManifest('snake-arena')).orientation).toBe('landscape')
    const classic = JSON.parse(readFileSync(new URL('../../../public/game-packs/water-sort/manifest.json', import.meta.url), 'utf8'))
    serve(classic)
    expect((await loadGameManifest('water-sort')).orientation).toBeUndefined()
  })

  it('rejects unknown orientation values instead of expanding arbitrary packs', async () => {
    serve({ ...manifest, orientation: 'diagonal' })
    await expect(loadGameManifest('snake-arena')).rejects.toThrow('游戏包清单格式不支持')
  })

  it('passes the game launch options and keeps the host nonce authoritative', () => {
    const url = new URL(gameEntryUrl(manifest, 'private-nonce', { round: 'endless', difficulty: 'lively', hostNonce: 'forged' }), 'https://xiaban.xyz')
    expect(url.pathname).toBe('/game-packs/snake-arena/index.html')
    expect(url.searchParams.get('hostNonce')).toBe('private-nonce')
    expect(url.searchParams.get('round')).toBe('endless')
    expect(url.searchParams.get('difficulty')).toBe('lively')
  })

  it('adds nothing but the nonce for games without launch options', () => {
    const classic = JSON.parse(readFileSync(new URL('../../../public/game-packs/water-sort/manifest.json', import.meta.url), 'utf8'))
    const url = new URL(gameEntryUrl(classic, 'nonce'), 'https://xiaban.xyz')
    expect([...url.searchParams.keys()]).toEqual(['hostNonce'])
  })
})
