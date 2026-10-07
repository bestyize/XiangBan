import { describe, expect, it, vi } from 'vitest'
import { shareGameToSystem } from './shareAction.ts'

const data = { text: '我拿了 1250 分，你呢？', url: 'https://xiaban.xyz/games/snake-arena/share' }
const file = { name: '战绩.png', type: 'image/png' } as File

describe('game system share', () => {
  it('calls native share immediately with a prepared PNG when supported', async () => {
    const share = vi.fn().mockResolvedValue(undefined)
    const pending = shareGameToSystem({ share, canShare: () => true }, { ...data, file })
    expect(share).toHaveBeenCalledOnce()
    expect(share.mock.calls[0][0]).toMatchObject({ ...data, files: [file] })
    expect(await pending).toBe('shared')
  })
  it('shares a link when file sharing is unsupported or throws', async () => {
    for (const canShare of [() => false, () => { throw new Error('unsupported') }]) {
      const share = vi.fn().mockResolvedValue(undefined)
      expect(await shareGameToSystem({ share, canShare }, { ...data, file })).toBe('shared')
      expect(share.mock.calls[0][0].files).toBeUndefined()
      expect(share.mock.calls[0][0].url).toBe(data.url)
    }
  })
  it('treats user cancellation as a normal cancellation', async () => {
    expect(await shareGameToSystem({ share: async () => { throw { name: 'AbortError' } } }, data)).toBe('cancelled')
  })
  it('returns a fallback without pretending to share on unsupported browsers', async () => {
    expect(await shareGameToSystem({}, data)).toBe('unsupported')
  })
  it('propagates real failures so the UI can offer save and copy alternatives', async () => {
    await expect(shareGameToSystem({ share: async () => { throw new Error('denied') } }, data)).rejects.toThrow('denied')
  })
})
