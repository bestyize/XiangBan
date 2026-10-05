import QRCode from 'qrcode'
import { otherResultStats, RESULT_GAMES, type OtherResult } from './otherResultShare.ts'
import { RESULT_POSTER_HEIGHT, RESULT_POSTER_WIDTH } from './resultPoster.ts'

export async function renderOtherResultPoster(result: OtherResult, link: string): Promise<Blob> {
  if (document.fonts) await document.fonts.ready
  const canvas = document.createElement('canvas')
  canvas.width = RESULT_POSTER_WIDTH; canvas.height = RESULT_POSTER_HEIGHT
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('暂时无法生成战绩卡，可以先复制链接。')
  ctx.scale(3, 3)
  const text = (value: string, x: number, y: number, size: number, color = '#42664b', weight = 600) => {
    ctx.fillStyle = color; ctx.font = `${weight} ${size}px "PingFang SC", "Microsoft YaHei", system-ui, sans-serif`; ctx.fillText(value, x, y)
  }
  const rect = (x: number, y: number, width: number, height: number, radius: number, color: string) => {
    ctx.fillStyle = color; ctx.beginPath(); ctx.roundRect(x, y, width, height, radius); ctx.fill()
  }
  ctx.fillStyle = '#eaf3e3'; ctx.fillRect(0, 0, 360, 480)
  rect(18, 16, 324, 448, 24, '#fffef6')
  text(RESULT_GAMES[result.gameId], 36, 51, 16)
  ctx.textAlign = 'right'; text('下班啦 · 玩一局', 324, 50, 9, '#8d9d80')
  ctx.textAlign = 'center'; text('这一局，玩得漂亮', 180, 95, 22)
  const colors = ['#ffca28', '#29b6f6', '#66cc55', '#ff6b8a']
  if (result.gameId === 'water-sort') {
    for (let i = 0; i < 4; i++) {
      rect(89 + i * 48, 127, 38, 70, 11, '#e3eee6')
      rect(93 + i * 48, 137, 30, 56, 8, colors[i])
    }
  } else {
    const values = [2, 4, 8, 16]
    for (let i = 0; i < 4; i++) {
      const x = 122 + (i % 2) * 61, y = 117 + Math.floor(i / 2) * 48
      rect(x, y, 54, 42, 10, colors[i])
      if (result.gameId === 'merge-2048') text(String(values[i]), x + 27, y + 29, 23, '#fffef6', 750)
    }
  }
  const score = String(result.score)
  text(score, 180, 267, score.length > 7 ? 36 : 54, '#52865b', 850)
  text('本局积分', 180, 290, 10, '#91a186')
  const stats = otherResultStats(result)
  const width = (288 - (stats.length - 1) * 8) / stats.length
  stats.forEach(([value, label], i) => {
    const x = 36 + i * (width + 8)
    rect(x, 309, width, 49, 12, '#f1f5e9')
    text(value, x + width / 2, 333, 18, '#668263')
    text(label, x + width / 2, 349, 9, '#9ca98c')
  })
  ctx.fillStyle = '#e3ebd8'; ctx.fillRect(36, 373, 288, 1)
  const qr = document.createElement('canvas')
  await QRCode.toCanvas(qr, link, { width: 240, margin: 4, errorCorrectionLevel: 'M', color: { dark: '#36523d', light: '#ffffff' } })
  ctx.imageSmoothingEnabled = false; ctx.drawImage(qr, 244, 382, 80, 80); ctx.imageSmoothingEnabled = true
  ctx.textAlign = 'left'
  text('你能超过我吗？', 36, 404, 12, '#50744d')
  text('扫码看战绩，也来玩一局', 36, 423, 9, '#8fa07f')
  text('轻松玩一局，把快乐带回家', 36, 440, 9, '#8fa07f')
  return new Promise((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error('战绩卡生成失败，请重试。')), 'image/png'))
}
