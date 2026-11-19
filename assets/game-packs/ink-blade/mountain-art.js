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
  const FALLS = [[1455, 164, 221, 18], [2270, 122, 247, 18], [5100, 149, 214, 11]]
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
    // Axe-cut shadow faces: pool ink along a broken ridge, drag it down the
    // shale, then lift the brush to expose narrow veins of paper.
    if (rooted) {
      for (let k = 0; k < 7; k++) {
        const px = left + (.12 + .77 * rand()) * w, py = y + rand() * h * .35
        const width = (.045 + rand() * .12) * w, drop = (.29 + rand() * .42) * h
        const face = [[px, py], [px + width, py - 13], [px + width * .74, py + drop * .31],
          [px + width * .33, py + drop * .53], [px - width * .16, py + drop]]
        wash(g, face, gradient(g, px, py, px - 7, py + drop,
          [[0, A.rgba(INK,.30)], [.52,A.rgba(dark,.17)], [1,A.rgba(dark,0)]]))
        for (let j = 0; j < 7; j++) {
          const xx = px + rand() * width, yy = py + rand() * drop * .44
          dryLine(g, [[xx, yy], [xx - 2, yy + 8], [xx - width * .24, yy + 12 + rand() * drop * .21]],
            1.2 + rand() * 2.5, j % 4 ? INK : PAPER, j % 4 ? .39 : .64, seed + k * 31 + j, 4)
        }
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

  const TREES = {
    gate: {
      bark: [53, 54, 44], light: [131, 117, 89], foliage: [[26, 43, 34], [49, 66, 46], [88, 98, 66]],
      width: 27,
      trunk: [[0, 2], [6, -22], [-5, -44], [-30, -58], [-45, -77], [-40, -104], [-67, -125], [-84, -149], [-83, -163]],
      boughs: [
        [[-23,-54],[-11,-66],[11,-72],[38,-88],[58,-87]],
        [[-44,-79],[-61,-80],[-85,-77],[-109,-86],[-123,-90]],
        [[-40,-105],[-22,-113],[-9,-128],[14,-133],[33,-129]],
        [[-63,-122],[-84,-125],[-108,-134],[-134,-132]],
        [[-82,-148],[-64,-157],[-49,-169],[-33,-171]],
        [[-84,-151],[-105,-156],[-120,-165]],
      ],
    },
    ledge: {
      bark: [58, 59, 47], light: [141, 133, 100], foliage: [[44, 62, 54], [67, 86, 71], [107, 118, 91]],
      width: 16,
      trunk: [[0,1],[-5,-13],[3,-27],[28,-36],[51,-51],[65,-67],[91,-72]],
      boughs: [
        [[3,-27],[23,-28],[45,-25],[68,-29]],
        [[29,-37],[37,-53],[55,-63],[72,-59]],
        [[51,-51],[73,-50],[90,-54],[111,-55]],
        [[64,-65],[70,-77],[91,-83],[106,-83]],
      ],
    },
    wind: {
      bark: [61, 66, 57], light: [150, 150, 118], foliage: [[46, 65, 62], [73, 91, 78], [105, 124, 104]],
      width: 15,
      trunk: [[0,1],[4,-19],[15,-39],[36,-60],[57,-73],[86,-90],[115,-98]],
      boughs: [
        [[9,-29],[30,-37],[59,-38],[83,-49],[108,-52]],
        [[34,-58],[38,-79],[51,-89],[67,-96]],
        [[55,-72],[79,-67],[103,-73],[132,-73]],
        [[84,-89],[102,-109],[127,-116],[148,-120]],
      ],
    },
    court: {
      bark: [52, 51, 40], light: [135, 115, 81], foliage: [[24, 43, 29], [45, 67, 42], [100, 111, 68]],
      width: 31,
      trunk: [[0,2],[-8,-18],[-4,-43],[14,-61],[18,-88],[2,-103],[-15,-128],[-12,-152],[9,-170],[16,-192]],
      boughs: [
        [[-7,-38],[-27,-48],[-51,-52],[-72,-68],[-104,-65]],
        [[13,-62],[37,-64],[54,-78],[81,-80],[103,-79]],
        [[18,-88],[8,-89],[-17,-83],[-41,-91],[-67,-96]],
        [[5,-99],[24,-113],[45,-111],[67,-125],[87,-124],[112,-133]],
        [[-13,-128],[-36,-129],[-56,-141],[-84,-145]],
        [[-12,-151],[8,-154],[30,-166],[54,-165]],
        [[9,-172],[-5,-184],[-30,-191],[-55,-192]],
        [[15,-188],[36,-192],[53,-204]],
      ],
    },
    bell: {
      bark: [57, 52, 43], light: [156, 123, 78], foliage: [[78, 56, 41], [143, 91, 52], [181, 131, 69]],
      width: 18, broadleaf: true,
      trunk: [[0,1],[8,-21],[7,-42],[-6,-65],[-12,-91],[-3,-119],[9,-137]],
      boughs: [
        [[7,-43],[-18,-49],[-34,-61],[-61,-66]],
        [[-6,-65],[12,-71],[29,-85],[56,-92]],
        [[-11,-88],[-33,-95],[-48,-112],[-68,-120]],
        [[-4,-113],[16,-120],[36,-133],[60,-131]],
        [[5,-131],[-11,-145],[-24,-154]],
      ],
    },
  }

  function treeCluster(g, x, y, size, palette, airy, broadleaf, brush, rand) {
    const [shadow, midtone, highlight] = palette
    dab(g, x, y - 2, (broadleaf ? 20 : 24) * size, (broadleaf ? 15 : 11) * size, brush, .76)
    const sprays = broadleaf ? 13 : 8
    for (let j = 0; j < sprays; j++) {
      const spread = ((j + rand() * .7) / sprays - .5) * 2
      const px = x + spread * 19 * size
      const py = y + (rand() - .8) * (broadleaf ? 16 : 8) * size
      const axis = -1.55 + spread * 1.24 + airy
      if (broadleaf) {
        const tipX = px + Math.cos(axis) * 10 * size, tipY = py + Math.sin(axis) * 10 * size
        stroke(g, [[x, y + size * 2], [px, py], [tipX, tipY]], A.rgba(shadow, .65), .65)
        for (let n = 0; n < 5; n++) {
          const t = n / 5
          g.fillStyle = A.rgba(n === 3 ? highlight : n % 3 ? midtone : shadow, .66 + rand() * .27)
          A.leaf(g, A.lerp(px, tipX, t), A.lerp(py, tipY, t), axis + (n % 2 ? .96 : -.86),
            (4 + rand() * 6) * size, (1.7 + rand()) * size)
        }
      } else {
        g.fillStyle = A.rgba(shadow, .90)
        A.leaf(g, px, py + 2, axis, 10 * size, 3.3 * size)
        for (let n = 0; n < 11; n++) {
          const angle = axis + (n / 10 - .5) * 2.76 + (rand() - .5) * .21
          g.fillStyle = A.rgba(n % 7 === 0 ? highlight : n % 3 ? shadow : midtone, .7 + rand() * .25)
          A.leaf(g, px, py, angle, (5.5 + rand() * 8) * size, (.42 + rand() * .67) * size)
        }
      }
    }
  }

  function characterTree(g, x, y, scale, seed, tree, paint) {
    const { trunk, width, bark, light, foliage, boughs, broadleaf } = tree
    const rand = A.rng(seed)
    const shadow = mix(bark, INK, .55)
    g.save(); g.translate(x, y); g.scale(scale, Math.abs(scale))
    const roots = [[-30, 2], [-12, -4], [-5, -8], [4, -6], [13, -1], [34, 3]]
    g.fillStyle = pigment(bark)
    A.ribbon(g, roots, t => 1 + Math.sin(t * Math.PI) * width * .53)
    g.fillStyle = gradient(g, -16, 0, 13, 0, [[0, pigment(light)], [.44, pigment(bark)], [1, pigment(shadow)]])
    A.ribbon(g, trunk, t => 1.25 + width * (1 - t) ** 1.12)
    dryLine(g, trunk, width * .6, light, .78, seed, 8)
    dryLine(g, roots, width * .27, light, .77, seed + 7, 6)
    for (let j = 1; j < Math.min(trunk.length, 8); j++) {
      const [bx, by] = trunk[j]
      const spread = Math.max(2.5, width * (.44 - .051 * j))
      const ridges = [[bx - spread, by + 8], [bx - spread*.34, by + 4], [bx+spread*.09, by - 1]]
      axeStroke(g, bx - spread*.67, by+6, spread, -8, spread*.52, shadow, .76, rand)
      dryLine(g, ridges, 1.8, light, .84, seed+j, 4)
    }
    const canopy = []
    for (const points of boughs) {
      const direction = Math.sign(points.at(-1)[0] - points[0][0])
      g.fillStyle = pigment(shadow)
      A.ribbon(g, points, t => .62 + width * .30 * (1 - t) ** .93)
      dryLine(g, points, width * .13, light, .68, Math.floor(rand() * 7600), 5)
      for (let j = 2; j < points.length; j++) {
        const [bx, by] = points[j]
        const reach = direction * (5 + rand()*14), rise = 6 + rand()*12
        inkLine(g, [[bx-direction*5, by+2],[bx+reach*.44,by-rise*.65],[bx+reach,by-rise]], 2.7,
          bark, Math.floor(rand()*7600))
        canopy.push([bx+reach, by-rise, .73+rand()*.48, direction*.27])
        if (j % 2 === 0) {
          const fork = bx-direction*(8+rand()*6)
          inkLine(g, [[bx,by],[bx-direction*4,by-10],[fork,by-14]], 2, bark, Math.floor(rand()*7600))
          canopy.push([fork, by-14, .65+rand()*.3, -direction*.2])
        }
      }
    }
    const density = tree === TREES.court ? 1.27 : tree === TREES.gate ? 1.13 : 1
    for (const [bx, by, size, angle] of canopy) {
      treeCluster(g, bx, by, size * density, foliage, angle, broadleaf, paint.ink, rand)
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

  const GROUND_COLOR = {
    gate: { stone: [109, 109, 96], grass: [57, 76, 54], bloom: [137, 95, 125], bend: .16 },
    bridge: { stone: [96, 113, 110], grass: [64, 90, 73], bloom: [224, 219, 182], bend: .44 },
    wind: { stone: [102, 114, 104], grass: [81, 99, 71], bloom: [183, 155, 104], bend: .79, seedHeads: true },
    court: { stone: [118, 118, 91], grass: [65, 86, 48], bloom: [168, 110, 108], bend: -.19 },
    bell: { stone: [111, 103, 86], grass: [90, 93, 60], bloom: [203, 164, 97], bend: .22 },
    summit: { stone: [124, 114, 105], grass: [77, 88, 67], bloom: [217, 209, 178], bend: -.35 },
  }

  // Roots sit back from the fighting edge. Heights and group sizes vary between
  // sheltered stone seams, exposed gravel and swept ground.
  const GROUND_PATCHES = [
    [[106,34,.92], [263,43,1.08], [431,20,.72], [526,53,1.17], [654,31,.95], [755,49,1.26],
      [866,24,.78], [922,49,1.23], [1046,36,.73], [1116,52,1.18], [1224,26,.9], [1327,44,1.1]],
    [],
    [[2432,40,.95], [2499,24,.85], [2611,39,1.16], [2690,51,1.23], [2784,24,.71], [2847,45,1.31],
      [2935,33,.86], [3009,54,1.10], [3095,32,.92], [3157,49,1.23], [3233,25,.75]],
    [[3358,39,.96], [3436,23,.81], [3515,48,1.28], [3602,31,.84], [3688,49,1.2], [3765,40,.63],
      [3884,49,.72], [3968,41,.88], [4072,27,1.0], [4152,51,1.22], [4215,24,.68]],
    [[4280,41,.96], [4374,24,.76], [4482,45,1.19], [4559,30,.84], [4642,51,1.22], [4742,27,.75],
      [4817,45,.86], [4917,33,.98], [4993,49,1.14], [5104,26,.96], [5180,43,1.21]],
    [[5272,16,.81], [5366,18,.88], [5431,16,.8], [5498,16,.66], [5564,18,.55], [5641,14,.45],
      [5747,15,.48], [5825,14,.57], [5887,17,.68], [5935,15,.79], [6004,18,.87], [6080,15,.75], [6150,16,.71]],
  ]
  const GROUND_BY_AREA = [GROUND_COLOR.gate, GROUND_COLOR.bridge, GROUND_COLOR.wind,
    GROUND_COLOR.court, GROUND_COLOR.bell, GROUND_COLOR.summit]

  function groundStone(g, x, y, width, height, tone, paint, rand) {
    const dark = mix(tone, INK, .60), light = mix(tone, PAPER, .49)
    const slope = (rand() - .5) * .46
    const foot = [x - width * (.44 + rand() * .14), y - height * .07]
    const ledge = [x - width * (.31 + rand() * .21), y - height * (.42 + rand() * .3)]
    const crown = [x + width * slope, y - height * (.86 + rand() * .14)]
    const shoulder = [x + width * (.17 + rand() * .25), y - height * (.43 + rand() * .38)]
    const side = [x + width * (.46 + rand() * .09), y - height * (.10 + rand() * .15)]
    const base = [x + width * (.13 + rand() * .27), y + height * .16]
    const split = [x + width * (slope - .15), y - height * (.23 + rand() * .22)]
    const seed = Math.floor(rand() * 9000)
    const crownEdge = chippedEdge([foot, ledge, crown, shoulder], seed)
    dab(g, x + width * .12, y + 1, width * .78, height * .43, paint.ink, .43)
    wash(g, [...crownEdge, side, base], pigment(dark))
    wash(g, [...crownEdge, split], gradient(g, x, y - height, x, y,
      [[0, pigment(light)], [.60, pigment(tone)], [1, pigment(mix(tone, dark, .48))]]))
    wash(g, [split, shoulder, side, base], A.rgba(tone, .63))
    dryLine(g, [ledge, crown, shoulder], 1.75, light, .82, seed + 1, 4)
    for (let i = 0; i < 4; i++) {
      const px = A.lerp(ledge[0], shoulder[0], .20 + i * .16)
      const py = y - height * (.34 + rand() * .18)
      axeStroke(g, px, py, width * .13, height * .17, 1.2 + rand() * 1.7, dark, .43, rand)
    }
  }

  function groundBlades(g, x, y, height, palette, rand) {
    const { grass, bend } = palette
    const shaded = mix(grass, INK, .49), sunlit = mix(grass, PAPER, .37)
    for (let j = 0; j < 14; j++) {
      const spread = (j / 13 - .5) * 2, root = x + (rand() - .5) * height * .48
      const h = height * (.37 + rand() * .67)
      const lean = h * (spread * .72 + bend)
      const points = [[root, y], [root + lean * .24, y - h * .56], [root + lean * .69, y - h], [root + lean, y - h * .91]]
      g.fillStyle = A.rgba(j % 6 === 0 ? sunlit : j % 3 ? grass : shaded, .68 + rand() * .31)
      const width = .8 + height * .13 * rand()
      A.ribbon(g, points, t => width * (1 - t) + .10)
    }
  }

  function groundBlooms(g, x, y, height, palette, rand) {
    const { bloom, grass, bend, seedHeads } = palette
    const flowers = 3 + Math.floor(rand() * 4)
    for (let j = 0; j < flowers; j++) {
      const root = x + (rand() - .5) * height * .82 + j * .7, rise = height * (.57 + rand() * .47)
      const end = root + rise * ((rand() - .5) * .6 + bend * .73)
      const top = y - rise, size = height * (.073 + rand() * .048)
      stroke(g, [[root, y], [root + (end - root) * .38, y - rise * .52], [end, top]], A.rgba(grass, .92), .65 + size * .18)
      g.fillStyle = A.rgba(grass, .76)
      A.leaf(g, root + (end - root) * .29, y - rise * .37, j % 2 ? -2.62 : -.64, rise * .34, size * .79)
      if (seedHeads) {
        g.fillStyle = pigment(bloom)
        A.leaf(g, end, top + 1, -1.06, size * 3.6, size * .86)
        g.fillStyle = A.rgba(PAPER, .71)
        A.leaf(g, end + .5, top - 1, -1.09, size * 2.5, size * .23)
        continue
      }
      for (let petal = 0; petal < 5; petal++) {
        const angle = petal * TAU / 5 + j * .37
        g.fillStyle = pigment(petal % 3 ? bloom : mix(bloom, PAPER, .49))
        A.leaf(g, end, top, angle, size * (1.14 + rand() * .43), size * .72)
      }
      g.fillStyle = pigment(mix(bloom, INK, .24))
      g.fillRect(end - size * .27, top - size * .25, size * .62, size * .56)
    }
  }

  function groundPatch(g, x, y, size, palette, paint, seed) {
    const rand = A.rng(seed + Math.round(x) * 31)
    const stone = palette.stone, direction = rand() < .47 ? -1 : 1
    const root = x + direction * (17 + rand() * 8) * size
    const meadow = rand() < .34
    const width = (meadow ? 10 + rand() * 13 : 20 + rand() * 25) * size
    const shadow = mix(stone, INK, .31)
    const seam = [[x - 41 * size, y + size * 3], [x - 20 * size, y + size], [x + 8 * size, y + size * 2],
      [x + 20 * size, y - size], [x + 31 * size, y + size]]
    dryLine(g, seam, 2.1 * size, shadow, .52, seed + 41, 4)
    for (let j = 0; j < 9; j++) {
      const drift = (rand() - .5) * 91 * size, px = x + drift
      const py = y + (rand() - .28) * 12 * size
      const w = (2 + rand() ** 2 * 10) * size
      inkPebble(g, px, py, w, w * (.24 + rand() * .2), shadow, rand)
    }
    groundStone(g, x - 8 * size, y - 1.5 * size, width, width * (.17 + rand() * .22), stone, paint, rand)
    groundStone(g, x + 11 * size, y + size, width * .46, width * (.11 + rand() * .07), mix(stone, PAPER, .11), paint, rand)
    groundBlades(g, root, y, (13 + rand() * 10) * size, palette, rand)
    groundBlades(g, x - 20 * size, y + size * 3, (6 + rand() * (meadow ? 11 : 6)) * size, palette, rand)
    if (meadow) {
      groundBlades(g, x + 5 * size, y - size * 2, (9 + rand() * 9) * size, palette, rand)
    }
    if (size > .57 && rand() > .15) {
      groundBlooms(g, root + direction * 10 * size, y + 1, (19 + rand() * 9) * size, palette, rand)
    }
  }

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

    // Rain threads emerge from a shelf of moss and turn into heavier ink in the joints.
    for (const [x, y, span] of [[-119,-158,29],[-91,-187,25],[-103,-62,31],[67,-176,22],[79,-41,29]]) {
      const joint = [[x,y],[x+span*.3,y-2],[x+span*.62,y+1],[x+span,y-1]]
      inkLine(g, joint, 3.8, [56,61,49], Math.round(x + 631))
      for (let k=0;k<34;k++) {
        const t=rand(), xx=x+t*span, yy=y-1.5-rand()**2*5
        g.fillStyle = k%4 ? '#58634c' : '#81906b'
        A.leaf(g,xx,yy,-2.8+rand()*2.5,1.5+rand()*4,1+rand())
        if (k%8===0) axeStroke(g,xx,yy+2,-2,6+rand()*14,3,[74,75,59],.45,rand)
      }
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
    sprites[side] ||= A.sprite(280, 236, a.mountain.detailRes, ink => {
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
      ink.globalCompositeOperation = 'source-over'
      characterTree(ink, 11, 2, side > 0 ? .75 : .62, seed, TREES.ledge, a.mountain.paint)
      groundPatch(ink, -18, 18, .68, GROUND_COLOR.bridge, a.mountain.paint, seed)
      groundPatch(ink, 39, 9, .73, GROUND_COLOR.bridge, a.mountain.paint, seed + 11)
      groundPatch(ink, 73, 9, .46, GROUND_COLOR.bridge, a.mountain.paint, seed + 19)
    }, 168, 116)
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
      for (const d of [0, 5, 10]) {
        stroke(g, [[x-11,G-48+d],[x-3,G-45+d],[x+8,G-47+d]], '#584435', 3.5)
        stroke(g, [[x-9,G-48+d],[x-3,G-46+d],[x+5,G-47+d]], '#c9b38d', 1.1)
      }
    }
    const cable = Array.from({ length: 22 }, (_, i) => {
      const t = i / 21
      return [A.lerp(1510, 2220, t), G - 211 + 116 * t * (1 - t) + 5 * t]
    })
    beam(g, [[1441, 93], cable[0]], 3.3, 697)
    beam(g, [cable.at(-1), [2332, 93]], 3.3, 698)
    stroke(g, cable, '#483a2cc4', 1.2)
    dryLine(g, cable, 3.1, [61, 47, 37], .85, 690, 5)
    // Twisted hemp catches light on one side. Cloth knots mark the loaded suspension ties.
    for (let i = 1; i < cable.length; i++) {
      const [x,y] = cable[i-1], [xx,yy] = cable[i]
      for (let j=0;j<8;j++) {
        const px=A.lerp(x,xx,j/8), py=A.lerp(y,yy,j/8)
        stroke(g, [[px,py+.8],[px+2.4,py-1.1]], '#b49b74', 1.05)
      }
      if (i%4!==2) continue
      const deck = G + 4
      stroke(g, [[xx,yy+1],[xx-1,yy+32],[xx+2,deck]], '#76634fc2', 1.5)
      stroke(g, [[xx-2,deck-2],[xx+2,deck],[xx+6,deck-2]], '#534838', 2.1)
      for (let k=-2;k<=2;k++) stroke(g, [[xx+k*1.8,yy-1.8],[xx+k*1.8-1,yy+3]], '#433e34', 1.4)
      wash(g, [[xx+3,yy+2],[xx+5,yy+3],[xx+9,yy+14],[xx+6,yy+12],[xx+4,yy+16]], '#8c5844')
    }
    planks(g, from, G, to - from, 18, 209, paint)
    for (const x of [1524, 1694, 1789, 2026, 2148, 2281]) {
      const rand = A.rng(x * 17)
      for (let j = 0; j < 9; j++) {
        const px = x + rand() * 41, py = G + .6 + rand() * 5
        g.fillStyle = A.rgba([137, 119, 78], .66 + rand() * .24)
        A.leaf(g, px, py, (rand() - .5) * 2, 2 + rand() * 5, .55 + rand() * .68)
      }
    }
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
      g.save(); g.translate(x, 0)
      // Worn cinnabar is strongest under the roof and in the deep edge of the post.
      wash(g,[[-7,103],[4,104],[4,161],[1,173],[2,194],[-2,207],[-4,195],[-7,210]],'#844f40')
      dryLine(g,[[-5,110],[-4,140],[-5,179],[-5,201]],2.7,[194,157,111],.76,x,5)
      for (const y of [141,149,278]) {
        wash(g,[[-8,y],[-1,y+1],[8,y-1],[7,y+3],[-7,y+4]],'#45483f')
        stroke(g,[[-7,y],[-1,y+1],[7,y]],'#b4a17b',1)
      }
      g.restore()
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
    characterTree(g, 3479, G + 2, -.51, 718, TREES.ledge, paint)
    characterTree(g, 3553, G + 2, 1.06, 707, TREES.court, paint)
    stoneLantern(g, 3422, G, .74, paint)
    stoneLantern(g, 4067, G, .62, paint)
    wash(g, [[3893, G - 18], [3903, G - 27], [3916, G - 32], [3977, G - 30], [4006, G - 32], [4020, G - 20],
      [4011, G - 15], [3989, G - 14], [3947, G - 17], [3910, G - 15]], '#a6a08a')
    wash(g, [[3928, G - 16], [3939, G - 16], [3939, G], [3926, G], [3924, G - 6]], '#77735e')
    wash(g, [[3983, G - 16], [3994, G - 16], [3997, G], [3981, G]], '#77735e')
    inkLine(g, [[3894, G - 18], [3912, G - 26], [3948, G - 25], [4008, G - 27], [4020, G - 20]], 3, [62, 59, 49], 721, .75)
    dryLine(g, [[3908, G - 20], [3944, G - 22], [3980, G - 20], [4006, G - 22]], 4.3, PAPER, .71, 719)
    const rand = A.rng(717)
    for (const [x,y] of [[3909,G-21],[3963,G-24],[3994,G-24]]) {
      const fault=[[x,y],[x+6,y-1],[x+12,y+1],[x+17,y],[x+22,y+2]]
      dryLine(g,fault,2.8,[65,64,55],.81,x,5)
    }
    for (let i = 0; i < 39; i++) {
      const x = (i < 20 ? 3457 : 4018) + rand() * 120
      const y = G - rand() ** 2 * 12
      g.fillStyle = i % 4 ? '#9a7d518a' : '#a74737ab'
      A.leaf(g, x, y, -2.2 + rand() * 2.2, 2.5 + rand() * 7, 1)
    }
  }

  function lanternWash(g, x, y, rx, ry, strength) {
    g.save(); g.translate(x, y); g.scale(rx, ry)
    const glow = g.createRadialGradient(0, -.12, 0, 0, -.12, 1)
    glow.addColorStop(0, A.rgba([255, 215, 143], strength))
    glow.addColorStop(.37, A.rgba([243, 180, 101], strength * .63))
    glow.addColorStop(.71, A.rgba([193, 135, 72], strength * .19))
    glow.addColorStop(1, '#d4a05a00')
    g.fillStyle = glow
    g.fillRect(-1, -1.12, 2, 2)
    g.restore()
  }

  function lanternGround(g, x, paint) {
    const rand = A.rng(7642)
    for (const light of [x - 156, x + 156]) {
      lanternWash(g, light, G + 14, 112, 23, .66)
      for (let j = 0; j < 19; j++) {
        const px = light + (rand() - .5) * 129, y = G + 3 + rand() ** 1.5 * 32
        const span = 4 + rand() * 18
        dab(g, px, y, span * .63, 2 + rand() * 2.6, paint.paper, .35)
        axeStroke(g, px, y, span, (rand() - .53) * 2, 1.35, [171, 120, 65], .47, rand)
      }
      wash(g, [[light - 21,G+1],[light-12,G+1],[light-40,G+31],[light-59,G+35]], '#38362738')
    }
  }

  function bellTower(g, x, paint) {
    for (const side of [-1, 1]) {
      lanternWash(g, x + side * 153, 205, 104, 125, .66)
      lanternWash(g, x + side * 115, 167, 80, 31, .42)
    }
    lanternWash(g, x, 206, 100, 63, .34)
    // The two side aisles funnel the eye toward the bronze bell in the open bay.
    for (const side of [-1, 1]) {
      g.save(); g.translate(x, 0); g.scale(side, 1)
      const edge = [[143, 55], [208, 54], [298, 42], [372, 22], [442, -5]]
      wash(g, [[140, -65], [444, -65], ...edge.slice().reverse()], '#303b3a')
      dryLine(g, edge, 7, [194, 183, 154], .85, 3061)
      for (let i = 0; i < 26; i++) {
        const t = i / 26, xx = 150 + 280 * t, yy = 54 - 58 * t * t
        inkLine(g, [[xx + 9, yy - 27], [xx + 5, yy - 11], [xx, yy]], 3.7, [132, 141, 129], 3100 + i, .71)
      }
      g.restore()
    }
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
    beam(g, [[x-35,144],[x-23,154],[x-7,157],[x+7,157],[x+23,154],[x+35,144]],5,x+140,true)
    for(const side of [-1,1]) {
      inkLine(g,[[x+side*31,145],[x+side*22,143],[x+side*16,148],[x+side*17,151]],3.3,[82,68,47],x+side)
    }
    roof(g, x, 100, 430, paint, 753)
    roof(g, x, 29, 302, paint, 759)
    lantern(g, x - 159, 141); lantern(g, x + 159, 142)
    stoneLantern(g, x - 261, G, .58, paint, true); stoneLantern(g, x + 274, G, .68, paint, true)
  }

  // A close pine silhouetted against the open summit. Model the pads with
  // forks and needle fans so the sky still shows between the loaded brushes.
  function summitPine(g, paint) {
    const rand = A.rng(3817)
    const shaded = [24, 38, 33], jade = [55, 78, 60], tips = [104, 114, 78]
    function limb(points, width) {
      g.fillStyle = '#383931'
      A.ribbon(g, points, t => .65 + width * (1 - t) ** .83)
      dryLine(g, points, width * .42, [155, 129, 92], .73, Math.floor(rand() * 7000), 5)
    }
    function tuft(x, y, size, sign) {
      const fans = 7 + Math.floor(rand() * 5)
      dab(g, x, y - 2, 23 * size, 13 * size, paint.ink, .98)
      for (let f = 0; f < fans; f++) {
        const side = (f + rand() * .7) / fans * 2 - 1
        const px = x + side * size * 19, py = y - (1 - side * side) * 4 * size + (rand() - .5) * 5
        const axis = -Math.PI * .48 + side * .96 + sign * .22
        const length = (8 + rand() * 5) * size
        g.fillStyle = A.rgba(shaded, .94)
        A.leaf(g, px, py + 2, axis, length + 1, 3.8 * size)
        for (let n = 0; n < 13; n++) {
          const t = n / 12
          const a = axis + (t - .5) * 2.6 + (rand() - .5) * .18
          g.fillStyle = A.rgba(n % 5 ? shaded : jade, .72 + rand() * .27)
          A.leaf(g, px, py, a, length * (.58 + rand() * .72), (.5 + rand() * .52) * size)
        }
        g.fillStyle = A.rgba(tips, .7)
        A.leaf(g, px - 1, py - 2, axis - .55, length * .67, .65 * size)
      }
    }
    g.save(); g.translate(5352, G + 1)
    limb([[-18, 0], [-4, -7], [7, -6], [19, -2], [33, -4]], 6)
    const trunk = [[0, 1], [3, -13], [-5, -33], [-20, -48], [-27, -67], [-21, -82],
      [-31, -104], [-28, -122], [-15, -131], [-10, -148], [3, -159], [26, -166]]
    limb(trunk, 18)
    dryLine(g, [[-3, -3], [0, -17], [-8, -31], [-22, -48], [-25, -64]], 4.1,
      [209, 180, 132], .88, 3781, 4)
    const boughs = [
      [[-27, -72], [-43, -69], [-62, -78], [-81, -76], [-98, -83]],
      [[-26, -104], [-47, -107], [-55, -120], [-76, -124], [-84, -132]],
      [[-5, -32], [14, -47], [35, -48], [53, -63], [77, -67]],
      [[-24, -80], [-3, -92], [25, -90], [55, -101], [86, -98], [113, -107], [131, -111]],
      [[-26, -121], [-9, -118], [4, -132], [24, -136], [45, -146], [66, -146]],
      [[-10, -148], [-16, -158], [-12, -171], [4, -174], [15, -180]],
      [[-9, -149], [4, -158], [26, -164], [44, -162], [56, -169]],
    ]
    for (const bough of boughs) {
      const dir = Math.sign(bough.at(-1)[0] - bough[0][0])
      limb(bough, 6.7)
      for (let j = 2; j < bough.length; j++) {
        const [x, y] = bough[j], rise = 5 + rand() * 7
        const tip = x + dir * (5 + rand() * 7)
        limb([[x - dir * 6, y + 3], [x + dir, y - rise * .65], [tip, y - rise]], 2.7)
        // A short fork above the bough carries a second rank, behind the lit needle tips.
        if (j < bough.length - 1) {
          const fork = x - dir * (5 + rand() * 8), height = rise + 7 + rand() * 10
          limb([[x - dir * 7, y], [fork - dir * 4, y - height * .58], [fork, y - height]], 2.2)
          tuft(fork, y - height, .87 + rand() * .35, -dir)
        }
        tuft(tip, y - rise, .9 + rand() * .45, dir)
        if (j % 2 === 0) tuft(x - dir * 5, y - 3, .73 + rand() * .26, -dir)
      }
    }
    g.restore()
  }

  function summitSapling(g, x, size, lean, seed) {
    const rand = A.rng(seed)
    g.save(); g.translate(x, G); g.scale(lean * size, size)
    const stem = [[0, 1], [2, -11], [-3, -22], [1, -30], [10, -41], [9, -52]]
    g.fillStyle = '#504f41'; A.ribbon(g, stem, t => 4.3 * (1 - t) + .5)
    for (let j = 0; j < 7; j++) {
      const [sx, sy] = stem[1 + Math.floor(j * .62)]
      const dir = j % 2 ? -1 : 1
      const x1 = sx + dir * (7 + 5 * rand()), y1 = sy - 8 - 3 * rand()
      const x2 = x1 + dir * (7 + 9 * rand()), y2 = y1 - 4 - 6 * rand()
      stroke(g, [[sx, sy], [x1, y1], [x2, y2]], '#666551', 1.45)
      for (let k = 0; k < 9; k++) {
        const t = k / 9, px = A.lerp(x1, x2, t), py = A.lerp(y1, y2, t)
        const angle = (dir > 0 ? -.6 : Math.PI + .6) + (k % 2 ? -.75 : .65) + (rand() - .5) * .55
        g.fillStyle = k % 4 === 0 ? '#808163' : k % 3 ? '#495849' : '#666e51'
        A.leaf(g, px, py, angle, 3.8 + rand() * 4.2, 1.4 + rand() * .9)
      }
    }
    g.restore()
  }

  function summitTerrace(g, paint) {
    wash(g, [[5310, G - 1], [5330, G - 14], [5372, G - 12], [5394, G - 2]], '#655e58')
    summitPine(g, paint)
    stoneLantern(g, 5468, G, .56, paint)
    stoneLantern(g, 5850, G, .47, paint)
    for (const [x, size, lean] of [[5286, .57, -1], [5425, .66, 1], [5821, .48, -1], [6014, .82, 1], [6108, .51, 1]]) {
      summitSapling(g, x, size, lean, x)
    }

    // A pale, split front plane carries the inscription. Load the side and foot with ink.
    g.save(); g.translate(5960, G); g.scale(.79, .79)
    const body = [[-46, 1], [-36, -24], [-34, -77], [-31, -105], [-24, -113],
      [-19, -128], [-5, -126], [12, -133], [23, -129], [34, -111], [32, -94],
      [38, -75], [35, -28], [43, -11], [46, 1]]
    g.save(); path(g, body, true); g.clip()
    g.fillStyle = gradient(g, -42, -114, 40, -36, [[0, '#a59d86'], [.39, '#d1c6a9'], [.68, '#b7ad97'], [1, '#867d70']])
    g.fillRect(-47, -135, 95, 138)
    const seam = [[13, -132], [21, -111], [16, -98], [24, -74], [23, -47], [29, -22], [24, 1]]
    wash(g, [...seam, [48, 3], [48, -134]], '#756f64')
    const rand = A.rng(806)
    for (let k = 0; k < 54; k++) {
      const x = -43 + rand() * 88, y = -132 + rand() * 134
      const edge = x < -18 || x > 17 || y > -32
      dab(g, x, y, 6 + rand() * 17, 4 + rand() * 12, k % 4 ? paint.ink : paint.paper, edge ? .30 : .085)
      if (edge) axeStroke(g, x, y, -3 - rand() * 7, 5 + rand() * 14, 2 + rand() * 6, INK, .44, rand)
    }
    dryLine(g, seam, 4.5, INK, .81, 886, 7)
    dryLine(g, body.slice(1, 9), 4.1, PAPER, .95, 889, 7)
    g.restore()
    inkLine(g, body.slice(0, 6), 3.2, [70, 65, 57], 796)
    dryLine(g, body.slice(8), 3.8, INK, .86, 804, 7)

    // 止 and 戈 share a carved edge, with varying pressure on each individual stroke.
    const strokes = [
      [[-4,-101],[-3,-97],[-3.5,-76]], [[-3,-89],[3,-90],[10,-91]],
      [[-14,-91],[-13,-87],[-13,-76]], [[-20,-75],[-14,-74],[-4,-75],[14,-76]],
      [[-18,-56],[-13,-55],[0,-57],[13,-60]], [[-5,-68],[-3,-64],[0,-53],[5,-43],[11,-37],[15,-36],[16,-43]],
      [[8,-55],[6,-51],[0,-45],[-9,-40],[-15,-38]], [[5,-68],[8,-66],[11,-63]],
    ]
    for (const [j, line] of strokes.entries()) {
      stroke(g, line.map(([px, py]) => [px + .85, py + .9]), '#e2d4b3', 2.8)
      g.fillStyle = '#343934'
      A.ribbon(g, line, t => 1.4 + 2.9 * Math.sin(.55 + t * 2.48))
      dryLine(g, line, 3.4, INK, .82, 860 + j, 4)
    }
    wash(g, [[-7,-26],[1,-26],[1,-19],[-7,-18]], '#925846')
    dryLine(g, [[-6,-22],[-3,-24],[-2,-20],[0,-22]], 1.1, PAPER, .85, 880, 3)
    for (let k = 0; k < 32; k++) {
      const x = -33 + rand() * 70, y = -3 - rand() ** 2 * 16
      g.fillStyle = k % 3 ? '#59604a' : '#7e8061'
      A.leaf(g, x, y, rand() * Math.PI, 1 + rand() * 4, 1.3)
    }
    g.restore()
  }

  function chapterScenery(g, i, paint) {
    const rand = A.rng(2080 + i)
    if (i === 0) {
      // The split stele is partly buried in the scree, with seal ink surviving in its cuts.
      g.save(); g.translate(1265, G); g.rotate(.085)
      const slab = [[-35,2],[-30,-87],[-22,-118],[5,-127],[29,-109],[35,-23],[43,3]]
      wash(g, slab, '#716e5c'); dryLine(g, slab.slice(0,5), 4, INK, .87, 1781, 7)
      for (let j=0;j<45;j++) {
        const x=-23+rand()*49, y=-101+rand()*94
        dryLine(g,[[x,y],[x-3,y+4],[x-6,y+11]],2.4,j%3?INK:PAPER,.37,173+j,4)
      }
      g.fillStyle='#383d35'; g.textAlign='center'; g.font='600 21px '+FONT
      g.fillText('問',0,-76); g.fillText('山',0,-48)
      inkLine(g,[[-13,-27],[11,-29]],3.5,[129,56,43],199)
      g.restore()
    } else if (i === 1) {
      // A timber winding drum above the chasm; its basket hangs below the walk.
      g.save();g.translate(1810,G)
      beam(g,[[-32,0],[-18,-111],[32,-116]],7,1310)
      beam(g,[[-27,-65],[10,-113]],5,1311)
      g.strokeStyle='#4d4437';g.lineWidth=4;g.beginPath();g.arc(20,-110,13,0,TAU);g.stroke()
      for(let j=0;j<6;j++){const q=j*Math.PI/3;stroke(g,[[20,-110],[20+12*Math.cos(q),-110+12*Math.sin(q)]],'#574c3c',1.8)}
      stroke(g,[[31,-111],[34,32]],'#544a3c',1.4)
      wash(g,[[18,33],[51,31],[47,48],[24,50]],'#75644e')
      for(let j=0;j<5;j++)stroke(g,[[21+j*6,33],[25+j*5,48]],'#ae9a77',1)
      g.restore()
    } else if (i === 2) {
      for (const x of [2547, 2800, 3010]) {
        const y = 115 + rand()*17, n = 37 + rand()*25
        const silk=[[x,y],[x+15,y+1],[x+13,y+n],[x+9,y+n-8],[x+6,y+n+4],[x+5,y+n-6],[x+1,y+n]]
        wash(g,silk,'#735446');dryLine(g,silk.slice(1,5),2.3,PAPER,.5,x,5)
        for(let j=0;j<6;j++)stroke(g,[[x+2+j*1.8,y+4],[x+2+j*1.8,y+n-10]],'#bea37a70',.55)
      }
    } else if (i === 3) {
      // An actual opening in the old lime wall frames the distant range.
      g.save();g.translate(3833,G)
      g.save()
      g.beginPath();g.arc(8,-37,53,0,TAU);g.clip()
      g.fillStyle = gradient(g,0,-88,0,1,[[0,'#eadcbb32'],[.57,'#eadcb277'],[1,'#d0c49eeb']])
      g.fillRect(-50,-96,116,111)
      wash(g,[[-16,4],[-4,-16],[12,-29],[24,-27],[18,-16],[25,2]],'#b2a68f')
      for (const [x,y,span] of [[-16,-2,46],[-7,-9,31],[1,-16,22],[9,-23,14]]) {
        dryLine(g,[[x,y],[x+span*.58,y-.6],[x+span,y+1]],2.4,[94,91,74],.61,713+x,5)
      }
      characterTree(g,37,-9,-.53,814,{
        ...TREES.bell, foliage: [[103,55,43],[161,76,55],[195,113,72]],
      },paint)
      pine(g,-34,-10,.19,3172,[120,127,99])
      g.restore()
      g.beginPath();g.rect(-147,-94,294,96);g.arc(8,-37,53,0,TAU,true)
      g.fillStyle='#c1b9a1';g.fill('evenodd');g.clip('evenodd')
      // The ring is masked by the actual opening. Radial joints turn across its face.
      g.save();g.translate(8,-37)
      for(let k=0;k<18;k++){
        const angle = k*TAU/18, end = angle+TAU/18-.012
        g.beginPath();g.arc(0,0,62,angle,end);g.arc(0,0,52,end,angle,true);g.closePath()
        g.fillStyle=k%4 ? '#777b70' : '#929382';g.fill()
        stroke(g,[[54*Math.cos(angle),54*Math.sin(angle)],[61*Math.cos(angle),61*Math.sin(angle)]],'#545b50',1.2)
      }
      g.restore()
      for (const [x,y,w,h] of [[-125,-26,35,19],[-95,-8,44,17],[94,-33,27,21]]) {
        for(let k=0;k<19;k++) dab(g,x+(rand()-.5)*w,y+(rand()-.5)*h,6+rand()*8,3+rand()*4,paint.ink,.32)
      }
      for(let j=0;j<110;j++){
        const x=-148+rand()*296,y=-94+rand()*96
        dryLine(g,[[x,y],[x+3,y+1],[x+6+rand()*14,y+1.6]],2.4,j%3?PAPER:INK,.26,630+j,4)
      }
      for(let j=0;j<8;j++){const x=-147+j*38;stroke(g,[[x,-94],[x-3,-78],[x+3,-70],[x+1,-60]],'#716e5b75',1)}
      g.restore()
      inkLine(g,[[3683,G-94],[3761,G-97],[3856,G-93],[3983,G-96]],5.5,INK,910,.85)
    } else if (i === 4) {
      // Shadowed side gallery, lit beneath the eaves; front pillars belong to the bell tower.
      g.save();g.translate(4432,G)
      wash(g,[[-110,0],[-107,-72],[106,-72],[111,0]],'#656963')
      for(let x=-94;x<97;x+=34){
        wash(g,[[x,-57],[x+22,-57],[x+22,-22],[x,-22]],'#c8a778')
        for(let y=-54;y<-22;y+=8)stroke(g,[[x,y],[x+22,y]],'#53564d',1.4)
        stroke(g,[[x+11,-57],[x+11,-22]],'#464a44',2)
      }
      g.translate(0,-63);g.scale(.86,.58);roof(g,0,0,267,paint,1591);g.restore()
    } else {
      // Old practice swords stand in the rock on either side of the empty duelling ground.
      for (const x of [5382,5407,5444,5883,5911,5935]) {
        g.save();g.translate(x,G+1);g.rotate((rand()-.5)*.51)
        const h=35+rand()*38
        wash(g,[[-2,0],[-3,-h],[2,-h],[1,0]],'#59615d')
        stroke(g,[[1,-h],[0,-4]],'#dfd8bc',1)
        inkLine(g,[[-8,-h],[8,-h-1]],2.8,INK,x)
        stroke(g,[[0,-h],[0,-h-12]],'#353733',3)
        wash(g,[[1,-h-10],[6,-h-11],[13,-h-6],[18,-h-7],[12,-h-3]],'#87483c')
        g.restore()
      }
    }
  }

  function terrain(a, i) {
    const from = a.world.areas[i].x, to = a.world.areas[i + 1]?.x ?? a.world.length + 110
    const { paint, detailRes } = a.mountain
    return A.strip(to - from, -70, 479, detailRes, (g, lo, hi) => {
      g.translate(-from, 0)
      if (i === 0) {
        characterTree(g, 630, G + 2, -.47, 845, TREES.ledge, paint)
        characterTree(g, 799, G + 2, .91, 701, TREES.gate, paint)
        stoneLantern(g, 1196, G, .68, paint)
        stoneLantern(g, 552, G, .61, paint)
      }
      chapterScenery(g, i, paint)
      if (i === 2) {
        characterTree(g, 2442, G + 1, -.52, 919, TREES.wind, paint)
        characterTree(g, 3108, G + 1, .75, 924, TREES.wind, paint)
        corridor(g, paint)
      }
      if (i === 3) courtyard(g, paint)
      if (i === 4) {
        characterTree(g, 4563, G + 1, .74, 1857, TREES.bell, paint)
        characterTree(g, 5087, G + 2, -.60, 1861, TREES.bell, paint)
        bellTower(g, a.world.bell.x, paint)
      }
      if (i === 5) summitTerrace(g, paint)
      if (i !== 1) {
        const start = i === 2 ? BRIDGE[1] - 10 : from
        const end = i === 0 ? BRIDGE[0] + 10 : to
        if (from + lo < end && from + hi > start) {
          terrace(g, start, end, from + lo, from + hi, a.world.areas.at(-1).x)
        }
      }
      if (i === 5) {
        // A narrow sunlit tread breaks into a steep ink face directly below the duel.
        const lip = []
        for (let x = from; x <= to + 6; x += 6) {
          lip.push([x, G + 23 + 5 * Math.sin(x * .019) + 3 * Math.sin(x * .055) + A.hash(x, 383) * 2])
        }
        g.save(); path(g, [...lip, [to + 6, G + 112], [from, G + 112]], true); g.clip()
        g.fillStyle = gradient(g, 0, G + 16, 0, G + 95, [[0, '#514d49'], [.34, '#736b63'], [1, '#a19483']])
        g.fillRect(from, G + 14, to - from, 99)
        for (let x = from; x < to + 24; x += 3 + Math.round(16 * A.hash(x, 15))) {
          const t = A.hash(x, 370), y = G + 18 + 18 * A.hash(x, 18), n = 13 + 48 * t
          const cleft = [[x, y], [x - 3, y + n * .28], [x - 10, y + n * .53], [x - 14, y + n]]
          dryLine(g, cleft, 2 + t * 4, t > .62 ? PAPER : INK, .56, x, 5)
        }
        g.restore()
        dryLine(g, lip, 2.9, PAPER, .84, 3687, 5)
        // Broken horizontal strata under the arena, well below the actors' feet.
        // Old sword cuts run obliquely across the lip, deepening at the edge of the dais.
        for (const [x, y, span] of [[5542, 321, 58], [5618, 332, 88], [5637, 337, 46], [5802, 327, 74]]) {
          const cut = [[x, y], [x + span * .27, y - 4], [x + span * .41, y - 4], [x + span, y - 14]]
          inkLine(g, cut, 3.7, [70, 58, 58], x, .92)
          dryLine(g, cut.map(([xx, yy]) => [xx, yy + 1.8]), 2.2, PAPER, .75, x + 1, 4)
        }
        const rand = A.rng(909)
        for (let x = from + 14; x < to; x += 100 + rand() * 147) {
          const y = G + 26 + rand() * 18, width = 57 + rand() * 74
          const lip = [[x, y], [x + 12, y - 3], [x + width * .63, y - 1], [x + width, y - 6]]
          wash(g, [...lip, [x + width - 16, y + 3], [x + width * .48, y + 7], [x + 4, y + 5]],
            A.rgba([124, 112, 104], .17))
          dryLine(g, lip, 2.5, [97, 89, 82], .44, Math.round(x), 5)
        }
      }
      if (i === 0) {
        stoneGate(g)
        // Scree fans below the gate: the pale chips echo the falling-rock tell.
        const stone = A.rng(629)
        for (const base of [843, 1135, 1290]) for (let j = 0; j < 28; j++) {
          const t = stone(), x = base + t * 104, y = G + 9 + t * 20 + stone() * 14
          const w = 1.7 + stone() * 8 * (1 - t)
          wash(g, [[x, y], [x + w * .43, y - w * .48], [x + w, y - 1], [x + w * .91, y + 1]], '#776b56')
          stroke(g, [[x, y], [x + w * .43, y - w * .48]], '#e0cfab', .85)
        }
      }
      if (i === 3) {
        // Raked arcs around the old practice stone leave a still centre for the duel.
        for (let j = 0; j < 9; j++) {
          const arc = []
          for (let k = 0; k <= 26; k++) {
            const q = -.45 + k / 26 * 3.4
            arc.push([3828 + (101 + 10 * j) * Math.cos(q), G + 17 + (8 + j * 1.6) * Math.sin(q)])
          }
          dryLine(g, arc, 1.7, j % 2 ? INK : PAPER, .48, 4100 + j, 4)
        }
      }
      if (i === 4) lanternGround(g, a.world.bell.x, paint)
      for (const [x, depth, size] of GROUND_PATCHES[i]) {
        if (x + 85 < from + lo || x - 85 > from + hi) continue
        groundPatch(g, x, G + depth, size, GROUND_BY_AREA[i], paint, 510 + i * 101)
      }
      for (const p of a.world.platforms) if (p.x >= from && p.x < to) A.drawSprite(g, a.mountain.platforms.get(p), p.x, p.y)
    })
  }

  function platform(p, res, paint) {
    return A.sprite(p.w + 26, G - p.y + 20, res, g => {
      if (p.material === 'cantilever') {
        const depth = G - p.y
        // Joints gather at the old bridge posts; leave open air under the free end.
        beam(g, [[14, depth + 1], [17, 8]], 7, p.x + 3, true)
        beam(g, [[17, depth - 1], [p.w - 13, 7]], 5.8, p.x + 5, true)
        beam(g, [[14, 9], [p.w - 6, 9]], 5.2, p.x + 7)
        planks(g, 0, 0, p.w, 8, p.x, paint)
        for (let i = 0; i < 3; i++) stroke(g, [[10, 9 + i * 2], [21, 11 + i * 2]], '#cfb995', 1.1)
      } else if (p.material === 'ledge') {
        const depth = G - p.y
        const face = [[0, 0], [p.w, 0], [p.w + 4, depth], [p.w * .52, depth + 2],
          [p.w * .57, depth * .58], [p.w * .27, depth * .30], [8, 14]]
        wash(g, face, gradient(g, 0, 0, 12, depth, [[0, '#a29e88'], [.24, '#868c7e'], [1, '#c5c3ac']]))
        for (let j = 0; j < 23; j++) {
          const x = 26 + A.hash(j, p.x) * (p.w - 32), y = A.hash(j, 17) * 14
          dryLine(g, [[x, y], [x - 2, y + 5], [x + 9, y + 13]], 2.6, INK, .38, p.x + j, 4)
        }
        dryLine(g, [[0, 0], [p.w * .28, 1], [p.w * .67, -1], [p.w, 0]], 3, INK, .88, p.x, 8)
      } else if (p.material === 'wood') {
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
      foreground: {}, panels: new Map(), ridges: [], cliffs: [], banks: [], world, res, detailRes, paint,
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

  function cascadeCliff(res, paint) {
    return A.sprite(450, 436, res, g => {
      const profile = [
        [-326,-106],[-119,-106],[-93,-74],[-89,-33],[-47,-23],[-34,6],[11,18],[40,42],[65,45],
        [71,87],[90,110],[93,149],[112,181],[107,209],[127,237],[137,309],[155,330],[-326,330],
      ]
      const cliff = chippedEdge(profile,1296)
      g.save(); path(g,cliff,true);g.clip()
      g.fillStyle = gradient(g,-170,0,105,252,[[0,'#2e423f'],[.42,'#546359'],[.73,'#788074'],[1,'#91968a']])
      g.fillRect(-326,-106,485,437)
      const rand = A.rng(1867)
      for (let i = 0; i < 64; i++) {
        const x = -235 + rand() * 354, y = -93 + rand() * 425
        dab(g,x,y,17 + rand() * 45,15 + rand() * 69,i % 9 ? paint.ink : paint.paper,.16 + rand() * .18)
        axeStroke(g,x,y,7 + rand() * 20,19 + rand() * 91,6 + rand() * 16,
          i % 5 ? [38,54,49] : [174,174,145],.34,rand)
      }
      const folds = [
        [[-115,-96],[-116,-32],[-128,7],[-104,58],[-113,105],[-84,176],[-67,219]],
        [[-50,-26],[-50,17],[-20,51],[-27,95],[-45,135],[-33,168],[-24,220]],
        [[29,39],[44,63],[57,119],[42,172],[70,202],[63,236],[83,273]],
      ]
      for (const [i, fold] of folds.entries()) {
        inkLine(g,fold,14,[34,49,45],1946+i,.77)
        dryLine(g,fold.map(([x,y])=>[x-4,y]),5.8,[179,180,145],.45,1950+i,8)
      }
      g.restore()
      dryLine(g,profile.slice(2,14),4.1,[165,168,145],.54,1954,7)
      // Feather the left crop into the larger valley wash on the approach.
      g.globalCompositeOperation = 'destination-in'
      g.fillStyle = gradient(g,-296,0,-220,0,[[0,'#0000'],[.48,'#0007'],[1,'#000']])
      g.fillRect(-296,-106,450,436)
      g.globalCompositeOperation = 'source-over'
    }, 296, 106)
  }

  function gateCascade(g, a, cam, clock, layer) {
    const scroll = .84, x = 786 * scroll
    const left = cam.left * scroll
    if (x + 155 < left || x - 296 > left + cam.viewW) return
    g.save(); layer(g, cam, scroll)
    a.mountain.cascadeCliff ||= cascadeCliff(a.mountain.detailRes, a.mountain.paint)
    A.drawSprite(g, a.mountain.cascadeCliff, x, 0)
    const bank = [[x-28,38],[x-9,38],[x+8,41],[x+29,51]]
    inkLine(g, bank, 7.8, [100,112,101], 1911, .88)
    dryLine(g, bank, 5.9, PAPER, .88, 1917, 7)
    waterfall(g, x-9, 39, 274, 91, clock)
    waterfall(g, x+12, 49, 265, 70, clock*.87)
    const drift = Math.sin(clock * .67) * 11
    g.drawImage(a.mountain.cloud, x-121+drift, 265, 242, 61)
    g.drawImage(a.mountain.cloud, x-141-drift, 289, 270, 47)
    g.restore()
  }

  function bridgeCloudSea(g, a, cam, clock, layer, amount) {
    const scroll = .80, origin = 1690 * scroll - 170
    a.mountain.cloudSea ||= A.sprite(910, 162, a.mountain.res, cloud => {
      const rand = A.rng(2907)
      // Overlap pigment and paper in soft lobes. A few dry highlights carry the sweep across the gap.
      for (let i = 0; i < 12; i++) {
        const t = i / 11, x = 44 + 816 * t
        const crest = 91 - t * 43 + Math.sin(t * TAU * 2.3) * 9
        const envelope = Math.sin(Math.PI * (.10 + .80 * t))
        dab(cloud,x,crest,101 + rand()*75,23 + 25 * envelope,a.mountain.paint.paper,.79)
        dab(cloud,x + 12,crest + 22,113 + rand()*53,17 + 16 * envelope,a.mountain.paint.paper,.61)
      }
      const coolBrush = tintBrush(brushStamp(2857),[111,134,135])
      for (let i = 0; i < 72; i++) {
        const x = 26 + rand()*854, t=x/910
        const crest = 76 - t*43 + Math.sin(t*TAU*2.3)*7
        const y = crest + (rand()-.32)*(12+40*Math.sin(Math.PI*t))
        dab(cloud,x,y,24+rand()*102,3+rand()*12,i%8 ? a.mountain.paint.paper : coolBrush,.27+rand()*.36)
      }
      for (const [i, ribbon] of [
        [[40,76],[212,83],[327,65],[478,68],[600,55],[768,56],[901,40]],
        [[105,105],[250,101],[416,91],[540,96],[656,85],[825,77]],
        [[174,124],[316,112],[439,120],[584,106],[736,105]],
      ].entries()) dryLine(cloud,ribbon,7-i*1.4,PAPER,.51-i*.10,2924+i,9)
    })
    g.save(); layer(g, cam, scroll)
    A.drawSprite(g,a.mountain.cloudSea,origin+Math.sin(clock*.23)*27,188,amount*.91)
    g.restore()
  }

  // A distant stretch of the pilgrims' path sits below the playable traverse.
  // Paint to a small retained surface once, so its fine brush marks never swim.
  function summitHermitage(res) {
    return A.sprite(320, 285, res, g => {
      const rock = [77, 76, 76], paper = [219, 209, 194]
      rockInk(g, 164, 84, 246, 189, rock, paper, 1663, 1, .77, .73, true)
      // Follow the same chipped crest as the rock wash, including the roots of the pine.
      const crest = rockProfile(164, 84, 246, 189, PROFILES[1], 1696, true)
      const surface = x => crest.find(([px]) => px >= x)[1]
      const bench = [[143, 96], [196, 98], [215, 92]]
      const footing = crest.filter(([x]) => x >= 138 && x <= 217).reverse().map(([x, y]) => [x, y + 9])
      wash(g, [...bench, ...footing], '#9e9788')
      dryLine(g, bench, 2.3, PAPER, .66, 1710)
      for (let j = 0; j < 10; j++) {
        const x = 144 + 4.9 * j, length = Math.max(2, surface(x) - 96)
        dryLine(g, [[x, 98], [x - 2, 98 + length * .52], [x - 3, 98 + length]], 1.8, rock, .45, 1720 + j, 4)
      }
      pine(g, 129, surface(129) + 4, .22, 1692, [83, 86, 76])

      // Three bays face the viewer. The short return wall, plinth and eave recede together.
      wash(g, [[147, 92], [195, 94], [210, 89], [210, 93], [195, 98], [147, 96]], '#716d62')
      wash(g, [[149, 77], [194, 79], [194, 94], [149, 92]], '#c1b5a0')
      wash(g, [[194, 79], [207, 70], [207, 89], [194, 94]], '#9c9282')
      wash(g, [[150, 78], [194, 80], [194, 82], [150, 80]], '#8b8375')
      wash(g, [[168, 81], [177, 81.4], [177, 93], [168, 92.7]], '#625f58')
      for (const x of [154, 181]) {
        const y = 82 + (x - 154) * .045
        wash(g, [[x, y], [x + 7, y + .3], [x + 7, y + 6], [x, y + 5.7]], '#7a776a')
        stroke(g, [[x + 3.4, y], [x + 3.4, y + 5.8]], '#b0a68f', .8)
      }
      for (const x of [149, 165, 179, 194]) {
        const lean = (x - 149) * .045
        stroke(g, [[x, 78 + lean], [x, 92 + lean]], '#73695d', 1.35)
      }

      // A continuous ridge and a gently sagging tiled plane, with the gable visible at right.
      wash(g, [[194, 78], [201, 59], [213, 70], [207, 72]], '#aca08d')
      const eave = [[139, 72], [144, 75], [154, 76.5], [181, 78], [193, 78.1], [200, 76.6]]
      const ridge = [[151, 58], [154, 59.6], [192, 61], [201, 59.5]]
      wash(g, [...ridge, ...eave.slice().reverse()], '#696c67')
      for (let j = 0; j < 14; j++) {
        const t = j / 13, x = 153 + t * 46, tip = 144 + t * 52
        stroke(g, [[x, 61 + t], [x - 2, 67 + t], [tip, 74.7 + 2 * t]], j % 3 ? '#797b72' : '#8a8a7e', .68)
      }
      stroke(g, ridge, '#575b57', 1.6)
      stroke(g, eave, '#515653', 2)
      stroke(g, [[199, 77], [201, 60], [206, 66], [213, 70], [215, 69]], '#5e625c', 1.65)
      dryLine(g, eave.slice(1), 1.3, PAPER, .50, 1743, 4)
    })
  }

  function valleyCrossing(res) {
    return A.sprite(410, 121, res, g => {
      const ink = [79, 94, 89]
      const deck = []
      for (let i = 0; i <= 24; i++) {
        const t = i / 24, x = 64 + t * 224, y = 48 + 16 * Math.sin(Math.PI * t) - t * 11
        deck.push([x, y])
        if (i % 2 === 0) stroke(g, [[x, y + 1], [x - 1, y - 8]], A.rgba(ink, .72), 1)
      }
      dryLine(g, deck, 3.4, ink, .86, 831, 5)
      stroke(g, deck.map(([x, y]) => [x, y - 8]), A.rgba(ink, .59), .9)
      for (const [x, y, w] of [[38, 47, 88], [328, 37, 94]]) {
        rockInk(g, x, y, w, 78, ink, [216, 221, 202], Math.round(x), 1, .64, .5)
      }
      // A wayside shelter anchors the far end of the crossing.
      wash(g, [[287, 26], [296, 24], [307, 15], [325, 17], [336, 23], [343, 23],
        [337, 28], [298, 30]], A.rgba(ink, .85))
      for (const x of [298, 330]) stroke(g, [[x, 27], [x, 41]], A.rgba(ink, .88), 1.8)
    })
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

  // The skyline sits below eye level at the summit. Each range has its own
  // parallax and wash; a strip of cloud hides its foot before the next range.
  const SUMMIT_RANGES = [
    { scroll: .09, base: 222, peaks: [9, 20], widths: [25, 65], color: [113, 113, 129], alpha: .42 },
    { scroll: .17, base: 251, peaks: [12, 37], widths: [28, 81], color: [98, 106, 121], alpha: .62 },
    { scroll: .32, base: 286, peaks: [17, 53], widths: [34, 104], color: [82, 91, 102], alpha: .79 },
    { scroll: .49, base: 325, peaks: [21, 63], widths: [41, 128], color: [64, 74, 78], alpha: .86 },
  ]

  function summitView(g, a, cam, clock, layer, amount) {
    const cache = a.mountain.summitRanges ||= []
    for (const [i, r] of SUMMIT_RANGES.entries()) {
      const seed = 3321 + i * 79
      // The horizon is continuous; nearer ridges surface in clusters with cloud-filled gaps.
      const top = r.base - r.peaks[1] - 12
      const range = cache[i] ||= i < 2 ? A.mountains({
        width: 2000, top, height: 122, base: r.base,
        peakW: r.widths, peakH: r.peaks, gap: [.5, 1.27], round: 1.05,
        seed, res: a.mountain.res, detail: 11 + i * 3, detailAmp: 5 + i * 3,
        depth: 15 + i * 7, strokeCell: 3.4, strokeDensity: .41 + i * .08,
        fadeH: 85, strokeLen: 4 + i * 2, outline: .38, color: r.color, alpha: r.alpha,
      }) : A.strip(2000, top, 129, a.mountain.res, ink => {
        const rand = A.rng(seed), paper = [229, 214, 196]
        for (let x = -40; x < 2140; x += 150 + rand() * 157) {
          const w = 114 + rand() * 117, h = 22 + rand() * (i === 2 ? 24 : 38)
          const y = r.base - h + rand() * 8
          rockInk(ink, x - w * .39, y + h * .23, w * .69, h * .88, r.color, paper,
            Math.round(x) + 1793, 2, .47, .48)
          rockInk(ink, x, y, w, h + 13, r.color, paper, Math.round(x) + seed, i % 3, .55 + i * .08, .69)
        }
      })
      layer(g, cam, r.scroll)
      g.globalAlpha = amount
      const origin = 5150 * r.scroll - 500
      g.save(); g.translate(origin, 0)
      A.drawStrip(g, range, cam.left * r.scroll - origin, cam.viewW)
      g.restore()
      if (i === 2) {
        a.mountain.hermitage ||= summitHermitage(a.mountain.res)
        g.save(); g.translate(5700 * r.scroll + 150, 245); g.scale(.36, .36)
        A.drawSprite(g, a.mountain.hermitage, 0, 0)
        g.restore()
      }
      mistBelt(g, a, cam, r.scroll, r.base - 11, 48 + i * 4, 205 + i * 15, clock, .84, seed)
    }
  }

  function background(g, a, cam, clock, env, layer) {
    paintSky(g, cam, env)
    const middle = cam.left + cam.viewW / 2
    const visible = (x, half, parallax) => x + half > cam.left * parallax - 12 && x - half < cam.left * parallax + cam.viewW + 12
    const climb = smooth((middle - 3200) / 2360)
    const chasm = smooth((middle - 1370) / 280) * (1 - smooth((middle - 2060) / 300))
    const dusk = smooth((middle - 4120) / 390) * (1 - smooth((middle - 4940) / 430))
    const summit = smooth((middle - 5000) / 395)
    const sunSize = 60 + 96 * climb - 62 * summit
    const sunX = A.lerp(448 + middle * .32, 5700 * FAR_SCROLL + 37, climb)
    const sunY = 72 + 31 * climb + 36 * summit

    g.save()
    layer(g, cam, FAR_SCROLL)
    // Layer watered ink into long cloud shoulders on paper. Leave the centre open for the ridge crests.
    a.mountain.cloudInk ||= A.sprite(550, 87, a.mountain.res, c => {
      const rand=A.rng(449), pigment=a.mountain.paint.ink
      for(let j=0;j<64;j++){
        const x=34+rand()*475, envelope=Math.sin(x/550*Math.PI)
        const y=50-19*envelope+Math.sin(x*.017)*6+rand()*11
        dab(c,x,y,26+rand()*52,3+rand()*11*envelope,pigment,.08+rand()*.12)
      }
    })
    const cloudLeft=cam.left*FAR_SCROLL
    for (let band=Math.floor((cloudLeft-525)/700);band*700<cloudLeft+cam.viewW;band++){
      A.drawSprite(g,a.mountain.cloudInk,band*700+12,29+18*Math.sin(band*3.7),.71)
    }
    g.globalAlpha = (.74 + climb * .21) * (1 - .78 * chasm) * (1 - .88 * dusk)
    g.drawImage(a.mountain.sun, sunX - sunSize / 2, sunY - sunSize / 2, sunSize, sunSize)
    if (climb > .35) {
      g.strokeStyle = A.rgba([64, 60, 64], .6 * climb)
      g.lineWidth = 1.15
      for (let i = 0; i < 6; i++) {
        const x = sunX + 38 + i * 12 + 16 * Math.sin(clock * .12)
        const y = sunY + 31 - (i < 3 ? i : 6 - i) * 4
        const wing = Math.sin(clock * 3 + i * .9) * 2.1
        g.beginPath(); g.moveTo(x - 4, y - wing); g.quadraticCurveTo(x - 1.5, y - 1.7, x, y)
        g.quadraticCurveTo(x + 1.5, y - 2, x + 4.3, y - wing); g.stroke()
      }
    }
    g.globalAlpha = (1 - .63 * chasm) * (1 - summit)
    g.translate(0, 55 * climb + 49 * chasm)
    for (let i = 0; i < RIDGES.length; i++) {
      const [x, , width] = RIDGES[i], at = x * FAR_SCROLL + 245
      if (summit === 1 || !visible(at, width / 2 + 12, FAR_SCROLL)) continue
      a.mountain.ridges[i] ||= distant(i, a.mountain.res * .79)
      A.drawSprite(g, a.mountain.ridges[i], at, 0)
    }
    mistBelt(g, a, cam, FAR_SCROLL, 169, 120, 267, clock, .52, 17)
    mistBelt(g, a, cam, FAR_SCROLL, 235, 137, 235, clock, .62, 23)
    birds(g, a, cam, clock)

    layer(g, cam, CLIFF_SCROLL)
    // As the path rises, the nearer shoulders sink below the sword line.
    g.globalAlpha = (1 - .53 * chasm) * (1 - summit)
    g.translate(0, 83 * climb + 38 * chasm)
    for (let i = 0; i < CLIFFS.length; i++) {
      const [x, , width] = CLIFFS[i], at = x * CLIFF_SCROLL + 120
      if (summit === 1 || !visible(at, width * 1.25, CLIFF_SCROLL)) continue
      a.mountain.cliffs[i] ||= closeCliff(i, a.mountain.res)
      A.drawSprite(g, a.mountain.cliffs[i], at, 0)
    }
    for (const [x, y, height, width] of FALLS) {
      const at = x * CLIFF_SCROLL + 120
      if (visible(at, width, CLIFF_SCROLL)) waterfall(g, at, y, height, width, clock)
    }
    const crossing = 1830 * CLIFF_SCROLL + 35
    if (visible(crossing + 205, 205, CLIFF_SCROLL)) {
      a.mountain.valleyBridge ||= valleyCrossing(a.mountain.res)
      g.save(); g.translate(crossing, 195); g.rotate(-.14)
      A.drawSprite(g, a.mountain.valleyBridge, 0, 0, .82); g.restore()
    }
    mistBelt(g, a, cam, CLIFF_SCROLL, 217, 125, 260, clock, .58, 37)
    mistBelt(g, a, cam, CLIFF_SCROLL, 280, 113, 270, clock, .70, 41)
    if (middle < 1450) gateCascade(g, a, cam, clock, layer)
    if (chasm > 0) bridgeCloudSea(g, a, cam, clock, layer, chasm)
    if (summit > 0) summitView(g, a, cam, clock, layer, summit)
    g.restore()
    if (dusk > 0) {
      // Deep indigo under the eaves, fading to paper behind the combat silhouettes.
      g.save(); g.setTransform(1, 0, 0, 1, 0, 0); g.globalAlpha = dusk
      g.fillStyle = gradient(g, 0, 0, 0, cam.height, [[0, '#283941ec'], [.36, '#405158b8'], [.69, '#77796754'], [1, '#a5876220']])
      g.fillRect(0, 0, cam.width, cam.height); g.restore()
    }
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
    g.fillStyle = '#9c4a37'
    g.beginPath(); g.moveTo(x, y)
    g.bezierCurveTo(x + dir * len * .38, y + lift * .45, x + dir * len * .77, y + lift + wave, x + dir * len, y + lift + wave - 5)
    const tipY = y + lift + wave
    g.lineTo(x + dir * len * .79, tipY - 1)
    g.lineTo(x + dir * (len + 4), tipY + 3)
    g.lineTo(x + dir * len * .73, tipY + 2)
    g.lineTo(x + dir * len * .88, tipY + 7)
    g.quadraticCurveTo(x + dir * len * .42, y + 17, x, y + 13)
    g.closePath(); g.fill()
    stroke(g, [[x + dir*3, y+9],[x+dir*len*.29,y+10+lift*.37],[x+dir*len*.62,y+lift+wave*.66+2]], '#623e329e', 2.2)
    stroke(g, [[x, y + 3], [x + dir * len * .35, y + lift * .4 + 4], [x + dir * len * .7, y + lift + wave + 1]], '#dec3a8', .8)
  }

  function paintBellBody(g) {
    stroke(g, [[0,-29],[-1,-13],[0,1]], '#464a43', 3)
    for (const side of [-1,1]) {
      g.beginPath();g.moveTo(0,-1);g.bezierCurveTo(side*16,-19,side*3,-17,0,-8)
      g.strokeStyle='#57553e';g.lineWidth=3.5;g.stroke()
    }
    g.fillStyle = gradient(g, -33, 0, 33, 0, [[0, '#524b3e'], [.33, '#b59c70'], [.64, '#897553'], [1, '#494839']])
    g.strokeStyle = '#4b473e'; g.lineWidth = 1.2
    g.beginPath(); g.moveTo(-14, 0); g.quadraticCurveTo(-27, 6, -30, 37); g.lineTo(-35, 45)
    g.quadraticCurveTo(0, 53, 35, 45); g.lineTo(30, 37); g.quadraticCurveTo(27, 6, 14, 0); g.closePath(); g.fill(); g.stroke()
    g.save(); g.clip()
    const rand = A.rng(3483)
    for(let i=0;i<94;i++) {
      const x=(rand()-.5)*71, y=rand()*52
      const oxidized = x < -11 || x > 17 || y > 36
      g.fillStyle=A.rgba(oxidized ? [52,82,68] : [221,185,121],oxidized ? .36 : .27)
      A.leaf(g,x,y,rand()*TAU,1+rand()*7,1+rand()*2)
    }
    for(let j=-2;j<=2;j++){
      const x=j*11
      const scroll=[[x-5,30],[x-5,24],[x+3,24],[x+3,29],[x-1,29],[x-1,27]]
      stroke(g,scroll,'#61583e',1.6)
      stroke(g,scroll.map(([px,py])=>[px-.45,py-.55]),'#ccae76',.85)
    }
    g.restore()
    for (let j = 0; j < 4; j++) {
      g.beginPath(); g.moveTo(-22 - j * 2, 9 + j * 9); g.quadraticCurveTo(0, 16 + j * 9, 22 + j * 2, 9 + j * 9); g.stroke()
    }
    // Worn raised bosses and a broken gold rim catch the last light.
    for (const side of [-1, 1]) {
      for (let row = 0; row < 3; row++) for (let col = 0; col < 2; col++) {
        const x = side * (6 + col * 8 + row * 1.2), y = 7 + row * 6
        g.fillStyle = '#615337'; g.fillRect(x, y, 2.6, 2.8)
        g.fillStyle = '#e3c18b'; g.fillRect(x, y, 1.6, 1)
      }
    }
    stroke(g, [[-31, 43], [-19, 46], [-6, 47]], '#eacb92', 1.5)
    for (const x of [-14, 0, 14]) stroke(g, [[x, 7], [x * 1.48, 40]], '#c8a77b', .72)
    stroke(g, [[0, 42], [0, 62]], '#4a4944', 2.5)
  }

  function bell(g, a, bell, clock, time, active) {
    const age = clock - bell.ringAt
    const swing = Math.sin(time * .94) * .026 + Math.sin(age * 13) * Math.exp(-age * 2.1) * bell.dir * .22
    g.save(); g.translate(bell.x, bell.y - 45); g.rotate(swing)
    a.mountain.bellBody ||= A.sprite(82, 100, a.mountain.detailRes, paintBellBody, 41, 32)
    A.drawSprite(g, a.mountain.bellBody, 0, 0)
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

  function windRibbon(res) {
    return A.sprite(260, 73, res, g => {
      const path = Array.from({ length: 29 }, (_, i) => {
        const t = i / 28
        return [3 + 247*t, 50 - 40*t + Math.sin(t*TAU*1.65 + .3) * (2.5 + 5*t)]
      })
      g.fillStyle = gradient(g,0,0,247,0,[[0,'#e2ebdc00'],[.19,'#edf1ddcc'],[.56,'#eef2ddce'],[1,'#e2ebdc00']])
      A.ribbon(g,path,t=>.5+6*Math.sin(Math.PI*t))
      dryLine(g,path,6.4,[237,243,224],.81,1784,9)
      dryLine(g,path.map(([x,y],i)=>[x,y+9+Math.sin(i)*2]),3.0,[101,133,127],.41,1811,7)
      dryLine(g,path.slice(9,23).map(([x,y])=>[x+8,y-8]),2.1,[248,244,222],.65,1824,5)
    })
  }

  function moving(g, a, cam, state, time, reducedMotion) {
    const left = cam.left - 85, right = cam.left + cam.viewW + 85
    const clock = state.clock
    for (const [x, y] of [[2525, 166], [2763, 160], [3020, 183]]) {
      if (x >= left && x <= right) pennant(g, x, y, reducedMotion ? 0 : time, state.wind)
    }
    if (state.bell && state.bell.x >= left && state.bell.x <= right) {
      const active = state.enemies.some(e => e.type === 'shizhen' && e.alive && e.state === 'mountTell' && Math.abs(e.x - state.bell.x) <= state.bell.radius)
      bell(g, a, state.bell, clock, reducedMotion ? 0 : time, active && clock - state.bell.ringAt >= 1.6)
    }
    const dash = state.player.action?.borrow
    if (dash && !reducedMotion) {
      const feet = state.player.y, end = state.player.x
      const sweep = Math.min(1, Math.abs(end - dash.from) / 55)
      for (let j = 0; j < 3; j++) {
        const from = A.lerp(dash.from, end, j * .17)
        g.beginPath(); g.moveTo(from, feet - 3 - j * 4)
        g.quadraticCurveTo((from + end) * .5, feet - 5 - (13 + j * 2) * sweep, end, feet - 8 - j * 3)
        g.strokeStyle = A.rgba(j === 1 ? PAPER : INK, .45 * sweep)
        g.lineWidth = j === 1 ? 1.8 : 1; g.stroke()
      }
    }
    for (const e of state.enemies) {
      const cut = e.needleCut
      if (cut && !reducedMotion && clock - cut.at < .85) {
        const t = (clock - cut.at) / .85
        for (let j = 0; j < 9; j++) {
          const x = cut.x + cut.dir * t * (33 + A.hash(j, 42) * 56)
          const y = cut.y - Math.sin(t * Math.PI) * (11 + j * 3) + t * t * 28
          g.fillStyle = A.rgba([64, 76, 55], .9 * (1 - t))
          A.leaf(g, x, y, j + t * 4, 5 + j % 4, .85)
        }
      }
      if (e.plantedAt == null) continue
      const age = clock - e.plantedAt
      if (age < 0 || age > 3.2) continue
      g.save(); g.translate(e.plantedX, e.y); g.scale(e.plantedDir, 1)
      g.globalAlpha *= Math.min(1, (3.2 - age) / .8)
      stroke(g, [[-16, 1], [-4, 0], [4, 2], [19, 0], [26, -1]], '#514a43', 1.6)
      stroke(g, [[2, 2], [8, 4], [16, 3]], '#7c7060', .8)
      g.restore()
    }
    if (reducedMotion) return
    if (state.bell && state.bell.x >= left && state.bell.x <= right) {
      const age = clock - state.bell.ringAt
      if (age >= 0 && age < 1.55) {
        for (let j = 0; j < 5; j++) {
          const t = age - j * .085
          if (t <= 0) continue
          const dir = j % 2 ? -1 : 1
          const x = state.bell.x + dir * (36 + j * 8 + t * 72)
          const y = state.bell.y - 110 - t * (23 + j * 4) + Math.sin(t * 4 + j) * 3
          const wing = Math.sin(t * 16 + j) * 3
          stroke(g, [[x - 4, y - wing], [x, y], [x + 4, y - wing * .72]],
            A.rgba(INK, Math.min(1, (1.55 - age) * 2) * .79), 1.3)
        }
      }
    }
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
      a.mountain.windRibbon ||= windRibbon(a.mountain.res)
      for (let j = 0; j < 4; j++) {
        const span = hi - lo + 240
        const phase = time * dir * (72 + power * 166) + A.hash(j,461) * span
        const x = lo - 120 + ((phase % span + span) % span)
        if (x + 260 < left || x - 260 > right) continue
        const opacity = smooth((x - lo + 91) / 132) * smooth((hi - x + 96) / 158)
        if (!opacity) continue
        g.save();g.translate(x,120 + j*26 + Math.sin(time*.66+j)*12);g.scale(dir*(.57 + power*.37),.70 + j*.065)
        A.drawSprite(g,a.mountain.windRibbon,0,0,opacity*(.25 + power*.56))
        g.restore()
      }
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
      for (let j = 0; j < 9; j++) {
        const span = hi - lo, phase = time * dir * (30 + power * 140) + A.hash(j, 73) * span
        const x = lo + ((phase % span + span) % span)
        if (x < left || x > right || a.world.wind.posts.some(post => (x - post) * dir >= 0 && (x - post) * dir < 47)) continue
        const y = 245 + Math.sin(time * 1.8 + j) * 10 + A.hash(j, 811) * 47
        g.fillStyle = A.rgba([75, 79, 63], .38 + power * .26)
        A.leaf(g, x, y, dir * .2 + Math.sin(time * 2 + j) * .45, 4 + j % 4, .85)
      }
    }
  }

  // Close brushwork frames the scroll from above and below. Keep the horizontal
  // band where enemies wind up completely clear, even on short landscape phones.
  const FOREGROUND = [
    [725, 'pine', 1], [1160, 'bank', 1], [1670, 'bank', -1], [2160, 'vine', -1],
    [2550, 'vine', 1], [2925, 'grass', -1], [3550, 'pine', 1], [3985, 'bank', -1],
    [4480, 'grass', 1], [4940, 'vine', -1], [5890, 'bank', 1],
  ]

  function nearBrush(kind, seed, res) {
    const rand = A.rng(seed), black=[30,34,29], grey=[84,86,70]
    return A.sprite(267, 113, res, g => {
      if (kind === 'pine' || kind === 'vine') {
        const limb = kind === 'pine'
          ? [[0,2],[24,10],[48,29],[77,30],[108,44],[137,48],[156,55],[186,53],[219,61],[246,56]]
          : [[0,0],[16,14],[44,16],[67,29],[96,32],[114,52],[132,57],[153,78],[182,92],[210,96]]
        g.fillStyle = A.rgba(black,.91)
        A.ribbon(g,limb,t=>.8 + 17 * (1-t)**1.3)
        dryLine(g,limb,9,black,.77,seed,8)
        dryLine(g,limb.map(([x,y])=>[x,y-2]),4.2,[139,129,103],.72,seed+5,4)
        if (kind === 'pine') {
          const palette = [[25,36,29],[45,62,47],[85,91,68]]
          const brush = tintBrush(brushStamp(seed), palette[0])
          for (const [index, reach, drop, size] of [[1,45,20,1.03],[3,52,22,.92],[4,42,-10,.86],[6,55,22,1.12],[8,28,-8,.80]]) {
            const [x,y] = limb[index], tx = x+reach, ty = y+drop
            inkLine(g,[[x,y],[x+reach*.38,y+drop*.74],[tx,ty]],4,black,seed+index)
            treeCluster(g,tx,ty,size,palette,.55,false,brush,rand)
            treeCluster(g,x+reach*.63,y+drop*.67-8,size*.72,palette,-.32,false,brush,rand)
          }
        } else {
          for (const [index, reach, drop] of [[1,40,30],[3,38,-2],[4,53,31],[5,17,44],[7,49,-11],[8,45,7]]) {
            const [x,y] = limb[index]
            const points = [[x,y],[x+reach*.29,y+drop*.57],[x+reach*.67,y+drop*.94],[x+reach,y+drop]]
            inkLine(g,points,2.4,black,seed+index)
            for(let n=1;n<=14;n++){
              const ix = n<=7 ? 1 : 2, t = (n%7)/7
              const px = A.lerp(points[ix][0],points[ix+1][0],t), py = A.lerp(points[ix][1],points[ix+1][1],t)
              const dir = n%2 ? 1 : -1
              g.fillStyle=A.rgba(n%4 ? black : grey,.62+rand()*.33)
              A.leaf(g,px,py,dir*(.72+rand()*.73),6+rand()*13,1.6+rand()*2.1)
            }
          }
        }
        return
      }
      for(let j=0;j<6;j++){
        const x=24+j*39+rand()*15, y=35+rand()*24,w=15+rand()*39
        const lip=[[x-w,y+8],[x-w*.5,y-4],[x+w*.22,y-11],[x+w*.6,y-9],[x+w,y+2]]
        wash(g,[...lip,[x+w*.55,y+20],[x-w*.6,y+23]],A.rgba(j%3?grey:black,.83))
        wash(g,[lip[1],lip[2],lip[3],[x+w*.35,y],[x-w*.6,y+12]],'#a39f83')
        dryLine(g,lip,3.1,black,.86,seed+j,6)
        for(let n=0;n<12;n++){const xx=x+(rand()-.5)*w,yy=y+rand()*8;stroke(g,[[xx,yy],[xx-4,yy+3],[xx-8,yy+4]],'#42453b9c',.9)}
      }
      for(let j=0;j<(kind==='grass'?68:24);j++){
        const x=31+rand()*202,y=39+rand()*22,dx=(rand()-.64)*32,h=8+rand()*31
        g.fillStyle=A.rgba(black,.37+rand()*.55)
        A.ribbon(g,[[x,y],[x+dx*.34,y-h*.65],[x+dx,y-h]],t=>.16+(1-t)*1.25)
      }
    })
  }

  function foreground(g,a,cam,layer) {
    const speed=1.14, left=cam.left*speed, right=left+cam.viewW
    g.save();layer(g,cam,speed)
    for (const [worldX,kind,dir] of FOREGROUND) {
      const x=worldX*speed
      if(x+267<left || x-267>right)continue
      // Share four brush studies across the scroll; cropped views differ by location and direction.
      const images=a.mountain.foreground
      images[kind] ||= nearBrush(kind,{pine:673,vine:951,bank:724,grass:859}[kind],a.mountain.res)
      const canopy=kind==='pine'||kind==='vine'
      g.save();g.translate(x,canopy?cam.top-22:G+14);g.scale(dir,1)
      A.drawSprite(g,images[kind],0,0,canopy?.91:.88);g.restore()
    }
    g.restore()
  }

  function attackGuides(g, state) {
    for (const fall of state.mountainFalls) {
      if (fall.landed) continue
      const q = Math.min(1,fall.age/fall.warning), x=fall.x, radius=33
      g.beginPath();g.ellipse(x,G-1,radius,5,0,0,TAU)
      g.fillStyle=A.rgba([102,57,44],.17+.38*q);g.fill()
      stroke(g,[[x-radius,G-3],[x-radius,G+2],[x+radius,G+2],[x+radius,G-3]],CINNABAR,1.4)
      const y=G-182+169*Math.pow(q,3)
      wash(g,[[x-12,y-9],[x+4,y-16],[x+17,y-7],[x+13,y+4],[x-7,y+7]],'#52574e')
      stroke(g,[[x-12,y-9],[x+3,y-13],[x+8,y-6]],'#b1ac94',1.9)
      for(let j=0;j<7;j++){
        const t=(q*2+j/7)%1,dx=(j-3)*6
        stroke(g,[[x+dx,y+9+t*23],[x+dx-2,y+12+t*23]],'#797968',1.35)
      }
    }
    const wave=state.bellWave
    if(wave) {
      for(const dir of [-1,1]){
        const x=wave.x+dir*wave.radius
        if(wave.t<.58) stroke(g,[[x,G-4],[x+dir*48,G-4]],CINNABAR,2)
        else {
          wash(g,[[x-dir*27,G],[x-dir*10,G-6],[x-dir*3,G-23],[x+dir*5,G-3],[x+dir*10,G]],'#65574b')
          stroke(g,[[x-dir*10,G-6],[x-dir*3,G-23],[x+dir*5,G-3]],'#c7ad85',1.8)
        }
      }
    }
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

  globalThis.XiabanInkMountain = { build, background, ground, moving, foreground, attackGuides }
})()
