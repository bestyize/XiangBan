import qrcode from './vendor/qrcode.js'
import { paletteFor, type AccentId } from './accentPalette.ts'
import type { SharePayload } from './detailShare.ts'

export const sharePapers = [
  { id: 'white', label: '纯白', swatch: '#ffffff', paper: '#ffffff', ink: '#1f2024', muted: '#6e7078', soft: '#f4f4f6', line: '#ececef' },
  { id: 'warm', label: '暖纸笺', swatch: '#f3e8d5', paper: '#f8f4ec', ink: '#2b231c', muted: '#7c6f62', soft: '#efe8dc', line: '#d8cbb8' },
  { id: 'ink', label: '墨夜', swatch: '#1d1e24', paper: '#1c1d22', ink: '#f2f2f5', muted: '#989aa4', soft: '#27282e', line: '#55565e' },
] as const
export type SharePaper = typeof sharePapers[number]['id']

interface CardOptions {
  paper: SharePaper
  accent: AccentId
  excerpt: boolean
  qr: boolean
}

const WIDTH = 288
const SCALE = 4
const PAD = 17
const INNER = WIDTH - PAD * 2
const FONT = '"PingFang SC","HarmonyOS Sans SC","Microsoft YaHei",system-ui,sans-serif'

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
    while (last && measure(`${last}…`) > maxWidth) last = Array.from(last).slice(0, -1).join('')
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
    const finish = (image: HTMLImageElement | null) => {
      clearTimeout(timer)
      img.onload = img.onerror = null
      resolve(image)
    }
    const timer = setTimeout(() => finish(null), 5000)
    img.onload = () => finish(img.naturalWidth ? img : null)
    img.onerror = () => finish(null)
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

