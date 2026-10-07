import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

type Board = number[][]
interface Shape {
  id: string
  name: string
  cells: { row: number; col: number }[]
  width: number
  height: number
  color: number
}
interface State {
  board: Board
  hand: (Shape | null)[]
  score: number
  lines: number
  over: boolean
}
interface Placement {
  board: Board
  clearedRows: number[]
  clearedCols: number[]
  clearedCells: { row: number; col: number }[]
  gained: number
}
interface Rules {
  SIZE: number
  SHAPES: Shape[]
  createBoard(): Board
  generateHand(random?: () => number): Shape[]
  canPlace(board: Board, shape: Shape | null, row: number, col: number): boolean
  hasSpace(board: Board, shape: Shape | null): boolean
  canPlay(board: Board, hand: (Shape | null)[]): boolean
  placeShape(board: Board, shape: Shape, row: number, col: number): Placement | null
  createGame(random?: () => number): State
  place(state: State, slot: number, row: number, col: number, random?: () => number): { state: State; placed: boolean; replenished?: boolean } & Partial<Placement>
}

// Run the same classic script shipped to the iframe, with no duplicate rules.
const context: { XiabanBlockPuzzle?: Rules } = {}
runInNewContext(readFileSync(new URL('../../public/game-packs/block-puzzle/rules.js', import.meta.url), 'utf8'), context)
const rules = context.XiabanBlockPuzzle!
const shape = (id: string) => rules.SHAPES.find((entry) => entry.id === id)!

function withHand(board: Board, hand: (Shape | null)[]): State {
  return { board, hand, score: 0, lines: 0, over: false }
}

