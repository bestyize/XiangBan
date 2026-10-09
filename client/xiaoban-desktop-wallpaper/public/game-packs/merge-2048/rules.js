(() => {
  'use strict'

  const SIZE = 4
  const DIRECTIONS = ['left', 'right', 'up', 'down']

  function createBoard() {
    return Array.from({ length: SIZE }, () => Array(SIZE).fill(0))
  }

  function addTile(board, random = Math.random) {
    const cells = []
    board.forEach((row, y) => row.forEach((value, x) => {
      if (value === 0) cells.push({ row: y, col: x })
    }))
    const next = board.map((row) => row.slice())
    if (cells.length === 0) return { board: next, cell: null }
    const cell = cells[Math.floor(random() * cells.length)]
    next[cell.row][cell.col] = random() < 0.9 ? 2 : 4
    return { board: next, cell }
  }

  function coordinate(direction, line, index) {
    if (direction === 'right') return { row: line, col: SIZE - 1 - index }
    if (direction === 'up') return { row: index, col: line }
    if (direction === 'down') return { row: SIZE - 1 - index, col: line }
    return { row: line, col: index }
  }

  function move(board, direction) {
    if (!DIRECTIONS.includes(direction)) throw new Error('不支持的移动方向')
    const next = createBoard()
    const merges = []
    let score = 0

    for (let line = 0; line < SIZE; line += 1) {
      const values = []
      for (let index = 0; index < SIZE; index += 1) {
        const { row, col } = coordinate(direction, line, index)
        if (board[row][col] !== 0) values.push(board[row][col])
      }
      let target = 0
      for (let index = 0; index < values.length; index += 1) {
        let value = values[index]
        const cell = coordinate(direction, line, target)
        if (values[index + 1] === value) {
          value *= 2
          score += value
          merges.push(cell)
          index += 1
        }
        next[cell.row][cell.col] = value
        target += 1
      }
    }

    const moved = board.some((row, y) => row.some((value, x) => value !== next[y][x]))
    return { board: next, score, moved, merges }
  }

  function canMove(board) {
    for (let row = 0; row < SIZE; row += 1) {
      for (let col = 0; col < SIZE; col += 1) {
        const value = board[row][col]
        if (value === 0) return true
        if (col + 1 < SIZE && value === board[row][col + 1]) return true
        if (row + 1 < SIZE && value === board[row + 1][col]) return true
      }
    }
    return false
  }

  function hasWon(board) {
    return board.some((row) => row.some((value) => value >= 2048))
  }

  globalThis.Xiaban2048 = Object.freeze({ createBoard, addTile, move, canMove, hasWon })
})()
