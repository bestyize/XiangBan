/* 水墨剑客 · scene composition: parallax ink-wash layers, props, weather and paper post-processing. */
(() => {
  'use strict'

  const art = globalThis.XiabanInkArt
  const rules = globalThis.XiabanInkBlade
  const WORLD = globalThis.XiabanInkLevels.volume(1)
  const { INK } = art
  const GROUND = rules.GROUND
  const WATER = [3330, 4230]
  const smooth = (t) => { const v = Math.max(0, Math.min(1, t)); return v * v * (3 - 2 * v) }
  const rgb = (c) => 'rgb(' + Math.round(c[0]) + ',' + Math.round(c[1]) + ',' + Math.round(c[2]) + ')'

  // Time of day along the scroll: noon bamboo, dusk ridge, grey bridge, night rain, moonlit duel.
  const SKY = [
    { x: 0, top: [219, 211, 190], low: [242, 235, 218], tint: [255, 248, 236, 0], sun: 1, moon: 0, rain: 0, leaves: 1, petals: 0, bamboo: 1 },
    { x: 1250, top: [214, 207, 188], low: [240, 233, 215], tint: [250, 244, 230, 0.08], sun: 1, moon: 0, rain: 0, leaves: 1, petals: 0, bamboo: 1 },
    { x: 2350, top: [210, 186, 162], low: [244, 222, 196], tint: [255, 226, 202, 0.26], sun: 1, moon: 0, rain: 0, leaves: 0.35, petals: 0, bamboo: 0.1 },
    { x: 3300, top: [203, 202, 195], low: [234, 231, 220], tint: [232, 236, 238, 0.16], sun: 0.4, moon: 0, rain: 0, leaves: 0.5, petals: 0, bamboo: 0.45 },
    { x: 4250, top: [150, 152, 156], low: [210, 208, 200], tint: [192, 199, 207, 0.42], sun: 0, moon: 0, rain: 1, leaves: 0.6, petals: 0, bamboo: 1 },
    { x: 5250, top: [40, 40, 46], low: [204, 194, 178], tint: [196, 190, 202, 0.3], sun: 0, moon: 1, rain: 0, leaves: 0, petals: 1, bamboo: 0 },
  ]
  const ENV_KEYS = ['sun', 'moon', 'rain', 'leaves', 'petals', 'bamboo']

  function environment(x) {
    let a = SKY[0], b = SKY[0], t = 0
    for (let i = 0; i < SKY.length - 1; i++) {
      const next = SKY[i + 1]
      if (x < next.x - 280) { a = b = SKY[i]; break }
      if (x < next.x + 280) { a = SKY[i]; b = next; t = smooth((x - next.x + 280) / 560); break }
      a = b = next
    }
    const mix = (p, q) => p.map((v, j) => v + (q[j] - v) * t)
    const out = { top: mix(a.top, b.top), low: mix(a.low, b.low), tint: mix(a.tint, b.tint) }
    for (const key of ENV_KEYS) out[key] = a[key] + (b[key] - a[key]) * t
    return out
  }

  const PROPS = [
    { s: 'lantern', x: 300 }, { s: 'gate', x: 690 }, { s: 'lantern', x: 1380 }, { s: 'stele', x: 1960 },
    { s: 'tree0', x: 2470 }, { s: 'tree1', x: 2870 }, { s: 'tree2', x: 3130 }, { s: 'pillar', x: 3368 }, { s: 'pillar', x: 4152 },
    { s: 'lantern', x: 4290 }, { s: 'shrine', x: 4440, alpha: 0.8 }, { s: 'lantern', x: 5245 }, { s: 'pine', x: 5560, alpha: 0.5 },
  ]

  function grass(res) {
    const random = art.rng(91)
    const period = 900
    const tufts = []
    for (let i = 0; i < 24; i++) {
      const blades = []
      const n = 3 + Math.floor(random() * 5)
      for (let k = 0; k < n; k++) blades.push({ a: -Math.PI / 2 + (k - n / 2) * 0.2 + 0.12 + (random() - 0.5) * 0.15, l: 30 + random() * 34, w: 1.8 + random() * 1.2, alpha: 0.65 + random() * 0.35 })
      tufts.push({ x: random() * period, blades })
    }
    return art.strip(period, 296, 100, res, (g, u0, u1) => {
      for (const shift of [-period, 0, period]) for (const tuft of tufts) {
        const x = tuft.x + shift
        if (x < u0 - 50 || x > u1 + 50) continue
        tuft.blades.forEach((blade, k) => {
          g.fillStyle = art.rgba(INK, blade.alpha)
          art.leaf(g, x + k * 2.4, 394, blade.a, blade.l, blade.w)
        })
      }
    })
  }

  function build(o) {
    const { P, viewW } = o
    const r = (f) => Math.max(0.6, P * f)
    const span = (f) => WORLD.length * f + viewW + 260
    const a = { P, viewW }
    a.grain = art.paperGrain(256)
    a.vignette = art.vignette(o.width, o.height)
    a.far = art.mountains({ width: span(0.06), top: 30, height: 236, base: 248, res: r(0.3), seed: 11, peakW: [26, 64], peakH: [85, 185], gap: [0.45, 1.05], round: 1.25, detail: 16, detailAmp: 9, color: [80, 92, 108], alpha: 0.3, depth: 70, strokeCell: 4, strokeDensity: 0.55, strokeLen: 9, outline: 0.55, trees: 0.12, treeH: 3, fadeH: 80 })
    a.mid = art.mountains({ width: span(0.16), top: 90, height: 182, base: 264, res: r(0.4), seed: 23, peakW: [60, 150], peakH: [45, 120], gap: [0.55, 1.1], round: 1.6, detail: 22, detailAmp: 12, color: [56, 62, 72], alpha: 0.42, depth: 46, strokeCell: 4, strokeDensity: 0.6, strokeLen: 8, outline: 0.75, trees: 0.4, treeH: 6, fadeH: 60, pavilions: [520, 1300] })
    a.near = art.mountains({ width: span(0.32), top: 170, height: 126, base: 289, res: r(0.5), seed: 37, peakW: [110, 260], peakH: [24, 74], gap: [0.6, 1], round: 2, detail: 26, detailAmp: 10, color: [40, 40, 42], alpha: 0.55, depth: 34, strokeCell: 3, strokeDensity: 0.65, strokeLen: 7, outline: 0.9, trees: 0.55, treeH: 9, ochre: 0.14, fadeH: 40, pavilions: [900, 1640], pavilionScale: 1.6 })
    a.bambooFar = art.bamboo({ period: 1100, count: 26, thick: [3, 6], lean: 0.05, alpha: [0.08, 0.2], top: -120, bottom: 292, res: r(0.45), seed: 51, color: [60, 66, 72], leaves: 0.3, leafLen: 16 })
    a.bambooNear = art.bamboo({ period: 1400, count: 9, thick: [6, 10], lean: 0.04, alpha: [0.24, 0.42], top: -120, bottom: 302, res: r(0.6), seed: 67, color: INK, leaves: 0.42, leafLen: 22 })
    a.fg = art.bamboo({ period: 2000, count: 3, thick: [13, 19], lean: 0.03, alpha: [0.55, 0.66], top: -150, bottom: 420, res: r(0.45), seed: 83, color: [14, 14, 16], leaves: 0.55, leafLen: 42 })
    a.grass = grass(r(0.6))
    a.ground = art.ground({ period: 1024, line: GROUND, depth: 92, res: r(1), seed: 5 })
    a.water = art.water({ period: 640, line: GROUND, depth: 92, res: r(0.8), seed: 9 })
    a.gate = art.gate(r(1)); a.lantern = art.lantern(r(1)); a.stele = art.stele(r(1)); a.shrine = art.shrine(r(0.9))
    a.tree0 = art.deadTree(r(0.9), 3, 1); a.tree1 = art.deadTree(r(0.9), 8, 0.8); a.tree2 = art.deadTree(r(0.9), 13, 1.15)
    a.pine = art.pine(r(0.9), 21); a.pillar = art.pillar(r(1), 4)
    a.platforms = WORLD.platforms.map((p, i) => p.x >= WATER[0] && p.x < WATER[1]
      ? { p, s: art.bridge(p.w, p.y, GROUND, r(1), 30 + i) }
      : { p, s: art.rock(p.w, p.y, GROUND, 40 + i, r(1)) })
    a.splats = [0, 1, 2, 3, 4, 5].map((i) => art.splat(100 + i, 128))
    a.redSplats = [0, 1, 2].map((i) => art.splat(200 + i, 128, art.CINNABAR))
    a.puff = art.softDisc(64, [44, 44, 48], 0.55)
    a.glow = art.softDisc(64, [226, 70, 44], 0.9)
    a.light = art.softDisc(64, [252, 248, 238], 0.95)
    a.sun = art.sun(256, [192, 58, 40], 2)
    a.moon = art.moon(256)
    a.mist = art.mist(512, 96)
    a.leaf = art.leafSprite([28, 30, 28], 0.85); a.petal = art.petalSprite()
    a.wave = art.waveSprite(r(1))
    return a
  }

  // cam: { left, top, viewW, viewH, P, width, height, sx, sy } — `width`/`height` are backing pixels.
  function layer(ctx, cam, f, skew = 0, pivot = GROUND) {
    const P = cam.P
    ctx.setTransform(P, 0, skew * P, P, -P * (cam.left * f + skew * pivot) + cam.sx * f, -P * cam.top + cam.sy * Math.min(1, f))
  }

  function mist(ctx, a, cam, f, y, h, alpha, drift, seed) {
    layer(ctx, cam, f)
    const spacing = 250
    const offset = cam.left * f
    const first = Math.floor((offset - drift - 500) / spacing)
    for (let n = first; n * spacing + drift < offset + cam.viewW + 200; n++) {
      const w = 380 * (0.7 + art.hash(n, seed) * 0.8)
      const u = n * spacing + art.hash(n, seed + 1) * 120 + drift
      ctx.globalAlpha = alpha * (0.55 + 0.45 * art.hash(n, seed + 2))
      ctx.drawImage(a.mist, u - w / 2, y - h / 2 + (art.hash(n, seed + 3) - 0.5) * h * 0.4, w, h)
    }
    ctx.globalAlpha = 1
  }

  function drawBackground(ctx, a, cam, e, clock) {
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    const sky = ctx.createLinearGradient(0, 0, 0, cam.height)
    sky.addColorStop(0, rgb(e.top)); sky.addColorStop(0.72, rgb(e.low)); sky.addColorStop(1, rgb(e.low))
    ctx.fillStyle = sky; ctx.fillRect(0, 0, cam.width, cam.height)
    const P = cam.P
    if (e.sun > 0.01) {
      const size = 96 * P
      const x = (cam.viewW * 0.74 - cam.left * 0.03) * P
      const y = (64 + cam.left / WORLD.length * 70 - cam.top) * P
      ctx.globalAlpha = e.sun * 0.92
      ctx.drawImage(a.sun, x - size / 2, y - size / 2, size, size)
    }
    if (e.moon > 0.01) {
      const size = 150 * P
      const x = (cam.viewW * 0.67 - (cam.left - 5300) * 0.05) * P
      const y = (82 - cam.top) * P
      ctx.globalAlpha = e.moon
      ctx.drawImage(a.moon, x - size / 2, y - size / 2, size, size)
    }
    ctx.globalAlpha = 1
    layer(ctx, cam, 0.06); art.drawStrip(ctx, a.far, cam.left * 0.06, cam.viewW)
    mist(ctx, a, cam, 0.1, 236, 64, 0.7, clock * 5, 7)
    layer(ctx, cam, 0.16); art.drawStrip(ctx, a.mid, cam.left * 0.16, cam.viewW)
    mist(ctx, a, cam, 0.24, 262, 50, 0.6, clock * 8, 13)
    layer(ctx, cam, 0.32); art.drawStrip(ctx, a.near, cam.left * 0.32, cam.viewW)
    mist(ctx, a, cam, 0.45, 287, 38, 0.55, clock * 11, 19)
    if (e.bamboo > 0.02) {
      const sway = Math.sin(clock * 0.6) * 0.01 + Math.sin(clock * 1.7) * 0.004
      layer(ctx, cam, 0.55, sway, 292)
      ctx.globalAlpha = e.bamboo
      art.drawStrip(ctx, a.bambooFar, cam.left * 0.55 - 30, cam.viewW + 60, 1100)
      ctx.globalAlpha = 1
    }
    mist(ctx, a, cam, 0.7, 295, 30, 0.5, clock * 14, 29)
    if (e.bamboo > 0.02) {
      const sway = Math.sin(clock * 0.7 + 1) * 0.013 + Math.sin(clock * 2.1) * 0.004
      layer(ctx, cam, 0.8, sway, 302)
      ctx.globalAlpha = e.bamboo * 0.95
      art.drawStrip(ctx, a.bambooNear, cam.left * 0.8 - 40, cam.viewW + 80, 1400)
      ctx.globalAlpha = 1
    }
  }

  function drawGround(ctx, a, cam, e, clock) {
    layer(ctx, cam, 1)
    const left = cam.left - 60, right = cam.left + cam.viewW + 60
    ctx.save()
    ctx.beginPath()
    if (left < WATER[0]) ctx.rect(left, 260, Math.min(right, WATER[0]) - left, 220)
    if (right > WATER[1]) ctx.rect(Math.max(left, WATER[1]), 260, right - Math.max(left, WATER[1]), 220)
    ctx.clip()
    art.drawStrip(ctx, a.ground, cam.left, cam.viewW, 1024)
    ctx.restore()
    if (right > WATER[0] && left < WATER[1]) {
      ctx.save()
      ctx.beginPath(); ctx.rect(WATER[0], 260, WATER[1] - WATER[0], 220); ctx.clip()
      const flow = (clock * 9) % 640
      ctx.translate(flow, 0)
      art.drawStrip(ctx, a.water, cam.left - flow, cam.viewW, 640)
      ctx.restore()
    }
    for (const prop of PROPS) {
      const s = a[prop.s]
      if (!s || prop.x + s.width < left || prop.x - s.width > right) continue
      art.drawSprite(ctx, s, prop.x, GROUND, prop.alpha ?? 1)
    }
    for (const entry of a.platforms) {
      if (entry.p.x + entry.p.w < left || entry.p.x > right) continue
      art.drawSprite(ctx, entry.s, entry.p.x, entry.p.y)
    }
  }

  function drawForeground(ctx, a, cam, e, clock) {
    const sway = Math.sin(clock * 0.9 + 2) * 0.012
    layer(ctx, cam, 1.3, sway, 400)
    ctx.globalAlpha = 0.35 + e.bamboo * 0.65
    art.drawStrip(ctx, a.fg, cam.left * 1.3 - 40, cam.viewW + 80, 2000)
    layer(ctx, cam, 1.18, sway * 1.6, 394)
    ctx.globalAlpha = 0.85
    art.drawStrip(ctx, a.grass, cam.left * 1.18 - 30, cam.viewW + 60, 900)
    ctx.globalAlpha = 1
  }

  function createWeather() { return { leaves: [], drops: [], ripples: [], rippleClock: 0 } }

  function spawnLeaf(w, cam, e, anywhere) {
    const petal = Math.random() < e.petals / Math.max(0.01, e.petals + e.leaves)
    return {
      x: cam.left + (anywhere ? Math.random() * cam.viewW : cam.viewW + 20 + Math.random() * 60), y: cam.top + (anywhere ? Math.random() * 220 : -20 - Math.random() * 60),
      vx: -18 - Math.random() * 34, vy: 20 + Math.random() * 22, spin: Math.random() * 6, vs: 1.5 + Math.random() * 3,
      size: (petal ? 6 : 9) + Math.random() * 6, petal, phase: Math.random() * 6,
    }
  }

  function updateWeather(w, cam, e, dt, clock) {
    const target = Math.round(13 * e.leaves + 16 * e.petals)
    while (w.leaves.length < target) w.leaves.push(spawnLeaf(w, cam, e, true))
    for (let i = w.leaves.length - 1; i >= 0; i--) {
      const leaf = w.leaves[i]
      leaf.x += (leaf.vx + Math.sin(clock * 1.2 + leaf.phase) * 22) * dt
      leaf.y += leaf.vy * dt
      leaf.spin += leaf.vs * dt
      if (leaf.x < cam.left - 40 || leaf.y > 330 || leaf.x > cam.left + cam.viewW + 140) {
        if (w.leaves.length > target) w.leaves.splice(i, 1)
        else w.leaves[i] = spawnLeaf(w, cam, e, false)
      }
    }
    const drops = Math.round(160 * e.rain)
    while (w.drops.length < drops) w.drops.push({ x: Math.random(), y: Math.random(), len: 0.03 + Math.random() * 0.03, speed: 1.6 + Math.random() * 0.9 })
    if (w.drops.length > drops) w.drops.length = drops
    for (const drop of w.drops) {
      drop.y += drop.speed * dt
      drop.x -= drop.speed * 0.16 * dt
      if (drop.y > 1.05) { drop.y = -0.05; drop.x = Math.random() * 1.2 }
    }
    w.rippleClock += dt * 40 * e.rain
    while (w.rippleClock > 1) {
      w.rippleClock -= 1
      w.ripples.push({ x: cam.left + Math.random() * cam.viewW, y: GROUND + 2 + Math.random() * 30, t: 0 })
    }
    for (let i = w.ripples.length - 1; i >= 0; i--) { w.ripples[i].t += dt; if (w.ripples[i].t > 0.5) w.ripples.splice(i, 1) }
  }

  function drawWeather(ctx, a, cam, w, e) {
    layer(ctx, cam, 1)
    for (const leaf of w.leaves) {
      ctx.save()
      ctx.translate(leaf.x, leaf.y)
      ctx.rotate(leaf.spin * 0.6 + (leaf.petal ? 0 : 2.4))
      ctx.scale(1, Math.cos(leaf.spin) * 0.9 + 0.1)
      ctx.drawImage(leaf.petal ? a.petal : a.leaf, -leaf.size / 2, -leaf.size / 2, leaf.size, leaf.size)
      ctx.restore()
    }
    if (w.ripples.length) {
      ctx.strokeStyle = 'rgba(40,46,54,0.35)'; ctx.lineWidth = 0.6
      for (const ripple of w.ripples) {
        const r = 2 + ripple.t * 18
        ctx.globalAlpha = 1 - ripple.t / 0.5
        ctx.beginPath(); ctx.ellipse(ripple.x, ripple.y, r, r * 0.22, 0, 0, Math.PI * 2); ctx.stroke()
      }
      ctx.globalAlpha = 1
    }
    if (w.drops.length) {
      ctx.setTransform(1, 0, 0, 1, 0, 0)
      ctx.strokeStyle = 'rgba(34,38,46,' + (0.28 * e.rain + 0.05) + ')'
      ctx.lineWidth = Math.max(1, cam.P * 0.32)
      ctx.beginPath()
      for (const drop of w.drops) {
        const x = drop.x * cam.width, y = drop.y * cam.height, l = drop.len * cam.height
        ctx.moveTo(x, y); ctx.lineTo(x - l * 0.16, y + l)
      }
      ctx.stroke()
    }
  }

  function drawPost(ctx, a, cam, e, quality) {
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    if (e.tint[3] > 0.01) {
      ctx.globalCompositeOperation = 'multiply'
      ctx.fillStyle = 'rgba(' + Math.round(e.tint[0]) + ',' + Math.round(e.tint[1]) + ',' + Math.round(e.tint[2]) + ',' + e.tint[3].toFixed(3) + ')'
      ctx.fillRect(0, 0, cam.width, cam.height)
    }
    if (quality > 0) {
      if (!a.grainPattern) a.grainPattern = ctx.createPattern(a.grain, 'repeat')
      ctx.globalCompositeOperation = 'multiply'
      ctx.fillStyle = a.grainPattern
      ctx.fillRect(0, 0, cam.width, cam.height)
    }
    ctx.globalCompositeOperation = 'source-over'
    ctx.drawImage(a.vignette, 0, 0, cam.width, cam.height)
  }

  globalThis.XiabanInkScene = { WATER, environment, build, layer, drawBackground, drawGround, drawForeground, createWeather, updateWeather, drawWeather, drawPost }
})()
