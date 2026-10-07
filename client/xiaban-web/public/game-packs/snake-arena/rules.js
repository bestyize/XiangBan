(() => {
  'use strict'

  const WORLD = { width: 2400, height: 1700 }
  const SPACING = 6
  const DURATION = 180
  const MIN_MASS = 32
  const MAX_MASS = 1e7
  const MAX_SCORE = 1e9
  const MAX_FOOD = 1100
  const BASE_SPEED = 152
  const BOOST_SPEED = 280
  const TURN_SPEED = 5.6
  const FOOD_SHAPES = ['dot', 'candy', 'star', 'fruit', 'gem']
  const COLORS = ['#ffca28', '#20b8f5', '#ff5b85', '#aa55ef', '#54cc45', '#ff852b', '#5685f5', '#d6cb20']
  const NAMES = ['小伴', '奶盖', '桃桃', '糯米', '薄荷', '布丁', '晚风', '小满', '泡芙', '团子', '阿橘', '米粒']
  const clamp = (value, min, max) => Math.max(min, Math.min(max, value))
  const angleDelta = (from, to) => Math.atan2(Math.sin(to - from), Math.cos(to - from))
  const distanceSq = (a, b) => (a.x - b.x) ** 2 + (a.y - b.y) ** 2

  function seededRandom(seed = 1) {
    let value = seed >>> 0
    return () => {
      value += 0x6D2B79F5
      let next = Math.imul(value ^ (value >>> 15), 1 | value)
      next ^= next + Math.imul(next ^ (next >>> 7), 61 | next)
      return ((next ^ (next >>> 14)) >>> 0) / 4294967296
    }
  }

  function segmentDistanceSq(point, a, b) {
    const dx = b.x - a.x
    const dy = b.y - a.y
    const t = clamp(((point.x - a.x) * dx + (point.y - a.y) * dy) / (dx * dx + dy * dy || 1), 0, 1)
    return (point.x - a.x - t * dx) ** 2 + (point.y - a.y - t * dy) ** 2
  }

  // Include intersections so even two fast heads crossing within a tick collide.
  function segmentsDistanceSq(a, b, c, d) {
    const cross = (p, q, r) => (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x)
    const abC = cross(a, b, c), abD = cross(a, b, d)
    const cdA = cross(c, d, a), cdB = cross(c, d, b)
    if (abC * abD < 0 && cdA * cdB < 0) return 0
    return Math.min(segmentDistanceSq(a, c, d), segmentDistanceSq(b, c, d), segmentDistanceSq(c, a, b), segmentDistanceSq(d, a, b))
  }

  function radius(snake) { return 8 + Math.min(34, Math.sqrt(Math.max(0, snake.mass - 60)) * 0.65) }
  function bodyLength(snake) { return 75 + Math.min(3000, Math.max(0, snake.mass) * 1.4) }
  function bodyPoints(snake) { return [snake, ...snake.trail] }

  function makeSnake(id, x, y, angle, mass = 60) {
    const snake = {
      id, name: NAMES[id % NAMES.length], color: COLORS[id % COLORS.length], x, y, angle,
      targetAngle: angle, mass, score: 0, kills: 0, alive: true, energy: 100,
      boosting: false, boostLocked: false, invulnerable: 2.5, respawnAt: 0, aiTimer: 0,
      carry: 0, boostDrop: 0, trail: [], previous: { x, y },
    }
    // New opponents start with a readable curved tail, then grow their traveled
    // path to the full mass-based length. A bounded start fits safely in the map.
    const length = id === 0 ? bodyLength(snake) : Math.min(480, bodyLength(snake))
    for (let distance = 0; distance <= length; distance += SPACING) {
      const bend = distance / 180
      snake.trail.push({ x: x - 180 * (Math.sin(angle + bend) - Math.sin(angle)), y: y + 180 * (Math.cos(angle + bend) - Math.cos(angle)) })
    }
    return snake
  }

  function makeFood(state, x, y, value = 1, remains = false, ownerId = null) {
    return { id: state.nextFoodId++, x, y, value, remains, ownerId, shape: FOOD_SHAPES[Math.floor(state.random() * FOOD_SHAPES.length)], color: Math.floor(state.random() * 5), phase: state.random() * Math.PI * 2 }
  }

  function scatterFood(state, count) {
    for (let i = 0; i < count && state.food.length < MAX_FOOD; i += 1) {
      state.food.push(makeFood(state, 45 + state.random() * (WORLD.width - 90), 45 + state.random() * (WORLD.height - 90), state.random() < 0.09 ? 3 : 1))
    }
  }

  function createGame({ seed = Date.now(), mode = 'cozy', roundType = 'timed' } = {}) {
    roundType = roundType === 'endless' ? 'endless' : 'timed'
    const state = {
      random: seededRandom(seed), mode, roundType, time: 0, remaining: roundType === 'endless' ? null : DURATION, phase: 'playing', reason: null,
      food: [], snakes: [], events: [], nextFoodId: 1, foodTimer: 0, reviveUsed: false, milestones: Object.create(null),
    }
    state.snakes.push(makeSnake(0, WORLD.width / 2, WORLD.height / 2, -0.2))
    state.snakes[0].invulnerable = 2
    const count = 10
    for (let id = 1; id <= count; id += 1) {
      const tier = id <= 6 ? 0 : id <= 9 ? 1 : 2
      const mass = opponentMass(state, tier, true)
      const snake = findSpawn(state, id, mass, [380, 480, 620][tier])
      if (snake) state.snakes.push(snake)
    }
    scatterFood(state, 600)
    // A short, visible trail of snacks makes the first movement immediately rewarding.
    for (let i = 1; i <= 10; i += 1) {
      state.food.push(makeFood(state, 1200 + i * 23, 850 - i * 4.5, i % 4 === 0 ? 3 : 1))
    }
    return state
  }

  function buildBodyGrid(snakes, includeGhosts = false) {
    const size = 96
    const grid = new Map()
    for (const snake of snakes) {
      if (!snake.alive || (snake.invulnerable > 0 && !includeGhosts)) continue
      const points = snake.trail, padding = radius(snake)
      // Index four edges as a bounded group; collision still checks every exact
      // constituent edge, so sharp turns do not become shortcut hitboxes.
      for (let i = 0; i < points.length; i += 4) {
        const a = i === 0 ? snake : points[i - 1], b = points[i]
        const c = points[i + 1] || null, d = points[i + 2] || null, e = points[i + 3] || null
        let minX = Math.min(a.x, b.x), maxX = Math.max(a.x, b.x), minY = Math.min(a.y, b.y), maxY = Math.max(a.y, b.y)
        if (c) { minX = Math.min(minX, c.x); maxX = Math.max(maxX, c.x); minY = Math.min(minY, c.y); maxY = Math.max(maxY, c.y) }
        if (d) { minX = Math.min(minX, d.x); maxX = Math.max(maxX, d.x); minY = Math.min(minY, d.y); maxY = Math.max(maxY, d.y) }
        if (e) { minX = Math.min(minX, e.x); maxX = Math.max(maxX, e.x); minY = Math.min(minY, e.y); maxY = Math.max(maxY, e.y) }
        const entry = { snake, a, b, c, d, e, radius: padding, minX, maxX, minY, maxY, stamp: 0 }
        for (let x = Math.floor((minX - padding) / size); x <= Math.floor((maxX + padding) / size); x += 1) {
          for (let y = Math.floor((minY - padding) / size); y <= Math.floor((maxY + padding) / size); y += 1) {
            const key = x + y * 64
            if (!grid.has(key)) grid.set(key, [])
            grid.get(key).push(entry)
          }
        }
      }
    }
    return { grid, size, query: 0 }
  }

  function nearbyBodies(index, point, reach) {
    const entries = [], stamp = ++index.query
    for (let x = Math.floor((point.x - reach) / index.size); x <= Math.floor((point.x + reach) / index.size); x += 1) {
      for (let y = Math.floor((point.y - reach) / index.size); y <= Math.floor((point.y + reach) / index.size); y += 1) {
        const bucket = index.grid.get(x + y * 64)
        if (!bucket) continue
        for (const entry of bucket) {
          if (entry.stamp === stamp) continue
          entry.stamp = stamp; entries.push(entry)
        }
      }
    }
    return entries
  }
  function bodyDistanceSq(point, entry) {
    let distance = segmentDistanceSq(point, entry.a, entry.b)
    if (entry.c) distance = Math.min(distance, segmentDistanceSq(point, entry.b, entry.c))
    if (entry.d) distance = Math.min(distance, segmentDistanceSq(point, entry.c, entry.d))
    if (entry.e) distance = Math.min(distance, segmentDistanceSq(point, entry.d, entry.e))
    return distance
  }
  function hitsBody(previous, head, entry, reach) {
    if (entry.maxX < Math.min(previous.x, head.x) - reach || entry.minX > Math.max(previous.x, head.x) + reach || entry.maxY < Math.min(previous.y, head.y) - reach || entry.minY > Math.max(previous.y, head.y) + reach) return false
    const squared = reach * reach
    return segmentsDistanceSq(previous, head, entry.a, entry.b) < squared
      || (entry.c && segmentsDistanceSq(previous, head, entry.b, entry.c) < squared)
      || (entry.d && segmentsDistanceSq(previous, head, entry.c, entry.d) < squared)
      || (entry.e && segmentsDistanceSq(previous, head, entry.d, entry.e) < squared)
  }

  function decideAI(state, snake, index, dt) {
    snake.aiTimer -= dt
    if (snake.aiTimer > 0) return
    snake.aiTimer = 0.13 + state.random() * 0.06
    const player = state.snakes[0]
    const gentleOpening = state.time < 12 && player.alive
    let target = null, foodCost = Infinity
    for (const food of state.food) {
      const distance = distanceSq(snake, food)
      if (distance > 450 * 450) continue
      // Let beginners collect nearby snacks instead of competing for the exact
      // same head space during their first few turns.
      if (gentleOpening && distanceSq(food, player) < 170 ** 2) continue
      const bearing = Math.atan2(food.y - snake.y, food.x - snake.x)
      const cost = distance * (1 + Math.abs(angleDelta(snake.angle, bearing)) * 0.8) / Math.sqrt(food.value)
      if (cost < foodCost) { foodCost = cost; target = food }
    }
    let desired = target ? Math.atan2(target.y - snake.y, target.x - snake.x) : Math.atan2(WORLD.height / 2 - snake.y, WORLD.width / 2 - snake.x)
    // The legacy difficulty argument remains compatible, but all rounds use
    // one consistent opponent behavior and movement model.
    if (state.time >= 20 && snake.id % 3 === 0 && player.alive && player.invulnerable <= 0 && distanceSq(snake, player) < 270 ** 2) {
      desired = Math.atan2(player.y + Math.sin(player.angle) * 95 - snake.y, player.x + Math.cos(player.angle) * 95 - snake.x)
    }
    const obstacles = nearbyBodies(index, snake, 190)
    const snakeRadius = radius(snake)
    let bestHeading = snake.angle, bestCost = Infinity
    for (const turn of [-1.4, -0.95, -0.55, 0, 0.55, 0.95, 1.4]) {
      const heading = snake.angle + turn
      let cost = Math.abs(angleDelta(heading, desired)) * 24 + Math.abs(turn) * 4
      for (const ahead of [40, 85, 140]) {
        const point = { x: snake.x + Math.cos(heading) * ahead, y: snake.y + Math.sin(heading) * ahead }
        const wall = Math.min(point.x, point.y, WORLD.width - point.x, WORLD.height - point.y)
        if (wall < 80) cost += (80 - wall) * 4
        if (gentleOpening) {
          const playerGap = Math.sqrt(distanceSq(point, player))
          if (playerGap < 170) cost += (170 - playerGap) * 14
          const aheadPlayer = { x: player.x + Math.cos(player.angle) * 300, y: player.y + Math.sin(player.angle) * 300 }
          const forwardGap = Math.sqrt(segmentDistanceSq(point, player, aheadPlayer))
          if (forwardGap < 170) cost += (170 - forwardGap) * 14
        }
        for (const entry of obstacles) {
          if (entry.snake.id === snake.id) continue
          const clearance = entry.radius + snakeRadius + 55
          const dx = Math.max(entry.minX - point.x, 0, point.x - entry.maxX), dy = Math.max(entry.minY - point.y, 0, point.y - entry.maxY)
          if (dx * dx + dy * dy >= clearance * clearance) continue
          const gap = Math.sqrt(bodyDistanceSq(point, entry)) - entry.radius - snakeRadius
          if (gap < 55) cost += (55 - gap) * (ahead === 40 ? 8 : 3)
        }
      }
      if (cost < bestCost) { bestCost = cost; bestHeading = heading }
    }
    snake.targetAngle = bestHeading
    snake.aiBoost = state.time >= 20 && bestCost < 30 && foodCost < 180 ** 2 && snake.energy > 45
  }

  function moveSnake(snake, target, boost, dt) {
    snake.previous = { x: snake.x, y: snake.y }
    if (Number.isFinite(target)) snake.targetAngle = target
    snake.angle += clamp(angleDelta(snake.angle, snake.targetAngle), -TURN_SPEED * dt, TURN_SPEED * dt)
    if (snake.energy <= 1) snake.boostLocked = true
    if (snake.energy >= 25) snake.boostLocked = false
    snake.boosting = Boolean(boost && !snake.boostLocked && snake.energy > 1 && snake.mass > MIN_MASS)
    snake.energy = clamp(snake.energy + (snake.boosting ? -31 : 22) * dt, 0, 100)
    const speed = snake.boosting ? BOOST_SPEED : BASE_SPEED
    const distance = speed * dt
    snake.x += Math.cos(snake.angle) * distance
    snake.y += Math.sin(snake.angle) * distance
    // Resample actual travel at constant arc length, rather than pulling a chain of circles.
    const oldCarry = snake.carry
    for (let travelled = SPACING - oldCarry; travelled <= distance; travelled += SPACING) {
      const t = travelled / distance
      snake.trail.unshift({ x: snake.previous.x + (snake.x - snake.previous.x) * t, y: snake.previous.y + (snake.y - snake.previous.y) * t })
    }
    snake.carry = (oldCarry + distance) % SPACING
    snake.trail.length = Math.min(snake.trail.length, Math.ceil(bodyLength(snake) / SPACING) + 1)
    snake.invulnerable = Math.max(0, snake.invulnerable - dt)
  }

  function killSnake(state, snake, killerId = null, reason = 'collision') {
    if (!snake.alive) return
    snake.alive = false
    snake.boosting = false
    snake.respawnAt = state.time + 3.5
    const count = Math.min(110, Math.max(12, Math.ceil(Math.sqrt(snake.mass) * 3)))
    // Replace ordinary snacks when the map is full, rather than losing the
    // large snake's reward merely because food density hit its safety cap.
    while (state.food.length + count > MAX_FOOD) {
      const ordinary = state.food.findIndex(food => !food.remains)
      if (ordinary < 0) break
      state.food.splice(ordinary, 1)
    }
    const available = MAX_FOOD - state.food.length
    const drops = Math.min(count, available)
    const totalValue = Math.max(36, Math.floor(snake.mass * 0.65))
    const points = bodyPoints(snake)
    for (let i = 0; i < drops; i += 1) {
      const point = points[Math.min(points.length - 1, Math.floor(i * points.length / drops))]
      const value = Math.floor(totalValue / drops) + (i < totalValue % drops ? 1 : 0)
      state.food.push(makeFood(state, clamp(point.x + (state.random() - 0.5) * 16, 20, WORLD.width - 20), clamp(point.y + (state.random() - 0.5) * 16, 20, WORLD.height - 20), Math.max(1, value), true, snake.id))
    }
    const killer = state.snakes.find((entry) => entry.id === killerId)
    const reward = Math.min(1e6, 80 + Math.floor(snake.mass * 0.25))
    if (killer) {
      killer.kills += 1; killer.score = Math.min(MAX_SCORE, killer.score + reward)
      if (killer.id === 0) {
        milestone(state, 'first-kill', killer.kills)
        if (snake.mass >= 200 && snake.mass >= killer.mass * 1.8) milestone(state, 'giant-slayer', Math.floor(snake.mass))
      }
    }
    state.events.push({ type: 'death', id: snake.id, killerId, x: snake.x, y: snake.y, color: snake.color, reason, reward: killer ? reward : 0, victimMass: snake.mass })
    if (snake.id === 0) { state.phase = 'over'; state.reason = reason }
  }

  function opponentMass(state, tier = Math.floor(state.random() * 3), initial = false) {
    const start = [[35, 75], [85, 150], [180, 260]], grown = [[45, 110], [180, 420], [650, 1100]]
    const progress = initial ? 0 : clamp(Math.max(state.time / 90, state.snakes[0].mass / 1000), 0, 1)
    const min = start[tier][0] + (grown[tier][0] - start[tier][0]) * progress
    const max = start[tier][1] + (grown[tier][1] - start[tier][1]) * progress
    return Math.floor(min + state.random() * (max - min))
  }

  function findSpawn(state, id, mass, playerDistance = 350) {
    const index = buildBodyGrid(state.snakes.filter(snake => snake.id !== id), true)
    for (let attempt = 0; attempt < 180; attempt += 1) {
      const x = 100 + state.random() * (WORLD.width - 200)
      const y = 100 + state.random() * (WORLD.height - 200)
      const snake = makeSnake(id, x, y, state.random() * Math.PI * 2, mass)
      const player = state.snakes[0]
      if (id !== 0 && player?.alive && distanceSq(snake, player) < playerDistance ** 2) continue
      const reserveOpening = id !== 0 && state.time === 0
      const openingEnd = { x: WORLD.width / 2 + Math.cos(-0.2) * 800, y: WORLD.height / 2 + Math.sin(-0.2) * 800 }
      const r = radius(snake)
      const points = bodyPoints(snake)
      let clear = true
      for (let i = 0; i < points.length; i += 3) {
        const point = points[i]
        if (point.x < r + 24 || point.y < r + 24 || point.x > WORLD.width - r - 24 || point.y > WORLD.height - r - 24) { clear = false; break }
        if (reserveOpening && segmentDistanceSq(point, player, openingEnd) < (r + 170) ** 2) { clear = false; break }
        for (const entry of nearbyBodies(index, point, r + 75)) {
          if (bodyDistanceSq(point, entry) < (r + entry.radius + 42) ** 2) { clear = false; break }
        }
        if (!clear) break
      }
      if (clear) return snake
    }
    return null
  }

  function respawn(state, dead) {
    const snake = findSpawn(state, dead.id, opponentMass(state))
    if (snake) state.snakes[state.snakes.indexOf(dead)] = snake
    else dead.respawnAt = state.time + 0.5
  }

  function milestone(state, key, value) {
    if (state.milestones[key]) return
    state.milestones[key] = true
    state.events.push({ type: 'milestone', key, value })
  }

  function activateShield(state, seconds = 3) {
    const player = state.snakes[0]
    if (state.phase !== 'playing' || !player?.alive || !Number.isFinite(seconds) || seconds <= 0) return false
    player.invulnerable = Math.max(player.invulnerable, Math.min(5, seconds))
    state.events.push({ type: 'shield', seconds: player.invulnerable })
    return true
  }

  function revivePlayer(state) {
    const player = state.snakes[0]
    if (state.phase !== 'over' || player?.alive || state.reviveUsed || !['wall', 'collision'].includes(state.reason) || (state.roundType !== 'endless' && state.remaining <= 0)) return false
    const mass = Math.max(60, Math.floor(player.mass * 0.6))
    let replacement = findSpawn(state, 0, mass)
    if (!replacement) {
      // A purchased revive must not fail merely because late-game bodies fill
      // the arena. Select an in-bounds fallback far from heads; its short tail
      // and nonattacking five-second shield let the player escape overlapping bodies.
      let safest = { x: WORLD.width / 2, y: WORLD.height / 2 }, bestClearance = -1
      for (let row = 0; row < 5; row += 1) for (let col = 0; col < 7; col += 1) {
        const point = { x: 200 + col * (WORLD.width - 400) / 6, y: 200 + row * (WORLD.height - 400) / 4 }
        const heads = state.snakes.filter(snake => snake.id !== 0 && snake.alive)
        const clearance = heads.length ? Math.min(...heads.map(snake => distanceSq(point, snake))) : Infinity
        if (clearance > bestClearance) { safest = point; bestClearance = clearance }
      }
      replacement = makeSnake(0, safest.x, safest.y, Math.atan2(WORLD.height / 2 - safest.y, WORLD.width / 2 - safest.x), mass)
      replacement.trail.length = Math.min(23, replacement.trail.length)
    }
    replacement.score = player.score
    replacement.kills = player.kills
    replacement.invulnerable = 5
    // Resuming never opens a new round: score, elapsed active time and milestones survive.
    state.snakes[0] = replacement
    state.food = state.food.filter(food => !(food.remains && food.ownerId === 0))
    state.phase = 'playing'; state.reason = null; state.reviveUsed = true
    state.events.push({ type: 'revive', mass, seconds: 5 })
    return true
  }

  function step(state, input = {}, dt = 1 / 60) {
    state.events = []
    if (state.phase !== 'playing' || !Number.isFinite(dt) || dt <= 0) return state.events
    dt = Math.min(dt, 1 / 30)
    state.time += dt
    state.remaining = state.roundType === 'endless' ? null : Math.max(0, DURATION - state.time)
    const aiDue = state.snakes.some(snake => snake.alive && snake.id !== 0 && !(snake.aiTimer > dt))
    const before = aiDue ? buildBodyGrid(state.snakes, true) : null
    for (const snake of state.snakes) {
      if (!snake.alive) { if (snake.id !== 0 && state.time >= snake.respawnAt) respawn(state, snake); continue }
      if (snake.id !== 0) decideAI(state, snake, before, dt)
      moveSnake(snake, snake.id === 0 ? input.angle : snake.targetAngle, snake.id === 0 ? input.boost : snake.aiBoost, dt)
      if (snake.boosting) {
        snake.boostDrop += dt
        if (snake.boostDrop >= 0.22) {
          snake.boostDrop -= 0.22
          snake.mass = Math.max(MIN_MASS, snake.mass - 1)
          const tail = snake.trail[snake.trail.length - 1]
          if (tail && state.food.length < MAX_FOOD) state.food.push(makeFood(state, clamp(tail.x, 20, WORLD.width - 20), clamp(tail.y, 20, WORLD.height - 20)))
        }
      }
    }
    const index = buildBodyGrid(state.snakes)
    const deaths = []
    for (const snake of state.snakes) {
      if (!snake.alive) continue
      const r = radius(snake)
      if (snake.invulnerable > 0) {
        // A shield also covers boundaries, without letting the snake escape the map.
        snake.x = clamp(snake.x, r, WORLD.width - r)
        snake.y = clamp(snake.y, r, WORLD.height - r)
        for (const point of snake.trail) {
          point.x = clamp(point.x, r, WORLD.width - r)
          point.y = clamp(point.y, r, WORLD.height - r)
        }
        continue
      }
      if (snake.x < r || snake.y < r || snake.x > WORLD.width - r || snake.y > WORLD.height - r) {
        deaths.push({ snake, reason: 'wall', killerId: null }); continue
      }
      const candidates = nearbyBodies(index, snake, r + 12)
      for (const entry of candidates) {
        if (entry.snake.id === snake.id) continue
        const reach = r + entry.radius - 2
        if (hitsBody(snake.previous, snake, entry, reach)) {
          deaths.push({ snake, reason: 'collision', killerId: entry.snake.id }); break
        }
      }
      // Swept head paths give symmetric results when two snakes meet head-on.
      for (const other of state.snakes) {
        if (other.id <= snake.id || !other.alive || other.invulnerable > 0) continue
        if (segmentsDistanceSq(snake.previous, snake, other.previous, other) < (r + radius(other) - 2) ** 2) {
          deaths.push({ snake, reason: 'collision', killerId: other.id })
          deaths.push({ snake: other, reason: 'collision', killerId: snake.id })
        }
      }
    }
    const uniqueDeaths = new Map(deaths.map((entry) => [entry.snake.id, entry]))
    for (const entry of uniqueDeaths.values()) killSnake(state, entry.snake, entry.killerId, entry.reason)

    const eaters = state.snakes.filter(snake => snake.alive).map(snake => ({ snake, reach: Math.max(34, radius(snake) + 19) ** 2 }))
    for (let i = state.food.length - 1; i >= 0; i -= 1) {
      const food = state.food[i]
      let eater = null, nearest = Infinity
      for (const candidate of eaters) {
        const snake = candidate.snake
        const distance = distanceSq(snake, food)
        if (distance < candidate.reach && distance < nearest) { eater = candidate; nearest = distance }
      }
      if (!eater) continue
      const snake = eater.snake
      snake.mass = Math.min(MAX_MASS, snake.mass + food.value)
      snake.score = Math.min(MAX_SCORE, snake.score + food.value * 10)
      eater.reach = Math.max(34, radius(snake) + 19) ** 2
      if (snake.id === 0) state.events.push({ type: 'eat', x: food.x, y: food.y, value: food.value, color: food.color })
      state.food.splice(i, 1)
    }
    state.foodTimer += dt
    if (state.foodTimer > 1) { state.foodTimer = 0; scatterFood(state, Math.min(22, Math.max(0, 620 - state.food.length))) }
    const player = state.snakes[0]
    if (player.alive) {
      for (const mass of [100, 250, 500, 1000, 2500, 5000]) if (player.mass >= mass) milestone(state, 'growth-' + mass, mass)
      const rank = rankings(state).findIndex(snake => snake.id === 0) + 1
      if (rank <= 3) milestone(state, 'top-3', rank)
      if (rank === 1) milestone(state, 'first-place', rank)
    }
    if (state.roundType !== 'endless' && state.remaining === 0 && state.phase === 'playing') { state.phase = 'over'; state.reason = 'timeout'; state.events.push({ type: 'finish' }) }
    return state.events
  }

  function rankings(state) {
    return state.snakes.filter((snake) => snake.alive || snake.id === 0).sort((a, b) => b.mass - a.mass || b.score - a.score || a.id - b.id)
  }

  globalThis.XiabanSnakeArena = { WORLD, DURATION, MIN_MASS, MAX_MASS, MAX_FOOD, BASE_SPEED, BOOST_SPEED, TURN_SPEED, FOOD_SHAPES, COLORS, createGame, step, activateShield, revivePlayer, radius, bodyLength, bodyPoints, rankings, angleDelta, segmentDistanceSq, segmentsDistanceSq, seededRandom }
})()
