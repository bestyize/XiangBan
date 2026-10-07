/* 水墨剑客 · ink effects: brush-stroke sword trails, splashes, stains, seals and the full-screen 一笔. */
(() => {
  'use strict'

  const GROUND = globalThis.XiabanInkBlade.GROUND
  const CINNABAR = '#b8322a'
  const INK = '#141416'
  const TAU = Math.PI * 2
  const ease = (t) => 1 - Math.pow(1 - Math.max(0, Math.min(1, t)), 3)

  function create() {
    return { parts: [], splats: [], decals: [], rings: [], sparks: [], seals: [], trails: [], ghosts: [], strike: null, invert: 0, flash: 0, flashColor: '#f6f0e2', letterbox: 0, letterTarget: 0, letterHold: 0, shake: 0 }
  }

  function clear(fx) {
    for (const key of ['parts', 'splats', 'decals', 'rings', 'sparks', 'seals', 'trails', 'ghosts']) fx[key].length = 0
    fx.strike = null; fx.invert = 0; fx.flash = 0; fx.letterbox = 0; fx.letterTarget = 0; fx.letterHold = 0; fx.shake = 0
  }

  const cap = (list, max) => { if (list.length > max) list.splice(0, list.length - max) }

  // Brush hairs for a crescent: outer and inner hairs are shorter, so the stroke tapers at both ends.
  function hairs(count, width, seed) {
    let s = seed
    const random = () => { s = (s * 16807) % 2147483647; return s / 2147483647 }
    const list = []
    for (let i = 0; i < count; i++) {
      const u = count === 1 ? 0 : i / (count - 1)
      const edge = Math.abs(u * 2 - 1)
      list.push({
        off: (u - 0.5) * width, start: edge * 0.3 + random() * 0.06, end: 1 - edge * 0.2 - random() * 0.06,
        alpha: (0.5 + random() * 0.5) * (1 - edge * 0.45), w: width / count * (1.15 + random() * 0.9),
        dash: random() < 0.45 ? [] : [6 + random() * 26, 0.8 + random() * 3.5],
      })
    }
    return list
  }

  function trail(fx, o) {
    fx.trails.push({
      x: o.x, y: o.y, follow: o.follow || null, facing: o.facing, a0: o.a0, a1: o.a1, r: o.r, t: 0,
      grow: o.grow ?? 0.08, life: o.life ?? 0.3, hairs: hairs(o.hairs ?? 11, o.width ?? 15, Math.floor(Math.random() * 1e6) + 1),
      color: o.color || INK, edge: o.edge || null, followFor: o.followFor ?? 0.12,
    })
    cap(fx.trails, 14)
  }

  function splat(fx, sprites, x, y, size, opts = {}) {
    fx.splats.push({ x, y, size, rot: opts.rot ?? Math.random() * TAU, t: 0, life: opts.life ?? 0.55, sprite: sprites[Math.floor(Math.random() * sprites.length)], alpha: opts.alpha ?? 0.92, squash: opts.squash ?? 1 })
    cap(fx.splats, 30)
  }

  function drops(fx, x, y, count, opts = {}) {
    const dir = opts.dir ?? 0, spread = opts.spread ?? Math.PI, speed = opts.speed ?? 260
    for (let i = 0; i < count; i++) {
      const a = (opts.angle ?? -Math.PI / 2) + (Math.random() - 0.5) * spread
      const v = speed * (0.35 + Math.random() * 0.85)
      fx.parts.push({
        kind: 'drop', x, y, vx: Math.cos(a) * v + dir * v * 0.6, vy: Math.sin(a) * v, r: (opts.size ?? 1.6) * (0.4 + Math.random()),
        t: 0, life: 0.6 + Math.random() * 0.7, color: opts.color || INK, stain: opts.stain ?? true, gravity: opts.gravity ?? 900,
      })
    }
    cap(fx.parts, 420)
  }

  function puff(fx, x, y, count, opts = {}) {
    for (let i = 0; i < count; i++) {
      const a = (opts.angle ?? -Math.PI / 2) + (Math.random() - 0.5) * (opts.spread ?? 2.4)
      const v = (opts.speed ?? 50) * (0.4 + Math.random())
      fx.parts.push({ kind: 'puff', x: x + (Math.random() - 0.5) * (opts.w ?? 10), y, vx: Math.cos(a) * v, vy: Math.sin(a) * v, r: (opts.size ?? 6) * (0.6 + Math.random() * 0.6), grow: opts.grow ?? 2.2, t: 0, life: (opts.life ?? 0.5) * (0.7 + Math.random() * 0.6), alpha: opts.alpha ?? 0.35, light: opts.light || false })
    }
    cap(fx.parts, 420)
  }

  // Ink gathers from the air into a figure.
  function converge(fx, x, y, count, radius = 70, life = 0.7) {
    for (let i = 0; i < count; i++) {
      const a = Math.random() * TAU, r = radius * (0.5 + Math.random() * 0.6)
      fx.parts.push({ kind: 'converge', sx: x + Math.cos(a) * r, sy: y + Math.sin(a) * r * 0.8, tx: x + (Math.random() - 0.5) * 14, ty: y + (Math.random() - 0.5) * 34, x, y, r: 1 + Math.random() * 2.2, t: -Math.random() * 0.25, life })
    }
    cap(fx.parts, 420)
  }

  function smoke(fx, x, y, opts = {}) {
    fx.parts.push({ kind: 'puff', x: x + (Math.random() - 0.5) * (opts.w ?? 20), y: y + (Math.random() - 0.5) * (opts.h ?? 40), vx: (Math.random() - 0.5) * 12, vy: -18 - Math.random() * 20, r: 3 + Math.random() * 4, grow: 2.6, t: 0, life: 0.9 + Math.random() * 0.6, alpha: opts.alpha ?? 0.22, light: false, red: opts.red || false })
    cap(fx.parts, 420)
  }

  function ring(fx, x, y, r0, r1, life, color, width) { fx.rings.push({ x, y, r0, r1, t: 0, life, color, width }); cap(fx.rings, 16) }

  function sparks(fx, x, y, count, color = CINNABAR, opts = {}) {
    for (let i = 0; i < count; i++) {
      fx.sparks.push({ x, y, a: (opts.angle ?? 0) + (Math.random() - 0.5) * (opts.spread ?? TAU), len: (opts.len ?? 14) * (0.5 + Math.random()), speed: (opts.speed ?? 260) * (0.5 + Math.random() * 0.8), t: 0, life: 0.18 + Math.random() * 0.14, color, width: opts.width ?? 1.4 })
    }
    cap(fx.sparks, 80)
  }

  function seal(fx, text, x, y, size = 18, life = 0.85, color = CINNABAR) {
    fx.seals.push({ text, x, y, size, t: 0, life, color, rot: (Math.random() - 0.5) * 0.25 })
    cap(fx.seals, 8)
  }

  function ghost(fx, draw, life = 0.8, rise = 14) { fx.ghosts.push({ draw, t: 0, life, rise }); cap(fx.ghosts, 24) }

  function shake(fx, amount) { fx.shake = Math.max(fx.shake, amount) }
  function flash(fx, amount, color = '#f6f0e2') { fx.flash = Math.max(fx.flash, amount); fx.flashColor = color }
  function invert(fx, amount = 1) { fx.invert = Math.max(fx.invert, amount) }
  function letterbox(fx, hold) { fx.letterHold = Math.max(fx.letterHold, hold) }

  function strike(fx, facing) {
    fx.strike = { t: 0, facing, hairs: hairs(34, 1, 977 + Math.floor(Math.random() * 1000)), wobble: Math.random() * 6 }
  }

  function update(fx, dt) {
    for (let i = fx.parts.length - 1; i >= 0; i--) {
      const p = fx.parts[i]
      p.t += dt
      if (p.t >= p.life) { fx.parts.splice(i, 1); continue }
      if (p.kind === 'drop') {
        p.vy += p.gravity * dt; p.vx *= Math.pow(0.6, dt)
        p.x += p.vx * dt; p.y += p.vy * dt
        if (p.y >= GROUND && p.vy > 0) {
          if (p.stain) fx.decals.push({ x: p.x, y: GROUND + 1 + Math.random() * 4, w: p.r * (2.2 + Math.random() * 2), h: p.r * (0.5 + Math.random() * 0.4), t: 0, alpha: 0.55, red: p.color !== INK })
          fx.parts.splice(i, 1)
        }
      } else if (p.kind === 'puff') {
        p.x += p.vx * dt; p.y += p.vy * dt; p.vx *= Math.pow(0.2, dt); p.vy *= Math.pow(0.3, dt)
      }
    }
    cap(fx.decals, 150)
    for (let i = fx.decals.length - 1; i >= 0; i--) { fx.decals[i].t += dt; if (fx.decals[i].t > 16) fx.decals.splice(i, 1) }
    for (const list of [fx.splats, fx.rings, fx.sparks, fx.seals, fx.trails, fx.ghosts]) {
      for (let i = list.length - 1; i >= 0; i--) { list[i].t += dt; if (list[i].t >= list[i].life) list.splice(i, 1) }
    }
    for (const tr of fx.trails) if (tr.follow && tr.t < tr.followFor) { const [x, y] = tr.follow(); tr.x = x; tr.y = y }
    if (fx.strike) { fx.strike.t += dt; if (fx.strike.t > 1.2) fx.strike = null }
    fx.invert = Math.max(0, fx.invert - dt * 7)
    fx.flash = Math.max(0, fx.flash - dt * 3.2)
    fx.shake = Math.max(0, fx.shake - dt * 22)
    fx.letterHold = Math.max(0, fx.letterHold - dt)
    fx.letterTarget = fx.letterHold > 0 || fx.strike ? 1 : 0
    fx.letterbox += (fx.letterTarget - fx.letterbox) * Math.min(1, dt * 7)
  }

  function drawDecals(ctx, fx) {
    for (const d of fx.decals) {
      const fade = d.t < 0.15 ? d.t / 0.15 : d.t > 12 ? 1 - (d.t - 12) / 4 : 1
      ctx.fillStyle = d.red ? 'rgba(178,46,34,' + d.alpha * fade + ')' : 'rgba(20,20,22,' + d.alpha * fade + ')'
      ctx.beginPath(); ctx.ellipse(d.x, d.y, d.w * (0.7 + Math.min(1, d.t * 4) * 0.3), d.h, 0, 0, TAU); ctx.fill()
    }
  }

  function drawGhosts(ctx, fx) {
    for (const g of fx.ghosts) {
      const q = g.t / g.life
      ctx.save()
      ctx.translate(0, -g.rise * ease(q))
      g.draw(ctx, Math.pow(1 - q, 1.4), q)
      ctx.restore()
    }
  }

  function drawTrail(ctx, tr) {
    const p = Math.min(1, tr.t / tr.grow)
    const fade = tr.t < tr.grow ? 1 : 1 - (tr.t - tr.grow) / (tr.life - tr.grow)
    const span = tr.a1 - tr.a0
    ctx.save()
    ctx.translate(tr.x, tr.y)
    ctx.scale(tr.facing, 1)
    ctx.lineCap = 'round'
    ctx.strokeStyle = tr.color
    for (const h of tr.hairs) {
      const end = Math.min(h.end, p)
      const start = h.start + (1 - fade) * 0.85 * Math.max(0, end - h.start)
      if (end <= start) continue
      ctx.globalAlpha = h.alpha * Math.min(1, fade * 1.4)
      ctx.lineWidth = h.w * (0.55 + 0.45 * fade)
      ctx.setLineDash(h.dash)
      ctx.beginPath(); ctx.arc(0, 0, tr.r + h.off, tr.a0 + span * start, tr.a0 + span * end, span < 0); ctx.stroke()
    }
    ctx.setLineDash([])
    if (tr.edge && fade > 0.2) {
      ctx.globalAlpha = fade * 0.9
      ctx.strokeStyle = tr.edge; ctx.lineWidth = 1.1
      const inner = tr.hairs[0]
      ctx.beginPath(); ctx.arc(0, 0, tr.r + inner.off - 0.6, tr.a0 + span * 0.15, tr.a0 + span * Math.min(0.92, p), span < 0); ctx.stroke()
    }
    ctx.restore()
  }

  function drawFront(ctx, fx, a) {
    for (const tr of fx.trails) drawTrail(ctx, tr)
    for (const s of fx.splats) {
      const grow = 0.55 + 0.45 * ease(s.t / 0.09)
      const alpha = s.alpha * (s.t < 0.12 ? 1 : Math.max(0, 1 - (s.t - 0.12) / (s.life - 0.12)))
      const size = s.size * grow
      ctx.save(); ctx.globalAlpha = alpha; ctx.translate(s.x, s.y); ctx.rotate(s.rot); ctx.scale(1, s.squash)
      ctx.drawImage(s.sprite, -size / 2, -size / 2, size, size); ctx.restore()
    }
    for (const p of fx.parts) {
      const q = p.t / p.life
      if (p.kind === 'drop') {
        ctx.globalAlpha = 1 - q * 0.4
        ctx.fillStyle = p.color
        const stretch = Math.min(3, 1 + Math.hypot(p.vx, p.vy) / 350)
        ctx.save(); ctx.translate(p.x, p.y); ctx.rotate(Math.atan2(p.vy, p.vx))
        ctx.beginPath(); ctx.ellipse(0, 0, p.r * stretch, p.r, 0, 0, TAU); ctx.fill(); ctx.restore()
      } else if (p.kind === 'puff') {
        const r = p.r * (1 + q * p.grow)
        ctx.globalAlpha = p.alpha * (1 - q)
        ctx.drawImage(p.red ? a.glow : p.light ? a.light : a.puff, p.x - r, p.y - r, r * 2, r * 2)
      } else if (p.kind === 'converge') {
        if (p.t < 0) continue
        const e = ease(q)
        const x = p.sx + (p.tx - p.sx) * e, y = p.sy + (p.ty - p.sy) * e
        ctx.globalAlpha = Math.min(1, q * 3) * (1 - Math.max(0, q - 0.85) / 0.15)
        ctx.fillStyle = INK
        ctx.beginPath(); ctx.arc(x, y, p.r * (1 - e * 0.4), 0, TAU); ctx.fill()
      }
    }
    ctx.globalAlpha = 1
    for (const r of fx.rings) {
      const q = r.t / r.life
      ctx.globalAlpha = 1 - q
      ctx.strokeStyle = r.color; ctx.lineWidth = r.width * (1 - q * 0.7)
      ctx.beginPath(); ctx.arc(r.x, r.y, r.r0 + (r.r1 - r.r0) * ease(q), 0, TAU); ctx.stroke()
    }
    ctx.lineCap = 'round'
    for (const s of fx.sparks) {
      const q = s.t / s.life, d = s.speed * s.t
      ctx.globalAlpha = 1 - q
      ctx.strokeStyle = s.color; ctx.lineWidth = s.width * (1 - q * 0.6)
      const c = Math.cos(s.a), n = Math.sin(s.a)
      ctx.beginPath(); ctx.moveTo(s.x + c * d, s.y + n * d); ctx.lineTo(s.x + c * (d + s.len * (1 - q)), s.y + n * (d + s.len * (1 - q))); ctx.stroke()
    }
    ctx.globalAlpha = 1
    for (const s of fx.seals) {
      const q = s.t / s.life
      const pop = s.t < 0.1 ? 1.6 - ease(s.t / 0.1) * 0.6 : 1
      ctx.save()
      ctx.globalAlpha = q < 0.75 ? 1 : 1 - (q - 0.75) / 0.25
      ctx.translate(s.x, s.y - q * 8); ctx.rotate(s.rot); ctx.scale(pop, pop)
      ctx.fillStyle = s.color
      ctx.beginPath(); ctx.roundRect(-s.size / 2, -s.size / 2, s.size, s.size, s.size * 0.12); ctx.fill()
      ctx.strokeStyle = 'rgba(251,239,226,0.55)'; ctx.lineWidth = s.size * 0.05
      ctx.beginPath(); ctx.roundRect(-s.size * 0.4, -s.size * 0.4, s.size * 0.8, s.size * 0.8, s.size * 0.08); ctx.stroke()
      ctx.fillStyle = '#fbefe2'
      ctx.font = '600 ' + Math.round(s.size * 0.62) + 'px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
      ctx.fillText(s.text, 0, s.size * 0.04)
      ctx.restore()
    }
  }

  // One horizontal stroke (一) across the page: pressed head, wet core, hairs splitting into 飞白 at the tail.
  function drawGreatStroke(ctx, st, W, H, p, fade) {
    const point = (t) => {
      const x = st.facing > 0 ? -0.04 * W + t * 1.1 * W : 1.04 * W - t * 1.1 * W
      const y = H * 0.55 + Math.sin(t * Math.PI * 1.1 + st.wobble) * H * 0.035 - t * t * H * 0.08
      return [x, y]
    }
    const half = (t) => H * 0.1 * (t < 0.08 ? 0.82 + t * 2.2 : t < 0.6 ? 1 - (t - 0.08) * 0.3 + Math.sin(t * 7) * 0.04 : Math.max(0.03, 0.84 * Math.pow(1 - (t - 0.6) / 0.4, 1.4)))
    const hash = (a, b) => { const s = Math.sin(a * 91.7 + b * 47.3 + st.wobble * 13) * 43758.5; return s - Math.floor(s) }
    ctx.save()
    ctx.globalAlpha = fade
    ctx.fillStyle = INK
    const end = Math.min(p, 0.97), steps = 48
    for (const [spread, alpha] of [[1.22, 0.16], [1.08, 0.28], [1, 1]]) {
      const top = [], bottom = []
      for (let i = 0; i <= steps; i++) {
        const t = end * i / steps
        const [x, y] = point(t)
        const w = half(t) * 0.74 * spread * (0.94 + hash(i, 1) * 0.1)
        top.push([x, y - w]); bottom.push([x, y + w * 0.9])
      }
      ctx.globalAlpha = fade * alpha
      ctx.beginPath(); ctx.moveTo(top[0][0], top[0][1])
      for (const q of top) ctx.lineTo(q[0], q[1])
      for (let i = bottom.length - 1; i >= 0; i--) ctx.lineTo(bottom[i][0], bottom[i][1])
      ctx.closePath(); ctx.fill()
    }
    const [hx, hy] = point(0)
    ctx.beginPath(); ctx.ellipse(hx + (st.facing > 0 ? 6 : -6), hy - H * 0.01, half(0) * 1.05, half(0) * 0.95, -0.3 * st.facing, 0, TAU); ctx.fill()
    ctx.strokeStyle = INK; ctx.lineCap = 'round'
    st.hairs.forEach((h, k) => {
      const reach = Math.min(p, h.end + 0.08)
      if (reach <= 0.02) return
      const edge = Math.abs(h.off) * 2
      ctx.lineWidth = Math.max(1, H * 0.008 * (0.6 + hash(k, 2) * 1.2))
      ctx.globalAlpha = fade * (0.45 + hash(k, 3) * 0.55)
      const segs = 34
      let drawing = false
      ctx.beginPath()
      for (let i = 0; i <= segs; i++) {
        const t = reach * i / segs
        const splay = Math.max(0, t - 0.55) * 1.6
        const gap = edge * 0.22 + Math.max(0, t - 0.5) * 1.3
        const [x, y] = point(t)
        const off = h.off * 2 * half(t) * (1 + splay) + Math.sin(t * 9 + k) * H * 0.002
        if (hash(i, k + 5) < gap) { drawing = false; continue }
        if (!drawing) { ctx.moveTo(x, y + off); drawing = true } else ctx.lineTo(x, y + off)
      }
      ctx.stroke()
    })
    ctx.globalAlpha = fade
    for (let i = 0; i < 26; i++) {
      const [x, y] = point(hash(i, 9) * Math.min(p, 0.35))
      const r = H * (0.003 + hash(i, 10) * 0.012)
      ctx.beginPath(); ctx.arc(x + (hash(i, 11) - 0.5) * H * 0.5, y + (hash(i, 12) - 0.5) * H * 0.42, r, 0, TAU); ctx.fill()
    }
    ctx.restore()
  }

  // Screen-space passes. `cam.width/height` are backing pixels.
  function drawScreen(ctx, fx, cam) {
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    const W = cam.width, H = cam.height
    const st = fx.strike
    if (st) {
      const veil = st.t < 0.15 ? st.t / 0.15 : st.t > 0.6 ? Math.max(0, 1 - (st.t - 0.6) / 0.6) : 1
      ctx.fillStyle = 'rgba(244,238,224,' + 0.55 * veil + ')'
      ctx.fillRect(0, 0, W, H)
      const p = Math.max(0, Math.min(1, (st.t - 0.15) / 0.35))
      const fade = st.t > 0.62 ? Math.max(0, 1 - (st.t - 0.62) / 0.58) : 1
      if (p > 0) drawGreatStroke(ctx, st, W, H, p, fade)
    }
    if (fx.flash > 0.01) {
      ctx.globalAlpha = Math.min(1, fx.flash)
      ctx.fillStyle = fx.flashColor; ctx.fillRect(0, 0, W, H)
      ctx.globalAlpha = 1
    }
    if (fx.invert > 0.02) {
      ctx.globalCompositeOperation = 'difference'
      ctx.fillStyle = 'rgba(240,234,220,' + Math.min(1, fx.invert) + ')'
      ctx.fillRect(0, 0, W, H)
      ctx.globalCompositeOperation = 'source-over'
    }
    if (fx.letterbox > 0.01) {
      const bar = H * 0.085 * fx.letterbox
      ctx.fillStyle = '#121214'
      ctx.fillRect(0, 0, W, bar); ctx.fillRect(0, H - bar, W, bar)
    }
  }

  globalThis.XiabanInkFx = { create, clear, trail, splat, drops, puff, converge, smoke, ring, sparks, seal, ghost, shake, flash, invert, letterbox, strike, update, drawDecals, drawGhosts, drawFront, drawScreen }
})()
