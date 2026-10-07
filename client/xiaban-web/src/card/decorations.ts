import type { Decoration } from './styles.ts'

/** Small deterministic PRNG so the same check-in always gets the same sky. */
function seeded(seed: number) {
  let s = (seed * 9301 + 49297) % 233280 || 1
  return () => {
    s = (s * 9301 + 49297) % 233280
    return s / 233280
  }
}

function sparkle(ctx: CanvasRenderingContext2D, x: number, y: number, r: number) {
  const k = r * 0.18
  ctx.beginPath()
  ctx.moveTo(x, y - r)
  ctx.quadraticCurveTo(x + k, y - k, x + r, y)
  ctx.quadraticCurveTo(x + k, y + k, x, y + r)
  ctx.quadraticCurveTo(x - k, y + k, x - r, y)
  ctx.quadraticCurveTo(x - k, y - k, x, y - r)
  ctx.fill()
}

function lantern(ctx: CanvasRenderingContext2D, x: number, y: number) {
  ctx.strokeStyle = '#ffd76a'
  ctx.lineWidth = 4
  ctx.beginPath()
  ctx.moveTo(x, y - 170)
  ctx.lineTo(x, y - 70)
  ctx.stroke()
  ctx.fillStyle = '#e8453c'
  ctx.beginPath()
  ctx.ellipse(x, y, 78, 66, 0, 0, Math.PI * 2)
  ctx.fill()
  ctx.fillStyle = '#ffd76a'
  ctx.fillRect(x - 40, y - 74, 80, 16)
  ctx.fillRect(x - 40, y + 58, 80, 16)
  ctx.fillRect(x - 4, y + 74, 8, 60)
}

export function drawDecoration(ctx: CanvasRenderingContext2D, decoration: Decoration, seed: number) {
  const rand = seeded(seed)
  ctx.save()
  if (decoration === 'stars' || decoration === 'moon') {
    for (let i = 0; i < 46; i++) {
      ctx.fillStyle = `rgba(255, 241, 201, ${0.25 + rand() * 0.5})`
      ctx.beginPath()
      ctx.arc(rand() * 1080, rand() * 1150, 2 + rand() * 3.5, 0, Math.PI * 2)
      ctx.fill()
    }
    ctx.fillStyle = 'rgba(255, 215, 120, 0.8)'
    for (let i = 0; i < 4; i++) sparkle(ctx, 80 + rand() * 920, 640 + rand() * 520, 10 + rand() * 12)
  }
  if (decoration === 'moon') {
    ctx.shadowColor = 'rgba(255, 230, 163, 0.7)'
    ctx.shadowBlur = 90
    ctx.fillStyle = '#ffe6a3'
    ctx.beginPath()
    ctx.arc(920, 780, 96, 0, Math.PI * 2)
    ctx.fill()
  }
  if (decoration === 'confetti') {
    const colors = ['#ffd76a', '#ff7e79', '#8cc8ff', '#7ee0a1', '#fff1c9']
    for (let i = 0; i < 70; i++) {
      ctx.save()
      ctx.translate(rand() * 1080, rand() * 1920)
      ctx.rotate(rand() * Math.PI)
      ctx.fillStyle = colors[i % colors.length]
      ctx.globalAlpha = 0.35 + rand() * 0.4
      ctx.fillRect(-9, -4, 18, 8)
      ctx.restore()
    }
  }
  if (decoration === 'lanterns') {
    // Beside the window, clear of the title and the speech bubble.
    lantern(ctx, 150, 860)
    lantern(ctx, 930, 820)
  }
  ctx.restore()
}
