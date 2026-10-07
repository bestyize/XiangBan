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

  function deadTree(res, seed, scale = 1) {
    const random = rng(seed)
    return sprite(220 * scale, 260 * scale, res, (g) => {
      g.scale(scale, scale)
      g.translate(-110, -258)
      const branch = (x, y, angle, len, width, depth) => {
        const x2 = x + Math.cos(angle) * len, y2 = y + Math.sin(angle) * len
        g.fillStyle = rgba(INK, 0.85)
        ribbon(g, [[x, y], [(x + x2) / 2 + (random() - 0.5) * len * 0.2, (y + y2) / 2], [x2, y2]], (t) => width * (1 - t * 0.55))
        if (depth <= 0 || width < 0.8) return
        const count = depth > 2 ? 2 : 1 + Math.floor(random() * 2)
        for (let i = 0; i < count; i++) branch(x2, y2, angle + (random() - 0.5) * 1.3 - 0.1, len * (0.55 + random() * 0.25), width * 0.62, depth - 1)
      }
      branch(110, 258, -Math.PI / 2 + 0.05, 90, 12, 5)
      branch(110, 200, -Math.PI / 2 - 0.7, 50, 5, 3)
      g.fillStyle = rgba(INK, 0.15); g.beginPath(); g.ellipse(110, 258, 40, 3, 0, 0, Math.PI * 2); g.fill()
    }, 110 * scale, 258 * scale)
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

  function pillar(res, seed) {
    const random = rng(seed)
    return sprite(40, 120, res, (g) => {
      g.fillStyle = rgba(INK, 0.75)
      g.beginPath(); g.moveTo(-9, 0); g.lineTo(-8, -88); g.lineTo(-2, -94); g.lineTo(3, -86); g.lineTo(8, -97); g.lineTo(9, 0); g.closePath(); g.fill()
      for (let i = 0; i < 12; i++) { g.fillStyle = rgba(PAPER, 0.18 + random() * 0.2); g.fillRect(-6 + random() * 10, -random() * 85, 1, 6 + random() * 10) }
      g.fillStyle = rgba(INK, 0.15); g.beginPath(); g.ellipse(0, 0, 16, 2.5, 0, 0, Math.PI * 2); g.fill()
    }, 20, 116)
  }

  function stele(res) {
    return sprite(40, 70, res, (g) => {
      g.fillStyle = rgba(INK, 0.72)
      g.beginPath(); g.moveTo(-11, 0); g.lineTo(-10, -44); g.quadraticCurveTo(0, -54, 10, -44); g.lineTo(11, 0); g.closePath(); g.fill()
      g.fillStyle = rgba(INK, 0.5); g.fillRect(-15, -4, 30, 4)
      g.fillStyle = rgba(PAPER, 0.55)
      for (let i = 0; i < 4; i++) g.fillRect(-1.2, -38 + i * 8, 2.4, 5)
    }, 20, 66)
  }

  function shrine(res) {
    return sprite(164, 140, res, (g) => {
      g.translate(-75, -138)
      g.fillStyle = rgba(INK, 0.55)
      g.fillRect(16, 60, 8, 78); g.fillRect(126, 60, 8, 78)
      g.fillRect(10, 128, 130, 10)
      g.fillStyle = rgba(INK, 0.72)
      g.beginPath(); g.moveTo(-6, 58); g.quadraticCurveTo(30, 52, 75, 22); g.quadraticCurveTo(120, 52, 156, 58); g.quadraticCurveTo(140, 66, 128, 64); g.lineTo(22, 64); g.quadraticCurveTo(10, 66, -6, 58); g.fill()
      g.fillStyle = 'rgba(214,120,62,0.45)'; g.fillRect(60, 86, 30, 22)
      g.fillStyle = rgba(INK, 0.5); g.fillRect(60, 84, 30, 3); g.fillRect(74, 86, 2, 22)
    }, 75, 138)
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

  globalThis.XiabanInkArt = {
    PAPER, INK, CINNABAR, rgba, rng, hash, noise, fbm, canvas, strip, drawStrip, paperGrain, vignette,
    ribbon, dryBrush, leaf, leafCluster, mountains, bamboo, ground, water, sprite, drawSprite, rock, bridge,
    gate, lantern, deadTree, pine, pillar, stele, shrine, splat, softDisc, sun, moon, mist, leafSprite, petalSprite, waveSprite, clamp, lerp,
  }
})()
