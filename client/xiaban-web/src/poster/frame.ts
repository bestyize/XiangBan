import QRCode from 'qrcode'
import { balancedWrap } from '../card/text.ts'
import { stillFrame, type RobotInput } from '../robot/expressions.ts'
import { robotDataUrl } from '../robot/render.ts'
import { VIEW } from '../robot/geometry.ts'
import { POSTER_THEMES, type PosterTheme, type PosterThemeId } from './theme.ts'

/**
 * Every share poster is laid out on a 360-unit-wide page (a phone screen) and rendered at 3× for a 1080-px image.
 * The frame is shared: background, the 相伴 header, 小伴 with a speech bubble, content panels and the QR footer.
 */
export const POSTER_SCALE = 3
export const PAGE_W = 360
export const POSTER_W = PAGE_W * POSTER_SCALE
export const MARGIN = 20
export const FOOTER_H = 92

export const SANS = '"PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Noto Sans SC", system-ui, sans-serif'
/** Display face (a subset: brand words, check-in titles, digits). Anything else falls back to the sans face. */
export const DISPLAY = '"Xiaban Display", "PingFang SC", sans-serif'

export interface Poster {
  canvas: HTMLCanvasElement
  ctx: CanvasRenderingContext2D
  theme: PosterTheme
  /** Page height in units */
  h: number
}

export function createPoster(heightUnits: number, theme: PosterThemeId | PosterTheme, failure: string): Poster {
  const canvas = document.createElement('canvas')
  canvas.width = POSTER_W
  canvas.height = Math.round(heightUnits * POSTER_SCALE)
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error(failure)
  ctx.scale(POSTER_SCALE, POSTER_SCALE)
  return { canvas, ctx, theme: typeof theme === 'string' ? POSTER_THEMES[theme] : theme, h: heightUnits }
}

export async function prepareFonts(displayText = '') {
  if (!document.fonts) return
  await Promise.all([document.fonts.load(`40px ${DISPLAY}`, `相伴${displayText}0123456789`), document.fonts.ready]).catch(() => undefined)
}

export function setFont(ctx: CanvasRenderingContext2D, size: number, weight = 400, family = SANS) {
  ctx.font = `${weight} ${size}px ${family}`
}

/** Shrinks the font until the text fits, then draws it. */
export function fitText(ctx: CanvasRenderingContext2D, text: string, x: number, y: number, size: number, maxWidth: number, weight = 600, family = SANS) {
  setFont(ctx, size, weight, family)
  while (size > 8 && ctx.measureText(text).width > maxWidth) setFont(ctx, --size, weight, family)
  ctx.fillText(text, x, y)
  return size
}

export function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number, fill: string) {
  ctx.fillStyle = fill
  ctx.beginPath()
  ctx.roundRect(x, y, w, h, r)
  ctx.fill()
}

export function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('image'))
    img.src = src
  })
}

/** Deterministic so the same result always gets the same sky. */
function seeded(seed: number) {
  let s = (Math.abs(Math.floor(seed)) * 9301 + 49297) % 233280 || 1
  return () => {
    s = (s * 9301 + 49297) % 233280
    return s / 233280
  }
}

/** Stars stay in the sky band above the content (starsTo), never behind text. */
export function drawBackground(p: Poster, options: { bg?: [string, string]; seed?: number; stars?: boolean; starsTo?: number } = {}) {
  const { ctx, theme, h } = p
  const [top, bottom] = options.bg ?? theme.bg
  const sky = ctx.createLinearGradient(0, 0, 0, h)
  sky.addColorStop(0, top)
  sky.addColorStop(1, bottom)
  ctx.fillStyle = sky
  ctx.fillRect(0, 0, PAGE_W, h)

  // A soft brand glow in the upper part, the same on every poster.
  const glow = ctx.createRadialGradient(PAGE_W / 2, 150, 0, PAGE_W / 2, 150, 260)
  glow.addColorStop(0, theme.glow)
  glow.addColorStop(1, 'rgba(229, 27, 62, 0)')
  ctx.fillStyle = glow
  ctx.fillRect(0, 0, PAGE_W, h)

  if (options.stars ?? theme.stars) {
    const rand = seeded(options.seed ?? 7)
    for (let i = 0; i < 26; i++) {
      ctx.fillStyle = `rgba(255, 241, 201, ${(0.18 + rand() * 0.4).toFixed(2)})`
      ctx.beginPath()
      ctx.arc(rand() * PAGE_W, 50 + rand() * ((options.starsTo ?? h * 0.42) - 50), 0.6 + rand() * 1.1, 0, Math.PI * 2)
      ctx.fill()
    }
  }
}

