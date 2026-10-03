(() => {
  'use strict'

  const rules = globalThis.XiabanWaterSort
  const HOST_NONCE = new URLSearchParams(window.location.search).get('hostNonce')
  const flavors = [
    { symbol: 'A', short: '柠', name: '柠檬' }, { symbol: 'B', short: '莓', name: '莓莓' },
    { symbol: 'C', short: '柚', name: '青柚' }, { symbol: 'D', short: '蓝', name: '蓝莓' },
    { symbol: 'E', short: '葡', name: '葡萄' }, { symbol: 'F', short: '桃', name: '蜜桃' },
    { symbol: 'G', short: '茶', name: '奶茶' }, { symbol: 'H', short: '汽', name: '苏打' },
  ]
  const boardNode = document.getElementById('board')
  const statusNode = document.getElementById('status')
  const levelSelect = document.getElementById('levelSelect')
  const undoButton = document.getElementById('undoButton')
  const resetButton = document.getElementById('resetButton')
  const exitButton = document.getElementById('exitButton')
  const overlay = document.getElementById('overlay')
  const nextButton = document.getElementById('nextButton')
  const replayButton = document.getElementById('replayButton')
  const completed = new Set()
  let hostReady = false
  let levelIndex = 0
  let highestUnlocked = 0
  let board = []
  let history = []
  let steps = 0
  let selected = null
  let won = false

  function score() { return rules.completedTubes(board) * 100 }

  function postToHost(type, payload = {}) {
    if (!hostReady || !HOST_NONCE || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, type, nonce: HOST_NONCE, ...payload }, '*')
  }

  function progress() {
    postToHost('game.progress', { score: score(), lines: 0, maxTile: 0 })
  }

  window.addEventListener('message', (event) => {
    const message = event.data
    if (hostReady || !HOST_NONCE || event.source !== window.parent || !message ||
      message.channel !== 'xiaban.game-host' || message.protocol !== 1 ||
      message.type !== 'host.init' || message.nonce !== HOST_NONCE) return
    hostReady = true
    document.body.classList.add('hosted')
    postToHost('game.ready')
    postToHost('game.start')
    progress()
    if (won) postToHost('game.result', { score: score(), lines: 0, maxTile: 0 })
  })

  function status(message, warning = false) {
    statusNode.textContent = message
    statusNode.dataset.tone = warning ? 'warning' : 'normal'
  }

  function render(flashIndex = null, focusIndex = null) {
    const buttons = board.map((tube, index) => {
      const button = document.createElement('button')
      button.type = 'button'
      button.className = 'tube-button'
      button.dataset.index = String(index)
      button.disabled = won
      button.setAttribute('aria-pressed', String(selected === index))
      const contents = tube.map((color) => flavors[color].symbol + ' ' + flavors[color].name).join('、')
      button.setAttribute('aria-label', '第 ' + (index + 1) + ' 杯，' +
        (tube.length ? '由底至顶：' + contents + '。剩余 ' + (rules.CAPACITY - tube.length) + ' 格' : '空杯，可装 4 格'))
      if (selected === index) button.classList.add('selected')
      if (selected !== null && rules.getPour(board, selected, index).valid) button.classList.add('can-receive')
      const sorted = tube.length === rules.CAPACITY && tube.every((color) => color === tube[0])
      if (sorted) button.classList.add('sorted')
      if (index === flashIndex) button.classList.add('flash')
      const glass = document.createElement('span')
      glass.className = 'glass'
      glass.setAttribute('aria-hidden', 'true')
      tube.forEach((color) => {
        const segment = document.createElement('span')
        segment.className = 'segment'
        segment.dataset.color = String(color)
        segment.textContent = flavors[color].symbol + '·' + flavors[color].short
        glass.append(segment)
      })
      if (!tube.length) {
        const label = document.createElement('span')
        label.className = 'empty-label'
        label.textContent = '空杯'
        glass.append(label)
      }
      const number = document.createElement('span')
      number.className = 'tube-number'
      number.textContent = (index + 1) + (sorted ? ' ✓' : '')
      number.setAttribute('aria-hidden', 'true')
      button.append(glass, number)
      button.addEventListener('click', () => choose(index))
      return button
    })
    boardNode.style.setProperty('--columns', String(Math.min(board.length, 5)))
    boardNode.replaceChildren(...buttons)
    document.getElementById('moves').textContent = String(steps)
    document.getElementById('levelNumber').textContent = (levelIndex + 1) + ' / ' + rules.levelCount
    document.getElementById('completed').textContent = completed.size + ' / ' + rules.levelCount
    undoButton.disabled = won || history.length === 0
    if (focusIndex !== null && !won) buttons[focusIndex].focus({ preventScroll: true })
  }

  function updateMenu() {
    levelSelect.replaceChildren(...Array.from({ length: rules.levelCount }, (_, index) => {
      const option = document.createElement('option')
      option.value = String(index)
      option.textContent = (index + 1) + '. ' + rules.getLevel(index).name +
        (completed.has(index) ? ' ✓' : index > highestUnlocked ? ' · 待解锁' : '')
      option.disabled = index > highestUnlocked
      return option
    }))
    levelSelect.value = String(levelIndex)
  }

  function setDialog(open) {
    overlay.hidden = !open
    resetButton.disabled = open
    exitButton.disabled = open
    levelSelect.disabled = open
  }

  function startLevel(index, focus = true) {
    const level = rules.getLevel(index)
    if (!level || index > highestUnlocked) return
    levelIndex = index
    board = level.board
    history = []
    selected = null
    steps = 0
    won = false
    setDialog(false)
    updateMenu()
    render(null, focus ? 0 : null)
    status('第 ' + (index + 1) + ' 关 · ' + level.name + '。点一杯，再点同色杯或空杯。')
    postToHost('game.start')
    progress()
  }

  function showWin() {
    won = true
    selected = null
    completed.add(levelIndex)
    highestUnlocked = Math.max(highestUnlocked, Math.min(levelIndex + 1, rules.levelCount - 1))
    updateMenu()
    render()
    const finalLevel = levelIndex === rules.levelCount - 1
    document.getElementById('resultTitle').textContent = finalLevel ? '今晚的饮料都调好啦！' : '第 ' + (levelIndex + 1) + ' 关完成！'
    document.getElementById('resultText').textContent = steps + ' 步，调好 ' + rules.completedTubes(board) + ' 杯饮料。' +
      (finalLevel ? '本次已完成 ' + completed.size + ' / ' + rules.levelCount + ' 关，菜单中可以重玩。' : '下一款口味，慢慢来。')
    nextButton.textContent = finalLevel ? '回到菜单' : '下一关'
    setDialog(true)
    status('本关完成！共倒水 ' + steps + ' 步。')
    postToHost('game.result', { score: score(), lines: 0, maxTile: 0 })
    nextButton.focus({ preventScroll: true })
  }

  function choose(index) {
    if (won) return
    if (selected === null) {
      if (!board[index].length) {
        status('这是空杯。先选一杯有饮料的。', true)
        return
      }
      selected = index
      const flavor = flavors[board[index][board[index].length - 1]]
      render(null, index)
      status('已选第 ' + (index + 1) + ' 杯，顶层是 ' + flavor.symbol + ' ' + flavor.name + '。点同口味或空杯来接。')
      return
    }
    if (selected === index) {
      selected = null
      render(null, index)
      status('已取消选择，可以重新选杯。')
      return
    }
    const result = rules.pour(board, selected, index)
    if (!result.moved) {
      const reasons = { full: '这杯已经满了，换一杯接。', color: '顶层口味不同，只能倒进同口味或空杯。', empty: '源杯是空的，请重新选择。', invalid: '这次无法倒水，请重新选杯。' }
      status(reasons[result.reason] || '不能倒进这杯。再点源杯可取消选择。', true)
      return
    }
    const from = selected
    history.push(rules.cloneBoard(board))
    board = result.board
    steps += 1
    selected = null
    render(index, index)
    progress()
    if (rules.isWon(board)) {
      showWin()
    } else if (!rules.hasMove(board)) {
      status('暂时没有能倒的杯子了。点“撤销”退一步，或重置本关。', true)
    } else {
      status('第 ' + (from + 1) + ' 杯 → 第 ' + (index + 1) + ' 杯，倒入 ' + result.amount + ' 格。继续慢慢调。')
    }
  }

  undoButton.addEventListener('click', () => {
    if (won || !history.length) return
    board = history.pop()
    steps -= 1
    selected = null
    render(null, 0)
    progress()
    status('已撤销一步，试试换个顺序。')
  })
  resetButton.addEventListener('click', () => startLevel(levelIndex))
  replayButton.addEventListener('click', () => startLevel(levelIndex))
  nextButton.addEventListener('click', () => {
    if (levelIndex < rules.levelCount - 1) startLevel(levelIndex + 1)
    else {
      setDialog(false)
      levelSelect.focus({ preventScroll: true })
      status('本次 8 关全部完成啦。菜单中可以挑一款饮料重玩。')
    }
  })
  levelSelect.addEventListener('change', () => startLevel(Number(levelSelect.value)))
  exitButton.addEventListener('click', () => {
    if (hostReady) postToHost('game.exit')
    else window.location.assign('/games')
  })
  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return
    if (!overlay.hidden) {
      if (event.key === 'Tab') {
        event.preventDefault()
        ;(document.activeElement === nextButton ? replayButton : nextButton).focus({ preventScroll: true })
      }
      return
    }
    if (event.key === 'Escape' && selected !== null) {
      event.preventDefault()
      const previous = selected
      selected = null
      render(null, previous)
      status('已取消选择。')
    }
  })

  startLevel(0, false)
})()
