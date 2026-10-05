interface SharePlatform {
  share?: (data: ShareData) => Promise<void>
  canShare?: (data: ShareData) => boolean
}

/** Called directly from a tap, using an already prepared file; never waits for image generation. */
export async function shareGameToSystem(platform: SharePlatform, data: { text: string; url: string; file?: File }): Promise<'shared' | 'cancelled' | 'unsupported'> {
  if (typeof platform.share !== 'function') return 'unsupported'
  let files: File[] | undefined
  try { if (data.file && platform.canShare?.({ files: [data.file] })) files = [data.file] } catch { /* Fall back to text and URL. */ }
  try {
    await platform.share({ title: '小伴贪吃蛇 · 快乐战绩', text: data.text, url: data.url, ...(files ? { files } : {}) })
    return 'shared'
  } catch (cause) {
    if (cause && typeof cause === 'object' && 'name' in cause && cause.name === 'AbortError') return 'cancelled'
    throw cause
  }
}
