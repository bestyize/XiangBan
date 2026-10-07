import {
  createPoster, DISPLAY, drawBackground, drawBubble, drawFooter, drawHeader, drawPanel, drawRobot, drawStat, fitText,
  loadImage, MARGIN, PAGE_W, POSTER_SCALE, POSTER_W, prepareFonts, setFont, toBlob, type Poster,
} from '../poster/frame.ts'
import type { PosterThemeId } from '../poster/theme.ts'
import type { GamePosterCopy } from './posterCopy.ts'

/** 3:4, what Xiaohongshu shows uncropped in its feed. */
const PAGE_H = 480
export const RESULT_POSTER_WIDTH = POSTER_W
export const RESULT_POSTER_HEIGHT = PAGE_H * POSTER_SCALE

async function drawCover(p: Poster, gameId: string, x: number, y: number, size: number) {
  const { ctx } = p
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(x, y, size, size, 9)
  ctx.clip()
  ctx.fillStyle = '#ffffff'
  ctx.fillRect(x, y, size, size)
  try {
    const img = await loadImage(`/game-packs/${gameId}/cover.svg`)
    const ratio = img.naturalWidth && img.naturalHeight ? img.naturalWidth / img.naturalHeight : 1
    const w = ratio >= 1 ? size * ratio : size
    const h = ratio >= 1 ? size : size / ratio
    ctx.drawImage(img, x + (size - w) / 2, y + (size - h) / 2, w, h)
  } catch {
    // The poster still works without the thumbnail.
  }
  ctx.restore()
}

export async function renderGamePoster(copy: GamePosterCopy, link: string, theme: PosterThemeId): Promise<Blob> {
  const failure = '战绩卡生成失败，请重试。'
  await prepareFonts(String(copy.score))
  const p = createPoster(PAGE_H, theme, failure)
  const { ctx, theme: t } = p

  drawBackground(p, { seed: copy.score, starsTo: 160 })
  await drawHeader(p, `玩一局 · ${copy.name}`)

  const robot = await drawRobot(p, copy.face, MARGIN + 54, 60, 108)
  drawBubble(p, copy.cheer, {
    x: robot.x + robot.w + 12, y: 0, w: PAGE_W - MARGIN - (robot.x + robot.w + 12), centerY: robot.y + robot.h * 0.46,
    size: 14, weight: 600, maxLines: 3, align: 'left', tail: { side: 'left', y: robot.y + robot.h * 0.46 },
  })

  const top = 168
  const height = 186
  drawPanel(p, MARGIN, top, PAGE_W - MARGIN * 2, height)
  await drawCover(p, copy.gameId, MARGIN + 14, top + 14, 34)
  ctx.textAlign = 'left'
  ctx.fillStyle = t.ink
  fitText(ctx, copy.name, MARGIN + 58, top + 29, 14, 200, 700)
  ctx.fillStyle = t.sub
  fitText(ctx, copy.mode, MARGIN + 58, top + 44, 9.5, 200, 500)

  ctx.textAlign = 'center'
  ctx.fillStyle = t.accent
  const score = String(copy.score)
  setFont(ctx, score.length > 6 ? 40 : 54, 400, DISPLAY)
  ctx.fillText(score, PAGE_W / 2, top + 106)
  ctx.fillStyle = t.sub
  setFont(ctx, 10, 500)
  ctx.fillText('本局积分', PAGE_W / 2, top + 124)

  const gap = 8
  const inner = PAGE_W - MARGIN * 2 - 28
  const w = (inner - gap * (copy.stats.length - 1)) / copy.stats.length
  copy.stats.forEach(([value, label], i) => drawStat(p, MARGIN + 14 + i * (w + gap), top + 132, w, value, label))

  await drawFooter(p, { headline: '你能超过我吗？', sub: '扫码看战绩，也来玩一局', link })
  return toBlob(p, failure)
}
