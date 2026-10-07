import {
  createPoster, drawBackground, drawBubble, drawFooter, drawHeader, drawPanel, drawRobot, loadImage, MARGIN, PAGE_W,
  POSTER_SCALE, POSTER_W, prepareFonts, setFont, toBlob, type Poster,
} from '../poster/frame.ts'
import type { PosterThemeId } from '../poster/theme.ts'
import { personaArtSvg, quizPalette } from './art.ts'
import type { QuizDefinition, QuizReport } from './types.ts'

const PAGE_H = 780
export const POSTER_WIDTH = POSTER_W
export const POSTER_HEIGHT = PAGE_H * POSTER_SCALE

function font(ctx: CanvasRenderingContext2D, size: number, weight = 400) {
  setFont(ctx, size, weight)
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

function drawTags(p: Poster, tags: readonly string[], accent: string, top: number) {
  const { ctx } = p
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
      ctx.fillStyle = accent + (p.theme.id === 'night' ? '33' : '3d')
      rounded(ctx, x, y, tag.width, 22, 11)
      ctx.fillStyle = p.theme.ink
      ctx.fillText(tag.label, x + tag.width / 2, y + 14.5)
      x += tag.width + 6
    }
  })
}

export async function renderQuizPoster(quiz: QuizDefinition, report: QuizReport, link: string, theme: PosterThemeId): Promise<Blob> {
  const failure = '当前浏览器暂时无法生成图片，可以先复制报告链接。'
  await prepareFonts()
  const p = createPoster(PAGE_H, theme, failure)
  const { ctx, theme: t } = p
  const { accent, ink: inkAccent } = quizPalette[quiz.id]
  const character = Boolean(quiz.persona)

  drawBackground(p, { seed: report.code.length * 31 + report.title.length, starsTo: 150 })
  await drawHeader(p, `趣味测试 · ${quiz.name}`)

  const robot = await drawRobot(p, { expression: 'happy', symbol: 'sparkle' }, MARGIN + 50, 58, 100)
  drawBubble(p, report.quote, {
    x: robot.x + robot.w + 12, y: 0, w: PAGE_W - MARGIN - (robot.x + robot.w + 12), centerY: robot.y + robot.h * 0.46,
    size: 13, weight: 500, lineHeight: 19, maxLines: 3, align: 'left', tail: { side: 'left', y: robot.y + robot.h * 0.46 },
  })

  const footerTop = PAGE_H - MARGIN - 34 - 92
  const top = 162
  const bottom = footerTop - 14
  drawPanel(p, MARGIN, top, PAGE_W - MARGIN * 2, bottom - top)
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(MARGIN, top, PAGE_W - MARGIN * 2, bottom - top, 18)
  ctx.clip()
  ctx.fillStyle = accent
  ctx.fillRect(MARGIN, top, PAGE_W - MARGIN * 2, 4)
  ctx.restore()

  ctx.textAlign = 'left'
  ctx.fillStyle = t.sub
  font(ctx, 9, 600)
  ctx.fillText(quiz.name + ' / ' + quiz.reportLabel, MARGIN + 18, top + 28)
  ctx.textAlign = 'right'
  ctx.fillStyle = t.faint
  font(ctx, 8.5)
  ctx.fillText('此刻的你', PAGE_W - MARGIN - 18, top + 28)

  let y = top + 40
  if (character) {
    const art = await loadImage('data:image/svg+xml;charset=utf-8,' + encodeURIComponent(personaArtSvg(quiz.id, report.code)))
      .catch(() => { throw new Error('插画没画好，请重试。') })
    ctx.drawImage(art, 120, y, 120, 100)
    y += 112
  }
  ctx.textAlign = 'center'
  ctx.fillStyle = t.ink
  fitFont(ctx, report.code, character ? 12 : 56, 278, character ? 600 : 800)
  ctx.fillText(report.code, 180, character ? y : y + 54)
  y += character ? 30 : 92
  fitFont(ctx, report.title, 23, 276, 700)
  ctx.fillText(report.title, 180, y)
  y += 20
  drawTags(p, report.tags, accent, y)
  y += 64

  ctx.fillStyle = t.panelLine
  ctx.fillRect(MARGIN + 18, y, PAGE_W - (MARGIN + 18) * 2, 0.7)
  y += 24
  ctx.textAlign = 'left'
  ctx.fillStyle = t.ink
  font(ctx, 12, 600)
  ctx.fillText(quiz.metricsTitle, MARGIN + 18, y)

  const first = y + 28
  const gap = Math.min(44, (bottom - 22 - first) / Math.max(1, report.metrics.length))
  report.metrics.forEach((metric, i) => {
    const my = first + i * gap
    ctx.textAlign = 'left'
    ctx.fillStyle = t.ink
    font(ctx, 10.5, 500)
    ctx.fillText(metric.label, MARGIN + 18, my)
    ctx.textAlign = 'right'
    ctx.fillStyle = t.sub
    font(ctx, 9)
    const value = metric.right
      ? metric.left.split(' · ')[0] + ' ' + metric.percent + '%  /  ' + metric.right.split(' · ')[0] + ' ' + (100 - metric.percent) + '%'
      : metric.percent + '%'
    ctx.fillText(value, PAGE_W - MARGIN - 18, my)
    ctx.fillStyle = t.chip
    rounded(ctx, MARGIN + 18, my + 9, PAGE_W - (MARGIN + 18) * 2, 5, 2.5)
    if (metric.percent > 0) {
      ctx.fillStyle = t.id === 'night' ? accent : inkAccent
      rounded(ctx, MARGIN + 18, my + 9, (PAGE_W - (MARGIN + 18) * 2) * metric.percent / 100, 5, 2.5)
    }
  })

  await drawFooter(p, { headline: '今天，也请站在自己这边。', sub: '扫码看完整报告，也给自己一张新名片', link, bottom: footerTop + 92 })

  ctx.textAlign = 'center'
  ctx.fillStyle = t.faint
  fitFont(ctx, quiz.shortCaveat, 8.3, 316, 400)
  ctx.fillText(quiz.shortCaveat, 180, PAGE_H - 36)
  font(ctx, 8)
  ctx.fillText('分享不包含逐题答案或账号信息', 180, PAGE_H - 22)

  return toBlob(p, '图片生成失败，请重试。')
}
