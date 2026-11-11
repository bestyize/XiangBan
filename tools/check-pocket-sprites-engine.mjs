import assert from 'node:assert/strict'
import { createGame, isWon, legalMoves, shuffle, SIZE, snapshot, swap } from '../assets/game-packs/pocket-sprites/engine.mjs'

const boardIndices = size => Array.from({ length: size * size }, (_, index) => index)
const baselineFor = size => boardIndices(size).map(index => (Math.floor(index / size) + 2 * (index % size)) % 6)
const indices = boardIndices(SIZE)
const emptyLayers = () => indices.map(() => 0)
const normalTile = color => ({ color, special: null })
const baseline = baselineFor(SIZE)
const defaults = {
  id: 'fixture', mask: indices.map(() => true), colors: [0, 1, 2, 3, 4, 5], moves: 16,
  goals: [{ type: 'color', color: 0, count: 200 }],
}

function fixture(overrides, options = {}, seed = 13) {
  const level = { ...defaults, ...options }
  level.initialBoard = baselineFor(level.size ?? SIZE).map((color, index) => level.mask[index] && !level.honey?.[index]
    ? overrides[index] ?? color
    : null)
  return createGame(level, { seed })
}

function clearOf(result) {
  assert.equal(result.valid, true)
  return result.events.find(event => event.type === 'clear')
}

function assertStable(game) {
  const { board, honey, level, status } = game
  const size = level.size ?? SIZE
  const color = tile => tile?.special === 'rainbow' ? undefined : tile?.color
  assert.equal(board.length, size * size)
  for (const index of boardIndices(size)) {
    assert.equal(Boolean(board[index]), level.mask[index] && !honey[index], `occupied cell ${index}`)
    if (color(board[index]) === undefined) continue
    const row = Math.floor(index / size)
    const col = index % size
    if (col < size - 2) assert.ok(color(board[index]) !== color(board[index + 1]) || color(board[index]) !== color(board[index + 2]))
    if (row < size - 2) assert.ok(color(board[index]) !== color(board[index + size]) || color(board[index]) !== color(board[index + 2 * size]))
  }
  if (status === 'playing') assert.ok(legalMoves(game).length)
}

const firstMatch = { 0: 0, 1: 0, 2: 1, 10: 0 }
const initial = fixture(firstMatch)
const before = snapshot(initial)
assert.deepEqual(swap(initial, 7, 8), { valid: false, events: [] })
const invalid = swap(initial, 6, 7)
assert.equal(invalid.valid, false)
assert.deepEqual(invalid.events.map(event => event.type), ['swap', 'revert'])
assert.equal(invalid.events[0].state.board[6].color, before.board[7].color)
assert.deepEqual(snapshot(initial), before)

for (const [indicesToMatch, target, source, special] of [
  [[0, 1, 2, 3], 2, 10, 'row'],
  [[0, 8, 16, 24], 16, 17, 'col'],
  [[0, 1, 2, 3, 4], 2, 10, 'rainbow'],
  [[2, 10, 17, 18, 19], 18, 26, 'bomb'],
  [[2, 10, 18, 19, 20], 18, 26, 'bomb'],
]) {
  const layout = new Set([...indicesToMatch, source])
  const initialTiles = Object.fromEntries([...layout].map(index => [index, index === target ? 1 : 0]))
  const game = fixture(initialTiles, { mask: indices.map(index => layout.has(index)) })
  assert.ok(legalMoves(game).some(([a, b]) => a === Math.min(target, source) && b === Math.max(target, source)))
  const result = swap(game, source, target)
  const first = clearOf(result)
  assert.deepEqual(first.created, [{ index: target, tile: { color: 0, special } }])
  assert.deepEqual(new Set(first.cleared), new Set(indicesToMatch))
  assert.equal(first.state.moves, defaults.moves - 1)
  // The tile turned into a special still contributes to the color goal.
  assert.equal(first.state.progress[0], indicesToMatch.length)
  assertStable(game)
}