async function robotImage(spec: RobotInput, widthUnits: number) {
  return loadImage(robotDataUrl(stillFrame(spec), Math.round(widthUnits * POSTER_SCALE)))
}

/** 小伴 under its own soft light. Returns its bounding box. */
export async function drawRobot(p: Poster, spec: RobotInput, cx: number, top: number, width: number) {
  const { ctx, theme } = p
  const height = (width * VIEW.h) / VIEW.w
  const cy = top + height * 0.45
  const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, width * 0.78)
  glow.addColorStop(0, theme.glow)
  glow.addColorStop(1, 'rgba(229, 27, 62, 0)')
  ctx.fillStyle = glow
  ctx.fillRect(cx - width, cy - width, width * 2, width * 2)
  ctx.drawImage(await robotImage(spec, width), cx - width / 2, top, width, height)
  return { x: cx - width / 2, y: top, w: width, h: height }
}

/** Brand row: a small 小伴 head, the 相伴 wordmark and what this poster is about. */
export async function drawHeader(p: Poster, kicker: string) {
  const { ctx, theme } = p
  ctx.drawImage(await robotImage('happy', 34), MARGIN - 2, 20, 34, 29.75)
  ctx.textAlign = 'left'
  ctx.fillStyle = theme.accent
  setFont(ctx, 19, 400, DISPLAY)
  ctx.fillText('相伴', MARGIN + 37, 42)
  ctx.textAlign = 'right'
  ctx.fillStyle = theme.sub
  fitText(ctx, kicker, PAGE_W - MARGIN, 40, 10, 200, 500)
}

export interface BubbleOptions {
  x: number
  y: number
  w: number
  size?: number
  weight?: number
  lineHeight?: number
  maxLines?: number
  align?: 'center' | 'left'
  /** Where the tail points: up at a centered 小伴, or left at one beside the bubble */
  tail: { side: 'up'; x: number } | { side: 'left'; y: number }
  /** Vertically center the bubble on this y instead of starting at options.y */
  centerY?: number
}

/** 小伴's speech bubble. Returns its bottom edge. */
export function drawBubble(p: Poster, text: string, o: BubbleOptions) {
  const { ctx, theme } = p
  const size = o.size ?? 16
  const lineHeight = o.lineHeight ?? Math.round(size * 1.5)
  const padX = Math.round(size * 1.05)
  const padY = Math.round(size * 0.8)
  setFont(ctx, size, o.weight ?? 500)
  const lines = balancedWrap(text, o.w - padX * 2, o.maxLines ?? 3, (s) => ctx.measureText(s).width)
  const height = lines.length * lineHeight + padY * 2
  const y = o.centerY !== undefined ? o.centerY - height / 2 : o.y
  const r = Math.min(18, height / 2)

  ctx.save()
  ctx.shadowColor = theme.id === 'night' ? 'rgba(0, 0, 0, 0.25)' : 'rgba(0, 0, 0, 0.06)'
  ctx.shadowBlur = 14
  ctx.shadowOffsetY = 4
  ctx.fillStyle = theme.bubble
  ctx.beginPath()
  ctx.roundRect(o.x, y, o.w, height, r)
  if (o.tail.side === 'up') {
    ctx.moveTo(o.tail.x - 9, y + 0.5)
    ctx.lineTo(o.tail.x, y - 9)
    ctx.lineTo(o.tail.x + 9, y + 0.5)
  } else {
    const ty = Math.min(Math.max(o.tail.y, y + r), y + height - r)
    ctx.moveTo(o.x + 0.5, ty - 8)
    ctx.lineTo(o.x - 9, ty)
    ctx.lineTo(o.x + 0.5, ty + 8)
  }
  ctx.fill()
  ctx.restore()

  ctx.fillStyle = theme.bubbleInk
  setFont(ctx, size, o.weight ?? 500)
  ctx.textAlign = o.align ?? 'center'
  const tx = (o.align ?? 'center') === 'center' ? o.x + o.w / 2 : o.x + padX
  // Baselines sit ~0.35em below each line's middle.
  lines.forEach((line, i) => ctx.fillText(line, tx, y + padY + lineHeight * i + lineHeight / 2 + size * 0.35))
  return y + height
}

