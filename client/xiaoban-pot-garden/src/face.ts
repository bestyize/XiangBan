import type { Expression } from './poses.ts'

export const FACE_WIDTH = 1024
export const FACE_HEIGHT = 600

export interface FaceState {
  expression: Expression
  /** 0 open, 1 shut. */
  blink: number
  /** Where the face looks, -1 to 1. */
  gazeX: number
  gazeY: number
  /** Animation clock in seconds, for the drifting z's. */
  time: number
}

const INK = '#fff3d8'
const GLOW = 'rgba(255, 214, 140, 0.95)'
const TEAR = '#9fd8ff'
const HEART = '#ffb9c8'

type Context = CanvasRenderingContext2D

/** Draws the shape twice: a wide soft halo first, then a crisp core, so the light seems to come from inside the glass. */
function lit(ctx: Context, color: string, width: number, draw: () => void, fill = false) {
  ctx.lineCap = 'round'
  ctx.lineJoin = 'round'
  for (const [blur, alpha] of [[46, 0.85], [16, 1]] as const) {
    ctx.shadowColor = GLOW
    ctx.shadowBlur = blur
    ctx.globalAlpha = alpha
    ctx.strokeStyle = color
    ctx.fillStyle = color
    ctx.lineWidth = width
    ctx.beginPath()
    draw()
    if (fill) ctx.fill()
    else ctx.stroke()
  }
  ctx.shadowBlur = 0
  ctx.globalAlpha = 1
}

const arcEye = (ctx: Context, x: number, y: number, radius: number, up: boolean, width = 30) =>
  lit(ctx, INK, width, () => (up ? ctx.arc(x, y, radius, Math.PI * 1.04, Math.PI * 1.96) : ctx.arc(x, y, radius, Math.PI * 0.04, Math.PI * 0.96)))

function pillEye(ctx: Context, x: number, y: number, w: number, h: number) {
  lit(ctx, INK, 4, () => ctx.roundRect(x - w / 2, y - h / 2, w, Math.max(h, 8), w / 2), true)
}

function dMouth(ctx: Context, x: number, y: number, w: number, h: number) {
  lit(ctx, INK, 8, () => {
    ctx.moveTo(x - w / 2, y)
    ctx.lineTo(x + w / 2, y)
    ctx.bezierCurveTo(x + w / 2, y + h * 1.15, x - w / 2, y + h * 1.15, x - w / 2, y)
    ctx.closePath()
  }, true)
}

function heart(ctx: Context, x: number, y: number, size: number) {
  lit(ctx, HEART, 6, () => {
    ctx.moveTo(x, y + size * 0.9)
    ctx.bezierCurveTo(x - size * 1.5, y - size * 0.1, x - size * 0.7, y - size * 1.2, x, y - size * 0.35)
    ctx.bezierCurveTo(x + size * 0.7, y - size * 1.2, x + size * 1.5, y - size * 0.1, x, y + size * 0.9)
  }, true)
}