const a = 27
const b = 28
for (const [first, second, combo, count] of [
  ['row', 'col', 'line-line', 15],
  ['col', 'bomb', 'line-bomb', 39],
  ['bomb', 'bomb', 'bomb-bomb', 25],
  ['rainbow', 'rainbow', 'rainbow-rainbow', 64],
  ['rainbow', null, 'rainbow-color', baseline.filter(color => color === baseline[b]).length + 1],
]) {
  const game = fixture({ [a]: { color: baseline[a], special: first }, [b]: { color: baseline[b], special: second } })
  const firstWave = clearOf(swap(game, a, b))
  assert.equal(firstWave.combo, combo)
  assert.equal(firstWave.cleared.length, count)
  assert.equal(new Set(firstWave.activated).size, second ? 2 : 1)
  assertStable(game)
}

const rainbowSpecial = fixture({ [a]: { color: baseline[a], special: 'rainbow' }, [b]: { color: baseline[b], special: 'row' } })
const rainbowWave = clearOf(swap(rainbowSpecial, a, b))
assert.equal(rainbowWave.combo, 'rainbow-special')
assert.equal(rainbowWave.effects.filter(effect => effect.special === 'row').length, baseline.filter(color => color === baseline[b]).length)

const chainedSpecial = fixture({ 24: { color: baseline[24], special: 'col' }, [a]: { color: baseline[a], special: 'row' }, [b]: { color: baseline[b], special: 'row' } })
const chainedWave = clearOf(swap(chainedSpecial, a, b))
assert.deepEqual(new Set(chainedWave.activated), new Set([24, a, b]))
assert.equal(chainedWave.cleared.length, 22)

const overlapIce = emptyLayers()
overlapIce[10] = 2
const overlap = fixture({
  0: 0, 1: 0, 2: { color: 1, special: 'row' },
  8: 1, 9: 1, 10: { color: 0, special: 'col' },
}, { ice: overlapIce, goals: [{ type: 'ice', count: 1 }] })
const overlapWave = clearOf(swap(overlap, 10, 2))
assert.equal(overlapWave.combo, 'line-line')
assert.ok([8, 9, 11].every(index => overlapWave.cleared.includes(index)))
assert.deepEqual(overlapWave.iceHit, [10])
assert.equal(overlapWave.state.ice[10], 1)
assert.equal(overlapWave.state.progress[0], 0)

const ice = emptyLayers()
const honey = emptyLayers()
ice[25] = 2
honey[26] = 2
const goals = [{ type: 'ice', count: 1 }, { type: 'honey', count: 1 }]
const doubleRainbow = { [a]: { color: baseline[a], special: 'rainbow' }, [b]: { color: baseline[b], special: 'rainbow' } }
const layered = fixture(doubleRainbow, { ice, honey, goals })
const layeredWave = clearOf(swap(layered, a, b))
assert.deepEqual(layeredWave.iceHit, [25])
assert.deepEqual(layeredWave.honeyHit, [26])
assert.equal(layeredWave.state.ice[25], 1)
assert.equal(layeredWave.state.honey[26], 1)
assert.deepEqual(layeredWave.state.progress, [0, 0])
assert.equal(layeredWave.state.board[26], null)
assert.equal(ice[25], 2)
assert.equal(honey[26], 2)

ice[25] = honey[26] = 1
const finalMove = fixture(doubleRainbow, { ice, honey, goals, moves: 1 })
const won = swap(finalMove, a, b)
assert.equal(won.events.at(-1).state.status, 'won')
assert.equal(finalMove.moves, 0)
assert.equal(isWon(finalMove), true)
assert.deepEqual(finalMove.progress, [1, 1])
assert.equal(finalMove.status, 'won')
assert.equal(swap(finalMove, a, b).valid, false)

const missing = new Set([26])
const gravityHoney = emptyLayers()
gravityHoney[28] = 1
const segmented = fixture({ 40: { color: baseline[40], special: 'row' }, 41: { color: baseline[41], special: 'row' } }, {
  mask: indices.map(index => !missing.has(index)), honey: gravityHoney,
})
const segmentTurn = swap(segmented, 40, 41)
const segmentFall = segmentTurn.events.find(event => event.type === 'fall')
assert.ok(segmentFall.drops.some(drop => drop.from === 34 && drop.to === 42))
assert.ok(segmentFall.drops.some(drop => drop.from === 36 && drop.to === 44))
assert.ok(segmentFall.drops.some(drop => drop.from === null && drop.to === 34))
assert.ok(segmentFall.drops.some(drop => drop.from === null && drop.to === 36))
assert.equal(segmentFall.state.board[26], null)
assert.equal(segmentFall.state.board[28], null)

