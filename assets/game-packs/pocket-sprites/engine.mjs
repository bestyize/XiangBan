export const SIZE = 8
const MAX_CASCADES = 40
const SPECIALS = new Set([null, 'row', 'col', 'bomb', 'rainbow'])

function createGeometry(size) {
  const cellCount = size * size
  const indices = Array.from({ length: cellCount }, (_, index) => index)
  const rows = Array.from({ length: size }, (_, row) => indices.slice(row * size, (row + 1) * size))
  const cols = Array.from({ length: size }, (_, col) => rows.map(row => row[col]))
  const lines = [...rows.map(cells => ({ cells, axis: 'row' })), ...cols.map(cells => ({ cells, axis: 'col' }))]
  const neighbors = indices.map(index => {
    const row = Math.floor(index / size)
    const col = index % size
    return [
      row > 0 ? index - size : -1,
      col > 0 ? index - 1 : -1,
      col < size - 1 ? index + 1 : -1,
      row < size - 1 ? index + size : -1,
    ].filter(cell => cell >= 0)
  })
  return { size, cellCount, indices, rows, cols, lines, neighbors }
}

const geometries = new Map([SIZE, 9].map(size => [size, createGeometry(size)]))

function geometryFor({ size = SIZE }) {
  const geometry = geometries.get(size)
  if (!geometry) throw new Error('The board size must be 8 or 9')
  return geometry
}

function colorOf(tile) {
  return tile?.special === 'rainbow' ? undefined : tile?.color
}

function normalTile(color) {
  return { color, special: null }
}

function nextRandom(game) {
  game.rngState = (game.rngState + 0x6D2B79F5) >>> 0
  let value = game.rngState
  value = Math.imul(value ^ (value >>> 15), value | 1)
  value ^= value + Math.imul(value ^ (value >>> 7), value | 61)
  return ((value ^ (value >>> 14)) >>> 0) / 0x100000000
}

function choose(game, values) {
  return values[Math.floor(nextRandom(game) * values.length)]
}

function isOpen(game, index) {
  return game.level.mask[index] && game.honey[index] === 0
}

function findRuns(board, { lines, size }) {
  const runs = []
  for (const { cells, axis } of lines) {
    let start = 0
    while (start < size) {
      const color = colorOf(board[cells[start]])
      let end = start + 1
      if (color !== undefined) {
        while (end < size && colorOf(board[cells[end]]) === color) end++
        if (end - start >= 3) runs.push({ cells: cells.slice(start, end), axis })
      }
      start = end
    }
  }
  return runs
}

function formsMatch(board, index, { size, rows, cols }) {
  const color = colorOf(board[index])
  if (color === undefined) return false
  const row = Math.floor(index / size)
  const col = index % size
  for (const [cells, position] of [[rows[row], col], [cols[col], row]]) {
    let first = position
    let last = position
    while (first > 0 && colorOf(board[cells[first - 1]]) === color) first--
    while (last < size - 1 && colorOf(board[cells[last + 1]]) === color) last++
    if (last - first >= 2) return true
  }
  return false
}

function exchange(board, a, b) {
  const tile = board[a]
  board[a] = board[b]
  board[b] = tile
}

function combines(first, second) {
  return first.special === 'rainbow' || second.special === 'rainbow' || Boolean(first.special && second.special)
}

function possibleMoves(board, geometry, firstOnly = false) {
  const { indices, neighbors } = geometry
  const moves = []
  for (const a of indices) {
    if (!board[a]) continue
    for (const b of neighbors[a]) {
      if (b <= a || !board[b]) continue
      let valid = combines(board[a], board[b])
      if (!valid && board[a].color !== board[b].color) {
        exchange(board, a, b)
        valid = formsMatch(board, a, geometry) || formsMatch(board, b, geometry)
        exchange(board, a, b)
      }
      if (valid) {
        moves.push([a, b])
        if (firstOnly) return moves
      }
    }
  }
  return moves
}

