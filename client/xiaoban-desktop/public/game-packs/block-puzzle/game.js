(() => {
  'use strict'

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

  function postToHost(type, payload = {}) {
    if (!hostReady || !HOST_NONCE || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, type, nonce: HOST_NONCE, ...payload }, '*')
  }

  window.addEventListener('message', (event) => {
    const message = event.data
    if (!HOST_NONCE || event.source !== window.parent || !message || message.channel !== 'xiaban.game-host' ||
      message.protocol !== 1 || message.type !== 'host.init' || message.nonce !== HOST_NONCE) return
    if (hostReady) {
      postToHost('game.ready')
      return
    }
    hostReady = true
    document.body.classList.add('hosted')
    postToHost('game.ready')
    postToHost('game.start')
    postToHost('game.progress', { score: state.score, lines: state.lines })
    if (state.over) postToHost('game.result', { score: state.score, lines: state.lines })
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
      cell.classList.remove('preview', 'preview-invalid')
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
    document.getElementById('resultScore').textContent = '本局 ' + state.score + ' 分 · 消除 ' + state.lines + ' 条行列'
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
    state = result.state
    best = Math.max(best, state.score)
    selected = null
    renderBoard(result)
    renderHand()
    const lineCount = result.clearedRows.length + result.clearedCols.length
    statusNode.textContent = lineCount
      ? '消掉 ' + lineCount + ' 条行列，+' + result.gained + ' 分！'
      : '+' + result.gained + ' 分。' + (result.replenished ? '三个新方块来啦。' : '继续摆，找机会消一整条。')
    postToHost('game.progress', { score: state.score, lines: state.lines })
    if (state.over) showResult()
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
    if (event.key === 'Escape') { event.preventDefault(); stopDrag(); exitGame(); return }
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

  function resetGame() {
    window.XiabanResultShare?.reset()
    if (state) stopDrag()
    state = rules.createGame()
    selected = null
    cursor = { row: 0, col: 0 }
    overlay.hidden = true
    restartButton.disabled = false
    exitButton.disabled = false
    renderBoard()
    renderHand()
    statusNode.textContent = '拖方块到棋盘，或选方块后点空格。'
    postToHost('game.start')
  }

  restartButton.addEventListener('click', resetGame)
  restartOverlay.addEventListener('click', () => { resetGame(); pieceButtons[0].focus({ preventScroll: true }) })
  exitButton.addEventListener('click', exitGame)
  window.addEventListener('resize', stopDrag)
  window.addEventListener('blur', () => { if (drag) stopDrag() })
  resetGame()
})()
