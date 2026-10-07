import { describe, expect, it } from 'vitest'
import { balancedWrap, wrapText } from './text.ts'

const measure = (s: string) => [...s].length * 10

describe('wrapText', () => {
  it('keeps short text on one line', () => {
    expect(wrapText('路上慢点', 100, 3, measure)).toEqual(['路上慢点'])
  })

  it('breaks by width and keeps punctuation off the start of a line', () => {
    expect(wrapText('今天辛苦啦，路上慢慢走', 50, 3, measure)).toEqual(['今天辛苦啦，', '路上慢慢走'])
  })

  it('ellipsizes when the text runs past the last line', () => {
    expect(wrapText('一二三四五六七八九十', 30, 2, measure)).toEqual(['一二三', '四五…'])
  })

  it('never splits a number or a Latin word', () => {
    expect(wrapText('下次冲 2048', 60, 3, measure)).toEqual(['下次冲', '2048'])
  })
})

describe('balancedWrap', () => {
  it('evens out the lines instead of leaving one character behind', () => {
    expect(wrapText('一口气围住四条蛇太猛了', 100, 3, measure)).toEqual(['一口气围住四条蛇太猛', '了'])
    expect(balancedWrap('一口气围住四条蛇太猛了', 100, 3, measure)).toEqual(['一口气围住四', '条蛇太猛了'])
  })

  it('leaves single lines and ellipsized text alone', () => {
    expect(balancedWrap('路上慢点', 100, 3, measure)).toEqual(['路上慢点'])
    expect(balancedWrap('一二三四五六七八九十', 30, 2, measure)).toEqual(['一二三', '四五…'])
  })
})
