import { COVER_LIMIT, fmtSize } from './content'

const WEBP = 'image/webp'
const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
const MAX_EDGE = 2048
const WEBP_QUALITY = 88

async function coverMime(file: File): Promise<string> {
  const header = new Uint8Array(await file.slice(0, 12).arrayBuffer())
  if (PNG_SIGNATURE.every((byte, i) => header[i] === byte)) return 'image/png'
  if (header[0] === 0xff && header[1] === 0xd8 && header[2] === 0xff) return 'image/jpeg'
  const signature = String.fromCharCode(...header)
  if (signature.startsWith('RIFF') && signature.slice(8) === 'WEBP') return WEBP
  throw new Error('请选择有效的 PNG、JPG 或 WebP 封面')
}

/** Prepare the cover before requesting a ticket, so the hash and size describe the WebP bytes. */
export async function preparePublishCover(file: File, signal: AbortSignal): Promise<File> {
  const mime = await coverMime(file)
  signal.throwIfAborted()
  const source = file.slice(0, file.size, mime)
  const name = `${file.name.replace(/\.[^.]+$/, '') || 'cover'}.webp`

  let image: ImageBitmap
  try {
    image = await createImageBitmap(source)
  } catch {
    throw new Error('无法读取封面图片，请重新选择')
  }

  try {
    signal.throwIfAborted()
    if (mime === WEBP && source.size <= COVER_LIMIT) return new File([source], name, { type: WEBP })

    const canvas = document.createElement('canvas')
    const context = canvas.getContext('2d')
    if (!context) throw new Error('当前浏览器无法转换封面图片')
    let scale = Math.min(1, MAX_EDGE / Math.max(image.width, image.height))
    let useNativeEncoder = true
    for (let attempt = 0; attempt < 3; attempt++) {
      canvas.width = Math.max(1, Math.round(image.width * scale))
      canvas.height = Math.max(1, Math.round(image.height * scale))
      context.imageSmoothingQuality = 'high'
      context.drawImage(image, 0, 0, canvas.width, canvas.height)
      let webp = useNativeEncoder
        ? await new Promise<Blob | null>(resolve => canvas.toBlob(resolve, WEBP, WEBP_QUALITY / 100))
        : null
      signal.throwIfAborted()
      // Some browsers return PNG when their canvas encoder does not support WebP.
      if (webp?.type !== WEBP) {
        useNativeEncoder = false
        try {
          const { default: encode } = await import('@jsquash/webp/encode.js')
          signal.throwIfAborted()
          const pixels = context.getImageData(0, 0, canvas.width, canvas.height)
          webp = new Blob([await encode(pixels, { quality: WEBP_QUALITY })], { type: WEBP })
        } catch {
          signal.throwIfAborted()
          throw new Error('无法生成 WebP 封面，请重试')
        }
      }
      signal.throwIfAborted()
      if (webp.size <= COVER_LIMIT) return new File([webp], name, { type: WEBP })
      scale *= 0.75
    }
    throw new Error(`封面转换后不能超过 ${fmtSize(COVER_LIMIT)}，请换一张图片`)
  } finally {
    image.close()
  }
}
