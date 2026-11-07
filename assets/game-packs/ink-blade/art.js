/* 水墨剑客 · procedural ink-wash art. Every texture is painted at load time from code; nothing is loaded from image files. */
(() => {
  'use strict'

  const PAPER = [236, 229, 211]
  const INK = [22, 22, 24]
  const CINNABAR = [178, 46, 34]
  const MAX_CHUNK = 2048

  const rgba = (c, a) => 'rgba(' + c[0] + ',' + c[1] + ',' + c[2] + ',' + a + ')'
  const lerp = (a, b, t) => a + (b - a) * t
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v))

  function rng(seed) {
    let value = seed >>> 0
    return () => {
      value = (value + 0x6d2b79f5) >>> 0
      let t = value
      t = Math.imul(t ^ (t >>> 15), t | 1)
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296
    }
  }

  function hash(i, seed) {
    const s = Math.sin(i * 127.1 + seed * 311.7) * 43758.5453
    return s - Math.floor(s)
  }

  // Smooth 1D value noise; `period` (lattice cells) makes it tile seamlessly.
  function noise(seed, period = 4096) {
    const random = rng(seed)
    const table = new Float32Array(period)
    for (let i = 0; i < period; i++) table[i] = random()
    return (x) => {
      const i = Math.floor(x), f = x - i
      const a = table[((i % period) + period) % period], b = table[(((i + 1) % period) + period) % period]
      return a + (b - a) * f * f * (3 - 2 * f)
    }
  }

  function fbm(n, x, octaves = 4) {
    let sum = 0, amp = 0.5, freq = 1, norm = 0
    for (let o = 0; o < octaves; o++) { sum += amp * n(x * freq + o * 31.7); norm += amp; amp *= 0.5; freq *= 2 }
    return sum / norm
  }

  function canvas(width, height) {
    const node = document.createElement('canvas')
    node.width = Math.max(1, Math.ceil(width)); node.height = Math.max(1, Math.ceil(height))
    return node
  }

  // A long painted band, split into chunks so huge parallax layers stay within mobile canvas limits.
  function strip(width, top, height, res, paint) {
    const pxW = Math.ceil(width * res), pxH = Math.ceil(height * res)
    const chunkPx = Math.min(MAX_CHUNK, pxW)
    const chunks = []
    for (let x0 = 0; x0 < pxW; x0 += chunkPx) {
      const node = canvas(Math.min(chunkPx, pxW - x0), pxH)
      const g = node.getContext('2d')
      g.setTransform(res, 0, 0, res, -x0, -top * res)
      g.lineCap = 'round'; g.lineJoin = 'round'
      paint(g, x0 / res, (x0 + node.width) / res)
      chunks.push({ node, x: x0 / res, w: node.width / res })
    }
    return { chunks, top, height, width }
  }

  function drawStrip(ctx, layer, offset, viewWidth, period = 0) {
    if (!layer || !Number.isFinite(offset)) return
    if (!period) {
      for (const chunk of layer.chunks) {
        if (chunk.x + chunk.w < offset - 2 || chunk.x > offset + viewWidth + 2) continue
        ctx.drawImage(chunk.node, chunk.x, layer.top, chunk.w, layer.height)
      }
      return
    }
    const first = Math.floor(offset / period)
    for (let n = first; n * period < offset + viewWidth; n++) {
      for (const chunk of layer.chunks) {
        const x = n * period + chunk.x
        if (x + chunk.w < offset - 2 || x > offset + viewWidth + 2) continue
        ctx.drawImage(chunk.node, x, layer.top, chunk.w, layer.height)
      }
    }
  }

  // ------------------------------------------------------------ paper

  function paperGrain(size = 256, seed = 7) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const image = g.createImageData(size, size)
    const random = rng(seed)
    for (let i = 0; i < size * size; i++) {
      const fleck = random()
      const v = 247 - random() * 9 - (fleck > 0.996 ? 26 : 0)
      image.data[i * 4] = v; image.data[i * 4 + 1] = v - 2; image.data[i * 4 + 2] = v - 7; image.data[i * 4 + 3] = 255
    }
    g.putImageData(image, 0, 0)
    // Long mulberry fibres, wrapped so the tile repeats without seams.
    g.lineCap = 'round'
    for (let i = 0; i < 70; i++) {
      const x = random() * size, y = random() * size, a = random() * Math.PI * 2, l = 8 + random() * 30
      g.strokeStyle = 'rgba(150,130,100,' + (0.05 + random() * 0.08) + ')'
      g.lineWidth = 0.4 + random() * 0.6
      for (const ox of [-size, 0, size]) for (const oy of [-size, 0, size]) {
        g.beginPath(); g.moveTo(x + ox, y + oy)
        g.quadraticCurveTo(x + ox + Math.cos(a) * l * 0.5 + 3, y + oy + Math.sin(a) * l * 0.5 - 2, x + ox + Math.cos(a) * l, y + oy + Math.sin(a) * l)
        g.stroke()
      }
    }
    return node
  }

  function vignette(width, height) {
    const w = 320, h = Math.max(120, Math.round(320 * height / Math.max(1, width)) || 180)
    const node = canvas(w, h)
    const g = node.getContext('2d')
    const gradient = g.createRadialGradient(w / 2, h * 0.46, Math.min(w, h) * 0.3, w / 2, h / 2, Math.max(w, h) * 0.66)
    gradient.addColorStop(0, 'rgba(70,52,34,0)')
    gradient.addColorStop(0.75, 'rgba(70,52,34,0.13)')
    gradient.addColorStop(1, 'rgba(48,34,22,0.42)')
    g.fillStyle = gradient; g.fillRect(0, 0, w, h)
    return node
  }

  // ------------------------------------------------------------ brushes

  // A tapered brush polyline: width follows `widthAt(t)`, drawn as one filled ribbon.
  function ribbon(g, points, widthAt) {
    if (points.length < 2) return
    const left = [], right = []
    for (let i = 0; i < points.length; i++) {
      const a = points[Math.max(0, i - 1)], b = points[Math.min(points.length - 1, i + 1)]
      const dx = b[0] - a[0], dy = b[1] - a[1], l = Math.hypot(dx, dy) || 1
      const w = widthAt(i / (points.length - 1)) / 2
      left.push([points[i][0] - dy / l * w, points[i][1] + dx / l * w])
      right.push([points[i][0] + dy / l * w, points[i][1] - dx / l * w])
    }
    g.beginPath(); g.moveTo(left[0][0], left[0][1])
    for (let i = 1; i < left.length; i++) g.lineTo(left[i][0], left[i][1])
    for (let i = right.length - 1; i >= 0; i--) g.lineTo(right[i][0], right[i][1])
    g.closePath(); g.fill()
  }

  // Dry-brush (飞白) stroke: several hair lines with broken dashes following a path function.
  function dryBrush(g, path, width, hairs, color, alpha, seed, steps = 40) {
    const random = rng(seed)
    for (let h = 0; h < hairs; h++) {
      const offset = (h / Math.max(1, hairs - 1) - 0.5) * width
      const edge = Math.abs(offset) / (width / 2 || 1)
      const start = edge * 0.18 * random(), end = 1 - edge * 0.22 * random()
      g.strokeStyle = rgba(color, alpha * (0.35 + random() * 0.65) * (1 - edge * 0.35))
      g.lineWidth = Math.max(0.35, width / hairs * (0.9 + random() * 0.9))
      g.setLineDash([2 + random() * 30, random() < 0.6 ? 0.5 + random() * 3 : 0.01])
      g.lineDashOffset = random() * 20
      g.beginPath()
      for (let i = 0; i <= steps; i++) {
        const t = lerp(start, end, i / steps)
        const [x, y, nx, ny] = path(t)
        if (i === 0) g.moveTo(x + nx * offset, y + ny * offset)
        else g.lineTo(x + nx * offset, y + ny * offset)
      }
      g.stroke()
    }
    g.setLineDash([])
  }

  function leaf(g, x, y, angle, length, width) {
    const c = Math.cos(angle), s = Math.sin(angle)
    const tx = x + c * length, ty = y + s * length
    const mx = x + c * length * 0.42, my = y + s * length * 0.42
    g.beginPath(); g.moveTo(x, y)
    g.quadraticCurveTo(mx - s * width, my + c * width, tx, ty)
    g.quadraticCurveTo(mx + s * width * 0.45, my - c * width * 0.45, x, y)
    g.fill()
  }

  function leafCluster(g, x, y, base, count, length, width, color, alpha, random) {
    for (let i = 0; i < count; i++) {
      const spread = (i - (count - 1) / 2) * (0.42 + random() * 0.2)
      g.fillStyle = rgba(color, alpha * (0.6 + random() * 0.4))
      leaf(g, x, y, base + spread, length * (0.75 + random() * 0.45), width * (0.8 + random() * 0.4))
    }
  }

  // ------------------------------------------------------------ mountains

  function mountains(o) {
    const random = rng(o.seed)
    const peaks = []
    let x = -80
    while (x < o.width + 80) {
      const w = lerp(o.peakW[0], o.peakW[1], random())
      peaks.push({ x, w, h: lerp(o.peakH[0], o.peakH[1], random()) })
      x += w * lerp(o.gap[0], o.gap[1], random())
    }
    const n = noise(o.seed + 11)
    const n2 = noise(o.seed + 23)
    const ridge = (x) => {
      let m = 0
      for (const p of peaks) {
        const t = (x - p.x) / p.w
        if (t > -1 && t < 1) m = Math.max(m, p.h * Math.pow(1 - t * t, o.round))
      }
      return o.base - m - (fbm(n, x / o.detail, 4) - 0.5) * o.detailAmp
    }
    const fade = canvas(1, 128)
    const fg = fade.getContext('2d')
    const grad = fg.createLinearGradient(0, 0, 0, 128)
    grad.addColorStop(0, rgba(o.color, 1)); grad.addColorStop(0.25, rgba(o.color, 0.55)); grad.addColorStop(1, rgba(o.color, 0))
    fg.fillStyle = grad; fg.fillRect(0, 0, 1, 128)
    const bottom = o.top + o.height

    return strip(o.width, o.top, o.height, o.res, (g, u0, u1) => {
      const a = Math.max(0, u0 - 12), b = Math.min(o.width, u1 + 12)
      const ys = []
      for (let x = a; x <= b; x += 1) ys.push(ridge(x))
      // Body wash.
      g.beginPath(); g.moveTo(a, bottom)
      for (let i = 0; i < ys.length; i++) g.lineTo(a + i, ys[i])
      g.lineTo(b, bottom); g.closePath()
      const body = g.createLinearGradient(0, o.base - o.peakH[1], 0, bottom)
      body.addColorStop(0, rgba(o.color, o.alpha * 0.42)); body.addColorStop(1, rgba(o.color, o.alpha * 0.06))
      g.fillStyle = body; g.fill()
      if (o.ochre) {
        const ochre = g.createLinearGradient(0, o.base - o.peakH[1] * 0.4, 0, bottom)
        ochre.addColorStop(0, 'rgba(168,116,64,0)'); ochre.addColorStop(1, 'rgba(168,116,64,' + o.ochre + ')')
        g.fillStyle = ochre; g.fill()
      }
      if (o.indigo) {
        // 花青 wash: cool blue-grey on the upper body, fading downward.
        const indigo = g.createLinearGradient(0, o.base - o.peakH[1], 0, o.base - o.peakH[1] * 0.1)
        indigo.addColorStop(0, 'rgba(70,96,120,' + o.indigo + ')'); indigo.addColorStop(1, 'rgba(70,96,120,0)')
        g.fillStyle = indigo; g.fill()
      }
      // Ink settles along ridges and bleeds downward (墨分五色).
      for (let i = 0; i < ys.length; i++) {
        const x = a + i
        const v = n2(x / 9)
        g.globalAlpha = o.alpha * (0.45 + 0.55 * v)
        g.drawImage(fade, x, ys[i] - 0.5, 1.15, o.depth * (0.55 + 0.9 * n(x / 23 + 50)))
      }
      g.globalAlpha = 1
      // Texture strokes (皴).
      const cell = o.strokeCell
      for (let c = Math.floor(a / cell); c * cell < b; c++) {
        if (hash(c, o.seed) > o.strokeDensity) continue
        const x = c * cell + hash(c, o.seed + 1) * cell
        const y0 = ridge(x)
        const y = y0 + hash(c, o.seed + 2) * o.depth * 0.9
        const slope = ridge(x + 2) - ridge(x - 2)
        const angle = Math.atan2(4, slope) + (hash(c, o.seed + 3) - 0.5) * 0.4
        const len = 3 + hash(c, o.seed + 4) * o.strokeLen
        g.strokeStyle = rgba(o.color, o.alpha * (0.3 + hash(c, o.seed + 5) * 0.5))
        g.lineWidth = 0.5 + hash(c, o.seed + 6) * 1.1
        g.beginPath(); g.moveTo(x, y); g.lineTo(x + Math.cos(angle) * len * 0.5, y + Math.sin(angle) * len); g.stroke()
      }
      // Broken outline.
      if (o.outline) {
        for (let x = a; x < b; x += 5) {
          const v = n(x / 14 + 9)
          if (v < 0.32) continue
          g.strokeStyle = rgba(o.color, o.alpha * o.outline * (0.5 + v * 0.5))
          g.lineWidth = 0.5 + v * 1.8
          g.beginPath(); g.moveTo(x, ridge(x)); g.lineTo(x + 3, ridge(x + 3)); g.lineTo(x + 5.5, ridge(x + 5.5)); g.stroke()
        }
      }
      // Distant trees dotted on ridges (点苔).
      if (o.trees) {
        for (let c = Math.floor(a / 6); c * 6 < b; c++) {
          if (hash(c, o.seed + 40) > o.trees) continue
          const x = c * 6 + hash(c, o.seed + 41) * 6
          const y = ridge(x) + 1
          const h = 3 + hash(c, o.seed + 42) * o.treeH
          g.strokeStyle = rgba(o.color, Math.min(1, o.alpha * 1.4)); g.lineWidth = 0.55
          g.beginPath(); g.moveTo(x, y); g.lineTo(x, y - h); g.stroke()
          g.fillStyle = rgba(o.color, Math.min(1, o.alpha * 1.25))
          for (let k = 0; k < 3; k++) {
            g.beginPath(); g.ellipse(x + (hash(c * 3 + k, 9) - 0.5) * 3, y - h * (0.55 + k * 0.2), 1.5 + hash(c + k, 3) * 1.3, 0.8 + hash(c + k, 5) * 0.6, 0, 0, Math.PI * 2); g.fill()
          }
        }
      }
      if (o.pavilions) for (const px of o.pavilions) if (px > a - 40 && px < b + 40) pavilion(g, px, ridge(px) + 2, o.pavilionScale || 1, o.color, Math.min(1, o.alpha * 1.6))
      if (o.wall && o.wall.to > a - 20 && o.wall.from < b + 20) {
        // Distant Great Wall (长城): a thin crenellated line riding the ridge, with small watchtowers.
        const w = o.wall, wa = w.alpha ?? o.alpha * 1.4
        const x0 = Math.max(a, w.from), x1 = Math.min(b, w.to)
        g.fillStyle = rgba(o.color, wa)
        for (let x = x0; x < x1; x += 1) {
          const y = ridge(x) - 1
          g.fillRect(x, y - 2.6, 1.2, 2.6 + 1.4)
        }
        for (let x = Math.ceil(x0 / 3) * 3; x < x1; x += 3) {
          if (Math.floor(x / 3) % 2) continue
          g.fillRect(x, ridge(x) - 4.4, 1.6, 1.8)
        }
        if (w.towerEvery) {
          for (let tx = Math.ceil(x0 / w.towerEvery) * w.towerEvery; tx < x1; tx += w.towerEvery) {
            const y = ridge(tx) - 1
            g.fillRect(tx - 3, y - 9, 6, 9.5)
            g.fillRect(tx - 4, y - 10.6, 8, 1.8)
            g.fillStyle = rgba(PAPER, 0.5); g.fillRect(tx - 1, y - 6.5, 2, 2.6); g.fillStyle = rgba(o.color, wa)
            for (const cx of [-4, -1.2, 1.6]) g.fillRect(tx + cx, y - 12.2, 1.6, 1.8)
          }
        }
      }
      if (o.snow) {
        // Snowfields are left as bare paper (留白): a ragged cap that sinks deeper on higher peaks, with ink hatching only on the shaded flank.
        const from = o.snowFrom ?? 40, cap = o.snowDepth ?? o.depth * 1.3
        const edge = []
        for (let i = 0; i < ys.length; i += 2) {
          const x = a + i
          const over = Math.max(0, o.base - ys[i] - from)
          const lump = 0.45 + 0.9 * n2(x / 6 + 40) + 0.35 * n(x / 19 + 70)
          edge.push(Math.min(cap, over * 0.9) * lump + 1.2 + 1.8 * n(x / 5 + 11))
        }
        g.beginPath(); g.moveTo(a, ys[0])
        for (let i = 0; i < edge.length; i++) g.lineTo(a + i * 2, ys[i * 2] ?? ys[ys.length - 1])
        for (let i = edge.length - 1; i >= 0; i--) g.lineTo(a + i * 2, (ys[i * 2] ?? ys[ys.length - 1]) + edge[i])
        g.closePath()
        g.fillStyle = 'rgba(250,250,252,' + o.snow + ')'; g.fill()
        for (let i = 1; i < edge.length - 1; i++) {
          const x = a + i * 2, slope = (ys[i * 2 + 2] ?? ys[i * 2]) - (ys[i * 2 - 2] ?? ys[i * 2])
          if (slope < 1.2 || edge[i] < 5 || hash(i + Math.floor(a), o.seed + 90) > 0.6) continue
          g.strokeStyle = rgba(o.color, o.alpha * (0.35 + 0.4 * n2(x / 5)))
          g.lineWidth = 0.6 + 0.8 * n(x / 8)
          const y0 = ys[i * 2] + edge[i] * 0.35
          g.beginPath(); g.moveTo(x, y0); g.lineTo(x + slope * 0.6, y0 + edge[i] * (0.5 + 0.5 * n2(x / 3))); g.stroke()
        }
        g.strokeStyle = rgba(o.color, o.alpha * 0.7); g.lineWidth = 0.7
        for (let i = 0; i < edge.length - 1; i += 3) {
          if (edge[i] < 6) continue
          const x = a + i * 2
          g.beginPath(); g.moveTo(x, ys[i * 2] + edge[i]); g.lineTo(x + 2.5, ys[i * 2] + edge[i] + 2.5 + 4 * n2(x / 4)); g.stroke()
        }
      }
      // Fade the base into the mist below.
      g.globalCompositeOperation = 'destination-out'
      const mist = g.createLinearGradient(0, bottom - o.fadeH, 0, bottom)
      mist.addColorStop(0, 'rgba(0,0,0,0)'); mist.addColorStop(1, 'rgba(0,0,0,1)')
      g.fillStyle = mist; g.fillRect(a, bottom - o.fadeH, b - a, o.fadeH)
      g.globalCompositeOperation = 'source-over'
    })
  }

  function pavilion(g, x, y, s, color, alpha) {
    g.save(); g.translate(x, y); g.scale(s, s)
    g.fillStyle = rgba(color, alpha); g.strokeStyle = rgba(color, alpha)
    g.lineWidth = 0.8
    g.fillRect(-5, -7, 1, 7); g.fillRect(4, -7, 1, 7)
    g.beginPath(); g.moveTo(-10, -7); g.quadraticCurveTo(-4, -8, 0, -13); g.quadraticCurveTo(4, -8, 10, -7); g.quadraticCurveTo(0, -9, -10, -7); g.fill()
    g.beginPath(); g.moveTo(-7, -11); g.quadraticCurveTo(-2, -12, 0, -16); g.quadraticCurveTo(2, -12, 7, -11); g.quadraticCurveTo(0, -12.5, -7, -11); g.fill()
    g.beginPath(); g.moveTo(0, -16); g.lineTo(0, -19); g.stroke()
    g.restore()
  }

  // ------------------------------------------------------------ bamboo

  function bamboo(o) {
    const random = rng(o.seed)
    const stalks = []
    for (let i = 0; i < o.count; i++) {
      stalks.push({
        x: random() * o.period, w: lerp(o.thick[0], o.thick[1], random()), lean: (random() - 0.5) * o.lean,
        alpha: lerp(o.alpha[0], o.alpha[1], random()), seg: lerp(22, 38, random()), seed: random() * 1000,
      })
    }
    stalks.sort((a, b) => a.alpha - b.alpha)
    const top = o.top, bottom = o.bottom
    return strip(o.period, top, bottom - top, o.res, (g, u0, u1) => {
      for (const stalk of stalks) for (const shift of [-o.period, 0, o.period]) {
        const x0 = stalk.x + shift
        if (x0 + 60 < u0 || x0 - 60 > u1) continue
        drawStalk(g, x0, stalk, top, bottom, o)
      }
    })
  }

  function drawStalk(g, x0, stalk, top, bottom, o) {
    const random = rng(Math.floor(stalk.seed * 1000))
    const color = o.color
    let y = bottom
    let w = stalk.w
    while (y > top - 40) {
      const len = stalk.seg * (0.85 + random() * 0.3)
      const y1 = y - len
      const xa = x0 + (bottom - y) * stalk.lean, xb = x0 + (bottom - y1) * stalk.lean
      const w1 = w * 0.985
      g.fillStyle = rgba(color, stalk.alpha)
      g.beginPath()
      g.moveTo(xa - w / 2, y - 0.7); g.lineTo(xb - w1 / 2, y1 + 0.9); g.lineTo(xb + w1 / 2, y1 + 0.9); g.lineTo(xa + w / 2, y - 0.7)
      g.closePath(); g.fill()
      // A pale dry-brush streak gives the stalk its roundness.
      g.fillStyle = rgba(PAPER, 0.25 + random() * 0.2)
      g.beginPath()
      g.moveTo(xa - w * 0.18, y - 2); g.lineTo(xb - w1 * 0.12, y1 + 3); g.lineTo(xb - w1 * 0.02, y1 + 3); g.lineTo(xa - w * 0.06, y - 2)
      g.closePath(); g.fill()
      // Node.
      g.strokeStyle = rgba(color, Math.min(1, stalk.alpha * 1.5))
      g.lineWidth = Math.max(0.7, w * 0.2)
      g.beginPath(); g.moveTo(xb - w1 * 0.7, y1 + 0.3); g.quadraticCurveTo(xb, y1 - 1.2, xb + w1 * 0.7, y1 + 0.6); g.stroke()
      if (o.leaves && y1 < bottom - 90 && random() < o.leaves) {
        const side = random() < 0.5 ? -1 : 1
        const tx = xb + side * (14 + random() * 22), ty = y1 - 6 - random() * 16
        g.strokeStyle = rgba(color, stalk.alpha * 0.9); g.lineWidth = 0.7
        g.beginPath(); g.moveTo(xb, y1); g.quadraticCurveTo((xb + tx) / 2, y1 - 2, tx, ty); g.stroke()
        leafCluster(g, tx, ty, side > 0 ? 0.55 : Math.PI - 0.55, 3 + Math.floor(random() * 3), o.leafLen, o.leafLen * 0.16, color, stalk.alpha * 1.15, random)
      }
      y = y1; w = w1
    }
  }

  // ------------------------------------------------------------ ground

  function ground(o) {
    const n = noise(o.seed, 64)
    const random = rng(o.seed)
    const tufts = [], pebbles = []
    for (let i = 0; i < o.period / 18; i++) tufts.push({ x: random() * o.period, h: 4 + random() * 10, k: 2 + Math.floor(random() * 4), a: 0.35 + random() * 0.45 })
    for (let i = 0; i < o.period / 30; i++) pebbles.push({ x: random() * o.period, y: o.line + 4 + random() * 34, r: 0.8 + random() * 3, a: 0.1 + random() * 0.3 })
    const lineY = (x) => o.line + (n((x / o.period) * 64) - 0.5) * 3
    return strip(o.period, o.line - 20, o.depth + 20, o.res, (g, u0, u1) => {
      const wash = g.createLinearGradient(0, o.line, 0, o.line + o.depth)
      wash.addColorStop(0, rgba(INK, 0.2)); wash.addColorStop(0.3, rgba(INK, 0.08)); wash.addColorStop(1, rgba(INK, 0.02))
      g.fillStyle = wash
      g.beginPath(); g.moveTo(u0 - 2, o.line + o.depth)
      for (let x = u0 - 2; x <= u1 + 2; x += 2) g.lineTo(x, lineY(x))
      g.lineTo(u1 + 2, o.line + o.depth); g.closePath(); g.fill()
      for (const shift of [-o.period, 0, o.period]) {
        dryBrush(g, (t) => {
          const x = shift + t * o.period
          return [x, lineY(x), 0, 1]
        }, 5, 9, INK, 0.85, o.seed + 3, 260)
        for (const p of pebbles) {
          const x = p.x + shift
          if (x < u0 - 10 || x > u1 + 10) continue
          g.fillStyle = rgba(INK, p.a)
          g.beginPath(); g.ellipse(x, p.y, p.r * 1.6, p.r * 0.7, 0, 0, Math.PI * 2); g.fill()
        }
        for (const t of tufts) {
          const x = t.x + shift
          if (x < u0 - 20 || x > u1 + 20) continue
          const y = lineY(x)
          for (let k = 0; k < t.k; k++) {
            const lean = (k - t.k / 2) * 0.35 + 0.15
            g.fillStyle = rgba(INK, t.a)
            leaf(g, x + k * 1.6, y + 1, -Math.PI / 2 + lean, t.h * (0.7 + (k % 2) * 0.4), 1.1)
          }
        }
      }
    })
  }

  // Water band used where the stream crosses the path (断桥).
  function water(o) {
    const random = rng(o.seed)
    return strip(o.period, o.line - 4, o.depth + 4, o.res, (g) => {
      const wash = g.createLinearGradient(0, o.line, 0, o.line + o.depth)
      wash.addColorStop(0, 'rgba(60,72,86,0.18)'); wash.addColorStop(1, 'rgba(60,72,86,0.04)')
      g.fillStyle = wash; g.fillRect(0, o.line, o.period, o.depth)
      for (let i = 0; i < o.period / 9; i++) {
        const x = random() * o.period, y = o.line + 3 + Math.pow(random(), 1.6) * (o.depth - 6)
        const len = 6 + random() * 26 * (1 - (y - o.line) / o.depth * 0.5)
        g.strokeStyle = 'rgba(40,48,58,' + (0.12 + random() * 0.3) + ')'
        g.lineWidth = 0.5 + random() * 0.9
        for (const shift of [-o.period, 0, o.period]) {
          g.beginPath(); g.moveTo(x + shift, y); g.quadraticCurveTo(x + shift + len / 2, y - 1.2, x + shift + len, y); g.stroke()
        }
      }
      dryBrush(g, (t) => [t * o.period, o.line + 0.5, 0, 1], 2.5, 5, [40, 48, 58], 0.5, o.seed + 1, 200)
    })
  }

  // Snow-covered ground: a bright field lit from the horizon, blue-grey drifts, tracks and dry grass breaking through.
  function snowGround(o) {
    const n = noise(o.seed, 64)
    const random = rng(o.seed + 3)
    const lineY = (x) => o.line + (n((x / o.period) * 64) - 0.5) * 4
    const drifts = []
    for (let i = 0; i < o.period / 55; i++) drifts.push({ x: random() * o.period, y: 10 + random() * (o.depth - 24), rx: 26 + random() * 60, ry: 2 + random() * 3.5 })
    const tufts = []
    for (let i = 0; i < o.period / 70; i++) tufts.push({ x: random() * o.period, n: 3 + Math.floor(random() * 4), h: 5 + random() * 11 })
    return strip(o.period, o.line - 20, o.depth + 20, o.res, (g, u0, u1) => {
      const field = g.createLinearGradient(0, o.line, 0, o.line + o.depth)
      field.addColorStop(0, 'rgba(255,255,255,1)')
      field.addColorStop(0.25, 'rgba(246,248,252,1)')
      field.addColorStop(1, 'rgba(222,229,241,1)')
      g.fillStyle = field
      g.beginPath(); g.moveTo(u0 - 2, o.line + o.depth)
      for (let x = u0 - 2; x <= u1 + 2; x += 2) g.lineTo(x, lineY(x))
      g.lineTo(u1 + 2, o.line + o.depth); g.closePath(); g.fill()

      for (const shift of [-o.period, 0, o.period]) {
        for (const d of drifts) {
          const x = d.x + shift
          if (x + d.rx < u0 || x - d.rx > u1) continue
          const y = o.line + d.y
          g.fillStyle = 'rgba(138,154,184,0.16)'
          g.beginPath(); g.ellipse(x, y + d.ry * 0.7, d.rx, d.ry, 0, 0, Math.PI * 2); g.fill()
          g.fillStyle = 'rgba(255,255,255,0.95)'
          g.beginPath(); g.ellipse(x - d.rx * 0.08, y - d.ry * 0.25, d.rx * 0.92, d.ry * 0.75, 0, 0, Math.PI * 2); g.fill()
        }
        dryBrush(g, (t) => {
          const x = shift + t * o.period
          return [x, lineY(x) + 1.5, 0, 1]
        }, 3, 6, [30, 32, 38], 0.8, o.seed + 5, 200)
        for (const tuft of tufts) {
          const x = tuft.x + shift
          if (x < u0 - 16 || x > u1 + 16) continue
          for (let k = 0; k < tuft.n; k++) {
            g.strokeStyle = 'rgba(40,36,34,' + (0.5 + 0.35 * hash(k + tuft.n, o.seed)) + ')'; g.lineWidth = 0.9
            const h = tuft.h * (0.55 + 0.6 * hash(k, o.seed + 8)), lean = (k - tuft.n / 2) * 2.4
            g.beginPath(); g.moveTo(x + k * 1.6, lineY(x) + 2); g.quadraticCurveTo(x + k * 1.6 + lean * 0.4, lineY(x) - h * 0.6, x + k * 1.6 + lean, lineY(x) - h); g.stroke()
          }
        }
        for (let i = 0; i < o.period / 30; i++) {
          const px = shift + (i * 29 + 11) % o.period
          if (px < u0 - 10 || px > u1 + 10) continue
          g.fillStyle = 'rgba(70,84,110,0.2)'
          g.beginPath(); g.ellipse(px, lineY(px) + 6 + (i * 7) % (o.depth - 20), 3.4, 1.2, 0.1, 0, Math.PI * 2); g.fill()
        }
      }
    })
  }

  // Smooth frozen ice lake with crystal reflection and hairline fractures.
  function iceSurface(o) {
    const random = rng(o.seed)
    return strip(o.period, o.line - 6, o.depth + 6, o.res, (g) => {
      const wash = g.createLinearGradient(0, o.line, 0, o.line + o.depth)
      wash.addColorStop(0, 'rgba(215,228,245,0.55)')
      wash.addColorStop(0.2, 'rgba(195,212,235,0.35)')
      wash.addColorStop(1, 'rgba(160,180,210,0.08)')
      g.fillStyle = wash; g.fillRect(0, o.line, o.period, o.depth)

      g.fillStyle = 'rgba(255,255,255,0.65)'
      g.fillRect(0, o.line, o.period, 2)

      g.strokeStyle = 'rgba(255,255,255,0.55)'; g.lineWidth = 0.8
      for (let i = 0; i < o.period / 45; i++) {
        const x = random() * o.period, y = o.line + 2 + random() * (o.depth - 15)
        const len = 15 + random() * 35, a = 0.3 + (random() - 0.5) * 0.8
        for (const shift of [-o.period, 0, o.period]) {
          g.beginPath()
          g.moveTo(x + shift, y)
          g.lineTo(x + shift + Math.cos(a) * len * 0.5, y + Math.sin(a) * len * 0.5 + 1)
          g.lineTo(x + shift + Math.cos(a) * len, y + Math.sin(a) * len)
          g.stroke()
        }
      }
      dryBrush(g, (t) => [t * o.period, o.line + 1, 0, 1], 2, 4, [170, 195, 225], 0.7, o.seed + 2, 180)
    })
  }

  // Thin ice: a translucent sheet over dark water, crazed with fractures. Features are fixed before drawing so the tile wraps seamlessly.
  function thinIce(o) {
    const random = rng(o.seed)
    const plates = []
    for (let i = 0; i < o.period / 38; i++) {
      plates.push({ x: random() * o.period, y: o.line + 3 + random() * (o.depth - 14), rx: 18 + random() * 44, ry: 3 + random() * 6, tilt: (random() - 0.5) * 0.12, alpha: 0.26 + random() * 0.3 })
    }
    const fractures = []
    for (let i = 0; i < o.period / 90; i++) {
      const cx = random() * o.period, cy = o.line + 8 + random() * (o.depth - 24)
      const arms = []
      const count = 5 + Math.floor(random() * 3)
      for (let k = 0; k < count; k++) {
        const angle = random() * Math.PI * 2, len = 14 + random() * 30
        const pts = [[0, 0]]
        let x = 0, y = 0
        for (let seg = 0; seg < 4; seg++) { x += Math.cos(angle + (random() - 0.5) * 0.9) * len / 4; y += Math.sin(angle + (random() - 0.5) * 0.9) * len / 4 * 0.5; pts.push([x, y]) }
        arms.push(pts)
      }
      fractures.push({ cx, cy, arms })
    }
    return strip(o.period, o.line - 4, o.depth + 4, o.res, (g, u0, u1) => {
      const water = g.createLinearGradient(0, o.line, 0, o.line + o.depth)
      water.addColorStop(0, 'rgba(46,66,98,0.62)'); water.addColorStop(0.5, 'rgba(70,92,126,0.4)'); water.addColorStop(1, 'rgba(96,120,152,0.14)')
      g.fillStyle = water; g.fillRect(u0 - 2, o.line, u1 - u0 + 4, o.depth)
      for (const shift of [-o.period, 0, o.period]) {
        for (const plate of plates) {
          g.fillStyle = 'rgba(232,240,250,' + plate.alpha + ')'
          g.beginPath(); g.ellipse(plate.x + shift, plate.y, plate.rx, plate.ry, plate.tilt, 0, Math.PI * 2); g.fill()
        }
      }
      g.fillStyle = 'rgba(244,248,255,0.86)'; g.fillRect(u0 - 2, o.line, u1 - u0 + 4, 2.4)
      for (const shift of [-o.period, 0, o.period]) {
        for (const crack of fractures) {
          for (const pts of crack.arms) {
            const ox = crack.cx + shift, oy = crack.cy
            g.strokeStyle = 'rgba(24,36,60,0.5)'; g.lineWidth = 0.9
            g.beginPath(); g.moveTo(ox, oy); for (const [x, y] of pts) g.lineTo(ox + x, oy + y); g.stroke()
            g.strokeStyle = 'rgba(255,255,255,0.7)'; g.lineWidth = 0.5
            g.beginPath(); g.moveTo(ox, oy - 0.8); for (const [x, y] of pts) g.lineTo(ox + x, oy + y - 0.8); g.stroke()
          }
        }
      }
    })
  }

  // A hanging icicle: origin is where it joins the rock, pointing down.
  function icicleSprite(res) {
    return sprite(28, 44, res, (g) => {
      const body = g.createLinearGradient(-7, 0, 7, 0)
      body.addColorStop(0, 'rgba(176,200,230,0.92)'); body.addColorStop(0.45, 'rgba(244,250,255,0.98)'); body.addColorStop(1, 'rgba(150,176,212,0.9)')
      g.fillStyle = body
      g.beginPath(); g.moveTo(-7, 0); g.lineTo(7, 0); g.lineTo(3.2, 18); g.lineTo(1.2, 38); g.lineTo(0, 43); g.lineTo(-1.6, 36); g.lineTo(-4.2, 16); g.closePath(); g.fill()
      g.strokeStyle = rgba(INK, 0.55); g.lineWidth = 0.9
      g.beginPath(); g.moveTo(7, 0); g.lineTo(3.2, 18); g.lineTo(1.2, 38); g.lineTo(0, 43); g.stroke()
      g.strokeStyle = 'rgba(255,255,255,0.95)'; g.lineWidth = 0.9
      g.beginPath(); g.moveTo(-3.6, 3); g.lineTo(-1.8, 24); g.stroke()
      g.strokeStyle = 'rgba(120,146,184,0.5)'; g.lineWidth = 0.6
      g.beginPath(); g.moveTo(2, 4); g.lineTo(1, 28); g.stroke()
    }, 14, 0)
  }

  // Winter reeds (枯芦 / 雪苇) swaying in wind.
  function reeds(o) {
    const random = rng(o.seed)
    const period = o.period || 900
    const clumps = []
    for (let i = 0; i < o.count; i++) {
      const x = random() * period
      const h = (o.height || 70) * (0.75 + random() * 0.5)
      const count = 3 + Math.floor(random() * 5)
      const stalks = []
      for (let k = 0; k < count; k++) {
        stalks.push({
          dx: (k - count / 2) * 3 + (random() - 0.5) * 2,
          h: h * (0.8 + random() * 0.4),
          lean: 0.18 + (random() - 0.5) * 0.15,
          a: (o.alpha || 0.6) * (0.6 + random() * 0.4),
        })
      }
      clumps.push({ x, stalks })
    }
    const top = o.top, bottom = o.bottom
    return strip(period, top, bottom - top, o.res, (g, u0, u1) => {
      for (const clump of clumps) for (const shift of [-period, 0, period]) {
        const cx = clump.x + shift
        if (cx + 50 < u0 || cx - 50 > u1) continue
        for (const s of clump.stalks) {
          const sx = cx + s.dx
          const tx = sx + (bottom - (bottom - s.h)) * s.lean
          const ty = bottom - s.h
          g.strokeStyle = 'rgba(45,42,40,' + s.a + ')'; g.lineWidth = 1.1
          g.beginPath(); g.moveTo(sx, bottom); g.quadraticCurveTo((sx + tx) / 2, (bottom + ty) / 2 + 5, tx, ty); g.stroke()
          g.fillStyle = 'rgba(225,220,215,' + (s.a * 0.85) + ')'
          g.beginPath()
          g.ellipse(tx + 4, ty - 3, 11, 3.2, 0.45, 0, Math.PI * 2)
          g.fill()
        }
      }
    })
  }

  // ------------------------------------------------------------ props

  function sprite(width, height, res, paint, originX = 0, originY = 0) {
    const node = canvas(width * res, height * res)
    const g = node.getContext('2d')
    g.setTransform(res, 0, 0, res, originX * res, originY * res)
    g.lineCap = 'round'; g.lineJoin = 'round'
    paint(g)
    return { node, width, height, ox: originX, oy: originY }
  }

  function drawSprite(ctx, s, x, y, alpha = 1) {
    if (!s) return
    if (alpha !== 1) ctx.globalAlpha = alpha
    ctx.drawImage(s.node, x - s.ox, y - s.oy, s.width, s.height)
    if (alpha !== 1) ctx.globalAlpha = 1
  }

  function rock(w, topY, groundY, seed, res) {
    const random = rng(seed)
    const h = groundY - topY
    const pad = 16
    return sprite(w + pad * 2, h + 26, res, (g) => {
      const pts = []
      const steps = 14
      for (let i = 0; i <= steps; i++) pts.push([lerp(2, w - 2, i / steps), -1 + (random() - 0.5) * 2.4])
      const right = [[w + 4 + random() * 6, h * 0.35], [w + 8 + random() * 6, h * 0.75], [w + 10, h + 4]]
      const left = [[-10, h + 4], [-7 - random() * 6, h * 0.7], [-3 - random() * 4, h * 0.3]]
      const outline = [[0, 2], ...pts, [w, 2], ...right, ...left]
      const body = g.createLinearGradient(0, 0, 0, h)
      body.addColorStop(0, rgba(INK, 0.86)); body.addColorStop(0.18, rgba(INK, 0.5)); body.addColorStop(1, rgba(INK, 0.16))
      g.fillStyle = body
      g.beginPath(); g.moveTo(outline[0][0], outline[0][1])
      for (let i = 1; i < outline.length; i++) {
        const p = outline[i], q = outline[i - 1]
        g.quadraticCurveTo(q[0], q[1], (p[0] + q[0]) / 2, (p[1] + q[1]) / 2)
      }
      g.closePath(); g.fill()
      // Axe-cut texture strokes (斧劈皴).
      for (let i = 0; i < w * h / 160; i++) {
        const x = random() * w, y = 6 + random() * (h - 8)
        g.fillStyle = rgba(INK, 0.12 + random() * 0.35)
        g.beginPath()
        g.moveTo(x, y); g.lineTo(x + 3 + random() * 6, y + 2 + random() * 10); g.lineTo(x + 1 + random() * 2, y + 1); g.closePath(); g.fill()
      }
      for (const side of [left, right]) {
        g.strokeStyle = rgba(INK, 0.65); g.lineWidth = 1.6
        g.beginPath(); g.moveTo(side[0][0], side[0][1])
        for (const p of side) g.lineTo(p[0], p[1])
        g.stroke()
      }
      dryBrush(g, (t) => [lerp(-2, w + 2, t), Math.sin(t * Math.PI) * -1.2, 0, 1], 3.2, 6, INK, 0.95, seed + 3, 30)
      // Moss dots on top (点苔).
      for (let i = 0; i < w / 9; i++) {
        const x = random() * w
        g.fillStyle = rgba(INK, 0.6 + random() * 0.4)
        g.beginPath(); g.ellipse(x, -2 - random() * 2, 1.2 + random() * 1.6, 0.9 + random(), 0, 0, Math.PI * 2); g.fill()
      }
    }, pad, 4)
  }

  function bridge(w, topY, groundY, res, seed) {
    const random = rng(seed)
    const h = groundY - topY
    return sprite(w + 20, h + 30, res, (g) => {
      for (const px of [10, w * 0.5, w - 10]) {
        g.fillStyle = rgba(INK, 0.7)
        g.fillRect(px - 2.5, 0, 5, h + 16)
        g.fillStyle = rgba(INK, 0.15)
        g.beginPath(); g.ellipse(px, h + 16, 9, 2, 0, 0, Math.PI * 2); g.fill()
      }
      g.strokeStyle = rgba(INK, 0.5); g.lineWidth = 1
      for (let i = 0; i < 2; i++) { g.beginPath(); g.moveTo(10, 8 + i * 18); g.lineTo(w - 10, 26 - i * 18); g.stroke() }
      g.fillStyle = rgba(INK, 0.9)
      g.fillRect(-4, -1, w + 8, 6)
      for (let x = 0; x < w; x += 9 + random() * 4) { g.fillStyle = rgba(PAPER, 0.35); g.fillRect(x, 0, 0.8, 5) }
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 1.4
      g.beginPath(); g.moveTo(-2, -16); g.lineTo(w + 2, -16); g.stroke()
      for (let x = 0; x <= w; x += w / 5) { g.beginPath(); g.moveTo(x, 0); g.lineTo(x, -17); g.stroke() }
    }, 10, 18)
  }

  function gate(res) {
    // Three-bay memorial archway (三间四柱牌坊): a raised central roof over two lower side roofs.
    return sprite(300, 244, res, (g) => {
      const fill = (a) => { g.fillStyle = rgba(INK, a) }
      const roof = (cx, half, y, depth, alpha) => {
        fill(alpha)
        g.beginPath()
        g.moveTo(cx - half - 16, y - 11)
        g.quadraticCurveTo(cx - half + 2, y + 3, cx - half + 14, y + 3)
        g.lineTo(cx + half - 14, y + 3)
        g.quadraticCurveTo(cx + half - 2, y + 3, cx + half + 16, y - 11)
        g.quadraticCurveTo(cx + half - 2, y - depth * 0.45, cx + half * 0.8, y - depth)
        g.lineTo(cx - half * 0.8, y - depth)
        g.quadraticCurveTo(cx - half + 2, y - depth * 0.45, cx - half - 16, y - 11)
        g.fill()
        g.strokeStyle = rgba(PAPER, 0.22); g.lineWidth = 0.7
        for (let x = cx - half * 0.78; x < cx + half * 0.78; x += 6) { g.beginPath(); g.moveTo(x, y - depth + 2); g.lineTo(x + (x - cx) * 0.08, y); g.stroke() }
        g.strokeStyle = rgba(INK, 0.95); g.lineWidth = 3.4
        g.beginPath(); g.moveTo(cx - half * 0.86, y - depth); g.lineTo(cx + half * 0.86, y - depth); g.stroke()
        g.lineWidth = 2
        for (const side of [-1, 1]) {
          g.beginPath(); g.arc(cx + side * half * 0.86, y - depth - 4, 4, side > 0 ? Math.PI * 0.9 : -Math.PI * 0.1, side > 0 ? Math.PI * 2.2 : Math.PI * 1.2, side < 0); g.stroke()
        }
        fill(alpha * 0.8)
        for (let x = cx - half + 10; x <= cx + half - 10; x += 9) g.fillRect(x - 2.5, y + 3, 5, 5)
      }
      fill(0.12); g.beginPath(); g.ellipse(0, 0, 140, 4, 0, 0, Math.PI * 2); g.fill()
      for (const [x, top, w] of [[-104, 152, 10], [104, 152, 10], [-44, 186, 12], [44, 186, 12]]) {
        fill(0.86); g.fillRect(x - w / 2, -top, w, top)
        g.fillStyle = rgba(PAPER, 0.25); g.fillRect(x - w / 2 + 2, -top + 10, 1.6, top - 22)
        fill(0.5); g.beginPath(); g.roundRect(x - w / 2 - 5, -15, w + 10, 15, 2); g.fill()
        fill(0.62); g.beginPath(); g.arc(x + (x < 0 ? -1 : 1) * (w / 2 + 6), -11, 8, 0, Math.PI * 2); g.fill()
      }
      fill(0.85); g.fillRect(-112, -116, 224, 7); g.fillRect(-50, -158, 100, 8)
      fill(0.65); g.fillRect(-112, -132, 64, 5); g.fillRect(48, -132, 64, 5)
      roof(-74, 46, -152, 22, 0.9)
      roof(74, 46, -152, 22, 0.9)
      roof(0, 66, -190, 30, 0.95)
      g.fillStyle = rgba(PAPER, 0.94); g.fillRect(-30, -152, 60, 30)
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2.4; g.strokeRect(-30, -152, 60, 30)
      g.lineWidth = 0.8; g.strokeRect(-26, -148, 52, 22)
      g.fillStyle = rgba(CINNABAR, 0.95)
      g.font = '600 15px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'; g.textAlign = 'center'; g.textBaseline = 'middle'
      g.fillText('竹 林', 0, -136)
    }, 150, 240)
  }

  function lantern(res) {
    return sprite(30, 62, res, (g) => {
      g.fillStyle = rgba(INK, 0.8)
      g.fillRect(-4, -26, 8, 26)
      g.fillRect(-9, -2, 18, 3)
      g.beginPath(); g.moveTo(-12, -36); g.lineTo(12, -36); g.lineTo(8, -27); g.lineTo(-8, -27); g.closePath(); g.fill()
      g.beginPath(); g.moveTo(-14, -42); g.quadraticCurveTo(0, -52, 14, -42); g.lineTo(11, -40); g.lineTo(-11, -40); g.closePath(); g.fill()
      g.fillStyle = rgba(INK, 0.6); g.fillRect(-7, -40, 14, 4)
      g.fillStyle = 'rgba(214,120,62,0.75)'; g.fillRect(-4, -35, 8, 6)
      g.beginPath(); g.arc(0, -55, 2.2, 0, Math.PI * 2); g.fillStyle = rgba(INK, 0.8); g.fill()
    }, 15, 60)
  }

  function deadTree(res, seed, scale = 1, opts = {}) {
    const random = rng(seed)
    const half = opts.wide ? 170 : 110
    return sprite(half * 2 * scale, 260 * scale, res, (g) => {
      g.scale(scale, scale)
      g.translate(-110, -258)
      const branch = (x, y, angle, len, width, depth) => {
        const x2 = x + Math.cos(angle) * len, y2 = y + Math.sin(angle) * len
        g.fillStyle = rgba(INK, 0.85)
        ribbon(g, [[x, y], [(x + x2) / 2 + (random() - 0.5) * len * 0.2, (y + y2) / 2], [x2, y2]], (t) => width * (1 - t * 0.55))
        if (opts.snow && width > 2 && Math.abs(Math.cos(angle)) > 0.45) {
          g.fillStyle = 'rgba(252,252,255,0.96)'
          g.beginPath(); g.ellipse((x + x2) / 2, (y + y2) / 2 - width * 0.4, len * 0.34, Math.max(1.4, width * 0.34), Math.atan2(y2 - y, x2 - x), 0, Math.PI * 2); g.fill()
        }
        if (opts.blossom && depth <= 1) {
          const count = 2 + Math.floor(random() * 3)
          for (let k = 0; k < count; k++) blossom(g, x2 + (random() - 0.5) * 18, y2 + (random() - 0.5) * 14, 2 + random() * 1.5, random)
        }
        if (depth <= 0 || width < 0.8) return
        const count = depth > 2 ? 2 : 1 + Math.floor(random() * 2)
        for (let i = 0; i < count; i++) branch(x2, y2, angle + (random() - 0.5) * 1.3 - 0.1, len * (0.55 + random() * 0.25), width * 0.62, depth - 1)
      }
      branch(110, 258, -Math.PI / 2 + 0.05, 90, 12, 5)
      branch(110, 200, -Math.PI / 2 - 0.7, 50, 5, 3)
      g.fillStyle = rgba(INK, 0.15); g.beginPath(); g.ellipse(110, 258, 40, 3, 0, 0, Math.PI * 2); g.fill()
    }, half * scale, 258 * scale)
  }

  function pine(res, seed) {
    const random = rng(seed)
    return sprite(420, 330, res, (g) => {
      g.translate(-150, -328)
      g.fillStyle = rgba(INK, 0.9)
      const trunk = []
      for (let i = 0; i <= 14; i++) {
        const t = i / 14
        trunk.push([150 + Math.sin(t * 3.2) * 34 + t * 60, 328 - t * 240])
      }
      ribbon(g, trunk, (t) => 22 * (1 - t * 0.7))
      for (let i = 0; i < 40; i++) {
        const t = 0.2 + random() * 0.75
        const p = trunk[Math.floor(t * 14)]
        g.fillStyle = rgba(PAPER, 0.25)
        g.beginPath(); g.ellipse(p[0] + (random() - 0.5) * 8, p[1], 2 + random() * 3, 1, random(), 0, Math.PI * 2); g.fill()
      }
      const arms = [[0.45, -1, 150], [0.62, 1, 160], [0.8, -1, 120], [0.93, 1, 100], [0.98, -1, 70]]
      for (const [t, side, len] of arms) {
        const p = trunk[Math.round(t * 14)]
        const end = [p[0] + side * len, p[1] - 22 - random() * 20]
        g.fillStyle = rgba(INK, 0.9)
        ribbon(g, [p, [(p[0] + end[0]) / 2, p[1] - 10], end], (u) => 8 * (1 - u * 0.7))
        // Needle pads: layered dark ellipses with radiating needle strokes.
        for (let k = 0; k < 4; k++) {
          const cx = p[0] + side * len * (0.45 + k * 0.18), cy = p[1] - 26 - k * 4 - random() * 8
          g.fillStyle = rgba(INK, 0.55 + random() * 0.3)
          g.beginPath(); g.ellipse(cx, cy, 26 + random() * 10, 8 + random() * 4, (random() - 0.5) * 0.2, 0, Math.PI * 2); g.fill()
          g.strokeStyle = rgba(INK, 0.6); g.lineWidth = 0.6
          for (let m = 0; m < 22; m++) {
            const a = -Math.PI + random() * Math.PI, r = 18 + random() * 14
            g.beginPath(); g.moveTo(cx, cy + 2); g.lineTo(cx + Math.cos(a) * r, cy + Math.sin(a) * r * 0.5); g.stroke()
          }
        }
      }
    }, 150, 328)
  }

  function snowPine(res, seed) {
    const random = rng(seed)
    return sprite(420, 330, res, (g) => {
      g.translate(-150, -328)
      g.fillStyle = rgba(INK, 0.9)
      const trunk = []
      for (let i = 0; i <= 14; i++) {
        const t = i / 14
        trunk.push([150 + Math.sin(t * 3.2) * 34 + t * 60, 328 - t * 240])
      }
      ribbon(g, trunk, (t) => 22 * (1 - t * 0.7))
      for (let i = 0; i < 40; i++) {
        const t = 0.2 + random() * 0.75
        const p = trunk[Math.floor(t * 14)]
        g.fillStyle = rgba(PAPER, 0.25)
        g.beginPath(); g.ellipse(p[0] + (random() - 0.5) * 8, p[1], 2 + random() * 3, 1, random(), 0, Math.PI * 2); g.fill()
      }
      const arms = [[0.45, -1, 150], [0.62, 1, 160], [0.8, -1, 120], [0.93, 1, 100], [0.98, -1, 70]]
      for (const [t, side, len] of arms) {
        const p = trunk[Math.round(t * 14)]
        const end = [p[0] + side * len, p[1] - 22 - random() * 20]
        g.fillStyle = rgba(INK, 0.9)
        ribbon(g, [p, [(p[0] + end[0]) / 2, p[1] - 10], end], (u) => 8 * (1 - u * 0.7))
        for (let k = 0; k < 4; k++) {
          const cx = p[0] + side * len * (0.45 + k * 0.18), cy = p[1] - 26 - k * 4 - random() * 8
          g.fillStyle = rgba(INK, 0.3 + random() * 0.2)
          g.beginPath(); g.ellipse(cx, cy + 2, 22 + random() * 8, 6 + random() * 3, (random() - 0.5) * 0.2, 0, Math.PI * 2); g.fill()
          g.strokeStyle = rgba(INK, 0.72); g.lineWidth = 0.7
          for (let m = 0; m < 34; m++) {
            const a = -Math.PI * 0.95 + random() * Math.PI * 0.9, r = 14 + random() * 18
            g.beginPath(); g.moveTo(cx, cy); g.quadraticCurveTo(cx + Math.cos(a) * r * 0.6, cy + Math.sin(a) * r * 0.2, cx + Math.cos(a) * r, cy + Math.sin(a) * r * 0.55 + r * 0.3); g.stroke()
          }
          g.fillStyle = 'rgba(252,252,255,0.95)'
          for (let m = 0; m < 3; m++) {
            g.beginPath(); g.ellipse(cx + (m - 1) * 10 + (random() - 0.5) * 4, cy - 3.5 - (m === 1 ? 1.5 : 0), 8 + random() * 6, 3 + random() * 1.6, (random() - 0.5) * 0.3, 0, Math.PI * 2); g.fill()
          }
        }
      }
    }, 150, 328)
  }

  function snowPier(res) {
    return sprite(160, 90, res, (g) => {
      g.translate(-80, -85)
      g.fillStyle = 'rgba(210,225,240,0.3)'; g.fillRect(0, 85, 160, 5)
      for (const [px, ph, pw] of [[30, 68, 12], [65, 54, 10], [100, 72, 13], [130, 48, 9]]) {
        g.fillStyle = rgba(INK, 0.85); g.fillRect(px - pw / 2, 85 - ph, pw, ph)
        g.fillStyle = 'rgba(252,252,255,0.95)'
        g.beginPath(); g.ellipse(px, 85 - ph - 2, pw * 0.7, 4, 0, 0, Math.PI * 2); g.fill()
        g.strokeStyle = 'rgba(220,210,195,0.5)'; g.lineWidth = 1.4
        g.beginPath(); g.moveTo(px - pw / 2 - 1, 85 - ph + 16); g.lineTo(px + pw / 2 + 1, 85 - ph + 18); g.stroke()
      }
      g.fillStyle = rgba(INK, 0.78); g.fillRect(20, 48, 120, 8)
      g.fillStyle = 'rgba(252,252,255,0.92)'; g.fillRect(22, 45, 116, 4)
    }, 80, 85)
  }

  function pillar(res, seed, snowy = false) {
    const random = rng(seed)
    return sprite(40, 120, res, (g) => {
      g.fillStyle = rgba(INK, 0.75)
      g.beginPath(); g.moveTo(-9, 0); g.lineTo(-8, -88); g.lineTo(-2, -94); g.lineTo(3, -86); g.lineTo(8, -97); g.lineTo(9, 0); g.closePath(); g.fill()
      for (let i = 0; i < 12; i++) { g.fillStyle = rgba(PAPER, 0.18 + random() * 0.2); g.fillRect(-6 + random() * 10, -random() * 85, 1, 6 + random() * 10) }
      g.fillStyle = rgba(INK, 0.15); g.beginPath(); g.ellipse(0, 0, 16, 2.5, 0, 0, Math.PI * 2); g.fill()
      if (snowy) {
        g.fillStyle = 'rgba(252,252,255,0.96)'
        g.beginPath(); g.moveTo(-8.5, -86); g.lineTo(-8, -88); g.lineTo(-2, -95); g.lineTo(3, -88); g.lineTo(8, -99); g.lineTo(9.5, -88); g.quadraticCurveTo(0, -83, -8.5, -86); g.fill()
        g.beginPath(); g.ellipse(0, -1, 17, 3.6, 0, 0, Math.PI * 2); g.fill()
      }
    }, 20, 116)
  }

  function stele(res, snowy = false) {
    return sprite(40, 70, res, (g) => {
      g.fillStyle = rgba(INK, 0.72)
      g.beginPath(); g.moveTo(-11, 0); g.lineTo(-10, -44); g.quadraticCurveTo(0, -54, 10, -44); g.lineTo(11, 0); g.closePath(); g.fill()
      g.fillStyle = rgba(INK, 0.5); g.fillRect(-15, -4, 30, 4)
      g.fillStyle = rgba(PAPER, 0.55)
      for (let i = 0; i < 4; i++) g.fillRect(-1.2, -38 + i * 8, 2.4, 5)
      if (snowy) {
        g.fillStyle = 'rgba(252,252,255,0.97)'
        g.beginPath(); g.moveTo(-11.5, -43); g.quadraticCurveTo(0, -59, 11.5, -43); g.quadraticCurveTo(6, -46, 0, -44.5); g.quadraticCurveTo(-6, -47, -11.5, -43); g.fill()
        g.beginPath(); g.ellipse(0, -2, 19, 3.6, 0, 0, Math.PI * 2); g.fill()
      }
    }, 20, 66)
  }

  function shrine(res, snowy = false) {
    return sprite(164, 140, res, (g) => {
      g.translate(-75, -138)
      g.fillStyle = rgba(INK, 0.55)
      g.fillRect(16, 60, 8, 78); g.fillRect(126, 60, 8, 78)
      g.fillRect(10, 128, 130, 10)
      g.fillStyle = rgba(INK, 0.72)
      g.beginPath(); g.moveTo(-6, 58); g.quadraticCurveTo(30, 52, 75, 22); g.quadraticCurveTo(120, 52, 156, 58); g.quadraticCurveTo(140, 66, 128, 64); g.lineTo(22, 64); g.quadraticCurveTo(10, 66, -6, 58); g.fill()
      g.fillStyle = 'rgba(214,120,62,0.45)'; g.fillRect(60, 86, 30, 22)
      g.fillStyle = rgba(INK, 0.5); g.fillRect(60, 84, 30, 3); g.fillRect(74, 86, 2, 22)
      if (snowy) {
        g.fillStyle = 'rgba(252,252,255,0.96)'
        g.beginPath(); g.moveTo(-6, 57); g.quadraticCurveTo(30, 49, 75, 18); g.quadraticCurveTo(120, 49, 156, 57); g.quadraticCurveTo(122, 55, 75, 28); g.quadraticCurveTo(28, 55, -6, 57); g.fill()
        g.beginPath(); g.ellipse(75, 130, 74, 6, 0, 0, Math.PI * 2); g.fill()
      }
    }, 75, 138)
  }

  function blossom(g, x, y, rad, random) {
    const rot = random() * Math.PI * 2
    g.fillStyle = rgba(CINNABAR, 0.9)
    for (let k = 0; k < 5; k++) {
      const a = rot + k * 1.2566
      g.beginPath(); g.ellipse(x + Math.cos(a) * rad * 0.8, y + Math.sin(a) * rad * 0.8, rad * 0.62, rad * 0.5, a, 0, Math.PI * 2); g.fill()
    }
    g.fillStyle = 'rgba(250,226,156,0.95)'
    g.beginPath(); g.arc(x, y, rad * 0.3, 0, Math.PI * 2); g.fill()
  }

  // Snow-laden boulder used as a platform: ink body, a thick overhanging cap of bare paper, icicles and a drift at the foot.
  function snowRock(w, topY, groundY, seed, res) {
    const random = rng(seed)
    const h = groundY - topY
    const pad = 16
    return sprite(w + pad * 2, h + 36, res, (g) => {
      const pts = []
      const steps = 14
      for (let i = 0; i <= steps; i++) pts.push([lerp(2, w - 2, i / steps), -1 + (random() - 0.5) * 2.4])
      const right = [[w + 4 + random() * 6, h * 0.35], [w + 8 + random() * 6, h * 0.75], [w + 10, h + 4]]
      const left = [[-10, h + 4], [-7 - random() * 6, h * 0.7], [-3 - random() * 4, h * 0.3]]
      const outline = [[0, 2], ...pts, [w, 2], ...right, ...left]
      const body = g.createLinearGradient(0, 0, 0, h)
      body.addColorStop(0, rgba(INK, 0.82)); body.addColorStop(0.3, rgba([40, 48, 66], 0.6)); body.addColorStop(1, rgba([70, 84, 110], 0.3))
      g.beginPath(); g.moveTo(outline[0][0], outline[0][1])
      for (let i = 1; i < outline.length; i++) {
        const p = outline[i], q = outline[i - 1]
        g.quadraticCurveTo(q[0], q[1], (p[0] + q[0]) / 2, (p[1] + q[1]) / 2)
      }
      g.closePath()
      g.fillStyle = 'rgba(216,224,238,0.97)'; g.fill()
      g.fillStyle = body; g.fill()
      for (let i = 0; i < w * h / 190; i++) {
        const x = random() * w, y = 8 + random() * (h - 10)
        g.fillStyle = rgba(INK, 0.1 + random() * 0.3)
        g.beginPath()
        g.moveTo(x, y); g.lineTo(x + 3 + random() * 6, y + 2 + random() * 10); g.lineTo(x + 1 + random() * 2, y + 1); g.closePath(); g.fill()
      }
      for (const side of [left, right]) {
        g.strokeStyle = rgba(INK, 0.6); g.lineWidth = 1.5
        g.beginPath(); g.moveTo(side[0][0], side[0][1])
        for (const p of side) g.lineTo(p[0], p[1])
        g.stroke()
      }
      const cap = 16
      g.fillStyle = 'rgba(253,253,255,0.98)'
      g.beginPath(); g.moveTo(-7, 4)
      for (let i = 0; i <= cap; i++) g.lineTo(lerp(-7, w + 7, i / cap), -0.5 - 2 * Math.sin((i / cap) * Math.PI) - random() * 0.8)
      for (let i = cap; i >= 0; i--) g.lineTo(lerp(-7, w + 7, i / cap), 6 + 4 * random() + (i % 2 ? 4 * random() : 0))
      g.closePath(); g.fill()
      g.strokeStyle = 'rgba(130,148,182,0.35)'; g.lineWidth = 0.8
      g.beginPath()
      for (let i = 1; i < cap; i += 1) { const x = lerp(-7, w + 7, i / cap); g.moveTo(x, 7 + random() * 3); g.lineTo(x + 3, 8 + random() * 4) }
      g.stroke()
      g.fillStyle = 'rgba(253,253,255,0.95)'
      g.beginPath(); g.ellipse(w / 2, h + 3, w * 0.62, 4.5, 0, 0, Math.PI * 2); g.fill()
      g.fillStyle = 'rgba(138,154,184,0.22)'
      g.beginPath(); g.ellipse(w / 2 + 6, h + 6, w * 0.55, 2.4, 0, 0, Math.PI * 2); g.fill()
    }, pad, 8)
  }

  // Distant snow pines in layered drooping tiers, each tier carrying its own cap of snow.
  function snowForest(o) {
    const random = rng(o.seed)
    const trees = []
    for (let i = 0; i < o.count; i++) {
      trees.push({ x: random() * o.period, h: lerp(o.height[0], o.height[1], random()), tiers: 4 + Math.floor(random() * 3), lean: (random() - 0.5) * 0.08, w: lerp(0.26, 0.38, random()) })
    }
    return strip(o.period, o.top, o.base - o.top + 4, o.res, (g, u0, u1) => {
      for (const shift of [-o.period, 0, o.period]) for (const t of trees) {
        const x0 = t.x + shift
        if (x0 + t.h * t.w < u0 || x0 - t.h * t.w > u1) continue
        g.strokeStyle = rgba(o.color, Math.min(1, o.alpha * 1.1)); g.lineWidth = Math.max(0.8, t.h * 0.024)
        g.beginPath(); g.moveTo(x0, o.base); g.lineTo(x0 + t.lean * t.h, o.base - t.h); g.stroke()
        for (let k = 0; k < t.tiers; k++) {
          const f = k / t.tiers
          const y = o.base - t.h * (0.16 + 0.8 * f)
          const half = t.h * t.w * (1 - f * 0.82) * (0.8 + 0.2 * hash(k, o.seed))
          const cx = x0 + t.lean * (o.base - y)
          g.fillStyle = rgba(o.color, o.alpha * (0.75 + 0.25 * hash(k + 1, o.seed + 2)))
          g.beginPath()
          g.moveTo(cx - half, y + half * 0.35)
          g.quadraticCurveTo(cx - half * 0.35, y - half * 0.05, cx, y - half * 0.3)
          g.quadraticCurveTo(cx + half * 0.35, y - half * 0.05, cx + half, y + half * 0.35)
          g.quadraticCurveTo(cx, y + half * 0.12, cx - half, y + half * 0.35)
          g.fill()
          g.fillStyle = 'rgba(252,252,255,' + (o.snow ?? 0.92) + ')'
          g.beginPath()
          g.moveTo(cx - half * 0.78, y + half * 0.2)
          g.quadraticCurveTo(cx - half * 0.3, y - half * 0.1, cx, y - half * 0.3)
          g.quadraticCurveTo(cx + half * 0.3, y - half * 0.1, cx + half * 0.78, y + half * 0.2)
          g.quadraticCurveTo(cx + half * 0.2, y + half * 0.04, cx, y - half * 0.06)
          g.quadraticCurveTo(cx - half * 0.2, y + half * 0.06, cx - half * 0.78, y + half * 0.2)
          g.fill()
        }
      }
    })
  }

  // Foreground: heavy plum boughs hanging in from the top edge, snow on their backs and cinnabar blossoms at the tips.
  function plumBranches(o) {
    const random = rng(o.seed)
    const items = []
    for (let i = 0; i < o.count; i++) items.push({ x: (i + 0.2 + random() * 0.6) * (o.period / o.count), seed: Math.floor(random() * 1e6) + 1, dir: random() < 0.5 ? -1 : 1, len: 150 + random() * 60 })
    return strip(o.period, o.top, o.height, o.res, (g, u0, u1) => {
      for (const shift of [-o.period, 0, o.period]) for (const it of items) {
        const x0 = it.x + shift
        if (x0 + 280 < u0 || x0 - 280 > u1) continue
        const r = rng(it.seed)
        const bough = (x, y, angle, len, width, depth) => {
          const N = 6, pts = [[x, y]]
          let heading = angle, cx = x, cy = y
          for (let i = 0; i < N; i++) {
            heading += (Math.PI / 2 - heading) * 0.16 + (r() - 0.5) * 0.35
            cx += Math.cos(heading) * len / N; cy += Math.sin(heading) * len / N
            pts.push([cx, cy])
          }
          g.fillStyle = rgba(INK, 0.82)
          ribbon(g, pts, (t) => width * (1 - t * 0.65))
          for (let i = 1; i <= N; i++) {
            const [px, py] = pts[i - 1], [qx, qy] = pts[i]
            const seg = Math.atan2(qy - py, qx - px), w = width * (1 - (i / N) * 0.65)
            if (w > 2.2 && Math.abs(Math.cos(seg)) > 0.4) {
              g.fillStyle = 'rgba(252,252,255,0.96)'
              g.beginPath(); g.ellipse((px + qx) / 2, (py + qy) / 2 - w * 0.4, len / N * 0.62, Math.max(1.5, w * 0.36), seg, 0, Math.PI * 2); g.fill()
            }
            if (depth > 0 && i > 1 && i < N && r() < 0.55) bough(qx, qy, seg + (r() < 0.5 ? -1 : 1) * (0.6 + r() * 0.5), len * 0.5, width * 0.5, depth - 1)
          }
          if (depth <= 1) {
            for (const [bx, by] of [pts[N], pts[N - 1], pts[N - 2]]) {
              const count = 2 + Math.floor(r() * 3)
              for (let k = 0; k < count; k++) blossom(g, bx + (r() - 0.5) * 20, by + (r() - 0.5) * 16, 2.2 + r() * 1.6, r)
            }
          }
        }
        bough(x0, o.top + 112, Math.PI / 2 + it.dir * 0.85, it.len, 12, 2)
      }
    })
  }

  // A drifting band of pale ink wash for a winter sky.
  function inkCloud(seed) {
    const random = rng(seed)
    const node = canvas(512, 128)
    const g = node.getContext('2d')
    for (let i = 0; i < 8; i++) {
      const cx = 70 + random() * 372, cy = 50 + random() * 36, rx = 60 + random() * 90, ry = 18 + random() * 22
      g.save(); g.translate(cx, cy); g.scale(1, ry / rx)
      const gradient = g.createRadialGradient(0, 0, 0, 0, 0, rx)
      gradient.addColorStop(0, 'rgba(92,104,128,0.14)'); gradient.addColorStop(0.6, 'rgba(92,104,128,0.06)'); gradient.addColorStop(1, 'rgba(92,104,128,0)')
      g.fillStyle = gradient; g.fillRect(-rx, -rx, rx * 2, rx * 2); g.restore()
    }
    return node
  }

  // ------------------------------------------------------------ effects sprites

  function splat(seed, size = 128, color = INK) {
    const random = rng(seed)
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const c = size / 2
    g.fillStyle = rgba(color, 1)
    g.beginPath()
    const lobes = 18
    for (let i = 0; i <= lobes; i++) {
      const a = i / lobes * Math.PI * 2
      const r = size * (0.12 + random() * 0.1)
      const x = c + Math.cos(a) * r, y = c + Math.sin(a) * r
      if (i === 0) g.moveTo(x, y); else g.quadraticCurveTo(c + Math.cos(a - 0.17) * r * 1.3, c + Math.sin(a - 0.17) * r * 1.3, x, y)
    }
    g.fill()
    for (let i = 0; i < 9; i++) {
      const a = random() * Math.PI * 2, len = size * (0.2 + random() * 0.26)
      g.fillStyle = rgba(color, 0.9)
      ribbon(g, [[c, c], [c + Math.cos(a) * len * 0.5, c + Math.sin(a) * len * 0.5], [c + Math.cos(a) * len, c + Math.sin(a) * len]], (t) => size * 0.07 * (1 - t))
    }
    for (let i = 0; i < 26; i++) {
      const a = random() * Math.PI * 2, r = size * (0.18 + Math.pow(random(), 0.7) * 0.3)
      g.beginPath(); g.arc(c + Math.cos(a) * r, c + Math.sin(a) * r, size * (0.006 + random() * 0.026), 0, Math.PI * 2); g.fill()
    }
    return node
  }

  function softDisc(size, color, alphaCore, alphaEdge = 0) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const gradient = g.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2)
    gradient.addColorStop(0, rgba(color, alphaCore)); gradient.addColorStop(0.6, rgba(color, alphaCore * 0.55)); gradient.addColorStop(1, rgba(color, alphaEdge))
    g.fillStyle = gradient; g.fillRect(0, 0, size, size)
    return node
  }

  function sun(size, color, seed) {
    const random = rng(seed)
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const c = size / 2
    const halo = g.createRadialGradient(c, c, size * 0.3, c, c, size * 0.5)
    halo.addColorStop(0, rgba(color, 0.3)); halo.addColorStop(1, rgba(color, 0))
    g.fillStyle = halo; g.fillRect(0, 0, size, size)
    for (let i = 0; i < 7; i++) {
      g.fillStyle = rgba(color, 0.16 + random() * 0.1)
      g.beginPath(); g.arc(c + (random() - 0.5) * size * 0.03, c + (random() - 0.5) * size * 0.03, size * (0.27 + random() * 0.04), 0, Math.PI * 2); g.fill()
    }
    return node
  }

  function moon(size) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const c = size / 2
    const glow = g.createRadialGradient(c, c, size * 0.2, c, c, size * 0.5)
    glow.addColorStop(0, 'rgba(250,246,234,0.5)'); glow.addColorStop(1, 'rgba(250,246,234,0)')
    g.fillStyle = glow; g.fillRect(0, 0, size, size)
    g.fillStyle = 'rgba(252,249,238,0.97)'
    g.beginPath(); g.arc(c, c, size * 0.24, 0, Math.PI * 2); g.fill()
    const random = rng(4)
    for (let i = 0; i < 9; i++) {
      g.fillStyle = 'rgba(160,156,146,' + (0.06 + random() * 0.08) + ')'
      g.beginPath(); g.arc(c + (random() - 0.5) * size * 0.3, c + (random() - 0.5) * size * 0.3, size * (0.02 + random() * 0.05), 0, Math.PI * 2); g.fill()
    }
    return node
  }

  function mist(width, height) {
    const node = canvas(width, height)
    const g = node.getContext('2d')
    g.save(); g.scale(width / height, 1)
    const r = height / 2
    const gradient = g.createRadialGradient(r, r, 0, r, r, r)
    gradient.addColorStop(0, rgba(PAPER, 0.85)); gradient.addColorStop(0.55, rgba(PAPER, 0.42)); gradient.addColorStop(1, rgba(PAPER, 0))
    g.fillStyle = gradient; g.fillRect(0, 0, height, height)
    g.restore()
    return node
  }

  function leafSprite(color, alpha, size = 32) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    g.fillStyle = rgba(color, alpha)
    leaf(g, size * 0.1, size / 2, 0, size * 0.85, size * 0.13)
    return node
  }

  function petalSprite(size = 24) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    g.fillStyle = rgba(CINNABAR, 0.85)
    g.beginPath(); g.ellipse(size / 2, size / 2, size * 0.32, size * 0.2, 0.3, 0, Math.PI * 2); g.fill()
    g.fillStyle = 'rgba(230,120,100,0.6)'
    g.beginPath(); g.ellipse(size * 0.45, size * 0.48, size * 0.15, size * 0.08, 0.3, 0, Math.PI * 2); g.fill()
    return node
  }

  // Ground wave crest used by the boss's ink wave.
  function waveSprite(res) {
    return sprite(70, 50, res, (g) => {
      const random = rng(19)
      g.fillStyle = rgba(INK, 0.92)
      g.beginPath(); g.moveTo(-30, 0); g.quadraticCurveTo(-10, -4, 4, -30); g.quadraticCurveTo(14, -44, 26, -34); g.quadraticCurveTo(14, -36, 12, -22); g.quadraticCurveTo(10, -8, 30, 0); g.closePath(); g.fill()
      for (let i = 0; i < 14; i++) {
        g.beginPath(); g.arc(-10 + random() * 40, -random() * 40, 0.6 + random() * 1.8, 0, Math.PI * 2); g.fill()
      }
      dryBrush(g, (t) => [lerp(-34, 30, t), -2 + Math.sin(t * Math.PI) * -4, 0, 1], 4, 6, INK, 0.7, 4, 20)
    }, 34, 46)
  }

  function iceSpikeSprite(res) {
    return sprite(80, 54, res, (g) => {
      g.fillStyle = 'rgba(225,238,252,0.92)'
      g.beginPath()
      g.moveTo(-36, 0); g.lineTo(-24, -36); g.lineTo(-12, -8)
      g.lineTo(0, -48); g.lineTo(12, -18); g.lineTo(26, -38); g.lineTo(36, 0)
      g.closePath(); g.fill()
      g.fillStyle = rgba(INK, 0.75)
      g.beginPath()
      g.moveTo(-24, -36); g.lineTo(-12, -8); g.lineTo(0, -48); g.lineTo(4, 0); g.lineTo(-16, 0)
      g.closePath(); g.fill()
      g.strokeStyle = 'rgba(255,255,255,0.85)'; g.lineWidth = 1.2
      g.beginPath(); g.moveTo(-24, -36); g.lineTo(-24, 0); g.moveTo(0, -48); g.lineTo(0, 0); g.moveTo(26, -38); g.lineTo(26, 0); g.stroke()
    }, 40, 50)
  }

  function snowflakeSprite(size = 24) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const c = size / 2
    const gradient = g.createRadialGradient(c, c, 0, c, c, c)
    gradient.addColorStop(0, 'rgba(255,255,255,1)'); gradient.addColorStop(0.42, 'rgba(255,255,255,0.96)')
    gradient.addColorStop(0.6, 'rgba(120,138,170,0.34)'); gradient.addColorStop(1, 'rgba(120,138,170,0)')
    g.fillStyle = gradient; g.fillRect(0, 0, size, size)
    return node
  }

  // ============================================================ 烽火卷 · 浅绛 (volume 3)
  const OCHRE = [168, 116, 64], OCHRE_L = [196, 150, 88], GOLD = [200, 152, 60], INDIGO = [70, 96, 120]
  const KAITI = '"Kaiti SC","STKaiti","KaiTi","Songti SC",serif'

  // Packed earth and gravel with a thin ochre wash that thins downward.
  function desertGround(o) {
    const n = noise(o.seed, 64)
    const random = rng(o.seed + 5)
    const lineY = (x) => o.line + (n((x / o.period) * 64) - 0.5) * 3.5
    const pebbles = [], tufts = [], ruts = [], patches = []
    for (let i = 0; i < o.period / 14; i++) pebbles.push({ x: random() * o.period, y: o.line + 3 + Math.pow(random(), 1.3) * (o.depth - 6), r: 0.6 + random() * 2.6, a: 0.12 + random() * 0.32, ochre: random() < 0.4 })
    for (let i = 0; i < o.period / 75; i++) tufts.push({ x: random() * o.period, h: 5 + random() * 9, k: 3 + Math.floor(random() * 3), a: 0.3 + random() * 0.35 })
    for (let i = 0; i < 3; i++) ruts.push({ x: random() * o.period, len: 120 + random() * 220, y: o.line + 8 + random() * (o.depth * 0.5) })
    for (let i = 0; i < o.period / 90; i++) patches.push({ x: random() * o.period, y: o.line + 6 + random() * (o.depth - 14), rx: 30 + random() * 70, ry: 3 + random() * 5, c: random() < 0.5 ? OCHRE_L : INK, a: 0.05 + random() * 0.07 })
    return strip(o.period, o.line - 20, o.depth + 20, o.res, (g, u0, u1) => {
      const wash = g.createLinearGradient(0, o.line, 0, o.line + o.depth)
      wash.addColorStop(0, rgba(OCHRE, 0.42)); wash.addColorStop(0.35, rgba(OCHRE, 0.2)); wash.addColorStop(1, rgba(OCHRE, 0.03))
      g.fillStyle = wash
      g.beginPath(); g.moveTo(u0 - 2, o.line + o.depth)
      for (let x = u0 - 2; x <= u1 + 2; x += 2) g.lineTo(x, lineY(x))
      g.lineTo(u1 + 2, o.line + o.depth); g.closePath(); g.fill()
      const ink = g.createLinearGradient(0, o.line, 0, o.line + o.depth * 0.6)
      ink.addColorStop(0, rgba(INK, 0.14)); ink.addColorStop(1, rgba(INK, 0))
      g.fillStyle = ink; g.fill()
      for (const shift of [-o.period, 0, o.period]) {
        for (const p of patches) {
          const x = p.x + shift
          if (x + p.rx < u0 || x - p.rx > u1) continue
          g.fillStyle = rgba(p.c, p.a); g.beginPath(); g.ellipse(x, p.y, p.rx, p.ry, 0, 0, Math.PI * 2); g.fill()
        }
        for (const r of ruts) {
          g.strokeStyle = rgba(INK, 0.1); g.lineWidth = 0.8; g.setLineDash([14, 5, 6, 4])
          g.beginPath(); g.moveTo(r.x + shift, r.y); g.quadraticCurveTo(r.x + shift + r.len / 2, r.y + 1.6, r.x + shift + r.len, r.y - 0.5); g.stroke()
          g.beginPath(); g.moveTo(r.x + shift, r.y + 4); g.quadraticCurveTo(r.x + shift + r.len / 2, r.y + 5.4, r.x + shift + r.len, r.y + 3.4); g.stroke()
          g.setLineDash([])
        }
        dryBrush(g, (t) => { const x = shift + t * o.period; return [x, lineY(x), 0, 1] }, 5, 9, INK, 0.82, o.seed + 3, 260)
        for (const p of pebbles) {
          const x = p.x + shift
          if (x < u0 - 6 || x > u1 + 6) continue
          g.fillStyle = p.ochre ? rgba(OCHRE, p.a * 1.2) : rgba(INK, p.a)
          g.beginPath(); g.ellipse(x, p.y, p.r * 1.5, p.r * 0.7, 0, 0, Math.PI * 2); g.fill()
          if (p.r > 2) { g.fillStyle = rgba(INK, p.a * 0.8); g.fillRect(x - p.r, p.y + p.r * 0.5, p.r * 2.2, 0.7) }
        }
        for (const t of tufts) {
          const x = t.x + shift
          if (x < u0 - 20 || x > u1 + 20) continue
          const y = lineY(x)
          for (let k = 0; k < t.k; k++) {
            g.fillStyle = k % 2 ? rgba(OCHRE, t.a) : rgba(INK, t.a * 0.8)
            leaf(g, x + k * 1.4, y + 1, -Math.PI / 2 + (k - t.k / 2) * 0.4 + 0.1, t.h * (0.6 + (k % 2) * 0.5), 0.9)
          }
        }
      }
    })
  }

  // Autumn grass (枯草): thin curved blades, or the charred stubble left after the fire.
  function dryGrass(o) {
    const random = rng(o.seed)
    const count = Math.round(o.period * (o.burnt ? 0.5 : 1.05))
    const blades = []
    for (let i = 0; i < count; i++) {
      const burnt = o.burnt
      blades.push({
        x: random() * o.period, h: burnt ? 2 + random() * o.height * 0.16 : o.height * (0.6 + random() * 0.4),
        lean: (random() - 0.5) * (burnt ? 0.5 : 1.1), bend: (random() - 0.5) * 0.9, w: burnt ? 1.1 + random() * 0.8 : 0.7 + random() * 0.9,
        pick: random(), a: burnt ? 0.5 + random() * 0.4 : 0.28 + random() * 0.42, seed: random(),
      })
    }
    const stalks = []
    if (o.burnt) for (let i = 0; i < o.period / 70; i++) stalks.push({ x: random() * o.period, h: o.height * (0.3 + random() * 0.4), bend: (random() > 0.5 ? 1 : -1) * (0.3 + random() * 0.6) })
    const ash = []
    if (o.burnt) for (let i = 0; i < o.period / 40; i++) ash.push({ x: random() * o.period, rx: 14 + random() * 40, ry: 1.5 + random() * 3, a: 0.1 + random() * 0.18 })
    return strip(o.period, o.line - o.height - 12, o.height + 18, o.res, (g, u0, u1) => {
      for (const shift of [-o.period, 0, o.period]) {
        for (const p of ash) {
          const x = p.x + shift
          if (x + p.rx < u0 || x - p.rx > u1) continue
          g.fillStyle = 'rgba(120,114,106,' + p.a + ')'; g.beginPath(); g.ellipse(x, o.line + 3, p.rx, p.ry, 0, 0, Math.PI * 2); g.fill()
        }
        for (const b of blades) {
          const x = b.x + shift
          if (x < u0 - o.height || x > u1 + o.height) continue
          const tx = x + b.lean * b.h * 0.7, ty = o.line + 1 - b.h
          if (o.burnt) {
            g.strokeStyle = rgba(INK, b.a); g.lineWidth = b.w
            g.beginPath(); g.moveTo(x, o.line + 2); g.lineTo(x + b.lean * b.h, ty + b.h * 0.2); g.stroke()
          } else {
            const col = b.pick < 0.45 ? OCHRE : b.pick < 0.75 ? OCHRE_L : INK
            g.strokeStyle = rgba(col, col === INK ? b.a * 0.7 : b.a); g.lineWidth = b.w
            g.beginPath(); g.moveTo(x, o.line + 2)
            g.quadraticCurveTo(x + b.lean * b.h * 0.15 + b.bend * 4, o.line - b.h * 0.55, tx + b.bend * b.h * 0.3, ty); g.stroke()
          }
        }
        for (const s of stalks) {
          const x = s.x + shift
          if (x < u0 - 30 || x > u1 + 30) continue
          g.strokeStyle = rgba(INK, 0.85); g.lineWidth = 1.1
          g.beginPath(); g.moveTo(x, o.line + 2); g.quadraticCurveTo(x + s.bend * 4, o.line - s.h * 0.6, x + s.bend * s.h * 0.55, o.line - s.h); g.stroke()
        }
      }
    })
  }

  // Clustered leaf dabs (点叶): golden ochre blots with ink dots and a few ink edge touches.
  function leafDabs(g, cx, cy, rx, ry, count, random, alpha, inkDots = 0.35) {
    for (let i = 0; i < count; i++) {
      const a = random() * Math.PI * 2, d = Math.sqrt(random())
      const x = cx + Math.cos(a) * rx * d, y = cy + Math.sin(a) * ry * d - (1 - d) * 0
      const r = 2.2 + random() * 3.6
      g.fillStyle = rgba(random() < 0.3 ? OCHRE_L : GOLD, alpha * (0.55 + random() * 0.45))
      g.beginPath(); g.ellipse(x, y, r * 1.25, r * 0.8, (random() - 0.5) * 1.2, 0, Math.PI * 2); g.fill()
      if (random() < inkDots) {
        g.fillStyle = rgba(INK, 0.35 + random() * 0.4)
        g.beginPath(); g.ellipse(x + (random() - 0.5) * 5, y + 1.5 + random() * 2.5, 0.8 + random() * 1.2, 0.6 + random() * 0.8, 0, 0, Math.PI * 2); g.fill()
      }
    }
  }

  // 胡杨: gnarled twisted split trunk under a cloud of golden leaf dabs. Origin at the trunk base.
  function poplar(res, seed, scale = 1) {
    const random = rng(seed)
    return sprite(320 * scale, 270 * scale, res, (g) => {
      g.scale(scale, scale)
      g.fillStyle = rgba(INK, 0.12); g.beginPath(); g.ellipse(0, 1, 52, 3.5, 0, 0, Math.PI * 2); g.fill()
      const tips = []
      const limb = (x, y, angle, len, width, depth) => {
        const pts = [[x, y]]
        let cx = x, cy = y, a = angle
        const segs = 6
        for (let i = 1; i <= segs; i++) {
          a += (random() - 0.5) * 0.7
          cx += Math.cos(a) * len / segs; cy += Math.sin(a) * len / segs
          pts.push([cx, cy])
        }
        g.fillStyle = rgba(INK, 0.84)
        ribbon(g, pts, (t) => width * (1 - t * 0.5) * (1 + 0.18 * Math.sin(t * 17 + seed)))
        g.strokeStyle = rgba(PAPER, 0.28); g.lineWidth = 0.7
        for (let i = 1; i < segs; i++) { g.beginPath(); g.moveTo(pts[i][0] - width * 0.1, pts[i][1] + 2); g.lineTo(pts[i + 1][0] - width * 0.1, pts[i + 1][1] + 3); g.stroke() }
        if (depth <= 0 || width < 2.2) { tips.push([cx, cy]); return }
        const kids = depth > 1 ? 2 : 1 + (random() > 0.5 ? 1 : 0)
        for (let k = 0; k < kids; k++) limb(cx, cy, a + (k === 0 ? -1 : 1) * (0.35 + random() * 0.55) * (kids === 1 ? 0.5 : 1), len * (0.62 + random() * 0.2), width * 0.62, depth - 1)
      }
      // Trunk: a wide twisting base that splits in two near the middle.
      const trunk = []
      for (let i = 0; i <= 10; i++) { const t = i / 10; trunk.push([Math.sin(t * 4.2 + seed) * 9 * t + t * 6, -t * 78]) }
      g.fillStyle = rgba(INK, 0.9)
      ribbon(g, trunk, (t) => 30 * (1 - t * 0.55) + (t < 0.15 ? 12 * (0.15 - t) / 0.15 : 0))
      // Cracked bark: split gap and pale fissures.
      g.fillStyle = rgba(PAPER, 0.5)
      g.beginPath(); g.moveTo(-1, -22); g.quadraticCurveTo(3, -42, 0, -62); g.quadraticCurveTo(-3, -42, -1, -22); g.fill()
      g.strokeStyle = rgba(PAPER, 0.3); g.lineWidth = 0.8
      for (let i = 0; i < 12; i++) {
        const t = 0.05 + random() * 0.8, p = trunk[Math.floor(t * 10)]
        const x = p[0] + (random() - 0.5) * 14, y = p[1]
        g.beginPath(); g.moveTo(x, y); g.quadraticCurveTo(x + (random() - 0.5) * 3, y - 6, x + (random() - 0.5) * 4, y - 10 - random() * 8); g.stroke()
      }
      dryBrush(g, (t) => [trunk[Math.round(t * 10)][0] - 11 * (1 - t * 0.5), trunk[Math.round(t * 10)][1], 0, 1], 5, 5, INK, 0.7, seed + 8, 14)
      // Washed ochre on the sunward side of the trunk.
      g.fillStyle = rgba(OCHRE, 0.28)
      ribbon(g, trunk.map((p, i) => [p[0] + 7 * (1 - i / 14), p[1]]), (t) => 7 * (1 - t * 0.6))
      const top = trunk[10]
      limb(top[0], top[1], -Math.PI / 2 - 0.8, 50, 15, 3)
      limb(top[0], top[1], -Math.PI / 2 + 0.7, 52, 15, 3)
      limb(top[0], top[1] + 20, -Math.PI / 2 - 1.15, 40, 9, 2)
      limb(trunk[6][0], trunk[6][1], -Math.PI / 2 + 1.15, 38, 8, 2)
      // Crown.
      for (const [x, y] of tips) {
        leafDabs(g, x, y - 4, 24, 15, 26, random, 0.42)
      }
      for (const [x, y] of tips) leafDabs(g, x + (random() - 0.5) * 10, y - 8, 20, 12, 10, random, 0.55, 0.5)
      g.globalAlpha = 1
    }, 160 * scale, 267 * scale)
  }

  // Distant poplar row for parallax.
  function poplarGrove(o) {
    const random = rng(o.seed)
    const trees = []
    for (let i = 0; i < o.count; i++) trees.push({ x: random() * o.period, h: lerp(o.height[0], o.height[1], random()), lean: (random() - 0.5) * 0.3, s: random() })
    const a = o.alpha ?? 0.3
    return strip(o.period, o.top, o.base - o.top + 4, o.res, (g, u0, u1) => {
      for (const shift of [-o.period, 0, o.period]) for (const t of trees) {
        const x = t.x + shift, h = t.h
        if (x < u0 - h || x > u1 + h) continue
        const rnd = rng(Math.floor(t.s * 1e6))
        const bend = (rnd() - 0.5) * 8
        g.fillStyle = rgba(o.color, Math.min(1, a * 1.9))
        ribbon(g, [[x, o.base], [x + bend * 0.5, o.base - h * 0.25], [x + bend, o.base - h * 0.5]], (u) => h * 0.07 * (1 - u * 0.6) + 0.8)
        g.strokeStyle = rgba(o.color, Math.min(1, a * 1.5)); g.lineWidth = Math.max(0.6, h * 0.025)
        for (const k of [-1, 1]) { g.beginPath(); g.moveTo(x + bend * 0.6, o.base - h * 0.35); g.quadraticCurveTo(x + bend + k * h * 0.12, o.base - h * 0.5, x + bend + k * h * 0.25, o.base - h * 0.66); g.stroke() }
        for (const [dx, dy, rr] of [[0, 0.62, 0.3], [-0.2, 0.52, 0.2], [0.22, 0.54, 0.22], [0.05, 0.78, 0.2]]) {
          g.fillStyle = rgba(random() < 0.3 ? OCHRE_L : GOLD, a * (0.7 + random() * 0.4))
          g.beginPath(); g.ellipse(x + bend + dx * h, o.base - dy * h, rr * h * 0.9, rr * h * 0.6, 0, 0, Math.PI * 2); g.fill()
        }
        leafDabs(g, x + bend, o.base - h * 0.62, h * 0.32, h * 0.2, Math.round(h / 6), random, a * 0.9, 0.25)
        g.fillStyle = rgba(o.color, a * 1.3)
        for (let i = 0; i < 5; i++) { g.beginPath(); g.arc(x + bend + (random() - 0.5) * h * 0.5, o.base - h * (0.5 + random() * 0.3), 0.7 + random() * 0.6, 0, Math.PI * 2); g.fill() }
      }
    })
  }

  function ochreLeafSprite(size = 32) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    g.translate(size / 2, size / 2)
    g.fillStyle = rgba(GOLD, 0.9)
    g.beginPath()
    const r = size * 0.36
    for (let i = 0; i <= 24; i++) {
      const a = (i / 24) * Math.PI * 2
      const rr = r * (0.92 + (i % 2 ? 0.1 : -0.04)) * (a > 3.4 ? 0.9 : 1)
      const x = Math.cos(a) * rr * 1.05, y = Math.sin(a) * rr * 0.88
      if (i === 0) g.moveTo(x, y); else g.lineTo(x, y)
    }
    g.closePath(); g.fill()
    g.fillStyle = rgba(OCHRE, 0.45); g.beginPath(); g.ellipse(r * 0.15, r * 0.2, r * 0.55, r * 0.35, 0.3, 0, Math.PI * 2); g.fill()
    g.strokeStyle = rgba(INK, 0.5); g.lineWidth = Math.max(0.5, size / 40)
    g.beginPath(); g.moveTo(-r * 1.15, r * 0.1); g.lineTo(r * 0.8, -r * 0.05); g.stroke()
    return node
  }

  function dustSprite(size = 64) {
    const node = canvas(size, size)
    const g = node.getContext('2d')
    const c = size / 2
    const random = rng(41)
    for (let i = 0; i < 9; i++) {
      const x = c + (random() - 0.5) * size * 0.28, y = c + (random() - 0.5) * size * 0.28, r = size * (0.2 + random() * 0.16)
      const grad = g.createRadialGradient(x, y, 0, x, y, r)
      grad.addColorStop(0, 'rgba(176,152,120,0.22)'); grad.addColorStop(0.6, 'rgba(176,152,120,0.1)'); grad.addColorStop(1, 'rgba(176,152,120,0)')
      g.fillStyle = grad; g.fillRect(0, 0, size, size)
    }
    return node
  }

  // Clay fire pot; the game draws the fuse ember itself.
  function potSprite(res) {
    return sprite(18, 22, res, (g) => {
      g.fillStyle = rgba(INK, 0.9)
      g.beginPath(); g.moveTo(-3.5, -5.6); g.bezierCurveTo(-9, -4, -9, 6, -3.5, 7); g.lineTo(3.5, 7); g.bezierCurveTo(9, 6, 9, -4, 3.5, -5.6); g.closePath(); g.fill()
      g.fillStyle = rgba(OCHRE, 0.55); g.beginPath(); g.ellipse(-2.2, 0.8, 3.4, 5, 0.2, 0, Math.PI * 2); g.fill()
      g.fillStyle = rgba(OCHRE_L, 0.4); g.beginPath(); g.ellipse(-3.4, -1.6, 1.4, 2.4, 0.3, 0, Math.PI * 2); g.fill()
      g.fillStyle = rgba(INK, 0.9); g.fillRect(-4.6, -7, 9.2, 2)
      g.strokeStyle = rgba(OCHRE_L, 0.9); g.lineWidth = 0.9
      for (const y of [-4.6, -3.4]) { g.beginPath(); g.moveTo(-3.6, y); g.lineTo(3.6, y + 0.4); g.stroke() }
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 0.9; g.beginPath(); g.moveTo(0, -7); g.quadraticCurveTo(1.6, -9, 1, -11); g.stroke()
    }, 9, 13)
  }

  // Rammed-earth (夯土) fill: ochre wash over pale ink, tamped layer lines, indigo-grey shade on one side, ink outline.
  // `trace()` must build the closed path; box = [x0, y0, x1, y1].
  function earthFill(g, trace, box, random, o = {}) {
    const [x0, y0, x1, y1] = box
    const w = x1 - x0, h = y1 - y0
    g.save()
    trace(); g.clip()
    const body = g.createLinearGradient(0, y0, 0, y1)
    body.addColorStop(0, rgba(OCHRE_L, o.top ?? 0.5)); body.addColorStop(1, rgba(OCHRE, o.bottom ?? 0.36))
    g.fillStyle = body; g.fillRect(x0 - 2, y0 - 2, w + 4, h + 4)
    g.fillStyle = rgba(INK, o.ink ?? 0.1); g.fillRect(x0 - 2, y0 - 2, w + 4, h + 4)
    // Shaded side.
    const shade = g.createLinearGradient(x0 + w * 0.5, 0, x1, 0)
    shade.addColorStop(0, rgba(INDIGO, 0)); shade.addColorStop(1, rgba(INDIGO, o.shade ?? 0.3))
    g.fillStyle = shade; g.fillRect(x0, y0, w, h)
    // Tamped layers.
    const step = o.layer ?? 9
    for (let y = y0 + step * 0.6; y < y1; y += step * (0.8 + random() * 0.5)) {
      g.strokeStyle = rgba(INK, 0.18 + random() * 0.2); g.lineWidth = 0.5 + random() * 0.7
      g.setLineDash([6 + random() * 24, 1 + random() * 5])
      g.beginPath(); g.moveTo(x0 - 2, y); g.lineTo(x0 + w * 0.4, y + (random() - 0.5) * 1.6); g.lineTo(x1 + 2, y + (random() - 0.5) * 2); g.stroke()
    }
    g.setLineDash([])
    // Weathering streaks and pits.
    for (let i = 0; i < w * h / 260; i++) {
      const x = x0 + random() * w, y = y0 + random() * h
      g.strokeStyle = rgba(INK, 0.1 + random() * 0.2); g.lineWidth = 0.5 + random() * 0.6
      g.beginPath(); g.moveTo(x, y); g.lineTo(x + (random() - 0.5) * 3, y + 3 + random() * 9); g.stroke()
    }
    g.restore()
  }

  function inkOutline(g, pts, width = 1.6, alpha = 0.85, closed = false) {
    g.strokeStyle = rgba(INK, alpha); g.lineWidth = width
    g.beginPath(); g.moveTo(pts[0][0], pts[0][1])
    for (let i = 1; i < pts.length; i++) g.lineTo(pts[i][0], pts[i][1])
    if (closed) g.closePath()
    g.stroke()
  }

  // 烽燧: tapered rammed-earth beacon tower. sprite.fireAt is the brazier position relative to the origin.
  function beaconTower(res) {
    const random = rng(311)
    const s = sprite(90, 175, res, (g) => {
      const bw = 37, tw = 24, topY = -126
      g.fillStyle = rgba(INK, 0.14); g.beginPath(); g.ellipse(0, 1, 52, 3.5, 0, 0, Math.PI * 2); g.fill()
      const trace = () => {
        g.beginPath(); g.moveTo(-bw - 3, 0); g.lineTo(-bw, -6); g.lineTo(-tw - 1, topY + 2); g.lineTo(-tw, topY)
        g.lineTo(-tw + 5, topY - 1); g.lineTo(tw - 2, topY - 1); g.lineTo(tw, topY); g.lineTo(tw + 1, topY + 2); g.lineTo(bw, -6); g.lineTo(bw + 3, 0); g.closePath()
      }
      earthFill(g, trace, [-bw - 3, topY - 12, bw + 3, 0], random, { layer: 7, shade: 0.34 })
      // Eroded base and rough edges.
      inkOutline(g, [[-bw - 3, 0], [-bw, -6], [-tw - 1, topY + 2], [-tw, topY]], 1.8, 0.9)
      inkOutline(g, [[bw + 3, 0], [bw, -6], [tw + 1, topY + 2], [tw, topY]], 1.8, 0.9)
      // Crenellated parapet.
      for (let i = 0; i < 5; i++) {
        const x = -tw - 1 + i * (2 * tw + 2) / 5 + 1
        g.beginPath(); g.rect(x, topY - 12, 7, 12); g.closePath()
        earthFill(g, () => { g.beginPath(); g.rect(x, topY - 12, 7, 12) }, [x, topY - 12, x + 7, topY], random, { layer: 4, shade: 0.2 })
        inkOutline(g, [[x, topY], [x, topY - 12], [x + 7, topY - 12], [x + 7, topY]], 1.2, 0.85)
      }
      dryBrush(g, (t) => [lerp(-tw - 2, tw + 2, t), topY - 0.5, 0, 1], 2.4, 5, INK, 0.9, 5, 24)
      // Opening and ladder holes.
      g.fillStyle = rgba(INK, 0.7)
      g.beginPath(); g.moveTo(-6, 0); g.lineTo(-6, -20); g.quadraticCurveTo(0, -27, 6, -20); g.lineTo(6, 0); g.closePath(); g.fill()
      g.fillStyle = rgba(INK, 0.55)
      for (const [x, y] of [[-17, -50], [14, -74], [-12, -98], [17, -112]]) g.fillRect(x, y, 4, 3)
      // Wooden frame and brazier (积薪).
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2.4
      g.beginPath(); g.moveTo(-14, topY - 1); g.lineTo(-10, topY - 30); g.moveTo(14, topY - 1); g.lineTo(10, topY - 30); g.stroke()
      g.lineWidth = 1.6; g.beginPath(); g.moveTo(-13, topY - 24); g.lineTo(13, topY - 24); g.moveTo(-12, topY - 9); g.lineTo(12, topY - 15); g.moveTo(12, topY - 9); g.lineTo(-12, topY - 15); g.stroke()
      g.fillStyle = rgba(INK, 0.9)
      g.beginPath(); g.moveTo(-12, topY - 24); g.lineTo(12, topY - 24); g.lineTo(8, topY - 31); g.lineTo(-8, topY - 31); g.closePath(); g.fill()
      g.strokeStyle = rgba(OCHRE, 0.9); g.lineWidth = 1.2
      for (let i = -3; i <= 3; i++) { g.beginPath(); g.moveTo(i * 3, topY - 25); g.lineTo(i * 3.4, topY - 31); g.stroke() }
      // Ground rubble and a drift of tufts.
      for (let i = 0; i < 12; i++) {
        const x = (random() - 0.5) * 100, r = 1 + random() * 2.6
        g.fillStyle = rgba(random() < 0.5 ? OCHRE : INK, 0.3 + random() * 0.3)
        g.beginPath(); g.ellipse(x, -r * 0.4, r * 1.4, r * 0.8, 0, 0, Math.PI * 2); g.fill()
      }
    }, 45, 172)
    s.fireAt = [0, -126 - 31]
    return s
  }

  // Broken low earthen wall used as cover; origin at base centre.
  function rampart(res, seed, h = 52) {
    const random = rng(seed)
    const hw = 17
    return sprite(52, h + 14, res, (g) => {
      g.fillStyle = rgba(INK, 0.14); g.beginPath(); g.ellipse(0, 1, 30, 3, 0, 0, Math.PI * 2); g.fill()
      const top = [[-hw - 1, -h * (0.55 + random() * 0.15)], [-hw * 0.6, -h * 0.82], [-hw * 0.35, -h * 0.7], [-hw * 0.1, -h], [hw * 0.2, -h * 0.86], [hw * 0.4, -h * 0.94], [hw * 0.62, -h * 0.66], [hw + 1, -h * 0.72]]
      const trace = () => {
        g.beginPath(); g.moveTo(-hw - 5, 0); g.lineTo(-hw - 1, -4)
        for (const p of top) g.lineTo(p[0], p[1])
        g.lineTo(hw + 2, -5); g.lineTo(hw + 6, 0); g.closePath()
      }
      earthFill(g, trace, [-hw - 5, -h, hw + 6, 0], random, { layer: 6, shade: 0.32 })
      inkOutline(g, [[-hw - 5, 0], [-hw - 1, -4], ...top, [hw + 2, -5], [hw + 6, 0]], 1.7, 0.88)
      dryBrush(g, (t) => { const i = Math.min(top.length - 2, Math.floor(t * (top.length - 1))), f = t * (top.length - 1) - i; return [lerp(top[i][0], top[i + 1][0], f), lerp(top[i][1], top[i + 1][1], f) - 0.5, 0, 1] }, 2.6, 5, INK, 0.9, seed + 2, 30)
      for (let i = 0; i < 9; i++) {
        const x = (random() - 0.5) * 46, r = 1 + random() * 2.8
        g.fillStyle = rgba(random() < 0.5 ? OCHRE : INK, 0.3 + random() * 0.4)
        g.beginPath(); g.ellipse(x, -r * 0.4, r * 1.4, r * 0.8, random(), 0, Math.PI * 2); g.fill()
      }
    }, 26, h + 12)
  }

  // Ruined rammed-earth block / broken wall-top: same contract as `rock`.
  function earthBlock(w, topY, groundY, seed, res) {
    const random = rng(seed)
    const h = groundY - topY
    const pad = 16
    return sprite(w + pad * 2, h + 26, res, (g) => {
      const left = [[-8 - random() * 3, h + 4], [-5 - random() * 3, h * 0.6], [-2, h * 0.25]]
      const right = [[w + 2, h * 0.25], [w + 6 + random() * 3, h * 0.65], [w + 9, h + 4]]
      const trace = () => {
        g.beginPath(); g.moveTo(left[0][0], left[0][1]); g.lineTo(left[1][0], left[1][1]); g.lineTo(left[2][0], left[2][1]); g.lineTo(0, 1)
        g.lineTo(w, 1); g.lineTo(right[0][0], right[0][1]); g.lineTo(right[1][0], right[1][1]); g.lineTo(right[2][0], right[2][1]); g.closePath()
      }
      earthFill(g, trace, [-10, 0, w + 10, h + 4], random, { layer: 8, top: 0.5, bottom: 0.2, ink: 0.14, shade: 0.22 })
      // Fade toward the ground like the stone platforms.
      g.save(); g.globalCompositeOperation = 'destination-out'
      const fade = g.createLinearGradient(0, h * 0.55, 0, h + 4)
      fade.addColorStop(0, 'rgba(0,0,0,0)'); fade.addColorStop(1, 'rgba(0,0,0,0.5)')
      g.fillStyle = fade; g.fillRect(-12, h * 0.55, w + 24, h * 0.5 + 6); g.restore()
      inkOutline(g, [left[0], left[1], left[2], [0, 1]], 1.6, 0.7)
      inkOutline(g, [[w, 1], right[0], right[1], right[2]], 1.6, 0.7)
      // Brick courses near the top.
      g.strokeStyle = rgba(INK, 0.3); g.lineWidth = 0.7
      for (let row = 0; row < 3 && row * 8 + 9 < h; row++) {
        const y = 9 + row * 8
        for (let x = (row % 2) * 7; x < w; x += 14 + random() * 4) { g.beginPath(); g.moveTo(x, y - 8); g.lineTo(x, y); g.stroke() }
      }
      // Firm walkable top edge.
      dryBrush(g, (t) => [lerp(-2, w + 2, t), Math.sin(t * Math.PI * 3) * -0.4, 0, 1], 3.6, 7, INK, 0.96, seed + 7, 40)
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 1.5
      g.beginPath(); g.moveTo(-1, 0.4); g.lineTo(w + 1, 0.4); g.stroke()
      // Loose bricks and tufts on top.
      for (let i = 0; i < Math.max(2, w / 40); i++) {
        const x = 6 + random() * (w - 14), bw = 5 + random() * 4
        g.fillStyle = rgba(OCHRE, 0.55); g.fillRect(x, -3, bw, 3)
        g.strokeStyle = rgba(INK, 0.75); g.lineWidth = 0.9; g.strokeRect(x, -3, bw, 3)
      }
      for (let i = 0; i < w / 18; i++) {
        const x = random() * w
        g.fillStyle = rgba(INK, 0.5)
        leaf(g, x, 0, -Math.PI / 2 + (random() - 0.5) * 0.8, 3 + random() * 4, 0.9)
      }
      for (let i = 0; i < 5; i++) {
        const x = left[0][0] + random() * (w + 20), r = 1 + random() * 2
        g.fillStyle = rgba(INK, 0.35); g.beginPath(); g.ellipse(x, h + 3, r * 1.6, r * 0.7, 0, 0, Math.PI * 2); g.fill()
      }
    }, pad, 4)
  }

  // Hanging war-banner cloth attached to a pole at x; flutters to the right. `glyph` is painted vertically.
  function flagCloth(g, x, y, w, h, cloth, clothAlpha, glyph, glyphColor, glyphAlpha, random, fontPx) {
    const ph = random() * 6
    // Rectangular banner with a notched swallowtail bottom and gently waving free edge.
    g.beginPath(); g.moveTo(x, y)
    const pts = []
    for (let i = 0; i <= 10; i++) { const t = i / 10; pts.push([x + w * t, y + Math.sin(t * 4 + ph) * 1.6 * t]) }
    for (const p of pts) g.lineTo(p[0], p[1])
    const ex = x + w + Math.sin(ph) * 1.5
    g.lineTo(ex, y + h * 0.5); g.lineTo(ex + 1, y + h)
    g.lineTo(x + w * 0.62, y + h - h * 0.12); g.lineTo(x + w * 0.3, y + h)
    g.lineTo(x, y + h - 2); g.closePath()
    g.fillStyle = rgba(cloth, clothAlpha); g.fill()
    g.fillStyle = rgba(INK, 0.1); g.fill()
    g.strokeStyle = rgba(INK, 0.82); g.lineWidth = 1.2; g.stroke()
    // Cloth folds.
    g.strokeStyle = rgba(INK, 0.16); g.lineWidth = 0.7
    for (let i = 1; i < 4; i++) { g.beginPath(); g.moveTo(x + w * i / 4, y + 2); g.quadraticCurveTo(x + w * i / 4 + 2, y + h * 0.5, x + w * i / 4 - 1, y + h - 4); g.stroke() }
    if (glyph) {
      g.fillStyle = rgba(glyphColor, glyphAlpha)
      g.font = '700 ' + fontPx + 'px ' + KAITI; g.textAlign = 'center'; g.textBaseline = 'middle'
      g.fillText(glyph, x + w * 0.5, y + h * 0.45)
    }
  }

  function banner(res, seed) {
    const random = rng(seed)
    return sprite(40, 120, res, (g) => {
      const inkCloth = seed % 2 === 1
      g.fillStyle = rgba(INK, 0.12); g.beginPath(); g.ellipse(0, 1, 12, 2.2, 0, 0, Math.PI * 2); g.fill()
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2.2
      g.beginPath(); g.moveTo(0, 0); g.lineTo(0, -116); g.stroke()
      g.fillStyle = rgba(INK, 0.9); g.beginPath(); g.moveTo(0, -122); g.lineTo(-2, -115); g.lineTo(2, -115); g.closePath(); g.fill()
      g.strokeStyle = rgba(OCHRE, 0.7); g.lineWidth = 0.8; g.beginPath(); g.moveTo(-0.4, -4); g.lineTo(-0.4, -110); g.stroke()
      const glyph = ['令', '戍', '烽', '守'][seed % 4]
      flagCloth(g, 1, -112, 28, 62, inkCloth ? INK : OCHRE_L, inkCloth ? 0.78 : 0.7, glyph, inkCloth ? PAPER : INK, inkCloth ? 0.7 : 0.45, random, 19)
      g.fillStyle = rgba(INK, 0.5); g.beginPath(); g.arc(0, -112, 2, 0, Math.PI * 2); g.fill()
    }, 6, 118)
  }

  // Four flame tongues for animation. The only place cinnabar appears in volume three.
  function flameSprites(res) {
    const specs = [
      { sway: 0.0, tip: 0.0, w: 1.0, h: 1.0, side: 0.1 },
      { sway: 3.0, tip: 2.5, w: 0.92, h: 0.94, side: -0.12 },
      { sway: -3.2, tip: -2.8, w: 1.04, h: 0.98, side: 0.2 },
      { sway: 1.4, tip: -1.2, w: 0.86, h: 1.0, side: -0.2 },
    ]
    return specs.map((s, idx) => sprite(22, 38, res, (g) => {
      const tongue = (hh, ww, sway, tip) => {
        g.beginPath(); g.moveTo(-ww, 0)
        g.bezierCurveTo(-ww * 1.15, -hh * 0.35, -ww * 0.4 + sway * 0.5, -hh * 0.62, tip + sway, -hh)
        g.bezierCurveTo(ww * 0.25 + sway * 0.6, -hh * 0.6, ww * 1.2, -hh * 0.35, ww, 0)
        g.bezierCurveTo(ww * 0.5, 2, -ww * 0.5, 2, -ww, 0); g.closePath()
      }
      // Smoky ink wisp around the flame.
      g.fillStyle = rgba(INK, 0.3)
      tongue(35 * s.h, 10.2 * s.w, s.sway * 1.1, s.tip * 1.1); g.fill()
      g.strokeStyle = rgba(INK, 0.5); g.lineWidth = 0.9; g.stroke()
      g.strokeStyle = rgba(INK, 0.35); g.lineWidth = 0.8
      g.beginPath(); g.moveTo(s.side * 22, -6); g.quadraticCurveTo(s.side * 36, -18, s.side * 18 + s.sway, -30 * s.h); g.stroke()
      // Cinnabar to ember body.
      const body = g.createLinearGradient(0, 0, 0, -32 * s.h)
      body.addColorStop(0, 'rgba(226,65,44,0.96)'); body.addColorStop(0.55, 'rgba(210,56,38,0.88)'); body.addColorStop(1, 'rgba(178,46,34,0.35)')
      g.fillStyle = body
      tongue(31 * s.h, 8.4 * s.w, s.sway, s.tip); g.fill()
      // Warm core.
      const core = g.createLinearGradient(0, 0, 0, -22 * s.h)
      core.addColorStop(0, 'rgba(244,170,70,0.95)'); core.addColorStop(1, 'rgba(226,65,44,0.2)')
      g.fillStyle = core
      tongue(21 * s.h, 5.2 * s.w, s.sway * 0.6, s.tip * 0.5); g.fill()
      // Pale heart.
      g.fillStyle = 'rgba(250,226,156,0.95)'
      tongue(11 * s.h, 3 * s.w, s.sway * 0.2, 0); g.fill()
      // Soft ink flick at the tip.
      g.fillStyle = rgba(INK, 0.4 + idx * 0.04)
      g.beginPath(); g.ellipse(s.tip + s.sway * 1.1, -34 * s.h, 1.1, 2, s.sway * 0.1, 0, Math.PI * 2); g.fill()
    }, 11, 37))
  }

  // Felt tents (毡帐), weapon rack and banners; origin base centre.
  function tents(res, seed) {
    const random = rng(seed)
    return sprite(280, 150, res, (g) => {
      g.fillStyle = rgba(INK, 0.1); g.beginPath(); g.ellipse(0, 1, 135, 4, 0, 0, Math.PI * 2); g.fill()
      const tent = (cx, w, h, wallFrac) => {
        const wy = -h * wallFrac
        const trace = () => {
          g.beginPath(); g.moveTo(cx - w / 2, 0); g.lineTo(cx - w / 2, wy)
          g.quadraticCurveTo(cx - w * 0.3, wy - h * 0.12, cx - 2, -h); g.lineTo(cx + 2, -h)
          g.quadraticCurveTo(cx + w * 0.3, wy - h * 0.12, cx + w / 2, wy); g.lineTo(cx + w / 2, 0)
          g.quadraticCurveTo(cx, 3, cx - w / 2, 0); g.closePath()
        }
        trace()
        const felt = g.createLinearGradient(cx - w / 2, 0, cx + w / 2, 0)
        felt.addColorStop(0, rgba(OCHRE_L, 0.5)); felt.addColorStop(0.6, rgba(OCHRE, 0.42)); felt.addColorStop(1, rgba(INDIGO, 0.5))
        g.fillStyle = felt; g.fill()
        g.fillStyle = rgba(INK, 0.1); g.fill()
        g.save(); trace(); g.clip()
        g.strokeStyle = rgba(INK, 0.34); g.lineWidth = 0.9
        for (let i = 1; i < 6; i++) { const x = cx - w / 2 + w * i / 6; g.beginPath(); g.moveTo(cx + (x - cx) * 0.1, -h); g.quadraticCurveTo(x, wy * 1.1, x + (x - cx) * 0.02, 0); g.stroke() }
        // Felt bands (ropes) around the tent.
        g.strokeStyle = rgba(INK, 0.55); g.lineWidth = 1.3
        for (const y of [wy, wy - h * 0.2]) { g.beginPath(); g.moveTo(cx - w / 2 - 2, y); g.quadraticCurveTo(cx, y + 5, cx + w / 2 + 2, y); g.stroke() }
        g.fillStyle = rgba(INK, 0.82)
        g.beginPath(); g.moveTo(cx - w * 0.09, 0); g.lineTo(cx - w * 0.09, -h * 0.28); g.quadraticCurveTo(cx, -h * 0.36, cx + w * 0.09, -h * 0.28); g.lineTo(cx + w * 0.09, 0); g.closePath(); g.fill()
        for (let i = 0; i < w * 0.7; i++) { g.fillStyle = rgba(INK, 0.06 + random() * 0.12); g.fillRect(cx - w / 2 + random() * w, -random() * h, 1 + random() * 2, 0.8) }
        g.restore()
        g.strokeStyle = rgba(INK, 0.88); g.lineWidth = 1.6
        trace(); g.stroke()
        g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 1.4; g.beginPath(); g.moveTo(cx, -h); g.lineTo(cx, -h - 7); g.stroke()
        // Guy ropes with pegs.
        g.strokeStyle = rgba(INK, 0.5); g.lineWidth = 0.7
        for (const side of [-1, 1]) {
          const px = cx + side * (w / 2 + 16 + random() * 8)
          g.beginPath(); g.moveTo(cx + side * w * 0.24, wy - h * 0.14); g.quadraticCurveTo((cx + px) / 2, wy * 0.2, px, 1); g.stroke()
          g.strokeStyle = rgba(INK, 0.8); g.lineWidth = 1.4; g.beginPath(); g.moveTo(px, 1); g.lineTo(px + side * 1.6, -3); g.stroke()
          g.strokeStyle = rgba(INK, 0.5); g.lineWidth = 0.7
        }
      }
      tent(-86, 78, 56, 0.4)
      tent(-8, 104, 76, 0.38)
      tent(86, 64, 44, 0.42)
      // Weapon rack.
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2
      g.beginPath(); g.moveTo(-138, 0); g.lineTo(-138, -30); g.moveTo(-122, 0); g.lineTo(-122, -30); g.moveTo(-142, -22); g.lineTo(-118, -22); g.moveTo(-142, -12); g.lineTo(-118, -12); g.stroke()
      g.lineWidth = 1.2
      for (let i = 0; i < 4; i++) {
        const x = -136 + i * 4.5, top = -66 - random() * 14
        g.beginPath(); g.moveTo(x, -4); g.lineTo(x + (i - 1.5) * 1.2, top); g.stroke()
        g.fillStyle = rgba(INK, 0.9); g.beginPath(); g.moveTo(x + (i - 1.5) * 1.2, top - 7); g.lineTo(x + (i - 1.5) * 1.2 - 1.6, top); g.lineTo(x + (i - 1.5) * 1.2 + 1.6, top); g.closePath(); g.fill()
      }
      // Tall banners.
      for (const [x, h, cloth, a] of [[130, 130, INK, 0.72], [-58, 112, OCHRE_L, 0.7]]) {
        g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2; g.beginPath(); g.moveTo(x, 0); g.lineTo(x, -h); g.stroke()
        flagCloth(g, x + 1, -h + 3, 22, 44, cloth, a, null, INK, 0, random, 0)
      }
    }, 140, 148)
  }

  // Ruled-line (界画) tiled roof with upturned eaves.
  function tileRoof(g, cx, half, y, rise, alpha) {
    g.beginPath()
    g.moveTo(cx - half - 9, y - 10)
    g.quadraticCurveTo(cx - half + 4, y + 3, cx - half + 18, y + 3)
    g.lineTo(cx + half - 18, y + 3)
    g.quadraticCurveTo(cx + half - 4, y + 3, cx + half + 9, y - 10)
    g.quadraticCurveTo(cx + half - 6, y - rise * 0.35, cx + half * 0.3, y - rise)
    g.lineTo(cx - half * 0.3, y - rise)
    g.quadraticCurveTo(cx - half + 6, y - rise * 0.35, cx - half - 9, y - 10)
    g.closePath()
    g.fillStyle = rgba(INK, alpha); g.fill()
    g.fillStyle = rgba(INDIGO, 0.3); g.fill()
    g.save(); g.clip()
    g.strokeStyle = rgba(PAPER, 0.22); g.lineWidth = 0.7
    for (let x = cx - half; x <= cx + half; x += 5) { g.beginPath(); g.moveTo(cx + (x - cx) * 0.3, y - rise); g.lineTo(x + (x - cx) * 0.06, y + 3); g.stroke() }
    g.restore()
    g.strokeStyle = rgba(INK, 0.95); g.lineWidth = 2.6
    g.beginPath(); g.moveTo(cx - half * 0.32, y - rise); g.lineTo(cx + half * 0.32, y - rise); g.stroke()
    g.lineWidth = 1.3
    g.beginPath(); g.moveTo(cx - half - 9, y - 10); g.quadraticCurveTo(cx - half + 4, y + 3, cx - half + 18, y + 3); g.lineTo(cx + half - 18, y + 3); g.quadraticCurveTo(cx + half - 4, y + 3, cx + half + 9, y - 10); g.stroke()
    // Ridge finials.
    g.fillStyle = rgba(INK, 0.95)
    for (const s of [-1, 1]) { g.beginPath(); g.moveTo(cx + s * half * 0.3, y - rise); g.lineTo(cx + s * half * 0.3 + s * 2, y - rise - 6); g.lineTo(cx + s * half * 0.3 - s * 3, y - rise); g.closePath(); g.fill() }
    // Bracket sets (斗拱) under the eaves.
    g.fillStyle = rgba(INK, alpha * 0.8)
    for (let x = cx - half + 14; x <= cx + half - 12; x += 9) g.fillRect(x - 2.2, y + 3, 4.4, 4.2)
  }

  // 关城: long city wall with a two-storey gate tower and an arched opening. Origin at base centre.
  function cityWall(res) {
    const random = rng(8801)
    return sprite(460, 270, res, (g) => {
      const wallTop = -92, wallHalf = 230
      // Wall body with indigo-grey shade toward the bottom.
      const wallTrace = () => { g.beginPath(); g.rect(-wallHalf, wallTop, wallHalf * 2, -wallTop) }
      earthFill(g, wallTrace, [-wallHalf, wallTop, wallHalf, 0], random, { layer: 8, top: 0.42, bottom: 0.3, ink: 0.12, shade: 0.12 })
      const shade = g.createLinearGradient(0, wallTop, 0, 0)
      shade.addColorStop(0, rgba(INDIGO, 0)); shade.addColorStop(1, rgba(INDIGO, 0.22))
      g.fillStyle = shade; g.fillRect(-wallHalf, wallTop, wallHalf * 2, -wallTop)
      // Brick joints.
      g.strokeStyle = rgba(INK, 0.16); g.lineWidth = 0.6
      for (let row = 0; row < 6; row++) {
        const y = wallTop + 14 + row * 14
        for (let x = -wallHalf + (row % 2) * 10; x < wallHalf; x += 20 + random() * 6) { g.beginPath(); g.moveTo(x, y - 14); g.lineTo(x, y); g.stroke() }
      }
      // Crenellations along both wings.
      for (let x = -wallHalf; x < wallHalf; x += 17) {
        if (Math.abs(x + 4) < 76) continue
        const hh = 9 + (random() < 0.12 ? -5 : 0)
        earthFill(g, () => { g.beginPath(); g.rect(x, wallTop - hh, 10, hh) }, [x, wallTop - hh, x + 10, wallTop], random, { layer: 4, shade: 0.2 })
        inkOutline(g, [[x, wallTop], [x, wallTop - hh], [x + 10, wallTop - hh], [x + 10, wallTop]], 1.2, 0.82)
      }
      dryBrush(g, (t) => [lerp(-wallHalf, wallHalf, t), wallTop - 0.4, 0, 1], 2.6, 5, INK, 0.85, 3, 90)
      g.strokeStyle = rgba(INK, 0.5); g.lineWidth = 1; g.beginPath(); g.moveTo(-wallHalf, wallTop + 4); g.lineTo(wallHalf, wallTop + 4); g.stroke()
      // Gate bastion.
      const baseTop = -112
      earthFill(g, () => { g.beginPath(); g.rect(-72, baseTop, 144, -baseTop) }, [-72, baseTop, 72, 0], random, { layer: 7, top: 0.5, bottom: 0.36, ink: 0.14, shade: 0.3 })
      inkOutline(g, [[-72, 0], [-72, baseTop], [72, baseTop], [72, 0]], 1.8, 0.88)
      g.strokeStyle = rgba(INK, 0.4); g.lineWidth = 1; g.beginPath(); g.moveTo(-76, baseTop + 6); g.lineTo(76, baseTop + 6); g.stroke()
      // Arched gateway.
      g.fillStyle = rgba(INK, 0.86)
      g.beginPath(); g.moveTo(-21, 0); g.lineTo(-21, -48); g.quadraticCurveTo(-21, -72, 0, -72); g.quadraticCurveTo(21, -72, 21, -48); g.lineTo(21, 0); g.closePath(); g.fill()
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2; g.beginPath(); g.moveTo(-25, 0); g.lineTo(-25, -48); g.quadraticCurveTo(-25, -77, 0, -77); g.quadraticCurveTo(25, -77, 25, -48); g.lineTo(25, 0); g.stroke()
      g.fillStyle = rgba(OCHRE, 0.3); g.fillRect(-1, -70, 2, 70)
      g.fillStyle = rgba(INK, 0.5)
      for (let i = -2; i <= 2; i++) for (let j = 0; j < 4; j++) { g.beginPath(); g.arc(i * 6.5 + 3, -10 - j * 14, 0.9, 0, Math.PI * 2); g.fill() }
      // Tower: lower storey.
      const colAlpha = 0.82
      const storey = (half, y0, y1, cols) => {
        g.fillStyle = rgba(OCHRE_L, 0.4); g.fillRect(-half, y1, half * 2, y0 - y1)
        g.fillStyle = rgba(INK, 0.08); g.fillRect(-half, y1, half * 2, y0 - y1)
        g.strokeStyle = rgba(INK, 0.4); g.lineWidth = 0.6
        for (let i = 0; i < cols; i++) {
          const x0 = -half + (half * 2) * i / cols + 4, x1 = -half + (half * 2) * (i + 1) / cols - 4
          g.fillStyle = rgba(INK, 0.35); g.fillRect(x0, y1 + 8, x1 - x0, (y0 - y1) - 14)
          for (let x = x0; x <= x1; x += 4) { g.beginPath(); g.moveTo(x, y1 + 8); g.lineTo(x, y0 - 6); g.stroke() }
          for (let y = y1 + 8; y <= y0 - 6; y += 5) { g.beginPath(); g.moveTo(x0, y); g.lineTo(x1, y); g.stroke() }
        }
        g.fillStyle = rgba(INK, colAlpha)
        for (let i = 0; i <= cols; i++) g.fillRect(-half + (half * 2) * i / cols - 2, y1, 4, y0 - y1)
        g.fillRect(-half - 3, y1, half * 2 + 6, 3.5); g.fillRect(-half - 3, y0 - 3, half * 2 + 6, 3)
      }
      storey(58, baseTop, baseTop - 42, 4)
      tileRoof(g, 0, 82, baseTop - 42, 20, 0.84)
      storey(40, baseTop - 62, baseTop - 100, 3)
      g.fillStyle = rgba(PAPER, 0.85); g.fillRect(-14, baseTop - 98, 28, 9)
      g.strokeStyle = rgba(INK, 0.7); g.lineWidth = 1; g.strokeRect(-14, baseTop - 98, 28, 9)
      g.fillStyle = rgba(INK, 0.8); g.font = '700 7px ' + KAITI; g.textAlign = 'center'; g.textBaseline = 'middle'; g.fillText('雄 关', 0, baseTop - 93.5)
      tileRoof(g, 0, 64, baseTop - 100, 34, 0.88)
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 1.4; g.beginPath(); g.moveTo(0, baseTop - 134); g.lineTo(0, baseTop - 148); g.stroke()
      g.fillStyle = rgba(INK, 0.9); g.beginPath(); g.arc(0, baseTop - 150, 2.3, 0, Math.PI * 2); g.fill()
      // Weathering at the wall feet and broken tips.
      for (let i = 0; i < 40; i++) {
        const x = (random() - 0.5) * 440, r = 1 + random() * 3
        g.fillStyle = rgba(random() < 0.5 ? OCHRE : INK, 0.15 + random() * 0.25)
        g.beginPath(); g.ellipse(x, -r * 0.3, r * 1.8, r * 0.6, 0, 0, Math.PI * 2); g.fill()
      }
      // Dissolve both far ends of the wall into the paper.
      g.save(); g.globalCompositeOperation = 'destination-out'
      for (const s of [-1, 1]) {
        const f = g.createLinearGradient(s * (wallHalf - 90), 0, s * wallHalf, 0)
        f.addColorStop(0, 'rgba(0,0,0,0)'); f.addColorStop(1, 'rgba(0,0,0,1)')
        g.fillStyle = f; g.fillRect(s > 0 ? wallHalf - 90 : -wallHalf, wallTop - 14, 90, 16 - wallTop)
      }
      g.restore()
    }, 230, 268)
  }

  // 将台: stepped command platform with banners and a drum. Origin at base centre.
  function dais(res) {
    const random = rng(9917)
    return sprite(380, 230, res, (g) => {
      g.fillStyle = rgba(INK, 0.14); g.beginPath(); g.ellipse(0, 1, 196, 4, 0, 0, Math.PI * 2); g.fill()
      const tier = (half, y0, y1, opts) => {
        earthFill(g, () => { g.beginPath(); g.moveTo(-half, y0); g.lineTo(-half + 3, y1); g.lineTo(half - 3, y1); g.lineTo(half, y0); g.closePath() }, [-half, y1, half, y0], random, opts)
        inkOutline(g, [[-half, y0], [-half + 3, y1], [half - 3, y1], [half, y0]], 1.7, 0.85)
        g.strokeStyle = rgba(INK, 0.3); g.lineWidth = 0.6
        for (let x = -half + 8; x < half - 4; x += 15 + random() * 5) { g.beginPath(); g.moveTo(x, y0); g.lineTo(x, y1 + 3); g.stroke() }
        dryBrush(g, (t) => [lerp(-half + 3, half - 3, t), y1 - 0.4, 0, 1], 2.8, 5, INK, 0.9, y1 + 9, 40)
      }
      tier(188, 0, -30, { layer: 8, shade: 0.28 })
      tier(150, -30, -58, { layer: 7, shade: 0.28 })
      tier(112, -58, -86, { layer: 7, shade: 0.28 })
      // Stairs.
      for (let i = 0; i < 14; i++) {
        const y = -i * 6.15, half = lerp(31, 28, i / 14)
        g.fillStyle = rgba(i % 2 ? OCHRE_L : PAPER, i % 2 ? 0.55 : 0.5); g.fillRect(-half, y - 6.15, half * 2, 6.15)
        g.strokeStyle = rgba(INK, 0.55); g.lineWidth = 0.9; g.beginPath(); g.moveTo(-half, y - 6.15); g.lineTo(half, y - 6.15); g.stroke()
      }
      inkOutline(g, [[-31, 0], [-28, -86]], 1.5, 0.8); inkOutline(g, [[31, 0], [28, -86]], 1.5, 0.8)
      // Railing around the top platform, open at the stairs.
      g.strokeStyle = rgba(INK, 0.86); g.lineWidth = 1.5
      for (const s of [-1, 1]) {
        g.beginPath(); g.moveTo(s * 30, -100); g.lineTo(s * 108, -100); g.moveTo(s * 30, -93); g.lineTo(s * 108, -93); g.stroke()
        for (let x = 30; x <= 108; x += 13) { g.lineWidth = 2; g.beginPath(); g.moveTo(s * x, -86); g.lineTo(s * x, -104); g.stroke() }
        g.lineWidth = 1.5
      }
      // Flag poles and banners.
      for (const [x, glyph] of [[-92, '萧'], [92, '令']]) {
        g.fillStyle = rgba(INK, 0.1); g.beginPath(); g.ellipse(x, -86, 7, 1.6, 0, 0, Math.PI * 2); g.fill()
        g.strokeStyle = rgba(INK, 0.92); g.lineWidth = 2.6; g.beginPath(); g.moveTo(x, -86); g.lineTo(x, -224); g.stroke()
        g.strokeStyle = rgba(OCHRE, 0.7); g.lineWidth = 0.8; g.beginPath(); g.moveTo(x - 0.6, -90); g.lineTo(x - 0.6, -218); g.stroke()
        g.fillStyle = rgba(INK, 0.92); g.beginPath(); g.moveTo(x, -229); g.lineTo(x - 2.4, -221); g.lineTo(x + 2.4, -221); g.closePath(); g.fill()
        flagCloth(g, x + 1.5, -219, 46, 96, OCHRE_L, 0.74, glyph, INK, 0.62, random, 40)
        g.strokeStyle = rgba(INK, 0.7); g.lineWidth = 1.3; g.beginPath(); g.moveTo(x, -217); g.lineTo(x + 47, -217); g.stroke()
      }
      // War drum on a stand.
      const dx = 62, dy = -100
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 2.2
      g.beginPath(); g.moveTo(dx - 14, -86); g.lineTo(dx - 10, dy + 2); g.moveTo(dx + 14, -86); g.lineTo(dx + 10, dy + 2); g.moveTo(dx - 16, -86); g.lineTo(dx + 16, -86); g.stroke()
      g.fillStyle = rgba(INK, 0.88)
      g.beginPath(); g.moveTo(dx - 16, dy - 3); g.quadraticCurveTo(dx, dy - 21, dx + 16, dy - 3); g.quadraticCurveTo(dx + 20, dy + 8, dx + 16, dy + 12); g.quadraticCurveTo(dx, dy + 16, dx - 16, dy + 12); g.quadraticCurveTo(dx - 20, dy + 8, dx - 16, dy - 3); g.closePath(); g.fill()
      g.fillStyle = rgba(OCHRE_L, 0.8); g.beginPath(); g.ellipse(dx + 15, dy + 5, 4.4, 9, 0.05, 0, Math.PI * 2); g.fill()
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 1; g.beginPath(); g.ellipse(dx + 15, dy + 5, 4.4, 9, 0.05, 0, Math.PI * 2); g.stroke()
      g.fillStyle = rgba(PAPER, 0.55)
      for (let i = 0; i < 6; i++) { g.beginPath(); g.arc(dx - 12 + i * 5, dy - 8 + Math.sin(i * 0.5 + 0.4) * -2 + (i % 2) * 0.1, 1, 0, Math.PI * 2); g.fill() }
      g.strokeStyle = rgba(INK, 0.9); g.lineWidth = 1.3; g.beginPath(); g.moveTo(dx - 6, dy - 16); g.lineTo(dx - 18, dy - 26); g.moveTo(dx + 2, dy - 16); g.lineTo(dx - 4, dy - 28); g.stroke()
      // Tufts and rubble at the foot.
      for (let i = 0; i < 22; i++) {
        const x = (random() - 0.5) * 390, r = 1 + random() * 2.4
        g.fillStyle = rgba(random() < 0.5 ? OCHRE : INK, 0.2 + random() * 0.3)
        g.beginPath(); g.ellipse(x, -r * 0.3, r * 1.6, r * 0.6, 0, 0, Math.PI * 2); g.fill()
      }
    }, 190, 228)
  }

  globalThis.XiabanInkArt = {
    PAPER, INK, CINNABAR, rgba, rng, hash, noise, fbm, canvas, strip, drawStrip, paperGrain, vignette,
    ribbon, dryBrush, leaf, leafCluster, mountains, bamboo, ground, snowGround, iceSurface, reeds, water,
    sprite, drawSprite, rock, bridge, gate, lantern, deadTree, pine, snowPine, snowPier, snowRock, snowForest, plumBranches, inkCloud, blossom, thinIce, icicleSprite, pillar, stele, shrine,
    splat, softDisc, sun, moon, mist, leafSprite, petalSprite, waveSprite, iceSpikeSprite, snowflakeSprite, clamp, lerp,
    desertGround, dryGrass, poplar, poplarGrove, beaconTower, rampart, earthBlock, cityWall, tents, dais, banner, ochreLeafSprite, dustSprite, potSprite, flameSprites,
  }
})()
