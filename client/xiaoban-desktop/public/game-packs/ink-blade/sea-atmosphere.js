/* 怒涛卷 · painted washes, coastal silhouettes and independently moving planes. */
(() => {
  'use strict'

  const A = globalThis.XiabanInkArt
  const rgba = (c, a) => `rgba(${c.join(',')},${a})`
  const lerp = (a, b, t) => a + (b - a) * t

  // Each panel reserves its own paper and silhouette. The x positions are world anchors;
  // offsets inside the paintings are relative to the middle of their encounter.
  const PANELS = [
    { x: 1030, clouds: [[-240,85,490,93,-.13,.43],[227,157,530,59,-.045,.33]], ink: [141,105,81], light: [252,217,163], shore: [[-390,190,96],[268,124,39],[411,213,66]], near: [[-434,193,105],[419,164,78]], front: [-446, 431] },
    { x: 1780, clouds: [[-270,150,510,67,-.045,.4],[294,79,538,116,-.14,.53]], ink: [147,111,73], light: [254,223,159], shore: [[-397,224,59],[-164,119,37],[302,212,71]], near: [[-372,135,47],[392,146,60]], front: [-435] },
    { x: 2790, clouds: [[-180,122,630,92,.05,.46],[285,189,457,49,-.07,.38]], ink: [78,106,110], light: [215,230,212], shore: [[-328,197,90],[45,192,56],[307,147,77]], near: [[-426,166,105],[400,197,101]], front: [-429, 434] },
    { x: 3710, clouds: [[-262,96,548,102,-.08,.52],[257,154,570,70,-.065,.41]], ink: [133,96,89], light: [246,199,148], shore: [[-385,181,106],[-205,155,71],[348,174,80]], near: [[-435,145,92],[422,157,117]], front: [-438, 433] },
    { x: 4700, clouds: [[-274,50,682,135,-.12,.93],[287,97,770,124,-.14,.97],[103,152,690,63,-.105,.82]], ink: [18,32,47], light: [186,215,213], shore: [[-403,196,107],[-206,193,55],[256,209,116]], near: [[-429,186,126],[392,188,101]], front: [-433, 433] },
    { x: 5660, clouds: [[-249,93,681,105,-.09,.84],[233,169,696,83,-.075,.69]], ink: [29,36,54], light: [226,221,190], shore: [[-361,198,82],[50,218,87],[362,178,110]], near: [[-397,145,97],[425,156,106]], front: [-436, 439] },
  ]

  // A bank of soft, broken ink deposits. Pigment collects where short paper fibres
  // intersect each wash. Rasterized once; all wide clouds share these small stamps.
  function washBrush(color, seed) {
    const w = 160, h = 96, node = A.canvas(w, h), g = node.getContext('2d')
    const pixels = g.createImageData(w, h)
    const n = A.noise(seed, 1024), random = A.rng(seed + 1)
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const u = (x / w - .5) * 2, v = (y / h - .5) * 2
        const clouds = A.fbm(n, x * .038 + n(y * .081) * 8 + y * .127, 4)
        const fibres = n(y * .6 + x * .027)
        const radius = u*u + v*v * 1.16
        const edge = Math.max(0, 1 - radius * (1.13 + clouds * .27))
        const pigment = Math.pow(edge, 1.35) * (.31 + clouds * .6 + fibres * .12)
        const i = (y * w + x) * 4, grain = (random() - .55) * 17
        pixels.data[i] = color[0] + grain; pixels.data[i+1] = color[1] + grain; pixels.data[i+2] = color[2] + grain
        pixels.data[i+3] = 255 * pigment
      }
    }
    g.putImageData(pixels, 0, 0)
    return node
  }

  function daub(g, brush, x, y, w, h, angle, alpha) {
    g.save(); g.translate(x, y); g.rotate(angle)
    g.globalAlpha = alpha; g.drawImage(brush, -w/2, -h/2, w, h); g.restore()
  }

  function paintClouds(panel, index) {
    const brush = washBrush(panel.ink, 351 + index * 19)
    const lit = washBrush(panel.light, 590 + index)
    const random = A.rng(190 + index)
    return A.sprite(1460, 290, .62, (g) => {
      for (const [x,y,w,h,angle,alpha] of panel.clouds) {
        daub(g, brush, x, y, w*1.4, h*1.48, angle, alpha*.64)
        daub(g, brush, x, y+6, w, h, angle, alpha)
        for (let i = 0; i < 12; i++) {
          const dx = (random()-.5)*w*.92, dy = (random()-.5)*h*.7 + dx * angle
          daub(g, brush, x+dx, y+dy, w*(.21+random()*.33), h*(.26+random()*.48), angle-.025+random()*.05, alpha*(.35+random()*.39))
        }
        daub(g, lit, x+w*.13, y+h*.34, w*.96, h*.24, angle, index === 4 ? .72 : .53)
      }
    }, 730, 14)
  }

  // Angular spines on the rocks are small, overlapping brush planes rather than an
  // outline. The lower wash dissolves into sea mist and leaves the battle lane legible.
  function rock(g, x, base, width, height, color, alpha, seed, fade = true) {
    const random = A.rng(seed), noise = A.noise(seed + 35)
    const ridge = (t) => {
      const peak = Math.max(0, 1 - Math.abs((t - .47) * 1.88))
      const mass = Math.pow(peak, .78) * (.77 + .24 * noise(t * 5.8))
      return base - height * (mass + (A.fbm(noise, t * 13, 4) - .49) * .34 * peak)
    }
    g.save()
    g.beginPath(); g.moveTo(x-width/2, base+12)
    for (let i = 0; i <= 96; i++) {
      const t = i/96
      g.lineTo(x+(t-.5)*width, ridge(t))
    }
    g.lineTo(x+width/2, base+12); g.closePath()
    const wash = g.createLinearGradient(0, base-height, 0, base+12)
    wash.addColorStop(0, rgba(color, alpha)); wash.addColorStop(.55, rgba(color, alpha*.77))
    wash.addColorStop(1, rgba(color, alpha*(fade ? .12 : .92)))
    g.fillStyle = wash; g.fill(); g.clip()
    for (let i = 0; i < 80; i++) {
      const t = random(), xx = x+(t-.5)*width, top = ridge(t)
      const y = lerp(top-2, base, random()*.83), length = 6+random()*height*.48
      const light = i%3 === 0
      g.strokeStyle = light ? rgba([226,218,178],alpha*(.06+random()*.29)) : rgba(color,alpha*(.1+random()*.32))
      g.lineWidth = .52 + random()*2.9
      g.beginPath(); g.moveTo(xx,y)
      g.lineTo(xx-3-length*.14,y+length*.37); g.lineTo(xx+length*.04,y+length*.62)
      g.lineTo(xx+length*.18,y+length); g.stroke()
    }
    g.restore()
    A.dryBrush(g, (t) => [x+(t-.5)*width,ridge(t),0,1], 3.9, 4, color, alpha*.44, seed+7, 70)
  }

  function paintShore(panel, index, near) {
    const wet = index === 2 || index === 4 || index === 5
    const color = near ? (wet ? [48,64,65] : [73,66,57]) : (wet ? [63,83,88] : [102,96,83])
    return A.sprite(1340, 176, near ? .83 : .67, (g) => {
      for (const [j, [x, w, h]] of (near ? panel.near : panel.shore).entries()) {
        const base = near ? 126 : 126 - j%2*9
        rock(g, x, base, w*1.43, h*.83, color, near ? .63 : .37, 729+index*29+j*13)
        if (near) rock(g, x+w*.26, base+6, w*.65, h*.54, color, .39, 404+j*17+index)
      }
    }, 670, 0)
  }

  function paintForeground(panel, index) {
    const dark = index === 4 || index === 5 ? [23,33,33] : [51,45,33]
    return A.sprite(1120, 114, .85, (g) => {
      for (const [i, x] of panel.front.entries()) {
        rock(g, x, 113, i%2 ? 168 : 174, index === 4 ? 62 : 49, dark, .92, 837+index*12+i, false)
        const random = A.rng(index*17+i+614)
        for (let j = 0; j < 9; j++) {
          const px = x+(random()-.5)*100, y = 90+random()*21
          const bend = random()*12-6, len = 15+random()*26
          g.fillStyle = rgba(dark,.36+random()*.46)
          A.ribbon(g, [[px,y],[px+bend*.24,y-len*.55],[px+bend,y-len]], t=>2.5*(1-t))
        }
      }
    }, 560, 0)
  }

  function paint(panel, index) {
    return {
      cloud: paintClouds(panel, index),
      far: paintShore(panel, index, false),
      near: paintShore(panel, index, true),
      front: paintForeground(panel, index),
    }
  }

  function frame(cache, index) {
    if (!cache.has(index)) {
      cache.set(index, paint(PANELS[index], index))
      // The long scroll only needs the two plates on either side of a transition.
      if (cache.size > 3) cache.delete(cache.keys().next().value)
    }
    return cache.get(index)
  }

  function composite(g, a, cam, e, time, setLayer, key, f, y) {
    setLayer(g, cam, f)
    const panes = e.seaA === e.seaB ? [[e.seaA,1]] : [[e.seaA,1-e.seaBlend],[e.seaB,e.seaBlend]]
    for (const [i, alpha] of panes) {
      if (alpha < .006) continue
      const panel = PANELS[i]
      const x = panel.x*f + cam.viewW*.5*(1-f)
      const drift = key === 'cloud' ? Math.sin(time*(.12+e.rain*.4)+i)*12 : 0
      A.drawSprite(g, frame(a.seaPaintings, i)[key], x+drift, y, alpha)
    }
  }

  function sky(g, a, cam, e, time, setLayer) {
    g.setTransform(1,0,0,1,0,0)
    const h = cam.height, w = cam.width
    const wash = g.createLinearGradient(0,0,0,h)
    wash.addColorStop(0,rgba(e.top,1)); wash.addColorStop(.49,rgba(e.middle,1))
    wash.addColorStop(.73,rgba(e.low,1)); wash.addColorStop(1,rgba(e.low,1))
    g.fillStyle = wash; g.fillRect(0,0,w,h)
    const lightX = cam.viewW*.70 - cam.left*.012
    const lightY = e.moon > .1 ? 86 : 99
    const cx = lightX*cam.P, cy = (lightY-cam.top)*cam.P
    const light = g.createRadialGradient(cx,cy,0,cx,cy,cam.P*(e.moon > .1 ? 125 : 210))
    light.addColorStop(0,rgba(e.glow,.19+e.sun*.23+e.moon*.16))
    light.addColorStop(.46,rgba(e.glow,.09+e.sun*.11))
    light.addColorStop(1,rgba(e.glow,0))
    g.fillStyle = light; g.fillRect(0,0,w,h)
    if (e.sun > .015 || e.moon > .015) {
      const moon = e.moon > e.sun, size = (moon ? 134 : 81) * cam.P
      g.globalAlpha = moon ? e.moon : e.sun
      g.drawImage(moon ? a.moon : a.sun, cx-size/2, cy-size/2, size, size)
      g.globalAlpha = 1
    }
    composite(g,a,cam,e,time,setLayer,'cloud',.055,-2)
    composite(g,a,cam,e,time,setLayer,'far',.14,106)
    return lightX
  }

  function shore(g, a, cam, e, time, setLayer) {
    composite(g,a,cam,e,time,setLayer,'near',.38,131)
  }

  function birds(g, cam, e, time, setLayer) {
    // Tiny distant gulls ride above the safe lane, away from the combat silhouettes.
    const weight = (e.seaA < 2 ? 1-e.seaBlend : 0) + (e.seaB < 2 ? e.seaBlend : 0)
    if (weight < .02) return
    setLayer(g,cam,.25)
    const travel = (time*14 + 175) % 1140
    g.save(); g.strokeStyle = rgba([66,60,51],weight*.42); g.lineWidth = .8
    for (let i = 0; i < 5; i++) {
      const x = 180+travel-i*24, y = 140+i*6+Math.sin(time*.6+i*.7)*8
      const flap = 2.5+Math.sin(time*6+i*1.4)*2.4
      g.beginPath();g.moveTo(x-6,y-flap)
      g.quadraticCurveTo(x-2,y-flap-1,x,y)
      g.quadraticCurveTo(x+2,y-1,x+7,y-flap*.72);g.stroke()
    }
    g.restore()
  }

  function drips(g, x, y, time) {
    g.save()
    for (const [i, [dx, top]] of [[-201,-195],[-96,-231],[108,-226],[267,-199]].entries()) {
      const t = (time*(.45+i*.07)+i*.27) % 1
      if (t < .72) {
        const p=t/.72, height=y+top+(-top-14)*p*p
        g.strokeStyle = 'rgba(224,234,207,.57)';g.lineWidth = .75
        g.beginPath();g.moveTo(x+dx,height-2-p*5);g.lineTo(x+dx,height+2);g.stroke()
      } else {
        const p=(t-.72)/.28
        g.strokeStyle = rgba([228,234,208],.37*(1-p));g.lineWidth=.65
        g.beginPath();g.ellipse(x+dx,y-14,3+p*11,1.2+p*2,0,0,Math.PI*2);g.stroke()
      }
    }
    g.restore()
  }

  function foreground(g, a, cam, e, time, setLayer) {
    composite(g,a,cam,e,time,setLayer,'front',1.2,265)
  }

  globalThis.XiabanInkSeaAtmosphere = { sky, shore, birds, drips, foreground }
})()
