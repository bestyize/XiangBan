(() => {
  'use strict'

  const SIZE = 8
  const patterns = [
    ['dot', '小方块', [[0, 0]]],
    ['h2', '横向两格', [[0, 0], [0, 1]]],
    ['v2', '竖向两格', [[0, 0], [1, 0]]],
    ['h3', '横向三格', [[0, 0], [0, 1], [0, 2]]],
    ['v3', '竖向三格', [[0, 0], [1, 0], [2, 0]]],
    ['h4', '横向四格', [[0, 0], [0, 1], [0, 2], [0, 3]]],
    ['v4', '竖向四格', [[0, 0], [1, 0], [2, 0], [3, 0]]],
    ['h5', '横向五格', [[0, 0], [0, 1], [0, 2], [0, 3], [0, 4]]],
    ['v5', '竖向五格', [[0, 0], [1, 0], [2, 0], [3, 0], [4, 0]]],
    ['square2', '四格正方形', [[0, 0], [0, 1], [1, 0], [1, 1]]],
    ['square3', '九格正方形', [[0, 0], [0, 1], [0, 2], [1, 0], [1, 1], [1, 2], [2, 0], [2, 1], [2, 2]]],
    ['corner1', '小折角', [[0, 0], [1, 0], [1, 1]]],
    ['corner2', '小折角', [[0, 1], [1, 0], [1, 1]]],
    ['corner3', '小折角', [[0, 0], [0, 1], [1, 0]]],
    ['corner4', '小折角', [[0, 0], [0, 1], [1, 1]]],
    ['l1', '长折角', [[0, 0], [1, 0], [2, 0], [2, 1]]],
    ['l2', '长折角', [[0, 1], [1, 1], [2, 0], [2, 1]]],
    ['l3', '长折角', [[0, 0], [0, 1], [1, 0], [2, 0]]],
    ['l4', '长折角', [[0, 0], [0, 1], [1, 1], [2, 1]]],
    ['t1', 'T形方块', [[0, 0], [0, 1], [0, 2], [1, 1]]],
    ['t2', 'T形方块', [[0, 1], [1, 0], [1, 1], [1, 2]]],
    ['z1', '阶梯方块', [[0, 0], [0, 1], [1, 1], [1, 2]]],
    ['z2', '阶梯方块', [[0, 1], [0, 2], [1, 0], [1, 1]]],
    ['rectH', '横向六格', [[0, 0], [0, 1], [0, 2], [1, 0], [1, 1], [1, 2]]],
    ['rectV', '竖向六格', [[0, 0], [0, 1], [1, 0], [1, 1], [2, 0], [2, 1]]],
    ['diagDown', '右下斜线', [[0, 0], [1, 1], [2, 2]]],
    ['diagUp', '左下斜线', [[0, 2], [1, 1], [2, 0]]],
  ]
  const SHAPES = Object.freeze(patterns.map(([id, name, cells], index) => Object.freeze({
    id, name,
    cells: Object.freeze(cells.map(([row, col]) => Object.freeze({ row, col }))),
    width: Math.max(...cells.map((cell) => cell[1])) + 1,
    height: Math.max(...cells.map((cell) => cell[0])) + 1,
    color: index % 6 + 1,
  })))

  function createBoard() {
    return Array.from({ length: SIZE }, () => Array(SIZE).fill(0))
  }

  function pickShape(pool, random) {
    return pool[Math.max(0, Math.min(pool.length - 1, Math.floor(random() * pool.length)))]
  }

  // Partial board awareness prevents an avoidable instant loss from a randomly
  // impossible hand. It does not guarantee every future hand can be finished.
  function generateHand(random = Math.random, board = null) {
    const fitting = board ? SHAPES.filter((shape) => hasSpace(board, shape)) : SHAPES
    if (!fitting.length) return Array.from({ length: 3 }, () => pickShape(SHAPES, random))
    const empty = board ? board.flat().filter((value) => value === 0).length : 64
    const guidedChance = board ? Math.min(.88, Math.max(.18, 1 - empty / 70)) : 0
    const hand = Array.from({ length: 3 }, () =>
      pickShape(board && random() < guidedChance ? fitting : SHAPES, random))
    if (board && !hand.some((shape) => hasSpace(board, shape))) {
      hand[Math.floor(random() * 3)] = pickShape(fitting, random)
    }
    return hand
  }

  function canPlace(board, shape, row, col) {
    if (!shape || !Number.isInteger(row) || !Number.isInteger(col)) return false
    return shape.cells.every((cell) => {
      const y = row + cell.row
      const x = col + cell.col
      return y >= 0 && y < SIZE && x >= 0 && x < SIZE && board[y][x] === 0
    })
  }

  function hasSpace(board, shape) {
    if (!shape) return false
    for (let row = 0; row <= SIZE - shape.height; row += 1) {
      for (let col = 0; col <= SIZE - shape.width; col += 1) {
        if (canPlace(board, shape, row, col)) return true
      }
    }
    return false
  }

  function canPlay(board, hand) {
    return hand.some((shape) => hasSpace(board, shape))
  }

  function placeShape(board, shape, row, col, combo = 0) {
    if (!canPlace(board, shape, row, col)) return null
    const next = board.map((line) => line.slice())
    shape.cells.forEach((cell) => { next[row + cell.row][col + cell.col] = shape.color })
    const clearedRows = [], clearedCols = []
    for (let i = 0; i < SIZE; i += 1) {
      if (next[i].every(Boolean)) clearedRows.push(i)
      if (next.every((line) => line[i] !== 0)) clearedCols.push(i)
    }
    const clearedCells = []
    for (let rowIndex = 0; rowIndex < SIZE; rowIndex += 1) {
      for (let colIndex = 0; colIndex < SIZE; colIndex += 1) {
        if (clearedRows.includes(rowIndex) || clearedCols.includes(colIndex)) {
          clearedCells.push({ row: rowIndex, col: colIndex })
          next[rowIndex][colIndex] = 0
        }
      }
    }
    const lines = clearedRows.length + clearedCols.length
    const base = lines <= 2 ? lines * 10 : 60 + (lines - 3) * 40
    const perfect = lines > 0 && next.every((line) => line.every((value) => value === 0))
    const gained = shape.cells.length +
      (lines ? Math.round(base * (1 + Math.max(0, combo - 1) * .5)) : 0) +
      (perfect ? 300 : 0)
    return { board: next, clearedRows, clearedCols, clearedCells, gained, perfect }
  }

  function createGame(random = Math.random) {
    return { board: createBoard(), hand: generateHand(random), score: 0, lines: 0,
      combo: 0, drought: 0, bestCombo: 0, over: false }
  }

  function place(state, slot, row, col, random = Math.random) {
    if (state.over || !Number.isInteger(slot) || slot < 0 || slot > 2) return { state, placed: false }
    const shape = state.hand[slot]
    if (!canPlace(state.board, shape, row, col)) return { state, placed: false }
    const placement = placeShape(state.board, shape, row, col)
    const cleared = placement.clearedRows.length + placement.clearedCols.length
    const drought = cleared ? 0 : Math.min(3, (state.drought || 0) + 1)
    const combo = cleared ? ((state.drought || 0) >= 3 ? 1 : (state.combo || 0) + 1)
      : (drought >= 3 ? 0 : (state.combo || 0))
    const result = cleared ? placeShape(state.board, shape, row, col, combo) : placement
    let hand = state.hand.slice()
    hand[slot] = null
    const replenished = hand.every((piece) => piece === null)
    if (replenished) hand = generateHand(random, result.board)
    const next = {
      board: result.board, hand,
      score: state.score + result.gained,
      lines: state.lines + cleared,
      combo, drought,
      bestCombo: Math.max(state.bestCombo || 0, combo),
      over: !canPlay(result.board, hand),
    }
    return { ...result, combo, state: next, placed: true, replenished }
  }

  function serializeGame(state) {
    return {
      board: state.board, hand: state.hand.map((shape) => shape?.id || null),
      score: state.score, lines: state.lines, combo: state.combo || 0,
      drought: state.drought || 0, bestCombo: state.bestCombo || 0,
    }
  }

  function restoreGame(data) {
    if (!data || typeof data !== 'object' ||
      !Array.isArray(data.board) || data.board.length !== SIZE ||
      !data.board.every((line) => Array.isArray(line) && line.length === SIZE &&
        line.every((value) => Number.isInteger(value) && value >= 0 && value <= 6))) return null
    if (!Array.isArray(data.hand) || data.hand.length !== 3) return null
    const hand = data.hand.map((id) =>
      id === null ? null : SHAPES.find((shape) => shape.id === id))
    if (hand.some((shape) => shape === undefined) || hand.every((shape) => shape === null)) return null
    for (const key of ['score', 'lines', 'combo', 'drought', 'bestCombo']) {
      if (!Number.isSafeInteger(data[key]) || data[key] < 0) return null
    }
    if (data.drought > 3 || data.combo > 100000 || data.bestCombo > 100000) return null
    const board = data.board.map((line) => line.slice())
    return { board, hand, score: data.score, lines: data.lines, combo: data.combo,
      drought: data.drought, bestCombo: data.bestCombo, over: !canPlay(board, hand) }
  }

  globalThis.XiabanBlockPuzzle = Object.freeze({ SIZE, SHAPES, createBoard, generateHand, canPlace, hasSpace, canPlay, placeShape, createGame, place, serializeGame, restoreGame })
})()
