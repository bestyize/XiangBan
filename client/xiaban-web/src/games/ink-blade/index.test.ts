import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { challengePath, resultShareLink } from '../common/share.ts'
import { inkBlade } from './index.ts'

const legacy = { gameId: 'ink-blade', score: 8120, lines: 24, maxTile: 31, level: 1, steps: 14 }
const stage = { ...legacy, score: 3260, lines: 11, maxTile: 18, steps: 6, stage: 103, seals: 7, grade: 3 }

describe('ink blade sharing', () => {
  it('round trips a 幅 result with its seals and grade', () => {
    expect(inkBlade.parse({ ...stage, token: 'secret' })).toEqual(stage)
    const url = new URL(resultShareLink(inkBlade, stage, 'https://xiaban.xyz', 'friend-code'))
    expect(url.hash).toContain('v=2')
    expect(inkBlade.read(url.hash)).toEqual(stage)
    expect(url.toString()).not.toContain('secret')
  })

  it('still opens version 1 links from the single-scroll release as 长卷', () => {
    expect(inkBlade.read('#v=1&s=8120&l=24&m=31&n=1&p=14')).toEqual({ ...legacy, stage: 100, seals: 0, grade: 0 })
    expect(inkBlade.parse(legacy)).toEqual({ ...legacy, stage: 100, seals: 0, grade: 0 })
  })

  it('rejects unknown 幅, out-of-range seals or grades and ambiguous fragments', () => {
    for (const change of [{ stage: 107 }, { stage: 0 }, { seals: 8 }, { seals: -1 }, { grade: 5 }, { grade: 1.5 }, { stage: '103' }]) {
      expect(inkBlade.parse({ ...stage, ...change })).toBeNull()
    }
    const hash = inkBlade.fragment(stage)
    for (const bad of [hash.replace('t=103', 't=999'), hash + '&t=101', hash.replace('v=2', 'v=3'), hash.replace('e=7', 'e=x'), hash + '&token=1']) {
      expect(inkBlade.read(bad)).toBeNull()
    }
  })

  it('writes share text and poster copy for the 幅', () => {
    expect(inkBlade.shareText(stage)).toBe('我在水墨剑客「鸦岭」画出神品（三印），拿了 3260 分，击败 11，最高连击 18，弹反 6。你能超过我吗？')
    expect(inkBlade.shareText({ ...legacy, stage: 100, seals: 0, grade: 0 })).toBe('我在水墨剑客一剑破无常，拿了 8120 分，击败 24，最高连击 31，弹反 14。你能超过我吗？')
    const poster = inkBlade.poster(stage)
    expect(poster.mode).toBe('竹林卷 · 第三幅 · 神品')
    expect(poster.cheer).toBe('「鸦岭」三印齐落，一笔不差')
    expect(poster.stats).toEqual([['11', '击败'], ['18', '最高连击'], ['6', '弹反']])
    expect(inkBlade.poster({ ...stage, level: 0, seals: 0, grade: 0, lines: 3 }).cheer).toBe('提剑再入，画卷还在等你')
    expect(inkBlade.subtitle(stage)).toBe('竹林卷 · 第三幅 · 神品 · 3260 分')
  })

  it('turns a shared result into a duel on the same 幅 with the score to beat', () => {
    const duel = new URL(challengePath(inkBlade, stage, 'friend-code'), 'https://xiaban.xyz')
    expect(duel.pathname).toBe('/games/ink-blade')
    expect(duel.searchParams.get('stage')).toBe('103')
    expect(duel.searchParams.get('target')).toBe('3260')
    expect(inkBlade.launchParams!(duel.searchParams)).toEqual({ stage: '103', target: '3260' })
    expect(inkBlade.launchParams!(new URLSearchParams('stage=999&target=5'))).toEqual({})
    expect(inkBlade.launchParams!(new URLSearchParams('stage=103&target=-1'))).toEqual({ stage: '103' })
  })

  it('ships a landscape manifest that keeps a save and is registered in the catalog', () => {
    const manifest = JSON.parse(readFileSync(new URL('../../../public/game-packs/ink-blade/manifest.json', import.meta.url), 'utf8'))
    const catalog = JSON.parse(readFileSync(new URL('../../../public/game-packs/catalog.json', import.meta.url), 'utf8'))
    expect(manifest).toMatchObject({ id: 'ink-blade', entry: 'index.html', orientation: 'landscape', runtimeProtocol: 1, save: true })
    expect(catalog.games).toContain('ink-blade')
  })
})