export function legalMoves(game) {
  return possibleMoves(game.board, geometryFor(game.level))
}

function playablePattern(game, { size, lines, neighbors }) {
  for (const { cells } of lines) {
    for (let start = 0; start <= size - 3; start++) {
      const match = cells.slice(start, start + 3)
      if (!match.every(index => isOpen(game, index))) continue
      for (const target of match) {
        const source = neighbors[target].find(index => isOpen(game, index) && !match.includes(index))
        if (source !== undefined) return { match, target, source }
      }
    }
  }
  return null
}

function playableBoard(game, geometry, pattern) {
  const { colors } = game.level
  const { size, cellCount, indices } = geometry
  for (let attempt = 0; attempt < 24; attempt++) {
    const board = Array(cellCount).fill(null)
    for (const index of indices) {
      if (!isOpen(game, index)) continue
      const allowed = colors.filter(color => {
        board[index] = normalTile(color)
        return !formsMatch(board, index, geometry)
      })
      board[index] = normalTile(choose(game, allowed))
    }
    if (possibleMoves(board, geometry, true).length) return board
  }

  // Three cells of a third color create a guaranteed swap in the checkerboard.
  // The target stays in its checkerboard color until swapped with the source.
  const board = indices.map(index => isOpen(game, index)
    ? normalTile(colors[(Math.floor(index / size) + index % size) % 2])
    : null)
  for (const index of [...pattern.match, pattern.source]) {
    if (index !== pattern.target) board[index] = normalTile(colors[2])
  }
  return board
}

function layers(values, name, mask, { cellCount, indices }) {
  if (values === undefined) return Array(cellCount).fill(0)
  if (!Array.isArray(values) || values.length !== cellCount || !indices.every(index =>
    Number.isSafeInteger(values[index]) && values[index] >= 0 && (mask[index] || values[index] === 0))) {
    throw new Error(`Invalid ${name} layout`)
  }
  return values.slice()
}

export function createGame(level, { seed = Math.floor(Math.random() * 0x100000000) } = {}) {
  const geometry = geometryFor(level)
  const { cellCount, indices } = geometry
  if (!Array.isArray(level.mask) || level.mask.length !== cellCount ||
      !indices.every(index => typeof level.mask[index] === 'boolean')) {
    throw new Error(`The board mask must contain ${cellCount} boolean values`)
  }
  const { colors, goals, moves } = level
  if (!Array.isArray(colors) || new Set(colors).size !== colors.length || colors.length < 3 ||
      ![...colors].every(color => Number.isInteger(color) && color >= 0 && color <= 5)) {
    throw new Error('The level needs at least three distinct colors from 0 to 5')
  }
  if (!Number.isSafeInteger(moves) || moves < 1) throw new Error('Invalid move limit')

  const ice = layers(level.ice, 'ice', level.mask, geometry)
  const honey = layers(level.honey, 'honey', level.mask, geometry)
  if (indices.some(index => ice[index] && honey[index])) throw new Error('Ice and honey overlap')
  const obstacleCounts = { ice: ice.filter(Boolean).length, honey: honey.filter(Boolean).length }
  if (!Array.isArray(goals) || goals.length === 0 || ![...goals].every(goal =>
    goal && Number.isSafeInteger(goal.count) && goal.count > 0 && (goal.type === 'color'
      ? colors.includes(goal.color)
      : (goal.type === 'ice' || goal.type === 'honey') && goal.count <= obstacleCounts[goal.type]))) {
    throw new Error('Invalid level goals')
  }

  const game = {
    level, board: null, ice, honey, moves, progress: goals.map(() => 0), score: 0,
    status: 'playing', rngState: seed >>> 0,
  }
  const pattern = playablePattern(game, geometry)
  if (!pattern) throw new Error('The level has no room for a legal match')
  if (level.initialBoard === undefined) {
    game.board = playableBoard(game, geometry, pattern)
    return game
  }

  if (!Array.isArray(level.initialBoard) || level.initialBoard.length !== cellCount) {
    throw new Error(`The initial board must contain ${cellCount} cells`)
  }
  game.board = indices.map(index => {
    const value = level.initialBoard[index]
    if (!isOpen(game, index) && value === null) return null
    const tile = typeof value === 'number' ? normalTile(value) : value
    if (!isOpen(game, index) || !tile || !colors.includes(tile.color) || !SPECIALS.has(tile.special)) {
      throw new Error(`Invalid initial tile at cell ${index}`)
    }
    return { ...tile }
  })
  if (findRuns(game.board, geometry).length || !possibleMoves(game.board, geometry, true).length) {
    throw new Error('The initial board must have no matches and at least one legal swap')
  }
  return game
}

