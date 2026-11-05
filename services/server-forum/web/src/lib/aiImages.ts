export interface ImageCapability { available: boolean; requires_paid?: boolean; sizes: string[]; formats?: string[] }
export interface ImageOptions { prompt: string; n: number; size?: string; output_format: 'png' | 'gif'; frame_count?: number; frame_duration_ms?: number }
export const UNKNOWN_IMAGE_RESULT = '生图结果未知，可能已生成并扣费，请先核对结果，不要直接重复生成。'
const MAX_IMAGE = 16 * 1024 * 1024

function validGif(bytes: Uint8Array): boolean {
  const number = (p:number) => bytes[p]! | bytes[p+1]! << 8
  const w = number(6), h = number(8)
  if (!w || !h || w > 8192 || h > 8192) return false
  let p = 13 + (bytes[10]! & 128 ? 3 * (1 << ((bytes[10]! & 7) + 1)) : 0), frames = 0
  const blocks = () => {
    while (p < bytes.length) {const n = bytes[p++]!; if (!n) return true; p += n}
    return false
  }
  while (p < bytes.length) {
    const type = bytes[p++]
    if (type === 0x3b) return frames >= 2
    if (type === 0x21) {p++; if (!blocks()) return false}
    else if (type === 0x2c) {
      if (p + 9 > bytes.length || !number(p+4) || !number(p+6) || number(p)+number(p+4) > w || number(p+2)+number(p+6) > h) return false
      if (++frames > 16 || frames * w * h > 32 * 1024 * 1024) return false
      const packed = bytes[p+8]!; p += 9
      if (packed & 128) p += 3 * (1 << ((packed & 7) + 1))
      if (p >= bytes.length || bytes[p]! < 2 || bytes[p]! > 8) return false
      p++; if (!blocks()) return false
    } else return false
  }
  return false
}

export function imagePayload(options: ImageOptions, capability: ImageCapability): Record<string, unknown> {
  const prompt = options.prompt.trim()
  if (!capability.available) throw Error(capability.requires_paid ? '图片生成需要先兑换付费额度。' : '图片生成暂未开放。')
  if ([...prompt].length < 1 || [...prompt].length > 4000) throw Error('图片描述需要 1 到 4000 字。')
  if (!Number.isInteger(options.n) || options.n < 1 || options.n > 4) throw Error('每次可生成 1 到 4 张图片。')
  const size = options.size || capability.sizes[0]
  if (!size || !capability.sizes.includes(size)) throw Error('请选择已开放的图片尺寸。')
  const payload: Record<string, unknown> = {prompt, n:options.n, size}
  if (options.output_format === 'gif') {
    if (!capability.formats?.includes('gif')) throw Error('当前图片服务尚未开放 GIF。')
    const frames = options.frame_count ?? 9, duration = options.frame_duration_ms ?? 200
    if (options.n !== 1 || !Number.isInteger(frames) || frames < 2 || frames > 16 || !Number.isInteger(duration) || duration < 20 || duration > 2000) throw Error('GIF 参数无效。')
    Object.assign(payload, {output_format:'gif', frame_count:frames, frame_duration_ms:duration})
  } else if (options.output_format !== 'png' || options.frame_count !== undefined || options.frame_duration_ms !== undefined) throw Error('图片格式或动画参数无效。')
  return payload
}

export async function boundedImageResponse(response: Response, limit = 48 * 1024 * 1024): Promise<Uint8Array> {
  if (!response.body || Number(response.headers.get('content-length')) > limit) throw Error('图片响应过大或无法读取。')
  const reader = response.body.getReader(), chunks: Uint8Array[] = []
  let length = 0
  try {
    while (true) {
      const {done, value} = await reader.read()
      if (done) break
      length += value.length
      if (length > limit) throw Error('图片响应过大。')
      chunks.push(value)
    }
  } finally { void reader.cancel().catch(() => {}); reader.releaseLock() }
  const bytes = new Uint8Array(length)
  let offset = 0
  for (const chunk of chunks) {bytes.set(chunk, offset); offset += chunk.length}
  return bytes
}

export function generatedImageBlob(bytes: Uint8Array): Blob {
  if (!bytes.length || bytes.length > MAX_IMAGE) throw Error('单张图片超过 16 MB 或内容为空。')
  const text = (start: number, end: number) => String.fromCharCode(...bytes.subarray(start, end))
  let mime = ''
  if (bytes.length >= 8 && [137,80,78,71,13,10,26,10].every((b,i) => bytes[i] === b)) mime = 'image/png'
  else if (bytes[0] === 255 && bytes[1] === 216 && bytes[2] === 255) mime = 'image/jpeg'
  else if (text(0,4) === 'RIFF' && text(8,12) === 'WEBP') mime = 'image/webp'
  else if (bytes.length >= 13 && ['GIF87a','GIF89a'].includes(text(0,6))) {
    if (!validGif(bytes)) throw Error('GIF 动画不完整或超过帧数、像素限制。')
    mime = 'image/gif'
  }
  if (!mime) throw Error('图片格式无法识别。')
  return new Blob([new Uint8Array(bytes)], {type:mime})
}

export async function generatedImageItems(value: unknown, count: number, signal: AbortSignal): Promise<Blob[]> {
  if (!value || typeof value !== 'object' || !('data' in value) || !Array.isArray(value.data) || !value.data.length || value.data.length > count) throw Error(UNKNOWN_IMAGE_RESULT)
  const images: Blob[] = []
  for (const item of value.data) {
    try {
      let bytes: Uint8Array
      if (typeof item?.b64_json === 'string') {
        if (item.b64_json.length > Math.ceil(MAX_IMAGE / 3) * 4) throw Error('图片过大')
        bytes = Uint8Array.from(atob(item.b64_json), c => c.charCodeAt(0))
      } else {
        const url = new URL(item?.url)
        if (url.protocol !== 'https:' || url.username || url.password) throw Error('图片地址无效')
        // Never send the account bearer or cookies to an image host, and never follow redirects.
        const response = await fetch(url, {signal, credentials:'omit', redirect:'error', referrerPolicy:'no-referrer'})
        if (!response.ok) throw Error('图片下载失败')
        bytes = await boundedImageResponse(response, MAX_IMAGE)
      }
      images.push(generatedImageBlob(bytes))
    } catch (error) {if (signal.aborted) throw error}
  }
  if (!images.length) throw Error(UNKNOWN_IMAGE_RESULT)
  return images
}
