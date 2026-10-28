// 分享海报：在浏览器里用 canvas 画一张竖版图片（封面、标题、摘要、二维码、品牌），不经过服务器。
import qrcode from './vendor/qrcode.js'

export interface PosterInput {
  /** 左上角的小标签，如“特效”“周刊” */
  tag: string
  title: string
  summary: string
  /** 封面图地址；加载失败或不给就画渐变底 */
  cover?: string | null
  /** 二维码指向的网址 */
  url: string
  /** 渐变底色的种子，同一内容每次画出来一样 */
  seed?: number
}

const W = 750
const H = 1180
const PAD = 56
const GRADIENTS = [
  ['#2b2b2e', '#5a5a60'],
  ['#3a3a3f', '#8d8d93'],
  ['#1f2933', '#52606d'],
  ['#44403c', '#a8a29e'],
  ['#262626', '#737373'],
]

/** 把文字按宽度折行，最多 maxLines 行，超出的部分以省略号结尾。中英文都逐字量宽，海报用足够了。 */
export function wrapText(measure: (s: string) => number, text: string, maxWidth: number, maxLines: number): string[] {
  const chars = Array.from(text.replace(/\s+/g, ' ').trim())
  const lines: string[] = []
  let line = ''
  let truncated = false
  for (const ch of chars) {
    if (measure(line + ch) > maxWidth && line) {
      lines.push(line)
      if (lines.length === maxLines) {
        truncated = true
        break
      }
      line = ch === ' ' ? '' : ch
    } else {
      line += ch
    }
  }
  if (!truncated && line) lines.push(line)
  if (truncated) {
    let last = lines[lines.length - 1]!
    while (last && measure(`${last}…`) > maxWidth) last = last.slice(0, -1)
    lines[lines.length - 1] = `${last}…`
  }
  return lines
}

/** 二维码的模块矩阵（true 为深色）。网址一般很短，取能装下的最小版本，纠错等级 M。 */
export function qrMatrix(text: string): boolean[][] {
  const qr = qrcode(0, 'M')
  qr.addData(text)
  qr.make()
  const n = qr.getModuleCount()
  return Array.from({ length: n }, (_, r) => Array.from({ length: n }, (_, c) => qr.isDark(r, c)))
}

function loadImage(src: string): Promise<HTMLImageElement | null> {
  return new Promise(resolve => {
    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.onload = () => resolve(img)
    img.onerror = () => resolve(null)
    img.src = src
  })
}

function drawCover(ctx: CanvasRenderingContext2D, img: HTMLImageElement, x: number, y: number, w: number, h: number) {
  // 居中裁切铺满
  const scale = Math.max(w / img.width, h / img.height)
  const sw = w / scale
  const sh = h / scale
  ctx.drawImage(img, (img.width - sw) / 2, (img.height - sh) / 2, sw, sh, x, y, w, h)
}

export async function renderPoster(input: PosterInput): Promise<HTMLCanvasElement> {
  const canvas = document.createElement('canvas')
  canvas.width = W
  canvas.height = H
  const ctx = canvas.getContext('2d')!
  const font = '"PingFang SC","HarmonyOS Sans SC","Microsoft YaHei",system-ui,sans-serif'

  ctx.fillStyle = '#ffffff'
  ctx.fillRect(0, 0, W, H)

  // 封面区
  const coverH = 520
  const img = input.cover ? await loadImage(input.cover) : null
  if (img) {
    drawCover(ctx, img, 0, 0, W, coverH)
  } else {
    const [a, b] = GRADIENTS[(input.seed ?? 0) % GRADIENTS.length]!
    const g = ctx.createLinearGradient(0, 0, W, coverH)
    g.addColorStop(0, a)
    g.addColorStop(1, b)
    ctx.fillStyle = g
    ctx.fillRect(0, 0, W, coverH)
  }
  ctx.fillStyle = 'rgba(0,0,0,0.45)'
  ctx.font = `500 26px ${font}`
  const tagW = ctx.measureText(input.tag).width + 28
  ctx.fillRect(PAD, coverH - 66, tagW, 42)
  ctx.fillStyle = '#ffffff'
  ctx.fillText(input.tag, PAD + 14, coverH - 36)

  // 标题与摘要
  ctx.fillStyle = '#222222'
  ctx.font = `600 44px ${font}`
  let y = coverH + 80
  for (const line of wrapText(s => ctx.measureText(s).width, input.title, W - PAD * 2, 2)) {
    ctx.fillText(line, PAD, y)
    y += 60
  }
  ctx.fillStyle = '#666666'
  ctx.font = `400 28px ${font}`
  y += 4
  for (const line of wrapText(s => ctx.measureText(s).width, input.summary, W - PAD * 2, 3)) {
    ctx.fillText(line, PAD, y)
    y += 42
  }

  // 底部：品牌与二维码
  const footY = H - 210
  ctx.fillStyle = '#ebebeb'
  ctx.fillRect(PAD, footY - 24, W - PAD * 2, 2)
  const matrix = qrMatrix(input.url)
  const quiet = 2
  const cells = matrix.length + quiet * 2
  const size = 150
  const cell = Math.floor(size / cells)
  const qrSize = cell * cells
  const qx = W - PAD - qrSize
  const qy = footY + 4
  ctx.fillStyle = '#ffffff'
  ctx.fillRect(qx, qy, qrSize, qrSize)
  ctx.fillStyle = '#000000'
  matrix.forEach((row, r) => row.forEach((dark, c) => dark && ctx.fillRect(qx + (c + quiet) * cell, qy + (r + quiet) * cell, cell, cell)))

  ctx.fillStyle = '#ff2442'
  ctx.font = `700 40px ${font}`
  ctx.fillText('超级小伴', PAD, footY + 64)
  ctx.fillStyle = '#999999'
  ctx.font = `400 24px ${font}`
  ctx.fillText('扫码查看，在电脑的小伴里一键应用', PAD, footY + 108)
  ctx.fillText(new URL(input.url).host, PAD, footY + 146)
  return canvas
}

export function canvasToBlob(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) => canvas.toBlob(b => (b ? resolve(b) : reject(new Error('生成图片失败'))), 'image/png'))
}
