// Desk Pal: walks along the exposed top edges of other windows. Needs the "windows" and "pointer" capabilities.
(() => {
  const nonce = new URLSearchParams(location.search).get('hostNonce')
  const canvas = document.getElementById('c')
  const ctx = canvas.getContext('2d')
  const W = 44, H = 48            // body size; pal.x is the middle of the feet, pal.y their height
  const SPEED = 60, GRAVITY = 1800, RIDE = 80, MIN_SPAN = 40
  let screen = { width: innerWidth, height: innerHeight }
  let windows = [], pointer = null, started = false, frames = 0
  const pal = { x: 0, y: 0, vy: 0, dir: 1, grounded: false, drag: null, rest: 0, blink: 0 }

  const post = (type, extra) => parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...extra }, '*')

  window.addEventListener('message', (event) => {
    const d = event.data
    if (event.source !== parent || !d || d.channel !== 'xiaban.game-host' || d.protocol !== 1 || d.nonce !== nonce) return
    if (d.type === 'host.init') {
      if (d.screen) screen = d.screen
      post('game.ready')
      if (!started) { started = true; requestAnimationFrame(tick) }
    } else if (d.type === 'host.frame') {
      if (d.windows) windows = d.windows
      if (d.pointer) pointer = d.pointer
      frames++
    }
  })

  // The top edge of each window, minus the stretches a window in front covers. Windows arrive front first.
  function floors() {
    const list = []
    windows.forEach((w, i) => {
      if (w.y <= 0) return
      let spans = [[w.x, w.x + w.w]]
      for (let j = 0; j < i && spans.length; j++) {
        const f = windows[j]
        if (f.y <= w.y && w.y < f.y + f.h) spans = spans.flatMap(([a, b]) => [[a, Math.min(b, f.x)], [Math.max(a, f.x + f.w), b]]).filter(([a, b]) => b - a > 0)
      }
      for (const [a, b] of spans) if (b - a >= MIN_SPAN) list.push({ y: w.y, a, b })
    })
    list.push({ y: screen.height, a: 0, b: screen.width })
    return list
  }

  function step(dt) {
    const list = floors()
    if (pal.drag) {
      if (pointer) { pal.x = pointer.x - pal.drag.dx; pal.y = pointer.y - pal.drag.dy }
      pal.vy = 0; pal.grounded = false
      return
    }
    if (pal.grounded) {
      // Stay on the nearest supporting edge, riding it up or down when its window is moved.
      let support = null
      for (const f of list) if (pal.x >= f.a && pal.x <= f.b && Math.abs(f.y - pal.y) <= RIDE && (!support || Math.abs(f.y - pal.y) < Math.abs(support.y - pal.y))) support = f
      if (!support) { pal.grounded = false; return }
      pal.y = support.y
      pal.rest -= dt
      if (pal.rest > 0) return
      pal.x += pal.dir * SPEED * dt
      if (pal.x < support.a + 8 || pal.x > support.b - 8) {
        if (support.y < screen.height && Math.random() < 0.4) { pal.x += pal.dir * 12; pal.grounded = false }
        else pal.dir = -pal.dir
      } else if (Math.random() < dt * 0.15) pal.rest = 1 + Math.random() * 2
      return
    }
    // Falling: land on the highest edge the feet pass through this frame.
    const before = pal.y
    pal.vy += GRAVITY * dt
    pal.y += pal.vy * dt
    let land = null
    for (const f of list) if (pal.x >= f.a && pal.x <= f.b && f.y >= before - 1 && f.y <= pal.y && (!land || f.y < land.y)) land = f
    if (land) { pal.y = land.y; pal.vy = 0; pal.grounded = true }
  }

  let sentAt = 0, sent = null
  function reportRegion(now) {
    const box = { x: Math.round(pal.x - W / 2 - 4), y: Math.round(pal.y - H - 4), w: W + 8, h: H + 8 }
    if (sent && Math.abs(box.x - sent.x) < 6 && Math.abs(box.y - sent.y) < 6 && now - sentAt < 1000) return
    if (now - sentAt < 100) return
    sent = box; sentAt = now
    post('game.regions', { rects: [box] })
  }

  function draw(now) {
    const dpr = devicePixelRatio || 1
    if (canvas.width !== Math.round(innerWidth * dpr)) { canvas.width = Math.round(innerWidth * dpr); canvas.height = Math.round(innerHeight * dpr) }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.clearRect(0, 0, innerWidth, innerHeight)
    const x = pal.x - W / 2, y = pal.y - H
    const bob = pal.grounded && pal.rest <= 0 ? Math.abs(Math.sin(now / 120)) * 3 : 0
    ctx.fillStyle = 'rgba(0,0,0,.18)'
    ctx.beginPath(); ctx.ellipse(pal.x, pal.y, W * 0.45, 4, 0, 0, 7); ctx.fill()
    const body = ctx.createLinearGradient(0, y, 0, y + H)
    body.addColorStop(0, '#ffe7b3'); body.addColorStop(1, '#f2b66b')
    ctx.fillStyle = body
    ctx.beginPath(); ctx.roundRect(x, y - bob, W, H - 4, 16); ctx.fill()
    // Eyes look toward the pointer.
    let lx = 0, ly = 0
    if (pointer) { const dx = pointer.x - pal.x, dy = pointer.y - (pal.y - H / 2); const n = Math.hypot(dx, dy) || 1; lx = dx / n * 2.5; ly = dy / n * 2.5 }
    const shut = now % 4000 < 120
    for (const ex of [pal.x - 9, pal.x + 9]) {
      const ey = y + 18 - bob
      if (shut) { ctx.strokeStyle = '#4a3320'; ctx.lineWidth = 2; ctx.beginPath(); ctx.moveTo(ex - 4, ey); ctx.lineTo(ex + 4, ey); ctx.stroke() }
      else { ctx.fillStyle = '#fff'; ctx.beginPath(); ctx.arc(ex, ey, 6, 0, 7); ctx.fill(); ctx.fillStyle = '#4a3320'; ctx.beginPath(); ctx.arc(ex + lx, ey + ly, 3, 0, 7); ctx.fill() }
    }
  }

  let last = 0
  function tick(now) {
    const dt = Math.min(0.05, (now - last) / 1000 || 0.016)
    last = now
    // Wait for the first reading of the screen, then drop in above the first window.
    if (!pal.y && frames) { pal.x = windows.length ? windows[0].x + windows[0].w / 2 : screen.width / 2; pal.y = 1 }
    if (pal.y) { step(dt); reportRegion(now) }
    draw(now)
    requestAnimationFrame(tick)
  }

  // The mouse only reaches the page over the pal's own box (see game.regions).
  addEventListener('pointerdown', (e) => { pal.drag = { dx: e.clientX - pal.x, dy: e.clientY - pal.y } })
  addEventListener('pointerup', () => { pal.drag = null })
  addEventListener('dblclick', () => post('game.exit'))
})()
