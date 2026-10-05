(() => {
  'use strict'

  const WORLD = { width: 2400, height: 1700 }
  const SPACING = 6
  const DURATION = 180
  const MIN_MASS = 32
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

  function radius(snake) { return 14 + Math.min(7, snake.mass * 0.009) }
  function bodyLength(snake) { return 105 + Math.min(1800, snake.mass * 2.05) }
  function bodyPoints(snake) { return [snake, ...snake.trail] }

  function makeSnake(id, x, y, angle, mass = 60) {
    const snake = {
      id, name: NAMES[id % NAMES.length], color: COLORS[id % COLORS.length], x, y, angle,
      targetAngle: angle, mass, score: 0, kills: 0, alive: true, energy: 100,
      boosting: false, boostLocked: false, invulnerable: 2.5, respawnAt: 0, aiTimer: 0,
      carry: 0, boostDrop: 0, trail: [], previous: { x, y },
    }
    for (let distance = 0; distance <= bodyLength(snake); distance += SPACING) {
      snake.trail.push({ x: x - Math.cos(angle) * distance, y: y - Math.sin(angle) * distance })
    }
    return snake
  }

  function makeFood(state, x, y, value = 1, remains = false) {
    return { id: state.nextFoodId++, x, y, value, remains, color: Math.floor(state.random() * 5), phase: state.random() * Math.PI * 2 }
  }

  function scatterFood(state, count) {
    for (let i = 0; i < count; i += 1) {
      state.food.push(makeFood(state, 45 + state.random() * (WORLD.width - 90), 45 + state.random() * (WORLD.height - 90), state.random() < 0.09 ? 3 : 1))
    }
  }

  function createGame({ seed = Date.now(), mode = 'cozy' } = {}) {
    const state = {
      random: seededRandom(seed), mode, time: 0, remaining: DURATION, phase: 'playing', reason: null,
      food: [], snakes: [], events: [], nextFoodId: 1, foodTimer: 0,
    }
    state.snakes.push(makeSnake(0, WORLD.width / 2, WORLD.height / 2, -0.2))
    state.snakes[0].invulnerable = mode === 'cozy' ? 5 : 3
    const count = mode === 'lively' ? 11 : 8
    for (let id = 1; id <= count; id += 1) {
      const theta = id * Math.PI * 2 / count + 0.35
      const range = (mode === 'cozy' ? 530 : 390) + (id % 3) * 100
      const x = WORLD.width / 2 + Math.cos(theta) * range
      const y = WORLD.height / 2 + Math.sin(theta) * range
      // Tangential starts keep whole bodies inside the map and give a safe first move.
      state.snakes.push(makeSnake(id, x, y, theta + Math.PI / 2, 55 + state.random() * 65))
    }
    scatterFood(state, 600)
    // A short, visible trail of snacks makes the first movement immediately rewarding.
    for (let i = 1; i <= 10; i += 1) {
      state.food.push(makeFood(state, 1200 + i * 23, 850 - i * 4.5, i % 4 === 0 ? 3 : 1))
    }
    return state
  }

  function buildBodyGrid(snakes, includeGhosts = false) {
    const size = 72
    const grid = new Map()
    for (const snake of snakes) {
      if (!snake.alive || (snake.invulnerable > 0 && !includeGhosts)) continue
      const points = bodyPoints(snake)
      for (let i = 1; i < points.length; i += 1) {
        const a = points[i - 1], b = points[i]
        const padding = radius(snake)
        const entry = { snake, a, b }
        for (let x = Math.floor((Math.min(a.x, b.x) - padding) / size); x <= Math.floor((Math.max(a.x, b.x) + padding) / size); x += 1) {
          for (let y = Math.floor((Math.min(a.y, b.y) - padding) / size); y <= Math.floor((Math.max(a.y, b.y) + padding) / size); y += 1) {
            const key = x + ':' + y
            if (!grid.has(key)) grid.set(key, [])
            grid.get(key).push(entry)
          }
        }
      }
    }
    return { grid, size }
  }

  function nearbyBodies(index, point, reach) {
    const entries = new Set()
    for (let x = Math.floor((point.x - reach) / index.size); x <= Math.floor((point.x + reach) / index.size); x += 1) {
      for (let y = Math.floor((point.y - reach) / index.size); y <= Math.floor((point.y + reach) / index.size); y += 1) {
        for (const entry of index.grid.get(x + ':' + y) || []) entries.add(entry)
      }
    }
    return entries
  }

  function decideAI(state, snake, index, dt) {
    snake.aiTimer -= dt
    if (snake.aiTimer > 0) return
    snake.aiTimer = 0.13 + state.random() * 0.06
    let target = null, foodCost = Infinity
    for (const food of state.food) {
      const distance = distanceSq(snake, food)
      if (distance > 450 * 450) continue
      const bearing = Math.atan2(food.y - snake.y, food.x - snake.x)
      const cost = distance * (1 + Math.abs(angleDelta(snake.angle, bearing)) * 0.8) / Math.sqrt(food.value)
      if (cost < foodCost) { foodCost = cost; target = food }
    }
    let desired = target ? Math.atan2(target.y - snake.y, target.x - snake.x) : Math.atan2(WORLD.height / 2 - snake.y, WORLD.width / 2 - snake.x)
    const player = state.snakes[0]
    // Some lively bots attempt a cut-off, but use the same movement and collisions as the player.
    if (state.mode === 'lively' && snake.id % 3 === 0 && player.alive && distanceSq(snake, player) < 270 ** 2) {
      desired = Math.atan2(player.y + Math.sin(player.angle) * 95 - snake.y, player.x + Math.cos(player.angle) * 95 - snake.x)
    }
    const obstacles = nearbyBodies(index, snake, 190)
    let bestHeading = snake.angle, bestCost = Infinity
    for (const turn of [-1.4, -0.95, -0.55, 0, 0.55, 0.95, 1.4]) {
      const heading = snake.angle + turn
      let cost = Math.abs(angleDelta(heading, desired)) * 24 + Math.abs(turn) * 4
      for (const ahead of [40, 85, 140]) {
        const point = { x: snake.x + Math.cos(heading) * ahead, y: snake.y + Math.sin(heading) * ahead }
        const wall = Math.min(point.x, point.y, WORLD.width - point.x, WORLD.height - point.y)
        if (wall < 80) cost += (80 - wall) * 4
        for (const entry of obstacles) {
          if (entry.snake.id === snake.id) continue
          const gap = Math.sqrt(segmentDistanceSq(point, entry.a, entry.b)) - radius(entry.snake) - radius(snake)
          if (gap < 55) cost += (55 - gap) * (ahead === 40 ? 8 : 3)
        }
      }
      if (cost < bestCost) { bestCost = cost; bestHeading = heading }
    }
    snake.targetAngle = bestHeading
    snake.aiBoost = state.mode === 'lively' && bestCost < 30 && foodCost < 180 ** 2 && snake.energy > 45
  }

  function moveSnake(snake, target, boost, dt) {
    snake.previous = { x: snake.x, y: snake.y }
    if (Number.isFinite(target)) snake.targetAngle = target
    snake.angle += clamp(angleDelta(snake.angle, snake.targetAngle), -3.1 * dt, 3.1 * dt)
    if (snake.energy <= 1) snake.boostLocked = true
    if (snake.energy >= 25) snake.boostLocked = false
    snake.boosting = Boolean(boost && !snake.boostLocked && snake.energy > 1 && snake.mass > MIN_MASS)
    snake.energy = clamp(snake.energy + (snake.boosting ? -31 : 22) * dt, 0, 100)
    const speed = snake.boosting ? 218 : 118
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
    const count = Math.min(65, Math.max(12, Math.ceil(snake.mass / 3)))
    const points = bodyPoints(snake)
    for (let i = 0; i < count && state.food.length < 1100; i += 1) {
      const point = points[Math.min(points.length - 1, Math.floor(i * points.length / count))]
      state.food.push(makeFood(state, clamp(point.x + (state.random() - 0.5) * 16, 20, WORLD.width - 20), clamp(point.y + (state.random() - 0.5) * 16, 20, WORLD.height - 20), 3, true))
    }
    const killer = state.snakes.find((entry) => entry.id === killerId)
    if (killer) { killer.kills += 1; killer.score += 80 }
    state.events.push({ type: 'death', id: snake.id, killerId, x: snake.x, y: snake.y, color: snake.color, reason })
    if (snake.id === 0) { state.phase = 'over'; state.reason = reason }
  }

  function respawn(state, dead) {
    for (let attempt = 0; attempt < 25; attempt += 1) {
      const x = 260 + state.random() * (WORLD.width - 520)
      const y = 260 + state.random() * (WORLD.height - 520)
      const clear = state.snakes.every((other) => !other.alive || bodyPoints(other).every((point) => distanceSq(point, { x, y }) > 230 ** 2))
      if (clear) {
        const angle = Math.atan2(WORLD.height / 2 - y, WORLD.width / 2 - x)
        const snake = makeSnake(dead.id, x, y, angle, 55 + state.random() * 30)
        // Spawn with a short tail so the head's clear area also contains the entire body.
        snake.trail.length = Math.min(25, snake.trail.length)
        state.snakes[state.snakes.indexOf(dead)] = snake
        return
      }
    }
  }

  function step(state, input = {}, dt = 1 / 60) {
    state.events = []
    if (state.phase !== 'playing' || !Number.isFinite(dt) || dt <= 0) return state.events
    dt = Math.min(dt, 1 / 30)
    state.time += dt
    state.remaining = Math.max(0, DURATION - state.time)
    const before = buildBodyGrid(state.snakes, true)
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
          if (tail && state.food.length < 1100) state.food.push(makeFood(state, tail.x, tail.y))
        }
      }
    }
    const index = buildBodyGrid(state.snakes)
    const deaths = []
    for (const snake of state.snakes) {
      if (!snake.alive) continue
      const r = radius(snake)
      if (snake.x < r || snake.y < r || snake.x > WORLD.width - r || snake.y > WORLD.height - r) {
        deaths.push({ snake, reason: 'wall', killerId: null }); continue
      }
      if (snake.invulnerable > 0) continue
      const candidates = nearbyBodies(index, snake, r + 12)
      for (const entry of candidates) {
        if (entry.snake.id === snake.id) continue
        const reach = r + radius(entry.snake) - 2
        if (segmentsDistanceSq(snake.previous, snake, entry.a, entry.b) < reach * reach) {
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

    for (let i = state.food.length - 1; i >= 0; i -= 1) {
      const food = state.food[i]
      let eater = null, nearest = Infinity
      for (const snake of state.snakes) {
        if (!snake.alive) continue
        const distance = distanceSq(snake, food)
        if (distance < (radius(snake) + 13) ** 2 && distance < nearest) { eater = snake; nearest = distance }
      }
      if (!eater) continue
      eater.mass = Math.min(1000, eater.mass + food.value)
      eater.score += food.value * 10
      if (eater.id === 0) state.events.push({ type: 'eat', x: food.x, y: food.y, value: food.value, color: food.color })
      state.food.splice(i, 1)
    }
    state.foodTimer += dt
    if (state.foodTimer > 1) { state.foodTimer = 0; scatterFood(state, Math.min(22, Math.max(0, 620 - state.food.length))) }
    if (state.remaining === 0 && state.phase === 'playing') { state.phase = 'over'; state.reason = 'timeout'; state.events.push({ type: 'finish' }) }
    return state.events
  }

  function rankings(state) {
    return state.snakes.filter((snake) => snake.alive || snake.id === 0).sort((a, b) => b.mass - a.mass || b.score - a.score || a.id - b.id)
  }

  globalThis.XiabanSnakeArena = { WORLD, DURATION, MIN_MASS, COLORS, createGame, step, radius, bodyLength, bodyPoints, rankings, angleDelta, segmentDistanceSq, segmentsDistanceSq, seededRandom }
})()
