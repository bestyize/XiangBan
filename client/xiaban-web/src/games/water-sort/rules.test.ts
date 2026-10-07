import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

type Board = number[][]
type Move = [number, number]
interface Rules {
  CAPACITY: number
  levelCount: number
  cloneBoard(board: Board): Board
  getLevel(index: number): { name: string; board: Board; capacity: number; colorCount: number } | null
  getPour(board: Board, from: number, to: number, capacity?: number): {
    valid: boolean; reason: string | null; amount: number; color: number | null
  }
  pour(board: Board, from: number, to: number, capacity?: number): {
    board: Board; moved: boolean; amount: number; reason: string | null
  }
  completedTubes(board: Board, capacity?: number): number
  isWon(board: Board, capacity?: number): boolean
  hasMove(board: Board, capacity?: number): boolean
}

// Test the exact non-module script served inside the host's sandbox.
const context: { XiabanWaterSort?: Rules } = {}
runInNewContext(readFileSync(new URL('../../../public/game-packs/water-sort/rules.js', import.meta.url), 'utf8'), context)
const rules = context.XiabanWaterSort!

// Concrete forward witnesses from the offline reverse-pour construction.
const witnesses: Move[][] = [
  [[3, 1], [2, 1], [3, 2], [2, 0], [3, 1], [2, 1], [3, 0], [2, 0]],
  [[1, 4], [1, 0], [3, 4], [2, 3], [3, 1], [0, 3], [1, 0], [1, 3], [2, 3], [0, 1], [2, 1], [3, 0], [4, 2], [4, 0]],
  [[3, 5], [4, 0], [5, 3], [0, 5], [3, 5], [0, 3], [4, 3], [1, 0], [0, 4], [2, 3], [1, 3], [2, 1], [2, 5], [5, 0], [4, 2], [4, 1]],
  [[6, 4], [6, 5], [4, 6], [2, 1], [4, 1], [6, 3], [1, 6], [5, 4], [0, 5], [1, 5], [4, 1], [0, 3], [2, 1], [5, 4], [0, 4], [0, 2], [5, 2], [6, 0]],
  [[7, 0], [3, 2], [4, 7], [7, 3], [2, 4], [7, 1], [6, 7], [4, 7], [1, 2], [1, 4], [5, 1], [1, 2], [7, 1], [0, 4], [3, 7], [6, 7], [3, 6], [0, 5], [6, 0], [7, 3]],
  [[7, 2], [4, 1], [3, 7], [1, 6], [5, 1], [1, 0], [6, 1], [7, 1], [2, 6], [2, 4], [7, 2], [4, 7], [0, 2], [4, 0], [3, 6], [5, 7], [4, 5], [7, 4], [3, 0], [3, 5], [6, 3]],
  [[6, 0], [3, 0], [4, 3], [6, 3], [5, 6], [8, 0], [2, 6], [1, 3], [1, 2], [5, 7], [5, 8], [5, 0], [4, 2], [6, 5], [2, 6], [1, 5], [7, 4], [8, 7], [4, 2], [7, 4], [7, 1], [8, 1]],
  [[6, 1], [1, 4], [8, 1], [5, 1], [6, 1], [7, 6], [2, 1], [5, 0], [3, 7], [3, 4], [7, 5], [0, 3], [9, 7], [8, 2], [2, 7], [9, 8], [9, 6], [4, 9], [6, 4], [6, 2], [5, 6], [9, 5], [2, 0], [8, 2]],
]

function replay(board: Board, moves: Move[]): Board {
  let result = board
  for (const [from, to] of moves) {
    const pour = rules.pour(result, from, to)
    expect(pour.moved, `legal pour ${from} → ${to}`).toBe(true)
    result = pour.board
  }
  return result
}

// Search independently of the construction witnesses. Equivalent glass
// permutations share a key. We do not move already finished glasses or move a
// homogeneous glass to an empty one (that only permutes identical states).
function solve(initial: Board, nodeLimit = 50_000): { moves: Move[] | null; visited: number } {
  const seen = new Set<string>()
  const maxDepth = 100
  let visited = 0
  function search(board: Board, path: Move[]): Move[] | null {
    if (rules.isWon(board)) return path
    if (visited >= nodeLimit || path.length >= maxDepth) return null
    const key = board.map((tube) => tube.join(',')).sort().join('|')
    if (seen.has(key)) return null
    seen.add(key)
    visited += 1
    const candidates: { move: Move; board: Board; priority: number }[] = []
    const firstEmpty = board.findIndex((tube) => tube.length === 0)
    for (let from = 0; from < board.length; from += 1) {
      const source = board[from]
      if (!source.length) continue
      const homogeneous = source.every((color) => color === source[0])
      if (homogeneous && source.length === rules.CAPACITY) continue
      for (let to = 0; to < board.length; to += 1) {
        if (board[to].length === 0 && (homogeneous || to !== firstEmpty)) continue
        if (!rules.getPour(board, from, to).valid) continue
        const next = rules.pour(board, from, to).board
        candidates.push({
          move: [from, to], board: next,
          priority: rules.completedTubes(next) * 10 + (board[to].length ? 3 : 0) + (next[from].length === 0 ? 2 : 0),
        })
      }
    }
    candidates.sort((a, b) => b.priority - a.priority)
    for (const candidate of candidates) {
      const answer = search(candidate.board, [...path, candidate.move])
      if (answer) return answer
    }
    return null
  }
  return { moves: search(initial, []), visited }
}