export function drawPanel(p: Poster, x: number, y: number, w: number, h: number, r = 18) {
  const { ctx, theme } = p
  ctx.save()
  if (theme.id === 'day') {
    ctx.shadowColor = 'rgba(0, 0, 0, 0.05)'
    ctx.shadowBlur = 18
    ctx.shadowOffsetY = 6
  }
  roundRect(ctx, x, y, w, h, r, theme.panel)
  ctx.restore()
  ctx.strokeStyle = theme.panelLine
  ctx.lineWidth = 0.8
  ctx.beginPath()
  ctx.roundRect(x + 0.4, y + 0.4, w - 0.8, h - 0.8, r)
  ctx.stroke()
}

/** A stat chip: big value over a small label. */
export function drawStat(p: Poster, x: number, y: number, w: number, value: string, label: string) {
  const { ctx, theme } = p
  roundRect(ctx, x, y, w, 46, 12, theme.chip)
  ctx.textAlign = 'center'
  ctx.fillStyle = theme.ink
  fitText(ctx, value, x + w / 2, y + 22, 17, w - 12, 700)
  ctx.fillStyle = theme.sub
  setFont(ctx, 9, 500)
  ctx.fillText(label, x + w / 2, y + 37)
}

export function linkHost(link: string) {
  try {
    return new URL(link).host
  } catch {
    return 'xiaban.xyz'
  }
}

/** The same bottom bar on every poster: a call to action, the brand, and the QR code. Returns its top edge. */
export async function drawFooter(p: Poster, { headline, sub, link, bottom = p.h - MARGIN }: { headline: string; sub: string; link: string; bottom?: number }) {
  const { ctx, theme } = p
  const top = bottom - FOOTER_H
  drawPanel(p, MARGIN, top, PAGE_W - MARGIN * 2, FOOTER_H, 18)

  const qrSize = 72
  const qrX = PAGE_W - MARGIN - 10 - qrSize
  const qrY = top + 10
  roundRect(ctx, qrX, qrY, qrSize, qrSize, 10, '#ffffff')
  if (theme.id === 'day') {
    ctx.strokeStyle = theme.panelLine
    ctx.lineWidth = 0.8
    ctx.beginPath()
    ctx.roundRect(qrX + 0.4, qrY + 0.4, qrSize - 0.8, qrSize - 0.8, 10)
    ctx.stroke()
  }
  const qr = document.createElement('canvas')
  const px = (qrSize - 10) * POSTER_SCALE
  await QRCode.toCanvas(qr, link, { margin: 0, width: px, errorCorrectionLevel: 'M', color: { dark: theme.qrInk, light: '#ffffff' } })
  ctx.imageSmoothingEnabled = false
  ctx.drawImage(qr, qrX + 5, qrY + 5, qrSize - 10, qrSize - 10)
  ctx.imageSmoothingEnabled = true

  const textW = qrX - MARGIN - 28
  ctx.textAlign = 'left'
  ctx.fillStyle = theme.ink
  fitText(ctx, headline, MARGIN + 16, top + 31, 16, textW, 700)
  ctx.fillStyle = theme.sub
  fitText(ctx, sub, MARGIN + 16, top + 51, 10.5, textW, 400)
  ctx.fillStyle = theme.accent
  setFont(ctx, 13, 400, DISPLAY)
  ctx.fillText('相伴', MARGIN + 16, top + 76)
  const brandW = ctx.measureText('相伴').width
  ctx.fillStyle = theme.faint
  setFont(ctx, 9.5, 500)
  ctx.fillText(`· ${linkHost(link)}`, MARGIN + 20 + brandW, top + 75)
  return top
}

export function toBlob(p: Poster, failure: string): Promise<Blob> {
  return new Promise((resolve, reject) => p.canvas.toBlob((b) => (b ? resolve(b) : reject(new Error(failure))), 'image/png'))
}
