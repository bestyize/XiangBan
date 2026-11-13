/* 悬山卷 · layered ink wash, painted architecture and animated mountain weather. */
(() => {
  'use strict'

  const A = globalThis.XiabanInkArt
  const R = globalThis.XiabanInkBlade
  const G = R.GROUND
  const TAU = Math.PI * 2
  const INK = [31, 35, 34]
  const PAPER = [247, 235, 210]
  const CINNABAR = '#ad3d32'
  const FONT = '"Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
  const FAR_SCROLL = .38, CLIFF_SCROLL = .68
  const PALETTES = [
    { far: [111, 118, 111], near: [60, 66, 60], wash: [232, 226, 200] },
    { far: [108, 131, 128], near: [62, 81, 81], wash: [224, 230, 214] },
    { far: [116, 128, 125], near: [67, 81, 81], wash: [230, 231, 213] },
    { far: [143, 132, 108], near: [86, 93, 75], wash: [238, 228, 205] },
    { far: [108, 124, 138], near: [55, 72, 84], wash: [224, 229, 220] },
    { far: [130, 121, 133], near: [74, 70, 82], wash: [237, 221, 204] },
  ]
  // World x, crest y, painted width, base y, profile, palette, ink strength.
  // The lighter ranges go down first. Landmarks stay in place as the camera walks past them.
  const RIDGES = [
    [-1020, 155, 330, 338, 2, 0, .28], [-320, 129, 315, 336, 1, 0, .29],
    [350, 168, 350, 325, 0, 0, .25], [980, 103, 331, 339, 2, 0, .32],
    [1620, 132, 381, 335, 1, 1, .28], [2290, 93, 305, 331, 0, 1, .35],
    [2960, 148, 410, 339, 2, 2, .27], [3500, 124, 382, 335, 0, 3, .31],
    [4200, 112, 370, 344, 1, 3, .33], [4840, 139, 372, 329, 2, 4, .31],
    [5540, 148, 370, 336, 1, 5, .31], [6270, 94, 325, 331, 2, 5, .35],
    [6920, 166, 390, 339, 1, 5, .29],
    [-630, 113, 315, 336, 0, 0, .49], [-25, 70, 235, 324, 1, 0, .57],
    [490, 122, 265, 325, 2, 0, .48], [1060, 102, 218, 325, 1, 0, .59],
    [1510, 65, 270, 338, 2, 1, .65], [1940, 39, 174, 329, 0, 1, .68],
    [2510, 114, 256, 335, 2, 2, .54], [3085, 79, 280, 327, 0, 2, .59],
    [3670, 132, 289, 334, 1, 3, .51], [4210, 78, 304, 326, 2, 3, .58],
    [4830, 53, 233, 332, 0, 4, .63], [5300, 111, 233, 324, 1, 4, .55],
    [5740, 157, 252, 329, 0, 5, .48], [6230, 111, 216, 321, 2, 5, .60],
    [6730, 66, 292, 325, 0, 5, .63],
  ]
  // The last value is the direction of each cliff's lower spur.
  const CLIFFS = [
    [-155, -45, 281, 2, 0, 1], [530, -85, 325, 0, 0, 1],
    [1460, 67, 210, 1, 1, 1], [2360, -45, 274, 1, 1, -1],
    [3220, 86, 248, 0, 2, -1], [4190, 47, 265, 2, 3, -1],
    [5160, 52, 270, 1, 4, -1], [6250, -19, 278, 1, 5, -1],
  ]
  const FALLS = [[645, 132, 236, 24], [1455, 164, 221, 18], [2270, 122, 247, 18], [5100, 149, 214, 11]]
  const EARTH = [
    [0, [195, 180, 149]], [1170, [195, 180, 149]], [1550, [182, 173, 148]],
    [2240, [178, 181, 161]], [2775, [176, 186, 175]], [3710, [209, 183, 143]],
    [4680, [174, 181, 179]], [5685, [194, 177, 165]],
  ]
  const BRIDGE = [1385, 2385]
  const PROFILES = [
    [[0, 1], [.06, .45], [.15, .35], [.2, .18], [.32, .15], [.42, .04], [.53, 0], [.63, .09], [.71, .29], [.78, .31], [.83, .58], [1, 1]],
    [[0, 1], [.13, .73], [.22, .65], [.32, .3], [.43, .27], [.53, .05], [.64, 0], [.74, .05], [.81, .22], [.94, .38], [1, 1]],
    [[0, 1], [.07, .67], [.17, .62], [.24, .39], [.36, .33], [.43, .1], [.51, .17], [.62, 0], [.74, .14], [.83, .17], [.95, .42], [1, 1]],
  ]

  function path(g, points, close = false) {
    g.beginPath()
    g.moveTo(points[0][0], points[0][1])
    for (let i = 1; i < points.length; i++) g.lineTo(points[i][0], points[i][1])
    if (close) g.closePath()
  }

  function wash(g, points, color) {
    path(g, points, true)
    g.fillStyle = color
    g.fill()
  }

  function stroke(g, points, color, width = 1) {
    path(g, points)
    g.strokeStyle = color
    g.lineWidth = width
    g.stroke()
  }

  function gradient(g, x, y, xx, yy, stops) {
    const color = g.createLinearGradient(x, y, xx, yy)
    for (const [p, value] of stops) color.addColorStop(p, value)
    return color
  }

  const mix = (from, to, q) => from.map((value, i) => Math.round(A.lerp(value, to[i], q)))
  const pigment = color => A.rgba(color, 1)
  const smooth = value => {
    const t = A.clamp(value, 0, 1)
    return t * t * (3 - 2 * t)
  }

  function earthAt(x) {
    const next = EARTH.findIndex(([at]) => x < at)
    if (next === 0) return EARTH[0][1]
    if (next < 0) return EARTH.at(-1)[1]
    const [lo, a] = EARTH[next - 1], [hi, b] = EARTH[next]
    return mix(a, b, smooth((x - lo) / (hi - lo)))
  }

  function brushStamp(seed) {
    const size = 96, half = size / 2, n = A.noise(seed, 128), rand = A.rng(seed + 1)
    const node = A.canvas(size, size), g = node.getContext('2d')
    const pixels = g.createImageData(size, size)
    for (let y = 0; y < size; y++) {
      for (let x = 0; x < size; x++) {
        const dx = (x - half) / half, dy = (y - half) / half, distance = Math.hypot(dx, dy)
        const angle = Math.atan2(dy, dx) / TAU + .5
        const edge = .80 + (n(angle * 128) - .5) * .24 + (n(angle * 256 + 31) - .5) * .07
        const q = A.clamp((edge - distance) / .44, 0, 1)
        const fibers = .45 + rand() * .43 + .12 * n(x * .63 + y * .12)
        pixels.data[(y * size + x) * 4 + 3] = 255 * q * q * (3 - 2 * q) * fibers
      }
    }
    g.putImageData(pixels, 0, 0)
    return node
  }

  function tintBrush(stamp, color) {
    const node = A.canvas(stamp.width, stamp.height), g = node.getContext('2d')
    g.drawImage(stamp, 0, 0)
    g.globalCompositeOperation = 'source-in'
    g.fillStyle = pigment(color); g.fillRect(0, 0, node.width, node.height)
    return node
  }

  function rockProfile(x, y, width, height, profile, seed, rooted) {
    const noise = A.noise(seed, 256), left = x - width / 2
    const points = []
    for (let i = 1; i < profile.length; i++) {
      const [a, b] = profile[i - 1], [c, d] = profile[i]
      const count = Math.max(2, Math.ceil((c - a) * width / 2.6))
      for (let j = 0; j < count; j++) {
        const t = j / count, u = A.lerp(a, c, t), distance = u * width
        const chips = (A.fbm(noise, distance * .098, 3) - .5) * 16
        const face = (noise(distance * .026 + 17) - .5) * 10
        points.push([left + distance, y + height * A.lerp(b, d, t) * (rooted ? .73 : 1) + chips + face])
      }
    }
    points.push([x + width / 2, y + height * (rooted ? .73 : 1)])
    return points
  }

  const inkRamps = new Map()
  function inkBleed(color, alpha) {
    const key = color.join(',') + '/' + alpha.toFixed(2)
    if (inkRamps.has(key)) return inkRamps.get(key)
    const node = A.canvas(2, 64), g = node.getContext('2d')
    g.fillStyle = gradient(g, 0, 0, 0, 64, [
      [0, A.rgba(color, alpha)], [.09, A.rgba(color, alpha * .78)],
      [.34, A.rgba(color, alpha * .29)], [.74, A.rgba(color, alpha * .08)], [1, A.rgba(color, 0)],
    ])
    g.fillRect(0, 0, 2, 64)
    inkRamps.set(key, node)
    return node
  }

  function rockInk(g, x, y, w, h, color, paper, seed, shape, strength = 1, detail = 1, rooted = false) {
    const rand = A.rng(seed), left = x - w / 2, bottom = y + h
    const edge = rockProfile(x, y, w, h, PROFILES[shape], seed + 33, rooted)
    const extent = rooted ? w * .2 : 0
    const dark = color.map((v, i) => Math.round(v * .62 + INK[i] * .38))
    g.save()
    // Lay down an opaque wash. Light ink is pigment mixed with paper, so a farther ridge
    // cannot show through the body of a nearer one. Fog is composited above the rock.
    const washClip = edge.map(([px, py], i) => [px, py + 1.3 + 4.3 * A.hash(i, seed)])
    if (rooted) washClip.push([x + w * .68, bottom + 21], [left - w * .16, bottom + 21])
    path(g, washClip, true); g.clip()
    g.fillStyle = gradient(g, 0, y, 0, bottom, [
      [0, pigment(mix(paper, color, strength * .86))],
      [.30, pigment(mix(paper, color, strength * .71))],
      [.66, pigment(mix(paper, color, strength * (rooted ? .57 : .36)))],
      [.91, pigment(mix(paper, color, strength * (rooted ? .44 : .12)))],
      [1, pigment(mix(paper, color, rooted ? strength * .39 : .015))],
    ])
    g.fillRect(left - extent, y - 9, w + extent * 2, h + (rooted ? 24 : 10))

    // Wet washes overlap in irregular patches. Their centres stay translucent, with heavier ink on the shadowed flank.
    for (let i = 0; i < 34 * detail; i++) {
      const px = left + rand() * w, py = y + h * rand() * .86
      const radius = w * (.09 + rand() * .23), skew = -.34 + rand() * .53
      const tone = i % 5 ? dark : PAPER
      const opacity = strength * (.075 + rand() ** 2 * .33) * (1 - (py - y) / h * .44)
      g.save(); g.translate(px, py); g.rotate(skew); g.scale(.63 + rand() * .28, 1.15 + rand() * .82)
      const wash = g.createRadialGradient(0, 0, 0, 0, 0, radius)
      wash.addColorStop(0, A.rgba(tone, opacity))
      wash.addColorStop(.42, A.rgba(tone, opacity * .83))
      wash.addColorStop(.73, A.rgba(tone, opacity * .32))
      wash.addColorStop(1, A.rgba(tone, 0))
      g.fillStyle = wash; g.fillRect(-radius, -radius, radius * 2, radius * 2); g.restore()
    }

    // Ink settles into the serrated ridge and bleeds down from the chipped edge.
    const ink = inkBleed(dark, strength * .71), ridgeDepth = Math.min(58, h * .25)
    for (let i = 0; i < edge.length - 1; i++) {
      const [px, py] = edge[i]
      const span = edge[i + 1][0] - px
      const shade = .28 + .72 * rand()
      g.globalAlpha = shade * (rooted ? 1 : Math.max(.16, (bottom - py) / h))
      g.drawImage(ink, px - .5, py - 1, span + 1.55, ridgeDepth * (.3 + rand() * 1.13))
    }
    g.globalAlpha = 1

    // Tapered, interrupted cun marks follow the local rock planes. Tight clusters leave open washes between them.
    const marks = Math.round(w * h * .0032 * detail)
    const clusters = Math.max(4, Math.round(w / 29))
    for (let i = 0; i < clusters; i++) {
      const cx = left + w * (.03 + rand() * .94), cy = y + h * (.08 + rand() * .79)
      const slant = (rand() - .52) * .65, radius = 17 + rand() * 47
      for (let j = 0; j < marks / clusters; j++) {
        const px = cx + (rand() - .5) * radius, py = cy + (rand() - .46) * radius * 1.5
        const length = 2.7 + rand() ** 2 * (11 + radius * .6)
        const drift = length * (slant + (rand() - .5) * .43)
        const width = .45 + rand() ** 2 * 2.6
        const alpha = strength * (.18 + rand() * .36) * (rooted ? .94 : A.clamp((bottom - py) / (h * .51), 0, 1))
        g.fillStyle = A.rgba(j % 5 ? dark : PAPER, alpha)
        A.ribbon(g, [[px, py], [px + drift * .33 - width, py + length * .32], [px + drift, py + length]],
          t => .12 + Math.sin(Math.PI * t) * width)
      }
      for (let j = 0; j < 10 * detail; j++) {
        const px = cx + (rand() - .5) * radius, py = cy + (rand() - .5) * radius
        const fade = rooted ? 1 : A.clamp((bottom - py) / (h * .54), 0, 1)
        g.fillStyle = A.rgba(j % 4 ? dark : PAPER, strength * (.1 + rand() * .27) * fade)
        g.beginPath(); g.ellipse(px, py, .4 + rand() * 2.2, .4 + rand(), -.56 + rand(), 0, TAU); g.fill()
      }
    }
    g.restore()

    // Alternate soft wet edges with broken brush crests and ink dots near the ridge.
    for (let j = 4; j < edge.length - 7; j += 4) {
      const [px, py] = edge[j]
      const alpha = strength * (rooted ? .65 : Math.max(0, (bottom - py) / h) * .49)
      if (rand() < .25 || alpha < .035) continue
      g.fillStyle = A.rgba(dark, alpha * (.34 + rand() * .53))
      A.ribbon(g, edge.slice(j - 2, j + 4), t => .2 + Math.sin(Math.PI * t) * (.7 + 1.4 * A.hash(j, seed + 8)))
      if (rand() < .58) {
        for (let k = 0; k < 6; k++) {
          g.fillStyle = A.rgba(dark, alpha * (.16 + rand() * .57))
          g.beginPath()
          g.ellipse(px + (rand() - .5) * 17, py + 3 + rand() * 9, .43 + rand() * 1.5, .35 + rand() * 1.9, -.4, 0, TAU)
          g.fill()
        }
      }
    }
  }

  function pine(g, x, y, scale, seed, color = [42, 54, 43]) {
    const rand = A.rng(seed)
    g.save(); g.translate(x, y); g.scale(scale, Math.abs(scale))
    const trunk = [[0, 5], [-13, -20], [-18, -48], [-38, -72], [-41, -106], [-67, -132], [-75, -168], [-59, -190]]
    g.fillStyle = gradient(g, -59, -176, 12, -4, [
      [0, A.rgba(color, .95)], [.72, A.rgba(color, 1)], [1, '#292d26'],
    ])
    A.ribbon(g, trunk, t => 23 * (1 - t) ** 1.7 + 2)
    A.dryBrush(g, t => {
      const index = Math.min(trunk.length - 2, Math.floor(t * (trunk.length - 1)))
      const q = t * (trunk.length - 1) - index
      return [A.lerp(trunk[index][0], trunk[index + 1][0], q), A.lerp(trunk[index][1], trunk[index + 1][1], q), 1, .2]
    }, 10, 9, [211, 188, 151], .49, seed, 30)
    for (const [at, side, width, rise] of [[1, 1, 71, 25], [3, -1, 110, 34], [4, 1, 154, 35], [5, -1, 99, 16], [6, 1, 121, 28], [7, -1, 53, 9]]) {
      const [bx, by] = trunk[at]
      const end = [bx + side * width, by - rise]
      g.fillStyle = A.rgba(color, .92)
      A.ribbon(g, [[bx, by], [bx + side * width * .28, by - rise * .48], [bx + side * width * .65, by - rise * .57], end], t => 9 * (1 - t) + .5)
      for (let j = 0; j < 7; j++) {
        const q = .29 + j * .128
        const px = bx + side * width * q, py = by - rise * q - rand() * 9
        const needleWash = g.createRadialGradient(px, py - 1, 0, px, py - 1, 18)
        needleWash.addColorStop(0, A.rgba(color, .64 + rand() * .14))
        needleWash.addColorStop(.47, A.rgba(color, .32)); needleWash.addColorStop(1, A.rgba(color, 0))
        g.fillStyle = needleWash
        g.save(); g.translate(px, py); g.scale(1, .44); g.translate(-px, -py)
        g.fillRect(px - 21, py - 21, 42, 42); g.restore()
        stroke(g, [[px, py + 7], [px + side * 5, py - 1], [px + side * 12, py - 6]], A.rgba(color, .73), 1.1)
        for (let n = 0; n < 14; n++) {
          const angle = -Math.PI + rand() * Math.PI, distance = 4 + rand() * 21
          const xx = px + Math.cos(angle) * distance, yy = py + Math.sin(angle) * distance * .48
          g.fillStyle = A.rgba(color, .25 + rand() * .53)
          A.leaf(g, xx, yy, angle, 5 + rand() * 11, .65 + rand() * 1.45)
        }
      }
    }
    g.restore()
  }

  function distant(index, res) {
    const [, y, w, base, shape, palette, strength] = RIDGES[index]
    const { far, wash: paper } = PALETTES[palette]
    return A.sprite(w + 24, 390, res, g => {
      rockInk(g, 0, y, w, base - y, far, paper, 561 + index * 37, shape, strength, strength < .4 ? .28 : .55)
    }, w / 2 + 12, 20)
  }

  function closeCliff(index, res) {
    const [, y, w, shape, palette, inner] = CLIFFS[index]
    const { far, near, wash: paper } = PALETTES[palette]
    const base = G + 68, height = base - y
    return A.sprite(w * 2.5, 500, res, g => {
      rockInk(g, inner * w * .19, y + 45, w * 1.37, height - 34, far, paper,
        940 + index * 39, (shape + 1) % 3, .68, .65, true)
      rockInk(g, 0, y, w, height, near, paper, 610 + index * 31, shape, .94, .9, true)
      // A lower spur joins the cliff to the valley floor.
      const toe = mix(paper, near, .57)
      rockInk(g, inner * w * .37, 246, w * .62, base - 240, near, toe,
        1050 + index * 23, (shape + 2) % 3, .58, .44, true)
      for (let k = 0; k < 3; k++) {
        pine(g, (k - 1) * w * .19, y + height * (.46 + k * .072), inner * (.14 + k * .055), 760 + index * 20 + k, near)
      }
    }, w * 1.25, 105)
  }

  function lantern(g, x, y, lit = true) {
    stroke(g, [[x, y - 13], [x, y]], '#50473c', 1)
    if (lit) {
      const halo = g.createRadialGradient(x, y + 8, 1, x, y + 8, 31)
      halo.addColorStop(0, 'rgba(252,207,127,.38)'); halo.addColorStop(1, 'rgba(252,207,127,0)')
      g.fillStyle = halo; g.fillRect(x - 31, y - 23, 62, 62)
    }
    wash(g, [[x - 6, y], [x + 6, y], [x + 9, y + 8], [x + 6, y + 17], [x - 6, y + 17], [x - 9, y + 8]], lit ? '#e9c381' : '#ad7960')
    for (const d of [-1, 1]) stroke(g, [[x + d * 3, y + 1], [x + d * 5, y + 8], [x + d * 3, y + 16]], '#856748', .65)
    stroke(g, [[x - 6, y], [x + 6, y]], '#463e37', 2)
    stroke(g, [[x - 6, y + 17], [x + 6, y + 17]], '#463e37', 2)
    stroke(g, [[x, y + 18], [x, y + 23]], '#a34338', 1)
  }

  function dryLine(g, points, width, color, alpha, seed, hairs = 7) {
    A.dryBrush(g, t => {
      const i = Math.min(points.length - 2, Math.floor(t * (points.length - 1)))
      const q = t * (points.length - 1) - i
      const [x, y] = points[i], [xx, yy] = points[i + 1]
      const length = Math.hypot(xx - x, yy - y) || 1
      return [A.lerp(x, xx, q), A.lerp(y, yy, q), (y - yy) / length, (xx - x) / length]
    }, width, hairs, color, alpha, seed, Math.max(16, points.length * 3))
  }

  function inkLine(g, points, width, color, seed, alpha = .88) {
    const n = A.noise(seed, 64)
    g.fillStyle = A.rgba(color, alpha * .57)
    A.ribbon(g, points, t => width * (.46 + n(t * 11 + .4) * .46) * (.4 + Math.sin(t * Math.PI) * .7))
    dryLine(g, points, width * 1.31, color, alpha, seed, 6)
  }

  function dab(g, x, y, rx, ry, brush, alpha) {
    g.globalAlpha = alpha
    g.drawImage(brush, x - rx, y - ry, rx * 2, ry * 2)
    g.globalAlpha = 1
  }

  function roof(g, x, y, w, paint, seed) {
    const rand = A.rng(seed), n = A.noise(seed + 7, 64)
    const left = x - w / 2, right = x + w / 2, ridge = w * .24
    const rim = [], crest = []
    for (let i = 0; i <= 46; i++) {
      const t = i / 46, edge = Math.abs(t * 2 - 1)
      rim.push([A.lerp(left - 8, right + 8, t), y + 15 - edge ** 4 * 31 + (n(t * 31) - .5) * 3.6])
    }
    for (let i = 0; i <= 22; i++) {
      const t = i / 22
      crest.push([x + (2 * t - 1) * ridge, y - 47 + Math.sin(t * Math.PI) * 4 + (n(t * 22 + 4) - .5) * 2])
    }
    const outline = [rim[0], [left + 21, y - 14], [x - ridge - 15, y - 37], ...crest,
      [x + ridge + 15, y - 37], [right - 24, y - 15], ...rim.slice().reverse()]
    const tone = [69, 77, 80]
    g.save()
    path(g, outline, true); g.clip()
    g.fillStyle = gradient(g, 0, y - 49, 0, y + 20, [
      [0, pigment(mix(tone, PAPER, .28))], [.27, pigment(tone)], [.71, '#353a3a'], [1, '#202524'],
    ])
    g.fillRect(left - 13, y - 53, w + 26, 74)
    for (let i = 0; i < w / 13; i++) {
      const px = left + rand() * w
      dab(g, px, y - 31 + rand() * 46, 11 + rand() * 43, 6 + rand() * 17, i % 4 ? paint.ink : paint.paper, .10 + rand() * .47)
    }

    // A few tile courses and groups of rafters survive as flying-white strokes.
    for (let i = 1; i < 44; i += 1 + Math.floor(rand() * 2.4)) {
      const t = i / 46, [xx, yy] = rim[i], angle = (xx - x) * .35
      const start = y - 40 + rand() * 22
      const points = [[x + (xx - x) * .49, start], [xx - angle * .35, y - 10 + rand() * 5], [xx, yy - rand() * 5]]
      dryLine(g, points, 1.2 + rand() * 2.5, i % 3 ? [202, 193, 173] : INK, .16 + rand() * .47, seed + i, 3)
      if (t > .11 && t < .9 && i % 3 === 0) {
        dryLine(g, [[xx - 13, yy - 8], [xx + 6, yy - 6]], 1.2, PAPER, .3, seed + i + 80, 3)
      }
    }
    for (let i = 0; i < 15; i++) {
      const cx = x + (rand() - .5) * w * .79, cy = y - 4 - rand() * 28, span = 7 + rand() * w * .13
      const wash = [[cx, cy], [cx + span * .43, cy - .8], [cx + span, cy + rand() * 2]]
      dryLine(g, wash, 1.6 + rand() * 1.4, [183, 179, 162], .2 + rand() * .21, seed + i + 94, 5)
    }
    g.restore()
    inkLine(g, rim, 6.9, [28, 34, 33], seed + 23, .94)
    inkLine(g, crest, 4.1, [44, 47, 43], seed + 32)
    dryLine(g, crest.map(([xx, yy]) => [xx, yy - 1]), 2.8, PAPER, .55, seed + 41)
    for (const d of [-1, 1]) {
      const xx = x + d * (w / 2 - 3)
      inkLine(g, [[xx - d * 21, y - 1], [xx - d * 2, y - 11], [xx + d * 7, y - 23], [xx + d * 4, y - 26]],
        4.4, INK, seed + d + 53)
    }
  }

  function beam(g, points, width, seed, cool = false) {
    const color = cool ? [51, 61, 64] : [65, 53, 44]
    inkLine(g, points, width, color, seed, .95)
    const grain = points.map(([x, y]) => [x - width * .16, y - width * .11])
    dryLine(g, grain, width * .36, cool ? [177, 182, 171] : [198, 167, 124], .63, seed + 3, 5)
  }

  function timber(g, x, y, width, height, cool = false) {
    const seed = Math.round(x * 3 + y), n = A.noise(seed, 64)
    const left = [], right = []
    for (let i = 0; i <= 12; i++) {
      const t = i / 12, cy = y + height * t
      left.push([x + (n(t * 17) - .5) * 2.7, cy])
      right.push([x + width + (n(t * 13 + 23) - .5) * 2.7, cy])
    }
    const tone = cool ? [59, 66, 68] : [77, 60, 47]
    g.save()
    path(g, [...left, ...right.slice().reverse()], true); g.clip()
    g.fillStyle = gradient(g, x, y, x + width, y + height * .03, [
      [0, pigment(mix(tone, PAPER, .31))], [.34, pigment(tone)], [.76, pigment(mix(tone, INK, .36))], [1, pigment(tone)],
    ])
    g.fillRect(x - 2, y, width + 4, height)
    const middle = left.map(([xx, yy], i) => [(xx + right[i][0]) * .5, yy])
    dryLine(g, middle, width * .86, [181, 161, 130], .31, seed + 3)
    dryLine(g, right, width * .5, INK, .92, seed + 8)
    for (let j = 1; j < 5; j++) {
      const q = j / 5, xx = x + width * (n(j + 7) * .76 + .12)
      inkLine(g, [[xx, y + height * q], [xx - 1.7, y + height * (q + .05)], [xx + 1.4, y + height * (q + .13)]],
        2 + n(j) * 2, INK, seed + j, .58)
    }
    g.restore()
    dryLine(g, left, 2.4, INK, .67, seed + 19)
    dryLine(g, right, 2.1, INK, .79, seed + 21)
  }

  // A pressed side-brush mark has a blunt shoulder and breaks into a few dry fibers.
  function axeStroke(g, x, y, dx, dy, width, color, alpha, rand) {
    const length = Math.hypot(dx, dy)
    const nx = -dy / length, ny = dx / length
    const bend = (rand() - .5) * width
    const point = (t, across) => [x + dx * t + nx * across, y + dy * t + ny * across]
    wash(g, [point(0, -width * .44), point(.14, width * .45), point(.46, width * .24 + bend),
      point(.93, bend), point(.68, bend - width * .12), point(.25, -width * .4)],
    gradient(g, x, y, x + dx, y + dy, [[0, A.rgba(color, alpha)], [.37, A.rgba(color, alpha * .65)], [1, A.rgba(color, 0)]]))
    for (let i = 0; i < 3; i++) {
      const start = .05 + rand() * .39, end = .56 + rand() * .45
      const offset = (rand() - .5) * width * .85
      stroke(g, [point(start, offset), point(.5 * (start + end), offset + bend * .65), point(end, offset * .63 + bend)],
        A.rgba(color, alpha * (.34 + rand() * .4)), .37 + rand() * .52)
    }
  }

  function inkPebble(g, x, y, w, h, tone, rand) {
    const rim = [[x - w * .56, y], [x - w * .33, y - h * .66], [x + w * .03, y - h],
      [x + w * .45, y - h * (.46 + rand() * .22)], [x + w * .58, y - h * .11], [x + w * .18, y + h * .24]]
    wash(g, rim, A.rgba(tone, .25 + rand() * .36))
    if (w >= 5) {
      wash(g, [rim[1], rim[2], rim[3], [x + w * .13, y - h * .35], [x - w * .23, y - h * .3]], A.rgba(PAPER, .46))
      stroke(g, [rim[0], rim[5], rim[4]], A.rgba(INK, .43), .6 + rand() * .43)
    }
  }

  const PATH_NOISE = A.noise(849, 256)
  const pathEdge = x => G + (A.fbm(PATH_NOISE, x / 26, 3) - .52) * 3.4

  function terrace(g, from, to, visibleFrom, visibleTo, exposeAt) {
    const width = to - from
    const inkAt = x => mix([52, 50, 44], earthAt(x), .14)
    const points = []
    const lo = Math.floor(visibleFrom / 3) * 3 - 6
    const hi = Math.ceil(visibleTo / 3) * 3 + 6
    for (let x = lo; x <= hi; x += 3) points.push([x, pathEdge(x)])
    const stops = [from, ...EARTH.map(([x]) => x).filter(x => x > from && x < to), to]
    const edgeInk = [67, 63, 51]

    g.save()
    g.beginPath(); g.rect(from, G - 14, width, 126); g.clip()
    g.save()
    path(g, [...points, [hi + 2, G + 108], [lo - 2, G + 108]], true); g.clip()
    g.fillStyle = gradient(g, from, 0, to, 0,
      stops.map(x => [(x - from) / width, pigment(mix(earthAt(x), PAPER, .48))]))
    g.fillRect(lo - 3, G - 4, hi - lo + 6, 113)
    const pooled = [87, 76, 57]
    g.fillStyle = gradient(g, 0, G - 1, 0, G + 103, [
      [0, A.rgba(pooled, .29)], [.07, A.rgba(pooled, .02)], [.27, A.rgba(PAPER, .27)],
      [.68, A.rgba(PAPER, .1)], [1, A.rgba(pooled, .22)],
    ])
    g.fillRect(lo - 3, G - 4, hi - lo + 6, 113)

    // Leave open paper between clusters. Ochre sits under the sharper flakes and grit.
    const cellWidth = 113
    for (let cell = Math.floor((lo - 160) / cellWidth); cell * cellWidth < hi; cell++) {
      const rand = A.rng(818 + cell * 127)
      const x = cell * cellWidth + 13 + rand() * 61
      const exposed = x >= exposeAt
      const y = G + 8 + rand() * 46, span = 23 + rand() * 76, depth = 4 + rand() * 8
      const slant = (rand() - .48) * 12
      const ink = inkAt(x + span * .5)
      const tint = mix(earthAt(x + span * .5), ink, exposed ? .37 : .24)
      const plane = [[x, y + depth * .56], [x + span * .18, y], [x + span * .61, y + slant * .6],
        [x + span, y + slant + 3]]
      g.fillStyle = gradient(g, x, y, x + span, y + slant + 3,
        [[0, A.rgba(tint, 0)], [.18, A.rgba(tint, .38)], [.63, A.rgba(tint, .22)], [1, A.rgba(tint, 0)]])
      A.ribbon(g, plane, t => .9 + depth * Math.sin(t * Math.PI) * (1.1 - t * .32))
      dryLine(g, plane, 2.7, tint, .35, 818 + Math.round(x), 5)

      const count = 4 + Math.floor(rand() * 5)
      for (let j = 0; j < count; j++) {
        const px = x + rand() * span, py = y + rand() * (depth + 6) + slant * (px - x) / span
        const length = 3 + rand() ** 1.6 * 25
        axeStroke(g, px, py, length, length * (.18 + slant * .02) + (rand() - .5) * 3,
          .85 + rand() * 2.5, j % 5 ? ink : PAPER, .44 + rand() * .28, rand)
      }
      if (exposed && rand() < .72) {
        // The summit has occasional ledges beneath the drier surface marks.
        const length = 24 + rand() * 63
        axeStroke(g, x + rand() * 42, G + 40 + rand() * 34, length, 7 + rand() * 15, 4.6, ink, .44, rand)
      }
    }

    for (let i = Math.floor((lo - 39) / 24); i * 24 < hi; i++) {
      const rand = A.rng(918 + i * 127)
      const x = i * 24 + rand() * 28, y = G + 3 + rand() ** 1.6 * 54
      const size = 1.4 + rand() ** 2.3 * 9.8
      const ink = inkAt(x)
      inkPebble(g, x, y, size, size * (.23 + rand() * .19), ink, rand)
      if (i % 4 === 0) {
        const length = 7 + rand() * 27
        axeStroke(g, x - length * .22, y + 3, length, (rand() - .45) * 4, 1.6, ink, .39, rand)
      }
    }

    g.restore()

    stroke(g, points, A.rgba(edgeInk, .64), 1)
    for (let i = Math.floor((lo - 84) / 47); i * 47 < hi; i++) {
      const rand = A.rng(933 + i * 67)
      const x = i * 47 + rand() * 18, length = 16 + rand() * 51
      A.dryBrush(g, t => [x + t * length, pathEdge(x + t * length) + .3, 0, 1],
        2.7 + rand() * 1.7, 6, edgeInk, .78, 1046 + i, Math.ceil(length / 3))
    }
    for (let i = Math.floor((lo - 47) / 43); i * 43 < hi + 12; i++) {
      const rand = A.rng(967 + i * 61)
      const x = i * 43 + rand() * 33, y = pathEdge(x)
      const group = 3 + Math.floor(rand() * 4)
      for (let j = 0; j < group; j++) {
        const xx = x + (rand() - .5) * 19
        const height = 1.5 + rand() ** 2 * (x >= exposeAt ? 6 : 10)
        g.fillStyle = A.rgba(mix([47, 53, 44], earthAt(xx), rand() * .23), .29 + rand() * .43)
        A.leaf(g, xx, y + 1 + rand() * 2, -1.65 + (rand() - .5) * 1.24, height, .44 + rand() * .7)
      }
    }
    g.restore()
  }

  function planks(g, x0, y, width, height, seed, paint) {
    const rand = A.rng(seed), n = A.noise(seed + 9, 128)
    const top = [], bottom = []
    for (let x = x0; x <= x0 + width; x += 4) {
      top.push([x, y + (n((x - x0) / 17) - .5) * 2])
      bottom.push([x, y + height + (n((x - x0) / 11 + 23) - .5) * 4])
    }
    top.push([x0 + width, y]); bottom.push([x0 + width, y + height])
    g.save(); path(g, [...top, ...bottom.slice().reverse()], true); g.clip()
    g.fillStyle = gradient(g, 0, y, 0, y + height, [[0, '#b29b76'], [.28, '#947e62'], [.61, '#605144'], [1, '#342e2b']])
    g.fillRect(x0 - 2, y - 2, width + 4, height + 5)
    for (let x = x0; x < x0 + width; x += 15 + rand() ** 1.4 * 42) {
      const z = x + rand() * 6
      dab(g, z, y + height * (.27 + rand() * .48), 6 + rand() * 31, 2 + rand() * 6, rand() < .55 ? paint.ink : paint.paper, .13 + rand() * .28)
      if (rand() > .21) {
        inkLine(g, [[z, y], [z + 2, y + height * .58], [z - 1, y + height]], 1.3 + rand() * 1.7, [49, 40, 33], seed + Math.round(z), .7)
      }
    }
    for (const offset of [3, 7, 13]) {
      if (offset >= height) continue
      dryLine(g, top.map(([x, yy]) => [x, yy + offset]), 1.6 + offset * .18, offset === 3 ? PAPER : INK, .64, seed + offset)
    }
    g.restore()
    stroke(g, top, '#514638b5', 1)
    dryLine(g, top, 3.6, [45, 42, 35], .86, seed)
  }

  function gateStone(g, rock, face, rand, washAlpha = 1) {
    const shade = mix(face, [68, 69, 60], .28)
    const light = mix(face, PAPER, .14)
    const [left, top, right, bottom] = rock.reduce(([l, t, r, b], [x, y]) =>
      [Math.min(l, x), Math.min(t, y), Math.max(r, x), Math.max(b, y)], [Infinity, Infinity, -Infinity, -Infinity])
    g.save(); path(g, rock, true); g.clip()
    g.fillStyle = gradient(g, left, top, right - (right - left) * .24, bottom, [
      [0, A.rgba(light, washAlpha)], [.38, A.rgba(face, washAlpha)], [1, A.rgba(shade, washAlpha)],
    ])
    g.fillRect(left, top, right - left, bottom - top)
    const dir = (right + left < 0 ? .92 : .63) + rand() * .23
    const marks = Math.ceil((right - left) * (bottom - top) / 74)
    for (let i = 0; i < marks; i++) {
      const x = A.lerp(left, right, rand()), y = A.lerp(top, bottom, rand())
      const length = 2.5 + rand() ** 2 * 14
      axeStroke(g, x, y, length * dir, 1.2 + length * (.37 + rand() * .44), 1 + rand() * 2.9,
        i % 13 ? shade : light, .26 + rand() * .45, rand)
    }
    g.restore()
  }

  function gateSeam(g, points, color, seed, width = 1.2) {
    const rand = A.rng(seed)
    for (let i = 1; i < points.length; i++) {
      const [x, y] = points[i - 1], [xx, yy] = points[i]
      const start = rand() * .2, end = .65 + rand() * .32
      stroke(g, [[A.lerp(x, xx, start), A.lerp(y, yy, start)], [A.lerp(x, xx, end), A.lerp(y, yy, end)]],
        A.rgba(color, .36 + rand() * .26), width * (.48 + rand() * .78))
    }
  }

  function chippedEdge(points, seed) {
    const n = A.noise(seed, 128), chips = []
    for (let i = 1; i < points.length; i++) {
      const [x, y] = points[i - 1], [xx, yy] = points[i]
      const dx = xx - x, dy = yy - y, length = Math.hypot(dx, dy)
      const count = Math.ceil(length / 4)
      for (let j = 0; j < count; j++) {
        const t = j / count
        const chip = (n(i * 17 + j * 1.28) - .5) * 3.5 * Math.sin(t * Math.PI)
        chips.push([x + dx * t - dy / length * chip, y + dy * t + dx / length * chip])
      }
    }
    chips.push(points.at(-1))
    return chips
  }

  function stoneGate(g) {
    g.save(); g.translate(1006, G)
    const foot = A.rng(533), rand = A.rng(542)
    const stone = [153, 146, 125], shade = [86, 85, 74]
    for (const [x, y, span] of [[-145, 2, 88], [-122, 6, 41], [53, 2, 98], [88, 6, 46]]) {
      axeStroke(g, x, y, span, 1.6, 3.9, shade, .46, foot)
    }
    const outer = [[-139, 1], [-131, -24], [-128, -55], [-134, -82], [-126, -112], [-140, -138], [-131, -162],
      [-113, -177], [-109, -201], [-83, -216], [-56, -218], [-29, -231], [-8, -225], [22, -227],
      [47, -219], [76, -220], [87, -207], [116, -190], [128, -172], [125, -142],
      [133, -116], [118, -91], [126, -67], [119, -44], [123, -19], [140, 2]]
    const opening = [[-61, 2], [-60, -33], [-57, -59], [-61, -81], [-60, -110], [-54, -135],
      [-39, -154], [-21, -164], [1, -170], [24, -162], [43, -148], [54, -129],
      [60, -107], [57, -84], [61, -62], [58, -36], [65, 2]]
    const arch = chippedEdge(opening.slice(4), 573)
    const mouth = [...chippedEdge(opening.slice(0, 5), 574).slice(0, -1), ...arch]
    const silhouette = [...chippedEdge(outer, 560), ...mouth.reverse()]
    g.save(); path(g, silhouette, true); g.clip()
    g.fillStyle = gradient(g, -133, -200, 126, -6, [
      [0, '#8c8878'], [.28, '#afa78c'], [.56, '#a6a089'], [1, '#858677'],
    ])
    g.fillRect(-144, -233, 290, 237)

    const shoulders = [
      [[-141, -139], [-107, -195], [-68, -214], [-33, -215], [-55, -175], [-86, -158], [-106, -127]],
      [[-81, -216], [-32, -231], [19, -229], [40, -210], [21, -186], [-25, -186], [-49, -197]],
      [[40, -217], [76, -218], [111, -195], [138, -165], [103, -142], [77, -164], [53, -168]],
      [[-146, -102], [-124, -117], [-94, -119], [-82, -100], [-93, -78], [-122, -69], [-145, -83]],
      [[83, -101], [116, -117], [141, -114], [130, -82], [104, -72], [84, -75]],
      [[-145, -51], [-116, -64], [-92, -57], [-79, -34], [-103, -20], [-136, -24]],
      [[87, -48], [126, -55], [142, -35], [134, -17], [108, -12], [93, -21]],
    ]
    for (const rock of shoulders) gateStone(g, rock, mix(stone, PAPER, .06 + rand() * .10), rand, .54)

    // Face stones have offset joints and independent changes of value. The arch is
    // read from the cut faces and their cast shadow, with only short ink accents.
    for (const d of [-1, 1]) {
      g.save(); g.scale(d, 1)
      let previous = [[58, 2], [89, 1], [112, 1], [143, 4]]
      for (const y of [-24, -53, -83, -115]) {
        const row = [[60, y + rand() * 4], [83 + rand() * 13, y - 3 + rand() * 6], [109 + rand() * 10, y - 3], [144, y - 5 + rand() * 5]]
        for (let i = 0; i < row.length - 1; i++) {
          const face = mix(stone, PAPER, .035 + rand() * .11)
          gateStone(g, [row[i], row[i + 1], previous[i + 1], previous[i]], face, rand)
        }
        gateSeam(g, row, shade, Math.round(759 + y * d), 1.5)
        previous = row
      }
      g.restore()
    }
    const joints = [0, .30, .62, 1.01, 1.39, 1.67, 2.06, 2.45, 2.81, Math.PI]
    for (let i = 1; i < joints.length; i++) {
      const a = joints[i - 1], b = joints[i]
      const at = (t, rx, ry) => [Math.cos(t) * rx, -112 - Math.sin(t) * ry]
      const rock = [at(a, 83, 76), at((a + b) / 2, 89 + rand() * 8, 81 + rand() * 5),
        at(b, 84, 76), at(b, 59, 58), at((a + b) / 2, 59, 58), at(a, 60, 58)]
      gateStone(g, rock, mix(stone, PAPER, .095 + rand() * .12), rand)
      gateSeam(g, [rock[0], at(a, 73, 69), rock[5]], shade, 893 + i, 1.1)
    }

    // Brush along the rock grain across the masonry joints to tie the tones together.
    const strata = [
      [[-134, -177], [-118, -170], [-114, -151], [-96, -139], [-89, -117]],
      [[-123, -138], [-115, -123], [-121, -103], [-113, -89]],
      [[-121, -74], [-108, -67], [-104, -53], [-94, -49]],
      [[49, -214], [67, -203], [87, -199], [96, -183], [107, -179]],
      [[109, -157], [100, -139], [103, -119], [95, -106], [99, -83]],
      [[112, -64], [106, -51], [110, -28], [119, -13], [130, -6]],
      [[-124, -32], [-115, -20], [-120, -8], [-129, 3]],
    ]
    for (const [i, points] of strata.entries()) {
      const [x, y] = points[0], [xx, yy] = points.at(-1)
      g.fillStyle = gradient(g, x, y, xx, yy, [[0, A.rgba(shade, 0)], [.28, A.rgba(shade, .24)],
        [.67, A.rgba(shade, .19)], [1, A.rgba(shade, 0)]])
      A.ribbon(g, points, t => 3 + 8 * Math.sin(Math.PI * t))
      dryLine(g, points, 3.8, shade, .47, 813 + i, 6)
    }

    // The texture continues across the fitted stone, with sparse raindrop marks on the high rock
    // and denser touches at the foot of each pier.
    for (let i = 0; i < 300; i++) {
      const x = (rand() - .5) * 277, y = -232 + rand() * 232
      const length = .9 + rand() ** 2 * 4.2
      const tone = i % 9 ? shade : PAPER
      stroke(g, [[x, y], [x - length * .24, y + length]],
        A.rgba(tone, .19 + rand() * .32), .32 + rand() * .85)
    }
    for (const d of [-1, 1]) {
      for (let i = 0; i < 39; i++) {
        const x = d * (77 + rand() ** .72 * 56), y = -1 - rand() ** 2.3 * 77
        const length = 1 + rand() ** 2 * 7
        axeStroke(g, x, y, d * (.8 + length * .33), length, .8 + rand() * 1.6, [64, 70, 60], .28 + rand() * .37, rand)
      }
    }
    g.restore()

    const innerFace = opening.slice(4).map(([x, y], i) => [
      x - (i < 6 ? 0 : 4 + i * .35), y + (i > 6 ? (i < 10 ? 8 : 0) : 2 + i),
    ])
    wash(g, [...arch, ...chippedEdge(innerFace, 589).reverse()],
      gradient(g, -64, -150, 66, -71, [[0, '#53584e36'], [.42, '#424a42a3'], [1, '#777a695e']]))
    for (const [start, end] of [[0, 5], [8, 12], [17, 21], [22, 25]]) {
      gateSeam(g, outer.slice(start, end), [66, 67, 57], 646 + start, 1.25)
    }
    for (const [start, end] of [[1, 4], [6, 9], [11, 14]]) {
      gateSeam(g, opening.slice(start, end).map(([x, y]) => [x * 1.07, y - 1]), [82, 82, 69], 682 + start, 1.1)
    }

    const plaque = [[-29, -218], [-11, -219], [34, -215], [32, -192], [11, -189], [-31, -192]]
    gateStone(g, plaque, [166, 151, 119], rand)
    gateSeam(g, plaque.slice(1, 4), [77, 71, 57], 581, 1.2)
    g.fillStyle = '#3e3f33'; g.font = '600 16px ' + FONT; g.textAlign = 'center'
    g.fillText('悬 山', 2, -198)
    g.save(); g.translate(-94, -116); g.scale(.76, .76); lantern(g, 0, 0, false); g.restore()
    g.save(); g.translate(92, -111); g.scale(.76, .76); lantern(g, 0, 0, false); g.restore()
    g.restore()
  }

  function bridgeAbutment(g, a, x, side, seed) {
    const top = [[-168, 1.4], [-81, 1.7], [-33, -.5], [12, .8], [28, 2], [50, -1.3], [67, 1.2], [87, .5], [108, 2.2]]
    const edge = [[99, 14], [93, 29], [71, 38], [66, 57], [43, 69], [31, 104], [24, 110], [-168, 110]]
    const stone = earthAt(x)
    const sprites = a.mountain.banks
    sprites[side] ||= A.sprite(280, 121, a.mountain.detailRes, ink => {
      ink.save(); path(ink, [...top, ...edge], true); ink.clip()
      ink.fillStyle = gradient(ink, 0, 0, 24, 105,
        [[0, pigment(mix(stone, PAPER, .36))], [.29, '#9b9881'], [.66, '#7e8270'], [1, '#616a62']])
      ink.fillRect(-168, -5, 280, 115)
      const rand = A.rng(seed)
      for (let i = 0; i < 44; i++) {
        const px = -67 + rand() * 177, y = 3 + rand() * 102
        const length = 5 + rand() ** 2 * 52
        axeStroke(ink, px, y, length, 2 + length * .24, 1.4 + rand() * 3.8,
          i % 5 ? [63, 66, 57] : PAPER, .19 + rand() * .39, rand)
      }
      ink.restore()
      dryLine(ink, top.slice(2), 3.6, INK, .76, seed + 21, 7)
      dryLine(ink, [top.at(-1), ...edge.slice(0, -1)], 3.4, INK, .50, seed + 25)
      ink.globalCompositeOperation = 'destination-in'
      ink.fillStyle = gradient(ink, -168, 0, -15, 0, [[0, '#0000'], [.25, '#0002'], [.78, '#000c'], [1, '#000']])
      ink.fillRect(-168, -6, 280, 121)
    }, 168, 6)
    g.save(); g.translate(x, G); g.scale(side, 1)
    A.drawSprite(g, sprites[side], 0, 0)
    g.restore()
  }

  function suspension(g, paint) {
    const [from, to] = BRIDGE
    const rail = x => G - 43 + 13 * Math.sin((x - from) / (to - from) * Math.PI)
    for (let x = from + 44; x <= to - 32; x += 25 + A.hash(x, 14) * 21) {
      const tilt = (A.hash(x, 25) - .5) * 8, head = rail(x) - 4 - A.hash(x, 27) * 9
      beam(g, [[x - 2, G + 6], [x - tilt * .3, G - 12], [x + tilt, head]], 3.5, Math.round(x))
    }
    for (const [lo, hi] of [[1417, 1655], [1720, 1910], [1950, 2116], [2170, 2347]]) {
      for (const y of [0, 12]) {
        const points = Array.from({ length: 12 }, (_, i) => {
          const x = A.lerp(lo, hi, i / 11)
          return [x, rail(x) + y + 2 * Math.sin(i * .9)]
        })
        inkLine(g, points, y ? 2.8 : 3.4, [70, 58, 49], lo + y, .8)
        dryLine(g, points.map(([x, yy]) => [x, yy - .8]), 1.8, PAPER, .43, hi + y, 4)
      }
    }
    for (const x of [1445, 1654, 1890, 2122, 2328]) {
      const anchor = x === 1445 || x === 2328
      const top = anchor ? 91 : G - 68
      timber(g, x - 8, top, 14, G + 66 - top)
      if (!anchor) beam(g, [[x - 4, G - 8], [x + 34, G + 21], [x + 88, G + 58]], 11, x + 3)
      for (const d of [0, 5, 10]) stroke(g, [[x - 11, G - 48 + d], [x + 7, G - 47 + d]], '#c1a380', 1.7)
    }
    const cable = Array.from({ length: 22 }, (_, i) => {
      const t = i / 21
      return [A.lerp(1510, 2220, t), G - 211 + 116 * t * (1 - t) + 5 * t]
    })
    beam(g, [[1441, 93], cable[0]], 3.3, 697)
    beam(g, [cable.at(-1), [2332, 93]], 3.3, 698)
    stroke(g, cable, '#483a2cc4', 1.2)
    dryLine(g, cable, 3.1, [61, 47, 37], .85, 690, 5)
    planks(g, from, G, to - from, 18, 209, paint)
    for (const x of [1498, 1634, 1819, 1974, 2150, 2282]) {
      inkLine(g, [[x, G + 13], [x + 17, G + 12], [x + 43, G + 13]], 5.5, [43, 39, 35], x)
      dryLine(g, [[x, G + 11], [x + 42, G + 11]], 2.3, [170, 143, 109], .66, x + 19, 4)
    }
  }

  function footing(g, x, y, w, seed) {
    const corners = [[x - w, y], [x - w + 3, y - 6], [x - w + 7, y - 15], [x + w - 7, y - 14],
      [x + w - 4, y - 8], [x + w, y]]
    wash(g, corners, '#898779')
    inkLine(g, [corners[0], corners[1], corners[2], corners[3], corners[4]], 2.6, INK, seed, .62)
    dryLine(g, [[x - w + 2, y - 4], [x - 2, y - 6], [x + w - 4, y - 7]], 2.2, PAPER, .64, seed + 6, 4)
  }

  function corridor(g, paint) {
    for (const x of [2455, 2580, 2705, 2830, 2955]) {
      timber(g, x - 8, 103, 16, 194, true)
      footing(g, x, G - 1, 19, x)
      beam(g, [[x - 6, 153], [x + 15, 130], [x + 40, 109]], 9.5, x + 29, true)
      beam(g, [[x + 6, 151], [x - 14, 130], [x - 36, 111]], 8.1, x + 32, true)
    }
    for (const y of [111, 133]) {
      beam(g, [[2439, y], [2535, y + 1], [2700, y - 1], [2820, y + .7], [2966, y - .4]], 13, 3100 + y, true)
    }
    roof(g, 2700, 104, 564, paint, 696)
    lantern(g, 2475, 133); lantern(g, 2904, 134)
    for (const [a, b] of [[2447, 2590], [2665, 2790], [2870, 3030]]) {
      beam(g, [[a, G - 39], [(a + b) / 2, G - 37], [b, G - 40]], 4.3, a, true)
      for (let x = a; x < b; x += 20 + A.hash(x, 27) * 13) {
        inkLine(g, [[x - 1, G - 9], [x + A.hash(x, 15) * 3, G - 38]], 2.8, [74, 81, 79], Math.round(x), .71)
      }
    }
  }

  function stoneLantern(g, x, y, scale, paint, lit = false) {
    g.save(); g.translate(x, y); g.scale(scale, scale)
    footing(g, 0, 0, 21, x)
    wash(g, [[-9, -10], [-4, -23], [-6, -41], [7, -42], [6, -25], [11, -11]], '#888878')
    inkLine(g, [[-7, -11], [-4, -26], [-5, -42]], 3.2, INK, x + 2, .66)
    dab(g, 3, -30, 7, 13, paint.paper, .45)
    wash(g, [[-14, -39], [-11, -58], [13, -58], [13, -41], [5, -38]], '#494e48')
    if (lit) lantern(g, 0, -55)
    else wash(g, [[-7, -41], [-6, -50], [5, -53], [7, -42]], '#b6a47b')
    const cap = [[-24, -55], [-16, -59], [-6, -63], [-2, -71], [3, -72], [8, -64], [23, -56], [10, -52], [-5, -53]]
    wash(g, cap, '#525a54')
    inkLine(g, cap.slice(0, 7), 3.5, INK, x + 8, .71)
    dryLine(g, [[-19, -55], [-8, -57], [7, -57], [19, -55]], 2.5, PAPER, .66, x + 6, 4)
    g.restore()
  }

  function courtyard(g, paint) {
    pine(g, 3553, G + 2, 1.16, 707, [47, 61, 43])
    pine(g, 3483, G + 6, -.51, 718, [67, 79, 61])
    stoneLantern(g, 3422, G, .74, paint)
    stoneLantern(g, 4067, G, .62, paint)
    wash(g, [[3893, G - 18], [3903, G - 27], [3916, G - 32], [3977, G - 30], [4006, G - 32], [4020, G - 20],
      [4011, G - 15], [3989, G - 14], [3947, G - 17], [3910, G - 15]], '#a6a08a')
    wash(g, [[3928, G - 16], [3939, G - 16], [3939, G], [3926, G], [3924, G - 6]], '#77735e')
    wash(g, [[3983, G - 16], [3994, G - 16], [3997, G], [3981, G]], '#77735e')
    inkLine(g, [[3894, G - 18], [3912, G - 26], [3948, G - 25], [4008, G - 27], [4020, G - 20]], 3, [62, 59, 49], 721, .75)
    dryLine(g, [[3908, G - 20], [3944, G - 22], [3980, G - 20], [4006, G - 22]], 4.3, PAPER, .71, 719)
    const rand = A.rng(717)
    for (let i = 0; i < 39; i++) {
      const x = (i < 20 ? 3457 : 4018) + rand() * 120
      const y = G - rand() ** 2 * 12
      g.fillStyle = i % 4 ? '#9a7d518a' : '#a74737ab'
      A.leaf(g, x, y, -2.2 + rand() * 2.2, 2.5 + rand() * 7, 1)
    }
  }

  function bellTower(g, x, paint) {
    for (const d of [-173, -113, 113, 173]) {
      const y = Math.abs(d) === 173 ? 107 : 50
      timber(g, x + d - 8, y, 16, G - y, true)
      footing(g, x + d, G, 19, x + d)
    }
    for (const y of [128, 167]) {
      beam(g, [[x - 177, y + 3], [x - 92, y + 1], [x + 30, y + 2], [x + 112, y + 1], [x + 174, y]], 11, x + y, true)
    }
    for (let d = -134; d <= 134; d += 33) {
      inkLine(g, [[x + d - 12, 160], [x + d, 143], [x + d + 13, 160]], 3.6, [74, 80, 75], x + d, .73)
    }
    roof(g, x, 100, 430, paint, 753)
    roof(g, x, 29, 302, paint, 759)
    lantern(g, x - 159, 141); lantern(g, x + 159, 142)
    stoneLantern(g, x - 261, G, .58, paint, true); stoneLantern(g, x + 274, G, .68, paint, true)
  }

  function summitTerrace(g, paint) {
    pine(g, 5340, G + 4, .7, 811, [58, 60, 64])
    wash(g, [[5310, G - 1], [5330, G - 14], [5372, G - 12], [5394, G - 2]], '#655e58')
    stoneLantern(g, 5468, G, .56, paint)
    stoneLantern(g, 5850, G, .47, paint)
    const x = 5960
    const body = [[x - 45, G], [x - 37, G - 17], [x - 32, G - 50], [x - 33, G - 88], [x - 29, G - 108],
      [x - 17, G - 128], [x - 2, G - 123], [x + 16, G - 132], [x + 32, G - 115], [x + 36, G - 91],
      [x + 34, G - 57], [x + 38, G - 21], [x + 44, G]]
    g.save(); path(g, body, true); g.clip()
    g.fillStyle = gradient(g, x - 33, G, x + 41, G, [[0, '#77726d'], [.42, '#a79d8d'], [1, '#585457']])
    g.fillRect(x - 46, G - 134, 93, 136)
    for (let j = 0; j < 20; j++) {
      dab(g, x - 32 + A.hash(j, 792) * 72, G - 131 + A.hash(j, 795) * 132, 15, 18,
        j % 4 ? paint.ink : paint.paper, .16 + A.hash(j, 796) * .27)
    }
    g.restore()
    inkLine(g, body, 4.4, [51, 46, 48], 796, .89)
    dryLine(g, [[x - 31, G - 14], [x - 25, G - 105], [x - 17, G - 121], [x + 18, G - 126], [x + 30, G - 111]], 3.2, PAPER, .77, 803)
    inkLine(g, [[x + 16, G - 126], [x + 6, G - 105], [x + 17, G - 99], [x + 6, G - 83]], 2.9, INK, 806, .64)
    g.fillStyle = '#47464a'; g.font = '600 23px ' + FONT; g.textAlign = 'center'
    g.fillText('止', x - 1, G - 75); g.fillText('戈', x + 1, G - 47)
    inkLine(g, [[x - 13, G - 32], [x + 14, G - 32]], 3.3, [154, 66, 54], 810)
  }

  function terrain(a, i) {
    const from = a.world.areas[i].x, to = a.world.areas[i + 1]?.x ?? a.world.length + 110
    const { paint, detailRes } = a.mountain
    return A.strip(to - from, -70, 479, detailRes, (g, lo, hi) => {
      g.translate(-from, 0)
      if (i === 0) {
        pine(g, 799, G + 5, .88, 701)
        stoneLantern(g, 1196, G, .68, paint)
        stoneLantern(g, 552, G, .61, paint)
      }
      if (i === 2) corridor(g, paint)
      if (i === 3) courtyard(g, paint)
      if (i === 4) bellTower(g, a.world.bell.x, paint)
      if (i === 5) summitTerrace(g, paint)
      if (i !== 1) {
        const start = i === 2 ? BRIDGE[1] - 10 : from
        const end = i === 0 ? BRIDGE[0] + 10 : to
        if (from + lo < end && from + hi > start) {
          terrace(g, start, end, from + lo, from + hi, a.world.areas.at(-1).x)
        }
      }
      if (i === 0) stoneGate(g)
      for (const p of a.world.platforms) if (p.x >= from && p.x < to) A.drawSprite(g, a.mountain.platforms.get(p), p.x, p.y)
    })
  }

  function platform(p, res, paint) {
    return A.sprite(p.w + 26, G - p.y + 20, res, g => {
      if (p.material === 'wood') {
        for (const x of [8, p.w - 15]) timber(g, x, 6, 8, G - p.y - 6)
        beam(g, [[10, 11], [p.w - 9, G - p.y - 1]], 7, p.x + 1)
        beam(g, [[p.w - 8, 11], [13, G - p.y - 1]], 7, p.x + 2)
        planks(g, 0, 0, p.w, 12, p.x, paint)
      } else {
        const depth = G - p.y, tone = [86, 95, 91]
        const lip = [[0, 0], [p.w * .21, .7], [p.w * .51, -.7], [p.w * .79, .2], [p.w, 0]]
        wash(g, [...lip, [p.w - 4, 13], [p.w - 25, 19], [p.w - 33, depth - 4], [18, depth], [21, 18], [7, 14]],
          gradient(g, 0, 0, 0, depth, [[0, '#b7af97'], [.34, pigment(tone)], [1, '#777d71']]))
        for (let i = 0; i < 12; i++) {
          const x = 21 + A.hash(i, p.x) * (p.w - 48), y = 5 + A.hash(i, p.x + 1) * (depth - 13)
          inkLine(g, [[x, y], [x - 5, y + 3], [x + 7, y + 10]], 2.9, i % 4 ? INK : PAPER, i + p.x, .52)
        }
        stroke(g, lip, '#434842c4', 1)
        dryLine(g, lip, 4.2, INK, .92, p.x, 8)
      }
    }, 12, 5)
  }

  function build(resolution, world) {
    const res = Math.min(1.2, resolution(.86))
    const detailRes = Math.max(res, Math.min(1.55, resolution(1)))
    const stamp = brushStamp(861)
    const paint = { ink: tintBrush(stamp, INK), paper: tintBrush(stamp, PAPER) }
    return {
      panels: new Map(), ridges: [], cliffs: [], banks: [], world, res, detailRes, paint,
      platforms: new Map(world.platforms.map(p => [p, platform(p, res, paint)])),
      cloud: A.mist(480, 102),
      sun: A.sun(256, [195, 72, 51], 49),
    }
  }

  function panelCache(a, i) {
    const cache = a.mountain.panels
    if (cache.has(i)) {
      const current = cache.get(i)
      cache.delete(i); cache.set(i, current)
      return current
    }
    const value = {}
    cache.set(i, value)
    if (cache.size > 3) cache.delete(cache.keys().next().value)
    return value
  }

  function mistBelt(g, a, cam, parallax, y, height, spacing, clock, strength, seed) {
    g.save()
    const alpha = g.globalAlpha
    const left = cam.left * parallax, right = left + cam.viewW
    for (let j = Math.floor((left - 400) / spacing); j * spacing < right + 400; j++) {
      const worldX = j * spacing + (A.hash(j, seed) - .5) * spacing * .47
      const drift = Math.sin(clock * .16 + j * 2.1 + seed) * 27
      const lift = Math.sin(clock * .29 + j * 1.7) * 8
      const width = 345 + A.hash(seed, j) * 146
      g.globalAlpha = alpha * strength * (.52 + A.hash(j, seed + 1) * .34)
      g.drawImage(a.mountain.cloud, worldX + drift - width / 2, y + lift, width, height)
    }
    g.restore()
  }

  function waterfall(g, x, y, height, width, clock) {
    g.save(); g.translate(x, y)
    const washColor = gradient(g, 0, 0, 0, height, [[0, '#f0f6e800'], [.12, '#f0f6e85a'], [.37, '#f0f6e899'], [.77, '#e3f0e778'], [1, '#e3f0e700']])
    const left = [[-width * .13, 0], [-width * .23, height * .25], [-width * .18, height * .48],
      [-width * .43, height * .79], [-width * .58, height]]
    const right = [[width * .1, 0], [width * .19, height * .23], [width * .27, height * .50],
      [width * .32, height * .76], [width * .57, height]]
    wash(g, [...left, ...right.reverse()], washColor)
    for (let i = 0; i < 7; i++) {
      const q = (clock * (.43 + i * .024) + i * .23) % 1, py = q * height
      const px = (A.hash(i, 8) - .5) * width * (.4 + q * .7)
      stroke(g, [[px, py], [px + .7, py + 8], [px + 1.5, Math.min(height, py + 15 + q * 19)]],
        A.rgba(PAPER, .44 * Math.sin(q * Math.PI)), .65 + i * .11)
    }
    g.restore()
  }

  function birds(g, a, cam, clock) {
    g.save()
    const left = cam.left * FAR_SCROLL, right = left + cam.viewW
    const start = -420, span = a.world.length * FAR_SCROLL + 1900
    for (let j = 0; j < 6; j++) {
      const size = 6 + A.hash(j, 45) * 5
      const x = start + (clock * 24 + 600 + j * 26) % span
      if (x < left - size || x > right + size) continue
      const y = 130 + Math.sin(clock * .41 + j * .63) * 7 + j * 3
      const wing = Math.sin(clock * 4.4 + j * .95) * size * .4
      g.beginPath(); g.moveTo(x - size, y - wing)
      g.quadraticCurveTo(x - size * .45, y - 4 - wing * .4, x, y)
      g.quadraticCurveTo(x + size * .5, y - 4 - wing * .5, x + size * .9, y - wing + 1)
      g.lineWidth = .85 + A.hash(j, 45) * .55
      g.strokeStyle = '#3b424889'; g.stroke()
    }
    g.restore()
  }

  function paintSky(g, cam, env) {
    g.setTransform(1, 0, 0, 1, 0, 0)
    const rgb = c => pigment(c.map(Math.round))
    g.fillStyle = gradient(g, 0, 0, 0, cam.height, [[0, rgb(env.top)], [.37, rgb(env.middle)], [.82, rgb(env.low)], [1, rgb(env.low)]])
    g.fillRect(0, 0, cam.width, cam.height)
  }

  function background(g, a, cam, clock, env, layer) {
    paintSky(g, cam, env)
    const middle = cam.left + cam.viewW / 2
    const visible = (x, half, parallax) => x + half > cam.left * parallax - 12 && x - half < cam.left * parallax + cam.viewW + 12
    const climb = smooth((middle - 3200) / 2360)
    const sunSize = 60 + 96 * climb
    const sunX = 448 + middle * .32 + 450 * climb
    const sunY = 72 + 47 * climb

    g.save()
    layer(g, cam, FAR_SCROLL)
    g.globalAlpha = .74 + climb * .21
    g.drawImage(a.mountain.sun, sunX - sunSize / 2, sunY - sunSize / 2, sunSize, sunSize)
    g.globalAlpha = 1
    for (let i = 0; i < RIDGES.length; i++) {
      const [x, , width] = RIDGES[i], at = x * FAR_SCROLL + 245
      if (!visible(at, width / 2 + 12, FAR_SCROLL)) continue
      a.mountain.ridges[i] ||= distant(i, a.mountain.res * .79)
      A.drawSprite(g, a.mountain.ridges[i], at, 0)
    }
    mistBelt(g, a, cam, FAR_SCROLL, 169, 120, 267, clock, .52, 17)
    mistBelt(g, a, cam, FAR_SCROLL, 235, 137, 235, clock, .62, 23)
    birds(g, a, cam, clock)

    layer(g, cam, CLIFF_SCROLL)
    for (let i = 0; i < CLIFFS.length; i++) {
      const [x, , width] = CLIFFS[i], at = x * CLIFF_SCROLL + 120
      if (!visible(at, width * 1.25, CLIFF_SCROLL)) continue
      a.mountain.cliffs[i] ||= closeCliff(i, a.mountain.res)
      A.drawSprite(g, a.mountain.cliffs[i], at, 0)
    }
    for (const [x, y, height, width] of FALLS) {
      const at = x * CLIFF_SCROLL + 120
      if (visible(at, width, CLIFF_SCROLL)) waterfall(g, at, y, height, width, clock)
    }
    mistBelt(g, a, cam, CLIFF_SCROLL, 217, 125, 260, clock, .58, 37)
    mistBelt(g, a, cam, CLIFF_SCROLL, 280, 113, 270, clock, .70, 41)
    g.restore()
  }

  function ground(g, a, cam) {
    const left = cam.left - 16, right = cam.left + cam.viewW + 16
    const [bridgeStart, bridgeEnd] = BRIDGE
    if (left <= bridgeEnd && right >= bridgeStart) {
      const { paint, detailRes } = a.mountain
      a.mountain.bridge ||= A.strip(bridgeEnd - bridgeStart, 79, 293, detailRes, ink => {
        ink.translate(-bridgeStart, 0)
        suspension(ink, paint)
      })
      g.save(); g.translate(bridgeStart, 0)
      A.drawStrip(g, a.mountain.bridge, left - bridgeStart, right - left)
      g.restore()
    }
    if (left < 0) terrace(g, -a.world.length, 0, left, 0, Infinity)
    for (let i = 0; i < a.world.areas.length; i++) {
      const from = a.world.areas[i].x, to = a.world.areas[i + 1]?.x ?? a.world.length + 110
      if (to < left || from > right) continue
      const cached = panelCache(a, i)
      cached.floor ||= terrain(a, i)
      g.save(); g.translate(from, 0)
      A.drawStrip(g, cached.floor, left - from, right - left)
      g.restore()
    }
    if (left < bridgeStart + 110 && right > bridgeStart - 168) bridgeAbutment(g, a, bridgeStart, 1, 691)
    if (left < bridgeEnd + 168 && right > bridgeEnd - 110) bridgeAbutment(g, a, bridgeEnd, -1, 694)
    const end = a.world.length + 110
    if (right > end) terrace(g, end, end + a.world.length, end, right, a.world.areas.at(-1).x)
  }

  function pennant(g, x, y, time, wind) {
    const dir = wind.dir || 1, power = wind.power || 0
    stroke(g, [[x, G - 1], [x, y - 6]], '#617071', 2.5)
    const len = 26 + power * 48
    const wave = Math.sin(time * (4 + power * 8) + x * .08) * (4 + power * 9)
    const lift = 16 - power * 21
    g.fillStyle = '#a55341'
    g.beginPath(); g.moveTo(x, y)
    g.bezierCurveTo(x + dir * len * .38, y + lift * .45, x + dir * len * .77, y + lift + wave, x + dir * len, y + lift + wave - 5)
    g.lineTo(x + dir * len * .85, y + lift + wave + 3)
    g.quadraticCurveTo(x + dir * len * .42, y + 17, x, y + 13)
    g.closePath(); g.fill()
    stroke(g, [[x, y + 3], [x + dir * len * .35, y + lift * .4 + 4], [x + dir * len * .7, y + lift + wave + 1]], '#dec3a8', .8)
  }

  function bell(g, bell, clock, time, active) {
    const age = clock - bell.ringAt
    const swing = Math.sin(time * .94) * .026 + Math.sin(age * 13) * Math.exp(-age * 2.1) * bell.dir * .22
    g.save(); g.translate(bell.x, bell.y - 45); g.rotate(swing)
    stroke(g, [[0, -28], [0, 4]], '#535354', 3)
    g.fillStyle = gradient(g, -33, 0, 33, 0, [[0, '#524b3e'], [.33, '#b59c70'], [.64, '#897553'], [1, '#494839']])
    g.strokeStyle = '#4b473e'; g.lineWidth = 1.2
    g.beginPath(); g.moveTo(-14, 0); g.quadraticCurveTo(-27, 6, -30, 37); g.lineTo(-35, 45)
    g.quadraticCurveTo(0, 53, 35, 45); g.lineTo(30, 37); g.quadraticCurveTo(27, 6, 14, 0); g.closePath(); g.fill(); g.stroke()
    for (let j = 0; j < 4; j++) {
      g.beginPath(); g.moveTo(-22 - j * 2, 9 + j * 9); g.quadraticCurveTo(0, 16 + j * 9, 22 + j * 2, 9 + j * 9); g.stroke()
    }
    for (const x of [-14, 0, 14]) stroke(g, [[x, 7], [x * 1.48, 40]], '#c8a77b', .72)
    stroke(g, [[0, 42], [0, 62]], '#4a4944', 2.5)
    if (active) {
      stroke(g, [[-23, 37], [-29, 45], [0, 49], [29, 45], [23, 37]], '#efd09e', 1.5)
      g.fillStyle = '#9b3b31'; g.beginPath(); g.roundRect(-9, 15, 18, 18, 2); g.fill()
      g.fillStyle = '#faebc9'; g.font = '600 11px ' + FONT; g.textAlign = 'center'; g.textBaseline = 'middle'
      g.fillText('击', 0, 25)
    }
    g.restore()
    if (age < 1.15) {
      for (let j = 0; j < 2; j++) {
        const q = Math.max(0, age - j * .11), size = 26 + q * 155
        g.strokeStyle = A.rgba([230, 200, 137], .78 * (1 - age / 1.15)); g.lineWidth = 2.3 - age
        g.beginPath(); g.ellipse(bell.x, bell.y + 4, size, size * .22, 0, 0, TAU); g.stroke()
      }
    }
  }

  function moving(g, a, cam, state, time, reducedMotion) {
    const left = cam.left - 85, right = cam.left + cam.viewW + 85
    const clock = state.clock
    for (const [x, y] of [[2525, 166], [2763, 160], [3020, 183]]) {
      if (x >= left && x <= right) pennant(g, x, y, reducedMotion ? 0 : time, state.wind)
    }
    if (state.bell && state.bell.x >= left && state.bell.x <= right) {
      const active = state.enemies.some(e => e.type === 'shizhen' && e.alive && e.delay <= 0 && Math.abs(e.x - state.bell.x) <= state.bell.radius)
      bell(g, state.bell, clock, reducedMotion ? 0 : time, active && clock - state.bell.ringAt >= 1.6)
    }
    if (reducedMotion) return
    if (left < 4080 && right > 3400) {
      for (let j = 0; j < 16; j++) {
        const q = (time * (.09 + A.hash(j, 87) * .09) + A.hash(j, 19)) % 1
        const x = 3447 + A.hash(j, 38) * 280 + q * 86 + Math.sin(time * 1.7 + j) * 14
        const y = 104 + A.hash(j, 17) * 70 + q * 130
        g.fillStyle = A.rgba(j % 3 ? [160, 118, 64] : [161, 62, 48], .8 * Math.sin(q * Math.PI))
        A.leaf(g, x, y, time * (1 + j * .034) + j, 4 + j % 5, 1.2)
      }
    }
    if (state.wind.phase !== 'calm') {
      const dir = state.wind.dir || 1, power = state.wind.power
      const [lo, hi] = a.world.wind.zone
      g.strokeStyle = A.rgba([245, 242, 226], .14 + .32 * power); g.lineWidth = .9
      g.beginPath()
      for (let j = 0; j < 20; j++) {
        const width = hi - lo + 80
        const phase = time * dir * (28 + power * 175) + A.hash(j, 371) * width
        const x = lo + ((phase % width + width) % width)
        const y = 145 + A.hash(j, 408) * 150
        g.moveTo(x, y); g.quadraticCurveTo(x + dir * 18, y + 3, x + dir * (30 + power * 70), y - 4)
      }
      g.stroke()
    }
  }

  function attackGuides(g, state) {
    for (const enemy of state.enemies) {
      if (!enemy.alive || enemy.delay > 0) continue
      const progress = Math.max(0, 1 - enemy.t / (enemy.teleTotal || 1))
      if (enemy.state === 'mountTell' && enemy.sweep) {
        const hit = R.summitReach(enemy)
        const y = hit.y1 - 5
        g.fillStyle = gradient(g, 0, y - 16, 0, y + 2, [[0, '#a53a2600'], [1, A.rgba([181, 57, 40], .13 + progress * .31)]])
        g.fillRect(hit.x0, y - 16, hit.x1 - hit.x0, 18)
        stroke(g, [[hit.x0, y], [hit.x1, y]], '#e3bd9888', 2.2)
        const from = enemy.facing > 0 ? hit.x0 : hit.x1
        const to = enemy.facing > 0 ? hit.x1 : hit.x0
        stroke(g, [[from, y], [A.lerp(from, to, progress), y]], CINNABAR, 2.1)
      } else if (enemy.state === 'ropeTell') {
        const x = enemy.aimX, y = G - 2
        g.beginPath(); g.ellipse(x, y, 23, 5, 0, 0, TAU)
        g.fillStyle = '#b94c3433'; g.fill()
        g.beginPath(); g.ellipse(x, y, 23, 5, 0, -Math.PI / 2, -Math.PI / 2 + TAU * progress)
        g.strokeStyle = CINNABAR; g.lineWidth = 1.5; g.stroke()
        g.setLineDash([4, 7])
        stroke(g, [[enemy.x, enemy.y - 26], [x, y - 4]], A.rgba([121, 65, 54], .18 + progress * .33), 1.1)
        g.setLineDash([])
      } else if (enemy.state === 'bowTell' && enemy.boltAim) {
        const shot = R.arrowLaunch(enemy, enemy.boltAim)
        const points = Array.from({ length: 8 }, (_, i) => {
          const t = shot.flight * i / 7
          return [shot.x + shot.vx * t, shot.y + shot.vy * t + R.ARROW_GRAVITY * t * t / 2]
        })
        g.setLineDash([3, 8])
        stroke(g, points, A.rgba([162, 66, 42], .16 + progress * .43), 1.2)
        g.setLineDash([])
      }
    }
  }

  globalThis.XiabanInkMountain = { build, background, ground, moving, attackGuides }
})()