describe('water sort game pack rules', () => {
  it('pours the entire contiguous top flavor, but leaves other flavors in place', () => {
    const board = [[0, 1, 1, 1], [1], []]
    expect(rules.pour(board, 0, 1)).toEqual({ board: [[0], [1, 1, 1, 1], []], moved: true, amount: 3, reason: null })
    expect(rules.pour(board, 0, 2).board).toEqual([[0], [1], [1, 1, 1]])
  })

  it('limits a pour to remaining capacity, preserving the rest of the top run', () => {
    expect(rules.pour([[0, 0, 0, 0], [1, 2, 0]], 0, 1)).toEqual({
      board: [[0, 0, 0], [1, 2, 0, 0]], moved: true, amount: 1, reason: null,
    })
  })

  it('stops at the boundary between two flavors rather than pouring every matching unit', () => {
    expect(rules.pour([[0, 1, 0], []], 0, 1).board).toEqual([[0, 1], [0]])
  })

  it.each([
    { board: [[0], [1]], from: 0, to: 1, reason: 'color' },
    { board: [[0], [0, 0, 0, 0]], from: 0, to: 1, reason: 'full' },
    { board: [[], [0]], from: 0, to: 1, reason: 'empty' },
    { board: [[0], []], from: 0, to: 0, reason: 'same' },
    { board: [[0], []], from: -1, to: 1, reason: 'invalid' },
    { board: [[0], []], from: 0, to: 2, reason: 'invalid' },
    { board: [[0], []], from: 0.5, to: 1, reason: 'invalid' },
    { board: [[0, 0, 0, 0, 0], []], from: 0, to: 1, reason: 'invalid' },
  ])('rejects $reason without changing the board', ({ board, from, to, reason }) => {
    const result = rules.pour(board, from, to)
    expect(result).toEqual({ board, moved: false, amount: 0, reason })
    expect(result.board).not.toBe(board)
  })

  it('returns independent arrays for legal moves, illegal moves, and level loads', () => {
    const board = [[0, 1, 1], [1], []]
    const before = board.map((tube) => tube.slice())
    for (const to of [0, 1, 2]) {
      const result = rules.pour(board, 0, to)
      expect(board).toEqual(before)
      result.board.forEach((tube, index) => expect(tube).not.toBe(board[index]))
      result.board[2].push(7)
      expect(board).toEqual(before)
    }
    const level = rules.getLevel(0)!
    level.board[0].push(7)
    expect(rules.getLevel(0)!.board[0]).toEqual([])
  })

  it('conserves every flavor and never overfills a glass during a legal pour', () => {
    const board = [[0, 1, 1], [1], [2, 2], []]
    const originalUnits = board.flat().sort()
    for (let from = 0; from < board.length; from += 1) {
      for (let to = 0; to < board.length; to += 1) {
        const next = rules.pour(board, from, to).board
        expect(next.flat().sort()).toEqual(originalUnits)
        expect(next.every((tube) => tube.length <= 4)).toBe(true)
      }
    }
  })

  it('requires full homogeneous glasses to win, allowing the spare glasses to be empty', () => {
    expect(rules.isWon([[0, 0, 0, 0], [], [1, 1, 1, 1], []])).toBe(true)
    expect(rules.isWon([[0, 0], [0, 0], [1, 1, 1, 1], []])).toBe(false)
    expect(rules.isWon([[0, 0, 1, 1], [1, 1, 0, 0], []])).toBe(false)
    expect(rules.isWon([[], []])).toBe(false)
    expect(rules.completedTubes([[0, 0, 0, 0], [1, 1], [2, 1, 2, 1], []])).toBe(1)
  })

  it('detects a dead end and allows a pour into a matching non-full glass', () => {
    expect(rules.hasMove([[0, 1, 0, 1], [1, 0, 1, 0]])).toBe(false)
    expect(rules.hasMove([[0, 1], [1]])).toBe(true)
    expect(rules.hasMove([[], []])).toBe(false)
  })

  it('honors an explicit capacity and rejects invalid capacities or level indexes', () => {
    expect(rules.pour([[0, 0, 0], [0]], 0, 1, 3).board).toEqual([[0], [0, 0, 0]])
    expect(rules.getPour([[0], []], 0, 1, 0).reason).toBe('invalid')
    expect(rules.getLevel(-1)).toBeNull()
    expect(rules.getLevel(8)).toBeNull()
    expect(rules.getLevel(0.5)).toBeNull()
  })

  it('provides eight progressively larger fixed puzzles with exactly one full glass per flavor', () => {
    expect(rules.levelCount).toBe(8)
    const sizes: number[] = []
    for (let index = 0; index < rules.levelCount; index += 1) {
      const level = rules.getLevel(index)!
      sizes.push(level.colorCount)
      expect(level.capacity).toBe(4)
      expect(level.board).toHaveLength(level.colorCount + 2)
      expect(rules.isWon(level.board)).toBe(false)
      expect(level.board.every((tube) => tube.length <= 4)).toBe(true)
      for (let color = 0; color < level.colorCount; color += 1) {
        expect(level.board.flat().filter((unit) => unit === color)).toHaveLength(4)
      }
    }
    expect(sizes).toEqual([2, 3, 4, 5, 6, 6, 7, 8])
  })

  it.each(Array.from({ length: 8 }, (_, index) => index))('proves level %i is solvable by replaying its complete construction witness', (index) => {
    expect(rules.isWon(replay(rules.getLevel(index)!.board, witnesses[index]))).toBe(true)
  })

  it.each(Array.from({ length: 8 }, (_, index) => index))('independently solves level %i within the search budget', (index) => {
    const level = rules.getLevel(index)!
    const answer = solve(level.board)
    expect(answer.visited).toBeLessThan(50_000)
    expect(answer.moves).not.toBeNull()
    expect(rules.isWon(replay(level.board, answer.moves!))).toBe(true)
  })
})