export function snapshot(game) {
  return {
    board: game.board.map(tile => tile && { ...tile }),
    ice: game.ice.slice(), honey: game.honey.slice(), moves: game.moves,
    progress: game.progress.slice(), score: game.score, status: game.status, rngState: game.rngState,
  }
}

export function isWon(game) {
  return game.level.goals.every((goal, index) => game.progress[index] >= goal.count)
}

function rearrangeTiles(game, cells, tiles, geometry) {
  const inventory = Array.from({ length: 7 }, () => [])
  for (const tile of tiles) inventory[tile.special === 'rainbow' ? 6 : tile.color].push(tile)

  // Place tiles without making matches as we go. An unconstrained random
  // permutation rarely survives a full board with only three colors.
  for (let attempt = 0; attempt < 48; attempt++) {
    const remaining = inventory.map(bucket => bucket.slice())
    const board = Array(geometry.cellCount).fill(null)
    for (const cell of cells) {
      let options = []
      for (const bucket of remaining) {
        if (!bucket.length) continue
        board[cell] = bucket[0]
        if (!formsMatch(board, cell, geometry)) options.push(bucket)
      }
      if (!options.length) {
        board[cell] = null
        break
      }
      // Spend the largest buckets first so one color cannot dominate the tail.
      const largest = Math.max(...options.map(bucket => bucket.length))
      options = options.filter(bucket => bucket.length === largest)
      const bucket = choose(game, options)
      const offset = Math.floor(nextRandom(game) * bucket.length)
      const [tile] = bucket.splice(offset, 1)
      board[cell] = tile
    }
    if (cells.every(cell => board[cell]) && possibleMoves(board, geometry, true).length) return board
  }
  return null
}

export function shuffle(game) {
  const geometry = geometryFor(game.level)
  const pattern = playablePattern(game, geometry)
  if (!pattern) return false
  const cells = geometry.indices.filter(index => isOpen(game, index))
  const tiles = cells.map(index => game.board[index])
  const rearranged = tiles.every(Boolean) && rearrangeTiles(game, cells, tiles, geometry)
  if (rearranged) {
    game.board = rearranged
    return true
  }

  // Skewed color inventories can have no stable permutation. Keep the earned
  // specials and fill the remaining cells with a fresh, playable arrangement.
  const board = playableBoard(game, geometry, pattern)
  const available = cells.slice()
  for (const tile of tiles) {
    if (!tile?.special) continue
    const sameColor = available.findIndex(index => board[index].color === tile.color)
    const [cell] = available.splice(Math.max(0, sameColor), 1)
    board[cell] = { color: board[cell].color, special: tile.special }
  }
  game.board = board
  return true
}

function matchGroups(runs) {
  const groups = []
  for (const run of runs) {
    const group = { cells: new Set(run.cells), runs: [run] }
    // A run can join several previously independent groups at its crossings.
    for (let index = groups.length - 1; index >= 0; index--) {
      const other = groups[index]
      if (!run.cells.some(cell => other.cells.has(cell))) continue
      for (const cell of other.cells) group.cells.add(cell)
      group.runs.push(...other.runs)
      groups.splice(index, 1)
    }
    groups.push(group)
  }
  return groups
}

