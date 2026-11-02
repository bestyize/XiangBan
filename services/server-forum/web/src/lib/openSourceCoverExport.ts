import type { OpenSourceProject } from './openSource'
import { openSourceCoverRules, resolveOpenSourceCover } from './openSourceCovers'

function mix(hex: string, ratio: number) {
  const parts = [1, 3, 5].map(i => Math.round(parseInt(hex.slice(i, i + 2), 16) * ratio + 255 * (1 - ratio)))
  return `rgb(${parts.join(',')})`
}
async function loadImage(url?: string) {
  if (!url) return null
  return new Promise<HTMLImageElement | null>(resolve => {
    const image = new Image(); image.crossOrigin = 'anonymous'
    const timeout = window.setTimeout(() => { image.src = ''; resolve(null) }, 15000)
    image.onload = () => { clearTimeout(timeout); resolve(image) }; image.onerror = () => { clearTimeout(timeout); resolve(null) }; image.src = url
  })
}

/** Only used by the curated-project editor. Exporting a cover never calls a model. */
export async function exportOpenSourceCover(input: OpenSourceProject): Promise<Blob> {
  const project = resolveOpenSourceCover(input)
  const [icon, media] = await Promise.all([loadImage(project.icon_url), loadImage(project.cover_style === 'illustration' ? undefined : project.cover_url)])
  if (project.icon_url && !icon || project.cover_url && project.cover_style !== 'illustration' && !media) throw new Error('素材无法导出。请先采集为站内素材，或上传本地图片后重试。')
  const canvas = document.createElement('canvas'); canvas.width = openSourceCoverRules.width; canvas.height = openSourceCoverRules.height
  const ctx = canvas.getContext('2d')!
  const w = canvas.width, h = canvas.height, accent = project.cover_accent!
  ctx.fillStyle = mix(accent, .07); ctx.fillRect(0, 0, w, h)
  function box(x: number, y: number, width: number, height: number, radius = 22, stroke = '#e7e7e7') {
    ctx.beginPath(); ctx.roundRect(x, y, width, height, radius); ctx.fillStyle = '#fff'; ctx.fill(); ctx.strokeStyle = stroke; ctx.lineWidth = 2; ctx.stroke()
  }
  function text(value: string, x: number, y: number, size = 32, color = '#444', font = 'sans-serif', max = w) {
    ctx.fillStyle = color; ctx.font = `400 ${size}px ${font}`
    let line = value
    while (ctx.measureText(line).width > max && line.length) line = line.slice(0, -1)
    ctx.fillText(line === value ? line : `${line}…`, x, y)
  }
  function picture(image: HTMLImageElement, x: number, y: number, width: number, height: number, crop: boolean, fx = .5, fy = .46) {
    const scale = crop ? Math.max(width / image.naturalWidth, height / image.naturalHeight) : Math.min(width / image.naturalWidth, height / image.naturalHeight)
    const dw = image.naturalWidth * scale, dh = image.naturalHeight * scale
    ctx.save(); ctx.beginPath(); ctx.rect(x, y, width, height); ctx.clip()
    ctx.drawImage(image, x + (width - dw) * fx, y + (height - dh) * fy, dw, dh); ctx.restore()
  }
  if (project.cover_style === 'photo' && media) picture(media, 0, 0, w, h, true, project.cover_focus_x! / 100, project.cover_focus_y! / 100)
  if (project.cover_style === 'screen' && media) {
    const wide = project.cover_variant === 'wide'
    const x = wide ? 0 : w * (project.cover_variant === 'offset' ? .04 : .08), y = h * (wide ? .25 : .28)
    const width = w * (wide ? 1 : .88), height = h * (wide ? .75 : .8)
    ctx.save(); ctx.beginPath(); ctx.roundRect(x, y, width, height, wide ? 0 : 20); ctx.clip(); picture(media, x, y, width, height, true, 0, 0); ctx.restore()
  }
  const photo = project.cover_style === 'photo'
  const bx = w * .05, by = photo ? h * .78 : h * .07, iconSize = 76
  if (photo) { ctx.font = '500 36px sans-serif'; box(bx - 12, by - 12, Math.min(700, ctx.measureText(project.name).width + (icon ? 116 : 40)), 100, 24) }
  if (icon) {
    box(bx, by, iconSize, iconSize, 16)
    picture(icon, bx + 8, by + 8, iconSize - 16, iconSize - 16, project.icon_fit === 'mark-left', project.icon_fit === 'mark-left' ? 0 : .5, .5)
  }
  text(project.name, bx + (icon ? iconSize + 20 : 0), by + 48, 36, '#222', 'sans-serif', w * .65)
  if (project.cover_style === 'illustration') {
    const layout = project.cover_layout, lines = project.cover_lines!.slice(0, 3)
    ctx.textAlign = 'right'; text(layout === 'terminal' ? '命令示例' : layout === 'notes' ? '内容示意' : '流程示意', w * .95, h * .14, 28, '#777'); ctx.textAlign = 'left'
    if (layout === 'terminal') {
      const left = project.cover_variant === 'wide' ? .04 : project.cover_variant === 'offset' ? .10 : .07
      const right = project.cover_variant === 'wide' || project.cover_variant === 'offset' ? .04 : .07
      box(w * left, h * .31, w * (1-left-right), h * .57, 24)
      text(project.name.toLowerCase(), w * (left+.04), h * .45, 28, '#777', 'monospace')
      text(lines[0] || '', w * (left+.04), h * .60, 34, '#222', 'monospace', w * .76)
      text(lines.slice(1).join(' · '), w * (left+.04), h * .75, 28, '#777')
    } else if (layout === 'notes') {
      lines.forEach((line, index) => {
        const shift = project.cover_variant !== 'offset' ? 0 : index === 1 ? -.04 : index === 2 ? .025 : 0
        const x = w * ((project.cover_variant === 'wide' ? .07 : .12) + shift), y = h * (.32 + index * .19)
        box(x, y, w * (project.cover_variant === 'wide' ? .86 : .78), h * .15, 20); text(`0${index + 1}`, x + 28, y + 64, 28, '#888', 'monospace'); text(line, x + 106, y + 64, 34)
      })
    } else {
      const width = w * .22, gap = w * .10, left = (w - lines.length * width - (lines.length - 1) * gap) / 2
      lines.forEach((line, index) => {
        const shift = project.cover_variant === 'offset' ? index === 0 ? -.04 : index === 2 ? .04 : 0 : 0
        const x = left + index * (width + gap), y = h * (.45 + shift)
        box(x, y, width, h * .26, 28, mix(accent, .35)); ctx.strokeStyle = accent; ctx.lineWidth = 3; ctx.strokeRect(x + width / 2 - 12, y + 28, 24, 24)
        ctx.textAlign = 'center'; text(line, x + width / 2, y + 124, 36); ctx.textAlign = 'left'
        if (index < lines.length - 1) { const ax = x + width + 8, ay = h * .58; ctx.strokeStyle = accent; ctx.lineWidth = 3; ctx.beginPath(); ctx.moveTo(ax, ay); ctx.lineTo(ax + gap - 16, ay); ctx.moveTo(ax + gap - 34, ay - 14); ctx.lineTo(ax + gap - 16, ay); ctx.lineTo(ax + gap - 34, ay + 14); ctx.stroke() }
      })
    }
  }
  return new Promise((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error('封面导出失败')), 'image/png'))
}
export function downloadCover(blob: Blob, name: string) {
  const url = URL.createObjectURL(blob), anchor = document.createElement('a')
  anchor.href = url; anchor.download = `${name.replace(/[^\p{L}\p{N}._-]/gu, '-')}-cover.png`; anchor.click(); window.setTimeout(() => URL.revokeObjectURL(url), 1000)
}