const dead = createGame(defaults, { seed: 5 })
dead.board = indices.map(index => normalTile((Math.floor(index / SIZE) * 2 + index % SIZE) % 3))
assert.deepEqual(legalMoves(dead), [])
const inventory = game => game.board.filter(Boolean).map(tile => `${tile.color}:${tile.special}`).sort()
const beforeShuffle = inventory(dead)
assert.equal(shuffle(dead), true)
assert.deepEqual(inventory(dead), beforeShuffle)
assertStable(dead)

const skewed = fixture(firstMatch, { mask: indices.map(index => index !== 27), ice, honey })
skewed.board = skewed.board.map(tile => tile && normalTile(0))
skewed.board[0].special = 'row'
assert.equal(shuffle(skewed), true)
assert.equal(skewed.board.filter(tile => tile?.special === 'row').length, 1)
assert.deepEqual(skewed.ice, ice)
assert.deepEqual(skewed.honey, honey)
assertStable(skewed)

const narrow = { mask: indices.map(index => index < 4) }
let sawAutomaticShuffle = false
for (let seed = 0; seed < 16; seed++) {
  const game = fixture({ 0: 0, 1: 0, 2: 1, 3: 0 }, narrow, seed)
  const result = swap(game, 2, 3)
  if (result.events.some(event => event.type === 'shuffle')) sawAutomaticShuffle = true
  assert.equal(game.moves, defaults.moves - 1)
  assertStable(game)
}
assert.ok(sawAutomaticShuffle)

const lost = fixture({ 0: 0, 1: 0, 2: 1, 3: 0 }, { ...narrow, moves: 1 })
assert.equal(swap(lost, 2, 3).events.at(-1).state.status, 'lost')
assert.equal(lost.status, 'lost')

const endless = fixture({ 0: 0, 1: 0, 2: 1, 3: 0 }, narrow)
let forcedPicks = 0
// Three identical refill tiles force another match through the cascade limit.
Object.defineProperty(endless.level, 'colors', { get: () => forcedPicks++ < 120 ? [0] : defaults.colors })
const cappedTurn = swap(endless, 2, 3)
assert.equal(cappedTurn.events.filter(event => event.type === 'clear').length, 40)
assert.equal(cappedTurn.events.at(-1).type, 'shuffle')
assertStable(endless)

assert.throws(() => createGame({ ...defaults, mask: indices.map(index => index < 3) }), /legal match/)
assert.throws(() => fixture({ 0: 0, 1: 0, 2: 0, 3: 1 }, narrow), /initial board/)

let sawCascade = false
for (let seed = 0; seed < 40; seed++) {
  const game = createGame({ ...defaults, moves: 5, mask: indices.map(index => ![0, 7, 56, 63].includes(index)), ice, honey }, { seed })
  assertStable(game)
  const restored = Object.assign(createGame(game.level, { seed: 999 }), JSON.parse(JSON.stringify(snapshot(game))))
  const next = legalMoves(game)[0]
  const firstTurn = swap(game, ...next)
  assert.deepEqual(firstTurn, swap(restored, ...next))
  assert.deepEqual(snapshot(game), snapshot(restored))
  assertStable(game)
  while (game.status === 'playing') {
    const turn = swap(game, ...legalMoves(game)[0])
    const waves = turn.events.filter(event => event.type === 'clear')
    assert.deepEqual(waves.map(wave => wave.chain), waves.map((_, index) => index + 1))
    sawCascade ||= waves.length > 1
    assertStable(game)
  }
}
assert.ok(sawCascade)

const wideSize = 9
const wideIndices = boardIndices(wideSize)
const wideBaseline = baselineFor(wideSize)
const wide = { size: wideSize, mask: wideIndices.map(() => true) }
const wideTile = (index, special) => ({ color: wideBaseline[index], special })
const eightGame = createGame(defaults, { seed: 4 })
const eightMoves = legalMoves(eightGame)

