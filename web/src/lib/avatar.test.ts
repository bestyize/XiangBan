import { afterEach, describe, expect, it, vi } from 'vitest'
import { compressAvatar, MAX_AVATAR_BYTES, uploadAvatar } from './avatar.ts'
vi.mock('../api/client.ts', () => ({ getToken: () => 'test-token', ApiError: class extends Error {} }))
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks() })
describe('avatar compression and upload limits', () => {
  it('reduces quality before returning a bounded JPEG and releases the original URL', async () => {
    const revoke = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {})
    vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:avatar-test')
    let attempts = 0
    vi.stubGlobal('Image', class { src = ''; naturalWidth = 1200; naturalHeight = 800; decode() { return Promise.resolve() } })
    vi.stubGlobal('document', { createElement: () => ({ width: 0, height: 0,
      getContext: () => ({ fillStyle: '', fillRect() {}, drawImage() {} }),
      toBlob: (callback: (blob: Blob) => void) => { attempts += 1; callback(new Blob([new Uint8Array(attempts === 1 ? 70000 : MAX_AVATAR_BYTES)], { type: 'image/jpeg' })) },
    }) })
    const result = await compressAvatar(new File(['sample'], 'avatar.png', { type: 'image/png' }))
    expect(result.size).toBe(MAX_AVATAR_BYTES); expect(result.type).toBe('image/jpeg'); expect(attempts).toBe(2)
    expect(revoke).toHaveBeenCalledWith('blob:avatar-test')
  })
  it('never uploads an oversized or uncompressed source', async () => {
    const fetcher = vi.fn(); vi.stubGlobal('fetch', fetcher)
    await expect(uploadAvatar(new Blob([new Uint8Array(MAX_AVATAR_BYTES + 1)], { type: 'image/jpeg' }))).rejects.toThrow('64KB')
    await expect(uploadAvatar(new Blob(['png'], { type: 'image/png' }))).rejects.toThrow('JPEG')
    expect(fetcher).not.toHaveBeenCalled()
  })
  it('sends only the compressed JPEG blob, with the current session', async () => {
    const fetcher = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ userId: 1 }) }); vi.stubGlobal('fetch', fetcher)
    const compressed = new Blob(['jpeg'], { type: 'image/jpeg' })
    await uploadAvatar(compressed)
    const [, options] = fetcher.mock.calls[0]
    expect(options.body).toBe(compressed); expect(options.headers.Authorization).toBe('Bearer test-token')
  })
})
