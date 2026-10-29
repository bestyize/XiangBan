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

  function generateHand(random = Math.random) {
    return Array.from({ length: 3 }, () => SHAPES[Math.min(SHAPES.length - 1, Math.max(0, Math.floor(random() * SHAPES.length)))])
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

  function placeShape(board, shape, row, col) {
    if (!canPlace(board, shape, row, col)) return null
    const next = board.map((line) => line.slice())
    shape.cells.forEach((cell) => { next[row + cell.row][col + cell.col] = shape.color })
    // Detect every complete line before clearing; a crossing cell belongs to both.
    const clearedRows = []
    const clearedCols = []
    for (let index = 0; index < SIZE; index += 1) {
      if (next[index].every(Boolean)) clearedRows.push(index)
      if (next.every((line) => line[index] !== 0)) clearedCols.push(index)
    }
    const clearedCells = []
    for (let y = 0; y < SIZE; y += 1) {
      for (let x = 0; x < SIZE; x += 1) {
        if (clearedRows.includes(y) || clearedCols.includes(x)) {
          clearedCells.push({ row: y, col: x })
          next[y][x] = 0
        }
      }
    }
    const lines = clearedRows.length + clearedCols.length
    const gained = shape.cells.length + lines * 10 + lines * (lines - 1) * 5
    return { board: next, clearedRows, clearedCols, clearedCells, gained }
  }

  function createGame(random = Math.random) {
    return { board: createBoard(), hand: generateHand(random), score: 0, lines: 0, over: false }
  }

  function place(state, slot, row, col, random = Math.random) {
    if (state.over || !Number.isInteger(slot) || slot < 0 || slot > 2) return { state, placed: false }
    const result = placeShape(state.board, state.hand[slot], row, col)
    if (!result) return { state, placed: false }
    let hand = state.hand.slice()
    hand[slot] = null
    const replenished = hand.every((shape) => shape === null)
    if (replenished) hand = generateHand(random)
    const next = {
      board: result.board, hand,
      score: state.score + result.gained,
      lines: state.lines + result.clearedRows.length + result.clearedCols.length,
      over: !canPlay(result.board, hand),
    }
    return { ...result, state: next, placed: true, replenished }
  }

  globalThis.XiabanBlockPuzzle = Object.freeze({ SIZE, SHAPES, createBoard, generateHand, canPlace, hasSpace, canPlay, placeShape, createGame, place })
})()
