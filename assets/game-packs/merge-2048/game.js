(() => {
  'use strict'

  const runtime = globalThis.XiaobanBuiltinRuntime
  if (runtime) document.body.classList.add('runtime-v2')

  const rules = globalThis.Xiaban2048
  const HOST_NONCE = new URLSearchParams(window.location.search).get('hostNonce')
  const HOST_CHANNEL = 'xiaban.game-host'
  const PACK_CHANNEL = 'xiaban.game-pack'
  const PROTOCOL = 1
  const directionLabels = { left: '向左', right: '向右', up: '向上', down: '向下' }
  const keyDirections = {
    ArrowLeft: 'left', ArrowRight: 'right', ArrowUp: 'up', ArrowDown: 'down',
    a: 'left', d: 'right', w: 'up', s: 'down',
  }

  const boardNode = document.getElementById('board')
  const scoreNode = document.getElementById('score')
  const bestNode = document.getElementById('best')
  const largestNode = document.getElementById('largest')
  const statusNode = document.getElementById('status')
  const overlay = document.getElementById('overlay')
  const continueButton = document.getElementById('continueButton')
  const restartButton = document.getElementById('restartButton')
  const exitButton = document.getElementById('exitButton')
  const restartOverlay = document.getElementById('restartOverlay')
  const directionButtons = [...document.querySelectorAll('[data-direction]')]

  let board
  let score = 0
  let best = 0
  let won = false
  let overlayMode = null
  let pointerStart = null
  let hostReady = false
  let moveTimer = null
  let queuedDirection = null
  let soundEnabled = false
  let audioContext = null
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
  const sparks = document.getElementById('sparks')
  const soundButton = document.getElementById('soundButton')

  // Local synthesis keeps the pack offline; sound is explicitly opt-in.
  function playTone(value) {
    if (!soundEnabled || document.hidden) return
    try {
      const Audio = window.AudioContext || window.webkitAudioContext
      if (!Audio) return
      audioContext ||= new Audio()
      if (audioContext.state === 'suspended') { audioContext.resume().catch(() => {}); return }
      const time = audioContext.currentTime
      const frequency = 220 * Math.pow(2, (Math.log2(value || 2) % 12) / 12)
      for (const [ratio, volume] of [[1, .08], [2, .025]]) {
        const oscillator = audioContext.createOscillator()
        const gain = audioContext.createGain()
        oscillator.type = 'sine'
        oscillator.frequency.setValueAtTime(frequency * ratio, time)
        gain.gain.setValueAtTime(.0001, time)
        gain.gain.exponentialRampToValueAtTime(volume, time + .012)
        gain.gain.exponentialRampToValueAtTime(.0001, time + .55)
        oscillator.connect(gain).connect(audioContext.destination)
        oscillator.start(time)
        oscillator.stop(time + .6)
        oscillator.onended = () => { oscillator.disconnect(); gain.disconnect() }
      }
    } catch { /* Sound must never interrupt a move. */ }
  }

  function mergeFeedback(merges, gained) {
    if (!gained) return
    playTone(Math.max(...merges.map(({ row, col }) => board[row][col])))
    if (reducedMotion.matches) return
    const gain = document.getElementById('scoreGain')
    gain.textContent = '+' + gained
    gain.classList.remove('score-gain')
    void gain.offsetWidth
    gain.classList.add('score-gain')
    const gap = parseFloat(getComputedStyle(boardNode).gap)
    const step = (boardNode.clientWidth + gap) / 4
    const size = step - gap
    for (const { row, col } of merges) {
      for (let i = 0; i < 8; i++) {
        const spark = document.createElement('i')
        const angle = i / 8 * Math.PI * 2
        spark.className = 'spark'
        spark.style.left = (col * step + size / 2) + 'px'
        spark.style.top = (row * step + size / 2) + 'px'
        spark.style.setProperty('--dx', Math.cos(angle) * size * .8 + 'px')
        spark.style.setProperty('--dy', Math.sin(angle) * size * .8 + 'px')
        spark.addEventListener('animationend', () => spark.remove(), { once: true })
        sparks.append(spark)
      }
    }
  }

  function postToHost(type, payload = {}) {
    if (runtime) { if (hostReady) runtime.send(type, payload); return }
    if (!hostReady || !HOST_NONCE || window.parent === window) return
    window.parent.postMessage({ channel: PACK_CHANNEL, protocol: PROTOCOL, type, nonce: HOST_NONCE, ...payload }, '*')
  }

  function initialize() {
    if (hostReady) { postToHost('game.ready'); return }
    document.body.classList.add('hosted')
    hostReady = true
    postToHost('game.ready')
    postToHost('game.start')
    postToHost('game.progress', { score, maxTile: Math.max(...board.flat()) })
    if (overlayMode === 'lost') postToHost('game.result', { score, maxTile: Math.max(...board.flat()) })
    boardNode.focus({ preventScroll: true })
  }

  if (!runtime) window.addEventListener('message', (event) => {
    const message = event.data
    if (
      !HOST_NONCE || event.source !== window.parent || !message ||
      message.channel !== HOST_CHANNEL || message.protocol !== PROTOCOL ||
      message.type !== 'host.init' || message.nonce !== HOST_NONCE
    ) return
    initialize()
  })

  function render(spawned = null, merges = []) {
    const tiles = []
    board.forEach((row, y) => row.forEach((value, x) => {
      const cell = document.createElement('div')
      cell.className = 'cell'
      cell.setAttribute('aria-hidden', 'true')
      tiles.push(cell)
      if (!value) return
      const tile = document.createElement('div')
      tile.className = 'tile'
      tile.dataset.value = String(value)
      tile.dataset.row = String(y)
      tile.dataset.col = String(x)
      tile.style.setProperty('--x', x)
      tile.style.setProperty('--y', y)
      tile.setAttribute('aria-label', '第 ' + (y + 1) + ' 行第 ' + (x + 1) + ' 列：' + (value || '空'))
      const number = document.createElement('span')
      number.className = 'tile-number'
      number.textContent = String(value)
      tile.append(number)
      if (value >= 4096) tile.classList.add('tile-large')
      if (value >= 16384) tile.classList.add('tile-long')
      if (spawned?.row === y && spawned.col === x) tile.classList.add('tile-new')
      if (merges.some((cell) => cell.row === y && cell.col === x)) tile.classList.add('tile-merged')
      tiles.push(tile)
    }))
    boardNode.replaceChildren(...tiles)
    scoreNode.textContent = String(score)
    bestNode.textContent = String(best)
    largestNode.textContent = String(Math.max(...board.flat()))
    const level = Math.min(11, Math.log2(Math.max(...board.flat())))
    document.getElementById('journeyValue').textContent = String(level).padStart(2, '0') + ' / 11'
    document.getElementById('journeyFill').style.width = (level / 11 * 100) + '%'
    document.getElementById('journeyTrack').setAttribute('aria-valuenow', String(level))
    postToHost('game.progress', { score, maxTile: Math.max(...board.flat()) })
  }

  function setControlsDisabled(disabled) {
    directionButtons.forEach((button) => { button.disabled = disabled })
    restartButton.disabled = disabled
    exitButton.disabled = disabled
    soundButton.disabled = disabled
  }

  function showResult(mode) {
    window.XiabanResultShare?.reset()
    if (mode !== 'won') window.XiabanResultShare?.finish({ gameId: 'merge-2048', score, maxTile: Math.max(...board.flat()) })
    if (mode !== 'won') postToHost('game.result', { score, maxTile: Math.max(...board.flat()) })
    overlayMode = mode
    pointerStart = null
    const isWin = mode === 'won'
    document.getElementById('resultEyebrow').textContent = isWin ? '每一次相遇，终成光芒' : '这一束光，值得铭记'
    document.getElementById('resultTitle').textContent = isWin ? '2048 · 光芒已至' : '落幕，亦是序章'
    document.getElementById('resultScore').textContent = '本局得分 ' + score + ' · 最大方块 ' + Math.max(...board.flat())
    continueButton.hidden = !isWin || !rules.canMove(board)
    restartOverlay.classList.toggle('primary-button', continueButton.hidden)
    overlay.hidden = false
    boardNode.setAttribute('tabindex', '-1')
    setControlsDisabled(true)
    statusNode.textContent = isWin ? '合到 2048 了，漂亮！' : '没有可以移动的方块了。'
    ;(continueButton.hidden ? restartOverlay : continueButton).focus({ preventScroll: true })
  }

  function hideResult() {
    overlayMode = null
    overlay.hidden = true
    boardNode.setAttribute('tabindex', '0')
    setControlsDisabled(false)
  }

  function move(direction) {
    if (overlayMode) return
    if (moveTimer !== null) { queuedDirection = direction; return }
    const result = rules.move(board, direction)
    if (!result.moved) {
      statusNode.textContent = '这个方向移不动，换个方向试试。'
      return
    }
    const finishMove = () => {
      moveTimer = null
      const spawned = rules.addTile(result.board)
      board = spawned.board
      score += result.score
      best = Math.max(best, score)
      render(spawned.cell, result.merges)
      mergeFeedback(result.merges, result.score)
      statusNode.textContent = result.score > 0
        ? directionLabels[direction] + '合并，+' + result.score + ' 分。'
        : directionLabels[direction] + '移动，继续找相同的数字。'
      if (!won && rules.hasWon(board)) {
        won = true
        showResult('won')
      } else if (!rules.canMove(board)) {
        showResult('lost')
      }
      const next = queuedDirection
      queuedDirection = null
      if (next && !overlayMode) move(next)
    }
    if (reducedMotion.matches || typeof boardNode.animate !== 'function') { finishMove(); return }
    const step = (boardNode.clientWidth + parseFloat(getComputedStyle(boardNode).gap)) / 4
    for (const { from, to } of result.motions) {
      const tile = boardNode.querySelector(`[data-row="${from.row}"][data-col="${from.col}"]`)
      tile.classList.remove('tile-new', 'tile-merged')
      tile.animate([
        { transform: 'translate(0, 0)' },
        { transform: `translate(${(to.col - from.col) * step}px, ${(to.row - from.row) * step}px)` },
      ], { duration: 125, easing: 'cubic-bezier(.2,.65,.3,1)', fill: 'forwards' })
    }
    moveTimer = setTimeout(finishMove, 125)
  }

  function resetGame() {
    clearTimeout(moveTimer)
    moveTimer = null
    queuedDirection = null
    sparks.replaceChildren()
    document.getElementById('scoreGain').classList.remove('score-gain')
    window.XiabanResultShare?.reset()
    board = rules.addTile(rules.addTile(rules.createBoard()).board).board
    score = 0
    won = false
    pointerStart = null
    hideResult()
    postToHost('game.start')
    render()
    statusNode.textContent = '滑动棋盘，开始你的第一步。'
    boardNode.focus({ preventScroll: true })
  }

  function exitGame() {
    if (hostReady) postToHost('game.exit')
    else window.location.assign('/games')
  }

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return
    if (event.key === 'Escape') {
      event.preventDefault()
      exitGame()
      return
    }
    if (overlayMode) {
      if (event.key === 'Tab') {
        const buttons = continueButton.hidden ? [restartOverlay] : [continueButton, restartOverlay]
        const shareButton = document.getElementById('shareResultButton')
        if (shareButton && !shareButton.hidden) buttons.unshift(shareButton)
        const index = buttons.indexOf(document.activeElement)
        const next = event.shiftKey ? (index - 1 + buttons.length) % buttons.length : (index + 1) % buttons.length
        event.preventDefault()
        buttons[next].focus()
      }
      return
    }
    const direction = keyDirections[event.key] || keyDirections[event.key.toLowerCase()]
    if (!direction) return
    event.preventDefault()
    move(direction)
  })

  boardNode.addEventListener('pointerdown', (event) => {
    if (overlayMode || !event.isPrimary || event.button !== 0) return
    pointerStart = { id: event.pointerId, x: event.clientX, y: event.clientY }
    boardNode.setPointerCapture(event.pointerId)
  })
  boardNode.addEventListener('pointerup', (event) => {
    if (!pointerStart || pointerStart.id !== event.pointerId) return
    const dx = event.clientX - pointerStart.x
    const dy = event.clientY - pointerStart.y
    pointerStart = null
    if (boardNode.hasPointerCapture(event.pointerId)) boardNode.releasePointerCapture(event.pointerId)
    const threshold = Math.max(16, boardNode.clientWidth * 0.06)
    if (Math.max(Math.abs(dx), Math.abs(dy)) < threshold) return
    move(Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'right' : 'left') : (dy > 0 ? 'down' : 'up'))
  })
  boardNode.addEventListener('pointercancel', () => { pointerStart = null })
  boardNode.addEventListener('lostpointercapture', () => { pointerStart = null })

  directionButtons.forEach((button) => {
    button.addEventListener('click', () => move(button.dataset.direction))
  })
  restartButton.addEventListener('click', resetGame)
  restartOverlay.addEventListener('click', resetGame)
  exitButton.addEventListener('click', exitGame)
  continueButton.addEventListener('click', () => {
    hideResult()
    statusNode.textContent = '继续挑战，看看能合出多大的数字。'
    boardNode.focus({ preventScroll: true })
  })

  soundButton.addEventListener('click', () => {
    soundEnabled = !soundEnabled
    soundButton.setAttribute('aria-pressed', String(soundEnabled))
    document.getElementById('soundState').textContent = soundEnabled ? '开' : '关'
    if (soundEnabled) playTone(8)
    else if (audioContext?.state === 'running') audioContext.suspend().catch(() => {})
  })
  document.addEventListener('visibilitychange', () => {
    if (document.hidden && audioContext?.state === 'running') audioContext.suspend().catch(() => {})
  })

  // Reserve only the header area that actually intersects the host capsule.
  function fitCapsule() {
    const style = getComputedStyle(document.documentElement)
    const size = key => parseFloat(style.getPropertyValue('--xb-capsule-' + key)) || 0
    const topbar = document.querySelector('.topbar')
    const rect = topbar.getBoundingClientRect()
    const top = size('top'), height = size('height'), width = size('width')
    const overlap = width > 0 && height > 0 && rect.top < top + height + 8 && rect.bottom > top - 8
    topbar.style.paddingRight = overlap ? Math.max(0, rect.right - (innerWidth - size('right') - width) + 8) + 'px' : ''
  }
  window.addEventListener('resize', fitCapsule)
  window.addEventListener('safearea', fitCapsule)
  fitCapsule()

  resetGame()
  if (runtime) runtime.connect(initialize, {
    layout: fitCapsule,
    hide: () => { if (audioContext?.state === 'running') audioContext.suspend().catch(() => {}) },
  })

})()
