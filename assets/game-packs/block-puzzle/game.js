(() => {
  'use strict'

  const runtime = globalThis.XiaobanBuiltinRuntime

  const rules = globalThis.XiabanBlockPuzzle
  const HOST_NONCE = new URLSearchParams(window.location.search).get('hostNonce')
  const boardNode = document.getElementById('board')
  const statusNode = document.getElementById('status')
  const overlay = document.getElementById('overlay')
  const ghost = document.getElementById('dragGhost')
  const restartButton = document.getElementById('restartButton')
  const restartOverlay = document.getElementById('restartOverlay')
  const exitButton = document.getElementById('exitButton')
  const pieceButtons = [...document.querySelectorAll('[data-slot]')]
  const cells = []

  let state
  let best = 0
  let selected = null
  let cursor = { row: 0, col: 0 }
  let drag = null
  let suppressClickUntil = 0
  let hostReady = false
  const storeKey = 'xiaoban.block-puzzle.classic.v2'
  const themeStoreKey = 'xiaoban.block-puzzle.theme.v1'
  const resumeStoreKey = 'xiaoban.block-puzzle.resume.v1'
  const hostStorage = globalThis.xb?.storage
  const settingsOverlay = document.getElementById('settingsOverlay')
  const settingsButton = document.getElementById('settingsButton')
  const closeSettingsButton = document.getElementById('closeSettingsButton')
  const soundButton = document.getElementById('soundButton')
  const settingsRestart = document.getElementById('settingsRestart')
  const continueButton = document.getElementById('continueButton')
  const comboBanner = document.getElementById('comboBanner')
  const sparkLayer = document.getElementById('sparkLayer')
  let soundEnabled = true
  let audio = null
  let turns = 0
  let recovering = Boolean(hostStorage)
  let hasLocalSave = false
  let hasLocalTheme = false
  let resumeSnapshot = null
  let writeQueue = Promise.resolve()
  // Visual progression belongs to the saved run, not to a global theme setting.
  let paletteIndex = 0
  let perfectCount = 0
  let scoreAnimation = 0
  let feedbackTimer = null
  let paletteTimer = null
  let resultTimer = null
  const PALETTES = ['rose', 'amber', 'mint', 'violet', 'sky']
  const scoreNode = document.getElementById('score')
  const rewardDiamond = document.getElementById('rewardDiamond')
  let diamondTimer = null
  const pendingNotes = new Set()
  const reducedMotion = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches
  function acceptTheme(data) {
    if (!data || typeof data !== 'object') return false
    if (!Number.isSafeInteger(data.paletteIndex) || data.paletteIndex < 0 ||
        !Number.isSafeInteger(data.perfectCount) || data.perfectCount < 0) return false
    paletteIndex = data.paletteIndex % PALETTES.length
    perfectCount = data.perfectCount
    applyPalette()
    return true
  }
  function themeSnapshot() { return { paletteIndex, perfectCount } }
  function readLocalTheme() {
    try { return JSON.parse(localStorage.getItem(themeStoreKey) || 'null') } catch { return null }
  }
  function saveTheme() {
    const snapshot = themeSnapshot()
    try { localStorage.setItem(themeStoreKey, JSON.stringify(snapshot)) } catch { /* sandbox storage */ }
    if (!recovering && hostStorage?.set) {
      writeQueue = writeQueue.catch(() => {}).then(() => hostStorage.set(themeStoreKey, snapshot))
    }
  }
  function showDiamond(result, isSetup = false) {
    const lines = result.clearedRows.length + result.clearedCols.length
    if (!isSetup && !result.perfect && lines < 2 && result.combo < 3) {
      rewardDiamond.hidden = true
      return
    }
    clearTimeout(diamondTimer)
    rewardDiamond.textContent = '+' + (isSetup ? result.buildUpBonus : result.gained)
    rewardDiamond.className = isSetup ? 'reward-diamond setup' : 'reward-diamond'
    rewardDiamond.hidden = false
    void rewardDiamond.offsetWidth
    rewardDiamond.classList.add('appearing')
    diamondTimer = setTimeout(() => { rewardDiamond.hidden = true }, isSetup ? 700 : 950)
  }
  function applyPalette() {
    document.body.dataset.palette = PALETTES[paletteIndex] || 'rose'
  }
  function clearVisualTimers() {
    clearTimeout(feedbackTimer)
    clearTimeout(paletteTimer)
    clearTimeout(resultTimer)
    clearTimeout(diamondTimer)
    rewardDiamond.hidden = true
    cancelAnimationFrame(scoreAnimation)
    cancelRewardNotes()
    feedbackTimer = paletteTimer = resultTimer = diamondTimer = null
  }
  function animateScore(from, to) {
    cancelAnimationFrame(scoreAnimation)
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches || to <= from) {
      scoreNode.textContent = to
      return
    }
    const duration = Math.min(470, 200 + (to - from) * 2)
    const started = performance.now()
    function frame(now) {
      const t = Math.min(1, (now - started) / duration)
      const eased = 1 - (1 - t) ** 3
      scoreNode.textContent = Math.round(from + (to - from) * eased)
      if (t < 1) scoreAnimation = requestAnimationFrame(frame)
    }
    scoreNode.textContent = from
    scoreAnimation = requestAnimationFrame(frame)
  }


  function readLocal() {
    try { return JSON.parse(localStorage.getItem(storeKey) || 'null') } catch { return null }
  }
  function readArchived() {
    try { return JSON.parse(localStorage.getItem(resumeStoreKey) || 'null') } catch { return null }
  }
  function offerContinue(snapshot) {
    const loaded = snapshot?.version === 2 ? rules.restoreGame(snapshot.game) : null
    if (!loaded || loaded.over ||
        (!loaded.score && loaded.board.every((row) => row.every((tile) => !tile)))) return false
    resumeSnapshot = snapshot
    continueButton.hidden = false
    return true
  }
  function archiveContinue() {
    if (!resumeSnapshot) return
    const snapshot = resumeSnapshot
    try { localStorage.setItem(resumeStoreKey, JSON.stringify(snapshot)) } catch { /* opaque origin */ }
    if (!recovering && hostStorage?.set) {
      writeQueue = writeQueue.catch(() => {}).then(() => hostStorage.set(resumeStoreKey, snapshot))
    }
  }
  function clearContinueArchive() {
    resumeSnapshot = null
    continueButton.hidden = true
    try { localStorage.removeItem?.(resumeStoreKey) } catch { /* opaque origin */ }
    if (!recovering && hostStorage?.remove) {
      writeQueue = writeQueue.catch(() => {}).then(() => hostStorage.remove(resumeStoreKey))
    }
  }
  function continuePrevious() {
    const snapshot = resumeSnapshot
    const loaded = snapshot?.version === 2 ? rules.restoreGame(snapshot.game) : null
    if (!loaded || loaded.over) return
    clearVisualTimers()
    if (drag) stopDrag()
    state = loaded
    best = Math.max(best, Number.isSafeInteger(snapshot.best) ? snapshot.best : 0, loaded.score)
    clearContinueArchive()
    selected = null
    cursor = { row: 0, col: 0 }
    overlay.hidden = true
    settingsOverlay.hidden = true
    restartButton.disabled = false
    exitButton.disabled = false
    renderBoard()
    renderHand()
    statusNode.textContent = '已继续上局。当前主题和最高分仍然保留。'
    turns += 1
    persist()
    postToHost('game.start')
    postToHost('game.progress', { score: state.score, lines: state.lines })
  }
  function soundUI() {
    soundButton.textContent = soundEnabled ? '声音：开启' : '声音：关闭'
    soundButton.setAttribute('aria-pressed', String(soundEnabled))
  }
  function acceptSave(data) {
    const loaded = data?.version === 2 ? rules.restoreGame(data.game) : null
    if (!loaded) return false
    // Enter with a new, empty board. Recover only preferences and records.
    // Restoring the unfinished board is an explicit choice in Settings.
    best = Math.max(best, Number.isSafeInteger(data.best) && data.best >= 0 ? data.best : 0, loaded.score)
    soundEnabled = data.soundEnabled !== false
    if (Number.isSafeInteger(data.paletteIndex) && data.paletteIndex >= 0) {
      paletteIndex = data.paletteIndex % PALETTES.length
    }
    if (Number.isSafeInteger(data.perfectCount) && data.perfectCount >= 0) {
      perfectCount = data.perfectCount
    }
    applyPalette()
    soundUI()
    return true
  }
  function persist() {
    const data = { version: 2, game: rules.serializeGame(state), best, soundEnabled, paletteIndex, perfectCount }
    try { localStorage.setItem(storeKey, JSON.stringify(data)) } catch { /* sandbox storage */ }
    try { localStorage.setItem(themeStoreKey, JSON.stringify(themeSnapshot())) } catch { /* sandbox storage */ }
    if (!recovering && hostStorage?.set) {
      // Serialize host writes so a slow old save cannot overtake the newest round.
      writeQueue = writeQueue.catch(() => {}).then(() => hostStorage.set(storeKey, data))
    }
  }
  async function restoreFromHost() {
    try {
      const saved = await hostStorage.get(storeKey)
      // Never swap the board behind someone already playing.
      if (!hasLocalSave && turns === 0 && acceptSave(saved)) {
        renderBoard()
        renderHand()
      }
      if (!resumeSnapshot) offerContinue(saved)
      const archived = await hostStorage.get(resumeStoreKey)
      if (!resumeSnapshot) offerContinue(archived)
      // Theme is independent of the round and survives restarts.
      const theme = await hostStorage.get(themeStoreKey)
      if (!hasLocalTheme && turns === 0 && acceptTheme(theme)) {
        renderBoard()
        renderHand()
      }
    } catch { /* optional platform storage */ }
    finally {
      recovering = false
      archiveContinue()
      persist()
      saveTheme()
    }
  }
  function tone(hz, seconds = .09, level = .055) {
    if (!soundEnabled) return
    try {
      const Audio = window.AudioContext || window.webkitAudioContext
      if (!Audio) return
      audio ||= new Audio()
      if (audio.state === 'suspended') void audio.resume()
      const oscillator = audio.createOscillator(), volume = audio.createGain()
      oscillator.type = 'sine'
      oscillator.frequency.setValueAtTime(hz, audio.currentTime)
      oscillator.frequency.exponentialRampToValueAtTime(hz * .72, audio.currentTime + seconds)
      volume.gain.setValueAtTime(Math.min(.065, Math.max(.005, level)), audio.currentTime)
      volume.gain.exponentialRampToValueAtTime(.001, audio.currentTime + seconds)
      oscillator.connect(volume).connect(audio.destination)
      oscillator.start()
      oscillator.stop(audio.currentTime + seconds)
    } catch { /* silent fallback */ }
  }
  function cancelRewardNotes() {
    for (const pending of pendingNotes) clearTimeout(pending)
    pendingNotes.clear()
  }
  function scheduleRewardNote(hz, duration, delay, volume = .047) {
    // Never let the last combo's melody leak into the following move, restart
    // or a temporarily hidden mobile WebView.
    const pending = setTimeout(() => {
      pendingNotes.delete(pending)
      tone(hz, duration, volume)
    }, delay)
    pendingNotes.add(pending)
  }
  function haptic(kind) {
    try {
      if (globalThis.xb) {
        if (globalThis.xb.can?.('haptics') && globalThis.xb.haptics?.impact) {
          void Promise.resolve(globalThis.xb.haptics.impact(kind)).catch(() => {})
        }
      } else if (kind !== 'light') navigator.vibrate?.(kind === 'heavy' ? 30 : 12)
    } catch { /* optional feedback */ }
  }
  // The clear travels along complete rows or columns. Its intensity is tied
  // to line count and combo milestones; effect nodes never intercept touches.
  function showFeedback(result) {
    sparkLayer.replaceChildren()
    const lines = result.clearedRows.length + result.clearedCols.length
    clearTimeout(feedbackTimer)
    if (!lines) {
      if (result.buildUpBonus > 0) {
        const message = result.newReadyLines > 0 ? 'NICE SETUP!' : 'KEEP GOING!'
        comboBanner.textContent = message + '  +' + result.buildUpBonus
        comboBanner.className = 'combo-banner tier-setup'
        comboBanner.hidden = false
        void comboBanner.offsetWidth
        comboBanner.classList.add('pop')
        showDiamond(result, true)
        if (!reducedMotion()) {
          const flash = document.createElement('span')
          flash.className = 'setup-glimmer'
          sparkLayer.append(flash)
        }
        feedbackTimer = setTimeout(() => {
          comboBanner.hidden = true
          sparkLayer.replaceChildren()
        }, 700)
      } else {
        comboBanner.hidden = true
        rewardDiamond.hidden = true
      }
      return
    }
    const tier = result.perfect ? 'perfect' : result.combo >= 8 ? 'legendary' :
      result.combo >= 5 ? 'amazing' : result.combo >= 3 || lines >= 3 ? 'great' : 'normal'
    const title = {
      perfect: 'PERFECT!', legendary: 'LEGENDARY!', amazing: 'AMAZING!',
      great: 'GREAT!', normal: lines >= 2 ? 'EXCELLENT!' : 'GOOD!',
    }[tier]
    comboBanner.textContent = title +
      (result.combo >= 2 ? '  COMBO ×' + result.combo : '') + '  +' + result.gained
    comboBanner.className = 'combo-banner tier-' + tier
    comboBanner.hidden = false
    void comboBanner.offsetWidth
    comboBanner.classList.add('pop')
    showDiamond(result)
    const reduced = reducedMotion()
    if (!reduced) {
      // One DOM insertion after constructing the FX scene keeps massive
      // row/column clears from triggering repeated paint work on Android.
      const effectNodes = []
      const boardColors = typeof window.getComputedStyle === 'function'
        ? window.getComputedStyle(document.body) : null
      const colorSnapshot = new Map()
      for (const cell of result.removedTiles || []) {
        if (!colorSnapshot.has(cell.color)) {
          const resolved = boardColors?.getPropertyValue('--color-' + cell.color)?.trim()
          colorSnapshot.set(cell.color, resolved || color(cell.color))
        }
      }
      // Render the departing blocks using their ORIGINAL stored colors. Even
      // after the board state is cleared they visibly rise, slide or twist away.
      const boardBounds = boardNode.getBoundingClientRect()
      const first = cells[0].getBoundingClientRect()
      const horizontalStep = cells[1].getBoundingClientRect().left - first.left
      const verticalStep = cells[rules.SIZE].getBoundingClientRect().top - first.top
      for (const cell of (result.removedTiles || []).slice(0, 64)) {
        const departing = document.createElement('span')
        departing.className = 'departing-tile'
        departing.style.left = (first.left - boardBounds.left + cell.col * horizontalStep) + 'px'
        departing.style.top = (first.top - boardBounds.top + cell.row * verticalStep) + 'px'
        departing.style.width = first.width + 'px'
        departing.style.height = first.height + 'px'
        departing.style.setProperty('--piece-color', colorSnapshot.get(cell.color) || color(cell.color))
        const rowClear = result.clearedRows.includes(cell.row)
        const colClear = result.clearedCols.includes(cell.col)
        departing.style.setProperty('--fly-x',
          (rowClear ? (cell.col < 4 ? -1 : 1) * (24 + cell.col * 4) : 0) + 'px')
        departing.style.setProperty('--fly-y',
          (colClear ? (cell.row < 4 ? -1 : 1) * (26 + cell.row * 4) : -10) + 'px')
        const rowIndex = result.clearedRows.indexOf(cell.row)
        const colIndex = result.clearedCols.indexOf(cell.col)
        // The same row (or column) departs as an ordered wave, rather than
        // forming a diagonal checkerboard pattern.
        const sweepOrder = rowIndex >= 0 ? rowIndex * 42 + cell.col * 12
          : colIndex * 42 + cell.row * 12
        departing.style.setProperty('--delay', (75 + sweepOrder) + 'ms')
        effectNodes.push(departing)
      }
      result.clearedRows.forEach((row, index) => {
        const streak = document.createElement('span')
        streak.className = 'clear-beam horizontal'
        streak.style.top = (row * 12.5) + '%'
        streak.style.setProperty('--delay', (20 + index * 42) + 'ms')
        effectNodes.push(streak)
      })
      result.clearedCols.forEach((col, index) => {
        const streak = document.createElement('span')
        streak.className = 'clear-beam vertical'
        streak.style.left = (col * 12.5) + '%'
        streak.style.setProperty('--delay', (20 + index * 42) + 'ms')
        effectNodes.push(streak)
      })
      // Emit tiles in board order, not random places: one continuous sweep.
      for (const cell of result.clearedCells.slice(0, lines >= 3 ? 56 : 32)) {
        const particle = document.createElement('i')
        particle.className = 'clear-fragment'
        particle.style.left = ((cell.col + .5) * 12.5) + '%'
        particle.style.top = ((cell.row + .5) * 12.5) + '%'
        particle.style.setProperty('--dx', ((cell.col - 3.5) * 9) + 'px')
        particle.style.setProperty('--dy', ((cell.row - 3.5) * 9) + 'px')
        const alignedIndex = result.clearedRows.includes(cell.row) ? cell.col : cell.row
        particle.style.setProperty('--delay', (95 + alignedIndex * 14) + 'ms')
        effectNodes.push(particle)
      }
      if (lines >= 2 || result.combo >= 3) {
        const shockwave = document.createElement('span')
        shockwave.className = 'clear-shockwave'
        effectNodes.push(shockwave)
      }
      if (result.perfect) {
        for (let i = 0; i < 20; i += 1) {
          const spark = document.createElement('i')
          spark.className = 'perfect-confetti'
          spark.style.left = ((i * 37) % 100) + '%'
          spark.style.top = ((i * 17) % 75) + '%'
          spark.style.setProperty('--dx', ((i * 19) % 120 - 60) + 'px')
          effectNodes.push(spark)
        }
      }
      sparkLayer.append(...effectNodes)
    }
    feedbackTimer = setTimeout(() => {
      comboBanner.hidden = true
      sparkLayer.replaceChildren()
    }, result.perfect ? 1150 : 850)
  }
  function playReward(result) {
    cancelRewardNotes()
    const lines = result.clearedRows.length + result.clearedCols.length
    if (!lines) {
      if (result.buildUpBonus > 0) {
        tone(400, .09, .035)
        scheduleRewardNote(610, .10, 90, .035)
      } else tone(270, .06)
      haptic('light')
      return
    }
    haptic(result.perfect || lines >= 3 ? 'heavy' : 'medium')
    const notes = Math.min(5, 1 + lines + Math.floor(result.combo / 3))
    const base = 470 + Math.min(340, result.combo * 38)
    // First audible hit follows the visual row flash, then notes accelerate
    // slightly on a larger multi-line clear without stacking harsh volume.
    for (let note = 0; note < notes; note += 1) {
      const duration = .095 + note * .014
      scheduleRewardNote(base * (1 + note * .18), duration, 55 + note * 72,
        Math.min(.053, .032 + note * .004))
    }
    if (result.perfect) {
      scheduleRewardNote(1050, .18, 385, .05)
      scheduleRewardNote(1340, .24, 490, .045)
    }
  }
  function celebratePerfect() {
    paletteIndex = (paletteIndex + 1) % PALETTES.length
    perfectCount += 1
    saveTheme()
    document.body.classList.add('palette-flash')
    paletteTimer = setTimeout(() => {
      applyPalette()
      document.body.classList.remove('palette-flash')
    }, 330)
  }


  function postToHost(type, payload = {}) {
    // Official Android/desktop protocol 2 uses the injected xb SDK, not hostNonce.
    if (globalThis.xb?.game) {
      if (!hostReady) return
      try {
        let operation
        if (type === 'game.ready') operation = globalThis.xb.ready()
        else if (type === 'game.start') operation = globalThis.xb.game.start()
        else if (type === 'game.progress') operation = globalThis.xb.game.progress({ score: payload.score })
        else if (type === 'game.result') operation = globalThis.xb.game.result({ score: payload.score })
        else if (type === 'game.exit') operation = globalThis.xb.exit()
        if (operation) void Promise.resolve(operation).catch(() => {})
      } catch { /* Host can be unavailable during shutdown. */ }
      return
    }
    if (runtime) { if (hostReady) runtime.send(type, payload); return }
    if (!hostReady || !HOST_NONCE || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, type, nonce: HOST_NONCE, ...payload }, '*')
  }

  function initialize() {
    if (hostReady) return
    hostReady = true
    document.body.classList.add('hosted')
    if (globalThis.xb?.info?.platform === 'android') document.body.classList.add('native-host')
    postToHost('game.ready')
    postToHost('game.start')
    postToHost('game.progress', { score: state.score, lines: state.lines })
    if (state.over) postToHost('game.result', { score: state.score, lines: state.lines })
  }

  if (!runtime) window.addEventListener('message', (event) => {
    const message = event.data
    if (!HOST_NONCE || event.source !== window.parent || !message || message.channel !== 'xiaban.game-host' ||
      message.protocol !== 1 || message.type !== 'host.init' || message.nonce !== HOST_NONCE) return
    initialize()
  })

  function color(value) { return 'var(--color-' + value + ')' }

  function drawShape(node, shape) {
    node.replaceChildren()
    if (!shape) return
    node.style.setProperty('--piece-width', shape.width)
    node.style.setProperty('--piece-height', shape.height)
    node.style.setProperty('--piece-color', color(shape.color))
    shape.cells.forEach((cell) => {
      const square = document.createElement('span')
      square.className = 'mini-cell'
      square.style.setProperty('--row', cell.row + 1)
      square.style.setProperty('--col', cell.col + 1)
      node.append(square)
    })
  }

  function clearPreview() {
    cells.forEach((cell, index) => {
      cell.classList.remove('preview', 'preview-invalid', 'preview-line')
      const value = state.board[Math.floor(index / rules.SIZE)][index % rules.SIZE]
      if (value) cell.style.setProperty('--piece-color', color(value))
      else cell.style.removeProperty('--piece-color')
    })
  }

  function showPreview(row, col) {
    clearPreview()
    const shape = selected === null ? null : state.hand[selected]
    if (!shape || state.over) return false
    const valid = rules.canPlace(state.board, shape, row, col)
    const projected = valid ? rules.placeShape(state.board, shape, row, col) : null
    if (projected) {
      cells.forEach((cell, i) => {
        if (projected.clearedRows.includes(Math.floor(i / 8)) || projected.clearedCols.includes(i % 8)) {
          cell.classList.add('preview-line')
        }
      })
    }
    shape.cells.forEach((offset) => {
      const y = row + offset.row
      const x = col + offset.col
      if (y < 0 || y >= rules.SIZE || x < 0 || x >= rules.SIZE) return
      const cell = cells[y * rules.SIZE + x]
      cell.style.setProperty('--piece-color', color(shape.color))
      cell.classList.add(valid ? 'preview' : 'preview-invalid')
    })
    return valid
  }

  function updateSelection() {
    pieceButtons.forEach((button, slot) => { button.setAttribute('aria-pressed', String(slot === selected)) })
  }

  function selectPiece(slot, focusBoard = false) {
    if (state.over || !state.hand[slot]) return
    selected = slot
    updateSelection()
    const shape = state.hand[slot]
    statusNode.textContent = rules.hasSpace(state.board, shape)
      ? '已选 ' + (slot + 1) + ' 号方块。点棋盘放置，左上格是落点。'
      : '这个方块暂时放不下，可以先放其他方块。'
    showPreview(cursor.row, cursor.col)
    if (focusBoard) cells[cursor.row * rules.SIZE + cursor.col].focus({ preventScroll: true })
  }

  function renderBoard(result = null) {
    cells.forEach((cell, index) => {
      const row = Math.floor(index / rules.SIZE)
      const col = index % rules.SIZE
      const value = state.board[row][col]
      cell.className = 'cell' + (value ? ' filled' : '')
      if (value) cell.style.setProperty('--piece-color', color(value))
      else cell.style.removeProperty('--piece-color')
      cell.setAttribute('aria-label', '第 ' + (row + 1) + ' 行第 ' + (col + 1) + ' 列，' + (value ? '已有方块' : '空格'))
      cell.disabled = state.over
      cell.tabIndex = !state.over && cursor.row === row && cursor.col === col ? 0 : -1
      if (result?.clearedCells.some((entry) => entry.row === row && entry.col === col)) {
        // Restart the short flash even if two consecutive moves clear this cell.
        void cell.offsetWidth
        cell.classList.add('clear-flash')
      }
    })
    document.getElementById('score').textContent = state.score
    document.getElementById('best').textContent = best
    document.getElementById('lines').textContent = state.lines
    const combo = document.getElementById('comboCount')
    combo.textContent = state.combo >= 2 ? 'COMBO ×' + state.combo : ''
    combo.hidden = state.combo < 2
  }

  function renderHand() {
    pieceButtons.forEach((button, slot) => {
      const shape = state.hand[slot]
      const blocked = shape && !rules.hasSpace(state.board, shape)
      button.disabled = !shape || state.over
      button.classList.toggle('used', !shape)
      button.classList.toggle('blocked', Boolean(blocked))
      drawShape(button.querySelector('.piece-art'), shape)
      button.querySelector('.piece-label').textContent = !shape ? '已放好' : (slot + 1) + ' 号' + (blocked ? ' · 暂放不下' : '')
      button.setAttribute('aria-label', !shape ? (slot + 1) + ' 号已使用' : (slot + 1) + ' 号，' + shape.name + '，' + shape.cells.length + ' 格' + (blocked ? '，暂时放不下' : '。选中后点棋盘或拖动放置'))
    })
    updateSelection()
  }

  function focusCursor() {
    cells.forEach((cell, index) => { cell.tabIndex = index === cursor.row * rules.SIZE + cursor.col ? 0 : -1 })
    cells[cursor.row * rules.SIZE + cursor.col].focus({ preventScroll: true })
  }

  function stopDrag() {
    const previous = drag
    drag = null
    ghost.hidden = true
    ghost.classList.remove('invalid')
    if (previous && pieceButtons[previous.slot].hasPointerCapture(previous.id)) {
      pieceButtons[previous.slot].releasePointerCapture(previous.id)
    }
    clearPreview()
  }

  function showResult() {
    window.XiabanResultShare?.finish({ gameId: 'block-puzzle', score: state.score, lines: state.lines })
    stopDrag()
    overlay.hidden = false
    restartButton.disabled = true
    exitButton.disabled = true
    document.getElementById('resultScore').textContent = state.score.toLocaleString('zh-CN')
    document.getElementById('resultLines').textContent = '消除 ' + state.lines + ' 行列 · 最佳连消 ×' + Math.max(1, state.bestCombo)
    statusNode.textContent = '剩余方块都放不下啦。'
    postToHost('game.result', { score: state.score, lines: state.lines })
    restartOverlay.focus({ preventScroll: true })
  }

  function placeAt(row, col) {
    if (state.over) return
    if (selected === null) {
      statusNode.textContent = '先在下方选一个方块，再点棋盘。'
      return
    }
    const result = rules.place(state, selected, row, col)
    if (!result.placed) {
      showPreview(row, col)
      statusNode.textContent = '这里放不下，换个空位试试。'
      return
    }
    const previousScore = state.score
    state = result.state
    if (result.perfect) celebratePerfect()
    best = Math.max(best, state.score)
    turns += 1
    persist()
    showFeedback(result)
    const lineCount = result.clearedRows.length + result.clearedCols.length
    playReward(result)
    selected = null
    renderBoard(result)
    animateScore(previousScore, state.score)
    renderHand()
    statusNode.textContent = result.perfect
      ? '完美全清！奖励 300 分，棋盘换新主题！'
      : lineCount
        ? (result.combo >= 3 ? '连消 ×' + result.combo + '！ ' : '') +
          '爆破 ' + lineCount + ' 条行列，+' + result.gained + ' 分！'
        : result.buildUpBonus > 0
          ? '布局奖励 +' + result.buildUpBonus + '！' +
            (result.newReadyLines ? ' 距离下一次消除只差一格。' : ' 稳稳铺垫，也能积累奖励。')
        : state.combo > 0 && state.drought < 3
          ? '连击保留！还有 ' + (3 - state.drought) + ' 步缓冲'
          : '+' + result.gained + ' 分。' + (result.replenished ? '新的一组三块来了。' : '继续布局，寻找爆破机会。')
    postToHost('game.progress', { score: state.score, lines: state.lines })
    if (state.over) {
      if (lineCount) resultTimer = setTimeout(showResult, result.perfect ? 900 : 610)
      else showResult()
    }
  }

  function moveDrag(event) {
    if (!drag || event.pointerId !== drag.id) return
    if (!drag.moving && Math.hypot(event.clientX - drag.x, event.clientY - drag.y) < 5) return
    drag.moving = true
    const shape = state.hand[drag.slot]
    const first = cells[0].getBoundingClientRect()
    const second = cells[1].getBoundingClientRect()
    const step = second.left - first.left
    const width = shape.width * step - 3
    const height = shape.height * step - 3
    const left = event.clientX - width / 2
    // Lift the entire shape above a finger so the landing cells stay visible.
    const top = drag.touch ? event.clientY - height - 42 : event.clientY - height / 2
    drag.row = Math.round((top - first.top) / step)
    drag.col = Math.round((left - first.left) / step)
    ghost.style.left = left + 'px'
    ghost.style.top = top + 'px'
    ghost.style.setProperty('--ghost-cell', first.width + 'px')
    ghost.hidden = false
    const valid = showPreview(drag.row, drag.col)
    ghost.classList.toggle('invalid', !valid)
    if (valid !== drag.valid) statusNode.textContent = valid ? '落点亮起啦，松手放置。' : '红色位置放不下，移到完整的空位。'
    drag.valid = valid
  }

  for (let row = 0; row < rules.SIZE; row += 1) {
    const rowNode = document.createElement('div')
    rowNode.className = 'board-row'
    rowNode.setAttribute('role', 'row')
    for (let col = 0; col < rules.SIZE; col += 1) {
      const cell = document.createElement('button')
      cell.type = 'button'
      cell.className = 'cell'
      cell.setAttribute('role', 'gridcell')
      cell.addEventListener('click', () => {
        cursor = { row, col }
        focusCursor()
        placeAt(row, col)
      })
      cell.addEventListener('pointerenter', (event) => {
        if (event.pointerType === 'mouse' && !drag && selected !== null) showPreview(row, col)
      })
      cell.addEventListener('focus', () => {
        cursor = { row, col }
        if (!drag) showPreview(row, col)
      })
      cells.push(cell)
      rowNode.append(cell)
    }
    boardNode.append(rowNode)
  }
  boardNode.addEventListener('pointerleave', () => { if (!drag) clearPreview() })

  pieceButtons.forEach((button, slot) => {
    button.addEventListener('click', () => { if (performance.now() >= suppressClickUntil) selectPiece(slot) })
    button.addEventListener('pointerdown', (event) => {
      if (!event.isPrimary || event.button !== 0 || state.over || !state.hand[slot]) return
      stopDrag()
      selectPiece(slot)
      drag = { id: event.pointerId, slot, x: event.clientX, y: event.clientY, touch: event.pointerType === 'touch', moving: false }
      drawShape(ghost, state.hand[slot])
      button.setPointerCapture(event.pointerId)
    })
    button.addEventListener('pointermove', moveDrag)
    button.addEventListener('pointerup', (event) => {
      if (!drag || drag.id !== event.pointerId) return
      // Recompute from the release coordinate in case the last move was coalesced.
      moveDrag(event)
      const previous = drag
      stopDrag()
      if (previous.moving) {
        suppressClickUntil = performance.now() + 300
        placeAt(previous.row, previous.col)
      }
    })
    button.addEventListener('pointercancel', stopDrag)
    button.addEventListener('lostpointercapture', () => { if (drag?.slot === slot) stopDrag() })
  })

  function exitGame() {
    if (hostReady) postToHost('game.exit')
    else window.location.assign('/games')
  }

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return
    if (event.key === 'Escape') {
      if (!settingsOverlay.hidden) { event.preventDefault(); settingsOverlay.hidden = true; settingsButton.focus(); return }
      if (!document.body.classList.contains('native-host')) { event.preventDefault(); stopDrag(); exitGame() }
      return
    }
    if (state.over) {
      if (event.key === 'Tab') {
        const shareButton = document.getElementById('shareResultButton')
        event.preventDefault()
        ;(shareButton && !shareButton.hidden && document.activeElement !== shareButton ? shareButton : restartOverlay).focus()
      }
      return
    }
    if (['1', '2', '3'].includes(event.key)) {
      event.preventDefault()
      selectPiece(Number(event.key) - 1, true)
      return
    }
    const direction = { ArrowLeft: [0, -1], ArrowRight: [0, 1], ArrowUp: [-1, 0], ArrowDown: [1, 0] }[event.key]
    if (direction && (selected !== null || boardNode.contains(document.activeElement))) {
      event.preventDefault()
      cursor = { row: Math.max(0, Math.min(7, cursor.row + direction[0])), col: Math.max(0, Math.min(7, cursor.col + direction[1])) }
      focusCursor()
    } else if ((event.key === 'Enter' || event.key === ' ') && boardNode.contains(document.activeElement)) {
      event.preventDefault()
      placeAt(cursor.row, cursor.col)
    }
  })

  function resetGame(fresh = true) {
    window.XiabanResultShare?.reset()
    clearVisualTimers()
    document.body.classList.remove('palette-flash')
    applyPalette()
    if (state) stopDrag()
    state = rules.createGame()
    if (!fresh) {
      const previous = readLocal()
      hasLocalSave = acceptSave(previous)
      if (!offerContinue(previous)) offerContinue(readArchived())
      hasLocalTheme = acceptTheme(readLocalTheme())
      // Preserve the old round in a separate archive before starting fresh.
    } else turns += 1
    selected = null
    cursor = { row: 0, col: 0 }
    overlay.hidden = true
    settingsOverlay.hidden = true
    comboBanner.hidden = true
    sparkLayer.replaceChildren()
    restartButton.disabled = false
    exitButton.disabled = false
    renderBoard()
    renderHand()
    statusNode.textContent = '拖方块到棋盘，或选方块后点空格。'
    postToHost('game.start')
    if (fresh) persist()
    if (state.over) showResult()
  }

  restartButton.addEventListener('click', () => resetGame())
  restartOverlay.addEventListener('click', () => { resetGame(); pieceButtons[0].focus({ preventScroll: true }) })
  settingsButton.addEventListener('click', () => { stopDrag(); settingsOverlay.hidden = false; closeSettingsButton.focus() })
  closeSettingsButton.addEventListener('click', () => { settingsOverlay.hidden = true; settingsButton.focus() })
  soundButton.addEventListener('click', () => { soundEnabled = !soundEnabled; soundUI(); persist(); if (soundEnabled) tone(440) })
  settingsRestart.addEventListener('click', () => { resetGame(); pieceButtons[0].focus() })
  continueButton.addEventListener('click', continuePrevious)
  soundUI()
  exitButton.addEventListener('click', exitGame)
  window.addEventListener('resize', stopDrag)
  window.addEventListener('blur', () => { if (drag) stopDrag() })
  resetGame(false)
  if (hostStorage?.get) void restoreFromHost()
  else { recovering = false; archiveContinue(); persist() }
  if (globalThis.xb?.ready) {
    initialize()
    globalThis.xb.on?.('hide', () => {
      if (drag) stopDrag()
      cancelRewardNotes()
      persist()
    })
  } else if (runtime) runtime.connect(initialize, { hide: () => {
    if (drag) stopDrag()
    cancelRewardNotes()
    persist()
  } })

})()
