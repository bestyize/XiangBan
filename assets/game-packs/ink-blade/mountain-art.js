/* 悬山 · Volume 1 ink-wash DNA: overlapping parabolic peaks with 6 stage profiles, 7-plane parallax depth, crisp dark-ink props. */
(() => {
  'use strict'

  const A = globalThis.XiabanInkArt
  const R = globalThis.XiabanInkBlade
  const G = R.GROUND
  const INK = A.INK
  const PAPER = A.PAPER
  const CELADON = [54, 88, 84]
  const CINNABAR = '#b8322a'
  const TAU = Math.PI * 2
  const BRIDGE = [1440, 2320]
  const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v))
  const smooth = (t) => { const v = clamp(t, 0, 1); return v * v * (3 - 2 * v) }

  // ---------------------------------------------------------------- Vol 1 style roofs & dark-ink props

  function darkRoof(g, cx, half, y, depth, alpha = 0.92) {
    g.fillStyle = A.rgba(INK, alpha)
    g.beginPath()
    g.moveTo(cx - half - 16, y - 11)
    g.quadraticCurveTo(cx - half + 2, y + 3, cx - half + 14, y + 3)
    g.lineTo(cx + half - 14, y + 3)
    g.quadraticCurveTo(cx + half - 2, y + 3, cx + half + 16, y - 11)
    g.quadraticCurveTo(cx + half - 2, y - depth * 0.45, cx + half * 0.8, y - depth)
    g.lineTo(cx - half * 0.8, y - depth)
    g.quadraticCurveTo(cx - half + 2, y - depth * 0.45, cx - half - 16, y - 11)
    g.closePath(); g.fill()
    g.strokeStyle = A.rgba(PAPER, 0.22); g.lineWidth = 0.7
    for (let x = cx - half * 0.78; x < cx + half * 0.78; x += 6) {
      g.beginPath(); g.moveTo(x, y - depth + 2); g.lineTo(x + (x - cx) * 0.08, y); g.stroke()
    }
    g.strokeStyle = A.rgba(INK, 0.95); g.lineWidth = 3.2
    g.beginPath(); g.moveTo(cx - half * 0.86, y - depth); g.lineTo(cx + half * 0.86, y - depth); g.stroke()
    g.lineWidth = 1.9
    for (const side of [-1, 1]) {
      g.beginPath()
      g.arc(cx + side * half * 0.86, y - depth - 3.8, 3.8, side > 0 ? Math.PI * 0.9 : -Math.PI * 0.1, side > 0 ? Math.PI * 2.2 : Math.PI * 1.2, side < 0)
      g.stroke()
    }
    g.fillStyle = A.rgba(INK, alpha * 0.82)
    for (let x = cx - half + 10; x <= cx + half - 10; x += 9) g.fillRect(x - 2.2, y + 3, 4.4, 4.5)
  }

  // 岩关山门: combines art.rock's dark-to-mist stone piers with art.gate's crisp three-bay ink roof & cinnabar plaque.
  function mountainGateSprite(res) {
    const rand = A.rng(601)
    return A.sprite(310, 232, res, (g) => {
      g.fillStyle = A.rgba(INK, 0.14); g.beginPath(); g.ellipse(0, 0, 142, 4, 0, 0, TAU); g.fill()

      // Left & right dark ink rock-cut pass piers (art.rock style: dark top, fades to mist at ground)
      for (const side of [-1, 1]) {
        const x0 = side < 0 ? -128 : 44
        const x1 = side < 0 ? -44 : 128
        const w = x1 - x0, h = 122
        const body = g.createLinearGradient(0, -h, 0, 0)
        body.addColorStop(0, A.rgba(INK, 0.88))
        body.addColorStop(0.28, A.rgba(INK, 0.56))
        body.addColorStop(1, A.rgba(INK, 0.18))
        g.fillStyle = body
        g.beginPath()
        if (side < 0) {
          g.moveTo(-132, 0); g.quadraticCurveTo(-126, -68, -112, -h + 4)
          g.lineTo(-44, -h); g.lineTo(-44, 0)
        } else {
          g.moveTo(44, 0); g.lineTo(44, -h); g.lineTo(112, -h + 4)
          g.quadraticCurveTo(126, -68, 132, 0)
        }
        g.closePath(); g.fill()
        // Axe-cut 斧劈皴 texture inside stone piers
        for (let i = 0; i < 24; i++) {
          const sx = x0 + 6 + rand() * (w - 14), sy = -h + 8 + rand() * (h - 18)
          g.fillStyle = A.rgba(INK, 0.14 + rand() * 0.32)
          g.beginPath(); g.moveTo(sx, sy); g.lineTo(sx + 4 + rand() * 6, sy + 3 + rand() * 11); g.lineTo(sx + 1, sy + 2); g.closePath(); g.fill()
        }
        A.dryBrush(g, (t) => [A.lerp(x0 + 4, x1 - 4, t), -h + Math.sin(t * Math.PI) * -1.2, 0, 1], 3.2, 6, INK, 0.94, 605 + side, 26)
        // Moss dots (点苔) on stone shoulders
        for (let i = 0; i < 9; i++) {
          g.fillStyle = A.rgba(INK, 0.65 + rand() * 0.3)
          g.beginPath(); g.ellipse(x0 + 8 + rand() * (w - 16), -h - 1.5 - rand() * 2, 1.4 + rand() * 1.6, 0.9 + rand(), 0, 0, TAU); g.fill()
        }
      }

      // Dark-ink stone arch beam spanning the opening
      const archGrad = g.createLinearGradient(0, -126, 0, -76)
      archGrad.addColorStop(0, A.rgba(INK, 0.88)); archGrad.addColorStop(1, A.rgba(INK, 0.68))
      g.fillStyle = archGrad
      g.beginPath()
      g.moveTo(-52, -74); g.lineTo(-52, -124); g.lineTo(52, -124); g.lineTo(52, -74)
      g.lineTo(44, -74); g.quadraticCurveTo(40, -108, 0, -110); g.quadraticCurveTo(-40, -108, -44, -74)
      g.closePath(); g.fill()

      // Solid dark-ink timber pillars rising from the stone pass (art.gate style)
      for (const [px, top, pw] of [[-96, 152, 9], [96, 152, 9], [-44, 184, 11], [44, 184, 11]]) {
        g.fillStyle = A.rgba(INK, 0.88); g.fillRect(px - pw / 2, -top, pw, top)
        g.fillStyle = A.rgba(PAPER, 0.24); g.fillRect(px - pw / 2 + 2, -top + 10, 1.5, top - 22)
        g.fillStyle = A.rgba(INK, 0.56); g.beginPath(); g.roundRect(px - pw / 2 - 4, -14, pw + 8, 14, 2); g.fill()
      }
      // Crossbeams & three-tiered upturned dark ink roofs (exact art.gate aesthetic)
      g.fillStyle = A.rgba(INK, 0.86); g.fillRect(-106, -122, 212, 7); g.fillRect(-50, -158, 100, 7)
      g.fillStyle = A.rgba(INK, 0.68); g.fillRect(-106, -136, 62, 5); g.fillRect(44, -136, 62, 5)
      darkRoof(g, -74, 44, -152, 21, 0.9)
      darkRoof(g, 74, 44, -152, 21, 0.9)
      darkRoof(g, 0, 64, -188, 28, 0.95)

      // Crisp paper plaque with double ink frame & cinnabar calligraphy (exact art.gate match)
      g.fillStyle = A.rgba(PAPER, 0.94); g.fillRect(-30, -152, 60, 29)
      g.strokeStyle = A.rgba(INK, 0.9); g.lineWidth = 2.3; g.strokeRect(-30, -152, 60, 29)
      g.lineWidth = 0.8; g.strokeRect(-26, -148, 52, 21)
      g.fillStyle = CINNABAR
      g.font = '600 15px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
      g.textAlign = 'center'; g.textBaseline = 'middle'
      g.fillText('悬 山', 0, -137)
    }, 155, 228)
  }

  // 古石碑: exact Vol 1 dark-ink monolith aesthetic (art.stele / art.pillar) with carved plinth, weathered notches & cinnabar seal.
  function mountainStele(res, chars, h = 76, seed = 611) {
    const rand = A.rng(seed)
    return A.sprite(56, h + 18, res, (g) => {
      g.fillStyle = A.rgba(INK, 0.15); g.beginPath(); g.ellipse(0, 0, 22, 2.8, 0, 0, TAU); g.fill()
      // Solid dark-ink monolith body
      const body = g.createLinearGradient(0, -h, 0, 0)
      body.addColorStop(0, A.rgba(INK, 0.84)); body.addColorStop(0.7, A.rgba(INK, 0.76)); body.addColorStop(1, A.rgba(INK, 0.58))
      g.fillStyle = body
      g.beginPath()
      g.moveTo(-13, 0); g.lineTo(-12.5, -h + 14)
      g.quadraticCurveTo(0, -h - 3, 12.5, -h + 14)
      g.lineTo(13, 0); g.closePath(); g.fill()
      // Pale weathered streaks (matches art.pillar)
      for (let i = 0; i < 8; i++) {
        g.fillStyle = A.rgba(PAPER, 0.16 + rand() * 0.18)
        g.fillRect(-9 + rand() * 17, -10 - rand() * (h - 22), 1, 5 + rand() * 9)
      }
      // Stepped stone pedestal base
      g.fillStyle = A.rgba(INK, 0.65); g.fillRect(-17, -8, 34, 4)
      g.fillStyle = A.rgba(INK, 0.82); g.fillRect(-20, -4, 40, 4)
      // Moss dots (点苔) on top shoulder
      for (let i = 0; i < 5; i++) {
        g.fillStyle = A.rgba(INK, 0.75)
        g.beginPath(); g.ellipse(-8 + i * 4 + (rand() - 0.5) * 2, -h + 5 + Math.abs(i - 2) * 2, 1.5, 0.9, 0, 0, TAU); g.fill()
      }
      // Crisp pale paper incised calligraphy + tiny cinnabar chop
      g.fillStyle = A.rgba(PAPER, 0.82)
      g.font = '600 13px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
      g.textAlign = 'center'; g.textBaseline = 'middle'
      chars.forEach((ch, idx) => g.fillText(ch, 0, -h + 24 + idx * 18))
      g.fillStyle = CINNABAR
      g.fillRect(-2.2, -17, 4.4, 4.4)
    }, 28, h + 14)
  }

  // ---------------------------------------------------------------- Song/Ming gnarled pine, wind-bent deadwood & plum painters

  function polyLengths(pts) {
    const cum = [0]
    for (let i = 1; i < pts.length; i++) {
      cum.push(cum[i - 1] + Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]))
    }
    return cum
  }

  function samplePoly(pts, cum, t) {
    const total = cum[cum.length - 1] || 1
    const target = clamp(t, 0, 1) * total
    let i = 0
    while (i < cum.length - 2 && cum[i + 1] < target) i++
    const seg = cum[i + 1] - cum[i] || 1
    const u = (target - cum[i]) / seg
    const x = A.lerp(pts[i][0], pts[i + 1][0], u)
    const y = A.lerp(pts[i][1], pts[i + 1][1], u)
    const dx = pts[i + 1][0] - pts[i][0], dy = pts[i + 1][1] - pts[i][1]
    const l = Math.hypot(dx, dy) || 1
    return [x, y, -dy / l, dx / l]
  }

  // Sharp angular "iron-wire" subdivision for gnarled limbs & claws.
  function subdivideGnarled(knots, segsPerSpan, jitter, rand) {
    const pts = []
    for (let i = 0; i < knots.length - 1; i++) {
      const [x0, y0] = knots[i], [x1, y1] = knots[i + 1]
      const dx = x1 - x0, dy = y1 - y0, l = Math.hypot(dx, dy) || 1
      const nx = -dy / l, ny = dx / l
      for (let s = 0; s < segsPerSpan; s++) {
        const t = s / segsPerSpan
        const jig = s === 0 ? 0 : (rand() - 0.5) * jitter
        pts.push([x0 + dx * t + nx * jig, y0 + dy * t + ny * jig])
      }
    }
    pts.push(knots[knots.length - 1])
    return pts
  }

  // Flowing Catmull-Rom dragon spine with organic bark roughness (avoids polygon facets on thick trunks).
  function splineTrunk(knots, segsPerSpan, jitter, rand) {
    const pts = []
    const n = knots.length
    const at = (idx) => knots[clamp(idx, 0, n - 1)]
    for (let i = 0; i < n - 1; i++) {
      const p0 = at(i - 1), p1 = at(i), p2 = at(i + 1), p3 = at(i + 2)
      for (let s = 0; s < segsPerSpan; s++) {
        const t = s / segsPerSpan, t2 = t * t, t3 = t2 * t
        const x = 0.5 * ((2 * p1[0]) + (-p0[0] + p2[0]) * t + (2 * p0[0] - 5 * p1[0] + 4 * p2[0] - p3[0]) * t2 + (-p0[0] + 3 * p1[0] - 3 * p2[0] + p3[0]) * t3)
        const y = 0.5 * ((2 * p1[1]) + (-p0[1] + p2[1]) * t + (2 * p0[1] - 5 * p1[1] + 4 * p2[1] - p3[1]) * t2 + (-p0[1] + 3 * p1[1] - 3 * p2[1] + p3[1]) * t3)
        const w = (i === 0 && s === 0) ? 0 : (rand() - 0.5) * jitter
        pts.push([x + w, y])
      }
    }
    pts.push(knots[n - 1])
    return pts
  }

  // Irregular organic ink-wash blob (flatter underside resting on twigs, billowing crest on top).
  function drawOrganicCloudBlob(g, ox, oy, rx, ry, rand) {
    const n = 9
    const pts = []
    for (let i = 0; i < n; i++) {
      const a = (i / n) * TAU + (rand() - 0.5) * 0.25
      const ca = Math.cos(a), sa = Math.sin(a)
      const rScale = sa > 0.08 ? (0.42 + rand() * 0.24) : (0.78 + rand() * 0.44)
      pts.push([ox + ca * rx * (0.82 + rand() * 0.34), oy + sa * ry * rScale])
    }
    g.beginPath()
    const m0x = (pts[n - 1][0] + pts[0][0]) * 0.5, m0y = (pts[n - 1][1] + pts[0][1]) * 0.5
    g.moveTo(m0x, m0y)
    for (let i = 0; i < n; i++) {
      const p = pts[i], q = pts[(i + 1) % n]
      g.quadraticCurveTo(p[0], p[1], (p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5)
    }
    g.closePath(); g.fill()
  }

  // Layered Song-style pine needle fans (松针叠簇): organic ink-celadon cloud base + radiating steel-wire needle fans.
  function pineCanopyCloud(g, cx, cy, rx, ry, rand, opts = {}) {
    const fade = opts.fade ?? 1
    const tilt = opts.tilt ?? 0
    const deep = [24, 34, 32]
    const nBlobs = 5 + Math.floor(rand() * 3)
    for (let i = 0; i < nBlobs; i++) {
      const ox = cx + (rand() - 0.5) * rx * 0.92
      const oy = cy - rand() * ry * 0.34 + ry * 0.1
      const crx = rx * (0.34 + rand() * 0.26)
      const cry = ry * (0.48 + rand() * 0.28)
      g.fillStyle = A.rgba(i % 2 === 0 ? deep : INK, (0.14 + rand() * 0.14) * fade)
      drawOrganicCloudBlob(g, ox, oy, crx, cry, rand)
    }

    const fanLayer = (count, color, aMin, aMax, lenScale, lwMin, lwMax) => {
      g.lineCap = 'round'
      for (let i = 0; i < count; i++) {
        const spread = (i / Math.max(1, count - 1) - 0.5) * 2
        const hx = cx + spread * rx * 0.8 + (rand() - 0.5) * 6
        const arch = 1 - Math.min(1, spread * spread)
        const hy = cy + ry * 0.16 - arch * ry * 0.42 + (rand() - 0.5) * ry * 0.22
        const fanTilt = tilt + spread * 0.24
        const nNeedles = 10 + Math.floor(rand() * 5)
        const baseLen = (7.2 + rand() * 4.4) * lenScale
        g.strokeStyle = A.rgba(color, (aMin + rand() * (aMax - aMin)) * fade)
        g.lineWidth = lwMin + rand() * (lwMax - lwMin)
        g.beginPath()
        for (let k = 0; k < nNeedles; k++) {
          const frac = k / (nNeedles - 1)
          const ang = -Math.PI * (0.93 - frac * 0.86) + fanTilt + (rand() - 0.5) * 0.09
          const len = baseLen * (0.76 + rand() * 0.44)
          g.moveTo(hx, hy)
          g.lineTo(hx + Math.cos(ang) * len, hy + Math.sin(ang) * len)
        }
        g.stroke()
      }
    }

    const nMid = Math.round(rx / 6.8) + 5
    const nDark = Math.round(rx / 8.2) + 4
    fanLayer(nMid, deep, 0.52, 0.74, 1.05, 0.88, 1.32)
    fanLayer(nDark, INK, 0.82, 0.96, 0.96, 0.78, 1.18)
  }

  // Weathered dragon-scale pine bark (松鳞皴): dry-brush edges, tortoise-shell scale gouges & brief Xuan-paper gaps.
  function drawBarkFissures(g, pts, widthAt, rand, seed, opts = {}) {
    const cum = polyLengths(pts)
    const scaleCount = opts.fissures ?? 16

    // Heavy broken dry-brush bristles along the shaded flank & rough sunlit edge
    A.dryBrush(g, (t) => {
      const [x, y, nx, ny] = samplePoly(pts, cum, t)
      const off = -widthAt(t) * 0.45
      return [x + nx * off, y + ny * off, nx, ny]
    }, opts.brushW ?? 6.2, 7, INK, 0.9, seed + 17, 32)

    A.dryBrush(g, (t) => {
      const [x, y, nx, ny] = samplePoly(pts, cum, t)
      const off = widthAt(t) * 0.44
      return [x + nx * off, y + ny * off, nx, ny]
    }, (opts.brushW ?? 6.2) * 0.55, 4, INK, 0.52, seed + 23, 26)

    // Brief crooked pine bark scales (松鳞皴): short jagged dark ink crescents paired with tiny broken paper cracks
    g.lineCap = 'round'
    for (let i = 0; i < scaleCount; i++) {
      const t = 0.05 + rand() * 0.76
      const [x, y, nx, ny] = samplePoly(pts, cum, t)
      const w = widthAt(t)
      const lat = (rand() - 0.5) * w * 0.52
      const sx = x + nx * lat, sy = y + ny * lat
      const sh = 3.2 + rand() * 4.5, sw = 1.6 + rand() * 2.2
      g.strokeStyle = A.rgba(PAPER, 0.22 + rand() * 0.14)
      g.lineWidth = 0.7 + rand() * 0.35
      g.beginPath()
      g.moveTo(sx - ny * sh * 0.5, sy + nx * sh * 0.5)
      g.lineTo(sx + nx * sw * 0.3, sy)
      g.lineTo(sx + ny * sh * 0.5, sy - nx * sh * 0.5)
      g.stroke()
      if (rand() < 0.65) {
        g.strokeStyle = A.rgba([10, 10, 12], 0.72)
        g.lineWidth = 1.05
        g.beginPath()
        g.arc(sx - nx * 1.4, sy + 1, sw * 1.35, -0.9, 1.1)
        g.stroke()
      }
    }

    // Gnarled trunk knot scars (木瘤/节疤)
    const scars = opts.scars ?? 2
    for (let i = 0; i < scars; i++) {
      const t = 0.26 + (i + 0.35 + rand() * 0.3) * 0.28
      const [x, y, nx, ny] = samplePoly(pts, cum, t)
      const side = i % 2 === 0 ? 1 : -1
      const w = widthAt(t)
      const kx = x + nx * side * w * 0.15, ky = y + ny * side * w * 0.15
      g.strokeStyle = A.rgba(INK, 0.9); g.lineWidth = 1.4
      g.beginPath(); g.ellipse(kx, ky, w * 0.18, w * 0.24, side * 0.2, 0, TAU); g.stroke()
      g.strokeStyle = A.rgba(PAPER, 0.28); g.lineWidth = 0.75
      g.beginPath(); g.arc(kx, ky, w * 0.11, side > 0 ? 0.4 : 3.5, side > 0 ? 2.2 : 5.3); g.stroke()
    }
  }

  // Draws one angular iron-bent bough with dry-brush underscore, moss dots & exposed bare deadwood claw tips.
  function drawGnarledLimb(g, knots, w0, w1, rand, seed, opts = {}) {
    const pts = subdivideGnarled(knots, 3, opts.jitter ?? 1.4, rand)
    const widthAt = (t) => A.lerp(w0, w1, Math.pow(t, 0.82)) * (1 + 0.12 * Math.sin(t * 17 + seed))
    g.fillStyle = A.rgba(INK, opts.alpha ?? 0.92)
    A.ribbon(g, pts, widthAt)
    if (w0 >= 5.2) {
      const cum = polyLengths(pts)
      A.dryBrush(g, (t) => {
        const [x, y, nx, ny] = samplePoly(pts, cum, t)
        const off = widthAt(t) * 0.42
        return [x + nx * off, y + ny * off, nx, ny]
      }, 2.6, 4, INK, 0.78, seed + 5, 16)
      for (let i = 0; i < (opts.moss ?? 2); i++) {
        const k = pts[2 + Math.floor(rand() * (pts.length - 4))]
        g.fillStyle = A.rgba(INK, 0.84 + rand() * 0.14)
        g.beginPath(); g.ellipse(k[0] + (rand() - 0.5) * 4, k[1] - widthAt(0.4) * 0.48, 1.4 + rand() * 1.1, 0.85, 0, 0, TAU); g.fill()
      }
    }
    return pts
  }

  // Craggy anchored claw roots starting inside the trunk core and gripping weathered boulders + grass at y = 0.
  function drawRootButtress(g, baseX, baseW, rand, dirSign = 1) {
    g.fillStyle = A.rgba(INK, 0.14)
    g.beginPath(); g.ellipse(baseX, 1.5, baseW * 2.15, 4.0, 0, 0, TAU); g.fill()
    const rootSpecs = [
      [-baseW * 0.24, -14, -baseW * 1.1, -4, -baseW * 1.9, 1.6, baseW * 0.38],
      [-baseW * 0.08, -12, -baseW * 0.52, -2, -baseW * 1.02, 2.2, baseW * 0.28],
      [baseW * 0.12, -14, baseW * 0.88, -5, baseW * 1.6, 1.4, baseW * 0.35],
      [baseW * 0.24, -10, baseW * 1.25 * dirSign, -3, baseW * 1.95 * dirSign, 2.0, baseW * 0.24],
    ]
    for (const [sx, sy, mx, my, ex, ey, rw] of rootSpecs) {
      const pts = splineTrunk([[baseX + sx, sy], [baseX + mx, my], [baseX + ex, ey]], 4, 0.8, rand)
      g.fillStyle = A.rgba(INK, 0.92)
      A.ribbon(g, pts, (t) => rw * (1 - t * 0.86))
    }
    // Low weathered ink boulders & grass tucked into root hollows
    for (const [rx, rw, rh] of [[-baseW * 1.2, 11, 3.6], [baseW * 1.05, 10, 3.2]]) {
      const bg = g.createLinearGradient(0, -rh, 0, 2)
      bg.addColorStop(0, A.rgba(INK, 0.68)); bg.addColorStop(1, A.rgba(INK, 0.16))
      g.fillStyle = bg
      g.beginPath()
      g.moveTo(baseX + rx - rw, 1)
      g.quadraticCurveTo(baseX + rx - rw * 0.5, -rh * 1.15, baseX + rx, -rh)
      g.quadraticCurveTo(baseX + rx + rw * 0.55, -rh * 0.85, baseX + rx + rw, 1)
      g.closePath(); g.fill()
    }
    g.strokeStyle = A.rgba(INK, 0.68); g.lineWidth = 1.0
    for (const gx of [-baseW * 1.4, -baseW * 0.45, baseW * 0.55, baseW * 1.35]) {
      g.beginPath(); g.moveTo(baseX + gx, 1); g.quadraticCurveTo(baseX + gx + (rand() - 0.5) * 4, -4, baseX + gx + (rand() - 0.5) * 7, -6 - rand() * 3); g.stroke()
    }
  }

  // 苍劲古松: sculpted Song/Ming mountain pine with flowing dragon spine, pine-scale bark, bare claw twigs & needle clouds.
  function vigorousPine(res, seed, opts = {}) {
    const scale = opts.scale ?? 1
    const mirror = opts.mirror ?? 1
    const style = opts.style ?? 'court'
    const rand = A.rng(seed)

    const CONFIGS = {
      court: {
        baseW: 25, flare: 18,
        knots: [[0, 0], [-6, -34], [12, -78], [-8, -124], [8, -168], [-12, -206], [-34, -222]],
        boughs: [
          { t: 0.35, knots: [[0, 0], [-34, 9], [-78, -4], [-118, -20]], w0: 11.2, w1: 2.0, claws: [[-136, -30, 3], [-82, -16, 2]],
            clouds: [{ dx: -64, dy: -8, rx: 40, ry: 23 }, { dx: -116, dy: -22, rx: 36, ry: 21 }] },
          { t: 0.48, knots: [[0, 0], [32, 7], [70, -6], [104, -18]], w0: 10.2, w1: 1.9, claws: [[124, -27, 2.6], [74, -18, 2.0]],
            clouds: [{ dx: 56, dy: -8, rx: 36, ry: 21 }, { dx: 102, dy: -20, rx: 32, ry: 19 }] },
          { t: 0.67, knots: [[0, 0], [-36, 5], [-78, -10], [-114, -22]], w0: 9.2, w1: 1.7, claws: [[-130, -32, 2.4]],
            clouds: [{ dx: -66, dy: -11, rx: 42, ry: 24 }, { dx: -112, dy: -24, rx: 32, ry: 19 }] },
          { t: 0.80, knots: [[0, 0], [26, 3], [56, -12], [78, -20]], w0: 7.0, w1: 1.5, claws: [[96, -28, 2.1]],
            clouds: [{ dx: 52, dy: -14, rx: 32, ry: 19 }] },
          { t: 1.0, knots: [[0, 0], [-18, -6], [-44, -14], [-64, -12]], w0: 5.8, w1: 1.3, claws: [[-80, -18, 1.9], [-24, -24, 1.8]],
            clouds: [{ dx: -16, dy: -12, rx: 36, ry: 22 }, { dx: -56, dy: -16, rx: 30, ry: 19 }] },
        ],
      },
      summit: {
        baseW: 24, flare: 17,
        knots: [[0, 0], [-8, -36], [-22, -76], [-6, -118], [-24, -162], [-42, -196], [-62, -210]],
        boughs: [
          { t: 0.40, knots: [[0, 0], [-28, 7], [-62, -6], [-92, -16]], w0: 9.8, w1: 1.9, claws: [[-110, -24, 2.4]],
            clouds: [{ dx: -52, dy: -8, rx: 36, ry: 21 }, { dx: -90, dy: -18, rx: 28, ry: 17 }] },
          // Heroic "迎客枝": long sweeping iron arm extending cleanly beneath boss HUD & sun
          { t: 0.45, knots: [[0, 0], [42, 11], [92, 6], [142, -10], [188, -18]], w0: 10.4, w1: 1.7, claws: [[210, -26, 2.5], [148, -22, 2.0]],
            clouds: [{ dx: 72, dy: 2, rx: 34, ry: 20 }, { dx: 132, dy: -12, rx: 38, ry: 22 }, { dx: 184, dy: -20, rx: 30, ry: 18 }] },
          { t: 0.72, knots: [[0, 0], [-30, 4], [-64, -10], [-86, -18]], w0: 7.8, w1: 1.5, claws: [[-102, -26, 2.0]],
            clouds: [{ dx: -48, dy: -10, rx: 34, ry: 20 }, { dx: -80, dy: -20, rx: 26, ry: 16 }] },
          { t: 0.91, knots: [[0, 0], [18, -3], [40, -12]], w0: 5.4, w1: 1.2, claws: [[56, -20, 1.7]],
            clouds: [{ dx: 26, dy: -12, rx: 28, ry: 17 }] },
          { t: 1.0, knots: [[0, 0], [-18, -7], [-42, -13]], w0: 5.6, w1: 1.3, claws: [[-58, -21, 1.8]],
            clouds: [{ dx: -16, dy: -13, rx: 34, ry: 20 }, { dx: -42, dy: -16, rx: 24, ry: 15 }] },
        ],
      },
      gate: {
        baseW: 22, flare: 16,
        knots: [[0, 0], [14, -30], [5, -68], [24, -110], [10, -148], [28, -176]],
        boughs: [
          { t: 0.38, knots: [[0, 0], [-24, 7], [-52, -4], [-74, -14]], w0: 8.8, w1: 1.7, claws: [[-90, -22, 2.2]],
            clouds: [{ dx: -48, dy: -8, rx: 34, ry: 20 }] },
          { t: 0.56, knots: [[0, 0], [28, 6], [64, -6], [92, -16]], w0: 9.0, w1: 1.6, claws: [[110, -24, 2.3]],
            clouds: [{ dx: 48, dy: -8, rx: 34, ry: 20 }, { dx: 88, dy: -18, rx: 28, ry: 17 }] },
          { t: 0.78, knots: [[0, 0], [-24, 3], [-50, -10], [-66, -16]], w0: 6.8, w1: 1.4, claws: [[-80, -24, 1.8]],
            clouds: [{ dx: -42, dy: -12, rx: 30, ry: 18 }] },
          { t: 1.0, knots: [[0, 0], [18, -8], [40, -14]], w0: 5.2, w1: 1.2, claws: [[56, -22, 1.7], [-12, -20, 1.6]],
            clouds: [{ dx: 12, dy: -12, rx: 34, ry: 20 }] },
        ],
      },
      bell: {
        baseW: 24, flare: 17,
        knots: [[0, 0], [10, -34], [-10, -78], [10, -124], [-4, -166], [14, -202]],
        boughs: [
          { t: 0.34, knots: [[0, 0], [-30, 7], [-68, -6], [-96, -18]], w0: 9.8, w1: 1.8, claws: [[-114, -27, 2.3], [-68, -16, 1.8]],
            clouds: [{ dx: -52, dy: -8, rx: 36, ry: 21 }, { dx: -92, dy: -20, rx: 30, ry: 18 }] },
          { t: 0.50, knots: [[0, 0], [28, 5], [62, -7], [88, -17]], w0: 9.0, w1: 1.6, claws: [[104, -26, 2.1]],
            clouds: [{ dx: 48, dy: -8, rx: 34, ry: 20 }, { dx: 84, dy: -19, rx: 28, ry: 17 }] },
          { t: 0.68, knots: [[0, 0], [-28, 4], [-60, -10], [-82, -18]], w0: 7.6, w1: 1.4, claws: [[-98, -26, 1.9]],
            clouds: [{ dx: -44, dy: -10, rx: 34, ry: 20 }, { dx: -76, dy: -19, rx: 26, ry: 16 }] },
          { t: 0.84, knots: [[0, 0], [20, 2], [44, -10], [62, -16]], w0: 6.2, w1: 1.3, claws: [[76, -24, 1.8]],
            clouds: [{ dx: 40, dy: -12, rx: 28, ry: 17 }] },
          { t: 1.0, knots: [[0, 0], [16, -7], [40, -13]], w0: 5.4, w1: 1.2, claws: [[56, -21, 1.7], [-20, -20, 1.6]],
            clouds: [{ dx: 8, dy: -13, rx: 36, ry: 21 }, { dx: 38, dy: -16, rx: 26, ry: 16 }] },
        ],
      },
    }

    const cfg = CONFIGS[style] || CONFIGS.court
    const sprW = 480 * scale, sprH = 300 * scale, ox = 230 * scale, oy = 286 * scale
    return A.sprite(sprW, sprH, res, (g) => {
      g.scale(scale * mirror, scale)
      const trunkPts = splineTrunk(cfg.knots, 5, 1.1, rand)
      const trunkCum = polyLengths(trunkPts)
      const jointTs = cfg.boughs.map((b) => b.t)
      const trunkW = (t) => {
        let swell = 0
        for (const jt of jointTs) swell += 0.22 * Math.exp(-Math.pow((t - jt) / 0.055, 2))
        const flare = t < 0.16 ? cfg.flare * Math.pow((0.16 - t) / 0.16, 1.4) : 0
        return cfg.baseW * (1 - t * 0.72) * (1 + swell + 0.08 * Math.sin(t * 25 + seed) + 0.05 * Math.cos(t * 43)) + flare
      }

      const resolved = cfg.boughs.map((b) => {
        const [tx, ty] = samplePoly(trunkPts, trunkCum, b.t)
        return { ...b, ax: tx, ay: ty }
      })

      // 1. Rear misted pine needle clusters
      for (const b of resolved) {
        if (b.clouds.length > 1) {
          const c = b.clouds[0]
          pineCanopyCloud(g, b.ax + c.dx, b.ay + c.dy - 4, c.rx * 0.96, c.ry * 0.96, rand, { fade: 0.78 })
        }
      }

      // 2. Ground shadow, claw roots grip & main dragon trunk
      drawRootButtress(g, 0, cfg.baseW * 0.55, rand, 1)
      g.fillStyle = A.rgba(INK, 0.93)
      A.ribbon(g, trunkPts, trunkW)
      drawBarkFissures(g, trunkPts, trunkW, rand, seed, { scars: 2 })

      // 3. Iron boughs, forked support twigs & exposed bare deadwood claw tips
      for (let idx = 0; idx < resolved.length; idx++) {
        const b = resolved[idx]
        const absKnots = b.knots.map(([dx, dy]) => [b.ax + dx, b.ay + dy])
        drawGnarledLimb(g, absKnots, b.w0, b.w1, rand, seed + idx * 13, { moss: 2 })
        if (b.knots.length >= 3) {
          const mid = absKnots[Math.floor(b.knots.length / 2)]
          const dir = Math.sign(b.knots[b.knots.length - 1][0]) || 1
          drawGnarledLimb(g, [mid, [mid[0] + dir * 12, mid[1] - 10], [mid[0] + dir * 22, mid[1] - 14]], b.w0 * 0.36, 1.0, rand, seed + idx * 31, { moss: 0, alpha: 0.88 })
        }
        for (const c of b.clouds) {
          const base = absKnots[Math.max(1, absKnots.length - 2)]
          drawGnarledLimb(g, [base, [b.ax + c.dx * 0.78, b.ay + c.dy * 0.55], [b.ax + c.dx, b.ay + c.dy + 3]], Math.max(2.2, b.w1 * 1.15), 0.9, rand, seed + Math.round(b.ax + c.dx), { moss: 0, alpha: 0.9, jitter: 0.7 })
        }
        for (let c = 0; c < b.claws.length; c++) {
          const [cdx, cdy, cw] = b.claws[c]
          const base = absKnots[c === 0 ? absKnots.length - 2 : Math.max(1, absKnots.length - 3)]
          const endX = b.ax + cdx, endY = b.ay + cdy
          const elbowX = (base[0] + endX) * 0.5 + (rand() - 0.5) * 8
          const elbowY = (base[1] + endY) * 0.5 - 6
          drawGnarledLimb(g, [base, [elbowX, elbowY], [endX, endY]], cw, 0.55, rand, seed + idx * 41 + c, { moss: 0, alpha: 0.94, jitter: 0.9 })
        }
      }

      // 4. Foreground crisp pine needle clouds over supporting boughs
      for (const b of resolved) {
        const startIdx = b.clouds.length > 1 ? 1 : 0
        for (let i = startIdx; i < b.clouds.length; i++) {
          const c = b.clouds[i]
          pineCanopyCloud(g, b.ax + c.dx, b.ay + c.dy - 2, c.rx, c.ry, rand, { fade: 0.98 })
        }
      }
    }, ox, oy)
  }

  // 风折古柏 / 铁骨古梅: wind-carved deadwood/cypress with antler twigs, twig-anchored needles or dark-cinnabar plum buds.
  function vigorousDeadTree(res, seed, opts = {}) {
    const scale = opts.scale ?? 1
    const windDir = opts.windDir ?? 1
    const lean = opts.lean ?? 0.35
    const blossom = Boolean(opts.blossom)
    const sparseNeedles = Boolean(opts.sparseNeedles)
    const rand = A.rng(seed)

    const sprW = 340 * scale, sprH = 240 * scale, ox = 155 * scale, oy = 228 * scale
    return A.sprite(sprW, sprH, res, (g) => {
      g.scale(scale, scale)
      const dxTop = lean * 135
      const knots = [
        [0, 0],
        [dxTop * 0.16 - windDir * 8, -32],
        [dxTop * 0.42 + windDir * 10, -72],
        [dxTop * 0.72 - windDir * 4, -116],
        [dxTop * 0.96 + windDir * 8, -156],
      ]
      const trunkPts = splineTrunk(knots, 5, 1.0, rand)
      const trunkCum = polyLengths(trunkPts)
      const baseW = blossom ? 16 : 19
      const trunkW = (t) => baseW * (1 - t * 0.76) * (1 + 0.11 * Math.sin(t * 21 + seed)) + (t < 0.16 ? 10 * (0.16 - t) / 0.16 : 0)

      drawRootButtress(g, 0, baseW * 0.5, rand, windDir)
      g.fillStyle = A.rgba(INK, 0.92)
      A.ribbon(g, trunkPts, trunkW)
      drawBarkFissures(g, trunkPts, trunkW, rand, seed, { brushW: 4.5, innerW: 2.8, fissures: 10, scars: 1 })

      const tips = []
      const canopyAnchors = []
      const antler = (x, y, ang, len, w, depth, carryCloud = false) => {
        const elbow = ang + (rand() - 0.5) * 0.55 + windDir * 0.12
        const x1 = x + Math.cos(ang) * len * 0.52, y1 = y + Math.sin(ang) * len * 0.52
        const x2 = x1 + Math.cos(elbow) * len * 0.52, y2 = y1 + Math.sin(elbow) * len * 0.52
        drawGnarledLimb(g, [[x, y], [x1, y1], [x2, y2]], w, Math.max(0.65, w * 0.44), rand, seed + Math.round(x + y), { moss: 0, alpha: 0.92, jitter: 0.8 })
        tips.push([x2, y2, elbow, depth])
        if (carryCloud && depth === 1) canopyAnchors.push([x2 + windDir * 4, y2 - 2, 26 + rand() * 8, 15 + rand() * 4])
        if (depth <= 0 || w < 1.3) return
        const kids = depth > 1 ? 2 : (rand() > 0.35 ? 2 : 1)
        for (let k = 0; k < kids; k++) {
          const spread = (k === 0 ? -1 : 1) * (0.38 + rand() * 0.42) + windDir * 0.18
          antler(x2, y2, elbow + spread * (kids === 1 ? 0.5 : 1), len * (0.58 + rand() * 0.2), w * 0.56, depth - 1, carryCloud && k === (windDir > 0 ? kids - 1 : 0))
        }
      }

      const [tMidX, tMidY] = samplePoly(trunkPts, trunkCum, 0.48)
      const [tHiX, tHiY] = samplePoly(trunkPts, trunkCum, 0.72)
      const topPt = trunkPts[trunkPts.length - 1]
      const mainAng = -Math.PI * 0.5 + windDir * 0.46
      antler(tMidX, tMidY, mainAng - windDir * 0.58, 48, 6.2, 2, false)
      antler(tHiX, tHiY, mainAng + windDir * 0.28, 52, 6.6, 2, true)
      antler(topPt[0], topPt[1], mainAng - windDir * 0.12, 46, 5.2, 2, true)
      antler(topPt[0], topPt[1], mainAng + windDir * 0.52, 40, 4.4, 2, true)

      if (sparseNeedles) {
        for (const [cx, cy, rx, ry] of canopyAnchors) {
          pineCanopyCloud(g, cx, cy, rx, ry, rand, { fade: 0.92, tilt: windDir * 0.14 })
        }
      }

      if (blossom) {
        // Restrained Song-ink plum buds & small 5-petal cinnabar blossoms along fine terminal twigs
        const darkCin = [164, 38, 30]
        const petalCin = [192, 54, 42]
        for (const [bx, by, , depth] of tips) {
          if (depth > 1 || rand() < 0.28) continue
          const buds = 2 + Math.floor(rand() * 3)
          for (let i = 0; i < buds; i++) {
            const px = bx + (rand() - 0.5) * 14, py = by + (rand() - 0.5) * 10
            const r = 1.6 + rand() * 1.4
            if (rand() < 0.55) {
              for (let p = 0; p < 5; p++) {
                const a = (p / 5) * TAU + rand() * 0.3
                g.fillStyle = A.rgba(p % 2 === 0 ? petalCin : darkCin, 0.78 + rand() * 0.18)
                g.beginPath(); g.arc(px + Math.cos(a) * r, py + Math.sin(a) * r, r * 0.62, 0, TAU); g.fill()
              }
              g.fillStyle = A.rgba(INK, 0.78); g.beginPath(); g.arc(px, py, 0.9, 0, TAU); g.fill()
            } else {
              g.fillStyle = A.rgba(darkCin, 0.84)
              g.beginPath(); g.arc(px, py, 1.3 + rand() * 0.7, 0, TAU); g.fill()
            }
          }
        }
      }
    }, ox, oy)
  }

  // ---------------------------------------------------------------- Vol 1 / Vol 2 style parallax pine forest & foreground framing

  // Mid-ground mountain pine & bamboo woods (0.55 & 0.80 parallax): asymmetric crooked Song pines + tall ink bamboo.
  function mountainForest(o) {
    const rand = A.rng(o.seed)
    const trees = []
    for (let i = 0; i < o.count; i++) {
      trees.push({
        x: rand() * o.period,
        h: A.lerp(o.height[0], o.height[1], rand()),
        tiers: 3 + Math.floor(rand() * 3),
        lean: (rand() - 0.5) * 0.24,
        w: A.lerp(0.24, 0.38, rand()),
        bamboo: rand() < 0.4,
        dir: rand() < 0.5 ? -1 : 1,
        tierSide: Array.from({ length: 5 }, (_, k) => (k % 2 === 0 ? 1 : -1) * (0.65 + rand() * 0.6)),
      })
    }
    return A.strip(o.period, o.top, o.base - o.top + 8, o.res, (g, u0, u1) => {
      for (const shift of [-o.period, 0, o.period]) for (const t of trees) {
        const x0 = t.x + shift
        if (x0 + t.h * 0.7 < u0 || x0 - t.h * 0.7 > u1) continue
        if (t.bamboo) {
          const topY = o.base - t.h * 1.18
          g.strokeStyle = A.rgba(o.color, o.alpha * 0.78); g.lineWidth = Math.max(1.2, t.h * 0.024)
          g.beginPath(); g.moveTo(x0, o.base); g.quadraticCurveTo(x0 + t.lean * t.h * 0.45, (o.base + topY) * 0.5, x0 + t.lean * t.h * 0.7, topY); g.stroke()
          for (let k = 0.32; k <= 0.95; k += 0.26) {
            const lx = x0 + t.lean * t.h * 0.7 * k, ly = o.base - (o.base - topY) * k
            A.leafCluster(g, lx, ly, k % 0.52 < 0.26 ? -0.45 : -2.65, 3, t.h * 0.16, 2.2, o.color, o.alpha * 0.88, rand)
          }
          continue
        }
        // Asymmetric gnarled mid-ground Song pine: sinuous tapered trunk + staggered bough clouds + short bristles
        const midY = o.base - t.h * 0.52, topY = o.base - t.h * 0.94
        const midX = x0 + t.lean * t.h * 0.45 + t.dir * t.h * 0.06
        const topX = x0 + t.lean * t.h * 0.85 - t.dir * t.h * 0.04
        g.fillStyle = A.rgba(o.color, Math.min(1, o.alpha * 1.18))
        A.ribbon(g, [[x0, o.base], [midX, midY], [topX, topY]], (u) => Math.max(0.8, t.h * 0.042 * (1 - u * 0.72)))
        for (let k = 0; k < t.tiers; k++) {
          const f = (k + 0.6) / (t.tiers + 0.6)
          const trY = A.lerp(o.base, topY, 0.26 + 0.68 * f)
          const trX = A.lerp(x0, topX, 0.26 + 0.68 * f) + Math.sin(f * Math.PI) * t.dir * t.h * 0.05
          const reach = t.h * t.w * (1 - f * 0.55) * t.tierSide[k]
          const bx = trX + reach, by = trY - Math.abs(reach) * 0.14
          g.strokeStyle = A.rgba(o.color, Math.min(1, o.alpha * 1.15)); g.lineWidth = Math.max(0.75, t.h * 0.02 * (1 - f * 0.4))
          g.beginPath(); g.moveTo(trX, trY); g.lineTo(trX + reach * 0.45, trY + 2); g.lineTo(bx, by); g.stroke()
          const rx = Math.abs(reach) * 0.72 + 4, ry = Math.max(4.5, rx * 0.42)
          const cx = trX + reach * 0.72, cy = by - ry * 0.2
          g.fillStyle = A.rgba(o.color, o.alpha * (0.64 + 0.26 * A.hash(k + 3, o.seed)))
          drawOrganicCloudBlob(g, cx, cy, rx, ry, rand)
          g.strokeStyle = A.rgba(o.color, Math.min(1, o.alpha * 1.1))
          g.lineWidth = 0.75
          g.beginPath()
          const needles = 5 + Math.round(rx / 5)
          for (let n = 0; n < needles; n++) {
            const nx = cx + (n / Math.max(1, needles - 1) - 0.5) * rx * 1.35
            const ang = -Math.PI * 0.5 + (rand() - 0.5) * 1.1
            const nl = 3.2 + rand() * 3.8
            g.moveTo(nx, cy); g.lineTo(nx + Math.cos(ang) * nl, cy + Math.sin(ang) * nl)
          }
          g.stroke()
        }
      }
    })
  }

  // Overlapping parabolic mountain peaks using art.mountains()'s exact brush engine, with 6 stage-specific peak profiles.
  function zoneWeights(wx) {
    const centers = [720, 1880, 2810, 3760, 4730, 5675]
    const w = [0, 0, 0, 0, 0, 0]
    if (wx <= centers[0]) { w[0] = 1; return w }
    if (wx >= centers[5]) { w[5] = 1; return w }
    for (let i = 0; i < 5; i++) {
      if (wx >= centers[i] && wx <= centers[i + 1]) {
        const t = smooth((wx - centers[i]) / (centers[i + 1] - centers[i]))
        w[i] = 1 - t; w[i + 1] = t
        return w
      }
    }
    return w
  }

  // Stage peak profiles: [wMin, wMax, hMin, hMax, round, skew, gapMin, gapMax]
  const PROFILES = [
    [28, 64, 115, 202, 1.15, 0.0,  0.42, 0.90], // 0 岩关: towering steep canyon peaks framing the pass
    [15, 34, 118, 206, 0.85, 0.0,  0.68, 1.38], // 1 悬栈: slender precipitous Huangshan/Zhangjiajie needle spires over wide cloud gaps
    [58, 136, 70, 146, 1.35, 0.42, 0.46, 0.92], // 2 风廊: strongly wind-tilted asymmetric granite ridges
    [74, 172, 48, 118, 1.95, 0.0,  0.54, 1.05], // 3 松庭: gentle rounded overlapping Jiangnan hills
    [40, 96,  86, 172, 1.28, -0.1, 0.45, 0.95], // 4 钟楼: deep solemn night peaks
    [54, 130, 18, 56,  1.65, 0.0,  0.62, 1.18], // 5 绝顶: low cloud-submerged summit crests far below the horizon
  ]

  function mountainScroll(o) {
    const random = A.rng(o.seed)
    const f = o.parallax, halfView = o.halfView, sc = o.scale
    const peaks = []
    let x = -80
    while (x < o.width + 80) {
      const wx = (x - halfView) / f
      const zw = zoneWeights(wx)
      let wMin = 0, wMax = 0, hMin = 0, hMax = 0, round = 0, skew = 0, gMin = 0, gMax = 0
      for (let i = 0; i < 6; i++) {
        const p = PROFILES[i], weight = zw[i]
        wMin += p[0] * weight; wMax += p[1] * weight
        hMin += p[2] * weight; hMax += p[3] * weight
        round += p[4] * weight; skew += p[5] * weight
        gMin += p[6] * weight; gMax += p[7] * weight
      }
      // Natural valley dip right behind the 岩关 stone gate (wx ~ 970) so the archway stands proud against the mist
      const passDip = 1 - 0.38 * Math.exp(-Math.pow((wx - 970) / 220, 2))
      const w = A.lerp(wMin, wMax, random()) * o.spacing
      const h = A.lerp(hMin, hMax, random()) * sc * passDip
      peaks.push({ x, w, h, round, skew })
      x += w * A.lerp(gMin, gMax, random())
    }

    const n = A.noise(o.seed + 11)
    const n2 = A.noise(o.seed + 23)
    const ridge = (u) => {
      let m = 0
      for (const p of peaks) {
        const raw = (u - p.x) / p.w
        if (raw > -1 && raw < 1) {
          const t = clamp(raw + p.skew * (1 - raw * raw), -1, 1)
          m = Math.max(m, p.h * Math.pow(1 - t * t, p.round))
        }
      }
      return o.base - m - (A.fbm(n, u / o.detail, 4) - 0.5) * o.detailAmp
    }

    const fade = A.canvas(1, 128)
    const fg = fade.getContext('2d')
    const grad = fg.createLinearGradient(0, 0, 0, 128)
    grad.addColorStop(0, A.rgba(o.color, 1)); grad.addColorStop(0.25, A.rgba(o.color, 0.55)); grad.addColorStop(1, A.rgba(o.color, 0))
    fg.fillStyle = grad; fg.fillRect(0, 0, 1, 128)
    const bottom = o.top + o.height

    return A.strip(o.width, o.top, o.height, o.res, (g, u0, u1) => {
      const a = Math.max(0, u0 - 12), b = Math.min(o.width, u1 + 12)
      const ys = []
      for (let u = a; u <= b; u += 1) ys.push(ridge(u))

      // Body wash (exact art.mountains formula)
      g.beginPath(); g.moveTo(a, bottom)
      for (let i = 0; i < ys.length; i++) g.lineTo(a + i, ys[i])
      g.lineTo(b, bottom); g.closePath()
      const body = g.createLinearGradient(0, o.base - 185 * sc, 0, bottom)
      body.addColorStop(0, A.rgba(o.color, o.alpha * 0.42)); body.addColorStop(1, A.rgba(o.color, o.alpha * 0.06))
      g.fillStyle = body; g.fill()

      if (o.ochre) {
        const ochre = g.createLinearGradient(0, o.base - 75 * sc, 0, bottom)
        ochre.addColorStop(0, 'rgba(168,116,64,0)'); ochre.addColorStop(1, 'rgba(168,116,64,' + o.ochre + ')')
        g.fillStyle = ochre; g.fill()
      }
      if (o.indigo) {
        const indigo = g.createLinearGradient(0, o.base - 185 * sc, 0, o.base - 20 * sc)
        indigo.addColorStop(0, 'rgba(70,96,120,' + o.indigo + ')'); indigo.addColorStop(1, 'rgba(70,96,120,0)')
        g.fillStyle = indigo; g.fill()
      }
      if (o.celadon) {
        const cel = g.createLinearGradient(0, o.base - 155 * sc, 0, bottom)
        cel.addColorStop(0, 'rgba(64,104,98,' + o.celadon + ')')
        cel.addColorStop(0.55, 'rgba(82,118,104,' + (o.celadon * 0.6) + ')')
        cel.addColorStop(1, 'rgba(82,118,104,0)')
        g.fillStyle = cel; g.fill()
      }

      // Ink settles along ridges and bleeds downward (墨分五色 — exact art.mountains formula)
      for (let i = 0; i < ys.length; i++) {
        const u = a + i
        const v = n2(u / 9)
        g.globalAlpha = o.alpha * (0.45 + 0.55 * v)
        g.drawImage(fade, u, ys[i] - 0.5, 1.15, o.depth * (0.55 + 0.9 * n(u / 23 + 50)))
      }
      g.globalAlpha = 1

      // Texture strokes (皴 — exact art.mountains formula)
      const cell = o.strokeCell
      for (let c = Math.floor(a / cell); c * cell < b; c++) {
        if (A.hash(c, o.seed) > o.strokeDensity) continue
        const u = c * cell + A.hash(c, o.seed + 1) * cell
        const y0 = ridge(u)
        const y = y0 + A.hash(c, o.seed + 2) * o.depth * 0.9
        const slope = ridge(u + 2) - ridge(u - 2)
        const angle = Math.atan2(4, slope) + (A.hash(c, o.seed + 3) - 0.5) * 0.4
        const len = 3 + A.hash(c, o.seed + 4) * o.strokeLen
        g.strokeStyle = A.rgba(o.color, o.alpha * (0.3 + A.hash(c, o.seed + 5) * 0.5))
        g.lineWidth = 0.5 + A.hash(c, o.seed + 6) * 1.1
        g.beginPath(); g.moveTo(u, y); g.lineTo(u + Math.cos(angle) * len * 0.5, y + Math.sin(angle) * len); g.stroke()
      }

      // Broken calligraphic outline
      for (let u = a; u < b; u += 5) {
        const v = n(u / 14 + 9)
        if (v < 0.32) continue
        g.strokeStyle = A.rgba(o.color, o.alpha * o.outline * (0.5 + v * 0.5))
        g.lineWidth = 0.5 + v * 1.8
        g.beginPath(); g.moveTo(u, ridge(u)); g.lineTo(u + 3, ridge(u + 3)); g.lineTo(u + 5.5, ridge(u + 5.5)); g.stroke()
      }

      // Distant trees dotted on ridges (点苔 — exact art.mountains formula)
      if (o.trees) {
        for (let c = Math.floor(a / 6); c * 6 < b; c++) {
          if (A.hash(c, o.seed + 40) > o.trees) continue
          const u = c * 6 + A.hash(c, o.seed + 41) * 6
          const y = ridge(u) + 1
          const h = 3 + A.hash(c, o.seed + 42) * o.treeH
          g.strokeStyle = A.rgba(o.color, Math.min(1, o.alpha * 1.4)); g.lineWidth = 0.55
          g.beginPath(); g.moveTo(u, y); g.lineTo(u, y - h); g.stroke()
          g.fillStyle = A.rgba(o.color, Math.min(1, o.alpha * 1.25))
          for (let k = 0; k < 3; k++) {
            g.beginPath(); g.ellipse(u + (A.hash(c * 3 + k, 9) - 0.5) * 3, y - h * (0.55 + k * 0.2), 1.5 + A.hash(c + k, 3) * 1.3, 0.8 + A.hash(c + k, 5) * 0.6, 0, 0, TAU); g.fill()
          }
        }
      }

      // Distant mountain pavilions perched on ridges
      if (o.pavilionWorlds) {
        for (const wxPos of o.pavilionWorlds) {
          const u = wxPos * f + halfView
          if (u < a - 40 || u > b + 40) continue
          const py = ridge(u) + 2, s = o.pavilionScale || 1
          const col = A.rgba(o.color, Math.min(1, o.alpha * 1.6))
          g.save(); g.translate(u, py); g.scale(s, s)
          g.fillStyle = col
          g.fillRect(-5, -8, 1.4, 8); g.fillRect(3.6, -8, 1.4, 8)
          g.beginPath(); g.moveTo(-11, -8); g.quadraticCurveTo(0, -14, 11, -8); g.lineTo(8, -7); g.lineTo(-8, -7); g.closePath(); g.fill()
          g.beginPath(); g.moveTo(-7, -12); g.quadraticCurveTo(0, -17, 7, -12); g.lineTo(5, -11); g.lineTo(-5, -11); g.closePath(); g.fill()
          g.restore()
        }
      }

      // Soft bottom fade into mist
      g.save(); g.globalCompositeOperation = 'destination-out'
      const mistOut = g.createLinearGradient(0, bottom - o.fadeH, 0, bottom)
      mistOut.addColorStop(0, 'rgba(0,0,0,0)'); mistOut.addColorStop(1, 'rgba(0,0,0,0.92)')
      g.fillStyle = mistOut; g.fillRect(a, bottom - o.fadeH, b - a, o.fadeH)
      g.restore()
    })
  }

  // ---------------------------------------------------------------- corridor, moon gate, belfry, platforms

  function corridorSprite(res, posts) {
    const x0 = posts[0] - 42, x1 = posts[posts.length - 1] + 42
    const w = x1 - x0 + 60, ox = 30 - x0
    const eaveY = 148
    return {
      x: x0 - 30,
      s: A.sprite(w, 210, res, (g) => {
        g.translate(ox, 0)
        g.strokeStyle = A.rgba(INK, 0.28); g.lineWidth = 1.5
        g.beginPath(); g.moveTo(posts[0], -34); g.lineTo(posts[posts.length - 1], -34); g.stroke()
        g.lineWidth = 0.9
        for (let x = posts[0] + 20; x < posts[posts.length - 1]; x += 24) {
          g.beginPath(); g.moveTo(x, -34); g.lineTo(x, -2); g.stroke()
        }
        const colH = G - eaveY
        for (const px of posts) {
          g.fillStyle = A.rgba(INK, 0.64); g.fillRect(px - 4.5, -colH, 9, colH)
          g.fillStyle = A.rgba(PAPER, 0.24); g.fillRect(px - 2, -colH + 8, 1.5, colH - 18)
          g.fillStyle = A.rgba(INK, 0.52); g.beginPath(); g.roundRect(px - 8, -10, 16, 10, 2); g.fill()
        }
        g.fillStyle = A.rgba(INK, 0.72); g.fillRect(x0 + 8, -colH, x1 - x0 - 16, 7)
        darkRoof(g, (x0 + x1) / 2, (x1 - x0) / 2 + 8, -colH - 1, 26, 0.78)
      }, 0, 206),
    }
  }

  function moonGateSprite(res) {
    return A.sprite(300, 150, res, (g) => {
      g.fillStyle = A.rgba(INK, 0.12); g.beginPath(); g.ellipse(0, 0, 134, 3.5, 0, 0, TAU); g.fill()
      g.save()
      g.beginPath()
      g.rect(-126, -112, 252, 112)
      g.arc(0, -48, 43, 0, TAU, true)
      const wall = g.createLinearGradient(0, -112, 0, 0)
      wall.addColorStop(0, A.rgba(INK, 0.32))
      wall.addColorStop(0.25, A.rgba(PAPER, 0.76))
      wall.addColorStop(1, A.rgba(INK, 0.22))
      g.fillStyle = wall; g.fill('evenodd')
      g.restore()
      g.strokeStyle = A.rgba(INK, 0.65); g.lineWidth = 2.4
      g.beginPath(); g.arc(0, -48, 43, 0, TAU); g.stroke()
      g.strokeStyle = A.rgba(INK, 0.3); g.lineWidth = 1
      g.beginPath(); g.arc(0, -48, 46.5, 0, TAU); g.stroke()
      darkRoof(g, 0, 128, -112, 13, 0.84)
    }, 150, 146)
  }

  function belfrySprite(res) {
    return A.sprite(360, 224, res, (g) => {
      const eave = -162
      g.fillStyle = A.rgba(INK, 0.16); g.beginPath(); g.ellipse(0, 0, 166, 4, 0, 0, TAU); g.fill()
      for (const px of [-136, -72, 72, 136]) {
        g.fillStyle = A.rgba(INK, 0.78); g.fillRect(px - 5, eave, 10, -eave)
        g.fillStyle = A.rgba(PAPER, 0.22); g.fillRect(px - 2.5, eave + 12, 1.5, -eave - 24)
        g.fillStyle = A.rgba(INK, 0.56); g.beginPath(); g.roundRect(px - 8.5, -12, 17, 12, 2); g.fill()
      }
      g.fillStyle = A.rgba(INK, 0.82)
      g.fillRect(-142, eave, 284, 7)
      g.fillRect(-78, -122, 156, 6)
      darkRoof(g, 0, 152, eave - 1, 32, 0.88)
    }, 180, 218)
  }

  // Platforms use Volume 1's exact art.rock & art.bridge painters!
  function platformSprite(p, res, seed) {
    if (p.material === 'stone' || p.material === 'ledge') return A.rock(p.w, p.y, G, seed, res)
    return A.bridge(p.w, p.y, G, res, seed)
  }

  // ---------------------------------------------------------------- build & render hooks

  function build(r, world, viewW = 720) {
    const halfView = viewW * 0.5
    const span = (f) => Math.ceil(world.length * f + viewW + 260)
    return {
      far: mountainScroll({
        width: span(0.16), top: 26, height: 240, base: 250, res: r(0.3), seed: 601, parallax: 0.16, halfView,
        scale: 1.0, spacing: 0.9, detail: 16, detailAmp: 9, color: [76, 90, 104], alpha: 0.31, depth: 70,
        strokeCell: 4, strokeDensity: 0.55, strokeLen: 9, outline: 0.55, trees: 0.14, treeH: 3, fadeH: 80,
        indigo: 0.16, celadon: 0.1,
      }),
      mid: mountainScroll({
        width: span(0.32), top: 84, height: 186, base: 265, res: r(0.4), seed: 619, parallax: 0.32, halfView,
        scale: 0.68, spacing: 1.05, detail: 22, detailAmp: 12, color: [52, 62, 70], alpha: 0.43, depth: 46,
        strokeCell: 4, strokeDensity: 0.6, strokeLen: 8, outline: 0.75, trees: 0.42, treeH: 6, fadeH: 60,
        celadon: 0.16, pavilionWorlds: [1080, 3520, 4480], pavilionScale: 1.05,
      }),
      near: mountainScroll({
        width: span(0.52), top: 166, height: 128, base: 289, res: r(0.5), seed: 637, parallax: 0.52, halfView,
        scale: 0.42, spacing: 1.2, detail: 26, detailAmp: 10, color: [38, 40, 44], alpha: 0.55, depth: 34,
        strokeCell: 3, strokeDensity: 0.65, strokeLen: 7, outline: 0.9, trees: 0.56, treeH: 9, fadeH: 40,
        celadon: 0.1, ochre: 0.12, pavilionWorlds: [1320, 3720, 4880], pavilionScale: 1.55,
      }),
      forestFar: mountainForest({ period: 1100, count: 22, height: [36, 70], base: 292, top: 205, res: r(0.45), seed: 643, color: [56, 64, 68], alpha: 0.28 }),
      forestNear: mountainForest({ period: 1450, count: 10, height: [76, 126], base: 302, top: 165, res: r(0.6), seed: 659, color: [28, 32, 36], alpha: 0.46 }),
      fg: A.bamboo({ period: 2100, count: 3, thick: [12, 18], lean: 0.03, alpha: [0.5, 0.62], top: -150, bottom: 420, res: r(0.45), seed: 677, color: [14, 14, 16], leaves: 0.52, leafLen: 40 }),
      arch: mountainGateSprite(r(1)),
      steleGate: mountainStele(r(1), ['问', '山'], 72, 611),
      steleSummit: mountainStele(r(1), ['止', '戈'], 82, 619),
      corridor: corridorSprite(r(1), world.wind.posts),
      moonGate: moonGateSprite(r(1)),
      belfry: belfrySprite(r(1)),
      pineGate: vigorousPine(r(0.95), 6011, { style: 'gate', scale: 0.78 }),
      pineCliff: vigorousPine(r(0.95), 6017, { style: 'gate', scale: 0.56, mirror: -1 }),
      treeWindL: vigorousDeadTree(r(0.95), 6031, { scale: 0.76, windDir: 1, lean: 0.36, sparseNeedles: true }),
      treeWindR: vigorousDeadTree(r(0.95), 6037, { scale: 0.88, windDir: 1, lean: 0.42, sparseNeedles: true }),
      pineCourt: vigorousPine(r(0.95), 6041, { style: 'court', scale: 0.96 }),
      treePlum: vigorousDeadTree(r(0.95), 6047, { scale: 0.76, windDir: -1, lean: -0.34, blossom: true }),
      pineBell: vigorousPine(r(0.95), 6053, { style: 'bell', scale: 0.86 }),
      pineSummit: vigorousPine(r(0.95), 6061, { style: 'summit', scale: 0.96 }),
      platforms: world.platforms.map((p, i) => ({ p, s: platformSprite(p, r(1), 60 + i) })),
    }
  }

  function drawMountains(ctx, a, cam, e, clock, layer, mistFn) {
    const m = a.mountain
    const cx = cam.left + cam.viewW * 0.5
    const summitDrop = smooth((cam.left - 4800) / 520) * 52
    // Cloud-sea density peaks in 悬栈 (1440..2330) and 绝顶 (5230..6120) so spires/crests emerge from a sea of clouds!
    const bridgeCloud = smooth((cx - 1250) / 280) * (1 - smooth((cx - 2180) / 280))
    const summitCloud = smooth((cx - 5050) / 360)
    const cloudSea = Math.max(bridgeCloud, summitCloud)

    const drawLayer = (strip, f, yShift) => {
      layer(ctx, cam, f)
      if (yShift) ctx.translate(0, yShift)
      A.drawStrip(ctx, strip, cam.left * f, cam.viewW)
    }
    drawLayer(m.far, 0.16, summitDrop * 0.95)
    mistFn(ctx, a, cam, 0.22, 236 + summitDrop * 0.35, 64 + cloudSea * 28, 0.7 + cloudSea * 0.22, clock * 5, 7)
    drawLayer(m.mid, 0.32, summitDrop * 0.72)
    mistFn(ctx, a, cam, 0.42, 262 + summitDrop * 0.2, 50 + cloudSea * 34, 0.6 + cloudSea * 0.28, clock * 8, 13)
    drawLayer(m.near, 0.52, summitDrop * 0.48)
    mistFn(ctx, a, cam, 0.64, 287, 38 + cloudSea * 26, 0.55 + cloudSea * 0.3, clock * 11, 19)

    // Mid-distance swaying mountain pines & bamboo (0.55 & 0.80 parallax — exact Vol 1 / Vol 2 depth formula!)
    const forest = e.forest ?? 0
    if (forest > 0.02) {
      const sway = Math.sin(clock * 0.6) * 0.008 + Math.sin(clock * 1.7) * 0.003
      layer(ctx, cam, 0.55, sway, 292)
      ctx.globalAlpha = forest
      A.drawStrip(ctx, m.forestFar, cam.left * 0.55 - 30, cam.viewW + 60, 1100)
      ctx.globalAlpha = 1
      mistFn(ctx, a, cam, 0.7, 295, 32, 0.5, clock * 14, 29)
      layer(ctx, cam, 0.8, sway * 1.5, 302)
      ctx.globalAlpha = forest * 0.92
      A.drawStrip(ctx, m.forestNear, cam.left * 0.8 - 40, cam.viewW + 80, 1450)
      ctx.globalAlpha = 1
    } else {
      mistFn(ctx, a, cam, 0.7, 295, 34, 0.55, clock * 14, 29)
    }

    // Summit ink wild geese exclusively at 绝顶.
    if (cam.left > 5050) {
      layer(ctx, cam, 0.08)
      const base = 5230 * 0.08 + cam.viewW * 0.42 - ((clock * 6) % 240) + 90
      ctx.strokeStyle = A.rgba(INK, 0.54); ctx.lineWidth = 1.3; ctx.lineCap = 'round'
      for (let i = 0; i < 6; i++) {
        const gx = base + i * 17
        const gy = 84 + Math.abs(i - 2) * 7 + Math.sin(clock * 2.6 + i) * 1.6
        const flap = Math.sin(clock * 5.2 + i * 1.1) * 2.5
        ctx.beginPath()
        ctx.moveTo(gx - 5.5, gy - flap); ctx.quadraticCurveTo(gx - 2, gy - 2.5, gx, gy)
        ctx.quadraticCurveTo(gx + 2, gy - 2.5, gx + 5.5, gy - flap)
        ctx.stroke()
      }
    }
  }

  function ground(ctx, a, cam, e, clock) {
    const m = a.mountain
    const left = cam.left - 60, right = cam.left + cam.viewW + 60

    ctx.save()
    ctx.beginPath()
    if (left < BRIDGE[0]) ctx.rect(left, 260, Math.min(right, BRIDGE[0]) - left, 220)
    if (right > BRIDGE[1]) ctx.rect(Math.max(left, BRIDGE[1]), 260, right - Math.max(left, BRIDGE[1]), 220)
    ctx.clip()
    A.drawStrip(ctx, a.ground, cam.left, cam.viewW, 1024)
    ctx.restore()

    // 悬栈 wooden plank bridge over cloud-sea mist (matches Vol 1 art.bridge style).
    if (right > BRIDGE[0] - 40 && left < BRIDGE[1] + 40) {
      const b0 = Math.max(left, BRIDGE[0]), b1 = Math.min(right, BRIDGE[1])
      const sea = ctx.createLinearGradient(0, G - 6, 0, G + 100)
      sea.addColorStop(0, A.rgba(PAPER, 0.14))
      sea.addColorStop(0.35, A.rgba(PAPER, 0.78))
      sea.addColorStop(1, A.rgba(PAPER, 0.96))
      ctx.fillStyle = sea; ctx.fillRect(b0, G - 6, b1 - b0, 120)
      ctx.save()
      ctx.beginPath(); ctx.rect(BRIDGE[0], G + 2, BRIDGE[1] - BRIDGE[0], 110); ctx.clip()
      for (let i = 0; i < 6; i++) {
        const span = BRIDGE[1] - BRIDGE[0] + 280
        const cx = BRIDGE[0] - 140 + ((i * 190 + clock * (8 + (i % 3) * 3)) % span)
        ctx.globalAlpha = 0.45 + (i % 2) * 0.16
        ctx.drawImage(a.mist, cx - 120, G + 10 + (i % 3) * 14, 240, 42)
      }
      ctx.restore(); ctx.globalAlpha = 1
      for (let px = Math.floor(BRIDGE[0] / 140) * 140; px <= BRIDGE[1]; px += 140) {
        if (px < left - 40 || px > right + 40) continue
        ctx.fillStyle = A.rgba(INK, 0.72); ctx.fillRect(px - 3.5, G + 2, 7, 92)
        ctx.fillStyle = A.rgba(INK, 0.65); ctx.fillRect(px - 1.5, G - 26, 3, 26)
      }
      ctx.strokeStyle = A.rgba(INK, 0.55); gLine(ctx, Math.max(BRIDGE[0], left), Math.min(BRIDGE[1], right), G - 18, 1.3)
      ctx.fillStyle = A.rgba(INK, 0.88); ctx.fillRect(BRIDGE[0], G - 1, BRIDGE[1] - BRIDGE[0], 7)
      ctx.fillStyle = A.rgba(PAPER, 0.28); ctx.fillRect(BRIDGE[0], G + 1, BRIDGE[1] - BRIDGE[0], 1.4)
    }

    const put = (s, x, y = G, alpha = 1) => {
      if (!s || x + s.width < left || x - s.width > right) return
      A.drawSprite(ctx, s, x, y, alpha)
    }
    // 岩关: stone lanterns, mountain pass gate with flanking cliff pines, and 问山 stele
    put(a.lantern, 680)
    put(m.pineGate, 774, G, 0.86)
    put(m.arch, 960, G, 0.96)
    put(a.lantern, 1245)
    put(m.pineCliff, 1372, G, 0.78)
    put(m.steleGate, 1335, G, 0.94)

    // 风廊: five-column wind corridor framed by wind-carved cliff cypresses
    put(m.treeWindL, 2425, G, 0.76)
    if (m.corridor.x + m.corridor.s.width > left && m.corridor.x < right) {
      A.drawSprite(ctx, m.corridor.s, m.corridor.x, G, 0.9)
    }
    put(m.treeWindR, 3032, G, 0.84)

    // 松庭: sculpted court pine, moon gate & gnarled cinnabar plum
    put(a.lantern, 3405)
    put(m.pineCourt, 3535, G, 0.84)
    put(m.treePlum, 3985, G, 0.84)
    put(m.moonGate, 3825, G, 0.92)

    // 钟楼: solemn night pine, stone lantern, belfry & warm hanging lanterns
    put(a.lantern, 4420)
    put(m.pineBell, 4505, G, 0.84)
    put(m.belfry, 4784, G, 0.92)
    if (right > 4560 && left < 5010) {
      for (const lx of [4648, 4920]) {
        ctx.fillStyle = A.rgba(INK, 0.78); ctx.fillRect(lx - 4, 152, 8, 12)
        ctx.fillStyle = 'rgba(232,174,88,0.85)'; ctx.fillRect(lx - 2.5, 154, 5, 8)
        ctx.globalAlpha = 0.25 + Math.sin(clock * 4 + lx) * 0.04
        ctx.drawImage(a.light, lx - 38, 122, 76, 76)
        ctx.globalAlpha = 1
      }
    }

    // 绝顶: summit welcoming pine (reaching cleanly beneath boss bar & sun) + 止戈 stele
    put(m.pineSummit, 5452, G, 0.78)
    put(m.steleSummit, 5980, G, 0.94)

    for (const entry of m.platforms) {
      if (entry.p.x + entry.p.w < left || entry.p.x > right) continue
      A.drawSprite(ctx, entry.s, entry.p.x, entry.p.y)
    }
  }

  function gLine(ctx, x0, x1, y, w) {
    ctx.lineWidth = w
    ctx.beginPath(); ctx.moveTo(x0, y); ctx.lineTo(x1, y); ctx.stroke()
  }

  function moving(ctx, a, cam, state, clock, reducedMotion) {
    const left = cam.left - 60, right = cam.left + cam.viewW + 60
    const wind = state.world.wind
    if (wind && right > wind.zone[0] - 80 && left < wind.zone[1] + 80) {
      const power = state.wind?.power || 0
      const dir = state.wind?.dir || 1
      const phase = state.wind?.phase || 'calm'
      for (const px of wind.posts) {
        if (px < left - 30 || px > right + 30) continue
        const sway = (phase === 'calm' ? Math.sin(clock * 2.2 + px) * 4 : dir * (12 + power * 24 + Math.sin(clock * 14 + px) * 3))
        const drop = phase === 'calm' ? 24 : 18 - power * 8
        ctx.fillStyle = phase === 'calm' ? A.rgba(INK, 0.55) : CINNABAR
        ctx.globalAlpha = phase === 'calm' ? 0.68 : 0.82
        ctx.beginPath()
        ctx.moveTo(px + 2, 155)
        ctx.lineTo(px + 2 + sway, 155 + drop * 0.55)
        ctx.lineTo(px + 2 + sway * 0.75, 155 + drop)
        ctx.lineTo(px - 2, 155 + 4)
        ctx.closePath(); ctx.fill()
      }
      ctx.globalAlpha = 1
      if (power > 0.06 && !reducedMotion) {
        ctx.strokeStyle = 'rgba(239,231,212,' + (0.2 + power * 0.32).toFixed(3) + ')'
        ctx.lineWidth = 1.1
        const speed = dir * (160 + power * 260)
        for (let i = 0; i < 12; i++) {
          const span = wind.zone[1] - wind.zone[0]
          const u = ((i * 97 + clock * speed) % span + span) % span
          const wx = wind.zone[0] + u
          if (wx < left || wx > right) continue
          const wy = 164 + (i * 43) % 126
          const len = 26 + power * 42
          ctx.beginPath(); ctx.moveTo(wx, wy); ctx.lineTo(wx - dir * len, wy); ctx.stroke()
        }
      }
    }

    const bell = state.bell
    if (bell && right > bell.x - 60 && left < bell.x + 60) {
      const since = Math.max(0, state.clock - bell.ringAt)
      const swing = since < 2.2 && !reducedMotion
        ? Math.sin(since * 11) * Math.exp(-since * 1.8) * 0.34 * (bell.dir || 1)
        : Math.sin(clock * 1.1) * 0.015
      ctx.save()
      ctx.translate(bell.x, 184)
      ctx.rotate(swing)
      ctx.strokeStyle = A.rgba(INK, 0.82); ctx.lineWidth = 2.6
      ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(0, 12); ctx.stroke()
      ctx.fillStyle = '#3b3528'
      ctx.beginPath()
      ctx.moveTo(-11, 14); ctx.quadraticCurveTo(-15, 28, -21, 44)
      ctx.lineTo(21, 44); ctx.quadraticCurveTo(15, 28, 11, 14)
      ctx.quadraticCurveTo(0, 10, -11, 14); ctx.closePath(); ctx.fill()
      ctx.strokeStyle = 'rgba(236,204,136,0.58)'; ctx.lineWidth = 1.2
      ctx.beginPath(); ctx.moveTo(-16, 36); ctx.lineTo(16, 36); ctx.moveTo(-13, 25); ctx.lineTo(13, 25); ctx.stroke()
      ctx.strokeStyle = 'rgba(236,204,136,0.34)'; ctx.lineWidth = 1
      ctx.beginPath(); ctx.moveTo(-10, 15); ctx.quadraticCurveTo(-14, 28, -19, 43); ctx.stroke()
      ctx.strokeStyle = A.rgba(INK, 0.88); ctx.lineWidth = 1.4
      ctx.beginPath(); ctx.moveTo(-22, 44); ctx.lineTo(22, 44); ctx.stroke()
      ctx.fillStyle = A.rgba(INK, 0.85)
      ctx.beginPath(); ctx.arc(0, 46, 3.2, 0, TAU); ctx.fill()
      ctx.restore()
    }
  }

  // Exact Vol 1 foreground formula: 1.30 parallax semi-transparent framing stalks + 1.18 parallax bottom grass!
  function foreground(ctx, a, cam, e, clock, layer) {
    const m = a.mountain
    const sway = Math.sin(clock * 0.9 + 2) * 0.012
    const forest = e.forest ?? 0
    if (m.fg && forest > 0.05) {
      layer(ctx, cam, 1.3, sway, 400)
      ctx.globalAlpha = 0.25 + forest * 0.55
      A.drawStrip(ctx, m.fg, cam.left * 1.3 - 40, cam.viewW + 80, 2100)
      ctx.globalAlpha = 1
    }
    if (!a.grass) return
    layer(ctx, cam, 1.18, sway * 1.6, 394)
    ctx.save()
    const gLeft = cam.left * 1.18 - 40, gRight = gLeft + cam.viewW + 80
    const b0 = BRIDGE[0] * 1.18, b1 = BRIDGE[1] * 1.18
    ctx.beginPath()
    if (gLeft < b0) ctx.rect(gLeft, 260, Math.min(gRight, b0) - gLeft, 200)
    if (gRight > b1) ctx.rect(Math.max(gLeft, b1), 260, gRight - Math.max(gLeft, b1), 200)
    ctx.clip()
    ctx.globalAlpha = 0.82
    A.drawStrip(ctx, a.grass, gLeft, cam.viewW + 80, 900)
    ctx.restore()
    ctx.globalAlpha = 1
  }

  function attackGuides(ctx, state) {
    for (const enemy of state.enemies) {
      if (!enemy.alive || enemy.delay > 0) continue
      const progress = Math.max(0, 1 - enemy.t / (enemy.teleTotal || 1))
      if (enemy.state === 'mountTell' && enemy.sweep) {
        const hit = R.summitReach(enemy)
        const y = hit.y1 - 5
        const grad = ctx.createLinearGradient(0, y - 16, 0, y + 2)
        grad.addColorStop(0, 'rgba(184,50,42,0)')
        grad.addColorStop(1, 'rgba(184,50,42,' + (0.14 + progress * 0.34).toFixed(3) + ')')
        ctx.fillStyle = grad
        ctx.fillRect(hit.x0, y - 16, hit.x1 - hit.x0, 18)
        const from = enemy.facing > 0 ? hit.x0 : hit.x1
        const to = enemy.facing > 0 ? hit.x1 : hit.x0
        ctx.strokeStyle = CINNABAR; ctx.lineWidth = 2.1
        ctx.beginPath(); ctx.moveTo(from, y); ctx.lineTo(A.lerp(from, to, progress), y); ctx.stroke()
      } else if (enemy.state === 'bowTell' && enemy.boltAim) {
        const shot = R.arrowLaunch(enemy, enemy.boltAim)
        ctx.save()
        ctx.setLineDash([3, 8])
        ctx.strokeStyle = 'rgba(184,50,42,' + (0.16 + progress * 0.42).toFixed(3) + ')'
        ctx.lineWidth = 1.2
        ctx.beginPath()
        for (let i = 0; i < 8; i++) {
          const t = shot.flight * i / 7
          const px = shot.x + shot.vx * t
          const py = shot.y + shot.vy * t + R.ARROW_GRAVITY * t * t / 2
          if (i === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py)
        }
        ctx.stroke()
        ctx.restore()
      }
    }
  }

  globalThis.XiabanInkMountain = { build, drawMountains, ground, moving, foreground, attackGuides }
})()
