import QRCode from 'qrcode'
import type { Checkin, Profile } from '../api/types.ts'
import { catDataUrl } from '../cat/catSvg.ts'
import { moodFace } from '../cat/moods.ts'
import { formatDay } from '../lib/time.ts'
import { drawDecoration } from './decorations.ts'
import { cardStatLine, type CardStyle } from './styles.ts'
import { wrapText } from './text.ts'

export const CARD_W = 1080
export const CARD_H = 1920
const DISPLAY = '"Xiaban Display", "PingFang SC", sans-serif'
const SANS = '"PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Noto Sans SC", sans-serif'
const INK = '#1e1a2e'
const WINDOW = '#fff1c9'

export interface CardInput {
  checkin: Checkin
  profile: Profile
  style: CardStyle
  showCity: boolean
  /** QR target, carrying from=card and the inviter's code */
  link: string
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = reject
    img.src = src
  })
}

function archPath(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number) {
  ctx.beginPath()
  ctx.moveTo(x, y + h)
  ctx.lineTo(x, y + w / 2)
  ctx.arc(x + w / 2, y + w / 2, w / 2, Math.PI, 0)
  ctx.lineTo(x + w, y + h)
  ctx.closePath()
}

// Vertical budget: title and time to ~600, window 620–1124, bubble from 1170 (≤3 lines, ends by 1490),
// one stat line, then the QR footer from 1600.
async function drawWindow(ctx: CanvasRenderingContext2D, checkin: Checkin) {
  const [x, y, w, h] = [320, 620, 440, 480]
  const glow = ctx.createRadialGradient(x + w / 2, y + h * 0.38, 0, x + w / 2, y + h * 0.38, w * 0.8)
  glow.addColorStop(0, '#ffe9ad')
  glow.addColorStop(0.62, '#ffb938')
  glow.addColorStop(1, '#e89a12')
  ctx.save()
  ctx.shadowColor = 'rgba(255, 185, 56, 0.45)'
  ctx.shadowBlur = 90
  archPath(ctx, x, y, w, h)
  ctx.fillStyle = glow
  ctx.fill()
  ctx.restore()

  ctx.save()
  archPath(ctx, x, y, w, h)
  ctx.clip()
  ctx.fillStyle = 'rgba(31, 39, 88, 0.45)'
  ctx.fillRect(x + w / 2 - 6, y, 12, h)
  ctx.fillRect(x, y + h * 0.44, w, 12)
  const cat = await loadImage(catDataUrl(moodFace[checkin.mood]))
  ctx.drawImage(cat, x + 20, y + h - 390, 400, 400)
  ctx.restore()

  ctx.fillStyle = '#3d4a94'
  ctx.beginPath()
  ctx.roundRect(x - 30, y + h, w + 60, 26, 13)
  ctx.fill()
}

function drawBubble(ctx: CanvasRenderingContext2D, text: string, top: number): number {
  ctx.font = `52px ${SANS}`
  const lines = wrapText(text, 820, 3, (s) => ctx.measureText(s).width)
  const lineHeight = 76
  const height = lines.length * lineHeight + 92
  ctx.fillStyle = WINDOW
  ctx.beginPath()
  ctx.roundRect(90, top, 900, height, 44)
  ctx.moveTo(512, top)
  ctx.lineTo(540, top - 30)
  ctx.lineTo(568, top)
  ctx.fill()
  ctx.fillStyle = INK
  ctx.textAlign = 'center'
  lines.forEach((line, i) => ctx.fillText(line, 540, top + 88 + i * lineHeight))
  return top + height
}

async function drawFooter(ctx: CanvasRenderingContext2D, style: CardStyle, link: string) {
  const qr = document.createElement('canvas')
  await QRCode.toCanvas(qr, link, { margin: 0, width: 220, errorCorrectionLevel: 'M', color: { dark: INK, light: '#ffffff' } })
  ctx.fillStyle = '#ffffff'
  ctx.beginPath()
  ctx.roundRect(90, 1600, 256, 256, 28)
  ctx.fill()
  ctx.drawImage(qr, 108, 1618, 220, 220)

  ctx.textAlign = 'left'
  ctx.fillStyle = style.accent
  ctx.font = `80px ${DISPLAY}`
  ctx.fillText('你下班了么？', 396, 1700)
  ctx.fillStyle = WINDOW
  ctx.font = `46px ${SANS}`
  ctx.fillText('xiaban.xyz', 398, 1772)
  ctx.fillStyle = 'rgba(255, 241, 201, 0.6)'
  ctx.font = `34px ${SANS}`
  ctx.fillText('扫码，让小伴也陪你下班', 398, 1830)
}

/** Draws the 1080×1920 card on the device (PRD 5.7) and returns it as a PNG. */
export async function renderCard({ checkin, profile, style, showCity, link }: CardInput): Promise<Blob> {
  await Promise.all([document.fonts.load(`120px ${DISPLAY}`, checkin.title.name + checkin.localTime), document.fonts.ready])
  const canvas = document.createElement('canvas')
  canvas.width = CARD_W
  canvas.height = CARD_H
  const ctx = canvas.getContext('2d')!

  const sky = ctx.createLinearGradient(0, 0, 0, CARD_H)
  sky.addColorStop(0, style.sky[0])
  sky.addColorStop(1, style.sky[1])
  ctx.fillStyle = sky
  ctx.fillRect(0, 0, CARD_W, CARD_H)
  drawDecoration(ctx, style.decoration, Number(checkin.id) || 1)

  ctx.textAlign = 'left'
  ctx.fillStyle = style.accent
  ctx.font = `64px ${DISPLAY}`
  ctx.fillText('下班了么', 90, 170)
  const date = formatDay(new Date(checkin.checkedAt))
  const city = showCity && profile.cityName && profile.cityCode !== 'other' ? `${profile.cityName}  ` : ''
  ctx.textAlign = 'right'
  ctx.fillStyle = 'rgba(255, 241, 201, 0.75)'
  ctx.font = `40px ${SANS}`
  ctx.fillText(city + date, CARD_W - 90, 162)

  const title = checkin.title.name
  ctx.save()
  ctx.translate(90, 430)
  ctx.rotate(-0.035)
  ctx.textAlign = 'left'
  ctx.fillStyle = style.accent
  ctx.font = `${Math.min(220, Math.floor(900 / [...title].length))}px ${DISPLAY}`
  ctx.fillText(title, 0, 0)
  ctx.restore()

  ctx.textAlign = 'left'
  ctx.fillStyle = WINDOW
  ctx.font = `120px ${DISPLAY}`
  ctx.fillText(checkin.localTime, 90, 580)
  const timeWidth = ctx.measureText(checkin.localTime).width
  ctx.font = `52px ${SANS}`
  ctx.fillStyle = 'rgba(255, 241, 201, 0.8)'
  ctx.fillText('下班', 90 + timeWidth + 24, 576)

  await drawWindow(ctx, checkin)
  const bubbleBottom = drawBubble(ctx, checkin.reply.text, 1170)

  ctx.textAlign = 'center'
  ctx.fillStyle = WINDOW
  ctx.font = `46px ${SANS}`
  ctx.fillText(cardStatLine(checkin, profile.companionName), 540, Math.min(bubbleBottom + 90, 1560))

  await drawFooter(ctx, style, link)
  return new Promise((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('生成图片失败'))), 'image/png'))
}
