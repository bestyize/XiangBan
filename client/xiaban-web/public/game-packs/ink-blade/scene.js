/* 水墨剑客 · scene composition: parallax ink-wash layers, props, weather and paper post-processing. */
(() => {
  'use strict'

  const art = globalThis.XiabanInkArt
  const rules = globalThis.XiabanInkBlade
  const { INK } = art
  const GROUND = rules.GROUND
  const WATER = [3330, 4230]
  const smooth = (t) => { const v = Math.max(0, Math.min(1, t)); return v * v * (3 - 2 * v) }
  const rgb = (c) => 'rgb(' + Math.round(c[0]) + ',' + Math.round(c[1]) + ',' + Math.round(c[2]) + ')'

  const SKY_BAMBOO = [
    { x: 0, top: [219, 211, 190], low: [242, 235, 218], tint: [255, 248, 236, 0], sun: 1, moon: 0, rain: 0, leaves: 1, petals: 0, bamboo: 1, snow: 0, reeds: 0 },
    { x: 1250, top: [214, 207, 188], low: [240, 233, 215], tint: [250, 244, 230, 0.08], sun: 1, moon: 0, rain: 0, leaves: 1, petals: 0, bamboo: 1, snow: 0, reeds: 0 },
    { x: 2350, top: [210, 186, 162], low: [244, 222, 196], tint: [255, 226, 202, 0.26], sun: 1, moon: 0, rain: 0, leaves: 0.35, petals: 0, bamboo: 0.1, snow: 0, reeds: 0 },
    { x: 3300, top: [203, 202, 195], low: [234, 231, 220], tint: [232, 236, 238, 0.16], sun: 0.4, moon: 0, rain: 0, leaves: 0.5, petals: 0, bamboo: 0.45, snow: 0, reeds: 0 },
    { x: 4250, top: [150, 152, 156], low: [210, 208, 200], tint: [192, 199, 207, 0.42], sun: 0, moon: 0, rain: 1, leaves: 0.6, petals: 0, bamboo: 1, snow: 0, reeds: 0 },
    { x: 5250, top: [40, 40, 46], low: [204, 194, 178], tint: [196, 190, 202, 0.3], sun: 0, moon: 1, rain: 0, leaves: 0, petals: 1, bamboo: 0, snow: 0, reeds: 0 },
  ]

  const SKY_SNOW = [
    { x: 0, top: [168, 174, 192], low: [232, 234, 242], tint: [210, 218, 232, 0.15], sun: 0.7, moon: 0, rain: 0, leaves: 0, petals: 0, bamboo: 0, snow: 0.55, reeds: 0.25, forest: 1, blizzard: 0 },
    { x: 1300, top: [176, 183, 200], low: [228, 233, 241], tint: [215, 222, 235, 0.18], sun: 0.5, moon: 0, rain: 0, leaves: 0, petals: 0, bamboo: 0, snow: 0.45, reeds: 0.35, forest: 1, blizzard: 0 },
    { x: 2300, top: [190, 202, 222], low: [236, 242, 250], tint: [200, 215, 238, 0.28], sun: 0.6, moon: 0, rain: 0, leaves: 0, petals: 0, bamboo: 0, snow: 0.3, reeds: 0, forest: 0.25, blizzard: 0 },
    { x: 3300, top: [180, 158, 172], low: [242, 226, 220], tint: [230, 200, 192, 0.22], sun: 0.75, moon: 0, rain: 0, leaves: 0, petals: 0, bamboo: 0, snow: 0.4, reeds: 1, forest: 0.5, blizzard: 0 },
    { x: 4200, top: [58, 64, 80], low: [170, 176, 192], tint: [150, 160, 180, 0.42], sun: 0, moon: 0, rain: 0, leaves: 0, petals: 0, bamboo: 0, snow: 1, reeds: 0.3, forest: 0.85, blizzard: 0.3 },
    { x: 5200, top: [26, 29, 42], low: [182, 188, 204], tint: [175, 180, 200, 0.32], sun: 0, moon: 1, rain: 0, leaves: 0, petals: 0, bamboo: 0, snow: 0.5, reeds: 0.4, forest: 0.9, blizzard: 0.15 },
  ]
  const ENV_KEYS = ['sun', 'moon', 'rain', 'leaves', 'petals', 'bamboo', 'snow', 'reeds', 'forest', 'blizzard']

  function environment(x, volumeId = 1) {
    const sky = volumeId === 2 ? SKY_SNOW : SKY_BAMBOO
    let a = sky[0], b = sky[0], t = 0
    for (let i = 0; i < sky.length - 1; i++) {
      const next = sky[i + 1]
      if (x < next.x - 280) { a = b = sky[i]; break }
      if (x < next.x + 280) { a = sky[i]; b = next; t = smooth((x - next.x + 280) / 560); break }
      a = b = next
    }
    const mix = (p, q) => p.map((v, j) => v + (q[j] - v) * t)
    const out = { top: mix(a.top, b.top), low: mix(a.low, b.low), tint: mix(a.tint, b.tint) }
    for (const key of ENV_KEYS) out[key] = (a[key] ?? 0) + ((b[key] ?? 0) - (a[key] ?? 0)) * t
    return out
  }

  const PROPS_BAMBOO = [
    { s: 'lantern', x: 300 }, { s: 'gate', x: 690 }, { s: 'lantern', x: 1380 }, { s: 'stele', x: 1960 },
    { s: 'tree0', x: 2470 }, { s: 'tree1', x: 2870 }, { s: 'tree2', x: 3130 }, { s: 'pillar', x: 3368 }, { s: 'pillar', x: 4152 },
    { s: 'lantern', x: 4290 }, { s: 'shrine', x: 4440, alpha: 0.8 }, { s: 'lantern', x: 5245 }, { s: 'pine', x: 5560, alpha: 0.5 },
  ]

  const PROPS_SNOW = [
    { s: 'snowPine', x: 300, alpha: 0.7 }, { s: 'stele', x: 740 }, { s: 'plumTree', x: 1090 }, { s: 'snowPier', x: 1880 },
    { s: 'snowTree0', x: 2420 }, { s: 'plumTree', x: 2850 }, { s: 'snowTree2', x: 3130 }, { s: 'pillar', x: 3350 }, { s: 'snowTree0', x: 4000 },
    { s: 'shrine', x: 4420, alpha: 0.8 }, { s: 'snowTree1', x: 4880 }, { s: 'snowPine', x: 5540, alpha: 0.6 }, { s: 'plumTree', x: 5760 },
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

  // A run of ice is cut from its repeating tile and faded at both ends so it melts into the snow.
  function zoneSprite(layer, period, z0, z1, res, feather = 46) {
    const x0 = z0 - feather, w = z1 - z0 + feather * 2
    const node = art.canvas(w * res, layer.height * res)
    const g = node.getContext('2d')
    g.setTransform(res, 0, 0, res, -x0 * res, -layer.top * res)
    art.drawStrip(g, layer, x0, w, period)
    g.setTransform(1, 0, 0, 1, 0, 0)
    g.globalCompositeOperation = 'destination-in'
    const mask = g.createLinearGradient(0, 0, node.width, 0)
    const f = feather / w
    const ramp = [[0, 0], [0.25, 0.06], [0.5, 0.35], [0.75, 0.8], [1, 1]]
    for (const [at, alpha] of ramp) mask.addColorStop(at * f, 'rgba(0,0,0,' + alpha + ')')
    for (const [at, alpha] of ramp.slice().reverse()) mask.addColorStop(1 - at * f, 'rgba(0,0,0,' + alpha + ')')
    g.fillStyle = mask; g.fillRect(0, 0, node.width, node.height)
    return { node, x: x0, top: layer.top, w, h: layer.height }
  }

  function build(o) {
    const { P, viewW } = o
    const volumeId = o.volumeId || 1
    const world = globalThis.XiabanInkLevels.volume(volumeId) || globalThis.XiabanInkLevels.volume(1)
    const r = (f) => Math.max(0.6, P * f)
    const span = (f) => world.length * f + viewW + 260
    const a = { P, viewW, volumeId, world }
    a.grain = art.paperGrain(256)
    a.vignette = art.vignette(o.width, o.height)

    if (volumeId === 2) {
      a.far = art.mountains({ width: span(0.06), top: 25, height: 240, base: 250, res: r(0.3), seed: 101, peakW: [30, 75], peakH: [90, 195], gap: [0.4, 1], round: 1.2, detail: 16, detailAmp: 10, color: [75, 84, 102], alpha: 0.35, depth: 75, strokeCell: 4, strokeDensity: 0.5, strokeLen: 8, outline: 0.6, trees: 0.1, treeH: 3, fadeH: 80, snow: 0.9, snowFrom: 70, snowDepth: 50 })
      a.mid = art.mountains({ width: span(0.16), top: 85, height: 186, base: 265, res: r(0.4), seed: 107, peakW: [70, 160], peakH: [50, 130], gap: [0.5, 1.1], round: 1.5, detail: 22, detailAmp: 12, color: [52, 58, 72], alpha: 0.45, depth: 50, strokeCell: 4, strokeDensity: 0.55, strokeLen: 8, outline: 0.75, trees: 0.35, treeH: 6, fadeH: 65, snow: 0.9, snowFrom: 52, snowDepth: 36 })
      a.near = art.mountains({ width: span(0.32), top: 165, height: 130, base: 290, res: r(0.5), seed: 113, peakW: [120, 280], peakH: [30, 80], gap: [0.55, 0.95], round: 1.9, detail: 26, detailAmp: 10, color: [36, 40, 50], alpha: 0.58, depth: 36, strokeCell: 3, strokeDensity: 0.6, strokeLen: 7, outline: 0.9, trees: 0.5, treeH: 9, fadeH: 45, snow: 0.94, snowFrom: 30, snowDepth: 24 })
      a.reeds = art.reeds({ period: 900, count: 22, height: 74, res: r(0.6), seed: 42, top: 220, bottom: GROUND + 10, alpha: 0.65 })
      a.fgReeds = art.reeds({ period: 1100, count: 8, height: 110, res: r(0.5), seed: 99, top: 190, bottom: GROUND + 40, alpha: 0.8 })
      a.ground = art.snowGround({ period: 1024, line: GROUND, depth: 92, res: r(1), seed: 7 })
      a.ice = art.iceSurface({ period: 800, line: GROUND, depth: 92, res: r(0.85), seed: 15 })
      a.forestFar = art.snowForest({ period: 1100, count: 26, height: [34, 66], base: 292, top: 210, res: r(0.45), seed: 131, color: [70, 80, 100], alpha: 0.34, snow: 0.85 })
      a.forestNear = art.snowForest({ period: 1500, count: 11, height: [78, 128], base: 304, top: 170, res: r(0.6), seed: 137, color: [30, 36, 50], alpha: 0.62, snow: 0.94 })
      a.fg = art.plumBranches({ period: 2300, count: 3, top: -150, height: 372, res: r(0.5), seed: 149 })
      a.clouds = [art.inkCloud(3), art.inkCloud(8)]
      a.snowPine = art.snowPine(r(0.9), 21)
      a.snowPier = art.snowPier(r(0.9), 11)
      a.snowflake = art.snowflakeSprite(24)
      a.iceSpike = art.iceSpikeSprite(r(1))
      a.thinIce = art.thinIce({ period: 800, line: GROUND, depth: 92, res: r(0.85), seed: 61 })
      a.iceSprites = (world.iceZones || []).map(([z0, z1]) => zoneSprite(a.ice, 800, z0, z1, r(0.85)))
      a.thinSprites = (world.thinIce || []).map(([z0, z1]) => zoneSprite(a.thinIce, 800, z0, z1, r(0.85), 60))
      a.icicle = art.icicleSprite(r(1))
      a.reedBack = art.reeds({ period: 180, count: 9, height: 80, res: r(0.7), seed: 55, top: 190, bottom: GROUND + 10, alpha: 0.92 })
      a.reedFront = art.reeds({ period: 210, count: 5, height: 96, res: r(0.7), seed: 57, top: 180, bottom: GROUND + 14, alpha: 0.85 })
    } else {
      a.far = art.mountains({ width: span(0.06), top: 30, height: 236, base: 248, res: r(0.3), seed: 11, peakW: [26, 64], peakH: [85, 185], gap: [0.45, 1.05], round: 1.25, detail: 16, detailAmp: 9, color: [80, 92, 108], alpha: 0.3, depth: 70, strokeCell: 4, strokeDensity: 0.55, strokeLen: 9, outline: 0.55, trees: 0.12, treeH: 3, fadeH: 80 })
      a.mid = art.mountains({ width: span(0.16), top: 90, height: 182, base: 264, res: r(0.4), seed: 23, peakW: [60, 150], peakH: [45, 120], gap: [0.55, 1.1], round: 1.6, detail: 22, detailAmp: 12, color: [56, 62, 72], alpha: 0.42, depth: 46, strokeCell: 4, strokeDensity: 0.6, strokeLen: 8, outline: 0.75, trees: 0.4, treeH: 6, fadeH: 60, pavilions: [520, 1300] })
      a.near = art.mountains({ width: span(0.32), top: 170, height: 126, base: 289, res: r(0.5), seed: 37, peakW: [110, 260], peakH: [24, 74], gap: [0.6, 1], round: 2, detail: 26, detailAmp: 10, color: [40, 40, 42], alpha: 0.55, depth: 34, strokeCell: 3, strokeDensity: 0.65, strokeLen: 7, outline: 0.9, trees: 0.55, treeH: 9, ochre: 0.14, fadeH: 40, pavilions: [900, 1640], pavilionScale: 1.6 })
      a.bambooFar = art.bamboo({ period: 1100, count: 26, thick: [3, 6], lean: 0.05, alpha: [0.08, 0.2], top: -120, bottom: 292, res: r(0.45), seed: 51, color: [60, 66, 72], leaves: 0.3, leafLen: 16 })
      a.bambooNear = art.bamboo({ period: 1400, count: 9, thick: [6, 10], lean: 0.04, alpha: [0.24, 0.42], top: -120, bottom: 302, res: r(0.6), seed: 67, color: INK, leaves: 0.42, leafLen: 22 })
      a.fg = art.bamboo({ period: 2000, count: 3, thick: [13, 19], lean: 0.03, alpha: [0.55, 0.66], top: -150, bottom: 420, res: r(0.45), seed: 83, color: [14, 14, 16], leaves: 0.55, leafLen: 42 })
      a.grass = grass(r(0.6))
      a.ground = art.ground({ period: 1024, line: GROUND, depth: 92, res: r(1), seed: 5 })
      a.water = art.water({ period: 640, line: GROUND, depth: 92, res: r(0.8), seed: 9 })
      a.pine = art.pine(r(0.9), 21)
    }

    a.gate = art.gate(r(1)); a.lantern = art.lantern(r(1)); a.stele = art.stele(r(1)); a.shrine = art.shrine(r(0.9))
    a.tree0 = art.deadTree(r(0.9), 3, 1); a.tree1 = art.deadTree(r(0.9), 8, 0.8); a.tree2 = art.deadTree(r(0.9), 13, 1.15)
    a.pillar = art.pillar(r(1), 4)
    if (volumeId === 2) {
      a.stele = art.stele(r(1), true); a.shrine = art.shrine(r(0.9), true); a.pillar = art.pillar(r(1), 4, true)
      a.snowTree0 = art.deadTree(r(0.9), 3, 1, { snow: true, wide: true }); a.snowTree1 = art.deadTree(r(0.9), 8, 0.8, { snow: true, wide: true }); a.snowTree2 = art.deadTree(r(0.9), 13, 1.15, { snow: true, wide: true })
      a.plumTree = art.deadTree(r(0.9), 17, 1.05, { snow: true, blossom: true, wide: true })
    }
    a.platforms = world.platforms.map((p, i) => volumeId === 2
      ? { p, s: art.snowRock(p.w, p.y, GROUND, 40 + i, r(1)) }
      : p.x >= WATER[0] && p.x < WATER[1]
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
      const y = (64 + cam.left / a.world.length * 70 - cam.top) * P
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
    if (a.volumeId === 2 && a.clouds) {
      layer(ctx, cam, 0.025)
      const first = Math.floor((cam.left * 0.025 - 700) / 640)
      for (let n = first; n * 640 < cam.left * 0.025 + cam.viewW + 100; n++) {
        const w = 520 * (0.8 + art.hash(n, 5) * 0.7)
        ctx.globalAlpha = (0.4 + 0.4 * art.hash(n, 6)) * (1 - e.moon * 0.6)
        ctx.drawImage(a.clouds[n & 1], n * 640 + clock * 2.5 + art.hash(n, 7) * 140, 18 + art.hash(n, 8) * 70, w, w * 0.25)
      }
      ctx.globalAlpha = 1
    }
    layer(ctx, cam, 0.06); art.drawStrip(ctx, a.far, cam.left * 0.06, cam.viewW)
    mist(ctx, a, cam, 0.1, 236, 64, 0.7, clock * 5, 7)
    layer(ctx, cam, 0.16); art.drawStrip(ctx, a.mid, cam.left * 0.16, cam.viewW)
    mist(ctx, a, cam, 0.24, 262, 50, 0.6, clock * 8, 13)
    layer(ctx, cam, 0.32); art.drawStrip(ctx, a.near, cam.left * 0.32, cam.viewW)
    mist(ctx, a, cam, 0.45, 287, 38, 0.55, clock * 11, 19)

    if (a.volumeId === 2) {
      if (e.forest > 0.02) {
        const sway = Math.sin(clock * 0.5) * 0.006
        layer(ctx, cam, 0.55, sway, 292)
        ctx.globalAlpha = e.forest
        art.drawStrip(ctx, a.forestFar, cam.left * 0.55 - 30, cam.viewW + 60, 1100)
        ctx.globalAlpha = 1
        mist(ctx, a, cam, 0.7, 297, 34, 0.5, clock * 14, 29)
        layer(ctx, cam, 0.8, sway * 1.6, 304)
        ctx.globalAlpha = e.forest * 0.95
        art.drawStrip(ctx, a.forestNear, cam.left * 0.8 - 40, cam.viewW + 80, 1500)
        ctx.globalAlpha = 1
      }
      if (e.reeds > 0.02 && a.reeds) {
        const sway = Math.sin(clock * 1.1) * 0.018
        layer(ctx, cam, 0.72, sway, GROUND)
        ctx.globalAlpha = e.reeds * 0.9
        art.drawStrip(ctx, a.reeds, cam.left * 0.72 - 30, cam.viewW + 60, 900)
        ctx.globalAlpha = 1
      }
    } else {
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
  }

  function drawGround(ctx, a, cam, e, clock) {
    layer(ctx, cam, 1)
    const left = cam.left - 60, right = cam.left + cam.viewW + 60

    if (a.volumeId === 2) {
      const field = ctx.createLinearGradient(0, GROUND, 0, GROUND + 130)
      field.addColorStop(0, 'rgba(250,251,255,1)'); field.addColorStop(1, 'rgba(214,223,238,1)')
      ctx.fillStyle = field
      ctx.fillRect(left, GROUND + 1, right - left, 400)
      art.drawStrip(ctx, a.ground, cam.left, cam.viewW, 1024)
      for (const sp of a.iceSprites) if (right > sp.x && left < sp.x + sp.w) ctx.drawImage(sp.node, sp.x, sp.top, sp.w, sp.h)
      for (const sp of a.thinSprites) if (right > sp.x && left < sp.x + sp.w) ctx.drawImage(sp.node, sp.x, sp.top, sp.w, sp.h)
      const dusk = Math.min(0.5, e.moon * 0.34 + (e.blizzard || 0) * 0.3)
      if (dusk > 0.01) {
        ctx.fillStyle = 'rgba(24,30,52,' + dusk.toFixed(3) + ')'
        ctx.fillRect(left, GROUND - 3, right - left, 400)
      }
      for (const prop of PROPS_SNOW) {
        const s = a[prop.s]
        if (!s || prop.x + s.width < left || prop.x - s.width > right) continue
        art.drawSprite(ctx, s, prop.x, GROUND, prop.alpha ?? 1)
      }
      drawReedBeds(ctx, a, a.reedBack, cam, e, clock, 1)
    } else {
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
      for (const prop of PROPS_BAMBOO) {
        const s = a[prop.s]
        if (!s || prop.x + s.width < left || prop.x - s.width > right) continue
        art.drawSprite(ctx, s, prop.x, GROUND, prop.alpha ?? 1)
      }
    }
    for (const entry of a.platforms) {
      if (entry.p.x + entry.p.w < left || entry.p.x > right) continue
      art.drawSprite(ctx, entry.s, entry.p.x, entry.p.y)
    }
  }

  // Reed beds: stalks behind the player always, and a second rank in front that closes up while the player hides.
  function drawReedBeds(ctx, a, strip, cam, e, clock, alpha) {
    if (!strip) return
    const beds = a.world.reeds || []
    const left = cam.left - 60, right = cam.left + cam.viewW + 60
    for (const [r0, r1] of beds) {
      if (right < r0 - 30 || left > r1 + 30) continue
      ctx.save()
      ctx.beginPath(); ctx.rect(r0 - 18, 120, r1 - r0 + 36, 260); ctx.clip()
      const wave = Math.sin(clock * 1.4 + r0 * 0.01) * (0.012 + (e.blizzard || 0) * 0.02)
      ctx.transform(1, 0, wave, 1, -wave * GROUND, 0)
      ctx.globalAlpha = alpha
      art.drawStrip(ctx, strip, cam.left, cam.viewW, strip.width)
      ctx.restore()
    }
  }

  function drawCover(ctx, a, cam, e, clock, hide) {
    if (a.volumeId !== 2) return
    layer(ctx, cam, 1)
    drawReedBeds(ctx, a, a.reedFront, cam, e, clock, 0.22 + hide * 0.7)
  }

  function drawForeground(ctx, a, cam, e, clock) {
    if (a.volumeId === 2) {
      const gale = e.blizzard || 0
      const sway = Math.sin(clock * 0.8 + 2) * (0.006 + gale * 0.01)
      layer(ctx, cam, 1.3, sway, 400)
      ctx.globalAlpha = 0.5 + e.forest * 0.3
      art.drawStrip(ctx, a.fg, cam.left * 1.3 - 40, cam.viewW + 80, 2300)
      ctx.globalAlpha = 1
      if (a.fgReeds && e.reeds > 0.05) {
        const rs = Math.sin(clock * 1.2 + 2) * (0.02 + gale * 0.02)
        layer(ctx, cam, 1.25, rs, 400)
        ctx.globalAlpha = 0.25 + e.reeds * 0.6
        art.drawStrip(ctx, a.fgReeds, cam.left * 1.25 - 40, cam.viewW + 80, 1100)
        ctx.globalAlpha = 1
      }
      if (gale > 0.05) mist(ctx, a, cam, 1.12, GROUND - 2, 54, 0.62 * gale, clock * 70, 41)
    } else {
      const sway = Math.sin(clock * 0.9 + 2) * 0.012
      layer(ctx, cam, 1.3, sway, 400)
      ctx.globalAlpha = 0.35 + e.bamboo * 0.65
      art.drawStrip(ctx, a.fg, cam.left * 1.3 - 40, cam.viewW + 80, 2000)
      layer(ctx, cam, 1.18, sway * 1.6, 394)
      ctx.globalAlpha = 0.85
      art.drawStrip(ctx, a.grass, cam.left * 1.18 - 30, cam.viewW + 60, 900)
      ctx.globalAlpha = 1
    }
  }

  function createWeather() { return { leaves: [], drops: [], snow: [], ripples: [], rippleClock: 0, gale: 0, dir: -1 } }

  function spawnLeaf(w, cam, e, anywhere) {
    const petal = Math.random() < e.petals / Math.max(0.01, e.petals + e.leaves)
    return {
      x: cam.left + (anywhere ? Math.random() * cam.viewW : cam.viewW + 20 + Math.random() * 60), y: cam.top + (anywhere ? Math.random() * 220 : -20 - Math.random() * 60),
      vx: -18 - Math.random() * 34, vy: 20 + Math.random() * 22, spin: Math.random() * 6, vs: 1.5 + Math.random() * 3,
      size: (petal ? 6 : 9) + Math.random() * 6, petal, phase: Math.random() * 6,
    }
  }

  function updateWeather(w, cam, e, dt, clock) {
    const target = Math.round(13 * (e.leaves || 0) + 16 * (e.petals || 0))
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
    w.gale += ((e.blizzard || 0) - w.gale) * Math.min(1, dt * 1.2)
    const gale = w.gale
    w.dir += ((gale > 0.45 && e.windDir ? e.windDir : -1) - w.dir) * Math.min(1, dt * 2.4)
    const gust = 1 + gale * (0.55 * Math.sin(clock * 0.35) + 0.35 * Math.sin(clock * 0.93 + 1))
    const snowCount = Math.round(70 * (e.snow || 0) + 110 * gale)
    while (w.snow.length < snowCount) w.snow.push({ x: Math.random(), y: Math.random(), z: 0.25 + Math.random() * 0.75, phase: Math.random() * 6 })
    if (w.snow.length > snowCount) w.snow.length = snowCount
    w.gust = gust
    for (const flake of w.snow) {
      const vx = w.dir * (22 + 34 * flake.z) * (1 + gale * 5.5 * flake.z) * gust
      const vy = 26 + 46 * flake.z + gale * 26 * flake.z
      flake.x += (vx + Math.sin(clock * 1.6 + flake.phase) * 14 * (1 - gale * 0.6)) * dt / cam.viewW
      flake.y += vy * dt / cam.viewH
      if (flake.y > 1.04) { flake.y = -0.04; flake.x = Math.random() * 1.15 }
      if (flake.x < -0.06) { flake.x += 1.14; flake.y = Math.random() }
      else if (flake.x > 1.08) { flake.x -= 1.14; flake.y = Math.random() }
    }
    const drops = Math.round(160 * (e.rain || 0))
    while (w.drops.length < drops) w.drops.push({ x: Math.random(), y: Math.random(), len: 0.03 + Math.random() * 0.03, speed: 1.6 + Math.random() * 0.9 })
    if (w.drops.length > drops) w.drops.length = drops
    for (const drop of w.drops) {
      drop.y += drop.speed * dt
      drop.x -= drop.speed * 0.16 * dt
      if (drop.y > 1.05) { drop.y = -0.05; drop.x = Math.random() * 1.2 }
    }
    w.rippleClock += dt * 40 * (e.rain || 0)
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
    if (w.snow.length) {
      ctx.setTransform(1, 0, 0, 1, 0, 0)
      const P = cam.P
      const streak = w.gale > 0.12
      if (streak) {
        ctx.strokeStyle = 'rgba(246,249,255,' + (0.14 + w.gale * 0.3).toFixed(3) + ')'
        ctx.lineWidth = Math.max(1, P * 0.9)
        ctx.beginPath()
        for (const flake of w.snow) {
          if (flake.z < 0.55) continue
          const x = flake.x * cam.width, y = flake.y * cam.height, l = (16 + 46 * flake.z) * P * w.gale * (w.gust || 1)
          ctx.moveTo(x, y); ctx.lineTo(x - Math.sign(w.dir || -1) * l, y - l * 0.1)
        }
        ctx.stroke()
      }
      for (const flake of w.snow) {
        const size = (3.2 + 11 * flake.z * flake.z) * P
        ctx.globalAlpha = 0.5 + 0.45 * flake.z
        ctx.drawImage(a.snowflake, flake.x * cam.width - size / 2, flake.y * cam.height - size / 2, size, size)
      }
      ctx.globalAlpha = 1
      layer(ctx, cam, 1)
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

  globalThis.XiabanInkScene = { WATER, environment, build, layer, drawBackground, drawGround, drawCover, drawForeground, createWeather, updateWeather, drawWeather, drawPost }
})()