export function drawFace(canvas: HTMLCanvasElement, state: FaceState): void {
  const ctx = canvas.getContext('2d')!
  const w = canvas.width
  const h = canvas.height
  ctx.setTransform(w / FACE_WIDTH, 0, 0, h / FACE_HEIGHT, 0, 0)
  ctx.fillStyle = '#000'
  ctx.fillRect(0, 0, FACE_WIDTH, FACE_HEIGHT)
  const gx = state.gazeX * 34
  const gy = state.gazeY * 22
  const cx = FACE_WIDTH / 2 + gx
  const eyeY = 282 + gy
  const mouthY = 398 + gy * 0.6
  const spread = 252
  const open = 1 - state.blink

  switch (state.expression) {
    case 'idle':
      pillEye(ctx, cx - spread, eyeY, 58, 118 * Math.max(0.08, open))
      pillEye(ctx, cx + spread, eyeY, 58, 118 * Math.max(0.08, open))
      lit(ctx, INK, 16, () => ctx.arc(cx, mouthY - 22, 38, Math.PI * 0.18, Math.PI * 0.82))
      break
    case 'happy':
      arcEye(ctx, cx - spread, eyeY + 34, 108, true, 46)
      arcEye(ctx, cx + spread, eyeY + 34, 108, true, 46)
      dMouth(ctx, cx, mouthY - 46, 200, 118)
      break
    case 'excited':
      for (const side of [-1, 1]) {
        const x = cx + side * spread
        lit(ctx, INK, 30, () => {
          ctx.moveTo(x - side * 46, eyeY - 56)
          ctx.lineTo(x + side * 38, eyeY)
          ctx.lineTo(x - side * 46, eyeY + 56)
        })
      }
      dMouth(ctx, cx, mouthY - 6, 168, 92)
      break
    case 'sleepy':
      for (const side of [-1, 1]) arcEye(ctx, cx + side * spread, eyeY - 34, 58, false, 26)
      lit(ctx, INK, 14, () => ctx.ellipse(cx, mouthY + 6, 22, 16, 0, 0, Math.PI * 2))
      for (let i = 0; i < 3; i++) {
        const phase = (state.time * 0.55 + i / 3) % 1
        ctx.globalAlpha = Math.sin(phase * Math.PI)
        ctx.fillStyle = INK
        ctx.shadowColor = GLOW
        ctx.shadowBlur = 18
        ctx.font = `700 ${46 + i * 12}px system-ui, sans-serif`
        ctx.fillText('z', 760 + phase * 90, 150 - phase * 70)
      }
      ctx.globalAlpha = 1
      ctx.shadowBlur = 0
      break
    case 'sad':
      for (const side of [-1, 1]) {
        const x = cx + side * spread
        lit(ctx, INK, 28, () => {
          ctx.moveTo(x - 58, eyeY - 4 - side * -10)
          ctx.quadraticCurveTo(x, eyeY - 52 + side * 4, x + 58, eyeY + 14 + side * -10)
        })
      }
      lit(ctx, INK, 16, () => ctx.arc(cx, mouthY + 52, 46, Math.PI * 1.15, Math.PI * 1.85))
      lit(ctx, TEAR, 4, () => {
        const x = cx + spread + 36
        const y = eyeY + 80 + ((state.time * 70) % 60)
        ctx.moveTo(x, y - 24)
        ctx.bezierCurveTo(x + 22, y + 2, x + 18, y + 26, x, y + 26)
        ctx.bezierCurveTo(x - 18, y + 26, x - 22, y + 2, x, y - 24)
      }, true)
      break
    case 'surprised':
      for (const side of [-1, 1]) lit(ctx, INK, 22, () => ctx.arc(cx + side * spread, eyeY, 50 * (0.35 + 0.65 * open), 0, Math.PI * 2))
      lit(ctx, INK, 18, () => ctx.ellipse(cx, mouthY + 14, 34, 42, 0, 0, Math.PI * 2))
      break
    case 'curious':
      pillEye(ctx, cx - spread, eyeY + 6, 58, 104 * Math.max(0.08, open))
      lit(ctx, INK, 22, () => ctx.arc(cx + spread, eyeY - 6, 52 * (0.4 + 0.6 * open), 0, Math.PI * 2))
      lit(ctx, INK, 14, () => {
        ctx.moveTo(cx - 26, mouthY + 4)
        ctx.quadraticCurveTo(cx + 6, mouthY - 12, cx + 40, mouthY - 2)
      })
      break
    case 'love':
      heart(ctx, cx - spread, eyeY, 58 + Math.sin(state.time * 6) * 3)
      heart(ctx, cx + spread, eyeY, 58 + Math.sin(state.time * 6) * 3)
      dMouth(ctx, cx, mouthY, 118, 66)
      break
  }
  ctx.setTransform(1, 0, 0, 1, 0, 0)
}
