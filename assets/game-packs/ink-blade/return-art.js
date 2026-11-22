/* 水墨剑客 · 卷七《归墨卷》美术：浅绛逐幅褪回水墨，至留白与终幅白纸；残烽、断关、荒驿、竹海、石灯、山门石阶、墨渍重聚与留白一笔。 */
(() => {
  'use strict'

  const art = globalThis.XiabanInkArt
  const rules = globalThis.XiabanInkBlade
  const GROUND = rules.GROUND
  const PAPER = '#efe7d4'
  const INK = '#18181b'
  const CINNABAR = '#b8322a'
  const TAU = Math.PI * 2
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v))
  const smooth = (t) => { const v = clamp(t, 0, 1); return v * v * (3 - 2 * v) }
  const rgb = (c) => 'rgb(' + Math.round(c[0]) + ',' + Math.round(c[1]) + ',' + Math.round(c[2]) + ')'

  function sprite(w, h, ox, oy, res, draw) {
    const node = art.canvas(Math.ceil(w * res), Math.ceil(h * res))
    const g = node.getContext('2d')
    g.scale(res, res)
    g.translate(ox, oy)
    g.lineCap = 'round'; g.lineJoin = 'round'
    draw(g)
    return { node, width: w, height: h, ox, oy }
  }

  function fallenBanner(res) {
    return sprite(170, 96, 26, 86, res, (g) => {
      // Slanted snapped banner pole buried in dust
      g.strokeStyle = 'rgba(28,26,24,0.88)'; g.lineWidth = 3.2
      g.beginPath(); g.moveTo(0, 2); g.lineTo(116, -42); g.stroke()
      // Tattered banner cloth trailing down to the ground
      g.fillStyle = 'rgba(42,36,32,0.72)'
      g.beginPath()
      g.moveTo(56, -20); g.lineTo(112, -40); g.quadraticCurveTo(124, -18, 134, -2)
      g.lineTo(114, -8); g.lineTo(104, 1); g.lineTo(88, -6); g.lineTo(68, 2); g.closePath(); g.fill()
      // Faint faded cinnabar trim along top edge
      g.strokeStyle = 'rgba(156,64,48,0.45)'; g.lineWidth = 1.4
      g.beginPath(); g.moveTo(58, -19); g.lineTo(110, -39); g.stroke()
    })
  }

  function brokenHalberd(res) {
    return sprite(64, 96, 32, 86, res, (g) => {
      g.strokeStyle = 'rgba(24,24,26,0.9)'; g.lineWidth = 2.6
      g.beginPath(); g.moveTo(-4, 2); g.lineTo(8, -72); g.stroke()
      // Crescent side blade
      g.fillStyle = 'rgba(30,30,34,0.85)'
      g.beginPath(); g.moveTo(6, -64); g.quadraticCurveTo(22, -60, 18, -42); g.quadraticCurveTo(11, -50, 5, -52); g.closePath(); g.fill()
      // Spear tip
      g.beginPath(); g.moveTo(8, -72); g.lineTo(11, -84); g.lineTo(5, -72); g.closePath(); g.fill()
      // Faded tassel
      g.strokeStyle = 'rgba(162,56,42,0.48)'; g.lineWidth = 1.3
      g.beginPath(); g.moveTo(6, -54); g.quadraticCurveTo(-4, -44, -8, -32); g.stroke()
    })
  }

  function buriedDrum(res) {
    return sprite(100, 62, 50, 54, res, (g) => {
      // Half-buried war drum leaning into earth
      g.fillStyle = 'rgba(38,34,30,0.86)'
      g.beginPath(); g.ellipse(0, -14, 28, 20, -0.22, Math.PI * 0.9, Math.PI * 2.15); g.lineTo(24, 2); g.lineTo(-28, 2); g.closePath(); g.fill()
      g.fillStyle = 'rgba(222,212,192,0.85)'
      g.beginPath(); g.ellipse(-10, -16, 13, 18, -0.22, 0, TAU); g.fill()
      g.strokeStyle = 'rgba(24,24,26,0.85)'; g.lineWidth = 1.6; g.stroke()
      // Crack across drumhead
      g.strokeStyle = 'rgba(24,24,26,0.75)'; g.lineWidth = 1.1
      g.beginPath(); g.moveTo(-14, -28); g.lineTo(-9, -15); g.lineTo(-13, -4); g.stroke()
    })
  }

  function ruinedGate(res) {
    return sprite(240, 180, 120, 170, res, (g) => {
      // Half-collapsed pass archway in desaturated ink-ochre
      g.fillStyle = 'rgba(54,50,46,0.78)'
      g.beginPath()
      g.moveTo(-104, 2); g.lineTo(-98, -118); g.lineTo(-42, -124); g.lineTo(-22, -96)
      g.lineTo(-26, -62); g.quadraticCurveTo(-36, -64, -36, -2); g.closePath(); g.fill()
      g.beginPath()
      g.moveTo(28, 2); g.quadraticCurveTo(28, -62, 16, -68); g.lineTo(22, -94)
      g.lineTo(66, -88); g.lineTo(92, -62); g.lineTo(100, 2); g.closePath(); g.fill()
      // Broken lintel & rubble
      g.strokeStyle = 'rgba(26,24,24,0.65)'; g.lineWidth = 1.5
      g.beginPath(); g.moveTo(-98, -118); g.lineTo(-42, -124); g.lineTo(-22, -96); g.stroke()
      g.fillStyle = 'rgba(38,36,34,0.55)'
      for (const [rx, ry, rw, rh] of [[-12, -3, 18, 7], [6, -2, 14, 6], [42, -90, 24, 6]]) {
        g.beginPath(); g.ellipse(rx, ry, rw, rh, 0.1, 0, TAU); g.fill()
      }
    })
  }

  function postPavilion(res) {
    return sprite(180, 150, 90, 140, res, (g) => {
      // Empty frontier waystation pavilion
      g.strokeStyle = 'rgba(26,26,28,0.82)'; g.lineWidth = 3.2
      for (const px of [-48, -18, 18, 48]) { g.beginPath(); g.moveTo(px, 2); g.lineTo(px, -74); g.stroke() }
      // Curved tiled roof
      g.fillStyle = 'rgba(28,28,32,0.86)'
      g.beginPath()
      g.moveTo(-76, -68); g.quadraticCurveTo(-24, -84, 0, -112); g.quadraticCurveTo(24, -84, 76, -68)
      g.quadraticCurveTo(24, -72, 0, -76); g.quadraticCurveTo(-24, -72, -76, -68); g.closePath(); g.fill()
      // Stone bench inside
      g.fillStyle = 'rgba(54,52,50,0.65)'
      g.fillRect(-30, -14, 60, 5); g.fillRect(-24, -9, 6, 11); g.fillRect(18, -9, 6, 11)
    })
  }

  function hitchingPost(res) {
    return sprite(48, 76, 24, 68, res, (g) => {
      g.fillStyle = 'rgba(34,34,36,0.85)'
      g.beginPath(); g.moveTo(-6, 2); g.lineTo(-5, -52); g.quadraticCurveTo(0, -58, 5, -52); g.lineTo(6, 2); g.closePath(); g.fill()
      // Frayed rope hanging from ring
      g.strokeStyle = 'rgba(46,44,42,0.65)'; g.lineWidth = 1.4
      g.beginPath(); g.arc(0, -38, 4, 0, TAU); g.stroke()
      g.beginPath(); g.moveTo(2, -34); g.quadraticCurveTo(10, -22, 6, -6); g.stroke()
    })
  }

  function gateSteps(res) {
    return sprite(320, 48, 160, 36, res, (g) => {
      // Wide low stone steps leading up to the mountain gate
      for (let i = 0; i < 3; i++) {
        const w = 136 - i * 16, y = -i * 5
        g.fillStyle = 'rgba(42,42,46,' + (0.28 + i * 0.08) + ')'
        g.fillRect(-w, y - 5, w * 2, 5)
        g.strokeStyle = 'rgba(24,24,26,0.45)'; g.lineWidth = 0.9
        g.beginPath(); g.moveTo(-w, y - 5); g.lineTo(w, y - 5); g.stroke()
      }
    })
  }

  const PROPS_RETURN = [
    // 残烽
    { s: 'buriedDrum', x: 340, alpha: 0.9 },
    { s: 'fallenBanner', x: 560, alpha: 0.92 },
    { s: 'brokenHalberd', x: 820, alpha: 0.9 },
    { s: 'brokenHalberd', x: 1180, alpha: 0.82 },
    // 断关
    { s: 'ruinedGate', x: 1750, alpha: 0.84 },
    // 荒驿
    { s: 'hitchingPost', x: 2450, alpha: 0.88 },
    { s: 'postPavilion', x: 2770, alpha: 0.82 },
    { s: 'beacon', x: 3120, alpha: 0.55 },
    // 竹海 (back of Vol 1's 竹林 gate!)
    { s: 'gate', x: 3440, alpha: 0.88, flipX: true },
    { s: 'stele', x: 3960, alpha: 0.88 },
    // 留白 (3 stone lanterns matching whiteout lights)
    { s: 'lantern', x: 4440, alpha: 0.92 },
    { s: 'lantern', x: 4810, alpha: 0.92 },
    { s: 'lantern', x: 5020, alpha: 0.92 },
    // 无名 (Vol 1 mountain gate steps & pine, which fade out in stage 3!)
    { s: 'pine', x: 5540, alpha: 0.55, bossFade: true },
    { s: 'gateSteps', x: 5840, alpha: 0.9, bossFade: true },
  ]

  function fadeAt(x) {
    const stops = [[0, 0.35], [1300, 0.22], [2300, 0.10], [3300, 0]]
    if (x <= stops[0][0]) return stops[0][1]
    for (let i = 0; i < stops.length - 1; i++) {
      const [x0, v0] = stops[i], [x1, v1] = stops[i + 1]
      if (x <= x1) return v0 + (v1 - v0) * smooth((x - x0) / (x1 - x0))
    }
    return 0
  }

  function build(r, span, world) {
    const a = {}
    // Pure ink mountain backbone (shared visual language with Vol 1)
    a.far = art.mountains({ width: span(0.06), top: 30, height: 236, base: 248, res: r(0.3), seed: 11, peakW: [26, 64], peakH: [85, 185], gap: [0.45, 1.05], round: 1.25, detail: 16, detailAmp: 9, color: [84, 88, 94], alpha: 0.3, depth: 70, strokeCell: 4, strokeDensity: 0.52, strokeLen: 9, outline: 0.52, trees: 0.08, treeH: 3, fadeH: 80 })
    a.mid = art.mountains({ width: span(0.16), top: 90, height: 182, base: 264, res: r(0.4), seed: 23, peakW: [60, 150], peakH: [45, 120], gap: [0.55, 1.1], round: 1.6, detail: 22, detailAmp: 12, color: [58, 60, 64], alpha: 0.42, depth: 46, strokeCell: 4, strokeDensity: 0.58, strokeLen: 8, outline: 0.72, trees: 0.28, treeH: 6, fadeH: 60, pavilions: [520, 1180] })
    a.near = art.mountains({ width: span(0.32), top: 170, height: 126, base: 289, res: r(0.5), seed: 37, peakW: [110, 260], peakH: [24, 74], gap: [0.6, 1], round: 2, detail: 26, detailAmp: 10, color: [38, 38, 40], alpha: 0.54, depth: 34, strokeCell: 3, strokeDensity: 0.62, strokeLen: 7, outline: 0.88, trees: 0.38, treeH: 8, fadeH: 40 })
    // Faint 浅绛 frontier overlay (drawn at alpha proportional to fadeAt(x), disappears by 竹海!)
    a.farWash = art.mountains({ width: span(0.06), top: 30, height: 236, base: 250, res: r(0.28), seed: 201, peakW: [40, 110], peakH: [60, 150], gap: [0.5, 1.1], round: 1.6, detail: 18, detailAmp: 9, color: [86, 92, 104], alpha: 0.28, depth: 70, strokeCell: 4, strokeDensity: 0.42, strokeLen: 9, outline: 0.45, trees: 0, treeH: 3, fadeH: 80, indigo: 0.22 })
    a.midWash = art.mountains({ width: span(0.16), top: 90, height: 182, base: 266, res: r(0.38), seed: 211, peakW: [90, 200], peakH: [40, 110], gap: [0.5, 1], round: 2.2, detail: 22, detailAmp: 12, color: [72, 66, 60], alpha: 0.36, depth: 46, strokeCell: 4, strokeDensity: 0.5, strokeLen: 8, outline: 0.65, trees: 0.05, treeH: 4, fadeH: 60, ochre: 0.16, indigo: 0.1, wall: { from: 0, to: span(0.16) * 0.55, alpha: 0.42, towerEvery: 260 } })
    a.nearWash = art.mountains({ width: span(0.32), top: 170, height: 126, base: 290, res: r(0.45), seed: 223, peakW: [140, 300], peakH: [18, 60], gap: [0.6, 1], round: 2.4, detail: 26, detailAmp: 8, color: [54, 46, 38], alpha: 0.46, depth: 34, strokeCell: 3, strokeDensity: 0.5, strokeLen: 7, outline: 0.8, trees: 0.1, treeH: 6, ochre: 0.3, fadeH: 40 })
    // Bamboo groves (reused from Vol 1 for 竹海)
    a.bambooFar = art.bamboo({ period: 1100, count: 26, thick: [3, 6], lean: 0.05, alpha: [0.08, 0.2], top: -120, bottom: 292, res: r(0.45), seed: 51, color: [60, 64, 68], leaves: 0.3, leafLen: 16 })
    a.bambooNear = art.bamboo({ period: 1400, count: 9, thick: [6, 10], lean: 0.04, alpha: [0.24, 0.42], top: -120, bottom: 302, res: r(0.6), seed: 67, color: art.INK, leaves: 0.42, leafLen: 22 })
    a.fg = art.bamboo({ period: 2000, count: 3, thick: [13, 19], lean: 0.03, alpha: [0.48, 0.6], top: -150, bottom: 420, res: r(0.45), seed: 83, color: [14, 14, 16], leaves: 0.55, leafLen: 42 })
    // Grounds: pure ink ground + desert overlay that fades out
    a.ground = art.ground({ period: 1024, line: GROUND, depth: 92, res: r(1), seed: 5 })
    a.groundWash = art.desertGround({ period: 1024, line: GROUND, depth: 92, res: r(0.9), seed: 17 })
    a.dust = art.dustSprite(64)
    a.pine = art.pine(r(0.9), 21)
    a.beacon = art.beaconTower(r(0.9))
    a.ramparts = (world.ramparts || []).map((wall, i) => ({ wall, s: art.rampart(r(1), 7 + i, wall.h) }))
    // Custom Volume 7 landmark sprites
    a.fallenBanner = fallenBanner(r(1))
    a.brokenHalberd = brokenHalberd(r(1))
    a.buriedDrum = buriedDrum(r(1))
    a.ruinedGate = ruinedGate(r(0.95))
    a.postPavilion = postPavilion(r(0.95))
    a.hitchingPost = hitchingPost(r(1))
    a.gateSteps = gateSteps(r(1))
    return a
  }

  function background(ctx, a, cam, e, clock, layer, mist) {
    const center = cam.left + cam.viewW / 2
    const fade = e.fade ?? fadeAt(center)
    const wash = clamp(fade / 0.35, 0, 1)
    const blank = clamp(e.blank || 0, 0, 1)
    const inkVis = 1 - blank

    ctx.setTransform(1, 0, 0, 1, 0, 0)
    const sky = ctx.createLinearGradient(0, 0, 0, cam.height)
    sky.addColorStop(0, rgb(e.top)); sky.addColorStop(0.72, rgb(e.low)); sky.addColorStop(1, rgb(e.low))
    ctx.fillStyle = sky; ctx.fillRect(0, 0, cam.width, cam.height)

    const P = cam.P
    if (e.sun > 0.01) {
      const size = (96 + blank * 24) * P
      const x = (cam.viewW * (0.72 - blank * 0.22) - cam.left * 0.025 * (1 - blank)) * P
      const y = (74 + (1 - blank) * (cam.left / a.world.length * 46) - cam.top) * P
      ctx.globalAlpha = e.sun * 0.92
      ctx.drawImage(a.sun, x - size / 2, y - size / 2, size, size)
    }
    if (e.moon > 0.01 && inkVis > 0.02) {
      const size = 150 * P
      const x = (cam.viewW * 0.67 - (cam.left - 5300) * 0.05) * P
      const y = (82 - cam.top) * P
      ctx.globalAlpha = e.moon * inkVis
      ctx.drawImage(a.moon, x - size / 2, y - size / 2, size, size)
    }
    ctx.globalAlpha = 1

    if (inkVis > 0.02) {
      layer(ctx, cam, 0.06)
      ctx.globalAlpha = inkVis * (1 - wash * 0.55)
      art.drawStrip(ctx, a.far, cam.left * 0.06, cam.viewW)
      if (wash > 0.02) { ctx.globalAlpha = inkVis * wash; art.drawStrip(ctx, a.farWash, cam.left * 0.06, cam.viewW) }
      ctx.globalAlpha = 1
      mist(ctx, a, cam, 0.1, 236, 64, 0.68 * inkVis, clock * 5, 7)

      layer(ctx, cam, 0.16)
      ctx.globalAlpha = inkVis * (1 - wash * 0.55)
      art.drawStrip(ctx, a.mid, cam.left * 0.16, cam.viewW)
      if (wash > 0.02) { ctx.globalAlpha = inkVis * wash; art.drawStrip(ctx, a.midWash, cam.left * 0.16, cam.viewW) }
      ctx.globalAlpha = 1
      mist(ctx, a, cam, 0.24, 262, 50, 0.58 * inkVis, clock * 8, 13)

      layer(ctx, cam, 0.32)
      ctx.globalAlpha = inkVis * (1 - wash * 0.55)
      art.drawStrip(ctx, a.near, cam.left * 0.32, cam.viewW)
      if (wash > 0.02) { ctx.globalAlpha = inkVis * wash; art.drawStrip(ctx, a.nearWash, cam.left * 0.32, cam.viewW) }
      ctx.globalAlpha = 1
      mist(ctx, a, cam, 0.45, 287, 38, 0.52 * inkVis, clock * 11, 19)

      if (e.bamboo > 0.02) {
        const sway = Math.sin(clock * 0.6) * 0.01 + Math.sin(clock * 1.7) * 0.004
        layer(ctx, cam, 0.55, sway, 292)
        ctx.globalAlpha = e.bamboo * inkVis
        art.drawStrip(ctx, a.bambooFar, cam.left * 0.55 - 30, cam.viewW + 60, 1100)
        ctx.globalAlpha = 1
        mist(ctx, a, cam, 0.7, 295, 30, 0.48 * inkVis, clock * 14, 29)
        layer(ctx, cam, 0.8, sway * 1.3, 302)
        ctx.globalAlpha = e.bamboo * 0.92 * inkVis
        art.drawStrip(ctx, a.bambooNear, cam.left * 0.8 - 40, cam.viewW + 80, 1400)
        ctx.globalAlpha = 1
      }
    }

    // Slow rising embers & grey ash in 残烽
    if ((e.embers || 0) > 0.05) {
      layer(ctx, cam, 0.85)
      const left = cam.left * 0.85
      for (let i = 0; i < 18; i++) {
        const q = (clock * 0.14 + art.hash(i, 71)) % 1
        const ex = left + art.hash(i, 73) * cam.viewW + Math.sin(clock * 1.2 + i) * 14
        const ey = GROUND - q * 240
        const red = i % 4 === 0
        ctx.globalAlpha = (e.embers || 0) * Math.sin(q * Math.PI) * (red ? 0.75 : 0.35)
        ctx.fillStyle = red ? CINNABAR : '#4a4844'
        ctx.beginPath(); ctx.arc(ex, ey, red ? 1.5 : 1.8, 0, TAU); ctx.fill()
      }
      ctx.globalAlpha = 1
    }
  }

  function ground(ctx, a, cam, e) {
    const left = cam.left - 60, right = cam.left + cam.viewW + 60
    const center = cam.left + cam.viewW / 2
    const fade = e.fade ?? fadeAt(center)
    const wash = clamp(fade / 0.35, 0, 1)
    const blank = clamp(e.blank || 0, 0, 1)

    if (blank < 0.98) {
      ctx.globalAlpha = 1 - blank
      art.drawStrip(ctx, a.ground, cam.left, cam.viewW, 1024)
      if (wash > 0.02 && a.groundWash) {
        ctx.globalAlpha = (1 - blank) * wash
        art.drawStrip(ctx, a.groundWash, cam.left, cam.viewW, 1024)
      }
      ctx.globalAlpha = 1
    }
    if (blank > 0.02) {
      // Stage 3 pure ink ground line on blank paper.
      ctx.save()
      ctx.globalAlpha = blank
      ctx.strokeStyle = INK; ctx.lineWidth = 1.8
      ctx.beginPath(); ctx.moveTo(left, GROUND + 0.5); ctx.lineTo(right, GROUND + 0.5); ctx.stroke()
      ctx.strokeStyle = 'rgba(24,24,26,0.22)'; ctx.lineWidth = 4
      ctx.beginPath(); ctx.moveTo(left, GROUND + 2.5); ctx.lineTo(right, GROUND + 2.5); ctx.stroke()
      ctx.restore()
    }

    for (const prop of PROPS_RETURN) {
      const s = a[prop.s]
      if (!s || prop.x + s.width < left || prop.x - s.width > right) continue
      const alpha = (prop.alpha ?? 1) * (prop.bossFade ? (1 - blank) : 1)
      if (alpha <= 0.02) continue
      if (prop.flipX) {
        ctx.save()
        ctx.translate(prop.x, GROUND); ctx.scale(-1, 1)
        ctx.globalAlpha = alpha
        ctx.drawImage(s.node, -s.ox, -s.oy, s.width, s.height)
        ctx.restore()
      } else {
        art.drawSprite(ctx, s, prop.x, GROUND, alpha)
      }
    }
    for (const entry of a.ramparts || []) {
      if (entry.wall.x + 40 < left || entry.wall.x - 40 > right) continue
      art.drawSprite(ctx, entry.s, entry.wall.x, GROUND)
    }
  }

  function foreground(ctx, a, cam, e, clock, layer) {
    const blank = clamp(e.blank || 0, 0, 1)
    if (blank > 0.9 || (e.bamboo || 0) < 0.05) return
    const sway = Math.sin(clock * 0.9 + 2) * 0.012
    layer(ctx, cam, 1.3, sway, 400)
    ctx.globalAlpha = e.bamboo * 0.55 * (1 - blank)
    art.drawStrip(ctx, a.fg, cam.left * 1.3 - 40, cam.viewW + 80, 2000)
    ctx.globalAlpha = 1
  }

  /** 不斩不散: pooled ink stains and gathering 「聚」 indicator. */
  function drawStains(ctx, state, clock) {
    if (!state.stains?.length) return
    ctx.save()
    for (const stain of state.stains) {
      if (stain.dead) continue
      const y = stain.y
      if (stain.phase === 'pool') {
        const pulse = 1 + Math.sin(clock * 4 + stain.id) * 0.06
        const near = stain.t > stain.wait - 0.7 ? (stain.t - (stain.wait - 0.7)) / 0.7 : 0
        ctx.fillStyle = 'rgba(18,18,22,' + (0.62 + near * 0.25).toFixed(3) + ')'
        ctx.beginPath(); ctx.ellipse(stain.x, y + 1, 22 * pulse, 4.5, 0, 0, TAU); ctx.fill()
        ctx.strokeStyle = 'rgba(20,20,24,' + (0.28 + near * 0.35).toFixed(3) + ')'
        ctx.lineWidth = 1.2
        ctx.beginPath(); ctx.ellipse(stain.x, y + 1, (26 + near * 8) * pulse, 6.2, 0, 0, TAU); ctx.stroke()
      } else if (stain.phase === 'gather') {
        const q = clamp(stain.t / stain.gather, 0, 1)
        const r = 34 * (1 - q * 0.65)
        ctx.fillStyle = 'rgba(18,18,22,0.82)'
        ctx.beginPath(); ctx.ellipse(stain.x, y + 1, 22 * (1 - q * 0.35), 4.8, 0, 0, TAU); ctx.fill()
        // Converging ink droplets rising into silhouette form
        for (let i = 0; i < 10; i++) {
          const a = (i / 10) * TAU + clock * 3
          const px = stain.x + Math.cos(a) * r
          const py = y - 18 * q + Math.sin(a) * r * 0.45
          ctx.beginPath(); ctx.arc(px, py, 1.8 + (i % 3) * 0.7, 0, TAU); ctx.fill()
        }
        // Gray 「聚」 seal overhead
        const size = 16 + Math.sin(clock * 14) * 1.4
        ctx.save()
        ctx.translate(stain.x, y - 34); ctx.rotate(-0.06)
        ctx.fillStyle = '#4a4843'
        ctx.beginPath(); ctx.roundRect(-size / 2, -size / 2, size, size, size * 0.14); ctx.fill()
        ctx.fillStyle = '#f4eddc'
        ctx.font = '600 ' + Math.round(size * 0.64) + 'px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
        ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
        ctx.fillText('聚', 0, size * 0.04)
        ctx.restore()
      }
    }
    ctx.restore()
  }

  /** 留白一笔: Nameless's paper-white horizontal sweep across the lower 70 units. */
  function drawBlankStroke(ctx, state, clock) {
    const st = state.blankStroke
    if (!st) return
    ctx.save()
    const w = st.x1 - st.x0
    if (st.phase === 'tell') {
      const q = clamp(st.t / st.tele, 0, 1)
      const pulse = 0.5 + 0.5 * Math.sin(clock * 18)
      ctx.fillStyle = 'rgba(244,238,224,' + (0.28 + q * 0.42).toFixed(3) + ')'
      ctx.fillRect(st.x0, GROUND - 70, w, 70)
      ctx.strokeStyle = 'rgba(184,50,42,' + (0.4 + q * 0.55).toFixed(3) + ')'
      ctx.lineWidth = 1.6
      ctx.setLineDash([10, 6])
      ctx.beginPath(); ctx.moveTo(st.x0, GROUND - 70); ctx.lineTo(st.x1, GROUND - 70); ctx.stroke()
      ctx.setLineDash([])
      // Warning seal 「危」
      const cx = (st.x0 + st.x1) / 2, size = 20 + pulse * 3
      ctx.translate(cx, GROUND - 96); ctx.rotate(-0.08)
      ctx.fillStyle = CINNABAR
      ctx.beginPath(); ctx.roundRect(-size / 2, -size / 2, size, size, size * 0.14); ctx.fill()
      ctx.fillStyle = '#fbefe2'
      ctx.font = '600 ' + Math.round(size * 0.66) + 'px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
      ctx.fillText('危', 0, size * 0.04)
    } else if (st.phase === 'active') {
      const q = clamp(st.t / st.dur, 0, 1)
      const reach = Math.min(1, q * 2.2)
      const fade = q < 0.65 ? 1 : Math.max(0, 1 - (q - 0.65) / 0.35)
      const xStart = st.dir > 0 ? st.x0 : st.x1
      const xEnd = xStart + st.dir * w * reach
      const left = Math.min(xStart, xEnd), spanW = Math.abs(xEnd - xStart)
      ctx.globalAlpha = fade
      // Pressed paper-white calligraphy body with dark dry-brush outline
      ctx.fillStyle = '#f6f0e2'
      ctx.fillRect(left, GROUND - 70, spanW, 70)
      ctx.strokeStyle = 'rgba(24,24,26,0.65)'; ctx.lineWidth = 2.2
      ctx.beginPath(); ctx.moveTo(left, GROUND - 70); ctx.lineTo(left + spanW, GROUND - 70); ctx.stroke()
      ctx.strokeStyle = 'rgba(24,24,26,0.35)'; ctx.lineWidth = 1
      for (let i = 0; i < 6; i++) {
        const y = GROUND - 62 + i * 11
        ctx.beginPath(); ctx.moveTo(left + (i % 2) * 18, y); ctx.lineTo(left + spanW * (0.72 + (i % 3) * 0.1), y); ctx.stroke()
      }
    }
    ctx.restore()
  }

  /** 留白: paper-white fog mask — inverse of Vol 3's night darkness. */
  function whiteoutAt(world, x) {
    const w = world?.whiteout
    if (!w) return 0
    const [z0, z1] = w.zone
    return clamp(Math.min((x - z0 + 120) / 240, (z1 + 120 - x) / 240), 0, 1)
  }

  function drawWhiteout(ctx, cam, state, clock, maskCanvas, cameraX) {
    const d = whiteoutAt(state.world, cameraX)
    if (d < 0.02) return { active: false, canvas: maskCanvas }
    const scale = 0.25
    const w = Math.max(1, Math.round(cam.width * scale)), h = Math.max(1, Math.round(cam.height * scale))
    let node = maskCanvas
    if (!node || node.width !== w || node.height !== h) {
      node = document.createElement('canvas'); node.width = w; node.height = h
    }
    const g = node.getContext('2d')
    const k2 = cam.P * scale
    g.globalCompositeOperation = 'source-over'
    g.clearRect(0, 0, w, h)
    g.fillStyle = 'rgba(241,235,221,' + (0.93 * d).toFixed(3) + ')'
    g.fillRect(0, 0, w, h)
    g.globalCompositeOperation = 'destination-out'
    const hole = (x, y, r, power = 1) => {
      const sx = (x - cam.left) * k2, sy = (y - cam.top) * k2, sr = r * k2
      if (sx + sr < 0 || sx - sr > w) return
      const grad = g.createRadialGradient(sx, sy, 0, sx, sy, sr)
      grad.addColorStop(0, 'rgba(0,0,0,' + power + ')')
      grad.addColorStop(0.55, 'rgba(0,0,0,' + power * 0.8 + ')')
      grad.addColorStop(1, 'rgba(0,0,0,0)')
      g.fillStyle = grad
      g.fillRect(sx - sr, sy - sr, sr * 2, sr * 2)
    }
    const p = state.player
    hole(p.x, p.y - 30, state.world.whiteout.radius)
    for (const x of state.world.whiteout.lights || []) {
      hole(x, GROUND - 26, 118 + 6 * Math.sin(clock * 6 + x), 0.92)
    }
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    ctx.drawImage(node, 0, 0, cam.width, cam.height)
    return { active: true, canvas: node }
  }

  /** Opening & closing calligraphy inscriptions (non-blocking, fades smoothly). */
  function drawCalligraphy(ctx, cam, text, alpha) {
    if (!text || alpha <= 0.01) return
    ctx.save()
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    ctx.globalAlpha = clamp(alpha, 0, 1)
    const size = Math.max(18, Math.round(cam.height * 0.056))
    ctx.font = '600 ' + size + 'px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
    ctx.fillStyle = INK
    ctx.shadowColor = 'rgba(244,238,224,0.9)'; ctx.shadowBlur = 12
    ctx.fillText(text, cam.width / 2, cam.height * 0.34)
    ctx.restore()
  }

  globalThis.XiabanInkReturnArt = {
    fadeAt, build, background, ground, foreground,
    drawStains, drawBlankStroke, whiteoutAt, drawWhiteout, drawCalligraphy,
  }
})()