describe('block puzzle game pack rules', () => {
  it('creates independent empty 8×8 boards and exactly three pieces', () => {
    const state = rules.createGame(() => 0)
    expect(state.board).toHaveLength(8)
    expect(state.board.every((row) => row.length === 8 && row.every((value) => value === 0))).toBe(true)
    expect(state.hand.map((entry) => entry!.id)).toEqual(['dot', 'dot', 'dot'])
    state.board[0][0] = 1
    expect(state.board[1][0]).toBe(0)
    expect(rules.createBoard()[0][0]).toBe(0)
    expect(state.over).toBe(false)
  })

  it('samples all three pieces independently, including the last shape', () => {
    const samples = [0, 0.4, 0.99999]
    const hand = rules.generateHand(() => samples.shift()!)
    expect(hand.map((entry) => entry.id)).toEqual(['dot', 'square3', 'rectV'])
    expect(samples).toEqual([])
  })

  it('every catalog shape fits an empty board and has unique occupied cells', () => {
    for (const entry of rules.SHAPES) {
      expect(rules.hasSpace(rules.createBoard(), entry)).toBe(true)
      expect(new Set(entry.cells.map((cell) => `${cell.row}:${cell.col}`)).size).toBe(entry.cells.length)
      expect(rules.canPlace(rules.createBoard(), entry, 8 - entry.height, 8 - entry.width)).toBe(true)
    }
  })

  it('places the actual shape rather than filling its bounding rectangle', () => {
    const board = rules.createBoard()
    const result = rules.placeShape(board, shape('corner1'), 1, 2)!
    expect(result.board[1].slice(2, 4)).toEqual([shape('corner1').color, 0])
    expect(result.board[2].slice(2, 4)).toEqual([shape('corner1').color, shape('corner1').color])
    expect(result.gained).toBe(3)
    expect(board.flat().every((value) => value === 0)).toBe(true)
  })

  it.each([
    { row: -1, col: 0 }, { row: 0, col: -1 }, { row: 7, col: 0 },
    { row: 0, col: 7 }, { row: 8, col: 8 }, { row: 0.5, col: 0 },
  ])('rejects out-of-bounds or fractional placement at $row,$col without mutation', ({ row, col }) => {
    const state = withHand(rules.createBoard(), [shape('square2'), shape('dot'), null])
    expect(rules.canPlace(state.board, state.hand[0], row, col)).toBe(false)
    expect(rules.place(state, 0, row, col)).toEqual({ state, placed: false })
    expect(state.board.flat().every((value) => value === 0)).toBe(true)
  })

  it('rejects overlapping cells but allows occupied gaps in a shape', () => {
    const board = rules.createBoard()
    board[1][1] = 3
    expect(rules.canPlace(board, shape('corner1'), 1, 1)).toBe(false)
    board[1][1] = 0
    board[1][2] = 3
    expect(rules.canPlace(board, shape('corner1'), 1, 1)).toBe(true)
    expect(rules.placeShape(board, shape('corner1'), 1, 1)!.board[1][2]).toBe(3)
  })

  it.each(['row', 'column'])('clears a completed $0 and awards placement plus line points', (direction) => {
    const board = rules.createBoard()
    for (let index = 0; index < 7; index += 1) {
      if (direction === 'row') board[3][index] = 2
      else board[index][3] = 2
    }
    const result = rules.placeShape(board, shape('dot'), direction === 'row' ? 3 : 7, direction === 'row' ? 7 : 3)!
    expect(result.gained).toBe(11)
    expect(result.clearedRows).toEqual(direction === 'row' ? [3] : [])
    expect(result.clearedCols).toEqual(direction === 'column' ? [3] : [])
    expect(result.clearedCells).toHaveLength(8)
    expect(result.board.flat().filter(Boolean)).toEqual([])
  })

  it('detects crossing row and column before clearing and counts the crossing cell once', () => {
    const board = rules.createBoard()
    board[2].fill(2)
    board.forEach((row) => { row[4] = 3 })
    board[2][4] = 0
    board[6][6] = 4
    const original = board.map((row) => row.slice())
    const result = rules.placeShape(board, shape('dot'), 2, 4)!
    expect(result.clearedRows).toEqual([2])
    expect(result.clearedCols).toEqual([4])
    expect(result.clearedCells).toHaveLength(15)
    expect(result.gained).toBe(31)
    expect(result.board.flat().filter(Boolean)).toEqual([4])
    expect(board).toEqual(original)
  })

  it('clears two rows and two columns in one placement, applying the multiple-line bonus', () => {
    const board = rules.createBoard()
    board[3].fill(1)
    board[4].fill(1)
    board.forEach((row) => { row[2] = 1; row[3] = 1 })
    board[3][2] = 0
    board[3][3] = 0
    board[4][2] = 0
    board[4][3] = 0
    const result = rules.placeShape(board, shape('square2'), 3, 2)!
    expect(result.clearedRows).toEqual([3, 4])
    expect(result.clearedCols).toEqual([2, 3])
    expect(result.gained).toBe(104)
    expect(result.clearedCells).toHaveLength(28)
    expect(result.board.flat().every((value) => value === 0)).toBe(true)
  })

  it('does not replace unused pieces and replenishes only after the third piece', () => {
    let randomCalls = 0
    const random = () => { randomCalls += 1; return 0 }
    const initial = withHand(rules.createBoard(), [shape('dot'), shape('dot'), shape('dot')])
    const first = rules.place(initial, 1, 0, 0, random)
    expect(first.state.hand.map((entry) => entry?.id ?? null)).toEqual(['dot', null, 'dot'])
    expect(first.replenished).toBe(false)
    const second = rules.place(first.state, 2, 0, 1, random)
    expect(randomCalls).toBe(0)
    const third = rules.place(second.state, 0, 0, 2, random)
    expect(third.replenished).toBe(true)
    expect(third.state.hand.every((entry) => entry?.id === 'dot')).toBe(true)
    expect(randomCalls).toBe(3)
    expect(third.state.score).toBe(3)
    expect(initial.hand.every(Boolean)).toBe(true)
  })

  it('keeps playing when any remaining piece fits even if a large piece cannot fit', () => {
    const board = Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row + col) % 2))
    expect(rules.hasSpace(board, shape('square2'))).toBe(false)
    expect(rules.canPlay(board, [shape('square2'), shape('dot'), null])).toBe(true)
    expect(rules.canPlay(board, [shape('square2'), null, null])).toBe(false)
  })

  it('ends when the only remaining shape cannot fit, without needing a full board', () => {
    const board = Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row + col) % 2))
    const initial = withHand(board, [shape('dot'), shape('square2'), null])
    const result = rules.place(initial, 0, 0, 0)
    expect(result.placed).toBe(true)
    expect(result.state.over).toBe(true)
    expect(result.state.board.flat().some((value) => value === 0)).toBe(true)
  })

  it('checks the freshly replenished hand for game over', () => {
    const board = Array.from({ length: 8 }, (_, row) => Array.from({ length: 8 }, (_, col) => (row + col) % 2))
    const initial = withHand(board, [shape('dot'), null, null])
    const result = rules.place(initial, 0, 0, 0, () => 0.4)
    expect(result.replenished).toBe(true)
    expect(result.state.hand.every((entry) => entry?.id === 'square3')).toBe(true)
    expect(result.state.over).toBe(true)
  })

  it('accumulates scores and line totals across moves', () => {
    const board = rules.createBoard()
    board[0].fill(1)
    board[0][7] = 0
    const initial = { ...withHand(board, [shape('dot'), shape('dot'), null]), score: 100, lines: 5 }
    const result = rules.place(initial, 0, 0, 7)
    expect(result.state.score).toBe(111)
    expect(result.state.lines).toBe(6)
    expect(initial.score).toBe(100)
    expect(initial.lines).toBe(5)
  })

  it('rejects consumed slots, invalid slots and all moves after game over without sampling', () => {
    const state = withHand(rules.createBoard(), [shape('dot'), null, null])
    const random = () => { throw new Error('should not sample') }
    for (const slot of [-1, 1, 2, 3, 0.5]) {
      expect(rules.place(state, slot, 0, 0, random)).toEqual({ state, placed: false })
    }
    const finished = { ...state, over: true }
    expect(rules.place(finished, 0, 0, 0, random)).toEqual({ state: finished, placed: false })
    expect(rules.hasSpace(state.board, null)).toBe(false)
  })
})
