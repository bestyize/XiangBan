import { CanvasTexture, SRGBColorSpace } from 'three'
import { rand } from './pot.ts'

const W = 1024
const H = 512

type Kind = 'top' | 'under' | 'bump'

const palettes = {
  top: { edge: '#3f8a14', mid: '#68b027', center: '#93cf36', base: 'rgba(24, 78, 10, 0.4)', tip: 'rgba(240, 255, 110, 0.16)', vein: '#d4ee82' },
  under: { edge: '#86bb48', mid: '#a3d05a', center: '#bfe078', base: 'rgba(60, 110, 30, 0.25)', tip: 'rgba(250, 255, 170, 0.2)', vein: '#eef8b4' },
}

/** Veins in texture space: the blade is stretched so that v = 0 and 1 are always its edges, which makes them converge on the tip. */
function veins(ctx: CanvasRenderingContext2D, color: string, scale: number, alpha: number) {
  ctx.strokeStyle = color
  ctx.lineCap = 'round'
  ctx.globalAlpha = alpha
  ctx.beginPath()
  ctx.moveTo(0, H / 2 - 9 * scale)
  ctx.lineTo(W * 0.97, H / 2 - 1.4 * scale)
  ctx.lineTo(W * 0.97, H / 2 + 1.4 * scale)
  ctx.lineTo(0, H / 2 + 9 * scale)
  ctx.closePath()
  ctx.fillStyle = color
  ctx.fill()
  for (const side of [-1, 1]) {
    for (let k = 0; k < 9; k++) {
      const x0 = W * (0.05 + k * 0.1)
      const reach = W * (0.27 - k * 0.012)
      ctx.lineWidth = (5.4 - k * 0.4) * scale
      ctx.beginPath()
      ctx.moveTo(x0, H / 2)
      ctx.quadraticCurveTo(x0 + reach * 0.35, H / 2 + side * H * 0.16, Math.min(W * 0.985, x0 + reach), H / 2 + side * H * 0.455)
      ctx.stroke()
    }
  }
  ctx.globalAlpha = 1
}

function paint(kind: Kind): HTMLCanvasElement {
  const canvas = document.createElement('canvas')
  canvas.width = W
  canvas.height = H
  const ctx = canvas.getContext('2d')!
  const random = rand(kind === 'top' ? 5 : kind === 'under' ? 9 : 13)
  if (kind === 'bump') {
    ctx.fillStyle = '#8a8a8a'
    ctx.fillRect(0, 0, W, H)
    for (let i = 0; i < 2600; i++) {
      ctx.strokeStyle = random() > 0.5 ? 'rgba(60,60,60,0.12)' : 'rgba(210,210,210,0.12)'
      ctx.lineWidth = 1 + random() * 1.5
      const x = random() * W
      const y = random() * H
      ctx.beginPath()
      ctx.moveTo(x, y)
      ctx.lineTo(x + (random() - 0.5) * 26, y + (random() - 0.5) * 26)
      ctx.stroke()
    }
    veins(ctx, '#2e2e2e', 1.4, 0.95)
    return canvas
  }
  const p = palettes[kind]
  const across = ctx.createLinearGradient(0, 0, 0, H)
  across.addColorStop(0, p.edge)
  across.addColorStop(0.28, p.mid)
  across.addColorStop(0.5, p.center)
  across.addColorStop(0.72, p.mid)
  across.addColorStop(1, p.edge)
  ctx.fillStyle = across
  ctx.fillRect(0, 0, W, H)
  const along = ctx.createLinearGradient(0, 0, W, 0)
  along.addColorStop(0, p.base)
  along.addColorStop(0.4, 'rgba(0,0,0,0)')
  along.addColorStop(1, p.tip)
  ctx.fillStyle = along
  ctx.fillRect(0, 0, W, H)
  for (let i = 0; i < 90; i++) {
    const x = random() * W
    const y = random() * H
    const r = 30 + random() * 90
    const blob = ctx.createRadialGradient(x, y, 0, x, y, r)
    const light = random() > 0.5
    blob.addColorStop(0, light ? 'rgba(220,255,120,0.09)' : 'rgba(20,90,10,0.08)')
    blob.addColorStop(1, 'rgba(0,0,0,0)')
    ctx.fillStyle = blob
    ctx.fillRect(x - r, y - r, r * 2, r * 2)
  }
  for (let i = 0; i < 2200; i++) {
    ctx.strokeStyle = random() > 0.5 ? 'rgba(30,100,10,0.07)' : 'rgba(235,255,150,0.07)'
    ctx.lineWidth = 1 + random() * 1.4
    const x = random() * W
    const y = random() * H
    ctx.beginPath()
    ctx.moveTo(x, y)
    ctx.lineTo(x + (random() - 0.5) * 30, y + (random() - 0.5) * 30)
    ctx.stroke()
  }
  veins(ctx, 'rgba(20, 80, 5, 0.5)', 1.35, 0.35)
  veins(ctx, p.vein, 0.8, 0.42)
  return canvas
}

function texture(kind: Kind): CanvasTexture {
  const t = new CanvasTexture(paint(kind))
  if (kind !== 'bump') t.colorSpace = SRGBColorSpace
  t.anisotropy = 8
  return t
}

export const makeLeafTextures = () => ({ top: texture('top'), under: texture('under'), bump: texture('bump') })
