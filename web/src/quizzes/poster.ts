import QRCode from 'qrcode'
import { personaArtSvg, quizPalette } from './art.ts'
import type { QuizDefinition, QuizReport } from './types.ts'

export const POSTER_WIDTH = 1080
export const POSTER_HEIGHT = 2340
const FONT = '"PingFang SC", "Microsoft YaHei", "Noto Sans SC", sans-serif'

function font(ctx: CanvasRenderingContext2D, size: number, weight = 400) {
  ctx.font = weight + ' ' + size + 'px ' + FONT
}

function rounded(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath()
  ctx.roundRect(x, y, w, h, r)
  ctx.fill()
}

function fitFont(ctx: CanvasRenderingContext2D, text: string, size: number, width: number, weight: number) {
  font(ctx, size, weight)
  while (size > 12 && ctx.measureText(text).width > width) font(ctx, --size, weight)
}

function textLines(ctx: CanvasRenderingContext2D, text: string, x: number, y: number, width: number, lineHeight: number, maxLines = 2) {
  const lines: string[] = []
  let line = ''
  for (const char of text) {
    if (ctx.measureText(line + char).width > width && line) {
      lines.push(line)
      line = char
    } else line += char
  }
  if (line) lines.push(line)
  // Balance a short two-line quote instead of leaving a few lonely characters.
  if (lines.length === 2) {
    const chars = Array.from(text)
    let best = Infinity
    for (let i = 1; i < chars.length; i++) {
      const left = chars.slice(0, i).join('')
      const right = chars.slice(i).join('')
      const a = ctx.measureText(left).width
      const b = ctx.measureText(right).width
      if (a > width || b > width) continue
      const cost = Math.abs(a - b) + (/[，。；！？]/.test(chars[i - 1]) ? 0 : width * 0.15)
      if (cost < best) {
        best = cost
        lines[0] = left
        lines[1] = right
      }
    }
  }
  if (lines.length > maxLines) {
    let last = lines[maxLines - 1]
    while (last && ctx.measureText(last + '…').width > width) last = last.slice(0, -1)
    lines[maxLines - 1] = last + '…'
  }
  lines.slice(0, maxLines).forEach((value, i) => ctx.fillText(value, x, y + i * lineHeight))
}

function drawTags(ctx: CanvasRenderingContext2D, tags: readonly string[], accent: string, top = 295) {
  font(ctx, 9.5, 500)
  const rows: { label: string; width: number }[][] = [[]]
  let used = 0
  for (const label of tags) {
    const width = ctx.measureText(label).width + 20
    if (used > 0 && used + 6 + width > 280) {
      rows.push([])
      used = 0
    }
    rows[rows.length - 1].push({ label, width })
    used += width + (used > 0 ? 6 : 0)
  }
  rows.slice(0, 2).forEach((row, i) => {
    let x = 180 - (row.reduce((sum, tag) => sum + tag.width, 0) + (row.length - 1) * 6) / 2
    const y = top + i * 28
    for (const tag of row) {
      ctx.fillStyle = accent + '30'
      rounded(ctx, x, y, tag.width, 22, 11)
      ctx.fillStyle = '#53645e'
      ctx.fillText(tag.label, x + tag.width / 2, y + 14.5)
      x += tag.width + 6
    }
  })
}