// 同一张高清 PNG 用于预览、下载、复制与系统分享。
export async function renderPoster(input: SharePayload, options: CardOptions): Promise<HTMLCanvasElement> {
  const paper = sharePapers.find(item => item.id === options.paper) ?? sharePapers[0]
  const brand = paletteFor(options.accent)
  const cover = input.cover ? await loadImage(input.cover) : null
  const canvas = document.createElement('canvas')
  const context = canvas.getContext('2d')
  if (!context) throw new Error('当前浏览器无法生成图片')
  const ctx = context
  const font = (size: number, weight: number) => { ctx.font = `${weight} ${size}px ${FONT}` }
  const wrap = (text: string, width: number, max: number) => wrapText(value => ctx.measureText(value).width, text, width, max)
  font(16, 700)
  const titles = wrap(input.title, INNER, 3)
  font(12, 400)
  const excerpt = options.excerpt && input.excerpt ? wrap(input.excerpt, INNER - 12, 3) : []
  const matrix = options.qr ? qrMatrix(input.url) : null
  let height = PAD + 20 + 11 + titles.length * 23 + 15
  if (cover) height += 113
  if (excerpt.length) height += 9 + excerpt.length * 18.5
  if (input.author) height += 31
  if (matrix) height += 71
  canvas.width = WIDTH * SCALE
  canvas.height = Math.ceil(height * SCALE)
  ctx.scale(SCALE, SCALE)
  ctx.textBaseline = 'middle'

  function rounded(x: number, y: number, w: number, h: number, radius: number) {
    ctx.beginPath()
    ctx.roundRect(x, y, w, h, radius)
  }

  function text(value: string, x: number, y: number, size: number, weight: number, color: string) {
    font(size, weight)
    ctx.fillStyle = color
    ctx.fillText(value, x, y)
  }

  ctx.fillStyle = paper.paper
  ctx.fillRect(0, 0, WIDTH, height)

  let y = PAD
  rounded(PAD, y, 20, 20, 6)
  ctx.fillStyle = brand.btn
  ctx.fill()
  ctx.strokeStyle = '#ffffff'
  ctx.lineWidth = 1.15
  ctx.lineJoin = 'round'
  ctx.beginPath()
  const x = PAD + 10
  const cy = y + 10
  ctx.moveTo(x, cy - 5)
  ctx.lineTo(x + 1.4, cy - 1.4)
  ctx.lineTo(x + 5, cy)
  ctx.lineTo(x + 1.4, cy + 1.4)
  ctx.lineTo(x, cy + 5)
  ctx.lineTo(x - 1.4, cy + 1.4)
  ctx.lineTo(x - 5, cy)
  ctx.lineTo(x - 1.4, cy - 1.4)
  ctx.closePath()
  ctx.stroke()
  text('超级小伴', PAD + 26, y + 10, 12, 650, paper.ink)

  font(10.5, 500)
  const badge = wrap(input.badge, 98, 1)[0] ?? ''
  const badgeWidth = ctx.measureText(badge).width + 16
  const badgeX = WIDTH - PAD - badgeWidth
  rounded(badgeX, y + 1, badgeWidth, 18, 9)
  ctx.fillStyle = paper.soft
  ctx.fill()
  text(badge, badgeX + 8, y + 10, 10.5, 500, paper.muted)
  y += 31

  if (cover) {
    ctx.save()
    rounded(PAD, y, INNER, 102, 10)
    ctx.clip()
    drawCover(ctx, cover, PAD, y, INNER, 102)
    ctx.restore()
    y += 113
  }

  for (const line of titles) {
    text(line, PAD, y + 11.5, 16, 700, paper.ink)
    y += 23
  }

  if (excerpt.length) {
    y += 9
    ctx.fillStyle = paper.id === 'white' ? brand.btn + '55' : paper.line
    ctx.fillRect(PAD, y, 2, excerpt.length * 18.5)
    for (const line of excerpt) {
      text(line, PAD + 12, y + 9.25, 12, 400, paper.muted)
      y += 18.5
    }
  }

  if (input.author) {
    y += 11
    const initial = Array.from(input.author.trim())[0] ?? ''
    rounded(PAD, y, 20, 20, 10)
    ctx.fillStyle = paper.soft
    ctx.fill()
    ctx.textAlign = 'center'
    text(initial, PAD + 10, y + 10, 10, 600, paper.muted)
    ctx.textAlign = 'left'
    const authorX = PAD + 27
    const kind = ' · ' + badge
    font(11.5, 400)
    const kindWidth = ctx.measureText(kind).width
    font(11.5, 600)
    const author = wrap(input.author, INNER - 27 - kindWidth, 1)[0] ?? ''
    const authorWidth = ctx.measureText(author).width
    text(author, authorX, y + 10, 11.5, 600, paper.muted)
    text(kind, authorX + authorWidth, y + 10, 11.5, 400, paper.muted)
    y += 20
  }

  if (matrix) {
    y += 13
    ctx.strokeStyle = paper.line
    ctx.lineWidth = 1
    ctx.setLineDash([3, 3])
    ctx.beginPath()
    ctx.moveTo(PAD, y)
    ctx.lineTo(WIDTH - PAD, y)
    ctx.stroke()
    ctx.setLineDash([])
    y += 12
    text('扫码查看完整内容', PAD, y + 14, 11.5, 600, paper.ink)
    text('长按或微信扫一扫打开小伴', PAD, y + 30, 10.5, 400, paper.muted)

    const quiet = 4
    const cells = matrix.length + quiet * 2
    const cell = Math.max(1, Math.floor(46 * SCALE / cells)) / SCALE
    const qrSize = cell * cells
    const frameX = WIDTH - PAD - 46
    const qx = Math.round((frameX + (46 - qrSize) / 2) * SCALE) / SCALE
    const qy = Math.round((y + (46 - qrSize) / 2) * SCALE) / SCALE
    rounded(frameX, y, 46, 46, 8)
    ctx.fillStyle = '#ffffff'
    ctx.fill()
    ctx.strokeStyle = '#e9e9ec'
    ctx.stroke()
    ctx.fillStyle = '#000000'
    matrix.forEach((row, r) => row.forEach((dark, c) => {
      if (dark) ctx.fillRect(qx + (c + quiet) * cell, qy + (r + quiet) * cell, cell, cell)
    }))
  }
  return canvas
}

export function canvasToBlob(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) => canvas.toBlob(b => (b ? resolve(b) : reject(new Error('生成图片失败'))), 'image/png'))
}
