/* 悬山 · pure ink-wash silhouettes matching figures.js: solid dark ink, cinnabar eyes, iconic weapons & hats. */
(() => {
  'use strict'

  const F = globalThis.XiabanInkFigures
  const A = globalThis.XiabanInkArt
  const TAU = Math.PI * 2
  const EMBER = '#e2412c'
  const CINNABAR = '#b8322a'
  const PALE = 'rgba(239,231,212,0.34)'
  const EDGE = 'rgba(239,231,212,0.58)'
  const lerp = (a, b, t) => a + (b - a) * t

  const STRIKES = {
    thrust:  { ready: -0.52, finish: -0.02, coil: 0.04, lunge: 0.59 },
    cleave:  { ready: -2.42, finish: 0.78, coil: -0.32, lunge: 0.64 },
    lift:    { ready: 0.68, finish: -1.35, coil: 0.13, lunge: 0.24 },
    shove:   { ready: -1.67, finish: -0.10, coil: -0.08, lunge: 0.56 },
    rush:    { ready: -2.85, finish: -0.09, coil: 0.41, lunge: 0.73 },
    reverse: { ready: 0.68, finish: -1.09, coil: 0.21, lunge: 0.48 },
    sweep:   { ready: -2.92, finish: 0.31, coil: -0.22, lunge: 0.68 },
  }
  const TWO_HANDED = new Set(['yue', 'monk'])

  function pose(e, t, anim, time, q, speed) {
    const heavy = e.type === 'yue'
    const bow = e.type === 'crossbow'
    t.lean = heavy ? 0.11 : 0.19
    t.crouch = heavy ? 2.5 : 1.5
    t.sw = heavy ? 0.18 : 0.55
    t.reach = heavy ? 0.92 : 1
    t.ou = 0.5; t.of = 0.2
    t.l1 = 0.18; t.l2 = -0.20; t.b1 = 0.28; t.b2 = 0.18
    t.flutter = 0.08 + Math.sin(time * 2.1 + e.seed) * 0.05
    if (e.type === 'umbrella') { t.ou = 1.15; t.of = 1.05; t.sw = 0.78 }
    if (e.type === 'monk') { t.sw = -0.41; t.reach = 1.15 }
    if (bow) { t.sw = 0.04; t.ou = 1.26; t.of = 1.5; t.reach = 1.05 }
    if (e.type === 'yue' && e.phase2) { t.lean = 0.22; t.sw = -0.27; t.l1 = 0.33; t.l2 = -0.36 }

    if (e.state === 'walk' || (speed > 25 && e.state === 'idle')) {
      const a = Math.min(0.85, speed / 140), sn = Math.sin(anim.phase), co = Math.cos(anim.phase)
      t.l1 = sn * 0.86 * a; t.l2 = -sn * 0.86 * a
      t.b1 = 0.25 + Math.max(0, co) * a; t.b2 = 0.25 + Math.max(0, -co) * a
      t.bob = Math.abs(co) * a * 1.4; t.flutter += a * 0.8
    }
    if (e.state === 'mountTell') {
      const { ready, coil } = STRIKES[e.move] || STRIKES.thrust
      const pull = Math.min(1, q * 2.65)
      t.sw = ready + (1 - pull) * 0.21
      t.lean = coil - pull * 0.08; t.crouch = 2.5 + pull * (e.sweep ? 5 : 2.6)
      t.reach = e.move === 'lift' ? 0.86 : 1.02
      t.l1 = 0.41; t.l2 = -0.55; t.b1 = 0.51; t.b2 = 0.23
      if (e.type === 'umbrella') { t.sw = -1.45; t.ou = 0.85; t.of = 1.21 }
    }
    if (e.state === 'mountAttack') {
      const { finish, lunge } = STRIKES[e.move] || STRIKES.thrust
      t.sw = finish; t.lean = lunge; t.crouch = e.sweep ? 8.5 : 3.5
      t.reach = 1.22; t.l1 = 0.91; t.l2 = -0.62; t.b1 = 0.74; t.b2 = 0.1
      t.flutter = 0.96; anim.snap = true
    }
    if (e.state === 'pineCross') {
      t.lean = 0.84; t.sw = -2.58; t.crouch = 8; t.flutter = 1; t.l1 = 0.92; t.l2 = -0.8
    }
    if (e.state === 'bowTell') {
      t.sw = -0.1; t.crouch = 2 + q * 2; t.lean = -0.08; t.reach = 1.19
      t.draw = q; t.l1 = 0.38; t.l2 = -0.42; t.b1 = 0.42
    }
    if (e.state === 'umbrellaOpen') {
      t.lean = -0.24; t.crouch = 3.5; t.ou = 0.52; t.of = 0.95
      t.sw = 2.16; t.reach = 0.77; t.l1 = 0.39; t.l2 = -0.36
    }
    if (e.state === 'recover') {
      t.lean = e.sweep ? 0.54 : 0.32; t.crouch = e.sweep ? 6.4 : 3.4
      t.sw = e.move === 'lift' ? -0.76 : e.sweep ? 0.60 : 0.47
      t.ou = 0.22; t.of = 0.61; t.l1 = 0.54; t.l2 = -0.42
    }
    if (e.state === 'hurt' || e.state === 'stagger' || e.state === 'broken' || e.state === 'dying') {
      const kneel = e.state === 'broken' || e.state === 'dying'
      t.lean = kneel ? 0.52 : -0.42; t.crouch = kneel ? 13 : 5
      t.sw = kneel ? 0.77 : -2.25; t.ou = 2; t.of = 2.4
      t.l1 = kneel ? 1.25 : 0.56; t.l2 = kneel ? -0.95 : -0.32
      t.b1 = kneel ? 1.82 : 0.6; t.b2 = kneel ? 1.17 : 0.24; t.head = kneel ? 0.27 : -0.3
    }
    if (e.state === 'roar') {
      const step = Math.sin(anim.phase)
      t.sw = -1.6; t.lean = -0.18; t.l1 = step * 0.57; t.l2 = -step * 0.57
      t.flutter = 1; t.reach = 0.85
    }
    return t
  }

  function drawEyes(g, x, y, glow, size) {
    g.save()
    g.fillStyle = 'rgba(226,65,44,' + (0.14 + glow * 0.24) + ')'
    g.beginPath(); g.ellipse(x - size * 0.2, y, size * (0.44 + glow * 0.22), size * (0.26 + glow * 0.12), -0.2, 0, TAU); g.fill()
    g.fillStyle = EMBER
    g.beginPath(); g.ellipse(x, y, size * 0.22, size * 0.14, -0.2, 0, TAU); g.fill()
    g.beginPath(); g.ellipse(x - size * 0.48, y + size * 0.06, size * 0.15, size * 0.11, -0.2, 0, TAU); g.fill()
    g.restore()
  }

  function paint(g, kind, p, x, y, facing, look = {}, time = 0, seed = 0) {
    const spec = F.KINDS[kind] || F.KINDS.disciple
    const s = F.skeleton(spec, p)
    const e = look.entity || {}
    const color = look.color || spec.color || '#1b1d21'
    const flash = Boolean(look.flash)
    const [lu, lf, au] = spec.limbs
    const [hx, hy] = s.head, [sx, sy] = s.shoulder, [bx, by] = s.hip
    let tip = s.hand

    g.save()
    g.translate(x, y)
    g.scale(facing, 1)
    if (look.alpha !== undefined) g.globalAlpha *= look.alpha
    g.lineJoin = 'round'; g.lineCap = 'round'

    // Wet-ink pale halo keeps dark silhouettes clear over wash layers (matches figures.js).
    if (look.halo) {
      g.save()
      g.globalAlpha *= look.halo
      g.fillStyle = 'rgba(239,231,212,1)'
      g.beginPath(); g.ellipse(bx, by - spec.torso * 0.35, spec.torso * 0.9, spec.torso * 1.35, p.lean, 0, TAU); g.fill()
      g.restore()
    }

    g.fillStyle = color; g.strokeStyle = color

    const limb = (pts, w) => A.ribbon(g, pts, (t) => w * (1 - t * 0.42))

    function weaponAngle(len, clearance) {
      if (p.sw <= 0) return p.sw
      return Math.min(p.sw, Math.asin(Math.max(-0.95, Math.min(0.95, (-s.hand[1] - clearance) / len))))
    }

    if (TWO_HANDED.has(kind)) {
      const len = kind === 'yue' ? 68 : 84
      const angle = weaponAngle(len, kind === 'yue' ? 3 : 4)
      s.offHand = [s.hand[0] - Math.cos(angle) * 15, s.hand[1] - Math.sin(angle) * 15]
      s.offElbow = [(s.offShoulder[0] + s.offHand[0]) * 0.5 - 3, (s.offShoulder[1] + s.offHand[1]) * 0.5 + 6]
    }

    // Flowing cape / cloak behind the silhouette for elite 燕回 and Boss 岳停云.
    if (kind === 'yanhui' || kind === 'yue') {
      const capeLen = kind === 'yue' ? 34 : 24
      const fl = p.flutter
      const wave = Math.sin(time * 5.2 + seed) * (2.5 + fl * 4)
      const back = 12 + fl * 18 + (kind === 'yue' ? 6 : 0)
      g.beginPath()
      g.moveTo(sx - 5, sy - 2)
      g.quadraticCurveTo(bx - 10 - back * 0.4, by + 6, bx - back, by + capeLen + wave)
      g.lineTo(bx - back + 11, by + capeLen - 4 + wave * 0.5)
      g.lineTo(bx - 6, by + 5)
      g.closePath(); g.fill()
    }

    // Back arm (unless two-handed drawn with weapon).
    if (!TWO_HANDED.has(kind)) limb([s.offShoulder, s.offElbow, s.offHand], au * 0.92)

    // Legs & feet (same tapered ink brush as figures.js).
    limb([s.hip, s.back.knee, s.back.foot], lu)
    g.lineWidth = lf
    g.beginPath(); g.moveTo(s.back.foot[0] - 1.5, s.back.foot[1]); g.lineTo(s.back.foot[0] + 3.5, s.back.foot[1] + 0.3); g.stroke()
    limb([s.hip, s.front.knee, s.front.foot], lu)
    g.beginPath(); g.moveTo(s.front.foot[0] - 1.5, s.front.foot[1]); g.lineTo(s.front.foot[0] + 4, s.front.foot[1] + 0.3); g.stroke()

    // Robe / tunic silhouette (pure ink).
    const long = spec.robe === 'long'
    const hemDepth = kind === 'yue' ? 31 : long ? 24 : 15
    const wave = Math.sin(time * 5 + seed) * (1 + p.flutter * 2.6)
    const backX = -9 - p.flutter * (long ? 14 : 8) - (long ? 3 : 0)
    const frontX = 7 + Math.max(0, Math.sin(p.l1)) * 6 + (long ? 3 : 0)
    const sf = [s.neck[0] + s.fwd[0] * (kind === 'yue' ? 6 : 4.8), s.neck[1] + s.fwd[1] * 4.5 - 1]
    const sb = [s.neck[0] - s.fwd[0] * (kind === 'yue' ? 6.5 : 5.2), s.neck[1] - s.fwd[1] * 4.5]
    const wf = [bx + s.fwd[0] * (kind === 'yue' ? 6.8 : 5.6), by + s.fwd[1] * 5.2]
    const wb = [bx - s.fwd[0] * (kind === 'yue' ? 7.5 : 6.4), by - s.fwd[1] * 5.6]
    const hf = [bx + frontX, by + hemDepth - 2]
    const hb = [bx + backX, by + hemDepth - 3 + wave - p.flutter * 3]
    g.beginPath()
    g.moveTo(sb[0], sb[1])
    g.quadraticCurveTo(s.neck[0], s.neck[1] - 2, sf[0], sf[1])
    g.quadraticCurveTo(wf[0] + 1.5, wf[1] - 3, wf[0], wf[1])
    g.quadraticCurveTo(hf[0] - 1, (wf[1] + hf[1]) * 0.5, hf[0], hf[1])
    g.quadraticCurveTo((hf[0] + hb[0]) * 0.5, by + hemDepth + 3 + wave * 0.4, hb[0], hb[1])
    g.quadraticCurveTo(wb[0] - 3 - p.flutter * 3, (wb[1] + hb[1]) * 0.5, wb[0], wb[1])
    g.quadraticCurveTo(sb[0] - 1.5, (sb[1] + wb[1]) * 0.5, sb[0], sb[1])
    g.closePath(); g.fill()

    if (!flash) {
      // Subtle pale waist sash line for torso definition.
      g.strokeStyle = PALE; g.lineWidth = 0.75
      g.beginPath(); g.moveTo(wb[0] + 1, wb[1]); g.lineTo(wf[0] - 1, wf[1]); g.stroke()
      g.fillStyle = color; g.strokeStyle = color
    }

    // Head + iconic hat/hair silhouette + glowing ember eyes.
    const r = spec.head
    g.beginPath(); g.arc(hx, hy, r, 0, TAU); g.fill()

    if (kind === 'umbrella') {
      // Wide conical mountain douli hat.
      g.save(); g.translate(hx, hy - r * 0.52); g.rotate(s.headTilt * 0.75)
      g.beginPath()
      g.moveTo(-16.5, 2.5); g.quadraticCurveTo(-6, -2.5, 0, -7.2)
      g.quadraticCurveTo(6, -2.5, 17, 2.4); g.quadraticCurveTo(0, 4.4, -16.5, 2.5); g.fill()
      if (!flash) {
        g.strokeStyle = PALE; g.lineWidth = 0.6
        g.beginPath(); g.moveTo(-10, -0.5); g.quadraticCurveTo(0, -3.4, 10, -0.7); g.stroke()
      }
      g.restore()
    } else if (kind === 'crossbow') {
      // Scout hood / peaked brim.
      g.beginPath()
      g.moveTo(hx + r * 1.15, hy - r * 0.5)
      g.quadraticCurveTo(hx - r * 0.2, hy - r * 1.9, hx - r * 2.4, hy + r * 0.55)
      g.quadraticCurveTo(hx - r * 1.1, hy + r * 0.25, hx - r * 0.3, hy + r)
      g.closePath(); g.fill()
    } else if (kind === 'monk') {
      // Clean bald head + prayer beads draped diagonally across chest.
      if (!flash) {
        g.fillStyle = PALE
        for (let j = 0; j < 7; j++) {
          const u = j / 6
          g.beginPath()
          g.arc(lerp(sb[0] + 2, wf[0] - 2, u), lerp(sb[1] + 2, wf[1] - 2, u) + Math.sin(u * Math.PI) * 3, 1.15, 0, TAU)
          g.fill()
        }
        g.fillStyle = color
      }
    } else {
      // Topknot + fluttering hair/cloth ribbon (disciple, yanhui, yue).
      const big = kind === 'yue' ? 1.3 : kind === 'yanhui' ? 1.15 : 0.9
      g.beginPath(); g.ellipse(hx - 2.2 * big, hy - r - 2.2 * big, 2.8 * big, 3.4 * big, -0.28, 0, TAU); g.fill()
      const tailLen = kind === 'yue' ? 22 : kind === 'yanhui' ? 16 : 11
      const sway = Math.sin(time * 6 + seed) * (2 + p.flutter * 3.5)
      const backPull = 4 + p.flutter * 10
      g.strokeStyle = color; g.lineWidth = kind === 'yue' ? 2.2 : 1.5
      g.beginPath()
      g.moveTo(hx - 3, hy - r - 1)
      g.quadraticCurveTo(hx - 6 - backPull * 0.6, hy - r + tailLen * 0.35 + sway * 0.4, hx - 5 - backPull, hy - r + tailLen + sway)
      g.stroke()
    }

    if (!flash) drawEyes(g, hx + r * 0.42, hy - r * 0.1, look.eyeGlow || 0, r)

    // Helper for pure-ink swords with pale edge glint (matches figures.js).
    function drawBlade(hand, angle, len, broad = false, tassel = false) {
      const w = broad ? 6.2 : 2.5
      g.save(); g.translate(hand[0], hand[1]); g.rotate(angle)
      g.fillStyle = color; g.strokeStyle = color
      // Grip & crossguard
      g.lineWidth = broad ? 3.2 : 2.2
      g.beginPath(); g.moveTo(broad ? -19 : -11, 0); g.lineTo(1, 0); g.stroke()
      g.lineWidth = broad ? 2.6 : 1.8
      g.beginPath(); g.moveTo(1, -w - 2.2); g.lineTo(1, w + 2.2); g.stroke()
      // Tapered ink blade
      g.beginPath()
      g.moveTo(2, -w * 0.62)
      g.lineTo(len - (broad ? 12 : 6), -w * (broad ? 0.56 : 0.35))
      g.lineTo(len, 0)
      g.lineTo(len - (broad ? 10 : 6), w * (broad ? 0.56 : 0.35))
      g.lineTo(2, w * 0.62)
      g.closePath(); g.fill()
      if (!flash) {
        g.strokeStyle = EDGE; g.lineWidth = broad ? 1.05 : 0.75
        g.beginPath(); g.moveTo(3, w * 0.42); g.lineTo(len - (broad ? 10 : 5), w * 0.36); g.lineTo(len - 1, 0); g.stroke()
        if (broad) {
          g.strokeStyle = PALE; g.lineWidth = 0.7
          g.beginPath(); g.moveTo(4, -w * 0.25); g.lineTo(len - 14, -w * 0.2); g.stroke()
        }
        // Subtle cinnabar tassel on Boss 岳停云's pommel (PRD 2.3 Boss identity).
        if (tassel) {
          const sw = Math.sin(time * 6 + seed) * 2.2
          g.strokeStyle = CINNABAR; g.lineWidth = 1.8
          g.beginPath(); g.moveTo(-19, 0); g.quadraticCurveTo(-25, 4 + sw * 0.5, -29, 10 + sw); g.stroke()
        }
      }
      g.restore()
      return [hand[0] + Math.cos(angle) * len, hand[1] + Math.sin(angle) * len]
    }

    // Two-handed off-arm + weapon + main arm.
    if (TWO_HANDED.has(kind)) limb([s.offShoulder, s.offElbow, s.offHand], au * 0.92)

    if (kind === 'umbrella') {
      // Iconic large ink iron umbrella held forward in off-hand.
      const open = e.state === 'umbrellaOpen'
      const ux = s.offHand[0] + (open ? 12 : 16)
      const uy = sy + (open ? -6 : 5)
      const rx = open ? 23 : 15
      const ry = open ? 32 : 26
      g.strokeStyle = color; g.lineWidth = 2.6
      g.beginPath(); g.moveTo(s.offHand[0] - 4, s.offHand[1]); g.lineTo(ux + 2, uy); g.stroke()
      g.fillStyle = color
      g.beginPath(); g.ellipse(ux, uy, rx, ry, open ? -0.22 : 0.08, 0, TAU); g.fill()
      if (!flash) {
        g.strokeStyle = PALE; g.lineWidth = 0.75
        for (let i = 0; i < 8; i++) {
          const a = (TAU * i) / 8
          g.beginPath(); g.moveTo(ux + 1, uy); g.lineTo(ux + Math.cos(a) * (rx - 1), uy + Math.sin(a) * (ry - 1)); g.stroke()
        }
        g.fillStyle = EDGE
        g.beginPath(); g.arc(ux + 1, uy, 1.9, 0, TAU); g.fill()
        g.fillStyle = color
      }
      limb([s.shoulder, s.elbow, s.hand], au)
      tip = drawBlade(s.hand, p.sw, 31, false)
    } else if (kind === 'crossbow') {
      limb([s.shoulder, s.elbow, s.hand], au)
      const wx = s.hand[0] + 6, wy = s.hand[1]
      g.fillStyle = color; g.strokeStyle = color
      // Crossbow stock
      g.fillRect(wx - 13, wy - 2.3, 38, 4)
      // Crossbow bow prod
      g.lineWidth = 2.6
      g.beginPath(); g.moveTo(wx + 14, wy - 16); g.quadraticCurveTo(wx + 24, wy, wx + 14, wy + 16); g.stroke()
      // String
      g.strokeStyle = flash ? color : EDGE; g.lineWidth = 0.8
      const drawBack = e.state === 'bowTell' ? 11 : 3
      g.beginPath(); g.moveTo(wx + 14, wy - 15); g.lineTo(wx + 12 - drawBack, wy); g.lineTo(wx + 14, wy + 15); g.stroke()
      if (e.state === 'bowTell') {
        g.strokeStyle = color; g.lineWidth = 1.3
        g.beginPath(); g.moveTo(wx + 2, wy); g.lineTo(wx + 34, wy); g.stroke()
      }
      tip = [wx + 34, wy]
    } else if (kind === 'monk') {
      limb([s.shoulder, s.elbow, s.hand], au)
      const len = 84
      const angle = weaponAngle(len, 4)
      g.save(); g.translate(s.hand[0], s.hand[1]); g.rotate(angle)
      g.strokeStyle = color; g.lineWidth = 3.8
      g.beginPath(); g.moveTo(-24, 0); g.lineTo(len, 0); g.stroke()
      if (!flash) {
        g.strokeStyle = EDGE; g.lineWidth = 1.1
        g.beginPath(); g.moveTo(len - 13, -1.6); g.lineTo(len, -1.6); g.moveTo(len - 13, 1.6); g.lineTo(len, 1.6); g.stroke()
        g.beginPath(); g.moveTo(-24, -1.6); g.lineTo(-17, -1.6); g.stroke()
      }
      g.restore()
      tip = [s.hand[0] + Math.cos(angle) * len, s.hand[1] + Math.sin(angle) * len]
    } else if (kind === 'yanhui') {
      // Off-hand reverse-grip short blade.
      drawBlade(s.offHand, p.of - 1.7, 25, false)
      limb([s.shoulder, s.elbow, s.hand], au)
      tip = drawBlade(s.hand, p.sw, 42, false)
    } else if (kind === 'yue') {
      limb([s.shoulder, s.elbow, s.hand], au)
      const angle = weaponAngle(68, 3)
      tip = drawBlade(s.hand, angle, 68, true, true)
    } else {
      // Disciple: clean straight sword.
      limb([s.shoulder, s.elbow, s.hand], au)
      tip = drawBlade(s.hand, p.sw, 34, false)
    }

    g.restore()
    const map = (v) => [x + v[0] * facing, y + v[1]]
    return { head: map(s.head), tip: map(tip), hand: map(s.hand), neck: map(s.neck), hip: map(s.hip) }
  }

  globalThis.XiabanInkMountainFigures = { pose, paint }
})()
