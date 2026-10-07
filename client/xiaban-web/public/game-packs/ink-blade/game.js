/* 水墨剑客 · controller: host protocol, input, round lifecycle, event-driven effects, rendering and HUD. */
(() => {
  'use strict'

  const rules = globalThis.XiabanInkBlade
  const art = globalThis.XiabanInkArt
  const fig = globalThis.XiabanInkFigures
  const scene = globalThis.XiabanInkScene
  const FX = globalThis.XiabanInkFx
  const audio = globalThis.XiabanInkAudio
  const viewport = globalThis.XiabanInkViewport
  const $ = (id) => document.getElementById(id)
  const gameNode = $('game')
  const canvas = $('scene'), ctx = canvas.getContext('2d', { alpha: false })
  const params = new URLSearchParams(location.search)
  const nonce = params.get('hostNonce')
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches
  const LV = globalThis.XiabanInkLevels
  const PROGRESS = globalThis.XiabanInkProgress
  const WORLD = LV.volume(1)
  const GAME_ID = 'ink-blade'
  const STEP = 1 / 60
  const CINNABAR = '#b8322a'
  const LOCAL_SAVE = 'xiaban.ink-blade.save'
  const hosted = Boolean(nonce) && window.parent !== window

  // A shared result can open one 幅 as a duel ("切磋帖"), with the friend's score to beat.
  const duel = (() => {
    const code = Number(params.get('stage'))
    const level = Number.isInteger(code) ? LV.levelByCode(code) : null
    const target = Number(params.get('target'))
    return level ? { level: level.id, target: Number.isInteger(target) && target > 0 && target <= 1e9 ? target : 0 } : null
  })()

  let touchUI = matchMedia('(any-pointer: coarse)').matches
  gameNode.classList.toggle('touch', touchUI)
  let selectedVolume = (duel && duel.level.startsWith('2-')) ? 2 : 1
  let save = PROGRESS.empty(), saveApplied = false, currentLevel = duel ? duel.level : '1-1'
  let difficulty = params.get('difficulty') === 'easy' ? 'easy' : 'normal'
  let state = rules.createGame({ difficulty, level: currentLevel }), phase = 'welcome'
  let resultDelay = 0, duelRound = false
  let view = null, preferredRotation = 1, width = 0, height = 0, dpr = 1, k = 1, viewW = 0, viewH = 0, worldTop = 0
  let assets = null, assetKey = '', rebuildTimer = 0, quality = 1, frameCost = 0, frameSamples = 0
  let hostReady = false, roundStarted = false, roundPosted = false, resultPosted = false
  let lastFrame = 0, accumulator = 0, clock = 0, lastProgress = 0, leaving = false, wasFullscreen = false
  let toastTimer = 0, promptTimer = 0, dashImage = 0, phantomImage = 0
  const shown = new Set()
  const camera = { x: 400 }
  const fx = FX.create()
  const weather = scene.createWeather()
  const anims = new Map()
  const hostSafeArea = { top: 0, right: 0, bottom: 0, left: 0 }

  // ------------------------------------------------------------ host protocol

  function postToHost(type, payload = {}) {
    if (!hostReady || !nonce || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...payload }, '*')
  }

  function metrics() {
    const p = state.player
    return { score: p.score, lines: p.kills, maxTile: p.maxCombo }
  }

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
    if (Object.hasOwn(data, 'save')) applySave(data.save)
    postToHost('game.ready')
    syncRound()
  })

  // ------------------------------------------------------------ save

  function applySave(raw) {
    if (saveApplied) return
    saveApplied = true
    save = PROGRESS.normalize(raw)
    if (!params.has('difficulty')) difficulty = save.difficulty
    if (save.volume && !duel) selectedVolume = save.volume
    syncDifficulty()
    refreshWelcome()
    if (phase === 'select') renderSelect()
  }

  /** Hosted packs hand the save to the website, which keeps it in the account or this browser. */
  function persistSave() {
    save = { ...save, difficulty, volume: selectedVolume }
    if (hosted) { postToHost('game.save', { data: save }); return }
    try { localStorage.setItem(LOCAL_SAVE, JSON.stringify(save)) } catch { /* Opened directly without storage. */ }
  }

  if (!hosted) {
    let stored = null
    try { stored = JSON.parse(localStorage.getItem(LOCAL_SAVE) || 'null') } catch { /* Storage is optional. */ }
    queueMicrotask(() => applySave(stored))
  }

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

  function buildAssets(key) {
    assetKey = key
    const vol = state.level ? state.level.volume : selectedVolume
    assets = scene.build({ P: k * dpr, viewW, width: canvas.width, height: canvas.height, volumeId: vol })
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
    // Frames and iframes can report an empty viewport before layout; wait for a real size.
    if (width < 2 || height < 2) return
    // Keep the backing store near 2.4 megapixels so the full-screen paper passes stay cheap on phones.
    const budget = quality > 0 ? 2400000 : 1300000
    dpr = Math.max(0.75, Math.min(window.devicePixelRatio || 1, 2, Math.sqrt(budget / Math.max(1, width * height))))
    canvas.width = Math.round(width * dpr); canvas.height = Math.round(height * dpr)
    k = Math.min(height / 360, width / 560)
    viewW = width / k; viewH = height / k
    worldTop = rules.GROUND + 62 - viewH
    const vol = state.level ? state.level.volume : selectedVolume
    const key = vol + ':' + Math.round(k * dpr * 16) + ':' + Math.round(viewW / 60)
    if (!assets || assets.volumeId !== vol) buildAssets(key)
    else {
      assets.vignette = art.vignette(canvas.width, canvas.height)
      if (key !== assetKey) { clearTimeout(rebuildTimer); rebuildTimer = setTimeout(() => buildAssets(key), 200) }
    }
    if (previous && (previous.physicalWidth !== view.physicalWidth || previous.physicalHeight !== view.physicalHeight || previous.rotation !== view.rotation)) {
      resetInput()
      if (previous.rotation !== view.rotation) pause('画面方向已调整，准备好后继续。')
    }
  }
  window.addEventListener('resize', resize)
  window.visualViewport?.addEventListener('resize', resize)

  // ------------------------------------------------------------ input

  const input = { left: false, right: false, pad: 0, jumpKey: false, jumpTouch: false, pressed: { jump: false, slash: false, dash: false, ink: false } }
  let pad = null
  const buttonPointers = new Map()

  function press(name) { if (phase === 'playing') input.pressed[name] = true }

  function takeInput() {
    const move = input.pad || (Number(input.right) - Number(input.left))
    const out = { move, jump: input.jumpKey || input.jumpTouch, jumpPressed: input.pressed.jump, slashPressed: input.pressed.slash, dashPressed: input.pressed.dash, inkPressed: input.pressed.ink }
    input.pressed.jump = input.pressed.slash = input.pressed.dash = input.pressed.ink = false
    return out
  }

  function resetPad() {
    if (pad && $('movePad').hasPointerCapture(pad.id)) $('movePad').releasePointerCapture(pad.id)
    pad = null; input.pad = 0
    $('moveKnob').style.transform = ''
    $('moveTrack').style.left = ''; $('moveTrack').style.top = ''; $('moveTrack').style.bottom = ''
    $('movePad').classList.remove('active')
  }

  function resetInput() {
    input.left = input.right = input.jumpKey = input.jumpTouch = false
    for (const key of Object.keys(input.pressed)) input.pressed[key] = false
    resetPad()
    for (const [button, id] of buttonPointers) { if (button.hasPointerCapture(id)) button.releasePointerCapture(id); button.classList.remove('pressed') }
    buttonPointers.clear()
  }

  function useTouch(value) {
    if (touchUI === value) return
    touchUI = value
    gameNode.classList.toggle('touch', value)
  }

  $('movePad').addEventListener('pointerdown', (event) => {
    if (phase !== 'playing' || pad) return
    event.preventDefault()
    useTouch(event.pointerType !== 'mouse')
    const point = viewport.point(view, event.clientX, event.clientY)
    pad = { id: event.pointerId, x0: point.x }
    $('movePad').setPointerCapture(event.pointerId)
    $('movePad').classList.add('active')
    const track = $('moveTrack')
    track.style.left = Math.max(8, point.x - track.offsetWidth / 2) + 'px'
    track.style.top = Math.max(0, point.y - $('movePad').offsetTop - track.offsetHeight / 2) + 'px'
    track.style.bottom = 'auto'
  })
  $('movePad').addEventListener('pointermove', (event) => {
    if (!pad || pad.id !== event.pointerId) return
    event.preventDefault()
    const point = viewport.point(view, event.clientX, event.clientY)
    let dx = point.x - pad.x0
    // The anchor trails the thumb so reversing direction never needs a long slide back.
    if (Math.abs(dx) > 56) { pad.x0 = point.x - Math.sign(dx) * 56; dx = Math.sign(dx) * 56 }
    input.pad = Math.abs(dx) < 7 ? 0 : Math.sign(dx) * Math.min(1, 0.6 + (Math.abs(dx) - 7) / 45)
    $('moveKnob').style.transform = 'translateX(' + Math.max(-46, Math.min(46, dx)) + 'px)'
  })
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) $('movePad').addEventListener(type, (event) => { if (pad?.id === event.pointerId) resetPad() })

  for (const [id, name] of [['slashButton', 'slash'], ['jumpButton', 'jump'], ['dashButton', 'dash'], ['inkButton', 'ink']]) {
    const button = $(id)
    button.addEventListener('pointerdown', (event) => {
      if (phase !== 'playing' || buttonPointers.has(button)) return
      event.preventDefault()
      useTouch(event.pointerType !== 'mouse')
      buttonPointers.set(button, event.pointerId)
      button.setPointerCapture(event.pointerId)
      button.classList.add('pressed')
      press(name)
      if (name === 'jump') input.jumpTouch = true
    })
    for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) button.addEventListener(type, (event) => {
      if (buttonPointers.get(button) !== event.pointerId) return
      buttonPointers.delete(button)
      button.classList.remove('pressed')
      if (name === 'jump') input.jumpTouch = false
    })
  }

  const KEYS = {
    a: 'left', arrowleft: 'left', d: 'right', arrowright: 'right',
    w: 'jump', arrowup: 'jump', k: 'jump', ' ': 'jump', j: 'slash', x: 'slash', l: 'dash', shift: 'dash', c: 'dash', i: 'ink', q: 'ink', u: 'ink',
  }

  function activeDialog() {
    return { paused: $('pauseOverlay'), result: $('resultOverlay'), welcome: $('welcome'), select: $('selectOverlay') }[phase] || null
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
    if (key === 'escape' && phase === 'select') { event.preventDefault(); backToWelcome(); return }
    if (key === 'escape' || key === 'p') { if (phase === 'playing') { event.preventDefault(); pause() } return }
    if (phase !== 'playing') return
    const action = KEYS[key]
    if (!action) return
    if (key === ' ' && document.activeElement instanceof HTMLButtonElement) return
    event.preventDefault()
    useTouch(false)
    if (action === 'left') input.left = true
    else if (action === 'right') input.right = true
    else {
      if (!event.repeat) press(action)
      if (action === 'jump') input.jumpKey = true
    }
  })
  document.addEventListener('keyup', (event) => {
    const action = KEYS[event.key.toLowerCase()]
    if (action === 'left') input.left = false
    if (action === 'right') input.right = false
    if (action === 'jump') input.jumpKey = false
  })
  canvas.addEventListener('pointerdown', (event) => {
    if (phase === 'playing' && event.pointerType === 'mouse' && event.button === 0) { useTouch(false); press('slash') }
  })
  document.addEventListener('visibilitychange', () => { if (document.hidden) pause('离开页面时画卷已停住，回来可以接着走。') })
  window.addEventListener('blur', () => { resetInput(); pause('操作暂时中断了，准备好后再继续。') })
  window.addEventListener('pagehide', () => { syncRound(); audio.close() })
  document.addEventListener('contextmenu', (event) => event.preventDefault())

  // ------------------------------------------------------------ chrome

  function toast(message, duration = 2400) {
    clearTimeout(toastTimer)
    $('toast').textContent = message
    $('toast').hidden = false
    toastTimer = setTimeout(() => { $('toast').hidden = true }, duration)
  }

  const PROMPTS = {
    move: ['按住屏幕左侧左右拖动，即可行走', 'A / D 或 ← → 行走'],
    jump: ['按「跳」跃起，空中还能再跳一次', 'K 或 空格 跳跃，空中还能再跳一次'],
    slash: ['按「斩」出剑，连按打出三段连斩', 'J 出剑，连按打出三段连斩；鼠标左键亦可'],
    dash: ['按「闪」疾步，冲刺瞬间刀剑不侵', 'L 或 Shift 闪身，冲刺瞬间刀剑不侵'],
    parry: ['敌刃朱光一闪时出剑，即可弹反', '敌刃朱光一闪时按 J，即可弹反'],
    deflect: ['箭到身前时出剑，可将箭斩回', '箭到身前时按 J，可将箭斩回'],
    shield: ['盾傀正面难破：弹反破势，或闪身绕到背后', '盾傀正面难破：弹反破势，或闪身绕到背后'],
    ink: ['墨意已满，按「一笔」挥出满纸一击', '墨意已满，按 I 挥出满纸一击'],
    execute: ['敌人失衡，此刻出剑即可处决', '敌人失衡，此刻按 J 即可处决'],
    broken: ['无常架势已破，速速出剑处决', '无常架势已破，速按 J 处决'],
    danger: ['「危」字为不可弹反之招，跳起或闪身躲开', '「危」字为不可弹反之招，跳起或闪身躲开'],
  }

  function showPrompt(key) {
    if (!PROMPTS[key]) return
    const node = $('prompt')
    node.textContent = PROMPTS[key][touchUI ? 0 : 1]
    node.hidden = true; void node.offsetWidth; node.hidden = false
    clearTimeout(promptTimer)
    promptTimer = setTimeout(() => { node.hidden = true }, key === 'move' ? 5200 : 4300)
  }

  function promptOnce(key) { if (!shown.has(key)) { shown.add(key); showPrompt(key) } }

  function showArea(sub, name) {
    const node = $('areaTitle')
    $('areaSub').textContent = sub
    $('areaName').textContent = name
    $('areaLabel').textContent = name
    node.hidden = true; void node.offsetWidth; node.hidden = false
  }

  function updateSoundButtons() {
    const on = audio.enabled
    $('soundButton').setAttribute('aria-pressed', String(on))
    $('soundButton').setAttribute('aria-label', on ? '关闭声音' : '开启声音')
    $('soundWaves').setAttribute('d', on ? 'M15 8q4 4 0 8m3-11q7 7 0 14' : 'm16 9 5 6m0-6-5 6')
    $('welcomeSound').setAttribute('aria-pressed', String(on))
    $('welcomeSound').textContent = on ? '声音 · 开' : '声音 · 关'
  }
  function toggleSound() { audio.setEnabled(!audio.enabled); updateSoundButtons(); audio.play('ui') }
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
    if (wasFullscreen && !active && !leaving) pause('已退出全屏，画卷替你停住了。')
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
    $('selectOverlay').hidden = name !== 'select'
    $('pauseOverlay').hidden = name !== 'paused'
    $('resultOverlay').hidden = name !== 'result'
    const playing = name === 'playing'
    $('hud').hidden = name === 'welcome' || name === 'select'
    $('controls').hidden = !playing
    if (!playing) { $('prompt').hidden = true; $('toast').hidden = true }
  }

  function startRound(levelId = currentLevel, asDuel = false) {
    audio.resume()
    if (touchUI && !document.fullscreenElement) void requestImmersive(false)
    currentLevel = LV.level(levelId) ? levelId : (selectedVolume === 2 ? '2-1' : '1-1')
    const lvl = LV.level(currentLevel)
    if (lvl) selectedVolume = lvl.volume
    duelRound = asDuel && Boolean(duel) && duel.level === currentLevel
    state = rules.createGame({ difficulty, level: currentLevel, seenPrompts: save.prompts })
    const vol = state.level ? state.level.volume : selectedVolume
    const key = vol + ':' + Math.round(k * dpr * 16) + ':' + Math.round(viewW / 60)
    if (!assets || assets.volumeId !== vol) buildAssets(key)
    resultDelay = 0
    globalThis.XiabanResultShare?.reset()
    phase = 'playing'; roundStarted = true; roundPosted = false; resultPosted = false
    resetInput(); FX.clear(fx); anims.clear()
    accumulator = 0; lastProgress = 0
    camera.x = cameraTarget()
    setOverlays('playing')
    $('bossBar').hidden = true
    $('areaTitle').hidden = true
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur()
    if (duelRound && duel.target) toast('切磋 · 超过 ' + duel.target + ' 分即破帖', 2400)
    updateHUD(true)
    syncRound()
    audio.play('ui')
  }

  // ------------------------------------------------------------ welcome & scroll select

  function levelLabel(level) {
    const world = LV.volume(level.volume) || WORLD
    return level.scroll ? world.name + ' · 长卷' : world.name + ' · ' + level.sub
  }
  function hurtText(level) { return level.hurt === 0 ? '无伤' : '受伤不超过 ' + level.hurt + ' 次' }

  function refreshWelcome() {
    const world = LV.volume(selectedVolume) || WORLD
    $('welcomeProgress').textContent = world.name + ' · 已得 ' + PROGRESS.sealCount(save, world.id) + ' / ' + PROGRESS.maxSeals(world.id) + ' 印'
    const level = duel && LV.level(duel.level)
    $('duelBanner').hidden = !level
    if (level) $('duelText').textContent = '朋友邀你切磋「' + level.name + '」' + (duel.target ? '：超过 ' + duel.target + ' 分' : '')
    $('startLabel').textContent = level ? '应战' : '入卷'
  }

  function openSelect() {
    phase = 'select'
    resetInput()
    setOverlays('select')
    renderSelect()
    const target = $('levelStrip').querySelector('[data-current="true"]') || $('levelStrip').querySelector('button:not(:disabled)')
    target?.scrollIntoView({ inline: 'center', block: 'nearest' })
    target?.focus({ preventScroll: true })
  }

  function backToWelcome() {
    phase = 'welcome'
    state = rules.createGame({ difficulty, level: currentLevel })
    camera.x = cameraTarget()
    setOverlays('welcome')
    refreshWelcome()
    $('startButton').focus({ preventScroll: true })
  }

  function selectVolume(vol) {
    selectedVolume = vol
    save = { ...save, volume: vol }
    persistSave()
    const firstLevel = vol === 2 ? '2-1' : '1-1'
    if (LV.level(currentLevel)?.volume !== vol) currentLevel = firstLevel
    state = rules.createGame({ difficulty, level: currentLevel })
    const key = vol + ':' + Math.round(k * dpr * 16) + ':' + Math.round(viewW / 60)
    if (!assets || assets.volumeId !== vol) buildAssets(key)
    camera.x = cameraTarget()
    renderSelect()
    audio.play('ui')
  }

  const el = (tag, className, text) => {
    const node = document.createElement(tag)
    if (className) node.className = className
    if (text !== undefined) node.textContent = text
    return node
  }

  function sealRow(mask, fresh = 0) {
    const row = el('span', 'seal-row')
    for (const seal of LV.SEALS) {
      const mark = el('i', 'mini-seal' + (mask & seal.bit ? ' on' : '') + (fresh & seal.bit ? ' fresh' : ''), seal.char)
      mark.setAttribute('aria-hidden', 'true')
      row.append(mark)
    }
    return row
  }

  function renderSelect() {
    const world = LV.volume(selectedVolume) || WORLD
    const total = PROGRESS.sealCount(save, world.id), max = PROGRESS.maxSeals(world.id)
    $('selectProgress').textContent = total + ' / ' + max + ' 印'
    for (const tab of [$('volTab1'), $('volTab2')]) {
      if (!tab) continue
      const vol = Number(tab.dataset.vol)
      const active = vol === selectedVolume
      tab.classList.toggle('active', active)
      tab.setAttribute('aria-selected', String(active))
    }
    const cards = LV.volumeLevels(world.id).map((level) => {
      const open = PROGRESS.unlocked(save, level.id)
      const entry = save.levels[level.id]
      const card = el('button', 'level-card' + (open ? '' : ' locked') + (entry ? ' cleared' : '') + (level.scroll ? ' scroll' : ''))
      card.type = 'button'
      card.dataset.level = level.id
      card.disabled = !open
      card.setAttribute('role', 'listitem')
      if (level.id === currentLevel) card.dataset.current = 'true'
      const thumb = el('canvas', 'level-thumb')
      thumb.width = 300; thumb.height = 170
      thumb.setAttribute('aria-hidden', 'true')
      card.append(thumb, el('small', 'level-sub', level.sub), el('strong', 'level-name', level.name))
      if (open) {
        card.append(sealRow(entry ? entry.seals : 0))
        card.append(el('span', 'level-best', entry ? LV.GRADES[entry.grade] + ' · ' + entry.best + ' 分' : '未曾入画'))
        card.append(el('span', 'level-goal', '全 · ' + hurtText(level) + '　技 · ' + level.challenge.text))
        const seals = entry ? LV.SEALS.filter((seal) => entry.seals & seal.bit).length : 0
        card.setAttribute('aria-label', level.sub + level.name + '，已得 ' + seals + ' 枚印' + (entry ? '，最佳' + LV.GRADES[entry.grade] + ' ' + entry.best + ' 分' : '，尚未通过'))
        card.addEventListener('click', () => startRound(level.id))
        queueMicrotask(() => drawThumb(thumb, level, Boolean(entry)))
      } else {
        card.append(el('span', 'level-lock', level.scroll ? (level.volume === 2 ? '通过「独钓」后展开' : '通过「无常」后展开') : '通过上一幅后展开'))
        card.setAttribute('aria-label', level.sub + level.name + '，尚未展开')
      }
      return card
    })
    $('levelStrip').replaceChildren(...cards)
  }

  // A finished 幅 is painted in full; an open one is a pale draft; a locked one stays blank paper (CSS only).
  function drawThumb(node, level, cleared) {
    const world = LV.volume(level.volume) || WORLD
    const g = node.getContext('2d')
    const P = node.height / 360
    const viewWidth = node.width / P
    const left = Math.max(0, Math.min(world.length - viewWidth, (level.boss ? level.start + 220 : level.start + 260) - viewWidth / 2))
    const cam = { left, top: rules.GROUND + 62 - 360, viewW: viewWidth, viewH: 360, P, width: node.width, height: node.height, sx: 0, sy: 0 }
    const env = scene.environment(left + viewWidth / 2, level.volume)
    const thumbAssets = (assets && assets.volumeId === level.volume) ? assets : scene.build({ P, viewW: viewWidth, width: node.width, height: node.height, volumeId: level.volume })
    scene.drawBackground(g, thumbAssets, cam, env, 4)
    scene.drawGround(g, thumbAssets, cam, env, 4)
    scene.drawPost(g, thumbAssets, cam, env, 0)
    node.classList.toggle('draft', !cleared)
  }

  function pause(message = '画卷停在此处，随时可以接着走。') {
    if (phase !== 'playing') return
    phase = 'paused'
    resetInput()
    $('pauseDescription').textContent = message
    setOverlays('paused')
    $('hud').hidden = false
    accumulator = 0
    syncRound()
    $('resumeButton').focus({ preventScroll: true })
  }

  function resume() {
    if (document.hidden || phase !== 'paused') return
    audio.resume(); resetInput()
    phase = 'playing'; accumulator = 0; lastFrame = 0
    setOverlays('playing')
    $('resumeButton').blur()
  }

  function formatTime(seconds) { return Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2, '0') }

  function showResult() {
    phase = 'result'
    resetInput()
    const result = rules.result(state)
    const level = state.level
    const p = state.player
    const outcome = PROGRESS.record(save, result)
    save = outcome.save
    persistSave()
    const area = (state.world ? state.world.areas : WORLD.areas)[Math.max(0, state.area)]
    const nextId = result.cleared ? PROGRESS.next(level.id) : null
    const isV2 = level.volume === 2
    $('resultSeal').textContent = result.cleared ? '品' : '败'
    $('resultSeal').classList.toggle('lose', !result.cleared)
    $('resultBadge').textContent = (difficulty === 'easy' ? '入门 · ' : '江湖 · ') + levelLabel(level)
    $('resultTitle').textContent = result.cleared
      ? (level.boss || level.scroll ? (p.damageTaken === 0 ? '片墨不沾身' : (isV2 ? '一剑断寒江' : '一剑破无常')) : level.name + ' · ' + (outcome.firstClear ? '入画' : '再临'))
      : result.kills >= 12 ? '墨尽，意未尽' : '画卷未完'
    $('resultDescription').textContent = result.cleared
      ? (level.boss || level.scroll ? (isV2 ? '钓叟收竿踏雪而去，江面重归沉寂。' : '斗笠客化作一池淡墨，竹林重归寂静。') : outcome.firstClear && nextId ? '下一幅「' + LV.level(nextId).name + '」已经展开。' : '墨迹未干，可再求更高一品。')
      : level.boss ? (isV2 ? '钓叟仍立在雪岸。朱光一闪，便是出剑之时。' : '无常仍立在月下。朱光一闪，便是出剑之时。') : '提剑再入，' + (area ? area.name : '江岸') + '还在等你。'
    $('resultScore').textContent = result.score
    $('resultGrade').textContent = result.cleared ? LV.GRADES[result.grade] + (outcome.newGrade && !outcome.firstClear ? ' · 新品' : '') : '本局积分'
    $('resultGrade').classList.toggle('graded', result.cleared)
    const seals = LV.SEALS.map((seal) => {
      const item = el('li', 'seal-slot' + (result.seals & seal.bit ? ' on' : '') + (outcome.gained & seal.bit ? ' fresh' : ''))
      const caption = seal.bit === 1 ? '通关' : seal.bit === 2 ? hurtText(level) : level.challenge.text
      item.append(el('i', 'mini-seal', seal.char), el('span', 'seal-caption', caption))
      if (outcome.gained & seal.bit) item.append(el('b', 'seal-new', '新'))
      return item
    })
    $('resultSeals').replaceChildren(...seals)
    $('resultKills').textContent = result.kills
    $('resultCombo').textContent = result.maxCombo
    $('resultParries').textContent = result.parries
    $('resultTime').textContent = formatTime(result.seconds)
    const bonus = state.bonus
    $('resultBonus').textContent = bonus ? '用时 +' + bonus.time + ' · 余血 +' + bonus.hp + (bonus.flawless ? ' · 无伤 +' + bonus.flawless : '') : result.executions ? '处决 ' + result.executions + ' 次' : ''
    const entry = save.levels[level.id]
    const duelLine = duelRound && duel.target ? (result.score > duel.target ? '破帖！超过朋友 ' + (result.score - duel.target) + ' 分 · ' : '距破帖还差 ' + (duel.target - result.score + 1) + ' 分 · ') : ''
    $('bestLabel').textContent = duelLine + (!entry ? '' : outcome.newBest && !outcome.firstClear ? '刷新本幅最佳' : '本幅最佳 ' + LV.GRADES[entry.grade] + ' ' + entry.best + ' 分')
    $('nextButton').hidden = !nextId
    if (nextId) $('nextButton').textContent = '下一幅 · ' + LV.level(nextId).name
    setOverlays('result')
    $('hud').hidden = false
    globalThis.XiabanResultShare?.finish({
      gameId: GAME_ID, score: result.score, lines: result.kills, maxTile: result.maxCombo, level: result.cleared ? 1 : 0, steps: result.parries,
      stage: level.code, seals: result.seals, grade: result.grade,
    })
    syncRound()
    audio.play(result.cleared ? 'victory' : 'defeat')
    ;(nextId ? $('nextButton') : $('restartButton')).focus({ preventScroll: true })
  }

  async function exitGame() {
    leaving = true
    resetInput()
    if (roundStarted) syncRound()
    await leaveFullscreen()
    if (hostReady) postToHost('game.exit')
    else window.location.assign('/games')
  }

  $('startButton').addEventListener('click', () => { if (duel) startRound(duel.level, true); else openSelect() })
  $('selectBack').addEventListener('click', backToWelcome)
  $('volTab1')?.addEventListener('click', () => selectVolume(1))
  $('volTab2')?.addEventListener('click', () => selectVolume(2))
  $('restartButton').addEventListener('click', () => startRound(currentLevel, duelRound))
  $('nextButton').addEventListener('click', () => { const nextId = PROGRESS.next(currentLevel); if (nextId) startRound(nextId) })
  $('mapButton').addEventListener('click', openSelect)
  $('resumeButton').addEventListener('click', resume)
  $('pauseButton').addEventListener('click', () => pause())
  // The shared share script posts to the host; leaving landscape first keeps the portrait page tidy.
  $('shareResultButton').addEventListener('click', () => { void leaveFullscreen() })
  for (const id of ['welcomeExit', 'pauseExit', 'resultExit']) $(id).addEventListener('click', () => { void exitGame() })
  for (const id of ['flipButton', 'welcomeFlip']) $(id).addEventListener('click', () => {
    if (!view.rotation) return
    pause('画面已换到另一边，准备好后继续。')
    preferredRotation *= -1
    resize()
  })
  function syncDifficulty() {
    for (const button of document.querySelectorAll('[data-difficulty]')) {
      const active = button.dataset.difficulty === difficulty
      button.classList.toggle('active', active); button.setAttribute('aria-pressed', String(active))
    }
  }
  for (const button of document.querySelectorAll('[data-difficulty]')) button.addEventListener('click', () => {
    difficulty = button.dataset.difficulty === 'easy' ? 'easy' : 'normal'
    syncDifficulty()
    persistSave()
    audio.play('ui')
  })
  syncDifficulty()
  refreshWelcome()

  // ------------------------------------------------------------ events → ink, sound, HUD

  const byId = (id) => state.enemies.find((enemy) => enemy.id === id)
  const anim = (key, entity) => { let a = anims.get(key); if (!a) { a = fig.createAnim(entity); anims.set(key, a) } return a }
  const ENEMY_SWINGS = {
    puppet: { a0: -2.35, a1: 0.85, r: 36, width: 11, color: '#272a30' },
    shield: { a0: -2.6, a1: 0.9, r: 36, width: 14, color: '#22252a' },
    boss: [{ a0: -2.4, a1: 0.9 }, { a0: 2.75, a1: -1.3 }, { a0: -2.9, a1: 1.25 }],
  }

  function snapshot(enemy) {
    const a = anims.get(enemy.id)
    const pose = a ? { ...a.pose } : fig.basePose()
    const kind = enemy.type
    const copy = { type: kind, x: enemy.x, y: enemy.y, facing: enemy.facing, state: enemy.state, vx: 0, vy: 0, seed: enemy.seed }
    return (c, alpha) => {
      const look = { alpha, color: '#1d1f23' }
      if (kind === 'crow') fig.drawCrow(c, copy, copy.x, copy.y, look, clock)
      else if (kind === 'dummy') fig.drawDummy(c, copy, copy.x, copy.y, look, a || { wobble: 0 }, clock)
      else fig.drawHumanoid(c, kind, pose, copy.x, copy.y, copy.facing, look, clock, enemy.seed)
    }
  }

  function handleEvents(events) {
    const p = state.player
    for (const e of events) {
      switch (e.type) {
        case 'slash': {
          const stage = rules.STAGES[e.stage]
          FX.trail(fx, {
            x: p.x, y: p.y - 34, follow: () => [state.player.x + state.player.facing * 3, state.player.y - 34], followFor: stage.a1,
            facing: e.facing, a0: stage.arc[0], a1: stage.arc[1], r: e.stage === 2 ? 50 : 45, width: e.stage === 2 ? 20 : 15,
            grow: stage.a1 * 0.95, life: e.stage === 2 ? 0.4 : 0.32, edge: e.stage === 2 ? CINNABAR : null, hairs: e.stage === 2 ? 14 : 11,
          })
          audio.play(e.stage === 2 ? 'heavy' : 'slash')
          break
        }
        case 'jump':
          FX.puff(fx, e.x, e.y, e.double ? 6 : 4, { speed: 40, size: 5, alpha: 0.28 })
          if (e.double) { anim('p', p).flip = 1; FX.ring(fx, e.x, e.y - 6, 4, 24, 0.3, 'rgba(20,20,22,0.5)', 2) }
          audio.play('jump')
          break
        case 'land':
          if (e.impact > 380) { FX.puff(fx, e.x, e.y, 6, { angle: Math.PI, spread: 0.9, speed: 70, size: 4, alpha: 0.3 }); FX.puff(fx, e.x, e.y, 6, { angle: 0, spread: 0.9, speed: 70, size: 4, alpha: 0.3 }) }
          audio.play('land')
          break
        case 'dash':
          FX.puff(fx, e.x - e.facing * 8, e.y - 4, 8, { angle: e.facing > 0 ? Math.PI : 0, spread: 0.6, speed: 90, size: 5, alpha: 0.32 })
          dashImage = 0.21
          audio.play('dash')
          break
        case 'hit': {
          const target = byId(e.id)
          FX.splat(fx, assets.splats, e.x + e.dir * 6, e.y, e.heavy ? 52 : 36, { rot: e.dir > 0 ? 0 : Math.PI, life: 0.5, alpha: 0.85 })
          FX.drops(fx, e.x, e.y, e.heavy ? 18 : 10, { angle: e.dir > 0 ? -0.5 : Math.PI + 0.5, spread: 1.4, speed: e.heavy ? 380 : 280 })
          FX.sparks(fx, e.x, e.y, 4, '#1d1f23', { angle: e.dir > 0 ? 0 : Math.PI, spread: 1.6, len: 12 })
          FX.shake(fx, e.heavy ? 5 : 3)
          if (target?.type === 'dummy') anim(target.id, target).wobbleV += e.dir * 9
          audio.play('hit')
          break
        }
        case 'parry':
          FX.ring(fx, e.x, e.y, 4, 54, 0.32, 'rgba(250,246,236,0.95)', 4)
          FX.ring(fx, e.x, e.y, 8, 70, 0.42, CINNABAR, 2)
          FX.sparks(fx, e.x, e.y, 14, CINNABAR, { len: 18, speed: 340 })
          FX.sparks(fx, e.x, e.y, 6, '#fbf5e8', { len: 22, speed: 420, width: 1.8 })
          FX.splat(fx, assets.redSplats, e.x, e.y, 30, { life: 0.4, alpha: 0.7 })
          FX.seal(fx, '弹', e.x, e.y - 30, 18)
          FX.flash(fx, 0.22)
          FX.shake(fx, 4)
          anim('p', p).parry = 0.22
          audio.play('parry')
          break
        case 'deflect':
          FX.sparks(fx, e.x, e.y, 10, CINNABAR, { len: 14, speed: 300 })
          FX.ring(fx, e.x, e.y, 3, 30, 0.25, 'rgba(250,246,236,0.95)', 3)
          FX.seal(fx, '返', e.x, e.y - 26, 16)
          FX.flash(fx, 0.12)
          audio.play('parry')
          break
        case 'cut':
          FX.splat(fx, assets.splats, e.x, e.y, 46, { life: 0.45 })
          FX.drops(fx, e.x, e.y, 16, { speed: 300 })
          FX.sparks(fx, e.x, e.y, 10, CINNABAR, { len: 16 })
          FX.seal(fx, '破', e.x, e.y - 30, 16)
          audio.play('parry')
          break
        case 'block':
          FX.sparks(fx, e.x, e.y, 8, '#8d877c', { len: 12, speed: 260 })
          FX.seal(fx, '格', e.x, e.y - 26, 14, 0.6, '#3a3835')
          FX.shake(fx, 2)
          audio.play('block')
          break
        case 'stagger':
          FX.seal(fx, '乱', e.x, e.y - 14, 15, 0.7, '#3a3835')
          promptOnce('execute')
          break
        case 'execute':
          FX.invert(fx, 1)
          FX.letterbox(fx, e.boss ? 1 : 0.6)
          FX.splat(fx, assets.splats, e.x, e.y, e.boss ? 120 : 92, { life: 0.8 })
          FX.splat(fx, assets.redSplats, e.x, e.y, e.boss ? 70 : 48, { life: 0.7, alpha: 0.8 })
          FX.drops(fx, e.x, e.y, e.boss ? 46 : 34, { speed: 420 })
          FX.sparks(fx, e.x, e.y, 16, CINNABAR, { len: 26, speed: 420 })
          FX.seal(fx, '斩', e.x, e.y - 40, 30, 1.1)
          FX.shake(fx, 9)
          audio.play('execute')
          break
        case 'kill': {
          const target = state.enemies.find((enemy) => enemy.id === e.id) || { id: e.id, type: e.kind, x: e.x, y: e.y, facing: e.facing, seed: 0 }
          const boss = e.kind === 'boss'
          FX.ghost(fx, snapshot({ ...target, x: e.x, y: e.y, facing: e.facing, type: e.kind }), boss ? 2 : 0.85, boss ? 24 : 14)
          const cy = e.kind === 'crow' ? e.y : e.y - 26
          FX.splat(fx, assets.splats, e.x, cy, boss ? 150 : 66, { life: boss ? 1.4 : 0.7 })
          FX.drops(fx, e.x, cy, boss ? 70 : 26, { speed: boss ? 480 : 330 })
          FX.puff(fx, e.x, cy, boss ? 20 : 8, { speed: 60, size: 9, alpha: 0.3, life: 0.9, w: 20 })
          if (boss) { FX.invert(fx, 1); FX.letterbox(fx, 3); FX.shake(fx, 12); FX.seal(fx, '胜', e.x, e.y - 90, 44, 2.6) }
          if (e.how !== 'execute') audio.play('kill')
          anims.delete(e.id)
          break
        }
        case 'spawn':
          FX.converge(fx, e.x, e.kind === 'crow' ? e.y : e.y - 28, e.kind === 'boss' ? 60 : 26, e.kind === 'boss' ? 110 : 70, e.kind === 'boss' ? 0.9 : 0.7)
          audio.play('spawn')
          break
        case 'tell':
          audio.play('tell')
          break
        case 'danger':
          FX.seal(fx, '危', e.x, e.y - 22, 22, 1.1)
          promptOnce('danger')
          audio.play('danger')
          break
        case 'enemySwing': {
          const enemy = byId(e.id)
          const spec = e.kind === 'boss' ? { ...ENEMY_SWINGS.boss[e.step ?? 0], r: 66, width: 24, color: '#0f1012' } : ENEMY_SWINGS[e.kind]
          if (!spec || !enemy) break
          FX.trail(fx, {
            x: enemy.x, y: enemy.y - (e.kind === 'boss' ? 50 : 36), follow: () => [enemy.x, enemy.y - (e.kind === 'boss' ? 50 : 36)], followFor: 0.12,
            facing: e.facing, a0: spec.a0, a1: spec.a1, r: spec.r, width: spec.width, grow: 0.09, life: 0.3, color: spec.color,
            edge: e.kind === 'boss' ? CINNABAR : null, hairs: e.kind === 'boss' ? 14 : 9,
          })
          audio.play(e.kind === 'boss' ? 'heavy' : 'slash')
          break
        }
        case 'shoot':
          FX.puff(fx, e.x, e.y, 3, { speed: 30, size: 3, alpha: 0.25 })
          audio.play('shoot')
          break
        case 'dart':
          FX.puff(fx, e.x, e.y, 2, { speed: 20, size: 2.2, alpha: 0.2 })
          audio.play('shoot')
          break
        case 'hook':
          FX.sparks(fx, e.x, e.y, 6, CINNABAR, { len: 14, speed: 280 })
          audio.play('danger')
          break
        case 'caw': audio.play('caw'); break
        case 'hop': FX.puff(fx, e.x, e.y, 5, { speed: 50, size: 4, alpha: 0.28 }); break
        case 'hurt':
          FX.flash(fx, 0.32, 'rgba(184,50,42,0.55)')
          FX.splat(fx, assets.redSplats, e.x, e.y, 40, { life: 0.5, alpha: 0.75 })
          FX.drops(fx, e.x, e.y, 12, { color: CINNABAR, speed: 260, angle: e.dir > 0 ? -0.6 : Math.PI + 0.6 })
          FX.shake(fx, 7)
          audio.play('hurt')
          break
        case 'death':
          FX.letterbox(fx, 2)
          FX.invert(fx, 0.6)
          break
        case 'strike':
          FX.strike(fx, e.facing)
          audio.play('strike')
          break
        case 'strikeHit':
          FX.invert(fx, 1)
          FX.shake(fx, 12)
          for (const enemy of state.enemies) if (enemy.alive && Math.abs(enemy.x - p.x) < 470) FX.splat(fx, assets.splats, enemy.x, enemy.y - 26, 80, { life: 0.8 })
          break
        case 'lock':
          if (e.id !== 'gate' && e.id !== 'boss') toast('墨障合围 · 斩尽方可前行', 1800)
          audio.play('gong')
          break
        case 'clear':
          if (e.id !== 'boss') toast('墨障散去，前路已开', 1800)
          audio.play('ui')
          break
        case 'area':
          showArea(e.sub, e.name)
          if (e.name === '无常') audio.play('gong')
          break
        case 'prompt':
          promptOnce(e.key)
          save = PROGRESS.rememberPrompt(save, e.key)
          break
        case 'inkReady':
          audio.play('ink')
          break
        case 'bossIntro': {
          FX.letterbox(fx, 2.2)
          $('bossBar').hidden = false
          const isAngler = e.kind === 'angler' || (state.level.boss && state.level.volume === 2)
          $('bossTitle').textContent = isAngler ? '苏寒江' : '无常'
          $('bossSub').textContent = isAngler ? '寒江钓叟' : '斗笠客'
          audio.play('gong')
          break
        }
        case 'bossPhase': {
          FX.invert(fx, 0.7)
          FX.shake(fx, 9)
          FX.ring(fx, e.x, e.y, 10, 220, 0.8, 'rgba(15,16,18,0.7)', 10)
          FX.ring(fx, e.x, e.y, 10, 160, 0.6, CINNABAR, 3)
          const anglerPhase = e.kind === 'angler' || (state.level.boss && state.level.volume === 2)
          toast(anglerPhase ? '断竿化剑 · 风雪大作' : '无常怒 · 墨气翻涌', 2000)
          audio.play('roar')
          break
        }
        case 'broken':
          FX.seal(fx, '破', e.x, e.y - 18, 26, 1.2)
          FX.sparks(fx, e.x, e.y + 20, 16, CINNABAR, { len: 20, speed: 320 })
          promptOnce('broken')
          audio.play('parry')
          break
        case 'bossDown':
          FX.invert(fx, 1); FX.letterbox(fx, 3.5); FX.shake(fx, 10)
          FX.splat(fx, assets.redSplats, e.x, e.y, 90, { life: 1.2, alpha: 0.85 })
          audio.play('execute')
          break
        case 'leap':
          FX.puff(fx, e.x, e.y, 10, { speed: 80, size: 7, alpha: 0.3 })
          break
        case 'slam':
          FX.shake(fx, 10)
          FX.ring(fx, e.x, e.y - 4, 6, 110, 0.5, 'rgba(15,16,18,0.55)', 5)
          FX.splat(fx, assets.splats, e.x, e.y - 4, 100, { life: 0.7, squash: 0.35, rot: 0 })
          FX.drops(fx, e.x, e.y - 4, 30, { speed: 420, spread: 2.2 })
          audio.play('slam')
          break
        case 'wave':
          audio.play('wave')
          break
        case 'vanish':
        case 'appear':
          FX.puff(fx, e.x, e.y, 16, { speed: 90, size: 10, alpha: 0.4, life: 0.8, w: 30 })
          FX.converge(fx, e.x, e.y, e.type === 'appear' ? 20 : 0, 60, 0.5)
          break
        case 'phantom':
          phantomImage = 0.6
          audio.play('dash')
          break
        case 'wave2':
          toast('又有墨影聚来', 1400)
          break
        case 'over':
          resultDelay = e.reason === 'clear' ? 2.2 : 1.2
          break
      }
    }
  }

  // ------------------------------------------------------------ rendering

  function cameraTarget() {
    const half = viewW / 2
    if (phase === 'welcome' || phase === 'select') return Math.max(half, state.player.x - viewW * 0.24) + Math.sin(clock * 0.12) * 18
    const p = state.player
    let x = p.x + p.facing * 62 + p.vx * 0.12
    if (state.lock) {
      const [a, b] = state.lock
      x = b - a >= viewW ? Math.max(a + half, Math.min(b - half, x)) : (a + b) / 2
    }
    return Math.max(half, Math.min(WORLD.length - half, x))
  }

  function surfaceBelow(x, y) {
    let best = rules.GROUND
    for (const platform of WORLD.platforms) if (x >= platform.x && x <= platform.x + platform.w && platform.y >= y - 1 && platform.y < best) best = platform.y
    return best
  }

  const inBossFight = () => Boolean(state.lock) && state.plan[state.encounter]?.id === 'boss'

  function shadow(x, y, size) {
    const ground = surfaceBelow(x, y)
    const lift = Math.max(0, ground - y)
    const s = size * Math.max(0.35, 1 - lift / 160)
    ctx.fillStyle = 'rgba(20,20,22,' + (0.2 * Math.max(0.3, 1 - lift / 140)) + ')'
    ctx.beginPath(); ctx.ellipse(x, ground + 1, s, s * 0.2, 0, 0, Math.PI * 2); ctx.fill()
  }

  function glint(x, y, power, size = 1) {
    const s = (8 + power * 14) * size
    ctx.save()
    ctx.globalAlpha = 0.5 + power * 0.5
    ctx.drawImage(assets.glow, x - s, y - s, s * 2, s * 2)
    ctx.strokeStyle = '#fbf5e8'; ctx.lineWidth = 1.1 * size
    const l = (6 + power * 14) * size
    ctx.beginPath(); ctx.moveTo(x - l, y); ctx.lineTo(x + l, y); ctx.moveTo(x, y - l * 0.7); ctx.lineTo(x, y + l * 0.7); ctx.stroke()
    ctx.restore()
  }

  function teleProgress(enemy) {
    const telling = enemy.state === 'tele' || enemy.state === 'crouch' || enemy.state === 'waveTell' || enemy.state === 'phantomTell' || (enemy.state === 'combo' && enemy.combo?.phase === 'tele')
    return telling && enemy.teleTotal ? 1 - Math.max(0, enemy.t) / enemy.teleTotal : -1
  }

  function drawEnemy(enemy, dt) {
    if (enemy.delay > 0 || !enemy.alive && enemy.state !== 'dying') return
    const a = anim(enemy.id, enemy)
    const flash = enemy.flash > 0
    const progress = teleProgress(enemy)
    const look = { color: flash ? '#f4eedf' : undefined, flash, eyeGlow: progress >= 0 ? progress : enemy.state === 'attack' || enemy.state === 'dive' ? 1 : 0.15, halo: 0.22 }
    if (enemy.state === 'spawn') look.alpha = Math.max(0, Math.min(1, 1 - enemy.t / (enemy.type === 'boss' ? 0.9 : 0.75)))
    if (enemy.state === 'dying') { look.alpha = Math.max(0.15, enemy.t / 2.6); if (Math.random() < 0.6) FX.smoke(fx, enemy.x, enemy.y - 36, { alpha: 0.3 }) }
    let out
    if (enemy.type === 'crow') {
      out = fig.drawCrow(ctx, enemy, enemy.x, enemy.y, look, clock)
    } else if (enemy.type === 'dummy') {
      fig.updateDummy(a, dt)
      shadow(enemy.x, enemy.y, 14)
      out = fig.drawDummy(ctx, enemy, enemy.x, enemy.y, look, a, clock)
    } else {
      const pose = fig.updatePose(enemy.type, enemy, a, dt, clock, rules)
      const isBoss = enemy.type === 'boss' || enemy.type === 'angler'
      shadow(enemy.x, enemy.y, isBoss ? 26 : 16)
      if (isBoss && enemy.phase2 && enemy.state !== 'dying' && Math.random() < 0.5) FX.smoke(fx, enemy.x, enemy.y - 40, { w: 30, h: 60, alpha: 0.2, red: Math.random() < 0.25 })
      if (isBoss && enemy.state === 'phantom' && phantomImage > 0) {
        const copy = { ...pose }, x = enemy.x, y = enemy.y, facing = enemy.facing
        FX.ghost(fx, (c, alpha) => fig.drawHumanoid(c, enemy.type, copy, x, y, facing, { alpha: alpha * 0.4, color: '#2a2a2e' }, clock, enemy.seed), 0.3, 0)
      }
      out = fig.drawHumanoid(ctx, enemy.type, pose, enemy.x, enemy.y, enemy.facing, look, clock, enemy.seed)
      if (!isBoss && Math.random() < dt * 1.8) FX.drops(fx, enemy.x + (Math.random() - 0.5) * 16, enemy.y - 20 - Math.random() * 20, 1, { speed: 20, size: 1.1 })
    }
    const isBoss = enemy.type === 'boss' || enemy.type === 'angler'
    if (progress >= 0 && out.tip && enemy.state !== 'crouch' && enemy.state !== 'phantomTell') glint(out.tip[0], out.tip[1], Math.pow(progress, 2.2), isBoss ? 1.25 : 1)
    if (enemy.state === 'stagger' || enemy.state === 'broken') {
      const pulse = 1 + Math.sin(clock * 10) * 0.08
      const top = out.head ? out.head[1] - (isBoss ? 30 : 22) : enemy.y - 70
      drawSeal('斩', enemy.x, top, 15 * pulse * (isBoss ? 1.3 : 1))
    }
  }

  function drawSeal(text, x, y, size) {
    ctx.save()
    ctx.translate(x, y); ctx.rotate(-0.08)
    ctx.fillStyle = CINNABAR
    ctx.beginPath(); ctx.roundRect(-size / 2, -size / 2, size, size, size * 0.14); ctx.fill()
    ctx.fillStyle = '#fbefe2'
    ctx.font = '600 ' + Math.round(size * 0.66) + 'px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
    ctx.fillText(text, 0, size * 0.04)
    ctx.restore()
  }

  function drawPlayer(dt) {
    const p = state.player
    const a = anim('p', p)
    const pose = fig.updatePose('player', p, a, dt, clock, rules)
    const s = fig.skeleton(fig.KINDS.player, pose)
    const neckX = p.x + s.neck[0] * p.facing - p.facing * 2, neckY = p.y + s.neck[1] + 1
    fig.updateScarf(a, neckX, neckY, p.facing, p.vx, dt, clock, -p.vx * 0.9 - 30 * p.facing - 12)
    shadow(p.x, p.y, 15)
    if (dashImage > 0) {
      dashImage -= dt
      const copy = { ...pose }, x = p.x, y = p.y, facing = p.facing
      FX.ghost(fx, (c, alpha) => fig.drawHumanoid(c, 'player', copy, x, y, facing, { alpha: alpha * 0.32, color: '#2a2a2e' }, clock), 0.24, 0)
    }
    const blink = p.iframes > 0 && p.hurt <= 0 && !p.action && Math.sin(clock * 40) > 0.3
    const look = { halo: 0.28, alpha: blink ? 0.45 : 1, edge: state.strike ? CINNABAR : undefined }
    fig.drawHumanoid(ctx, 'player', pose, p.x, p.y, p.facing, look, clock, 0, { scarf: a.scarf })
    const isWater = scene.WATER[0] < p.x && p.x < scene.WATER[1] && (state.level ? state.level.volume === 1 : selectedVolume === 1)
    const onIce = (state.world.iceZones || []).some(([x0, x1]) => p.x >= x0 && p.x <= x1)
    if ((isWater || onIce) && p.onGround && p.y >= rules.GROUND - 1) {
      ctx.save()
      ctx.translate(0, rules.GROUND * 1.55 + 2); ctx.scale(1, -0.55)
      fig.drawHumanoid(ctx, 'player', pose, p.x, p.y, p.facing, { alpha: onIce ? 0.22 : 0.13, color: onIce ? '#1a2230' : '#2a3038', flash: true }, clock)
      ctx.restore()
      if (Math.abs(p.vx) > 60 && Math.random() < dt * 14) FX.drops(fx, p.x - p.facing * 6, rules.GROUND - 2, 2, { speed: 120, size: 1, stain: false, color: onIce ? '#e0ecf8' : '#3a4450' })
    }
  }

  function drawProjectiles() {
    for (const shot of state.projectiles) {
      if (shot.type === 'arrow') {
        const angle = Math.atan2(shot.vy, shot.vx)
        const mine = shot.owner === 'player'
        ctx.save(); ctx.translate(shot.x, shot.y); ctx.rotate(angle)
        if (mine) { ctx.globalAlpha = 0.5; ctx.drawImage(assets.glow, -26, -8, 30, 16); ctx.globalAlpha = 1 }
        ctx.strokeStyle = mine ? CINNABAR : '#1a1b1e'; ctx.lineWidth = 1.2
        ctx.beginPath(); ctx.moveTo(-22, 0); ctx.lineTo(2, 0); ctx.stroke()
        ctx.fillStyle = CINNABAR
        ctx.beginPath(); ctx.moveTo(6, 0); ctx.lineTo(0, -2); ctx.lineTo(0, 2); ctx.fill()
        ctx.fillStyle = mine ? CINNABAR : '#1a1b1e'
        ctx.beginPath(); ctx.moveTo(-22, 0); ctx.lineTo(-17, -3); ctx.lineTo(-15, 0); ctx.lineTo(-17, 3); ctx.fill()
        ctx.restore()
        if (mine && Math.random() < 0.5) FX.drops(fx, shot.x, shot.y, 1, { speed: 20, size: 1, color: CINNABAR, stain: false, gravity: 200 })
      } else if (shot.type === 'dart') {
        const angle = Math.atan2(shot.vy, shot.vx)
        const mine = shot.owner === 'player'
        ctx.save(); ctx.translate(shot.x, shot.y); ctx.rotate(angle)
        if (mine) { ctx.globalAlpha = 0.5; ctx.drawImage(assets.glow, -14, -6, 20, 12); ctx.globalAlpha = 1 }
        ctx.strokeStyle = mine ? CINNABAR : '#181a1e'; ctx.lineWidth = 1.4
        ctx.beginPath(); ctx.moveTo(-10, 0); ctx.lineTo(6, 0); ctx.stroke()
        ctx.fillStyle = CINNABAR
        ctx.beginPath(); ctx.moveTo(8, 0); ctx.lineTo(3, -2); ctx.lineTo(3, 2); ctx.fill()
        ctx.restore()
      } else if (shot.type === 'hook') {
        const dir = Math.sign(shot.vx) || 1
        ctx.save(); ctx.translate(shot.x, shot.y)
        ctx.strokeStyle = 'rgba(235,225,210,0.6)'; ctx.lineWidth = 0.9
        ctx.beginPath(); ctx.moveTo(-dir * 36, 0); ctx.lineTo(0, 0); ctx.stroke()
        ctx.fillStyle = CINNABAR
        ctx.beginPath(); ctx.arc(0, 0, 3, 0, Math.PI * 2); ctx.fill()
        ctx.restore()
      } else if (shot.type === 'iceWave') {
        const dir = Math.sign(shot.vx) || 1
        const s = assets.iceSpike || assets.wave
        ctx.save(); ctx.translate(shot.x, rules.GROUND); ctx.scale(dir, 1 + Math.sin(clock * 25 + shot.id) * 0.08)
        ctx.drawImage(s.node, -s.ox, -s.oy, s.width, s.height)
        ctx.restore()
        if (Math.random() < 0.7) FX.drops(fx, shot.x - dir * 10, rules.GROUND - 24, 2, { speed: 160, size: 1.4, angle: dir > 0 ? -2.2 : -0.9, spread: 0.8, stain: false, color: '#b0c4de' })
      } else if (shot.type === 'wave') {
        const dir = Math.sign(shot.vx) || 1
        const s = assets.wave
        ctx.save(); ctx.translate(shot.x, rules.GROUND); ctx.scale(dir, 1 + Math.sin(clock * 30 + shot.id) * 0.06)
        ctx.drawImage(s.node, -s.ox, -s.oy, s.width, s.height)
        ctx.restore()
        if (Math.random() < 0.7) FX.drops(fx, shot.x - dir * 10, rules.GROUND - 24, 2, { speed: 160, size: 1.4, angle: dir > 0 ? -2.2 : -0.9, spread: 0.8, stain: Math.random() < 0.3 })
      }
    }
  }

  // Ink curtains mark the edges of a locked encounter.
  function drawBarrier() {
    if (!state.lock) return
    ctx.save()
    ctx.lineCap = 'round'
    for (const [x, side] of [[state.lock[0], 1], [state.lock[1], -1]]) {
      for (let i = 0; i < 6; i++) {
        ctx.strokeStyle = 'rgba(20,20,22,' + (0.1 + (i % 3) * 0.08) + ')'
        ctx.lineWidth = 1.2 + (i % 3) * 1.4
        ctx.beginPath()
        for (let y = worldTop - 10; y <= rules.GROUND + 4; y += 12) {
          const wx = x + side * i * 3 + Math.sin(y * 0.05 + clock * 2.2 + i) * 3
          if (y === worldTop - 10) ctx.moveTo(wx, y); else ctx.lineTo(wx, y)
        }
        ctx.stroke()
      }
      ctx.globalAlpha = 0.5
      ctx.drawImage(assets.puff, x - 20, rules.GROUND - 40, 40, 50)
      ctx.globalAlpha = 1
    }
    ctx.restore()
  }

  function render(dt) {
    if (!assets) return
    const fxDt = phase === 'paused' ? 0 : state.slow > 0 && phase === 'playing' ? dt * Math.max(0.35, state.slowScale) : dt
    camera.x += (cameraTarget() - camera.x) * (1 - Math.exp(-dt * (phase === 'welcome' ? 1.2 : 5.5)))
    const P = k * dpr
    const amount = reducedMotion ? 0 : fx.shake
    const cam = {
      left: camera.x - viewW / 2, top: worldTop, viewW, viewH, P, width: canvas.width, height: canvas.height,
      sx: (Math.random() - 0.5) * amount * P, sy: (Math.random() - 0.5) * amount * P,
    }
    const env = scene.environment(camera.x, state.level ? state.level.volume : selectedVolume)
    const night = env.moon > 0.5
    if (night !== gameNode.classList.contains('night')) gameNode.classList.toggle('night', night)
    scene.updateWeather(weather, cam, env, dt, clock)
    FX.update(fx, fxDt)
    phantomImage = Math.max(0, phantomImage - dt)
    scene.drawBackground(ctx, assets, cam, env, clock)
    scene.drawGround(ctx, assets, cam, env, clock)
    scene.layer(ctx, cam, 1)
    ctx.lineCap = 'round'; ctx.lineJoin = 'round'
    FX.drawDecals(ctx, fx)
    drawBarrier()
    FX.drawGhosts(ctx, fx)
    const simDt = phase === 'playing' && !state.strike && state.hitstop <= 0 ? fxDt : 0
    for (const enemy of state.enemies) drawEnemy(enemy, simDt)
    drawProjectiles()
    drawPlayer(phase === 'welcome' ? dt : simDt)
    FX.drawFront(ctx, fx, assets)
    scene.drawForeground(ctx, assets, cam, env, clock)
    scene.drawWeather(ctx, assets, cam, weather, env)
    scene.drawPost(ctx, assets, cam, env, quality)
    FX.drawScreen(ctx, fx, cam)
    audio.update({ combat: phase === 'playing' && Boolean(state.lock) && state.plan[state.encounter]?.id !== 'gate', boss: phase === 'playing' && inBossFight(), rain: env.rain })
  }

  // ------------------------------------------------------------ HUD

  const hud = { hp: -1, max: -1, ink: -1, score: -1, combo: -1, boss: -1, posture: -1 }
  function goalText() {
    const goal = rules.challenge(state)
    if (goal.stat === 'time') return '技 · ' + formatTime(goal.value) + ' / ' + formatTime(goal.goal)
    return '技 · ' + state.level.challenge.text + ' ' + Math.min(goal.value, goal.goal) + '/' + goal.goal
  }

  function updateHUD(force = false) {
    const p = state.player
    const goal = goalText()
    if (force || goal !== hud.goal) {
      $('goal').textContent = goal
      $('goal').classList.toggle('met', rules.challenge(state).met && state.level.challenge.stat !== 'time')
      hud.goal = goal
    }
    if (force || p.hp !== hud.hp || p.maxHp !== hud.max) {
      const lost = hud.hp > p.hp && !force ? p.hp : -1
      const pips = []
      for (let i = 0; i < p.maxHp; i++) {
        const pip = document.createElement('i')
        if (i >= p.hp) pip.className = 'empty' + (i === lost ? ' lost' : '')
        pips.push(pip)
      }
      $('hp').replaceChildren(...pips)
      $('hp').setAttribute('aria-label', '墨血 ' + p.hp + ' / ' + p.maxHp)
      hud.hp = p.hp; hud.max = p.maxHp
    }
    const ink = Math.floor(p.ink)
    if (force || ink !== hud.ink) {
      $('inkFill').style.width = ink + '%'
      $('inkMeter').setAttribute('aria-valuenow', String(ink))
      $('inkMeter').classList.toggle('full', ink >= 100)
      $('inkButton').disabled = ink < 100
      $('inkButton').style.setProperty('--ink', String(ink))
      hud.ink = ink
    }
    if (force || p.score !== hud.score) { $('score').textContent = p.score; hud.score = p.score }
    if (force || p.combo !== hud.combo) {
      $('combo').hidden = p.combo < 3
      if (p.combo >= 3) {
        $('comboCount').textContent = p.combo
        $('combo').classList.remove('pop'); void $('combo').offsetWidth; $('combo').classList.add('pop')
      }
      hud.combo = p.combo
    }
    const boss = state.enemies.find((enemy) => enemy.type === 'boss')
    if (boss) {
      const hp = Math.round(boss.hp / boss.maxHp * 1000), posture = Math.round(boss.posture)
      if (hp !== hud.boss) { $('bossHp').style.width = hp / 10 + '%'; hud.boss = hp }
      if (posture !== hud.posture) { $('bossPosture').style.width = posture + '%'; hud.posture = posture }
    } else if (!$('bossBar').hidden && !inBossFight()) $('bossBar').hidden = true
    gameNode.classList.toggle('boss-fight', !$('bossBar').hidden)
  }

  // ------------------------------------------------------------ loop

  function frame(now) {
    const dt = Math.max(0, Math.min(0.05, (now - (lastFrame || now)) / 1000))
    lastFrame = now
    clock += dt
    if (!leaving && phase === 'playing') {
      accumulator += dt
      let ticks = 0
      while (accumulator >= STEP && ticks < 5) {
        handleEvents(rules.step(state, takeInput(), STEP))
        accumulator -= STEP; ticks += 1
      }
      if (ticks === 5) accumulator = 0
      if (state.phase === 'over') { resultDelay -= dt; if (resultDelay <= 0) showResult() }
      if (state.time - lastProgress >= 1.5) { lastProgress = state.time; syncRound() }
      updateHUD()
    }
    {
      const started = performance.now()
      render(dt)
      // Phones that cannot hold the frame budget drop the paper grain and some resolution, once.
      if (phase === 'playing' && quality > 0) {
        frameCost = frameCost * 0.95 + (performance.now() - started) * 0.05
        frameSamples += 1
        if (frameSamples > 180 && frameCost > 14) { quality = 0; resize() }
      }
    }
    if (!stepping) requestAnimationFrame(frame)
  }

  // ?debug=1 exposes a frame stepper, because hidden preview tabs do not run animation frames.
  let stepping = false
  if (params.get('debug') === '1') {
    let debugClock = 0
    globalThis.__ink = {
      get state() { return state }, get phase() { return phase }, get save() { return save }, input,
      freeze: false,
      advance(seconds, each) {
        debugClock = Math.max(debugClock, lastFrame)
        for (let t = 0; t < seconds; t += STEP) { debugClock += STEP * 1000; stepping = true; frame(debugClock); stepping = false; each?.() }
      },
    }
  }

  resize()
  updateSoundButtons()
  updateHUD(true)
  camera.x = cameraTarget()
  requestAnimationFrame(frame)
})()
