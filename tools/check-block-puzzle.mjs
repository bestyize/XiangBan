import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { runInNewContext } from 'node:vm'

const file = fileURLToPath(new URL('../assets/game-packs/block-puzzle/rules.js', import.meta.url))
const sandbox = {}
runInNewContext(readFileSync(file, 'utf8'), sandbox, { filename: file })
const rules = sandbox.XiabanBlockPuzzle
const dot = rules.SHAPES.find(shape => shape.id === 'dot')
assert.equal(rules.SIZE, 8)
assert.equal(rules.SHAPES.length, 27)
assert.ok(rules.SHAPES.some(shape => shape.id === 'diagDown'))
assert.ok(rules.SHAPES.some(shape => shape.id === 'diagUp'))

const empty = rules.createBoard()
assert.ok(rules.canPlace(empty, dot, 7, 7))
assert.equal(rules.canPlace(empty, dot, 8, 0), false)
assert.equal(rules.canPlace(empty, dot, -1, 0), false)
const occupied = rules.createBoard()
occupied[3][3] = 1
assert.equal(rules.canPlace(occupied, dot, 3, 3), false)
assert.equal(rules.placeShape(occupied, dot, 3, 3), null)
assert.equal(occupied[3][3], 1)

const crossing = rules.createBoard()
for (let index = 1; index < 8; index++) {
  crossing[0][index] = 1
  crossing[index][0] = 1
}
crossing[5][5] = 1 // Avoid the all-clear bonus.
const crossResult = rules.placeShape(crossing, dot, 0, 0, 1)
assert.equal(crossResult.clearedRows.length, 1)
assert.equal(crossResult.clearedCols.length, 1)
assert.equal(crossResult.clearedCells.length, 15)
assert.equal(crossResult.gained, 21)
assert.equal(crossResult.perfect, false)
assert.equal(crossing[0][1], 1) // Pure rule must not mutate its input.
assert.equal(crossResult.board[0][1], 0)
assert.equal(crossResult.board[5][5], 1)

const perfectBoard = rules.createBoard()
for (let index = 1; index < 8; index++) perfectBoard[0][index] = 1
const perfect = rules.placeShape(perfectBoard, dot, 0, 0, 1)
assert.equal(perfect.perfect, true)
assert.equal(perfect.gained, 311)

const almostFull = rules.createBoard()
for (let row = 0; row < 8; row++) for (let col = 0; col < 8; col++) almostFull[row][col] = 1
almostFull[7][7] = 0
const generated = rules.generateHand(() => .99, almostFull)
assert.equal(generated.length, 3)
assert.ok(generated.some(shape => rules.hasSpace(almostFull, shape)))

const running = {
  board: rules.createBoard(), hand: [dot, dot, dot],
  score: 0, lines: 0, combo: 0, drought: 0, bestCombo: 0, over: false,
}
for (let index = 1; index < 8; index++) {
  running.board[0][index] = 1
  running.board[1][index] = 1
}
let turn = rules.place(running, 0, 0, 0, () => 0)
assert.equal(turn.placed, true)
assert.equal(turn.state.combo, 1)
turn = rules.place(turn.state, 1, 2, 2, () => 0)
assert.equal(turn.state.drought, 1)
turn = rules.place(turn.state, 2, 3, 3, () => 0)
assert.equal(turn.replenished, true)
assert.equal(turn.state.drought, 2)
turn = rules.place(turn.state, 0, 1, 0, () => 0)
assert.equal(turn.state.combo, 2)
assert.ok(turn.state.bestCombo >= 2)
const revived = rules.restoreGame(rules.serializeGame(turn.state))
assert.equal(revived.combo, 2)
assert.equal(revived.score, turn.state.score)
assert.equal(revived.board[2][2], turn.state.board[2][2])
const corrupt = rules.serializeGame(turn.state)
corrupt.board = [[]]
assert.equal(rules.restoreGame(corrupt), null)
const thirdGap = { ...turn.state, hand: [dot, dot, dot], board: rules.createBoard(), combo: 2, drought: 2 }
const resetCombo = rules.place(thirdGap, 0, 4, 4, () => 0)
assert.equal(resetCombo.state.combo, 0)
assert.equal(resetCombo.state.drought, 3)
console.log('PASS block-puzzle: geometry, cross clear, perfect bonus, hand safety, combo, persistence')
