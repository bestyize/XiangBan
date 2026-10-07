(() => {
  'use strict'

  const CAPACITY = 4
  // Each array reads from the bottom of a glass to its top. These are fixed,
  // verified puzzles; gameplay never needs random generation or a solver.
  const LEVELS = [
    { name: '柠檬初调', board: [[], [], [0, 1, 0, 1], [0, 1, 0, 1]] },
    { name: '三味小聚', board: [[], [0, 1, 0, 2], [2, 1, 0, 1], [1, 2], [0, 2]] },
    { name: '莓果晚风', board: [[2, 3, 0], [1, 1, 3, 2], [2, 0, 1, 3], [0], [1, 2, 3, 0], []] },
    { name: '暖灯茶室', board: [[2, 4, 3, 4], [4, 0], [2, 2, 1, 0], [], [1, 0, 3], [2, 4, 1], [0, 3, 1, 3]] },
    { name: '彩虹气泡', board: [[0, 5, 4], [2, 4, 2], [], [0, 3, 1], [4, 1, 3], [5, 5, 5, 2], [0, 0, 3, 1], [1, 2, 3, 4]] },
    { name: '晚霞茶会', board: [[0, 0, 2], [], [2, 4, 3], [5, 0, 3, 1], [5, 0, 4, 1], [5, 5, 4, 2], [3, 1], [4, 2, 1, 3]] },
    { name: '七味灵感', board: [[], [1, 5, 6, 3], [2, 6, 5], [3, 0], [4, 2, 6, 3], [0, 4, 2, 5], [6, 5, 3, 0], [1, 4, 2], [1, 1, 4, 0]] },
    { name: '今晚特调', board: [[0, 0, 3], [], [2, 0, 7, 1], [3, 3, 5, 6], [4, 5], [6, 3, 1], [0, 4, 1, 5], [7, 6, 6, 4], [2, 2, 7, 1], [5, 4, 2, 7]] },
  ]

  function cloneBoard(board) {
    return board.map((tube) => tube.slice())
  }

  function getLevel(index) {
    if (!Number.isInteger(index) || index < 0 || index >= LEVELS.length) return null
    const level = LEVELS[index]
    return { name: level.name, board: cloneBoard(level.board), capacity: CAPACITY, colorCount: new Set(level.board.flat()).size }
  }

  function getPour(board, from, to, capacity = CAPACITY) {
    if (!Array.isArray(board) || !Number.isInteger(from) || !Number.isInteger(to) ||
      from < 0 || to < 0 || from >= board.length || to >= board.length ||
      !Number.isInteger(capacity) || capacity < 1 ||
      !Array.isArray(board[from]) || !Array.isArray(board[to]) ||
      board[from].length > capacity || board[to].length > capacity) {
      return { valid: false, reason: 'invalid', amount: 0, color: null }
    }
    if (from === to) return { valid: false, reason: 'same', amount: 0, color: null }
    const source = board[from]
    const target = board[to]
    if (!source.length) return { valid: false, reason: 'empty', amount: 0, color: null }
    if (target.length === capacity) return { valid: false, reason: 'full', amount: 0, color: null }
    const color = source[source.length - 1]
    if (target.length && target[target.length - 1] !== color) {
      return { valid: false, reason: 'color', amount: 0, color: null }
    }
    let run = 1
    while (run < source.length && source[source.length - 1 - run] === color) run += 1
    return { valid: true, reason: null, amount: Math.min(run, capacity - target.length), color }
  }

  function pour(board, from, to, capacity = CAPACITY) {
    const move = getPour(board, from, to, capacity)
    const result = cloneBoard(board)
    if (move.valid) {
      result[from].splice(result[from].length - move.amount, move.amount)
      for (let i = 0; i < move.amount; i += 1) result[to].push(move.color)
    }
    return { board: result, moved: move.valid, amount: move.amount, reason: move.reason }
  }

  function completedTubes(board, capacity = CAPACITY) {
    return board.filter((tube) => tube.length === capacity && tube.every((color) => color === tube[0])).length
  }

  function isWon(board, capacity = CAPACITY) {
    return board.some((tube) => tube.length > 0) &&
      board.every((tube) => tube.length === 0 ||
        (tube.length === capacity && tube.every((color) => color === tube[0])))
  }

  function hasMove(board, capacity = CAPACITY) {
    return board.some((_, from) => board.some((__, to) => getPour(board, from, to, capacity).valid))
  }

  globalThis.XiabanWaterSort = Object.freeze({
    CAPACITY, levelCount: LEVELS.length, cloneBoard, getLevel, getPour, pour, completedTubes, isWon, hasMove,
  })
})()
