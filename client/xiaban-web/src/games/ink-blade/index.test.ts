import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { inkBlade } from './index.ts'

const result = { gameId: 'ink-blade', score: 8120, lines: 24, maxTile: 31, level: 1, steps: 14 }

describe('ink blade sharing', () => {
  it('round trips the result fragment and labels the combat metrics', () => {
    expect(inkBlade.parse(result)).toEqual(result)
    expect(inkBlade.read(inkBlade.fragment(result))).toEqual(result)
    expect(inkBlade.shareText(result)).toBe('我在水墨剑客一剑破无常，拿了 8120 分，击败 24，最高连击 31，弹反 14。你能超过我吗？')
    expect(inkBlade.shareText({ ...result, level: 0 })).toBe('我在水墨剑客拿了 8120 分，击败 24，最高连击 31，弹反 14。你能超过我吗？')
  })

  it('reacts on the poster to whether the scroll was cleared', () => {
    const cleared = inkBlade.poster(result)
    expect(cleared.mode).toBe('一卷终了')
    expect(cleared.face).toBe('proud')
    expect(cleared.cheer).toContain('14')
    expect(cleared.stats).toEqual([['24', '击败'], ['31', '最高连击'], ['14', '弹反']])
    const fallen = inkBlade.poster({ ...result, score: 900, lines: 3, maxTile: 4, level: 0, steps: 1 })
    expect(fallen.mode).toBe('画卷未完')
    expect(fallen.cheer).toBe('提剑再入，画卷还在等你')
  })

  it('ships a landscape manifest registered in the catalog', () => {
    const manifest = JSON.parse(readFileSync(new URL('../../../public/game-packs/ink-blade/manifest.json', import.meta.url), 'utf8'))
    const catalog = JSON.parse(readFileSync(new URL('../../../public/game-packs/catalog.json', import.meta.url), 'utf8'))
    expect(manifest).toMatchObject({ id: 'ink-blade', entry: 'index.html', orientation: 'landscape', runtimeProtocol: 1 })
    expect(catalog.games).toContain('ink-blade')
  })
})
