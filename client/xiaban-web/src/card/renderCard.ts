import type { Checkin, Profile } from '../api/types.ts'
import { formatDay } from '../lib/time.ts'
import { moodFace } from '../robot/moods.ts'
import {
  createPoster, DISPLAY, drawBackground, drawBubble, drawFooter, drawHeader, drawRobot, MARGIN, PAGE_W,
  POSTER_SCALE, POSTER_W, prepareFonts, setFont, toBlob,
} from '../poster/frame.ts'
import { POSTER_THEMES } from '../poster/theme.ts'
import { drawDecoration } from './decorations.ts'
import { cardStatLine, type CardStyle } from './styles.ts'

/** 9:16, a full phone screen for stories and moments. */
const PAGE_H = 640
export const CARD_W = POSTER_W
export const CARD_H = PAGE_H * POSTER_SCALE

export interface CardInput {
  checkin: Checkin
  profile: Profile
  style: CardStyle
  showCity: boolean
  /** QR target, carrying from=card and the inviter's code */
  link: string
}

/** Draws the 下班卡 (PRD 5.7) on the shared poster frame and returns it as a PNG. */
export async function renderCard({ checkin, profile, style, showCity, link }: CardInput): Promise<Blob> {
  const failure = '生成图片失败'
  await prepareFonts(checkin.title.name + checkin.localTime + '下班')
  // Variants recolor the title only; the 相伴 brand keeps its own red.
  const theme = POSTER_THEMES[style.theme]
  const p = createPoster(PAGE_H, theme, failure)
  const { ctx } = p
  const seed = Number(checkin.id) || 1

  drawBackground(p, { bg: style.sky, seed, stars: false })
  if (style.decoration) {
    // The festive decorations are drawn in output pixels.
    ctx.save()
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    drawDecoration(ctx, style.decoration, seed)
    ctx.restore()
  }

  const city = showCity && profile.cityName && profile.cityCode !== 'other' ? `${profile.cityName} · ` : ''
  await drawHeader(p, `下班啦 · ${city}${formatDay(new Date(checkin.checkedAt))}`)

  const title = checkin.title.name
  ctx.save()
  ctx.translate(MARGIN + 4, 140)
  ctx.rotate(-0.035)
  ctx.textAlign = 'left'
  ctx.fillStyle = style.accent ?? theme.accent
  setFont(ctx, Math.min(72, Math.floor(300 / [...title].length)), 400, DISPLAY)
  ctx.fillText(title, 0, 0)
  ctx.restore()

  ctx.textAlign = 'left'
  ctx.fillStyle = theme.ink
  setFont(ctx, 42, 400, DISPLAY)
  ctx.fillText(checkin.localTime, MARGIN + 4, 196)
  const timeW = ctx.measureText(checkin.localTime).width
  ctx.fillStyle = theme.sub
  setFont(ctx, 17, 500)
  ctx.fillText('下班', MARGIN + 12 + timeW, 194)

  const robot = await drawRobot(p, moodFace[checkin.mood], PAGE_W / 2, 214, 178)
  const bubbleBottom = drawBubble(p, checkin.reply.text, {
    x: MARGIN + 6, y: robot.y + robot.h + 18, w: PAGE_W - (MARGIN + 6) * 2,
    size: 16.5, maxLines: 3, tail: { side: 'up', x: PAGE_W / 2 },
  })

  ctx.textAlign = 'center'
  ctx.fillStyle = theme.ink
  setFont(ctx, 15, 500)
  ctx.fillText(cardStatLine(checkin, profile.companionName), PAGE_W / 2, Math.min(bubbleBottom + 34, 506))

  await drawFooter(p, { headline: '你下班啦？', sub: `扫码，让${profile.companionName || '小伴'}也陪你下班`, link })
  return toBlob(p, failure)
}
