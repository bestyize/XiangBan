import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { quizCatalog } from '../quizzes/catalog.ts'
import { defaultDisplaySettings, isDisplaySettings, moveDisplayItem, visibleDisplayItems, type DisplayItem, type DisplaySettings } from './display.ts'

const catalog = ['falling-blocks', 'merge-2048', 'block-puzzle', 'water-sort', 'snake-arena', 'ink-blade', 'dusk-glider'].map(id => ({ id, visible: true, name: id }))
const settings: DisplaySettings = { chatTabVisible: true, games: catalog, quizzes: quizCatalog.map(quiz => ({ id: quiz.id, name: quiz.name, visible: true })) }

describe('catalog presentation configuration', () => {
  it('retains original catalog order with no configuration', () => {
    const ids = JSON.parse(readFileSync(new URL('../../public/game-packs/catalog.json', import.meta.url), 'utf8')).games
    expect(catalog.map(item => item.id)).toEqual(ids)
    expect(visibleDisplayItems(catalog, [])).toEqual(catalog)
  })
  it('orders by admin configuration, hides disabled entries and appends new entries', () => {
    const configured = [{ ...catalog[4], visible: true }, { ...catalog[1], visible: false }]
    expect(visibleDisplayItems(catalog, configured).map(item => item.id)).toEqual(['snake-arena', 'falling-blocks', 'block-puzzle', 'water-sort', 'ink-blade', 'dusk-glider'])
    expect(catalog[1].visible).toBe(true)
  })
  it('handles all hidden items, stale entries and duplicate entries predictably', () => {
    expect(visibleDisplayItems(catalog, catalog.map(item => ({ ...item, visible: false })))).toEqual([])
    const configured = [{ id: 'old-pack', name: 'old', visible: false }, catalog[4], { ...catalog[4], visible: false }]
    expect(visibleDisplayItems(catalog, configured).map(item => item.id)).toEqual(['snake-arena', 'falling-blocks', 'merge-2048', 'block-puzzle', 'water-sort', 'ink-blade', 'dusk-glider'])
  })
  it('moves up, down and to the top without mutating visibility or the original list', () => {
    const hidden = catalog.map(item => ({ ...item, visible: item.id !== 'snake-arena' }))
    const top = moveDisplayItem(hidden, 4, 0)
    expect(top[0].id).toBe('snake-arena')
    expect(top[0].visible).toBe(false)
    expect(hidden[0].id).toBe('falling-blocks')
    expect(moveDisplayItem(top, 0, 1)[1].id).toBe('snake-arena')
    expect(moveDisplayItem(top, 1, 0)[0].id).toBe('falling-blocks')
    expect(moveDisplayItem(top, -1, 0)).toBe(top)
  })
  it('restores known default order and visibility for games and quizzes', () => {
    const modified = { chatTabVisible: false, games: settings.games.slice().reverse().map(item => ({ ...item, visible: false })), quizzes: settings.quizzes.slice().reverse() }
    const reset = defaultDisplaySettings(modified)
    expect(reset.chatTabVisible).toBe(true)
    expect(reset.games.map(item => item.id)).toEqual(catalog.map(item => item.id))
    expect(reset.quizzes.map(item => item.id)).toEqual(quizCatalog.map(item => item.id))
    expect(reset.games.every(item => item.visible)).toBe(true)
  })
  it('rejects malformed cached/public settings rather than interpreting strings as visibility', () => {
    expect(isDisplaySettings(settings)).toBe(true)
    expect(isDisplaySettings({ ...settings, chatTabVisible: 'false' })).toBe(false)
    expect(isDisplaySettings({ ...settings, games: [catalog[0], catalog[0]] })).toBe(false)
    expect(isDisplaySettings({ ...settings, games: [{ ...catalog[0], visible: 'false' } as unknown as DisplayItem] })).toBe(false)
    expect(isDisplaySettings(null)).toBe(false)
  })
})
