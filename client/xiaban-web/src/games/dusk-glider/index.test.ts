import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { duskGlider } from './index.ts'

const result = { gameId: 'dusk-glider', score: 5240, lines: 11, maxTile: 4, level: 3, steps: 18 }

describe('dusk glider sharing', () => {
  it('round trips the result fragment and labels the glide metrics', () => {
    expect(duskGlider.parse(result)).toEqual(result)
    expect(duskGlider.read(duskGlider.fragment(result))).toEqual(result)
    expect(duskGlider.shareText(result)).toBe('我在暮色滑翔飞过 3 座岛，拿了 5240 分，完美落地 11，最长连击 4，抵达岛屿 3。你能超过我吗？')
    expect(duskGlider.shareText({ ...result, level: 1 })).toBe('我在暮色滑翔拿了 5240 分，完美落地 11，最长连击 4，抵达岛屿 1。你能超过我吗？')
  })

  it('reacts on the poster to how far the glider flew', () => {
    const far = duskGlider.poster({ ...result, level: 6 })
    expect(far.mode).toBe('飞过 6 座岛')
    expect(far.face).toBe('proud')
    expect(far.cheer).toContain('6')
    expect(far.stats).toEqual([['11', '完美落地'], ['4', '最长连击'], ['6', '抵达岛屿']])
    const chained = duskGlider.poster({ ...result, maxTile: 7 })
    expect(chained.face).toBe('excited')
    expect(chained.cheer).toContain('7')
    const mid = duskGlider.poster(result)
    expect(mid.mode).toBe('飞过 3 座岛')
    expect(mid.face).toBe('happy')
    const first = duskGlider.poster({ ...result, score: 300, lines: 0, maxTile: 0, level: 1, steps: 1 })
    expect(first.mode).toBe('第一座岛')
    expect(first.face).toBe('happy')
    expect(first.cheer).toBe('天黑得有点快，再飞一次，前面还有岛')
  })

  it('ships a landscape manifest registered in the catalog', () => {
    const manifest = JSON.parse(readFileSync(new URL('../../../public/game-packs/dusk-glider/manifest.json', import.meta.url), 'utf8'))
    const catalog = JSON.parse(readFileSync(new URL('../../../public/game-packs/catalog.json', import.meta.url), 'utf8'))
    expect(manifest).toMatchObject({ id: 'dusk-glider', entry: 'index.html', orientation: 'landscape', runtimeProtocol: 1 })
    expect(catalog.games).toContain('dusk-glider')
  })
})
