import QRCode from 'qrcode'
import type { QuizDefinition, QuizReport } from './types.ts'

function rounded(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath()
  ctx.roundRect(x, y, w, h, r)
  ctx.fill()
}

function textLines(ctx: CanvasRenderingContext2D, text: string, x: number, y: number, width: number, lineHeight: number) {
  let line = ''
  let row = 0
  for (const char of text) {
    if (ctx.measureText(line + char).width > width && line) {
      ctx.fillText(line, x, y + row * lineHeight)
      row += 1
      line = char
    } else line += char
  }
  if (line) ctx.fillText(line, x, y + row * lineHeight)
}

export async function renderQuizPoster(quiz: QuizDefinition, report: QuizReport, link: string): Promise<Blob> {
  if (document.fonts) await document.fonts.ready
  const canvas = document.createElement('canvas')
  canvas.width = 1080
  canvas.height = 1560
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('当前浏览器暂时无法生成图片，可以先复制报告链接。')
  const accent = quiz.id === 'mbti' ? '#a6cdb5' : '#bfb2e2'
  const font = '"PingFang SC", "Microsoft YaHei", "Noto Sans SC", sans-serif'
  const background = ctx.createLinearGradient(0, 0, 1080, 1560)
  background.addColorStop(0, '#344760')
  background.addColorStop(1, '#182338')
  ctx.fillStyle = background
  ctx.fillRect(0, 0, 1080, 1560)
  ctx.strokeStyle = '#edbd7640'
  ctx.lineWidth = 2
  for (let radius = 130; radius < 560; radius += 100) {
    ctx.beginPath(); ctx.arc(1040, 70, radius, 0, Math.PI * 2); ctx.stroke()
  }
  ctx.fillStyle = '#edbd76'
  ctx.font = `600 35px ${font}`
  ctx.fillText('下班了么', 72, 96)
  ctx.fillStyle = '#e1ded6'
  ctx.font = `24px ${font}`
  ctx.textAlign = 'right'
  ctx.fillText('趣味测试 · 给自己一张新名片', 1008, 96)
  ctx.textAlign = 'left'
  ctx.fillStyle = '#f3eee4'
  rounded(ctx, 64, 164, 952, 1184, 40)
  ctx.fillStyle = '#69776f'
  ctx.font = `600 24px ${font}`
  ctx.fillText(`${quiz.name} / ${quiz.id === 'mbti' ? '性格偏好报告' : '打工人续航报告'}`, 112, 244)
  ctx.fillStyle = accent
  ctx.beginPath(); ctx.arc(928, 224, 27, 0, Math.PI * 2); ctx.fill()
  ctx.fillStyle = '#202a39'
  let codeSize = quiz.id === 'mbti' ? 144 : 114
  do { ctx.font = `800 ${codeSize--}px ${font}` } while (ctx.measureText(report.code).width > 840)
  ctx.fillText(report.code, 104, 401)
  ctx.font = `700 62px ${font}`
  ctx.fillText(report.title, 112, 484)
  ctx.fillStyle = '#586272'
  ctx.font = `36px ${font}`
  textLines(ctx, report.quote, 112, 558, 856, 50)
  ctx.font = `24px ${font}`
  let tagX = 112
  for (const tag of report.tags.slice(0, 4)) {
    const width = ctx.measureText(tag).width + 36
    if (tagX + width > 968) break
    ctx.fillStyle = `${accent}70`
    rounded(ctx, tagX, 646, width, 46, 23)
    ctx.fillStyle = '#344352'
    ctx.fillText(tag, tagX + 18, 677)
    tagX += width + 12
  }
  ctx.fillStyle = '#ddd9ce'
  ctx.fillRect(112, 727, 856, 1)
  report.metrics.forEach((metric, i) => {
    const y = 772 + i * 76
    ctx.fillStyle = '#202a39'
    ctx.font = `600 25px ${font}`
    ctx.fillText(metric.label, 112, y)
    ctx.textAlign = 'right'
    ctx.fillStyle = '#566171'
    ctx.font = `23px ${font}`
    ctx.fillText(metric.right ? `${metric.left} ${metric.percent}% / ${metric.right} ${100 - metric.percent}%` : metric.note, 968, y)
    ctx.textAlign = 'left'
    ctx.fillStyle = metric.right ? '#a798c0' : '#dedbd3'
    rounded(ctx, 112, y + 17, 856, 12, 6)
    if (metric.percent > 0) {
      ctx.fillStyle = accent
      rounded(ctx, 112, y + 17, 856 * metric.percent / 100, 12, 6)
    }
  })
  const qrUrl = await QRCode.toDataURL(link, { width: 152, margin: 1, errorCorrectionLevel: 'M', color: { dark: '#202a39', light: '#f3eee4' } })
  const qr = new Image()
  await new Promise<void>((resolve, reject) => {
    qr.onload = () => resolve()
    qr.onerror = () => reject(new Error('二维码生成失败，请重试或复制报告链接。'))
    qr.src = qrUrl
  })
  ctx.drawImage(qr, 810, 1160, 152, 152)
  ctx.fillStyle = '#202a39'
  ctx.font = `600 32px ${font}`
  ctx.fillText('今天，也请站在自己这边。', 112, 1205)
  ctx.fillStyle = '#66706e'
  ctx.font = `25px ${font}`
  ctx.fillText('扫码看完整报告 · 测测你是哪一型', 112, 1256)
  ctx.fillStyle = '#d6d8da'
  ctx.font = `25px ${font}`
  ctx.fillText(quiz.id === 'mbti' ? 'MBTI 风格探索 · 非官方量表 · 结果仅供参考' : 'M班TI · 原创趣味测试 · 不代表工作或心理状态评估', 72, 1420)
  ctx.fillStyle = '#aeb7c2'
  ctx.font = `23px ${font}`
  ctx.fillText('这张名片不包含逐题答案或账号信息。', 72, 1472)
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => blob ? resolve(blob) : reject(new Error('图片生成失败，请重试。')), 'image/png')
  })
}
