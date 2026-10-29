(() => {
  'use strict'

  const COLS = 10
  const ROWS = 20
  const CELL = 25
  const HOST_NONCE = new URLSearchParams(window.location.search).get('hostNonce')
  const HOST_CHANNEL = 'xiaban.game-host'
  const PACK_CHANNEL = 'xiaban.game-pack'
  const PROTOCOL = 1

  const SHAPES = {
    I: { color: '#00b9f5', matrix: [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]] },
    J: { color: '#4169ff', matrix: [[1,0,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]] },
    L: { color: '#ff8a16', matrix: [[0,0,1,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]] },
    O: { color: '#ffd12b', matrix: [[0,1,1,0],[0,1,1,0],[0,0,0,0],[0,0,0,0]] },
    S: { color: '#42d34f', matrix: [[0,1,1,0],[1,1,0,0],[0,0,0,0],[0,0,0,0]] },
    T: { color: '#a34dff', matrix: [[0,1,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]] },
    Z: { color: '#ff4f76', matrix: [[1,1,0,0],[0,1,1,0],[0,0,0,0],[0,0,0,0]] },
  }

  const boardCanvas = document.getElementById('board')
  const boardContext = boardCanvas.getContext('2d')
  const nextCanvas = document.getElementById('next')
  const nextContext = nextCanvas.getContext('2d')
  const scoreNode = document.getElementById('score')
  const linesNode = document.getElementById('lines')
  const levelNode = document.getElementById('level')
  const statusNode = document.getElementById('status')
  const pauseButton = document.getElementById('pauseButton')
  const gameOver = document.getElementById('gameOver')
  const resultScore = document.getElementById('resultScore')
  const resultTitle = document.getElementById('resultTitle')

  let board
  let current
  let nextType
  let bag
  let score
  let lines
  let level
  let paused
  let finished
  let timer = null
  let hostReady = false

  function postToHost(type, payload = {}) {
    if (!hostReady || !HOST_NONCE || window.parent === window) return
    window.parent.postMessage({
      channel: PACK_CHANNEL,
      protocol: PROTOCOL,
      type,
      nonce: HOST_NONCE,
      ...payload,
    }, '*')
  }

  window.addEventListener('message', (event) => {
    const message = event.data
    if (
      event.source !== window.parent ||
      !message ||
      message.channel !== HOST_CHANNEL ||
      message.protocol !== PROTOCOL ||
      message.type !== 'host.init' ||
      message.nonce !== HOST_NONCE
    ) return
    if (hostReady) { postToHost('game.ready'); return }
    document.body.classList.add('hosted')
    hostReady = true
    postToHost('game.ready')
    postToHost('game.start')
    postToHost('game.progress', { score, lines })
    if (finished) postToHost('game.result', { score, lines })
  })

  function shuffle(items) {
    for (let i = items.length - 1; i > 0; i -= 1) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[items[i], items[j]] = [items[j], items[i]]
    }
    return items
  }

  function takeType() {
    if (!bag.length) bag = shuffle(Object.keys(SHAPES))
    return bag.pop()
  }

  function emptyBoard() {
    return Array.from({ length: ROWS }, () => Array(COLS).fill(null))
  }

  function makePiece(type) {
    return {
      type,
      color: SHAPES[type].color,
      matrix: SHAPES[type].matrix.map((row) => row.slice()),
      x: 3,
      y: 0,
    }
  }

  function collides(matrix, x, y) {
    for (let row = 0; row < matrix.length; row += 1) {
      for (let col = 0; col < matrix[row].length; col += 1) {
        if (!matrix[row][col]) continue
        const boardX = x + col
        const boardY = y + row
        if (boardX < 0 || boardX >= COLS || boardY >= ROWS) return true
        if (boardY >= 0 && board[boardY][boardX]) return true
      }
    }
    return false
  }

  function drawBlock(context, x, y, color, size, alpha = 1) {
    context.globalAlpha = alpha
    context.fillStyle = color
    context.beginPath()
    context.roundRect(x * size + 1.5, y * size + 1.5, size - 3, size - 3, size * .14)
    if (alpha < 1) {
      context.globalAlpha = .5
      context.strokeStyle = color
      context.lineWidth = 1.3
      context.stroke()
    } else context.fill()
    context.globalAlpha = 1
  }

  function drawMatrix(context, matrix, offsetX, offsetY, color, size, alpha = 1) {
    matrix.forEach((row, y) => row.forEach((cell, x) => {
      if (cell) drawBlock(context, x + offsetX, y + offsetY, color, size, alpha)
    }))
  }

  function draw() {
    boardContext.fillStyle = '#fffdf5'
    boardContext.fillRect(0, 0, COLS * CELL, ROWS * CELL)
    boardContext.strokeStyle = '#e7eee0'
    boardContext.lineWidth = 1
    for (let x = 1; x < COLS; x += 1) {
      boardContext.beginPath()
      boardContext.moveTo(x * CELL + 0.5, 0)
      boardContext.lineTo(x * CELL + 0.5, ROWS * CELL)
      boardContext.stroke()
    }
    for (let y = 1; y < ROWS; y += 1) {
      boardContext.beginPath()
      boardContext.moveTo(0, y * CELL + 0.5)
      boardContext.lineTo(COLS * CELL, y * CELL + 0.5)
      boardContext.stroke()
    }

    board.forEach((row, y) => row.forEach((color, x) => {
      if (color) drawBlock(boardContext, x, y, color, CELL)
    }))

    if (current && !finished) {
      let ghostY = current.y
      while (!collides(current.matrix, current.x, ghostY + 1)) ghostY += 1
      drawMatrix(boardContext, current.matrix, current.x, ghostY, current.color, CELL, 0.22)
      drawMatrix(boardContext, current.matrix, current.x, current.y, current.color, CELL)
    }
  }

  function drawNext() {
    nextContext.clearRect(0, 0, nextCanvas.width, nextCanvas.height)
    if (!nextType) return
    const matrix = SHAPES[nextType].matrix
    const cells = matrix.flatMap((row, y) => row.flatMap((cell, x) => cell ? [{ x, y }] : []))
    const minX = Math.min(...cells.map(cell => cell.x)), maxX = Math.max(...cells.map(cell => cell.x))
    const minY = Math.min(...cells.map(cell => cell.y)), maxY = Math.max(...cells.map(cell => cell.y))
    const size = 20
    drawMatrix(nextContext, matrix, (nextCanvas.width / size - (maxX - minX + 1)) / 2 - minX, (nextCanvas.height / size - (maxY - minY + 1)) / 2 - minY, SHAPES[nextType].color, size)
  }

  function updateStats() {
    scoreNode.textContent = String(score)
    linesNode.textContent = String(lines)
    levelNode.textContent = String(level)
    postToHost('game.progress', { score, lines })
  }

  function dropInterval() {
    return Math.max(90, Math.floor(650 * Math.pow(0.82, level - 1)))
  }

  function startTimer() {
    if (timer !== null) window.clearInterval(timer)
    timer = null
    if (!paused && !finished) timer = window.setInterval(stepDown, dropInterval())
  }

  function spawnPiece() {
    current = makePiece(nextType)
    nextType = takeType()
    drawNext()
    if (collides(current.matrix, current.x, current.y)) endGame()
    draw()
  }

  function setStatus(text) {
    statusNode.textContent = text
  }

  function endGame() {
    window.XiabanResultShare?.finish({ gameId: 'falling-blocks', score, lines })
    postToHost('game.result', { score, lines })
    finished = true
    paused = false
    if (timer !== null) window.clearInterval(timer)
    timer = null
    resultTitle.textContent = '再来一局？'
    resultScore.textContent = '本局得分 ' + score + '，消除 ' + lines + ' 行'
    gameOver.hidden = false
    setStatus('这一局结束啦。')
    document.getElementById('shareResultButton')?.focus({ preventScroll: true })
  }

  function clearFullRows() {
    let cleared = 0
    for (let y = ROWS - 1; y >= 0; y -= 1) {
      if (board[y].every(Boolean)) {
        board.splice(y, 1)
        board.unshift(Array(COLS).fill(null))
        cleared += 1
        y += 1
      }
    }
    if (cleared > 0) {
      const points = [0, 100, 300, 500, 800][cleared] ?? cleared * 200
      score += points * level
      lines += cleared
      level = Math.floor(lines / 10) + 1
      updateStats()
      setStatus(cleared === 4 ? '漂亮！一次消掉四行。' : '消掉 ' + cleared + ' 行，继续保持。')
      startTimer()
    }
  }

  function lockPiece() {
    current.matrix.forEach((row, y) => row.forEach((cell, x) => {
      if (!cell) return
      const boardY = current.y + y
      const boardX = current.x + x
      if (boardY < 0) {
        endGame()
        return
      }
      board[boardY][boardX] = current.color
    }))
    if (finished) return
    clearFullRows()
    spawnPiece()
  }

  function stepDown() {
    if (paused || finished || !current) return
    if (!collides(current.matrix, current.x, current.y + 1)) {
      current.y += 1
      draw()
      return
    }
    lockPiece()
  }

  function move(dx) {
    if (paused || finished || !current) return
    if (!collides(current.matrix, current.x + dx, current.y)) {
      current.x += dx
      draw()
    }
  }

  function rotate() {
    if (paused || finished || !current) return
    const rotated = current.matrix[0].map((_, index) =>
      current.matrix.map((row) => row[index]).reverse(),
    )
    for (const kick of [0, -1, 1, -2, 2]) {
      if (!collides(rotated, current.x + kick, current.y)) {
        current.matrix = rotated
        current.x += kick
        draw()
        return
      }
    }
  }

  function hardDrop() {
    if (paused || finished || !current) return
    let distance = 0
    while (!collides(current.matrix, current.x, current.y + 1)) {
      current.y += 1
      distance += 1
    }
    score += distance * 2
    updateStats()
    lockPiece()
    draw()
  }

  function togglePause() {
    if (finished) return
    paused = !paused
    document.body.classList.toggle('paused', paused)
    pauseButton.textContent = paused ? '继续' : '暂停'
    setStatus(paused ? '已暂停，歇好再继续。' : '方向键移动，↑旋转，空格快速下落。')
    startTimer()
  }

  function resetGame() {
    window.XiabanResultShare?.reset()
    if (timer !== null) window.clearInterval(timer)
    timer = null
    board = emptyBoard()
    bag = []
    nextType = takeType()
    score = 0
    lines = 0
    level = 1
    paused = false
    document.body.classList.remove('paused')
    finished = false
    postToHost('game.start')
    pauseButton.textContent = '暂停'
    gameOver.hidden = true
    updateStats()
    setStatus('方向键移动，↑旋转，空格快速下落。')
    spawnPiece()
    startTimer()
  }

  function performAction(action) {
    if (action === 'left') move(-1)
    if (action === 'right') move(1)
    if (action === 'down') stepDown()
    if (action === 'rotate') rotate()
    if (action === 'drop') hardDrop()
  }

  document.querySelectorAll('[data-action]').forEach((button) => {
    button.addEventListener('click', () => performAction(button.getAttribute('data-action')))
  })

  document.addEventListener('keydown', (event) => {
    if (finished && event.key === 'Tab') {
      const shareButton = document.getElementById('shareResultButton')
      const restart = document.getElementById('restartOverlay')
      event.preventDefault()
      ;(shareButton && !shareButton.hidden && document.activeElement !== shareButton ? shareButton : restart).focus({ preventScroll: true })
      return
    }
    const actionByKey = {
      ArrowLeft: 'left',
      ArrowRight: 'right',
      ArrowDown: 'down',
      ArrowUp: 'rotate',
      x: 'rotate',
      X: 'rotate',
      ' ': 'drop',
    }
    if (event.key === 'Escape') {
      postToHost('game.exit')
      return
    }
    if (event.key.toLowerCase() === 'p') {
      event.preventDefault()
      togglePause()
      return
    }
    const action = actionByKey[event.key]
    if (!action) return
    event.preventDefault()
    performAction(action)
  })

  document.getElementById('pauseButton').addEventListener('click', togglePause)
  document.getElementById('restartButton').addEventListener('click', resetGame)
  document.getElementById('restartOverlay').addEventListener('click', resetGame)
  document.getElementById('exitButton').addEventListener('click', () => {
    if (!hostReady) {
      window.location.assign('/games')
      return
    }
    postToHost('game.exit')
  })

  resetGame()
  if (typeof ResizeObserver === 'function') {
    const resize = new ResizeObserver(([entry]) => {
      const pixels = Math.max(1, Math.round(entry.contentRect.width * Math.min(window.devicePixelRatio || 1, 3)))
      if (boardCanvas.width === pixels) return
      boardCanvas.width = pixels
      boardCanvas.height = pixels * 2
      boardContext.setTransform(pixels / (COLS * CELL), 0, 0, pixels / (COLS * CELL), 0, 0)
      draw()
    })
    resize.observe(boardCanvas)
  }
})()