function matchSpecial(board, group, preferred) {
  const longest = group.runs.reduce((best, run) => run.cells.length > best.cells.length ? run : best)
  let special
  let candidates
  if (longest.cells.length >= 5) {
    special = 'rainbow'
    candidates = longest.cells
  } else if (group.runs.length > 1) {
    special = 'bomb'
    candidates = [...group.cells].sort((a, b) =>
      group.runs.filter(run => run.cells.includes(b)).length - group.runs.filter(run => run.cells.includes(a)).length)
  } else if (longest.cells.length === 4) {
    special = longest.axis
    candidates = longest.cells
  } else {
    return null
  }
  const center = special === 'bomb' ? candidates[0] : candidates[Math.floor(candidates.length / 2)]
  const index = [...preferred, center, ...candidates].find(cell => candidates.includes(cell) && !board[cell].special)
  return index === undefined ? null : { index, tile: { color: board[index].color, special } }
}

function addLine(affected, cells) {
  for (const index of cells ?? []) affected.add(index)
}

function addSquare(affected, row, col, radius, size) {
  for (let r = Math.max(0, row - radius); r <= Math.min(size - 1, row + radius); r++) {
    for (let c = Math.max(0, col - radius); c <= Math.min(size - 1, col + radius); c++) {
      affected.add(r * size + c)
    }
  }
}

function affectSpecial(board, affected, index, special, { size, rows, cols, indices }) {
  const row = Math.floor(index / size)
  const col = index % size
  if (special === 'row') addLine(affected, rows[row])
  if (special === 'col') addLine(affected, cols[col])
  if (special === 'bomb') addSquare(affected, row, col, 1, size)
  if (special === 'rainbow') {
    const color = board[index].color
    for (const cell of indices) {
      if (colorOf(board[cell]) === color) affected.add(cell)
    }
  }
}

function affectCombo(board, affected, effects, overrides, [a, b], { size, rows, cols, indices }) {
  const first = board[a]
  const second = board[b]
  const row = Math.floor(b / size)
  const col = b % size
  affected.add(a)
  affected.add(b)

  if (first.special === 'rainbow' || second.special === 'rainbow') {
    const rainbow = first.special === 'rainbow' ? a : b
    const partner = rainbow === a ? b : a
    const { special, color } = board[partner]
    effects.set(rainbow, 'rainbow')
    if (special === 'rainbow') {
      effects.set(partner, 'rainbow')
      for (const index of indices) affected.add(index)
      return 'rainbow-rainbow'
    }
    for (const index of indices) {
      if (colorOf(board[index]) !== color) continue
      affected.add(index)
      if (special && !board[index].special) overrides.set(index, special)
    }
    return special ? 'rainbow-special' : 'rainbow-color'
  }

  effects.set(a, first.special)
  effects.set(b, second.special)
  if (first.special === 'bomb' && second.special === 'bomb') {
    addSquare(affected, row, col, 2, size)
    return 'bomb-bomb'
  }
  if (first.special === 'bomb' || second.special === 'bomb') {
    for (let offset = -1; offset <= 1; offset++) {
      addLine(affected, rows[row + offset])
      addLine(affected, cols[col + offset])
    }
    return 'line-bomb'
  }
  addLine(affected, rows[row])
  addLine(affected, cols[col])
  return 'line-line'
}

