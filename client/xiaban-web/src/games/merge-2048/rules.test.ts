import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

type Board = number[][]
type Cell = { row: number; col: number }
type Direction = 'left' | 'right' | 'up' | 'down'
interface Rules {
  createBoard(): Board
  addTile(board: Board, random?: () => number): { board: Board; cell: Cell | null }
  move(board: Board, direction: Direction): { board: Board; score: number; moved: boolean; merges: Cell[] }
  canMove(board: Board): boolean
  hasWon(board: Board): boolean
}

// Exercise the exact classic script shipped in the sandboxed game package.
const context: { Xiaban2048?: Rules } = {}
runInNewContext(readFileSync(new URL('../../../public/game-packs/merge-2048/rules.js', import.meta.url), 'utf8'), context)
const rules = context.Xiaban2048!

function withRow(row: number[]): Board {
  return [row, [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]
}

const stuckBoard = [
  [2, 4, 2, 4],
  [4, 2, 4, 2],
  [2, 4, 2, 4],
  [4, 2, 4, 2],
]

describe('2048 game pack rules', () => {
  it('creates independent rows and independent games', () => {
    const board = rules.createBoard()
    expect(board).toEqual(Array.from({ length: 4 }, () => [0, 0, 0, 0]))
    board[0][0] = 2
    expect(board[1][0]).toBe(0)
    expect(rules.createBoard()[0][0]).toBe(0)
  })

  it.each([
    { input: [2, 2, 2, 2], output: [4, 4, 0, 0], score: 8 },
    { input: [2, 2, 2, 0], output: [4, 2, 0, 0], score: 4 },
    { input: [4, 4, 8, 0], output: [8, 8, 0, 0], score: 8 },
    { input: [2, 0, 2, 4], output: [4, 4, 0, 0], score: 4 },
    { input: [0, 2, 0, 4], output: [2, 4, 0, 0], score: 0 },
  ])('merges $input once per tile with score $score', ({ input, output, score }) => {
    const result = rules.move(withRow(input), 'left')
    expect(result.board[0]).toEqual(output)
    expect(result.score).toBe(score)
    expect(result.moved).toBe(true)
  })

  it.each([
    { direction: 'left', cell: { row: 1, col: 0 } },
    { direction: 'right', cell: { row: 1, col: 3 } },
    { direction: 'up', cell: { row: 0, col: 1 } },
    { direction: 'down', cell: { row: 3, col: 1 } },
  ] satisfies { direction: Direction; cell: Cell }[])('slides and merges toward $direction', ({ direction, cell }) => {
    const board = rules.createBoard()
    board[1][1] = 2
    if (direction === 'left' || direction === 'right') board[1][2] = 2
    else board[2][1] = 2
    const result = rules.move(board, direction)
    expect(result.board[cell.row][cell.col]).toBe(4)
    expect(result.board.flat().filter(Boolean)).toEqual([4])
    expect(result.merges).toEqual([cell])
    expect(result.score).toBe(4)
  })

  it('merges from the leading edge when moving right', () => {
    const result = rules.move(withRow([2, 2, 2, 0]), 'right')
    expect(result.board[0]).toEqual([0, 0, 2, 4])
    expect(result.merges).toEqual([{ row: 0, col: 3 }])
  })

  it('reports an unchanged move without points or merges', () => {
    const board = withRow([2, 4, 8, 16])
    const result = rules.move(board, 'left')
    expect(result).toEqual({ board, score: 0, moved: false, merges: [] })
  })

  it('preserves its input and total tile value while moving', () => {
    const board = [[2, 2, 4, 0], [8, 0, 8, 16], [0, 4, 2, 2], [32, 32, 64, 64]]
    const original = board.map((row) => row.slice())
    for (const direction of ['left', 'right', 'up', 'down'] as const) {
      const result = rules.move(board, direction)
      expect(board).toEqual(original)
      expect(result.board.flat().reduce((sum, value) => sum + value, 0))
        .toBe(board.flat().reduce((sum, value) => sum + value, 0))
    }
  })

  it.each([
    { chance: 0.899, expected: 2 },
    { chance: 0.9, expected: 4 },
  ])('spawns $expected with sample $chance in an empty cell only', ({ chance, expected }) => {
    const board = stuckBoard.map((row) => row.slice())
    board[2][3] = 0
    const samples = [0.5, chance]
    const result = rules.addTile(board, () => samples.shift()!)
    expect(result.cell).toEqual({ row: 2, col: 3 })
    expect(result.board[2][3]).toBe(expected)
    expect(board[2][3]).toBe(0)
    expect(result.board.flat().filter(Boolean)).toHaveLength(16)
  })

  it('can select the last empty cell and never overwrites an occupied cell', () => {
    const board = withRow([2, 0, 0, 0])
    const result = rules.addTile(board, () => 0.999)
    expect(result.cell).toEqual({ row: 3, col: 3 })
    expect(result.board[0][0]).toBe(2)
    expect(result.board[3][3]).toBe(4)
    expect(result.board.flat().filter(Boolean)).toHaveLength(2)
  })

  it('does not spawn on a full board', () => {
    const result = rules.addTile(stuckBoard, () => { throw new Error('should not sample') })
    expect(result.cell).toBeNull()
    expect(result.board).toEqual(stuckBoard)
  })

  it('ends only when a full board has no horizontal or vertical merges', () => {
    expect(rules.canMove(stuckBoard)).toBe(false)
    const horizontal = stuckBoard.map((row) => row.slice())
    horizontal[3][3] = horizontal[3][2]
    expect(rules.canMove(horizontal)).toBe(true)
    const vertical = stuckBoard.map((row) => row.slice())
    vertical[3][3] = vertical[2][3]
    expect(rules.canMove(vertical)).toBe(true)
    const empty = stuckBoard.map((row) => row.slice())
    empty[3][3] = 0
    expect(rules.canMove(empty)).toBe(true)
  })

  it('recognizes reaching 2048 and continuing beyond it', () => {
    const before = withRow([1024, 1024, 0, 0])
    expect(rules.hasWon(before)).toBe(false)
    const result = rules.move(before, 'left')
    expect(result.score).toBe(2048)
    expect(rules.hasWon(result.board)).toBe(true)
    expect(rules.hasWon(withRow([4096, 0, 0, 0]))).toBe(true)
  })
})