const wideBoundary = fixture({ 7: wideTile(7, 'row'), 8: wideTile(8, 'col') }, wide)
assert.deepEqual(swap(wideBoundary, 8, 9), { valid: false, events: [] })
assert.deepEqual(swap(wideBoundary, 80, 81), { valid: false, events: [] })
assert.ok(legalMoves(wideBoundary).some(([a, b]) => a === 7 && b === 8))
const firstRowCross = clearOf(swap(wideBoundary, 7, 8))
assert.deepEqual(new Set(firstRowCross.cleared), new Set(wideIndices.filter(index => index < 9 || index % 9 === 8)))
assertStable(wideBoundary)

for (const [line, source, target, special] of [
  [[77, 78, 79, 80], 70, 79, 'row'],
  [[53, 62, 71, 80], 70, 71, 'col'],
]) {
  const active = [...line, source]
  const tiles = Object.fromEntries(active.map(index => [index, index === target ? 1 : 0]))
  const game = fixture(tiles, { ...wide, mask: wideIndices.map(index => active.includes(index)) })
  const wave = clearOf(swap(game, source, target))
  assert.deepEqual(wave.created, [{ index: target, tile: { color: 0, special } }])
  assert.deepEqual(new Set(wave.cleared), new Set(line))
  assertStable(game)
}

const rowRocket = fixture({ 74: 0, 75: { color: 0, special: 'row' }, 76: 1, 67: 0 }, wide)
const rowWave = clearOf(swap(rowRocket, 67, 76))
assert.deepEqual(rowWave.activated, [75])
assert.deepEqual(new Set(rowWave.cleared), new Set(wideIndices.slice(72)))

const colRocket = fixture({ 62: 0, 71: { color: 0, special: 'col' }, 80: 1, 79: 0 }, wide)
const colWave = clearOf(swap(colRocket, 79, 80))
assert.deepEqual(colWave.activated, [71])
assert.deepEqual(new Set(colWave.cleared), new Set(wideIndices.filter(index => index % 9 === 8)))

for (const [first, second, combo, affected] of [
  ['bomb', 'bomb', 'bomb-bomb', index => Math.floor(index / 9) >= 6 && index % 9 >= 6],
  ['row', 'bomb', 'line-bomb', index => Math.floor(index / 9) >= 7 || index % 9 >= 7],
  ['rainbow', 'rainbow', 'rainbow-rainbow', () => true],
]) {
  const game = fixture({ 79: wideTile(79, first), 80: wideTile(80, second) }, wide)
  const wave = clearOf(swap(game, 79, 80))
  assert.equal(wave.combo, combo)
  assert.deepEqual(new Set(wave.cleared), new Set(wideIndices.filter(affected)))
  assertStable(game)
}

const wideHoney = wideIndices.map(index => Number(index === 62))
const wideGravity = fixture({ 72: wideTile(72, 'row'), 73: wideTile(73, 'row') }, {
  ...wide, mask: wideIndices.map(index => index !== 61), honey: wideHoney,
})
const wideRestored = { level: wideGravity.level, ...JSON.parse(JSON.stringify(snapshot(wideGravity))) }
const wideGravityTurn = swap(wideGravity, 73, 72)
assert.deepEqual(wideGravityTurn, swap(wideRestored, 73, 72))
const wideFall = wideGravityTurn.events.find(event => event.type === 'fall')
for (const [from, to] of [[70, 79], [71, 80], [null, 70], [null, 71]]) {
  assert.ok(wideFall.drops.some(drop => drop.from === from && drop.to === to))
}
assert.equal(wideFall.state.board[61], null)
assert.equal(wideFall.state.board[62], null)
assert.equal(wideFall.state.honey[62], 1)
assertStable(wideGravity)

assert.deepEqual(legalMoves(eightGame), eightMoves)
assert.equal(swap(eightGame, ...eightMoves[0]).valid, true)
assertStable(eightGame)
assert.throws(() => createGame({ ...defaults, size: 7 }), /size must be 8 or 9/)
assert.throws(() => createGame({ ...defaults, size: 9 }), /81 boolean values/)

console.log('Pocket Sprites engine rules passed')
