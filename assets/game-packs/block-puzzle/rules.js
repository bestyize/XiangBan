(() => {
  'use strict'

  const SIZE = 8
  const patterns = [
    ['dot', '小方块', [[0, 0]]],
    ['h2', '横向两格', [[0, 0], [0, 1]]],
    ['v2', '竖向两格', [[0, 0], [1, 0]]],
    ['h3', '横向三格', [[0, 0], [0, 1], [0, 2]]],
    ['v3', '竖向三格', [[0, 0], [1, 0], [2, 0]]],
    ['h4', '横向四格', [[0, 0], [0, 1], [0, 2], [0, 3]]],
    ['v4', '竖向四格', [[0, 0], [1, 0], [2, 0], [3, 0]]],
    ['h5', '横向五格', [[0, 0], [0, 1], [0, 2], [0, 3], [0, 4]]],
    ['v5', '竖向五格', [[0, 0], [1, 0], [2, 0], [3, 0], [4, 0]]],
    ['square2', '四格正方形', [[0, 0], [0, 1], [1, 0], [1, 1]]],
    ['square3', '九格正方形', [[0, 0], [0, 1], [0, 2], [1, 0], [1, 1], [1, 2], [2, 0], [2, 1], [2, 2]]],
    ['corner1', '小折角', [[0, 0], [1, 0], [1, 1]]],
    ['corner2', '小折角', [[0, 1], [1, 0], [1, 1]]],
    ['corner3', '小折角', [[0, 0], [0, 1], [1, 0]]],
    ['corner4', '小折角', [[0, 0], [0, 1], [1, 1]]],
    ['l1', '长折角', [[0, 0], [1, 0], [2, 0], [2, 1]]],
    ['l2', '长折角', [[0, 1], [1, 1], [2, 0], [2, 1]]],
    ['l3', '长折角', [[0, 0], [0, 1], [1, 0], [2, 0]]],
    ['l4', '长折角', [[0, 0], [0, 1], [1, 1], [2, 1]]],
    ['t1', 'T形方块', [[0, 0], [0, 1], [0, 2], [1, 1]]],
    ['t2', 'T形方块', [[0, 1], [1, 0], [1, 1], [1, 2]]],
    ['z1', '阶梯方块', [[0, 0], [0, 1], [1, 1], [1, 2]]],
    ['z2', '阶梯方块', [[0, 1], [0, 2], [1, 0], [1, 1]]],
    ['rectH', '横向六格', [[0, 0], [0, 1], [0, 2], [1, 0], [1, 1], [1, 2]]],
    ['rectV', '竖向六格', [[0, 0], [0, 1], [1, 0], [1, 1], [2, 0], [2, 1]]],
    ['diagDown', '右下斜线', [[0, 0], [1, 1], [2, 2]]],
    ['diagUp', '左下斜线', [[0, 2], [1, 1], [2, 0]]],
  ]
  const SHAPES = Object.freeze(patterns.map(([id, name, cells], index) => Object.freeze({
    id, name,
    cells: Object.freeze(cells.map(([row, col]) => Object.freeze({ row, col }))),
    width: Math.max(...cells.map((cell) => cell[1])) + 1,
    height: Math.max(...cells.map((cell) => cell[0])) + 1,
    color: index % 6 + 1,
  })))

  function createBoard() {
    return Array.from({ length: SIZE }, () => Array(SIZE).fill(0))
  }

  function pickShape(pool, random) {
    return pool[Math.max(0, Math.min(pool.length - 1, Math.floor(random() * pool.length)))]
  }

  // The three-piece deal is the decision unit. Plan all three pieces together,
  // including all play orders, with a tightly bounded search suitable for WebView.
  // This is our own tuning model, not a claimed copy of any other game's RNG.
  const BITS = Array.from({ length: 256 }, (_, value) => {
    let count = 0
    for (let n = value; n; n &= n - 1) count += 1
    return count
  })
  const DEAL_CANDIDATES = 12
  const BEAM_WIDTH = 4
  const LINE_RECIPES = [
    ['h5', 'h2', 'dot'], ['h4', 'h3', 'dot'],
    ['h3', 'h3', 'h2'], ['h4', 'h2', 'h2'],
    ['v5', 'v2', 'dot'], ['v4', 'v3', 'dot'],
    ['v3', 'v3', 'v2'], ['v4', 'v2', 'v2'],
  ]
  const BY_ID = new Map(SHAPES.map((shape) => [shape.id, shape]))
  const MOVES = new Map(SHAPES.map((shape) => {
    const positions = []
    for (let row = 0; row <= SIZE - shape.height; row += 1) {
      for (let col = 0; col <= SIZE - shape.width; col += 1) {
        const masks = new Array(SIZE).fill(0)
        shape.cells.forEach((cell) => { masks[row + cell.row] |= 1 << (col + cell.col) })
        positions.push(masks.map((bits, index) => bits ? [index, bits] : null).filter(Boolean))
      }
    }
    return [shape.id, positions]
  }))

  function maskBoard(board) {
    return board.map((row) => row.reduce((bits, tile, col) => bits | (tile ? (1 << col) : 0), 0))
  }
  // Pack 8 four-bit column counters into one number. No carry occurs because
  // a column contains at most eight tiles; this avoids 64 inner-loop branches
  // on every simulated move and improves slower Android WebViews.
  const SPREAD = Array.from({ length: 256 }, (_, bits) => {
    let packed = 0
    for (let col = 0; col < SIZE; col += 1) if (bits & (1 << col)) packed += 2 ** (4 * col)
    return packed
  })
  function nearLineScore(rows) {
    let packed = 0, value = 0
    for (const bits of rows) {
      const count = BITS[bits]
      packed += SPREAD[bits]
      value += count * count
    }
    for (let col = 0; col < SIZE; col += 1) {
      const count = (packed >>> (col * 4)) & 15
      value += count * count
    }
    return value
  }
  function tryMaskPlace(rows, position) {
    for (const [row, bits] of position) if (rows[row] & bits) return null
    const after = rows.slice()
    for (const [row, bits] of position) after[row] |= bits
    let fullColumns = 255, fullRows = 0
    for (let row = 0; row < SIZE; row += 1) {
      fullColumns &= after[row]
      if (after[row] === 255) fullRows |= 1 << row
    }
    for (let row = 0; row < SIZE; row += 1) {
      after[row] = fullRows & (1 << row) ? 0 : after[row] & ~fullColumns
    }
    return { rows: after, lines: BITS[fullRows] + BITS[fullColumns] }
  }
  function handOpportunity(board, hand, beamWidth = BEAM_WIDTH) {
    const initial = { rows: maskBoard(board), used: 0, lines: 0, firstClearStep: 0, depth: 0, rank: 0 }
    let beam = [initial], best = initial
    for (let depth = 1; depth <= 3; depth += 1) {
      const expanded = []
      for (const current of beam) {
        for (let slot = 0; slot < 3; slot += 1) {
          if (current.used & (1 << slot)) continue
          for (const position of MOVES.get(hand[slot].id)) {
            const landed = tryMaskPlace(current.rows, position)
            if (!landed) continue
            const lines = current.lines + landed.lines
            const firstClearStep = current.firstClearStep || (landed.lines ? depth : 0)
            expanded.push({
              rows: landed.rows, used: current.used | (1 << slot),
              lines, firstClearStep, depth,
              rank: lines * 750 + nearLineScore(landed.rows) * 1.7
                + (firstClearStep >= 2 ? 6 : 0),
            })
          }
        }
      }
      if (!expanded.length) break
      expanded.sort((a, b) => b.rank - a.rank)
      const seen = new Set(), next = []
      for (const candidate of expanded) {
        const key = candidate.used + '/' + candidate.rows.join(',')
        if (seen.has(key)) continue
        seen.add(key)
        next.push(candidate)
        if (next.length >= beamWidth) break
      }
      beam = next
      if (beam[0]) best = beam[0]
    }
    return { deepest: best.depth, canFinish: best.depth === 3,
      clearedLines: best.lines, firstClearStep: best.firstClearStep }
  }

  // A long, gradual progression avoids the sharp difficulty spike formerly
  // introduced at deal 30. Crowded boards and sustained combo streaks receive
  // a small recovery bonus so difficult boards can still yield satisfying clears.
  // These are selection probabilities, not observed player success rates.
  function dealGuidance(deals, board = null, combo = 0, drought = 0) {
    // Continuous interpolation avoids sudden difficulty changes at round 25/81.
    const phases = [[1, .84], [24, .84], [80, .72], [160, .61], [320, .52]]
    let base = phases[phases.length - 1][1]
    for (let i = 1; i < phases.length; i += 1) {
      if (deals <= phases[i][0]) {
        const [fromDeal, fromChance] = phases[i - 1]
        const [toDeal, toChance] = phases[i]
        const progress = Math.max(0, (deals - fromDeal) / (toDeal - fromDeal))
        base = fromChance + (toChance - fromChance) * progress
        break
      }
    }
    if (board) {
      const empty = board.flat().filter((value) => value === 0).length
      if (empty < 32) base += .12
      else if (empty < 42) base += .06
    }
    if (combo >= 2) base += .04
    if (drought >= 2) base += .06
    return Math.min(.94, base)
  }
  function generateHand(random = Math.random, board = null, deals = 1, combo = 0, drought = 0) {
    const field = board || createBoard()
    const fitting = SHAPES.filter((shape) => hasSpace(field, shape))
    if (!fitting.length) return Array.from({ length: 3 }, () => pickShape(SHAPES, random))
    const empty = field.flat().filter((value) => !value).length
    const guidedChance = Math.min(0.88, Math.max(0.18, 1 - empty / 70))
    const wantOpportunity = random() < dealGuidance(deals, field, combo, drought)
    let best = null, bestGrade = -Infinity
    for (let attempt = 0; attempt < DEAL_CANDIDATES; attempt += 1) {
      let hand
      // Seed a few natural straight-line combinations among ordinary draws.
      // They still have to pass the actual board search to be preferred.
      if (attempt % 5 === 0) {
        hand = LINE_RECIPES[Math.floor((attempt / 5 + random() * LINE_RECIPES.length) % LINE_RECIPES.length)]
          .map((id) => BY_ID.get(id))
        hand = hand.slice()
        for (let index = hand.length - 1; index > 0; index -= 1) {
          const swap = Math.floor(random() * (index + 1))
          ;[hand[swap], hand[index]] = [hand[index], hand[swap]]
        }
      } else {
        hand = Array.from({ length: 3 }, () =>
          pickShape(random() < guidedChance ? fitting : SHAPES, random))
      }
      if (!hand.some((shape) => hasSpace(field, shape))) {
        hand[Math.floor(random() * 3)] = pickShape(fitting, random)
      }
      const result = handOpportunity(field, hand)
      const opportunity = result.canFinish && result.clearedLines > 0
      // A fully playable set wins over gratuitous impossible second/third moves.
      // A natural puzzle can still be difficult: no hint is shown to the user.
      const grade = result.deepest * 120 +
        (wantOpportunity ? (opportunity ? 400 : 0) + Math.min(3, result.clearedLines) * 14
          : (opportunity ? -100 : 100) - result.clearedLines * 12) +
        (opportunity && result.firstClearStep >= 2 ? 12 : 0) +
        random() * 16
      if (grade > bestGrade) { best = hand; bestGrade = grade }
    }
    return best || Array.from({ length: 3 }, () => pickShape(fitting, random))
  }

  function canPlace(board, shape, row, col) {
    if (!shape || !Number.isInteger(row) || !Number.isInteger(col)) return false
    return shape.cells.every((cell) => {
      const y = row + cell.row
      const x = col + cell.col
      return y >= 0 && y < SIZE && x >= 0 && x < SIZE && board[y][x] === 0
    })
  }

  function hasSpace(board, shape) {
    if (!shape) return false
    for (let row = 0; row <= SIZE - shape.height; row += 1) {
      for (let col = 0; col <= SIZE - shape.width; col += 1) {
        if (canPlace(board, shape, row, col)) return true
      }
    }
    return false
  }

  function canPlay(board, hand) {
    return hand.some((shape) => hasSpace(board, shape))
  }

  function placeShape(board, shape, row, col, combo = 0) {
    if (!canPlace(board, shape, row, col)) return null
    const next = board.map((line) => line.slice())
    shape.cells.forEach((cell) => { next[row + cell.row][col + cell.col] = shape.color })
    const clearedRows = [], clearedCols = []
    for (let i = 0; i < SIZE; i += 1) {
      if (next[i].every(Boolean)) clearedRows.push(i)
      if (next.every((line) => line[i] !== 0)) clearedCols.push(i)
    }
    const clearedCells = []
    for (let rowIndex = 0; rowIndex < SIZE; rowIndex += 1) {
      for (let colIndex = 0; colIndex < SIZE; colIndex += 1) {
        if (clearedRows.includes(rowIndex) || clearedCols.includes(colIndex)) {
          clearedCells.push({ row: rowIndex, col: colIndex })
          next[rowIndex][colIndex] = 0
        }
      }
    }
    const lines = clearedRows.length + clearedCols.length
    const base = lines <= 2 ? lines * 10 : 60 + (lines - 3) * 40
    const perfect = lines > 0 && next.every((line) => line.every((value) => value === 0))
    const gained = shape.cells.length +
      (lines ? Math.round(base * (1 + Math.max(0, combo - 1) * .5)) : 0) +
      (perfect ? 300 : 0)
    return { board: next, clearedRows, clearedCols, clearedCells, gained, perfect }
  }

  function createGame(random = Math.random) {
    return { board: createBoard(), hand: generateHand(random, createBoard(), 1), score: 0, lines: 0, deals: 1,
      combo: 0, drought: 0, bestCombo: 0, over: false }
  }

  function place(state, slot, row, col, random = Math.random) {
    if (state.over || !Number.isInteger(slot) || slot < 0 || slot > 2) return { state, placed: false }
    const shape = state.hand[slot]
    if (!canPlace(state.board, shape, row, col)) return { state, placed: false }
    const placement = placeShape(state.board, shape, row, col)
    const cleared = placement.clearedRows.length + placement.clearedCols.length
    const drought = cleared ? 0 : Math.min(3, (state.drought || 0) + 1)
    const combo = cleared ? ((state.drought || 0) >= 3 ? 1 : (state.combo || 0) + 1)
      : (drought >= 3 ? 0 : (state.combo || 0))
    const result = cleared ? placeShape(state.board, shape, row, col, combo) : placement
    let hand = state.hand.slice()
    hand[slot] = null
    const replenished = hand.every((piece) => piece === null)
    const deals = (state.deals || 1) + (replenished ? 1 : 0)
    if (replenished) hand = generateHand(random, result.board, deals, combo, drought)
    const next = {
      board: result.board, hand,
      score: state.score + result.gained,
      lines: state.lines + cleared, deals,
      combo, drought,
      bestCombo: Math.max(state.bestCombo || 0, combo),
      over: !canPlay(result.board, hand),
    }
    return { ...result, combo, state: next, placed: true, replenished }
  }

  function serializeGame(state) {
    return {
      board: state.board, hand: state.hand.map((shape) => shape?.id || null),
      score: state.score, lines: state.lines, deals: state.deals || 1, combo: state.combo || 0,
      drought: state.drought || 0, bestCombo: state.bestCombo || 0,
    }
  }

  function restoreGame(data) {
    if (!data || typeof data !== 'object' ||
      !Array.isArray(data.board) || data.board.length !== SIZE ||
      !data.board.every((line) => Array.isArray(line) && line.length === SIZE &&
        line.every((value) => Number.isInteger(value) && value >= 0 && value <= 6))) return null
    if (!Array.isArray(data.hand) || data.hand.length !== 3) return null
    const hand = data.hand.map((id) =>
      id === null ? null : SHAPES.find((shape) => shape.id === id))
    if (hand.some((shape) => shape === undefined) || hand.every((shape) => shape === null)) return null
    for (const key of ['score', 'lines', 'combo', 'drought', 'bestCombo']) {
      if (!Number.isSafeInteger(data[key]) || data[key] < 0) return null
    }
    if (data.drought > 3 || data.combo > 100000 || data.bestCombo > 100000) return null
    if (data.deals !== undefined && (!Number.isSafeInteger(data.deals) || data.deals < 1 || data.deals > 1000000)) return null
    const board = data.board.map((line) => line.slice())
    return { board, hand, score: data.score, lines: data.lines, combo: data.combo,
      drought: data.drought, bestCombo: data.bestCombo, deals: data.deals || 1,
      over: !canPlay(board, hand) }
  }

  globalThis.XiabanBlockPuzzle = Object.freeze({ SIZE, SHAPES, createBoard, generateHand, canPlace, hasSpace, canPlay, placeShape, createGame, place, serializeGame, restoreGame, handOpportunity, dealGuidance })
})()