function clearWave(game, runs, pair, preferred, chain, geometry) {
  const { board, ice, honey } = game
  const affected = new Set()
  const effects = new Map()
  const overrides = new Map()
  const created = []
  const combo = pair ? affectCombo(board, affected, effects, overrides, pair, geometry) : null
  for (const group of matchGroups(runs)) {
    for (const index of group.cells) affected.add(index)
    const special = matchSpecial(board, group, preferred)
    if (special) created.push(special)
  }
  // Newly affected cells are visited in the same wave, once per special.
  for (const index of affected) {
    const special = overrides.get(index) ?? board[index]?.special
    if (special && !effects.has(index)) {
      effects.set(index, special)
      affectSpecial(board, affected, index, special, geometry)
    }
  }

  const cleared = []
  const iceHit = []
  const honeyHits = new Set()
  const collected = Array(6).fill(0)
  let iceCleared = 0
  let honeyCleared = 0
  for (const index of affected) {
    if (honey[index]) honeyHits.add(index)
    if (!board[index]) continue
    for (const neighbor of geometry.neighbors[index]) {
      if (honey[neighbor]) honeyHits.add(neighbor)
    }
    // A tile used to create a special still counts as collected in the match.
    const color = colorOf(board[index])
    if (color !== undefined) collected[color]++
    board[index] = null
    cleared.push(index)
    if (ice[index]) {
      ice[index]--
      iceHit.push(index)
      if (!ice[index]) iceCleared++
    }
  }
  for (const index of honeyHits) {
    honey[index]--
    if (!honey[index]) honeyCleared++
  }
  for (const { index, tile } of created) board[index] = tile
  game.level.goals.forEach((goal, index) => {
    const gained = goal.type === 'color' ? collected[goal.color] : goal.type === 'ice' ? iceCleared : honeyCleared
    game.progress[index] = Math.min(goal.count, game.progress[index] + gained)
  })
  game.score += (cleared.length * 10 + (iceHit.length + honeyHits.size) * 20) * chain
  return {
    type: 'clear', chain, combo, cleared, created, activated: [...effects.keys()],
    effects: [...effects].map(([index, special]) => ({ index, special })),
    iceHit, honeyHit: [...honeyHits], state: snapshot(game),
  }
}

function fall(game, { size, cols, indices }) {
  const { board } = game
  const drops = []
  for (const cells of cols) {
    let write = size - 1
    for (let row = size - 1; row >= 0; row--) {
      const from = cells[row]
      if (!isOpen(game, from)) {
        write = row - 1
      } else if (board[from]) {
        const to = cells[write--]
        if (from !== to) {
          board[to] = board[from]
          board[from] = null
          drops.push({ from, to })
        }
      }
    }
  }
  for (const index of indices) {
    if (isOpen(game, index) && !board[index]) {
      board[index] = normalTile(choose(game, game.level.colors))
      drops.push({ from: null, to: index })
    }
  }
  return { type: 'fall', drops, state: snapshot(game) }
}

export function swap(game, a, b) {
  const events = []
  const geometry = geometryFor(game.level)
  if (game.status !== 'playing' || !Number.isInteger(a) || !Number.isInteger(b) ||
      !geometry.neighbors[a]?.includes(b) || !game.board[a] || !game.board[b]) {
    return { valid: false, events }
  }

  const pair = combines(game.board[a], game.board[b]) ? [a, b] : null
  const differentColors = game.board[a].color !== game.board[b].color
  exchange(game.board, a, b)
  const valid = Boolean(pair || (differentColors && (formsMatch(game.board, a, geometry) || formsMatch(game.board, b, geometry))))
  if (valid) game.moves--
  events.push({ type: 'swap', a, b, state: snapshot(game) })
  if (!valid) {
    exchange(game.board, a, b)
    events.push({ type: 'revert', a, b, state: snapshot(game) })
    return { valid, events }
  }

  let runs = findRuns(game.board, geometry)
  for (let chain = 1; chain <= MAX_CASCADES && (runs.length || (chain === 1 && pair)); chain++) {
    events.push(clearWave(game, runs, chain === 1 ? pair : null, chain === 1 ? [b, a] : [], chain, geometry))
    const fallEvent = fall(game, geometry)
    if (fallEvent.drops.length) events.push(fallEvent)
    runs = findRuns(game.board, geometry)
  }

  if (isWon(game)) game.status = 'won'
  else if (game.moves === 0) game.status = 'lost'
  if (runs.length || (game.status === 'playing' && !possibleMoves(game.board, geometry, true).length)) {
    if (shuffle(game)) events.push({ type: 'shuffle', state: snapshot(game) })
  }
  events.at(-1).state.status = game.status
  return { valid, events }
}