export async function renderQuizPoster(quiz: QuizDefinition, report: QuizReport, link: string): Promise<Blob> {
  if (document.fonts) await document.fonts.ready
  const canvas = document.createElement('canvas')
  canvas.width = POSTER_WIDTH
  canvas.height = POSTER_HEIGHT
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('当前浏览器暂时无法生成图片，可以先复制报告链接。')

  // Lay out a phone-sized portrait, then render at 3x without stretching content.
  ctx.scale(3, 3)
  const { accent, ink: inkAccent } = quizPalette[quiz.id]
  const character = Boolean(quiz.persona)
  const background = ctx.createLinearGradient(0, 0, 360, 780)
  background.addColorStop(0, '#2c3d54')
  background.addColorStop(1, '#182338')
  ctx.fillStyle = background
  ctx.fillRect(0, 0, 360, 780)

  ctx.strokeStyle = '#edbd7614'
  ctx.lineWidth = 0.7
  for (const radius of [54, 84, 114]) {
    ctx.beginPath()
    ctx.arc(356, 36, radius, 0, Math.PI * 2)
    ctx.stroke()
  }
  ctx.fillStyle = '#edbd76'
  font(ctx, 16, 600)
  ctx.fillText('相伴', 24, 43)
  ctx.fillStyle = '#b9c4d1'
  font(ctx, 9)
  ctx.textAlign = 'right'
  ctx.fillText('趣味测试 / 分享名片', 336, 42)

  ctx.save()
  ctx.shadowColor = '#0a13264d'
  ctx.shadowBlur = 24
  ctx.shadowOffsetY = 10
  ctx.fillStyle = '#f4f0e7'
  rounded(ctx, 20, 80, 320, 650, 23)
  ctx.restore()
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(20, 80, 320, 650, 23)
  ctx.clip()
  ctx.fillStyle = accent
  ctx.fillRect(20, 80, 320, 4)
  ctx.restore()

  ctx.textAlign = 'left'
  ctx.fillStyle = '#718077'
  font(ctx, 9, 600)
  ctx.fillText(quiz.name + ' / ' + quiz.reportLabel, 40, 115)
  ctx.textAlign = 'right'
  ctx.fillStyle = '#9aa299'
  font(ctx, 8)
  ctx.fillText('RESULT NOTE', 320, 115)

  if (character) {
    const art = new Image()
    await new Promise<void>((resolve, reject) => {
      art.onload = () => resolve()
      art.onerror = () => reject(new Error('插画没画好，请重试。'))
      art.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(personaArtSvg(quiz.id, report.code))
    })
    ctx.drawImage(art, 113, 123, 134, 112)
  }
  ctx.textAlign = 'center'
  ctx.fillStyle = '#243143'
  fitFont(ctx, report.code, character ? 12 : 62, 278, character ? 600 : 800)
  ctx.fillText(report.code, 180, character ? 244 : 183)
  fitFont(ctx, report.title, 23, 276, 700)
  ctx.fillText(report.title, 180, character ? 276 : 222)
  ctx.fillStyle = '#657064'
  font(ctx, 13)
  textLines(ctx, report.quote, 180, character ? 307 : 254, 268, 20)
  drawTags(ctx, report.tags, accent, character ? 350 : 295)

  ctx.fillStyle = '#deddd3'
  ctx.fillRect(40, character ? 410 : 350, 280, 0.7)
  ctx.textAlign = 'left'
  ctx.fillStyle = '#344352'
  font(ctx, 12, 600)
  ctx.fillText(quiz.metricsTitle, 40, character ? 436 : 376)
  ctx.fillStyle = '#849085'
  ctx.textAlign = 'right'
  font(ctx, 8)
  ctx.fillText('此刻的你', 320, character ? 435 : 375)

  report.metrics.forEach((metric, i) => {
    const y = (character ? 467 : 407) + i * 44
    ctx.textAlign = 'left'
    ctx.fillStyle = '#43505b'
    font(ctx, 10.5, 500)
    ctx.fillText(metric.label, 40, y)
    ctx.textAlign = 'right'
    ctx.fillStyle = '#6b766d'
    font(ctx, 9)
    const value = metric.right
      ? metric.left.split(' · ')[0] + ' ' + metric.percent + '%  /  ' + metric.right.split(' · ')[0] + ' ' + (100 - metric.percent) + '%'
      : metric.percent + '%'
    ctx.fillText(value, 320, y)
    ctx.fillStyle = metric.right ? '#d5cfdf' : '#e0dfd6'
    rounded(ctx, 40, y + 11, 280, 5, 2.5)
    if (metric.percent > 0) {
      ctx.fillStyle = inkAccent
      rounded(ctx, 40, y + 11, 280 * metric.percent / 100, 5, 2.5)
    }
  })

  ctx.fillStyle = '#deddd3'
  ctx.fillRect(40, 617, 280, 0.7)
  const qrUrl = await QRCode.toDataURL(link, {
    width: 246, margin: 4, errorCorrectionLevel: 'M',
    color: { dark: '#243143', light: '#f4f0e7' },
  })
  const qr = new Image()
  await new Promise<void>((resolve, reject) => {
    qr.onload = () => resolve()
    qr.onerror = () => reject(new Error('二维码生成失败，请重试或复制报告链接。'))
    qr.src = qrUrl
  })
  ctx.drawImage(qr, 238, 633, 82, 82)
  ctx.textAlign = 'left'
  ctx.fillStyle = '#43534a'
  font(ctx, 11.5, 600)
  ctx.fillText('今天，也请站在自己这边。', 40, 660)
  ctx.fillStyle = '#7c877c'
  font(ctx, 9.5)
  ctx.fillText('扫码看完整报告', 40, 682)
  ctx.fillText('也给自己一张新名片。', 40, 697)

  ctx.textAlign = 'center'
  ctx.fillStyle = '#bdc6d0'
  font(ctx, 8.3)
  fitFont(ctx, quiz.shortCaveat, 8.3, 316, 400)
  ctx.fillText(quiz.shortCaveat, 180, 752)
  ctx.fillStyle = '#899bad'
  font(ctx, 8)
  ctx.fillText('分享不包含逐题答案或账号信息', 180, 769)

  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => blob ? resolve(blob) : reject(new Error('图片生成失败，请重试。')), 'image/png')
  })
}
