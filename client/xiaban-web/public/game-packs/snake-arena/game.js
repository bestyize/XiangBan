(() => {
  'use strict'

  const rules = globalThis.XiabanSnakeArena
  const viewport = globalThis.XiabanSnakeViewport
  const $ = (id) => document.getElementById(id)
  const gameNode = $('game')
  const canvas = $('arena'), ctx = canvas.getContext('2d', { alpha: false })
  const map = $('map'), mapCtx = map.getContext('2d')
  const entryParams = new URLSearchParams(location.search)
  const nonce = entryParams.get('hostNonce')
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches
  const coarsePointer = matchMedia('(pointer: coarse)').matches
  const foodColors = ['#ff4e6a', '#ffc72c', '#a65cec', '#28b9f5', '#71ce36']
  let mode = entryParams.get('difficulty') === 'lively' ? 'lively' : 'cozy'
  let roundType = entryParams.get('round') === 'endless' ? 'endless' : 'timed'
  let state = rules.createGame({ mode, roundType }), phase = 'welcome', lastResult = null
  let width = 0, height = 0, dpr = 1, camera = { x: 1200, y: 850, zoom: 1 }
  let hostReady = false, roundStarted = false, roundPosted = false, resultPosted = false
  let best = 0, countdown = 0, lastFrame = 0, accumulator = 0, elapsedUI = 0, lastProgress = 0
  let audio = null, sound = false, lastEatTone = 0, toastTimer = 0, wasFullscreen = false, leaving = false
  let joystickPointer = null, boostPointer = null, joystickBase = null
  let direction = null, boost = false, keyboardBoost = false, peakMass = 60
  let needsDraw = true
  let view = null, preferredRotation = 1
  const hostSafeArea = { top: 0, right: 0, bottom: 0, left: 0 }
  const keys = new Set(), particles = [], floats = []

  function postToHost(type, payload = {}) {
    if (!hostReady || !nonce || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...payload }, '*')
  }

  function syncRound() {
    if (!hostReady || !roundStarted || resultPosted) return
    if (!roundPosted) { roundPosted = true; postToHost('game.start') }
    postToHost('game.progress', { score: state.snakes[0].score })
    if (state.phase === 'over' && !resultPosted) { resultPosted = true; postToHost('game.result', { score: state.snakes[0].score }) }
  }

  window.addEventListener('message', (event) => {
    const data = event.data
    if (!nonce || event.source !== window.parent || !data || data.channel !== 'xiaban.game-host' || data.protocol !== 1 || data.nonce !== nonce || !['host.init', 'host.layout'].includes(data.type)) return
    if (data.safeArea && typeof data.safeArea === 'object') {
      for (const side of ['top', 'right', 'bottom', 'left']) {
        const value = data.safeArea[side]
        if (Number.isFinite(value) && value >= 0 && value <= 160) hostSafeArea[side] = value
      }
      applySafeArea()
    }
    if (data.type === 'host.layout') return
    if (hostReady) { postToHost('game.ready'); return }
    hostReady = true
    postToHost('game.ready')
    syncRound()
  })

  function tone(frequency, duration = 0.08, volume = 0.035, delay = 0) {
    if (!sound || !audio || audio.state !== 'running') return
    const oscillator = audio.createOscillator(), gain = audio.createGain()
    const at = audio.currentTime + delay
    oscillator.type = 'sine'
    oscillator.frequency.setValueAtTime(frequency, at)
    oscillator.frequency.exponentialRampToValueAtTime(frequency * 0.75, at + duration)
    gain.gain.setValueAtTime(0, at)
    gain.gain.linearRampToValueAtTime(volume, at + 0.008)
    gain.gain.exponentialRampToValueAtTime(0.001, at + duration)
    oscillator.connect(gain); gain.connect(audio.destination)
    oscillator.start(at); oscillator.stop(at + duration + 0.02)
    oscillator.onended = () => { oscillator.disconnect(); gain.disconnect() }
  }

  function unlockAudio() {
    if (!sound) return
    try {
      const Audio = window.AudioContext || window.webkitAudioContext
      if (!audio && Audio) audio = new Audio()
      if (audio?.state === 'suspended') void audio.resume().catch(() => {})
    } catch { sound = false; updateSoundButton() }
  }

  function updateSoundButton() {
    $('soundButton').setAttribute('aria-pressed', String(sound))
    $('soundButton').setAttribute('aria-label', sound ? '关闭音效' : '开启音效')
    $('soundWaves').setAttribute('d', sound ? 'M15 8q4 4 0 8m3-11q7 7 0 14' : 'm16 9 5 6m0-6-5 6')
  }

  function toast(message, duration = 2300) {
    clearTimeout(toastTimer)
    $('toast').textContent = message
    $('toast').hidden = false
    toastTimer = setTimeout(() => { $('toast').hidden = true }, duration)
  }

  async function requestImmersive(feedback = true) {
    const root = document.documentElement
    const request = root.requestFullscreen || root.webkitRequestFullscreen
    if (!request || document.fullscreenEnabled === false) {
      if (feedback) toast('当前浏览器暂不支持全屏，画面已自动适配横玩。', 3400)
      return
    }
    try {
      if (!document.fullscreenElement && !document.webkitFullscreenElement) await request.call(root)
      if (screen.orientation?.lock) {
        try { await screen.orientation.lock('landscape') } catch { /* Manual rotation remains available. */ }
      }
    } catch {
      if (feedback) toast('全屏暂未开启，横持手机仍可继续玩。', 3000)
    }
  }

  const fullscreenAvailable = Boolean(document.documentElement.requestFullscreen || document.documentElement.webkitRequestFullscreen) && document.fullscreenEnabled !== false
  $('fullscreenButton').hidden = !fullscreenAvailable

  function fullscreenChange() {
    const active = Boolean(document.fullscreenElement || document.webkitFullscreenElement)
    $('fullscreenButton').setAttribute('aria-label', active ? '退出全屏' : '进入全屏')
    if (wasFullscreen && !active && !leaving) pause('已退出全屏，对局替你暂停了。')
    wasFullscreen = active
    resize()
  }
  document.addEventListener('fullscreenchange', fullscreenChange)
  document.addEventListener('webkitfullscreenchange', fullscreenChange)

  function resetInput() {
    const previousJoystick = joystickPointer, previousBoost = boostPointer
    direction = null; boost = false; keyboardBoost = false
    keys.clear(); joystickPointer = null; boostPointer = null; joystickBase = null
    if (previousJoystick?.node.hasPointerCapture(previousJoystick.id)) previousJoystick.node.releasePointerCapture(previousJoystick.id)
    if (previousBoost !== null && $('boostButton').hasPointerCapture(previousBoost)) $('boostButton').releasePointerCapture(previousBoost)
    $('stick').style.transform = ''
    $('joystick').style.left = ''; $('joystick').style.top = ''; $('joystick').style.bottom = ''
    $('joystick').classList.remove('active')
    $('boostButton').setAttribute('aria-pressed', 'false')
  }

  function pause(message = '喝口水，再回来继续长大。') {
    if (phase !== 'playing') return
    phase = 'paused'
    resetInput()
    $('pauseDescription').textContent = message
    $('pauseOverlay').hidden = false
    $('controls').hidden = true
    $('tutorial').hidden = true
    $('toast').hidden = true
    accumulator = 0
    syncRound()
    $('resumeButton').focus({ preventScroll: true })
  }

  function applySafeArea() {
    if (!view) return
    const style = getComputedStyle(document.documentElement)
    const physical = {}
    for (const side of ['top', 'right', 'bottom', 'left']) {
      const local = Math.max(0, Math.min(160, parseFloat(style.getPropertyValue('--viewport-safe-' + side)) || 0))
      physical[side] = Math.max(local, hostSafeArea[side])
    }
    const logical = viewport.safeArea(view, physical)
    for (const side of ['top', 'right', 'bottom', 'left']) gameNode.style.setProperty('--safe-' + side, logical[side] + 'px')
  }

  function resize() {
    needsDraw = true
    const previous = view
    view = viewport.layout(window.innerWidth, window.innerHeight, preferredRotation)
    width = view.width; height = view.height
    gameNode.style.width = width + 'px'; gameNode.style.height = height + 'px'
    gameNode.style.transform = viewport.transform(view)
    gameNode.dataset.rotation = String(view.rotation)
    for (const id of ['flipButton', 'welcomeFlip']) $(id).hidden = view.rotation === 0
    applySafeArea()
    // Cap backing pixels on high density phones; small sprites stay sharp without a 3x fill cost.
    dpr = Math.min(2, window.devicePixelRatio || 1)
    canvas.width = Math.round(width * dpr); canvas.height = Math.round(height * dpr)
    if (previous && (previous.physicalWidth !== view.physicalWidth || previous.physicalHeight !== view.physicalHeight || previous.rotation !== view.rotation)) {
      resetInput()
      // Native fullscreen may change orientation during the countdown; let the first start continue.
      if (previous.rotation !== view.rotation && state.time > 0) pause('画面方向已调整，准备好后继续这一局。')
    }
  }
  window.addEventListener('resize', resize)
  window.visualViewport?.addEventListener('resize', resize)

  function startRound() {
    unlockAudio()
    if (coarsePointer && !document.fullscreenElement) void requestImmersive(false)
    state = rules.createGame({ mode, roundType })
    lastResult = null
    $('shareResultButton').disabled = false
    phase = 'playing'; roundStarted = true; roundPosted = false; resultPosted = false
    peakMass = state.snakes[0].mass
    resetInput(); particles.length = 0; floats.length = 0
    camera = { x: state.snakes[0].x, y: state.snakes[0].y, zoom: desiredZoom() }
    countdown = 2.4; accumulator = 0; lastProgress = 0; elapsedUI = 1
    $('welcome').hidden = true; $('pauseOverlay').hidden = true; $('resultOverlay').hidden = true
    for (const id of ['hud', 'leaderboard', 'mapPanel', 'controls', 'tutorial']) $(id).hidden = false
    $('tutorial').style.opacity = '1'
    $('startButton').blur(); $('restartButton').blur()
    syncRound(); updateHUD(); resize()
    if (phase === 'playing') toast('3', 800)
  }

  function finishRound() {
    phase = 'result'
    resetInput()
    const player = state.snakes[0]
    lastResult = { score: player.score, mass: Math.floor(player.mass), kills: player.kills, seconds: Math.floor(state.time), roundType: state.roundType, difficulty: state.mode }
    const rank = rules.rankings(state).findIndex((entry) => entry.id === 0) + 1
    const isBest = player.score > best
    best = Math.max(best, player.score)
    $('resultScore').textContent = player.score
    $('resultMass').textContent = Math.floor(player.mass)
    $('resultKills').textContent = player.kills
    $('resultTime').textContent = formatTime(Math.floor(state.time))
    $('bestLabel').textContent = isBest ? '✦ 刷新了本次游玩的最高积分' : '本次游玩最高积分 ' + best
    $('resultBadge').textContent = state.roundType === 'endless' ? '无尽生存 · 长度第 ' + rank + ' 名' : state.reason === 'timeout' ? '三分钟完成 · 长度第 ' + rank + ' 名' : '快乐收集完成'
    $('resultTitle').textContent = state.reason === 'timeout' && rank === 1 ? '今晚，你是快乐小巨蛇' : player.kills > 0 ? '小小一条，也能漂亮反杀' : '今天也有好好长大'
    $('resultDescription').textContent = state.reason === 'wall' ? '小伴：边界有点硬，下次记得转弯。' : state.reason === 'collision' ? '小伴：快乐没白吃，下局绕开它。' : '小伴：吃到的快乐，都算数。'
    $('resultOverlay').hidden = false; $('controls').hidden = true; $('tutorial').hidden = true; $('toast').hidden = true
    syncRound(); updateHUD(); needsDraw = true
    tone(392, 0.13); tone(523, 0.15, 0.03, 0.12)
    $('restartButton').focus({ preventScroll: true })
  }

  async function shareResult() {
    if (phase !== 'result' || !lastResult) return
    if (window.parent !== window && !hostReady) { toast('战绩页还在连接，请稍后再试。'); return }
    $('shareResultButton').disabled = true
    try {
      try { screen.orientation?.unlock?.() } catch { /* Sharing uses the ordinary portrait page. */ }
      try {
        const exit = document.exitFullscreen || document.webkitExitFullscreen
        if (exit && (document.fullscreenElement || document.webkitFullscreenElement)) await exit.call(document)
      } catch { /* Fullscreen failure must not prevent opening the result page. */ }
      if (hostReady && window.parent !== window) postToHost('game.share', { result: lastResult })
      else {
        const result = lastResult
        const fragment = new URLSearchParams({ v: '1', s: String(result.score), l: String(result.mass), k: String(result.kills), t: String(result.seconds), r: result.roundType, d: result.difficulty })
        window.location.assign('/games/snake-arena/share?own=1#' + fragment)
      }
    } finally { $('shareResultButton').disabled = false }
  }

  async function exitGame() {
    leaving = true
    resetInput()
    if (roundStarted) syncRound()
    try { screen.orientation?.unlock?.() } catch { /* Browser may not expose orientation controls. */ }
    try {
      const exit = document.exitFullscreen || document.webkitExitFullscreen
      if (exit && (document.fullscreenElement || document.webkitFullscreenElement)) await exit.call(document)
    } catch { /* Navigation also exits fullscreen. */ }
    if (hostReady) postToHost('game.exit')
    else window.location.assign('/games')
  }

  $('startButton').addEventListener('click', startRound)
  $('restartButton').addEventListener('click', startRound)
  $('shareResultButton').addEventListener('click', () => { void shareResult() })
  $('resumeButton').addEventListener('click', () => {
    if (document.hidden) return
    unlockAudio(); resetInput(); phase = 'playing'; accumulator = 0
    $('pauseOverlay').hidden = true; $('controls').hidden = false
    $('resumeButton').blur()
    if (countdown > 0) toast(String(Math.ceil(countdown / 0.8)), 800)
    else toast('继续长大吧', 1200)
  })
  for (const id of ['exitButton', 'welcomeExit', 'pauseExit', 'resultExit']) $(id).addEventListener('click', () => { void exitGame() })
  $('pauseButton').addEventListener('click', () => pause())
  for (const id of ['flipButton', 'welcomeFlip']) $(id).addEventListener('click', () => {
    if (!view.rotation) return
    pause('画面已换到另一边，准备好后继续。')
    preferredRotation *= -1
    resize()
  })
  $('fullscreenButton').addEventListener('click', () => {
    if (document.fullscreenElement || document.webkitFullscreenElement) {
      const exit = document.exitFullscreen || document.webkitExitFullscreen
      if (exit) void Promise.resolve(exit.call(document)).catch(() => {})
    } else void requestImmersive()
  })
  $('soundButton').addEventListener('click', () => { sound = !sound; unlockAudio(); updateSoundButton(); tone(660) })
  for (const button of document.querySelectorAll('[data-mode]')) button.addEventListener('click', () => {
    mode = button.dataset.mode
    for (const entry of document.querySelectorAll('[data-mode]')) {
      const active = entry === button
      entry.classList.toggle('active', active); entry.setAttribute('aria-pressed', String(active))
    }
  })
  for (const button of document.querySelectorAll('[data-mode]')) {
    const active = button.dataset.mode === mode
    button.classList.toggle('active', active); button.setAttribute('aria-pressed', String(active))
  }
  for (const button of document.querySelectorAll('[data-round-type]')) button.addEventListener('click', () => {
    roundType = button.dataset.roundType
    for (const entry of document.querySelectorAll('[data-round-type]')) {
      const active = entry === button
      entry.classList.toggle('active', active); entry.setAttribute('aria-pressed', String(active))
    }
    $('roundSummary').textContent = roundType === 'endless' ? '不限时，活着就继续' : '三分钟一局'
  })
  for (const button of document.querySelectorAll('[data-round-type]')) {
    const active = button.dataset.roundType === roundType
    button.classList.toggle('active', active); button.setAttribute('aria-pressed', String(active))
  }
  $('roundSummary').textContent = roundType === 'endless' ? '不限时，活着就继续' : '三分钟一局'

  function joystickDown(event, node) {
    if (phase !== 'playing' || joystickPointer !== null || event.button !== 0) return
    const point = viewport.point(view, event.clientX, event.clientY)
    if (node === canvas && (event.pointerType === 'mouse' || point.x > width * 0.65)) return
    event.preventDefault()
    joystickPointer = { id: event.pointerId, node }
    node.setPointerCapture(event.pointerId)
    const stickSize = $('joystick').offsetWidth
    const rect = $('joystick').getBoundingClientRect()
    joystickBase = node === canvas ? point : viewport.point(view, rect.left + rect.width / 2, rect.top + rect.height / 2)
    if (node === canvas) {
      $('joystick').style.left = (joystickBase.x - stickSize / 2) + 'px'
      $('joystick').style.top = (joystickBase.y - stickSize / 2) + 'px'
      $('joystick').style.bottom = 'auto'
    }
    $('joystick').classList.add('active')
    joystickMove(event)
  }

  function joystickMove(event) {
    if (joystickPointer?.id !== event.pointerId || !joystickBase) return
    event.preventDefault()
    const point = viewport.point(view, event.clientX, event.clientY)
    const dx = point.x - joystickBase.x, dy = point.y - joystickBase.y
    const distance = Math.hypot(dx, dy), limit = $('joystick').offsetWidth * 0.29
    const factor = distance > limit ? limit / distance : 1
    $('stick').style.transform = 'translate(' + dx * factor + 'px,' + dy * factor + 'px)'
    if (distance > 7) direction = Math.atan2(dy, dx)
  }

  function joystickUp(event) {
    if (joystickPointer?.id !== event.pointerId) return
    joystickPointer = null; joystickBase = null; direction = null
    $('stick').style.transform = ''
    $('joystick').style.left = ''; $('joystick').style.top = ''; $('joystick').style.bottom = ''
    $('joystick').classList.remove('active')
  }
  for (const node of [canvas, $('joystick')]) {
    node.addEventListener('pointerdown', (event) => joystickDown(event, node))
    node.addEventListener('pointermove', joystickMove)
    for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) node.addEventListener(type, joystickUp)
  }
  canvas.addEventListener('pointermove', (event) => {
    if (event.pointerType !== 'mouse' || phase !== 'playing' || keys.size) return
    const player = state.snakes[0]
    const x = width / 2 + (player.x - camera.x) * camera.zoom
    const y = height / 2 + (player.y - camera.y) * camera.zoom
    const point = viewport.point(view, event.clientX, event.clientY)
    if (Math.hypot(point.x - x, point.y - y) > 16) direction = Math.atan2(point.y - y, point.x - x)
  })

  $('boostButton').addEventListener('pointerdown', (event) => {
    if (phase !== 'playing' || event.button !== 0 || boostPointer !== null) return
    event.preventDefault(); boostPointer = event.pointerId; boost = true
    $('boostButton').setPointerCapture(event.pointerId)
    $('boostButton').setAttribute('aria-pressed', 'true')
    tone(180, 0.07, 0.025)
  })
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) $('boostButton').addEventListener(type, (event) => {
    if (boostPointer !== event.pointerId) return
    boostPointer = null; boost = false
    $('boostButton').setAttribute('aria-pressed', String(keyboardBoost))
  })

  const controlKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'w', 'a', 's', 'd']
  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return
    if (event.key === 'Tab') {
      const dialog = phase === 'paused' ? $('pauseOverlay') : phase === 'result' ? $('resultOverlay') : phase === 'welcome' ? $('welcome') : null
      if (dialog) {
        const buttons = [...dialog.querySelectorAll('button')].filter((button) => !button.disabled && button.getBoundingClientRect().width > 0)
        const index = buttons.indexOf(document.activeElement)
        const next = index < 0 ? 0 : (index + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length
        event.preventDefault(); buttons[next]?.focus()
      }
      return
    }
    if (event.key === 'Escape' || event.key.toLowerCase() === 'p') { if (phase === 'playing') { event.preventDefault(); pause() } return }
    if (phase !== 'playing') return
    const key = event.key.length === 1 ? event.key.toLowerCase() : event.key
    if (controlKeys.includes(key)) { event.preventDefault(); keys.add(key) }
    if (event.code === 'Space' && (!(document.activeElement instanceof HTMLButtonElement) || document.activeElement === $('boostButton'))) {
      event.preventDefault(); keyboardBoost = true; $('boostButton').setAttribute('aria-pressed', 'true')
    }
  })
  document.addEventListener('keyup', (event) => {
    keys.delete(event.key.length === 1 ? event.key.toLowerCase() : event.key)
    if (event.code === 'Space') { keyboardBoost = false; $('boostButton').setAttribute('aria-pressed', String(boost)) }
  })
  document.addEventListener('visibilitychange', () => { if (document.hidden) pause('离开页面时已经暂停，回来可以接着玩。') })
  window.addEventListener('blur', () => { resetInput(); pause('操作暂时中断了，准备好后再继续。') })
  window.addEventListener('pagehide', () => { syncRound(); if (audio) void audio.close().catch(() => {}) })
  document.addEventListener('contextmenu', (event) => event.preventDefault())

  function inputAngle() {
    const x = Number(keys.has('ArrowRight') || keys.has('d')) - Number(keys.has('ArrowLeft') || keys.has('a'))
    const y = Number(keys.has('ArrowDown') || keys.has('s')) - Number(keys.has('ArrowUp') || keys.has('w'))
    return x || y ? Math.atan2(y, x) : direction
  }

  function formatTime(seconds) { return Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2, '0') }
  function updateHUD() {
    const player = state.snakes[0]
    $('score').textContent = player.score
    $('mass').textContent = Math.floor(player.mass)
    const endless = state.roundType === 'endless'
    $('time').textContent = formatTime(endless ? Math.floor(state.time) : Math.ceil(state.remaining))
    $('clockLabel').textContent = endless ? '无尽生存' : '三分钟竞技'
    document.querySelector('.clock').classList.toggle('urgent', !endless && state.remaining < 30)
    $('boostButton').style.setProperty('--energy', player.energy)
    $('boostButton').classList.toggle('exhausted', player.energy < 5 || player.mass <= rules.MIN_MASS)
    const ranked = rules.rankings(state)
    const playerRank = ranked.findIndex((entry) => entry.id === 0) + 1
    $('rankLabel').textContent = '第 ' + playerRank + ' 名'
    $('rankings').replaceChildren(...ranked.slice(0, height < 500 ? 3 : 5).map((entry, index) => {
      const row = document.createElement('li')
      if (entry.id === 0) row.className = 'mine'
      const number = document.createElement('span'); number.className = 'rank-number'; number.textContent = index + 1
      const dot = document.createElement('span'); dot.className = 'rank-dot'; dot.style.background = entry.color
      const name = document.createElement('span'); name.className = 'rank-name'; name.textContent = entry.name + (entry.id === 0 ? ' · 我' : '')
      const mass = document.createElement('span'); mass.className = 'rank-mass'; mass.textContent = Math.floor(entry.mass)
      row.append(number, dot, name, mass)
      return row
    }))
    mapCtx.fillStyle = '#dcead6'; mapCtx.fillRect(0, 0, 180, 128)
    const sx = 180 / rules.WORLD.width, sy = 128 / rules.WORLD.height
    mapCtx.fillStyle = '#eff5e9'
    mapCtx.fillRect((camera.x - width / camera.zoom / 2) * sx, (camera.y - height / camera.zoom / 2) * sy, width / camera.zoom * sx, height / camera.zoom * sy)
    for (const snake of state.snakes) {
      if (!snake.alive) continue
      mapCtx.beginPath(); mapCtx.arc(snake.x * sx, snake.y * sy, snake.id === 0 ? 4 : 2.5, 0, Math.PI * 2)
      mapCtx.fillStyle = snake.color; mapCtx.fill()
      if (snake.id === 0) { mapCtx.strokeStyle = '#fffdf5'; mapCtx.lineWidth = 2; mapCtx.stroke() }
    }
  }

  function addParticles(x, y, color, count = 5, force = 40) {
    if (reducedMotion) return
    for (let i = 0; i < count && particles.length < 105; i += 1) {
      const angle = Math.random() * Math.PI * 2, speed = force * (0.5 + Math.random())
      particles.push({ x, y, vx: Math.cos(angle) * speed, vy: Math.sin(angle) * speed, age: 0, life: 0.4 + Math.random() * 0.25, color, size: 2 + Math.random() * 3 })
    }
  }

  function handleEvents(events) {
    for (const event of events) {
      if (event.type === 'eat') {
        addParticles(event.x, event.y, foodColors[event.color], event.value > 1 ? 7 : 3)
        if (event.value > 1 && floats.length < 12) floats.push({ x: event.x, y: event.y, text: '+' + event.value * 10, age: 0, color: '#ab8950' })
        if (performance.now() - lastEatTone > 85) { tone(event.value > 1 ? 880 : 660, 0.07, 0.02); lastEatTone = performance.now() }
      }
      if (event.type === 'death') {
        addParticles(event.x, event.y, event.color, 17, 95)
        if (event.killerId === 0 && event.id !== 0) {
          toast('漂亮！' + state.snakes.find((entry) => entry.id === event.id).name + ' 撞上了你的身体 · +80', 2600)
          floats.push({ x: event.x, y: event.y, text: '+80 反杀', age: 0, color: '#7b9b63' })
          tone(660, 0.1); tone(880, 0.13, 0.03, 0.09)
        }
        if (event.id === 0) { tone(330, 0.16, 0.035); tone(220, 0.18, 0.025, 0.12) }
      }
    }
    const player = state.snakes[0]
    if (player.mass >= 150 && peakMass < 150) toast('长成小巨蛇啦，转弯记得留点空间。')
    peakMass = Math.max(peakMass, player.mass)
  }

  function desiredZoom() {
    const base = Math.max(0.58, Math.min(1.1, height / 510))
    return base * Math.max(0.77, 1 - Math.max(0, state.snakes[0].mass - 100) / 1500)
  }

  const tile = document.createElement('canvas')
  tile.width = 96; tile.height = 96
  const tileCtx = tile.getContext('2d')
  tileCtx.fillStyle = '#eaf3e7'; tileCtx.fillRect(0, 0, 96, 96)
  tileCtx.strokeStyle = '#d9e7d333'; tileCtx.lineWidth = 1
  tileCtx.beginPath(); tileCtx.moveTo(0.5, 0); tileCtx.lineTo(0.5, 96); tileCtx.moveTo(0, 0.5); tileCtx.lineTo(96, 0.5); tileCtx.stroke()
  for (const x of [24, 72]) for (const y of [24, 72]) { tileCtx.fillStyle = '#b8d1ac47'; tileCtx.beginPath(); tileCtx.arc(x, y, 1.1, 0, Math.PI * 2); tileCtx.fill() }
  const pattern = ctx.createPattern(tile, 'repeat')

  function foodSprite(color, special) {
    const sprite = document.createElement('canvas'); sprite.width = 64; sprite.height = 64
    const c = sprite.getContext('2d')
    c.save(); c.translate(32, 32)
    const radius = special ? 8 : 5.8
    // Each food dot is a single solid color, with no gloss or shaded edge.
    c.fillStyle = color; c.beginPath(); c.arc(0, 0, radius, 0, Math.PI * 2); c.fill()
    c.restore(); return sprite
  }
  const snacks = foodColors.map((color) => foodSprite(color, false))
  const treats = foodColors.map((color) => foodSprite(color, true))

  function skinSprite(color) {
    const sprite = document.createElement('canvas'); sprite.width = 64; sprite.height = 64
    const c = sprite.getContext('2d')
    c.fillStyle = color; c.beginPath(); c.arc(32, 32, 24, 0, Math.PI * 2); c.fill()
    return sprite
  }
  const skins = new Map(rules.COLORS.map((color) => [color, skinSprite(color)]))

  function drawSnake(snake) {
    if (!snake.alive) return
    const points = rules.bodyPoints(snake), r = rules.radius(snake)
    const halfW = width / camera.zoom / 2 + 55, halfH = height / camera.zoom / 2 + 55
    if (points.every((point) => Math.abs(point.x - camera.x) > halfW || Math.abs(point.y - camera.y) > halfH)) return
    ctx.save()
    ctx.lineCap = 'round'; ctx.lineJoin = 'round'
    // Crisp speed marks keep boosted snakes readable against the bright dots.
    if (snake.boosting) {
      ctx.save(); ctx.translate(snake.x, snake.y); ctx.rotate(snake.angle)
      ctx.lineWidth = 2; ctx.strokeStyle = snake.color
      for (const side of [-1, 1]) {
        ctx.beginPath(); ctx.moveTo(-r, side * (r + 9)); ctx.lineTo(-r * 3, side * (r + 9)); ctx.stroke()
        ctx.beginPath(); ctx.moveTo(-r * 2, side * (r + 16)); ctx.lineTo(-r * 3.5, side * (r + 16)); ctx.stroke()
      }
      ctx.restore()
    }
    const skin = skins.get(snake.color)
    const spacing = r * 1.3, beads = []
    let travelled = 0, nextBead = spacing
    for (let index = 1; index < points.length; index += 1) {
      const a = points[index - 1], b = points[index]
      const distance = Math.hypot(b.x - a.x, b.y - a.y)
      while (distance > 0 && nextBead <= travelled + distance) {
        const t = (nextBead - travelled) / distance
        beads.push({ x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t })
        nextBead += spacing
      }
      travelled += distance
    }
    for (let index = beads.length - 1; index >= 0; index -= 1) {
      const size = r * 64 / 24
      const point = beads[index]
      ctx.drawImage(skin, point.x - size / 2, point.y - size / 2, size, size)
    }

    ctx.translate(snake.x, snake.y); ctx.rotate(snake.angle)
    ctx.fillStyle = snake.color; ctx.beginPath(); ctx.arc(0, 0, r * 1.06, 0, Math.PI * 2); ctx.fill()
    const look = Math.max(-1.6, Math.min(1.6, rules.angleDelta(snake.angle, snake.targetAngle) * 2))
    for (const sign of [-1, 1]) {
      ctx.fillStyle = '#ffffff'; ctx.beginPath(); ctx.arc(r * 0.38, sign * r * 0.42, r * 0.36, 0, Math.PI * 2); ctx.fill()
      ctx.fillStyle = '#263447'; ctx.beginPath(); ctx.arc(r * 0.54, sign * r * 0.42 + look, r * 0.18, 0, Math.PI * 2); ctx.fill()
    }
    ctx.restore()
    ctx.save()
    ctx.font = (snake.id === 0 ? '600 ' : '') + '11px system-ui, sans-serif'
    ctx.textAlign = 'center'; ctx.textBaseline = 'bottom'; ctx.fillStyle = snake.id === 0 ? '#a78347' : '#789583'
    ctx.fillText(snake.name + (snake.id === 0 ? ' · 我' : ''), snake.x, snake.y - r - 13)
    if (snake.invulnerable > 0) {
      ctx.strokeStyle = snake.color; ctx.lineWidth = 1.5; ctx.setLineDash([4, 5]); ctx.beginPath(); ctx.arc(snake.x, snake.y, r + 14, 0, Math.PI * 2); ctx.stroke()
    }
    ctx.restore()
  }

  function draw(dt) {
    const player = state.snakes[0]
    const smooth = 1 - Math.exp(-dt * (reducedMotion ? 22 : 8))
    camera.x += (player.x - camera.x) * smooth; camera.y += (player.y - camera.y) * smooth
    camera.zoom += (desiredZoom() - camera.zoom) * (1 - Math.exp(-dt * 3))
    const left = camera.x - width / camera.zoom / 2, right = camera.x + width / camera.zoom / 2
    const top = camera.y - height / camera.zoom / 2, bottom = camera.y + height / camera.zoom / 2
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.fillStyle = '#d3e4cf'; ctx.fillRect(0, 0, width, height)
    ctx.translate(width / 2, height / 2); ctx.scale(camera.zoom, camera.zoom); ctx.translate(-camera.x, -camera.y)
    ctx.fillStyle = pattern; ctx.fillRect(0, 0, rules.WORLD.width, rules.WORLD.height)
    ctx.save(); ctx.strokeStyle = '#bad1ae'; ctx.lineWidth = 7; ctx.strokeRect(4, 4, rules.WORLD.width - 8, rules.WORLD.height - 8)
    const wallDistance = Math.min(player.x, player.y, rules.WORLD.width - player.x, rules.WORLD.height - player.y)
    ctx.strokeStyle = wallDistance < 110 ? '#d9a394' : '#f8fbefb3'; ctx.lineWidth = 2; ctx.setLineDash([11, 9]); ctx.strokeRect(15, 15, rules.WORLD.width - 30, rules.WORLD.height - 30); ctx.restore()
    // A few tiny clovers make the map feel like a meadow, while keeping danger easy to read.
    for (let i = 0; i < 32; i += 1) {
      const x = 90 + (i * 317) % 2210, y = 95 + (i * 223) % 1500
      if (x < left - 20 || x > right + 20 || y < top - 20 || y > bottom + 20) continue
      ctx.fillStyle = '#c2d9b64d'
      for (const sign of [-1, 1]) { ctx.beginPath(); ctx.ellipse(x + sign * 3, y, 3, 6, sign * 0.6, 0, Math.PI * 2); ctx.fill() }
    }
    for (const food of state.food) {
      if (food.x < left - 25 || food.x > right + 25 || food.y < top - 25 || food.y > bottom + 25) continue
      const sprite = food.value > 1 ? treats[food.color] : snacks[food.color]
      const size = food.value > 1 ? 60 : 52
      ctx.drawImage(sprite, food.x - size / 2, food.y - size / 2, size, size)
    }
    for (const snake of state.snakes) if (snake.id !== 0) drawSnake(snake)
    drawSnake(player)
    for (let i = particles.length - 1; i >= 0; i -= 1) {
      const p = particles[i]; p.age += dt
      if (p.age > p.life) { particles.splice(i, 1); continue }
      p.x += p.vx * dt; p.y += p.vy * dt; p.vx *= 0.96; p.vy *= 0.96
      ctx.globalAlpha = 1 - p.age / p.life; ctx.fillStyle = p.color
      ctx.beginPath(); ctx.arc(p.x, p.y, p.size * (1 - p.age / p.life * 0.5), 0, Math.PI * 2); ctx.fill()
    }
    ctx.globalAlpha = 1
    for (let i = floats.length - 1; i >= 0; i -= 1) {
      const f = floats[i]; f.age += dt
      if (f.age > 0.8) { floats.splice(i, 1); continue }
      ctx.globalAlpha = Math.min(1, (0.8 - f.age) * 3); ctx.fillStyle = f.color; ctx.font = '600 12px system-ui, sans-serif'; ctx.textAlign = 'center'
      ctx.fillText(f.text, f.x, f.y - 18 - f.age * 28)
    }
    ctx.globalAlpha = 1
  }

  function frame(now) {
    const dt = Math.min(0.07, (now - (lastFrame || now)) / 1000)
    lastFrame = now
    if (!leaving && phase === 'playing') {
      if (countdown > 0) {
        const before = Math.ceil(countdown / 0.8)
        countdown = Math.max(0, countdown - dt)
        const after = Math.ceil(countdown / 0.8)
        if (before !== after) { toast(after ? String(after) : '出发！开局有 ' + (mode === 'cozy' ? 5 : 3) + ' 秒保护，先试试转向。', after ? 800 : 2800); tone(after ? 440 : 660) }
      } else {
        accumulator += dt
        const input = { angle: inputAngle(), boost: boost || keyboardBoost }
        let ticks = 0
        while (accumulator >= 1 / 60 && ticks < 4 && state.phase === 'playing') {
          handleEvents(rules.step(state, input, 1 / 60)); accumulator -= 1 / 60; ticks += 1
        }
        if (ticks === 4) accumulator = Math.min(accumulator, 1 / 60)
        if (state.phase === 'over') finishRound()
        if (state.time > 10 && !$('tutorial').hidden) { $('tutorial').style.opacity = '0'; if (state.time > 12) $('tutorial').hidden = true }
        if (state.time - lastProgress >= 1.5) { lastProgress = state.time; syncRound() }
      }
      elapsedUI += dt
      if (elapsedUI > 0.22) { updateHUD(); elapsedUI = 0 }
    }
    if (!document.hidden && (phase === 'playing' || needsDraw || (phase === 'result' && particles.length))) {
      draw(phase === 'paused' ? 0 : dt)
      needsDraw = false
    }
    requestAnimationFrame(frame)
  }

  resize(); updateSoundButton(); updateHUD()
  camera.zoom = desiredZoom()
  requestAnimationFrame(frame)
})()
