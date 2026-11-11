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
assert.equal(crossResult.removedTiles.length, 15)
assert.ok(crossResult.removedTiles.every(tile => tile.color >= 1 && tile.color <= 6))
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
// Hold the next hand constant: the guided generator no longer promises a dot.
turn = rules.place({ ...turn.state, hand: [dot, dot, dot] }, 0, 1, 0, () => 0)
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

// A non-clearing placement can earn a modest, distinct setup reward.
const readyBoard = rules.createBoard()
for (let index = 0; index < 6; index++) readyBoard[0][index] = 1
const readyState = {
  board: readyBoard, hand: [dot, dot, dot],
  score: 0, lines: 0, combo: 0, drought: 0, setupMoves: 0, bestCombo: 0, over: false,
}
const setup = rules.place(readyState, 0, 0, 6, () => 0)
assert.equal(setup.placed, true)
assert.equal(setup.newReadyLines, 1)
assert.equal(setup.buildUpBonus, 8)
assert.equal(setup.gained, 9)
assert.equal(setup.state.lines, 0)
assert.equal(setup.state.combo, 0)
assert.equal(setup.state.setupMoves, 1)
assert.equal(rules.restoreGame(rules.serializeGame(setup.state)).setupMoves, 1)
const milestone = rules.place({ ...readyState, board: rules.createBoard(), setupMoves: 2 }, 0, 0, 0, () => 0)
assert.equal(milestone.buildUpBonus, 5)
assert.equal(milestone.gained, 6)
assert.equal(milestone.state.lines, 0)
assert.equal(milestone.state.setupMoves, 3)
const oldSetupSave = rules.serializeGame(setup.state)
delete oldSetupSave.setupMoves
assert.equal(rules.restoreGame(oldSetupSave).setupMoves, 0)

// Three-piece lookahead: five + two + one can complete a row in three moves
// regardless of the presentation order. Search must not mutate the board.
const h5 = rules.SHAPES.find(shape => shape.id === 'h5')
const h2 = rules.SHAPES.find(shape => shape.id === 'h2')
const horizontal = rules.handOpportunity(rules.createBoard(), [h5, h2, dot])
assert.equal(horizontal.canFinish, true)
assert.ok(horizontal.clearedLines >= 1)
assert.equal(horizontal.firstClearStep, 3)
assert.equal(horizontal.perfect, true)
const boardBefore = JSON.stringify(crossing)
rules.handOpportunity(crossing, [dot, h2, h5])
assert.equal(JSON.stringify(crossing), boardBefore)

// A tidy row with three separate gaps gets a real finish, even when no single
// piece can clear the line on its own.
const splitRow = rules.createBoard()
for (const col of [0, 2, 5, 7]) splitRow[3][col] = 1
const finishingHand = rules.generateHand(() => .15, splitRow, 8, 0, 0, 18)
const finishingPlan = rules.handOpportunity(splitRow, finishingHand)
assert.equal(finishingPlan.canFinish, true)
assert.equal(finishingPlan.perfect, true)
assert.equal(finishingPlan.firstClearStep, 3)

// Recorded-style finish: two occupied cells divide a column into gaps of
// two and four. The third piece stays in hand when the theme cycle resets.
const splitColumn = rules.createBoard()
splitColumn[2][5] = splitColumn[3][5] = 1
const finish = rules.generateHand(() => .15, splitColumn, 8, 0, 0, 18)
const two = finish.findIndex(shape => shape.id === 'v2')
const four = finish.findIndex(shape => shape.id === 'v4')
assert.ok(two >= 0 && four >= 0)
const firstFill = rules.place({ ...rules.createGame(() => .15), board: splitColumn, hand: finish, cycleMoves: 18 }, two, 0, 5)
assert.equal(firstFill.state.cycleMoves, 19)
assert.equal(rules.restoreGame(rules.serializeGame(firstFill.state)).cycleMoves, 19)
const finishTurn = rules.place(firstFill.state, four, 4, 5)
assert.equal(finishTurn.perfect, true)
assert.equal(finishTurn.state.cycleMoves, 0)
assert.equal(finishTurn.state.hand.filter(Boolean).length, 1)

const adjacentLines = rules.createBoard()
adjacentLines[3].fill(1, 0, 6)
adjacentLines[4].fill(1, 0, 6)
const doubleFinisher = rules.generateHand(() => .15, adjacentLines, 8, 0, 0, 18)
const square = doubleFinisher.find(shape => shape.id === 'square2')
assert.equal(rules.placeShape(adjacentLines, square, 3, 6)?.perfect, true)

// Difficulty rises gradually across hundreds of deals, not sharply after 30.
assert.equal(rules.dealGuidance(1), rules.dealGuidance(24))
assert.ok(rules.dealGuidance(25) > .83)
assert.ok(rules.dealGuidance(40) > rules.dealGuidance(80))
assert.ok(rules.dealGuidance(80) > rules.dealGuidance(160))
assert.ok(rules.dealGuidance(160) > rules.dealGuidance(320))
assert.equal(rules.dealGuidance(320), rules.dealGuidance(500))
assert.ok(rules.dealGuidance(90, almostFull, 3, 2) > rules.dealGuidance(90))
assert.ok(rules.dealGuidance(90, almostFull, 3, 2) <= .94)

// A compact seed sample checks that the generator creates playable short-term
// opportunities and that every dealt shape refers to the shipped shape library.
let seed = 61230
const random = () => {
  seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0
  return seed / 4294967296
}
let opportunities = 0
for (let iteration = 0; iteration < 20; iteration += 1) {
  const hand = rules.generateHand(random, rules.createBoard(), iteration * 16 + 1)
  assert.equal(hand.length, 3)
  for (const piece of hand) assert.ok(rules.SHAPES.includes(piece))
  const evaluation = rules.handOpportunity(rules.createBoard(), hand)
  assert.equal(evaluation.canFinish, true)
  if (evaluation.clearedLines >= 1) opportunities += 1
}
assert.ok(opportunities >= 8, 'expected frequent but not guaranteed 3-step clears')

// Older versions stored no deal counter; restoration must remain backward-compatible.
const oldSave = rules.serializeGame(turn.state)
delete oldSave.deals
delete oldSave.cycleMoves
assert.equal(rules.restoreGame(oldSave).deals, 1)
assert.equal(rules.restoreGame(oldSave).cycleMoves, 0)
assert.ok(rules.restoreGame(rules.serializeGame(turn.state)).deals >= 1)

console.log('PASS block-puzzle: geometry, combo, setup rewards, 3-step lookahead and saves')

