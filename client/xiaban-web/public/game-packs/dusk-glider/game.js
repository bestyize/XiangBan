/* 暮色滑翔 · controller: host protocol, input, round lifecycle, camera, rendering, HUD. */
(() => {
  'use strict'

  const THREE = globalThis.THREE
  const rules = globalThis.XiabanDuskGlider
  const World = globalThis.XiabanGliderWorld
  const Bird = globalThis.XiabanGliderBird
  const audio = globalThis.XiabanGliderAudio
  const viewport = globalThis.XiabanGliderViewport
  const $ = (id) => document.getElementById(id)
  const gameNode = $('game')
  const canvas = $('scene')
  const params = new URLSearchParams(location.search)
  const nonce = params.get('hostNonce')
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches
  const GAME_ID = 'dusk-glider'
  const MAX_STEP = 1 / 60
  const DAY_SCALE = 60

  let touchUI = matchMedia('(any-pointer: coarse)').matches
  gameNode.classList.toggle('touch', touchUI)
  let phase = 'welcome'
  let state = rules.createGame({ seed: 20261007 })
  let best = 0, resultDelay = 0
  let view = null, preferredRotation = 1, width = 0, height = 0
  let hostReady = false, roundStarted = false, roundPosted = false, resultPosted = false
  let lastFrame = 0, clock = 0, lastProgress = 0, leaving = false, wasFullscreen = false
  let toastTimer = 0, promptTimer = 0, calloutTimer = 0, islandTimer = 0
  let quality = touchUI ? 1 : 2, slowFrames = 0, frameAvg = 16
  const shown = new Set()
  const hostSafeArea = { top: 0, right: 0, bottom: 0, left: 0 }

  // ------------------------------------------------------------ renderer

  let renderer = null
  try {
    renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance', alpha: false })
  } catch {
    renderer = null
  }
  if (!renderer) {
    $('fallback').hidden = false
    $('welcome').hidden = true
    $('fallbackExit').addEventListener('click', () => { if (hostReady) postToHost('game.exit'); else location.assign('/games') })
  }

  const scene = new THREE.Scene()
  const camera = new THREE.PerspectiveCamera(34, 16 / 9, 0.5, 3200)
  let world = null, bird = null
  if (renderer) {
    renderer.toneMapping = THREE.ACESFilmicToneMapping
    renderer.toneMappingExposure = 1.18
    renderer.outputColorSpace = THREE.SRGBColorSpace
    renderer.shadowMap.enabled = true
    renderer.shadowMap.type = THREE.PCFShadowMap
    world = World.create(THREE, { scene, rules, state, quality, props: globalThis.XiabanGliderProps })
    bird = Bird.create(THREE, scene, { quality })
    bird.reset(state)
  }

  // ------------------------------------------------------------ host protocol

  function postToHost(type, payload = {}) {
    if (!hostReady || !nonce || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...payload }, '*')
  }

  function metrics() { return { score: state.score, lines: state.perfects, maxTile: state.maxChain } }

  function syncRound() {
    if (!hostReady || !roundStarted || resultPosted) return
    if (!roundPosted) { roundPosted = true; postToHost('game.start') }
    postToHost('game.progress', metrics())
    if (phase === 'result') { resultPosted = true; postToHost('game.result', metrics()) }
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

  // ------------------------------------------------------------ layout

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

  function pixelRatio() {
    const cap = quality > 1 ? 2 : quality > 0 ? 1.5 : 1
    const budget = quality > 1 ? 3200000 : quality > 0 ? 2000000 : 1100000
    return Math.max(0.6, Math.min(window.devicePixelRatio || 1, cap, Math.sqrt(budget / Math.max(1, width * height))))
  }

  function resize() {
    const previous = view
    view = viewport.layout(window.innerWidth, window.innerHeight, preferredRotation)
    width = view.width; height = view.height
    gameNode.style.width = width + 'px'; gameNode.style.height = height + 'px'
    gameNode.style.transform = viewport.transform(view)
    gameNode.dataset.rotation = String(view.rotation)
    for (const id of ['flipButton', 'welcomeFlip']) $(id).hidden = view.rotation === 0
    applySafeArea()
    if (width < 2 || height < 2 || !renderer) return
    renderer.setPixelRatio(pixelRatio())
    renderer.setSize(width, height, false)
    camera.aspect = width / height
    // Narrow screens widen the vertical field a little so the hills ahead stay in view.
    camera.fov = camera.aspect < 1.5 ? 40 : 34
    camera.updateProjectionMatrix()
    if (previous && (previous.physicalWidth !== view.physicalWidth || previous.physicalHeight !== view.physicalHeight || previous.rotation !== view.rotation)) {
      resetInput()
      if (previous.rotation !== view.rotation) pause('画面方向已调整，准备好后继续。')
    }
  }
  window.addEventListener('resize', resize)
  window.visualViewport?.addEventListener('resize', resize)

  // ------------------------------------------------------------ input

  const pointers = new Set()
  let keyHold = false

  function holding() { return keyHold || pointers.size > 0 }

  function resetInput() {
    pointers.clear(); keyHold = false
    $('holdRing').classList.remove('active')
  }

  function useTouch(value) {
    if (touchUI === value) return
    touchUI = value
    gameNode.classList.toggle('touch', value)
  }

  function interactive(target) { return target instanceof Element && target.closest('button, .overlay, .hud-actions') }

  gameNode.addEventListener('pointerdown', (event) => {
    if (phase !== 'playing' || interactive(event.target)) return
    event.preventDefault()
    useTouch(event.pointerType !== 'mouse')
    pointers.add(event.pointerId)
    try { gameNode.setPointerCapture(event.pointerId) } catch { /* Capture is optional. */ }
    const p = viewport.point(view, event.clientX, event.clientY)
    const ring = $('holdRing')
    ring.style.left = p.x + 'px'; ring.style.top = p.y + 'px'
    ring.classList.add('active')
  })
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) gameNode.addEventListener(type, (event) => {
    if (!pointers.delete(event.pointerId)) return
    if (!pointers.size) $('holdRing').classList.remove('active')
  })

  const HOLD_KEYS = new Set([' ', 'arrowdown', 's', 'enter'])

  function activeDialog() {
    return phase === 'paused' ? $('pauseOverlay') : phase === 'result' ? $('resultOverlay') : phase === 'welcome' ? $('welcome') : null
  }

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return
    if (event.key === 'Tab') {
      const dialog = activeDialog()
      if (dialog) {
        const buttons = [...dialog.querySelectorAll('button')].filter((button) => !button.disabled && !button.hidden && button.getBoundingClientRect().width > 0)
        const index = buttons.indexOf(document.activeElement)
        const next = index < 0 ? 0 : (index + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length
        event.preventDefault(); buttons[next]?.focus()
      }
      return
    }
    const key = event.key.toLowerCase()
    if (key === 'escape' || key === 'p') { if (phase === 'playing') { event.preventDefault(); pause() } return }
    if (phase !== 'playing' || !HOLD_KEYS.has(key)) return
    if ((key === ' ' || key === 'enter') && document.activeElement instanceof HTMLButtonElement) return
    event.preventDefault()
    useTouch(false)
    keyHold = true
  })
  document.addEventListener('keyup', (event) => { if (HOLD_KEYS.has(event.key.toLowerCase())) keyHold = false })
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) pause('离开页面时太阳替你停住了，回来可以接着飞。')
    else audio?.resume()
  })
  window.addEventListener('blur', () => { resetInput(); pause('操作暂时中断了，准备好后再继续。') })
  window.addEventListener('pagehide', () => { syncRound(); audio?.close() })
  document.addEventListener('contextmenu', (event) => event.preventDefault())

  // ------------------------------------------------------------ chrome

  function toast(message, duration = 2400) {
    clearTimeout(toastTimer)
    $('toast').textContent = message
    $('toast').hidden = false
    toastTimer = setTimeout(() => { $('toast').hidden = true }, duration)
  }

  const PROMPTS = {
    dive: ['按住屏幕，纸鸟变重，顺着下坡俯冲', '按住空格（或鼠标），纸鸟变重，顺着下坡俯冲'],
    release: ['到了谷底就松开，借上坡飞上天', '到了谷底就松开，借上坡飞上天'],
    land: ['落地时顺着下坡贴下去，就是完美落地', '落地时顺着下坡贴下去，就是完美落地'],
    fever: ['连续三次完美，乘着顺风积分翻倍', '连续三次完美，乘着顺风积分翻倍'],
    thud: ['别撞上坡：在空中按住，提前落到下坡', '别撞上坡：在空中按住，提前落到下坡'],
    sea: ['冲得够快，掠过海面还能打水漂', '冲得够快，掠过海面还能打水漂'],
    dusk: ['太阳快落了，赶在天黑前飞到下一座岛', '太阳快落了，赶在天黑前飞到下一座岛'],
  }

  function showPrompt(key) {
    if (!PROMPTS[key]) return
    const node = $('prompt')
    node.textContent = PROMPTS[key][touchUI ? 0 : 1]
    node.hidden = true; void node.offsetWidth; node.hidden = false
    clearTimeout(promptTimer)
    promptTimer = setTimeout(() => { node.hidden = true }, 4200)
  }
  function promptOnce(key) { if (!shown.has(key)) { shown.add(key); showPrompt(key) } }

  function callout(kind, title, points) {
    const node = $('callout')
    node.className = 'callout ' + kind
    $('calloutTitle').textContent = title
    $('calloutPoints').textContent = points ? '+' + points : ''
    node.hidden = true; void node.offsetWidth; node.hidden = false
    node.classList.add('show')
    clearTimeout(calloutTimer)
    calloutTimer = setTimeout(() => { node.hidden = true }, 1200)
  }

  function showIsland(index, bonus) {
    $('islandSub').textContent = index === 0 ? '第一座岛' : '第 ' + (index + 1) + ' 座岛'
    $('islandName').textContent = World.islandName(index)
    $('islandBonus').textContent = bonus ? '+' + Math.round(bonus) + ' 秒余晖' : '日落之前，飞得更远'
    const node = $('islandTitle')
    node.hidden = true; void node.offsetWidth; node.hidden = false
    clearTimeout(islandTimer)
    islandTimer = setTimeout(() => { node.hidden = true }, 3300)
  }

  function updateSoundButtons() {
    const on = Boolean(audio?.enabled)
    $('soundButton').setAttribute('aria-pressed', String(on))
    $('soundButton').setAttribute('aria-label', on ? '关闭声音' : '开启声音')
    $('soundWaves').setAttribute('d', on ? 'M15 8q4 4 0 8m3-11q7 7 0 14' : 'm16 9 5 6m0-6-5 6')
    $('welcomeSound').setAttribute('aria-pressed', String(on))
    $('welcomeSound').textContent = on ? '声音 · 开' : '声音 · 关'
  }
  function toggleSound() {
    if (!audio) return
    audio.setEnabled(!audio.enabled)
    audio.resume()
    updateSoundButtons()
    audio.play('ui')
  }
  $('soundButton').addEventListener('click', toggleSound)
  $('welcomeSound').addEventListener('click', toggleSound)

  async function requestImmersive(feedback = true) {
    const root = document.documentElement
    const request = root.requestFullscreen || root.webkitRequestFullscreen
    if (!request || document.fullscreenEnabled === false) { if (feedback) toast('当前浏览器暂不支持全屏，画面已自动适配横屏。', 3200); return }
    try {
      if (!document.fullscreenElement && !document.webkitFullscreenElement) await request.call(root)
      if (screen.orientation?.lock) { try { await screen.orientation.lock('landscape') } catch { /* Manual rotation still works. */ } }
    } catch { if (feedback) toast('全屏暂未开启，横持手机仍可继续。', 3000) }
  }
  async function leaveFullscreen() {
    try { screen.orientation?.unlock?.() } catch { /* Not every browser exposes orientation controls. */ }
    try {
      const exit = document.exitFullscreen || document.webkitExitFullscreen
      if (exit && (document.fullscreenElement || document.webkitFullscreenElement)) await exit.call(document)
    } catch { /* Navigation also leaves fullscreen. */ }
  }
  $('fullscreenButton').hidden = !(Boolean(document.documentElement.requestFullscreen || document.documentElement.webkitRequestFullscreen) && document.fullscreenEnabled !== false)
  function fullscreenChange() {
    const active = Boolean(document.fullscreenElement || document.webkitFullscreenElement)
    $('fullscreenButton').setAttribute('aria-label', active ? '退出全屏' : '进入全屏')
    if (wasFullscreen && !active && !leaving) pause('已退出全屏，太阳替你停住了。')
    wasFullscreen = active
    resize()
  }
  document.addEventListener('fullscreenchange', fullscreenChange)
  document.addEventListener('webkitfullscreenchange', fullscreenChange)
  $('fullscreenButton').addEventListener('click', () => {
    if (document.fullscreenElement || document.webkitFullscreenElement) void leaveFullscreen()
    else void requestImmersive()
  })

  function setOverlays(name) {
    $('welcome').hidden = name !== 'welcome'
    $('pauseOverlay').hidden = name !== 'paused'
    $('resultOverlay').hidden = name !== 'result'
    $('hud').hidden = name === 'welcome'
    if (name !== 'playing') { $('prompt').hidden = true; $('toast').hidden = true; $('callout').hidden = true }
  }

  function startRound() {
    if (!renderer) return
    audio?.resume()
    if (touchUI && !document.fullscreenElement) void requestImmersive(false)
    state = rules.createGame()
    world.reset(state)
    bird.reset(state)
    resultDelay = 0
    globalThis.XiabanResultShare?.reset()
    phase = 'playing'; roundStarted = true; roundPosted = false; resultPosted = false
    resetInput()
    lastProgress = 0
    snapCamera()
    setOverlays('playing')
    $('islandTitle').hidden = true
    for (const id of ['startButton', 'restartButton']) $(id).blur()
    updateHUD(true)
    showIsland(0, 0)
    setTimeout(() => { if (phase === 'playing') promptOnce('dive') }, 900)
    syncRound()
    audio?.play('ui')
  }

  function pause(message = '太阳也替你停住了，随时可以接着飞。') {
    if (phase !== 'playing') return
    phase = 'paused'
    resetInput()
    $('pauseDescription').textContent = message
    setOverlays('paused')
    $('hud').hidden = false
    syncRound()
    $('resumeButton').focus({ preventScroll: true })
  }

  function resume() {
    if (document.hidden || phase !== 'paused') return
    audio?.resume(); resetInput()
    phase = 'playing'; lastFrame = 0
    setOverlays('playing')
    $('resumeButton').blur()
  }

  function formatTime(seconds) { return Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2, '0') }

  function showResult() {
    phase = 'result'
    resetInput()
    const r = rules.result(state)
    const isBest = r.score > best
    best = Math.max(best, r.score)
    $('resultBadge').textContent = '夜色降临 · ' + World.islandName(r.islands - 1)
    $('resultTitle').textContent = r.islands >= 6 ? '追着落日飞过群岛' : r.maxChain >= 5 ? '乘风的纸鸟' : r.islands >= 3 ? '一路向着晚霞' : '纸鸟归巢'
    $('resultDescription').textContent = r.islands >= 2 ? '天色暗下来，纸鸟落在第 ' + r.islands + ' 座岛上歇脚。' : '天色暗下来了。下一次试着在谷底松开，飞得更高。'
    $('resultScore').textContent = r.score
    $('resultPerfects').textContent = r.perfects
    $('resultChain').textContent = r.maxChain
    $('resultIslands').textContent = r.islands
    $('resultDistance').textContent = r.distance
    $('resultBonus').textContent = '用时 ' + formatTime(r.seconds) + ' · 腾空 ' + r.flights + ' 次 · 最高 ' + r.topHeight + ' 米 · 极速 ' + r.topSpeed + ' km/h'
    $('bestLabel').textContent = isBest && r.score > 0 ? '本次游玩新高' : '本次游玩最高 ' + best
    setOverlays('result')
    $('hud').hidden = false
    globalThis.XiabanResultShare?.finish({ gameId: GAME_ID, score: r.score, lines: r.perfects, maxTile: r.maxChain, level: r.islands, steps: r.flights })
    syncRound()
    audio?.play('over')
    $('restartButton').focus({ preventScroll: true })
  }

  async function exitGame() {
    leaving = true
    resetInput()
    if (roundStarted) syncRound()
    await leaveFullscreen()
    if (hostReady) postToHost('game.exit')
    else window.location.assign('/games')
  }

  $('startButton').addEventListener('click', startRound)
  $('restartButton').addEventListener('click', startRound)
  $('resumeButton').addEventListener('click', resume)
  $('pauseButton').addEventListener('click', () => pause())
  $('shareResultButton').addEventListener('click', () => { void leaveFullscreen() })
  for (const id of ['welcomeExit', 'pauseExit', 'resultExit']) $(id).addEventListener('click', () => { void exitGame() })
  for (const id of ['flipButton', 'welcomeFlip']) $(id).addEventListener('click', () => {
    if (!view.rotation) return
    pause('画面已换到另一边，准备好后继续。')
    preferredRotation *= -1
    resize()
  })

  // ------------------------------------------------------------ events

  function dustColor() {
    const biome = World.biomeFor(state.island)
    return biome.grass[0]
  }

  function handleEvents(events) {
    for (const e of events) {
      switch (e.type) {
        case 'launch':
          bird.burst('launch', e.x, e.y, { dust: dustColor() })
          audio?.play('launch', { speed: e.speed })
          if (state.flights === 0) promptOnce('release')
          break
        case 'flight':
          if (e.air > 1.2 && !shown.has('land')) promptOnce('land')
          break
        case 'perfect':
          bird.burst('perfect', e.x, e.y, { chain: e.chain })
          callout(state.fever ? 'perfect fever' : 'perfect', e.chain > 1 ? '完美 ×' + e.chain : '完美落地', e.points)
          audio?.play('perfect', { chain: e.chain })
          if (e.chain === 1) promptOnce('fever')
          break
        case 'land':
          bird.burst('land', e.x, e.y, { dust: dustColor() })
          audio?.play('land')
          break
        case 'thud':
          bird.burst('thud', e.x, e.y, { dust: dustColor() })
          callout('thud', '磕了一下', 0)
          audio?.play('thud')
          promptOnce('thud')
          break
        case 'splash':
          bird.burst('splash', e.x, e.y)
          callout('splash', '扑通', 0)
          audio?.play('splash')
          promptOnce('sea')
          break
        case 'skip':
          bird.burst('skip', e.x, e.y)
          callout('skip', '打水漂', 0)
          audio?.play('skip')
          break
        case 'cloud':
          bird.burst('cloud', e.x, e.y)
          callout('cloud', '触到云了', e.points)
          audio?.play('cloud')
          break
        case 'fever':
          bird.burst('fever', e.x, e.y)
          callout('fever', '顺风 ×2', 0)
          audio?.play('fever')
          break
        case 'feverEnd':
          audio?.play('feverEnd')
          break
        case 'flap':
          bird.flap()
          audio?.play('flap')
          break
        case 'island':
          showIsland(e.index, e.bonus)
          bird.burst('island', state.bird.x + 10, state.bird.y + 4)
          audio?.play('island')
          if (e.dawn) toast('赶上了！余晖又亮了起来', 2200)
          break
        case 'night':
          toast('夜幕降临，纸鸟要歇了', 2200)
          audio?.play('night')
          break
        case 'over':
          resultDelay = 1.8
          break
      }
    }
  }

  // ------------------------------------------------------------ camera

  const cam = { x: 0, y: 20, z: 60, lookX: 0, lookY: 10, dist: 60 }
  const YAW = 0.2

  function cameraGoal() {
    const b = state.bird
    const ground = world.heightAt(b.x + 12, 0) * 0.5 + world.heightAt(b.x - 4, 0) * 0.5
    const lowest = Math.max(-1, Math.min(ground, b.y))
    const above = Math.max(0, b.y - lowest)
    const half = Math.tan(camera.fov * Math.PI / 360)
    const dist = Math.max(68, Math.min(170, (above * 0.55 + 12) / half + b.speed * 0.45))
    const lead = 14 + Math.min(20, b.speed * 0.4)
    return { lookX: b.x + lead, lookY: lowest + above * 0.5 + 3.5, dist }
  }

  function placeCamera() {
    camera.position.set(cam.lookX - Math.sin(YAW) * cam.dist, cam.lookY + cam.dist * 0.24, Math.cos(YAW) * cam.dist)
    camera.lookAt(cam.lookX, cam.lookY, 0)
  }

  function snapCamera() {
    const goal = cameraGoal()
    Object.assign(cam, goal)
    placeCamera()
  }

  function updateCamera(dt) {
    const goal = cameraGoal()
    const k = 1 - Math.exp(-dt * 4.5)
    cam.lookX += (goal.lookX - cam.lookX) * (1 - Math.exp(-dt * 9))
    cam.lookY += (goal.lookY - cam.lookY) * k
    // Pull back quickly when climbing, settle in slowly so the zoom never feels jumpy.
    const zoom = goal.dist > cam.dist ? 3 : 1.2
    cam.dist += (goal.dist - cam.dist) * (1 - Math.exp(-dt * zoom))
    placeCamera()
  }

  // ------------------------------------------------------------ HUD

  const hud = { score: -1, distance: -1, day: -1, chain: -1, fever: null, low: null, dark: null }
  function updateHUD(force = false) {
    if (force || state.score !== hud.score) { $('score').textContent = state.score; hud.score = state.score }
    if (force || state.distance !== hud.distance) { $('distance').textContent = state.distance; hud.distance = state.distance }
    const day = Math.round(Math.min(DAY_SCALE, state.daylight) * 10)
    if (force || day !== hud.day) {
      const pct = day / 10 / DAY_SCALE * 100
      $('daylightFill').style.width = pct + '%'
      $('daylightSun').style.left = pct + '%'
      $('daylight').setAttribute('aria-valuenow', String(Math.round(day / 10)))
      hud.day = day
    }
    const low = !state.night && state.daylight < 10, dark = state.night
    if (force || low !== hud.low || dark !== hud.dark) {
      $('daylight').classList.toggle('low', low)
      $('daylight').classList.toggle('dark', dark)
      $('daylightLabel').textContent = dark ? '夜幕' : low ? '天快黑了' : '余晖'
      hud.low = low; hud.dark = dark
      if (low && phase === 'playing') promptOnce('dusk')
    }
    if (force || state.chain !== hud.chain || state.fever !== hud.fever) {
      const node = $('chain')
      node.hidden = state.chain < 1
      node.classList.toggle('fever', state.fever)
      $('chainCount').textContent = state.chain
      $('chainLabel').textContent = state.fever ? '顺风 ×2' : '连续完美'
      if (state.chain > hud.chain && state.chain > 0) { node.classList.remove('pop'); void node.offsetWidth; node.classList.add('pop') }
      hud.chain = state.chain; hud.fever = state.fever
    }
  }

  // ------------------------------------------------------------ attract mode

  // Behind the welcome card the bird flies itself: weight on downhill ground, glide otherwise.
  function autopilot() {
    const b = state.bird
    if (b.ground) return rules.sample(state, b.x).d < 0.02
    const ahead = rules.sample(state, b.x + b.vx * 0.3)
    return b.vy < 0 && ahead.d < -0.08 && Math.atan2(b.vy, b.vx) > Math.atan(ahead.d) + 0.05
  }

  // ------------------------------------------------------------ loop

  function adaptQuality(dt) {
    if (phase !== 'playing' || quality === 0) return
    frameAvg = frameAvg * 0.96 + dt * 1000 * 0.04
    slowFrames = frameAvg > 24 ? slowFrames + 1 : Math.max(0, slowFrames - 2)
    if (slowFrames > 150) {
      quality -= 1; slowFrames = 0; frameAvg = 16
      world.setQuality(quality)
      renderer.shadowMap.enabled = quality > 0
      resize()
    }
  }

  function frame(now, manual) {
    if (!manual) requestAnimationFrame(frame)
    const dt = Math.max(0, Math.min(0.05, (now - (lastFrame || now)) / 1000))
    lastFrame = now
    clock += dt
    if (!renderer) return
    const simulate = !leaving && (phase === 'playing' || phase === 'welcome')
    if (simulate) {
      const steps = Math.max(1, Math.ceil(dt / MAX_STEP - 1e-6))
      for (let i = 0; i < steps; i++) {
        if (phase === 'welcome') {
          state.daylight = 50 + Math.sin(clock * 0.05) * 6
          const events = rules.step(state, { hold: autopilot() }, dt / steps)
          for (const e of events) if (e.type === 'perfect' || e.type === 'splash' || e.type === 'skip' || e.type === 'launch') bird.burst(e.type, e.x, e.y, { chain: 1, dust: dustColor() })
          if (state.phase !== 'playing' || state.bird.x > 5000) { state = rules.createGame({ seed: 20261007 }); world.reset(state); bird.reset(state); snapCamera() }
        } else handleEvents(rules.step(state, { hold: holding() }, dt / steps))
      }
      if (phase === 'playing') {
        if (state.phase === 'over') { resultDelay -= dt; if (resultDelay <= 0) showResult() }
        if (state.time - lastProgress >= 1.5) { lastProgress = state.time; syncRound() }
        updateHUD()
      }
    }
    const frameDt = phase === 'paused' ? 0 : dt
    updateCamera(frameDt || 0.0001)
    world.update(camera, state.bird, frameDt, clock)
    bird.update(state, frameDt, clock, world.env, world.heightAt, height * renderer.getPixelRatio(), camera.fov * Math.PI / 180)
    // The HUD switches to light text once the sky behind it has darkened past sunset.
    const dark = world.env.light < 0.26
    if (dark !== gameNode.classList.contains('night')) gameNode.classList.toggle('night', dark)
    const water = state.bird.ground && state.bird.y <= rules.SEA + 0.01
    audio?.update({ speed: state.bird.speed, air: !state.bird.ground, hold: state.bird.hold && phase === 'playing', fever: state.fever, night: world.env.night, water, playing: phase === 'playing' })
    renderer.render(scene, camera)
    adaptQuality(dt)
  }

  if (params.has('debug')) {
    let fake = 0
    globalThis.__glider = {
      get state() { return state }, get phase() { return phase }, camera, cam, renderer, world, bird, scene, get quality() { return quality },
      start: startRound, hold(on) { keyHold = on },
      // Drives the loop by hand for headless checks, where requestAnimationFrame does not fire.
      advance(seconds, hold) {
        if (hold !== undefined) keyHold = hold
        fake = Math.max(fake, lastFrame)
        for (let t = 0; t < seconds; t += 1 / 60) { fake += 1000 / 60; frame(fake, true) }
        return { x: state.bird.x, y: state.bird.y, score: state.score, phase }
      },
    }
  }

  resize()
  updateSoundButtons()
  updateHUD(true)
  if (renderer) {
    snapCamera()
    if (reducedMotion) cam.dist = 70
  }
  requestAnimationFrame(frame)
})()
