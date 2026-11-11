import assert from 'node:assert/strict'
import { createGame, isWon, legalMoves, SIZE, snapshot, swap } from '../assets/game-packs/pocket-sprites/engine.mjs'
import { LEVELS } from '../assets/game-packs/pocket-sprites/levels.mjs'

const seedCount = Number(process.env.POCKET_SPRITES_SEEDS ?? 12)
assert.ok(Number.isSafeInteger(seedCount) && seedCount > 0, 'POCKET_SPRITES_SEEDS must be a positive integer')

const specialValue = { row: 1, col: 1, bomb: 2, rainbow: 4 }
const tutorialSpecials = new Map([[3, 'row'], [8, 'bomb'], [14, 'rainbow']])
const tileColor = tile => tile?.special === 'rainbow' ? undefined : tile?.color

function validateBoard(game, label) {
  const size = game.level.size ?? SIZE
  assert.equal(game.board.length, size * size, `${label}: board size`)
  for (let index = 0; index < size * size; index++) {
    const row = Math.floor(index / size)
    const col = index % size
    const tile = game.board[index]
    assert.equal(Boolean(tile), game.level.mask[index] && game.honey[index] === 0, `${label}: cell ${index}`)
    if (!tile) continue
    assert.ok(game.level.colors.includes(tile.color), `${label}: tile color at ${index}`)
    const color = tileColor(tile)
    if (color === undefined) continue
    if (col + 2 < size) {
      assert.ok(color !== tileColor(game.board[index + 1]) || color !== tileColor(game.board[index + 2]), `${label}: horizontal match at ${index}`)
    }
    if (row + 2 < size) {
      assert.ok(color !== tileColor(game.board[index + size]) || color !== tileColor(game.board[index + 2 * size]), `${label}: vertical match at ${index}`)
    }
  }
  if (game.status === 'playing') assert.ok(legalMoves(game).length, `${label}: no playable move`)
}

function playTutorial(game, label) {
  const { tutorial } = game.level
  if (!tutorial) return
  assert.ok(typeof tutorial.text === 'string' && tutorial.text.trim().length, `${label}: missing tutorial text`)
  if (tutorial.swap === undefined) return
  assert.ok(game.level.initialBoard, `${label}: guided swap needs a fixed opening`)
  assert.ok(Array.isArray(tutorial.swap) && tutorial.swap.length === 2, `${label}: invalid tutorial swap`)
  const result = swap(game, ...tutorial.swap)
  assert.ok(result.valid, `${label}: tutorial swap is illegal`)
  const clear = result.events.find(event => event.type === 'clear')
  assert.ok(clear, `${label}: tutorial swap did not clear any tiles`)
  if (game.level.id === 1) assert.equal(clear.state.progress[0], 3, `${label}: tutorial must match three red birds`)
  const expected = tutorialSpecials.get(game.level.id)
  if (expected) {
    assert.ok(clear.created.some(({ tile }) => tile.special === expected), `${label}: tutorial must create ${expected}`)
  }
  validateBoard(game, `${label} after tutorial`)
}

function moveValue(game, trial) {
  if (trial.status === 'won') return Infinity
  const gained = trial.progress.reduce((sum, progress, index) => sum + progress - game.progress[index], 0)
  const damaged = trial.ice.reduce((sum, layers, index) => sum + game.ice[index] - layers + game.honey[index] - trial.honey[index], 0)
  const specials = trial.board.reduce((sum, tile) => sum + (specialValue[tile?.special] ?? 0), 0)
  return gained * 12 + damaged * 4 + specials * 3 + (trial.score - game.score) / 100
}

function bestNextGame(game, label) {
  let best
  let bestValue = -Infinity
  for (const move of legalMoves(game)) {
    const trial = { level: game.level, ...snapshot(game) }
    assert.ok(swap(trial, ...move).valid, `${label}: listed move is illegal`)
    const value = moveValue(game, trial)
    if (value > bestValue) {
      best = trial
      bestValue = value
    }
    if (value === Infinity) break
  }
  assert.ok(best, `${label}: no candidate move`)
  assert.equal(best.moves, game.moves - 1, `${label}: move cost`)
  return best
}

assert.ok(LEVELS.length, 'No levels')
for (const [index, level] of LEVELS.entries()) {
  assert.equal(level.id, index + 1, 'Levels must be ordered by ID')
  assert.ok(typeof level.name === 'string' && level.name.trim(), `Level ${level.id}: missing name`)
  assert.ok(level.moves >= 20 && level.moves <= 30, `Level ${level.id}: keep a 20–30 move budget`)
  assert.ok(level.goals.some(goal => goal.type === 'color'), `Level ${level.id}: missing collection objective`)
  assert.ok(level.goals.length <= 3, `Level ${level.id}: too many HUD objectives`)

  let wins = 0
  let movesToWin = 0
  let exampleLoss
  for (let seed = 0; seed < seedCount; seed++) {
    const label = `Level ${level.id}, seed ${seed}`
    let game = createGame(level, { seed })
    assert.equal(game.status, 'playing', `${label}: opening status`)
    assert.deepEqual(game.progress, level.goals.map(() => 0), `${label}: opening progress`)
    validateBoard(game, label)
    playTutorial(game, label)

    while (game.status === 'playing') {
      assert.ok(game.moves > 0, `${label}: playing with no moves left`)
      game = bestNextGame(game, label)
      validateBoard(game, label)
    }

    if (game.status === 'won') {
      assert.equal(isWon(game), true, `${label}: win with incomplete goals`)
      wins++
      movesToWin += level.moves - game.moves
    } else {
      assert.equal(game.status, 'lost', `${label}: unknown terminal status`)
      assert.equal(game.moves, 0, `${label}: loss with moves left`)
      assert.equal(isWon(game), false, `${label}: loss with completed goals`)
      const missing = level.goals.map((goal, goalIndex) => goal.count - game.progress[goalIndex])
      assert.ok(missing.every(count => count >= 0), `${label}: invalid remaining goal counts`)
      exampleLoss ??= `seed ${seed}, goals remaining [${missing.join(', ')}]`
    }
  }

  const average = wins ? `, ${(movesToWin / wins).toFixed(1)} mean moves for wins` : ''
  const loss = exampleLoss ? `, example loss (${exampleLoss})` : ''
  console.log(`Level ${String(level.id).padStart(2)} ${level.name}: ${wins}/${seedCount} greedy wins${average}${loss}`)
}
