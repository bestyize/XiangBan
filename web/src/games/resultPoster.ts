import QRCode from 'qrcode'
import { survivalTime, type SnakeResult } from './resultShare.ts'

export const RESULT_POSTER_WIDTH = 1080
export const RESULT_POSTER_HEIGHT = 1440
const FONT = '"PingFang SC", "Microsoft YaHei", system-ui, sans-serif'

export async function renderResultPoster(result: SnakeResult, link: string): Promise<Blob> {
  if (document.fonts) await document.fonts.ready
  const canvas = document.createElement('canvas')
  canvas.width = RESULT_POSTER_WIDTH; canvas.height = RESULT_POSTER_HEIGHT
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('暂时无法生成战绩卡，可以先复制链接。')
  const font = (size: number, weight = 500) => { ctx.font = `${weight} ${size}px ${FONT}` }
  const rounded = (x: number, y: number, width: number, height: number, radius: number, color: string) => {
    ctx.fillStyle = color; ctx.beginPath(); ctx.roundRect(x, y, width, height, radius); ctx.fill()
  }
  const circle = (x: number, y: number, radius: number, color: string) => {
    ctx.fillStyle = color; ctx.beginPath(); ctx.arc(x, y, radius, 0, Math.PI * 2); ctx.fill()
  }
  const fittedText = (text: string, x: number, y: number, size: number, maxWidth: number, weight = 600) => {
    font(size, weight)
    while (size > 10 && ctx.measureText(text).width > maxWidth) font(--size, weight)
    ctx.fillText(text, x, y)
  }
  ctx.scale(3, 3)
  ctx.fillStyle = '#eaf3e3'; ctx.fillRect(0, 0, 360, 480)
  rounded(18, 16, 324, 448, 24, '#fffef6')
  ctx.textAlign = 'left'; ctx.fillStyle = '#42664b'; font(16, 750); ctx.fillText('小伴贪吃蛇', 36, 51)
  ctx.textAlign = 'right'; ctx.fillStyle = '#8d9d80'; font(9); ctx.fillText(result.roundType === 'endless' ? '无尽生存' : '三分钟竞技', 324, 50)
  ctx.textAlign = 'center'; ctx.fillStyle = '#45684c'; font(21, 750); ctx.fillText('这一局，快乐长大', 180, 96)

  const body = [[105,184],[113,192],[125,198],[138,195],[147,186],[149,173],[147,160],[153,149],[164,145],[177,148],[184,158],[185,171],[194,182],[207,186],[220,180],[228,169],[229,156],[223,145],[212,137]]
  body.forEach(([x, y]) => circle(x, y, 10, '#ffca28'))
  circle(207, 130, 14, '#ffca28')
  circle(202, 126, 4.5, '#ffffff'); circle(213, 126, 4.5, '#ffffff')
  circle(203, 126, 2.1, '#263447'); circle(214, 126, 2.1, '#263447')
  circle(89, 141, 4.5, '#ff4e6a'); circle(265, 180, 4, '#aa55ef'); circle(121, 124, 3.5, '#20b8f5'); circle(259, 123, 4.5, '#54cc45')
  ctx.fillStyle = '#52865b'; fittedText(String(result.score), 180, 269, 55, 280, 850)
  ctx.fillStyle = '#91a186'; font(10); ctx.fillText('本局积分', 180, 290)

  const stats = [[String(result.mass), '最终长度'], [String(result.kills), '淘汰 AI'], [survivalTime(result.seconds), '存活时长']]
  stats.forEach(([value, label], index) => {
    const x = 36 + index * 98
    rounded(x, 309, 92, 49, 12, '#f1f5e9')
    ctx.fillStyle = '#668263'; fittedText(value, x + 46, 333, 18, 78)
    ctx.fillStyle = '#9ca98c'; font(9); ctx.fillText(label, x + 46, 349)
  })
  ctx.fillStyle = '#e3ebd8'; ctx.fillRect(36, 373, 288, 1)
  const qr = document.createElement('canvas')
  await QRCode.toCanvas(qr, link, { width: 240, margin: 4, errorCorrectionLevel: 'M', color: { dark: '#36523d', light: '#ffffff' } })
  ctx.imageSmoothingEnabled = false
  ctx.drawImage(qr, 244, 382, 80, 80)
  ctx.imageSmoothingEnabled = true
  ctx.textAlign = 'left'; ctx.fillStyle = '#50744d'; font(12, 700); ctx.fillText('你能超过我吗？', 36, 404)
  ctx.fillStyle = '#8fa07f'; font(9); ctx.fillText('扫码看战绩，也来玩一局', 36, 423)
  ctx.fillText(result.difficulty === 'lively' ? '热闹难度 · 本地 AI 对战' : '轻松难度 · 本地 AI 对战', 36, 440)

  return new Promise((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error('战绩卡生成失败，请重试。')), 'image/png'))
}
