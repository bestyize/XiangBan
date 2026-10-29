(() => {
  'use strict'

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

  function postToHost(type, payload = {}) {
    if (!hostReady || !HOST_NONCE || window.parent === window) return
    window.parent.postMessage({ channel: PACK_CHANNEL, protocol: PROTOCOL, type, nonce: HOST_NONCE, ...payload }, '*')
  }

  window.addEventListener('message', (event) => {
    const message = event.data
    if (
      !HOST_NONCE || event.source !== window.parent || !message ||
      message.channel !== HOST_CHANNEL || message.protocol !== PROTOCOL ||
      message.type !== 'host.init' || message.nonce !== HOST_NONCE
    ) return
    if (hostReady) { postToHost('game.ready'); return }
    document.body.classList.add('hosted')
    hostReady = true
    postToHost('game.ready')
    postToHost('game.start')
    postToHost('game.progress', { score, maxTile: Math.max(...board.flat()) })
    if (overlayMode === 'lost') postToHost('game.result', { score, maxTile: Math.max(...board.flat()) })
    boardNode.focus({ preventScroll: true })
  })

  function render(spawned = null, merges = []) {
    const tiles = []
    board.forEach((row, y) => row.forEach((value, x) => {
      const tile = document.createElement('div')
      tile.className = 'tile'
      tile.dataset.value = String(value)
      tile.setAttribute('aria-label', '第 ' + (y + 1) + ' 行第 ' + (x + 1) + ' 列：' + (value || '空'))
      tile.textContent = value ? String(value) : ''
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
    postToHost('game.progress', { score, maxTile: Math.max(...board.flat()) })
  }

  function setControlsDisabled(disabled) {
    directionButtons.forEach((button) => { button.disabled = disabled })
    restartButton.disabled = disabled
    exitButton.disabled = disabled
  }

  function showResult(mode) {
    window.XiabanResultShare?.reset()
    if (mode !== 'won') window.XiabanResultShare?.finish({ gameId: 'merge-2048', score, maxTile: Math.max(...board.flat()) })
    if (mode !== 'won') postToHost('game.result', { score, maxTile: Math.max(...board.flat()) })
    overlayMode = mode
    pointerStart = null
    const isWin = mode === 'won'
    document.getElementById('resultEyebrow').textContent = isWin ? '下班后的一个小成就' : '这一局结束啦'
    document.getElementById('resultTitle').textContent = isWin ? '合到 2048 啦！' : '歇一会，再来一局？'
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
    const result = rules.move(board, direction)
    if (!result.moved) {
      statusNode.textContent = '这个方向移不动，换个方向试试。'
      return
    }
    const spawned = rules.addTile(result.board)
    board = spawned.board
    score += result.score
    best = Math.max(best, score)
    render(spawned.cell, result.merges)
    statusNode.textContent = result.score > 0
      ? directionLabels[direction] + '合并，+' + result.score + ' 分。'
      : directionLabels[direction] + '移动，继续找相同的数字。'
    if (!won && rules.hasWon(board)) {
      won = true
      showResult('won')
    } else if (!rules.canMove(board)) {
      showResult('lost')
    }
  }

  function resetGame() {
    window.XiabanResultShare?.reset()
    board = rules.addTile(rules.addTile(rules.createBoard()).board).board
    score = 0
    won = false
    pointerStart = null
    hideResult()
    postToHost('game.start')
    render()
    statusNode.textContent = '滑动棋盘，或者按方向键，开始合数字。'
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

  resetGame()
})()
