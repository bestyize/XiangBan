/* 水墨剑客 · deterministic side-scrolling combat rules, free of DOM access so tests can run the shipped file. */
(() => {
  'use strict'

  const GROUND = 300
  const GRAVITY = 1900
  const RUN = 240
  const ARROW_GRAVITY = 260
  const LEVELS = globalThis.XiabanInkLevels

  const DIFFICULTIES = {
    easy: { hp: 7, parry: 0.24, aggression: 0.7, tokens: 1, tele: 1.2, bossHp: 34 },
    normal: { hp: 5, parry: 0.16, aggression: 1, tokens: 2, tele: 1, bossHp: 46 },
  }

  // Slash stages: duration, active frames, chain point, forward lunge, damage, reach and the drawn arc.
  const STAGES = [
    { dur: 0.3, a0: 0.035, a1: 0.12, chain: 0.15, lunge: 130, damage: 1, reach: 78, arc: [-2.25, 0.85] },
    { dur: 0.3, a0: 0.035, a1: 0.12, chain: 0.15, lunge: 130, damage: 1, reach: 78, arc: [0.95, -2.05] },
    { dur: 0.44, a0: 0.08, a1: 0.19, chain: 0.4, lunge: 310, damage: 2, reach: 90, arc: [-1.95, 1.45] },
  ]

  const ENEMY = {
    dummy: { hp: 2, w: 14, h: 50, score: 20 },
    puppet: { hp: 3, w: 15, h: 52, score: 100, speed: 108, reach: 80, tele: 0.5, active: 0.12, recover: 0.55, lunge: 330, damage: 1 },
    shield: { hp: 5, w: 19, h: 56, score: 200, speed: 70, reach: 86, tele: 0.7, active: 0.15, recover: 0.75, lunge: 240, damage: 1 },
    archer: { hp: 2, w: 14, h: 52, score: 120 },
    crow: { hp: 1, w: 14, h: 12, score: 60 },
    boss: { hp: 36, w: 21, h: 72, score: 2000 },
    ronin: { hp: 3, w: 15, h: 52, score: 180, speed: 130, reach: 85, tele: 0.44, active: 0.12, recover: 0.5, lunge: 360, damage: 1 },
    spearman: { hp: 4, w: 16, h: 56, score: 220, speed: 78, reach: 130, tele: 0.52, active: 0.14, recover: 0.65, lunge: 280, damage: 1 },
    hammer: { hp: 7, w: 22, h: 66, score: 320, speed: 52, reach: 105, tele: 0.68, active: 0.18, recover: 0.85, lunge: 190, damage: 1 },
    angler: { hp: 48, w: 20, h: 68, score: 2500 },
  }

  function mulberry(seed) {
    let value = seed >>> 0
    return () => {
      value = (value + 0x6d2b79f5) >>> 0
      let t = value
      t = Math.imul(t ^ (t >>> 15), t | 1)
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296
    }
  }

  const clamp = (value, min, max) => Math.max(min, Math.min(max, value))
  const sign = (value) => (value > 0 ? 1 : value < 0 ? -1 : 0)

  /** options.level is a 幅 id such as '1-3' (default: the whole first scroll); `start: 'boss'` is kept as '1-6'. */
  function createGame(options = {}) {
    const difficulty = options.difficulty === 'easy' ? 'easy' : 'normal'
    const diff = DIFFICULTIES[difficulty]
    const level = LEVELS.level(options.start === 'boss' ? '1-6' : options.level) || LEVELS.level('1-scroll')
    const world = LEVELS.volume(level.volume)
    const promptsShown = {}
    for (const key of options.seenPrompts || []) if (typeof key === 'string') promptsShown[key] = true
    const state = {
      difficulty, diff, level, world, plan: level.encounters.map((id) => world.encounters[id]), seed: options.seed ?? 20261006,
      phase: 'playing', reason: null, time: 0, clock: 0,
      hitstop: 0, slow: 0, slowScale: 1, strike: null,
      lock: null, encounter: 0, wave: 0, waveDelay: 0,
      area: -1, promptsShown, nextId: 1, enemies: [], projectiles: [],
      bonus: null, victoryT: 0, events: [],
      icicles: (world.icicles || []).map((spec, index) => ({ id: index, x: spec.x, top: spec.y, len: spec.len || 24, y: spec.y + (spec.len || 24), state: 'hang', vy: 0, hits: [], hitPlayer: false })),
      cracks: [], nextCrack: 0, wind: { phase: 'calm', dir: 0, power: 0, cycle: -1 },
      player: {
        x: level.start, y: GROUND, vx: 0, vy: 0, w: 13, h: 54, facing: 1,
        onGround: true, coyote: 0, airJumps: 1, airDash: 1,
        jumpBuffer: 0, slashBuffer: 0, dashBuffer: 0, inkBuffer: 0, jumpHeld: false, jumpCut: false,
        action: null, slashStartClock: -9, slashEndClock: -9, nextStage: 0, dashCd: 0,
        hp: diff.hp, maxHp: diff.hp, iframes: 0, hurt: 0, dead: false, deadT: 0,
        ink: 0, combo: 0, comboTimer: 0, maxCombo: 0, kills: 0, parries: 0, executions: 0,
        score: 0, damageTaken: 0, airTime: 0, airKills: 0, deflectKills: 0, bossExecutions: 0, strikes: 0,
        icicleKills: 0, crackKills: 0, slideKills: 0, ambushes: 0, standT: 0, coverT: 0, hideT: 0, spottedT: 0, hidden: false,
      },
    }
    state.random = mulberry(state.seed)
    return state
  }

  function emit(state, type, data = {}) { state.events.push({ type, ...data }) }

  function addScore(state, base) {
    const player = state.player
    player.score += Math.round(base * (1 + Math.min(player.combo, 40) * 0.05))
  }

  function addCombo(state) {
    const player = state.player
    player.combo += 1
    player.comboTimer = 2.6
    player.maxCombo = Math.max(player.maxCombo, player.combo)
  }

  function addInk(state, amount) {
    const player = state.player
    const before = player.ink
    player.ink = Math.min(100, player.ink + amount)
    if (before < 100 && player.ink >= 100) {
      emit(state, 'inkReady')
      prompt(state, 'ink')
    }
  }

  function prompt(state, key) {
    if (state.promptsShown[key]) return
    state.promptsShown[key] = true
    emit(state, 'prompt', { key })
  }

  function platformUnder(platforms, x, halfWidth, previousFeet, feet) {
    let landing = feet >= GROUND ? GROUND : null
    for (const platform of platforms) {
      if (x + halfWidth * 0.5 < platform.x || x - halfWidth * 0.5 > platform.x + platform.w) continue
      if (previousFeet <= platform.y + 0.5 && feet >= platform.y && (landing === null || platform.y < landing)) landing = platform.y
    }
    return landing
  }

  function supported(platforms, x, halfWidth, feet) {
    if (feet >= GROUND - 0.5) return true
    return platforms.some((platform) => Math.abs(feet - platform.y) < 0.6 && x + halfWidth * 0.5 >= platform.x && x - halfWidth * 0.5 <= platform.x + platform.w)
  }

  /** Walkable span: the locked arena during a fight, otherwise this 幅's window of the scroll. */
  function bounds(state) {
    if (state.lock) return state.lock
    return [state.level.range[0] + 16, state.level.range[1] - 16]
  }

  const inZones = (zones, x) => Boolean(zones) && zones.some(([x0, x1]) => x >= x0 && x <= x1)
  function onIceZone(state, x) { return inZones(state.world.iceZones, x) }
  function onThinIce(state, x) { return inZones(state.world.thinIce, x) }
  function inReeds(state, x) { return inZones(state.world.reeds, x) }

  /** Gusts in 雪江卷: a calm spell, a one-second warning, then a push that alternates direction. Returns px/s at x. */
  function updateWind(state) {
    const wind = state.world.wind
    if (!wind) return
    const [calm, tele, blow] = wind.cycle
    const total = calm + tele + blow
    const cycle = Math.floor(state.clock / total)
    const t = state.clock - cycle * total
    const next = state.wind
    const dir = cycle % 2 === 0 ? 1 : -1
    let phase = 'calm', power = 0
    if (t >= calm + tele) {
      phase = 'blow'
      const into = t - calm - tele
      power = Math.min(1, into / 0.4, (blow - into) / 0.4)
    } else if (t >= calm) {
      phase = 'tele'
      power = (t - calm) / tele * 0.35
    }
    if (phase !== next.phase) {
      const inside = state.player.x >= wind.zone[0] && state.player.x <= wind.zone[1]
      if (phase === 'tele' && inside) { emit(state, 'gust', { dir, phase }); prompt(state, 'gust') }
      if (phase === 'calm' && next.phase === 'blow' && inside) emit(state, 'gustEnd', {})
    }
    state.wind = { phase, dir, power: Math.max(0, power), cycle }
  }

  function windSpeed(state, x) {
    const wind = state.world.wind
    if (!wind || state.wind.phase !== 'blow' || x < wind.zone[0] || x > wind.zone[1]) return 0
    return state.wind.dir * wind.force * state.wind.power
  }

  // ---------------------------------------------------------------- player

  function startSlash(state) {
    const player = state.player
    const air = !player.onGround
    const current = player.action?.type === 'slash' ? player.action : null
    let stage = current ? (current.stage + 1) % STAGES.length : state.clock - player.slashEndClock < 0.3 ? player.nextStage : 0
    if (air && stage === 2) stage = 0
    player.action = { type: 'slash', stage, t: 0, hits: [], air, ambush: player.hidden }
    player.slashStartClock = state.clock
    player.slashBuffer = 0
    if (air && player.vy > -80) player.vy = Math.min(player.vy, 40)
    emit(state, 'slash', { stage, air, x: player.x, y: player.y, facing: player.facing })
  }

  function startDash(state, move) {
    const player = state.player
    if (move) player.facing = sign(move)
    if (!player.onGround) player.airDash -= 1
    player.action = { type: 'dash', t: 0, dur: 0.19 }
    player.dashBuffer = 0
    player.dashCd = 0.46
    player.iframes = Math.max(player.iframes, 0.25)
    player.vy = 0
    emit(state, 'dash', { x: player.x, y: player.y, facing: player.facing })
  }

  function startStrike(state) {
    const player = state.player
    player.ink = 0
    player.inkBuffer = 0
    player.strikes += 1
    player.action = null
    player.iframes = Math.max(player.iframes, 1.4)
    state.strike = { t: 0, applied: false, x: player.x, facing: player.facing }
    emit(state, 'strike', { x: player.x, y: player.y, facing: player.facing })
  }

  function slashBox(player, stage) {
    const reach = STAGES[stage].reach
    const x0 = player.facing > 0 ? player.x - 12 : player.x - reach
    const x1 = player.facing > 0 ? player.x + reach : player.x + 12
    return { x0, x1, y0: player.y - player.h - 22, y1: player.y + 8 }
  }

  function enemyBox(enemy) {
    if (enemy.type === 'crow') return { x0: enemy.x - 16, x1: enemy.x + 16, y0: enemy.y - 14, y1: enemy.y + 14 }
    return { x0: enemy.x - enemy.w, x1: enemy.x + enemy.w, y0: enemy.y - enemy.h, y1: enemy.y }
  }

  function overlaps(a, b) { return a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0 }

  function playerBox(player) { return { x0: player.x - player.w, x1: player.x + player.w, y0: player.y - player.h, y1: player.y } }

  function updatePlayer(state, input, dt) {
    const player = state.player
    const move = clamp(Number(input.move) || 0, -1, 1)
    player.iframes = Math.max(0, player.iframes - dt)
    player.hurt = Math.max(0, player.hurt - dt)
    player.dashCd -= dt
    player.coyote -= dt
    player.comboTimer -= dt
    if (player.comboTimer <= 0) player.combo = 0

    if (player.dead) {
      player.deadT += dt
      player.vy = Math.min(900, player.vy + GRAVITY * dt)
      player.vx *= Math.pow(0.04, dt)
      integratePlayer(state, dt)
      return
    }

    let action = player.action
    if (action) action.t += dt

    const canAct = player.hurt <= 0 && (!action ||
      (action.type === 'slash' && action.t >= STAGES[action.stage].chain) ||
      (action.type === 'dash' && action.t >= action.dur * 0.8))
    const canDash = player.hurt <= 0 && player.dashCd <= 0 && (player.onGround || player.airDash > 0) &&
      (!action || action.type !== 'slash' || action.t >= STAGES[action.stage].a1)

    if (player.inkBuffer > 0 && player.ink >= 100 && !state.strike && player.hurt <= 0) {
      startStrike(state)
      return
    }
    if (player.dashBuffer > 0 && canDash) startDash(state, move)
    else if (player.slashBuffer > 0 && canAct) {
      if (move && (!action || action.type !== 'slash' || action.t >= STAGES[action.stage].dur * 0.9)) player.facing = sign(move)
      startSlash(state)
    }
    action = player.action

    // Jumping may cancel a finished slash swing but never the dash.
    const jumpAllowed = player.hurt <= 0 && (!action || (action.type === 'slash' && action.t >= STAGES[action.stage].a1))
    if (player.jumpBuffer > 0 && jumpAllowed) {
      if (player.onGround || player.coyote > 0) {
        player.vy = -660; player.onGround = false; player.coyote = 0; player.jumpBuffer = 0; player.jumpCut = false
        player.action = null
        emit(state, 'jump', { x: player.x, y: player.y, double: false })
      } else if (player.airJumps > 0) {
        player.vy = -585; player.airJumps -= 1; player.jumpBuffer = 0; player.jumpCut = false
        player.action = null
        emit(state, 'jump', { x: player.x, y: player.y, double: true })
      }
    }
    action = player.action
    if (!player.jumpHeld && player.vy < -280 && !player.jumpCut && player.airTime > 0.09) { player.vy = -280; player.jumpCut = true }

    if (action?.type === 'dash') {
      player.vx = player.facing * 640
      player.vy = 0
      const onIce = player.onGround && onIceZone(state, player.x)
      if (action.t >= action.dur) { player.action = null; player.vx = player.facing * RUN * (onIce ? 1.25 : 0.8) }
    } else if (action?.type === 'slash') {
      const stage = STAGES[action.stage]
      const onIce = player.onGround && onIceZone(state, player.x)
      if (action.air) {
        player.vx += (move * RUN + windSpeed(state, player.x) - player.vx) * Math.min(1, dt * 4)
      } else if (action.t < stage.a1) {
        player.vx = player.facing * stage.lunge * (1 - action.t / stage.a1 * 0.6)
      } else {
        player.vx *= Math.pow(onIce ? 0.08 : 0.0005, dt)
      }
      if (action.t >= stage.a0 && action.t <= stage.a1 + 0.02) slashHits(state, action)
      if (action.t >= stage.dur) {
        player.action = null
        player.slashEndClock = state.clock
        player.nextStage = (action.stage + 1) % STAGES.length
      }
    } else if (player.hurt <= 0) {
      const target = move * RUN + windSpeed(state, player.x) * (player.onGround ? 0.55 : 1)
      const onIce = player.onGround && onIceZone(state, player.x)
      const accel = onIce ? 680 : (player.onGround ? 2600 : 1700)
      const delta = target - player.vx
      player.vx += clamp(delta, -accel * dt, accel * dt)
      if (move) player.facing = sign(move)
    }

    if (!action || action.type !== 'dash' || !player.action) {
      const fall = player.vy > 0 ? 1.18 : 1
      const slashHang = player.action?.type === 'slash' && player.action.air && player.action.t < 0.16 ? 0.35 : 1
      player.vy = Math.min(900, player.vy + GRAVITY * fall * slashHang * dt)
    }
    integratePlayer(state, dt)
    player.ink = Math.min(100, player.ink)
  }

  function integratePlayer(state, dt) {
    const player = state.player
    const [min, max] = bounds(state)
    player.x = clamp(player.x + player.vx * dt, min + player.w, max - player.w)
    const previousFeet = player.y
    player.y += player.vy * dt
    if (player.vy >= 0) {
      const landing = platformUnder(state.world.platforms, player.x, player.w * 2, previousFeet, player.y)
      if (landing !== null) {
        if (!player.onGround) emit(state, 'land', { x: player.x, y: landing, impact: player.vy })
        player.y = landing; player.vy = 0; player.onGround = true; player.airJumps = 1; player.airDash = 1; player.airTime = 0
        if (player.action?.type === 'slash' && player.action.air) player.action.air = false
      } else if (player.onGround && !supported(state.world.platforms, player.x, player.w * 2, player.y)) {
        player.onGround = false; player.coyote = 0.1
      }
    }
    if (player.onGround && !supported(state.world.platforms, player.x, player.w * 2, player.y)) { player.onGround = false; player.coyote = 0.1 }
    if (!player.onGround) player.airTime += dt
  }

  function slashHits(state, action) {
    const player = state.player
    const stage = STAGES[action.stage]
    const box = slashBox(player, action.stage)
    for (const enemy of state.enemies) {
      if (!enemy.alive || enemy.delay > 0 || enemy.state === 'spawn' || action.hits.includes(enemy.id)) continue
      if (!overlaps(box, enemyBox(enemy))) continue
      action.hits.push(enemy.id)
      const unaware = action.ambush && enemy.type !== 'boss' && enemy.type !== 'angler' && (enemy.state === 'idle' || enemy.state === 'walk')
      const before = enemy.hp
      hitEnemy(state, enemy, stage.damage + (unaware ? 2 : 0), action.stage === 2 ? 'heavy' : 'slash')
      if (unaware && !action.ambushed && enemy.hp < before) {
        action.ambushed = true
        player.ambushes += 1
        addScore(state, 120)
        emit(state, 'ambush', { x: enemy.x, y: enemy.y - enemy.h / 2 })
      }
    }
    for (const ice of state.icicles) {
      if (ice.state === 'hang' && overlaps(box, iceBox(ice))) dropIcicle(state, ice)
    }
    for (const shot of state.projectiles) {
      if (shot.owner !== 'enemy' || shot.dead || action.hits.includes('p' + shot.id)) continue
      const shotBox = shot.type === 'wave'
        ? { x0: shot.x - 18, x1: shot.x + 18, y0: GROUND - 38, y1: GROUND }
        : { x0: shot.x - 10, x1: shot.x + 10, y0: shot.y - 6, y1: shot.y + 6 }
      if (!overlaps(box, shotBox)) continue
      action.hits.push('p' + shot.id)
      deflect(state, shot)
    }
  }

  function deflect(state, shot) {
    const player = state.player
    player.parries += 1
    addCombo(state)
    addScore(state, 50)
    addInk(state, (shot.type === 'wave' || shot.type === 'iceWave') ? 12 : 16)
    state.hitstop = Math.max(state.hitstop, 0.06)
    if (shot.type === 'wave' || shot.type === 'iceWave') {
      shot.dead = true
      emit(state, 'cut', { x: shot.x, y: GROUND - 20 })
      return
    }
    shot.owner = 'player'
    shot.vx = player.facing * Math.max(520, Math.abs(shot.vx) * 1.3)
    shot.vy = -40
    shot.life = 1.6
    emit(state, 'deflect', { x: shot.x, y: shot.y })
  }

  function hurtPlayer(state, damage, fromX) {
    const player = state.player
    player.hp = Math.max(0, player.hp - damage)
    player.damageTaken += damage
    player.iframes = 1
    player.hurt = 0.3
    player.action = null
    player.combo = 0
    player.comboTimer = 0
    const away = sign(player.x - fromX) || -player.facing
    player.vx = away * 300
    player.vy = -330
    player.onGround = false
    state.hitstop = Math.max(state.hitstop, 0.1)
    emit(state, 'hurt', { x: player.x, y: player.y - 30, hp: player.hp, dir: away })
    if (player.hp <= 0) {
      player.dead = true
      state.slow = 1.1; state.slowScale = 0.32
      emit(state, 'death', { x: player.x, y: player.y })
    }
  }

  // Resolve one enemy strike reaching the player: parried, evaded by i-frames, or taken.
  function enemyStrike(state, attacker, damage, kind = 'melee') {
    const player = state.player
    if (player.dead) return 'miss'
    const action = player.action
    const facingAttacker = sign(attacker.x - player.x) === player.facing || Math.abs(attacker.x - player.x) < 10
    const inWindow = action?.type === 'slash' && state.clock - player.slashStartClock <= state.diff.parry
    if (kind !== 'unblockable' && inWindow && facingAttacker) {
      parry(state, attacker)
      return 'parry'
    }
    if (player.iframes > 0 || state.strike) return 'miss'
    hurtPlayer(state, damage, attacker.x)
    return 'hit'
  }

  function parry(state, attacker) {
    const player = state.player
    player.parries += 1
    addCombo(state)
    addScore(state, 50)
    addInk(state, 20)
    player.iframes = Math.max(player.iframes, 0.22)
    player.vx = -player.facing * 90
    state.hitstop = Math.max(state.hitstop, 0.09)
    state.slow = 0.42; state.slowScale = 0.28
    const x = (attacker.x + player.x) / 2
    emit(state, 'parry', { x, y: player.y - 38, target: attacker.type, id: attacker.id })
    if (attacker.type === 'crow') { kill(state, attacker, 'parry'); return }
    if (attacker.type === 'boss') {
      attacker.posture = Math.min(attacker.postureMax, attacker.posture + 22)
      attacker.postureClock = state.clock
      if (attacker.posture >= attacker.postureMax) breakBoss(state, attacker)
      return
    }
    attacker.state = 'stagger'
    attacker.openedAt = state.clock
    attacker.t = attacker.type === 'shield' ? 1.8 : 1.3
    attacker.vx = sign(attacker.x - player.x) * 170
    emit(state, 'stagger', { id: attacker.id, x: attacker.x, y: attacker.y - attacker.h })
  }

  function breakBoss(state, boss) {
    boss.state = 'broken'
    boss.openedAt = state.clock
    boss.t = 2.7
    boss.vx = 0
    boss.combo = null
    state.slow = 0.6; state.slowScale = 0.3
    emit(state, 'broken', { id: boss.id, x: boss.x, y: boss.y - boss.h })
  }

  function hitEnemy(state, enemy, damage, source) {
    const player = state.player
    if (!enemy.alive || enemy.state === 'spawn' || enemy.state === 'dying') return
    const fromFront = sign(player.x - enemy.x) === enemy.facing
    if (enemy.type === 'shield' && enemy.state !== 'stagger' && fromFront && source !== 'strike' && source !== 'icicle' && source !== 'crack') {
      state.hitstop = Math.max(state.hitstop, 0.05)
      if (source !== 'arrow') player.vx = -player.facing * 150
      emit(state, 'block', { x: enemy.x + enemy.facing * 18, y: enemy.y - 34 })
      return
    }
    const melee = source === 'slash' || source === 'heavy'
    // Only a swing begun after the opening can execute; the parrying swing itself never does.
    const fresh = melee && player.slashStartClock > (enemy.openedAt ?? Infinity)
    if (enemy.type === 'boss' || enemy.type === 'angler') {
      if (enemy.state === 'broken' && fresh) {
        enemy.hp -= 10
        enemy.posture = 0
        enemy.state = 'recover'; enemy.t = 1
        enemy.vx = sign(enemy.x - player.x) * 260
        player.executions += 1
        player.bossExecutions += 1
        addCombo(state); addScore(state, 300); addInk(state, 15)
        state.hitstop = Math.max(state.hitstop, 0.16); state.slow = 0.7; state.slowScale = 0.25
        emit(state, 'execute', { id: enemy.id, x: enemy.x, y: enemy.y - 40, boss: true })
      } else {
        enemy.hp -= source === 'strike' ? 6 : damage
        enemy.posture = Math.min(enemy.postureMax, enemy.posture + (source === 'strike' ? 30 : source === 'heavy' ? 6 : 3))
        // Pressing a guarded boss provokes a fast counter; the punish window is after its own attacks.
        if (melee && (enemy.state === 'idle' || enemy.state === 'walk')) {
          enemy.pressure = (enemy.pressure || 0) + 1
          if (enemy.pressure >= 2 && enemy.hp > damage) {
            enemy.pressure = 0
            enemy.facing = sign(player.x - enemy.x) || enemy.facing
            startCombo(state, enemy)
            enemy.t *= 0.8; enemy.teleTotal = enemy.t
          }
        }
        enemy.postureClock = state.clock
        enemy.flash = 0.12
        addCombo(state); addScore(state, 10 * damage); addInk(state, 5)
        state.hitstop = Math.max(state.hitstop, source === 'heavy' ? 0.08 : 0.045)
        emit(state, 'hit', { id: enemy.id, x: enemy.x, y: enemy.y - 40, dir: player.facing, heavy: source === 'heavy' })
        if (enemy.hp > 0 && enemy.posture >= enemy.postureMax && enemy.state !== 'broken') breakBoss(state, enemy)
      }
      if (enemy.hp <= 0) {
        enemy.hp = 0
        enemy.state = 'dying'; enemy.t = 2.6; enemy.vx = 0
        state.slow = 1.6; state.slowScale = 0.22
        killScore(state, enemy)
        emit(state, 'bossDown', { id: enemy.id, x: enemy.x, y: enemy.y - 40 })
      } else if (!enemy.phase2 && enemy.hp <= enemy.maxHp * 0.5) {
        enemy.phase2 = true
        enemy.state = 'roar'; enemy.t = 1.25; enemy.vx = 0; enemy.combo = null
        emit(state, 'bossPhase', { id: enemy.id, x: enemy.x, y: enemy.y - 40 })
      }
      return
    }
    if (enemy.state === 'stagger' && fresh) {
      player.executions += 1
      state.hitstop = Math.max(state.hitstop, 0.13); state.slow = 0.45; state.slowScale = 0.3
      addCombo(state); addScore(state, 150); addInk(state, 15)
      emit(state, 'execute', { id: enemy.id, x: enemy.x, y: enemy.y - enemy.h / 2, boss: false })
      kill(state, enemy, 'execute')
      return
    }
    enemy.hp -= source === 'strike' ? 4 : source === 'arrow' ? 3 : damage
    enemy.flash = 0.12
    addCombo(state); addScore(state, 10 * damage); addInk(state, 6)
    state.hitstop = Math.max(state.hitstop, source === 'heavy' ? 0.085 : 0.05)
    emit(state, 'hit', { id: enemy.id, x: enemy.x, y: enemy.type === 'crow' ? enemy.y : enemy.y - enemy.h / 2, dir: source === 'arrow' ? sign(enemy.x - player.x) || 1 : player.facing, heavy: source === 'heavy' })
    if (enemy.hp <= 0) { kill(state, enemy, source); return }
    // Late in a telegraph the strike is committed: hits land but do not interrupt, so a slightly early
    // press still meets the blade inside the parry window.
    const committed = enemy.state === 'attack' || (enemy.state === 'tele' && enemy.t < enemy.teleTotal * 0.55)
    if (enemy.type !== 'crow' && !committed) {
      enemy.state = 'hurt'
      enemy.t = enemy.type === 'shield' ? 0.22 : 0.3
      enemy.vx = (sign(enemy.x - player.x) || 1) * (source === 'heavy' ? 260 : 170)
    }
  }

  function killScore(state, enemy) {
    const player = state.player
    player.kills += 1
    addScore(state, ENEMY[enemy.type].score)
    addInk(state, 8)
  }

  function kill(state, enemy, how) {
    const player = state.player
    enemy.alive = false
    enemy.state = 'dead'
    killScore(state, enemy)
    if (how === 'arrow') player.deflectKills += 1
    else if (how === 'icicle') player.icicleKills += 1
    else if (how === 'crack') player.crackKills += 1
    else if (how !== 'strike') {
      if (!player.onGround) player.airKills += 1
      else if (onIceZone(state, player.x) && Math.abs(player.vx) >= 70) player.slideKills += 1
    }
    emit(state, 'kill', { id: enemy.id, kind: enemy.type, x: enemy.x, y: enemy.y, facing: enemy.facing, how })
  }

  // --------------------------------------------------------------- enemies

  function spawnEnemy(state, spec) {
    const lock = state.lock
    const kind = ENEMY[spec.type]
    const side = spec.side || 1
    const x = spec.x ?? (side > 0 ? lock[1] - 46 : lock[0] + 46)
    const crow = spec.type === 'crow'
    const enemy = {
      id: state.nextId++, type: spec.type, alive: true, delay: spec.delay || 0,
      x, y: crow ? 150 + state.random() * 30 : spec.y ?? GROUND, vx: 0, vy: 0, w: kind.w, h: kind.h,
      homeY: spec.y ?? GROUND, onPlatform: spec.y !== undefined && spec.y < GROUND,
      facing: sign(state.player.x - x) || -1, hp: kind.hp, maxHp: kind.hp,
      state: 'spawn', t: spec.type === 'boss' ? 0.9 : 0.75, cd: 0.4 + state.random() * 0.8, hopCd: 0, flash: 0,
      hitDone: false, side, seed: state.random() * 10, combo: null, move: null,
    }
    if (spec.type === 'boss' || spec.type === 'angler') {
      enemy.hp = enemy.maxHp = spec.type === 'angler' ? (state.difficulty === 'easy' ? 36 : 50) : state.diff.bossHp
      enemy.posture = 0; enemy.postureMax = 100; enemy.postureClock = 0; enemy.phase2 = false
      enemy.moves = 0
    }
    if (spec.type === 'shield') prompt(state, 'shield')
    state.enemies.push(enemy)
    if (!enemy.delay) emit(state, 'spawn', { id: enemy.id, kind: enemy.type, x: enemy.x, y: enemy.y })
    return enemy
  }

  function attackers(state) {
    let count = 0
    for (const enemy of state.enemies) {
      if (enemy.alive && enemy.type !== 'boss' && enemy.type !== 'angler' && (enemy.state === 'tele' || enemy.state === 'attack' || enemy.state === 'dive')) count += 1
    }
    return count
  }

  /** Cover from 芦丛 stops new attacks, so ranged and waiting enemies lose the player until they are spotted. */
  function mayAttack(state) { return attackers(state) < state.diff.tokens && !state.player.hidden }

  const WIND_MASS = { hammer: 0.15, spearman: 0.4, shield: 0.3 }

  function moveGround(state, enemy, dt) {
    const [min, max] = state.lock ? state.lock : [0, state.world.length]
    const braced = enemy.state === 'tele' || enemy.state === 'attack'
    const push = braced || enemy.onPlatform ? 0 : windSpeed(state, enemy.x) * (WIND_MASS[enemy.type] ?? 0.5)
    enemy.x = clamp(enemy.x + (enemy.vx + push) * dt, min + enemy.w, max - enemy.w)
    if (enemy.y < enemy.homeY || enemy.vy < 0) {
      enemy.vy = Math.min(900, enemy.vy + GRAVITY * dt)
      const previous = enemy.y
      enemy.y += enemy.vy * dt
      const landing = enemy.onPlatform ? (enemy.y >= enemy.homeY && previous <= enemy.homeY + 1 ? enemy.homeY : null) : (enemy.y >= GROUND ? GROUND : null)
      if (landing !== null) { enemy.y = landing; enemy.vy = 0 }
    }
  }

  function telegraph(state, enemy, duration, unblockable = false) {
    enemy.state = 'tele'
    enemy.t = duration * state.diff.tele
    enemy.teleTotal = enemy.t
    emit(state, unblockable ? 'danger' : 'tell', { id: enemy.id, x: enemy.x, y: enemy.type === 'crow' ? enemy.y : enemy.y - enemy.h, facing: enemy.facing })
  }

  function crowded(state, enemy) {
    for (const other of state.enemies) {
      if (other === enemy || !other.alive || other.type === 'crow' || other.delay > 0) continue
      const ahead = (other.x - enemy.x) * enemy.facing
      if (ahead > 0 && ahead < 36 && Math.abs(other.y - enemy.y) < 30) return true
    }
    return false
  }

  function updateMelee(state, enemy, dt) {
    const player = state.player
    const kind = ENEMY[enemy.type]
    const dx = player.x - enemy.x
    const distance = Math.abs(dx)
    switch (enemy.state) {
      case 'idle':
      case 'walk': {
        if (player.dead) { enemy.vx *= 0.8; break }
        // The shield bearer turns slowly, so a dash through it opens its back.
        if (enemy.type === 'shield' && sign(dx) && sign(dx) !== enemy.facing) {
          enemy.turn = (enemy.turn || 0) + dt
          if (enemy.turn > 0.5) { enemy.facing = sign(dx); enemy.turn = 0 }
        } else {
          enemy.turn = 0
          enemy.facing = sign(dx) || enemy.facing
        }
        const facingPlayer = sign(dx) === enemy.facing || distance < 8
        if (distance > kind.reach * 0.85) {
          enemy.state = 'walk'
          const speed = kind.speed * (crowded(state, enemy) ? 0.15 : 1)
          enemy.vx += (enemy.facing * speed - enemy.vx) * Math.min(1, dt * 8)
        } else {
          enemy.state = 'idle'
          enemy.vx *= Math.pow(0.02, dt)
          if (distance < 34) enemy.vx = -enemy.facing * 60
          if (enemy.cd <= 0 && facingPlayer && mayAttack(state)) {
            telegraph(state, enemy, kind.tele)
            prompt(state, 'parry')
          }
        }
        break
      }
      case 'tele':
        enemy.vx *= Math.pow(0.001, dt)
        if (enemy.t <= 0) {
          enemy.state = 'attack'; enemy.t = kind.active; enemy.hitDone = false
          enemy.vx = enemy.facing * kind.lunge
          emit(state, 'enemySwing', { id: enemy.id, x: enemy.x, y: enemy.y, facing: enemy.facing, kind: enemy.type })
        }
        break
      case 'attack': {
        if (!enemy.hitDone) {
          const reachBox = enemy.facing > 0
            ? { x0: enemy.x, x1: enemy.x + kind.reach, y0: enemy.y - enemy.h - 8, y1: enemy.y + 4 }
            : { x0: enemy.x - kind.reach, x1: enemy.x, y0: enemy.y - enemy.h - 8, y1: enemy.y + 4 }
          if (overlaps(reachBox, playerBox(player))) {
            enemy.hitDone = true
            const outcome = enemyStrike(state, enemy, kind.damage)
            if (outcome === 'parry') break
          }
        }
        enemy.vx *= Math.pow(0.02, dt)
        if (enemy.t <= 0) { enemy.state = 'recover'; enemy.t = kind.recover }
        break
      }
      case 'recover':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = (0.7 + state.random() * 1.1) / state.diff.aggression }
        break
      case 'hurt':
        enemy.vx *= Math.pow(0.004, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = Math.max(enemy.cd, 0.35) }
        break
      case 'stagger':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.5 }
        break
    }
    moveGround(state, enemy, dt)
  }

  function updateDummy(state, enemy) {
    if (enemy.state === 'hurt' && enemy.t <= 0) enemy.state = 'idle'
    enemy.vx = 0
  }

  function updateArcher(state, enemy, dt) {
    const player = state.player
    const dx = player.x - enemy.x
    enemy.hopCd -= dt
    switch (enemy.state) {
      case 'idle':
      case 'walk':
        if (!player.dead) enemy.facing = sign(dx) || enemy.facing
        enemy.vx *= Math.pow(0.01, dt)
        if (player.dead) break
        if (!enemy.onPlatform && Math.abs(dx) < 120 && enemy.hopCd <= 0 && enemy.y >= GROUND - 0.5) {
          enemy.state = 'hop'; enemy.vx = -enemy.facing * 290; enemy.vy = -440; enemy.hopCd = 2
          emit(state, 'hop', { id: enemy.id, x: enemy.x, y: enemy.y })
        } else if (enemy.cd <= 0 && Math.abs(dx) < 640 && !player.hidden) {
          telegraph(state, enemy, 0.8)
        }
        break
      case 'hop':
        if (enemy.vy === 0 && enemy.y >= enemy.homeY - 0.5) { enemy.state = 'idle'; enemy.vx = 0; enemy.cd = Math.max(enemy.cd, 0.4) }
        break
      case 'tele':
        enemy.vx = 0
        if (enemy.t <= 0) { shootArrow(state, enemy); enemy.state = 'recover'; enemy.t = 0.5 }
        break
      case 'recover':
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = (1.5 + state.random() * 1.1) / state.diff.aggression }
        break
      case 'hurt':
        enemy.vx *= Math.pow(0.004, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = Math.max(enemy.cd, 0.5) }
        break
    }
    moveGround(state, enemy, dt)
  }

  function shootArrow(state, enemy) {
    const player = state.player
    const sx = enemy.x + enemy.facing * 16, sy = enemy.y - 36
    const tx = player.x, ty = player.y - 30
    const vx = enemy.facing * 470
    const flight = Math.max(0.15, Math.abs(tx - sx) / 470)
    const vy = clamp((ty - sy) / flight - 0.5 * ARROW_GRAVITY * flight, -320, 320)
    state.projectiles.push({ id: state.nextId++, type: 'arrow', owner: 'enemy', x: sx, y: sy, vx, vy, life: 2.6, source: enemy.id, dead: false })
    emit(state, 'shoot', { id: enemy.id, x: sx, y: sy })
    prompt(state, 'deflect')
  }

  function updateCrow(state, enemy, dt) {
    const player = state.player
    const approach = (tx, ty, speed, rate) => {
      enemy.vx += (clamp((tx - enemy.x) * 2.2, -speed, speed) - enemy.vx) * Math.min(1, dt * rate)
      enemy.vy += (clamp((ty - enemy.y) * 2.2, -speed, speed) - enemy.vy) * Math.min(1, dt * rate)
    }
    switch (enemy.state) {
      case 'idle':
      case 'walk': {
        const ax = player.x + enemy.side * 135
        const ay = player.y - 150 + Math.sin(state.clock * 2.3 + enemy.seed) * 18
        approach(ax, ay, 175, 3)
        enemy.facing = sign(player.x - enemy.x) || enemy.facing
        if (!player.dead && enemy.cd <= 0 && mayAttack(state)) telegraph(state, enemy, 0.48)
        break
      }
      case 'tele':
        enemy.vx *= Math.pow(0.01, dt); enemy.vy *= Math.pow(0.01, dt)
        enemy.facing = sign(player.x - enemy.x) || enemy.facing
        if (enemy.t <= 0) {
          const dx = player.x - enemy.x, dy = player.y - 26 - enemy.y
          const length = Math.hypot(dx, dy) || 1
          enemy.vx = dx / length * 450; enemy.vy = dy / length * 450
          enemy.state = 'dive'; enemy.t = 0.95; enemy.hitDone = false
          emit(state, 'caw', { id: enemy.id, x: enemy.x, y: enemy.y })
        }
        break
      case 'dive':
        if (!enemy.hitDone && overlaps(enemyBox(enemy), playerBox(player))) {
          enemy.hitDone = true
          if (enemyStrike(state, enemy, 1) === 'parry') return
        }
        if (enemy.t <= 0 || enemy.y > GROUND - 14 || (enemy.hitDone && enemy.t < 0.75)) { enemy.state = 'rise'; enemy.t = 1.2 }
        break
      case 'rise':
        approach(enemy.x + enemy.facing * 40, player.y - 160, 230, 4)
        if (enemy.y < player.y - 115 || enemy.t <= 0) {
          enemy.state = 'idle'; enemy.side = -enemy.side
          enemy.cd = (1.2 + state.random() * 1.4) / state.diff.aggression
        }
        break
    }
    const [min, max] = state.lock ? state.lock : [0, state.world.length]
    enemy.x = clamp(enemy.x + enemy.vx * dt, min + 10, max - 10)
    enemy.y = clamp(enemy.y + enemy.vy * dt, 40, GROUND - 12)
  }

  function shootDart(state, enemy) {
    const sx = enemy.x + enemy.facing * 16, sy = enemy.y - 28
    const vx = enemy.facing * 520
    state.projectiles.push({ id: state.nextId++, type: 'dart', owner: 'enemy', x: sx, y: sy, vx, vy: 0, life: 1.8, source: enemy.id, dead: false })
    emit(state, 'dart', { id: enemy.id, x: sx, y: sy })
  }

  function spawnIceWave(state, boss, direction, speed, life = 1.2) {
    state.projectiles.push({ id: state.nextId++, type: 'iceWave', owner: 'enemy', x: boss.x + direction * 28, y: GROUND, vx: direction * speed, vy: 0, life, source: boss.id, dead: false })
  }

  function updateSpearman(state, enemy, dt) {
    const player = state.player
    const dx = player.x - enemy.x
    const distance = Math.abs(dx)
    switch (enemy.state) {
      case 'idle':
      case 'walk':
        if (!player.dead) enemy.facing = sign(dx) || enemy.facing
        if (player.dead) { enemy.vx *= 0.8; break }
        if (distance > 130 * 0.85) {
          enemy.state = 'walk'
          enemy.vx += (enemy.facing * 78 - enemy.vx) * Math.min(1, dt * 8)
        } else {
          enemy.state = 'idle'
          enemy.vx *= Math.pow(0.02, dt)
          if (enemy.cd <= 0 && mayAttack(state)) {
            enemy.facing = sign(dx) || enemy.facing
            const unblockable = state.random() < 0.6
            enemy.move = unblockable ? 'thrust' : 'sweep'
            telegraph(state, enemy, unblockable ? 0.54 : 0.48, unblockable)
          }
        }
        break
      case 'tele':
        enemy.vx *= Math.pow(0.001, dt)
        if (enemy.t <= 0) {
          enemy.state = 'attack'; enemy.t = 0.14; enemy.hitDone = false
          enemy.vx = enemy.facing * (enemy.move === 'thrust' ? 280 : 180)
          emit(state, 'enemySwing', { id: enemy.id, x: enemy.x, y: enemy.y, facing: enemy.facing, kind: enemy.type })
        }
        break
      case 'attack':
        if (!enemy.hitDone) {
          const reach = enemy.move === 'thrust' ? 140 : 115
          const reachBox = enemy.facing > 0
            ? { x0: enemy.x, x1: enemy.x + reach, y0: enemy.y - enemy.h - 8, y1: enemy.y + 4 }
            : { x0: enemy.x - reach, x1: enemy.x, y0: enemy.y - enemy.h - 8, y1: enemy.y + 4 }
          if (overlaps(reachBox, playerBox(player))) {
            enemy.hitDone = true
            const outcome = enemyStrike(state, enemy, 1, enemy.move === 'thrust' ? 'unblockable' : 'melee')
            if (outcome === 'parry') break
          }
        }
        enemy.vx *= Math.pow(0.02, dt)
        if (enemy.t <= 0) { enemy.state = 'recover'; enemy.t = 0.65 }
        break
      case 'recover':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = (0.8 + state.random() * 1.0) / state.diff.aggression }
        break
      case 'hurt':
        enemy.vx *= Math.pow(0.004, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.4 }
        break
      case 'stagger':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.5 }
        break
    }
    moveGround(state, enemy, dt)
  }

  function updateRonin(state, enemy, dt) {
    const player = state.player
    const dx = player.x - enemy.x
    const distance = Math.abs(dx)
    switch (enemy.state) {
      case 'idle':
      case 'walk':
        if (!player.dead) enemy.facing = sign(dx) || enemy.facing
        if (player.dead) { enemy.vx *= 0.8; break }
        if (distance > 135 && distance < 220 && enemy.cd <= 0 && state.random() < 0.45 && mayAttack(state)) {
          shootDart(state, enemy)
          enemy.cd = 1.6 / state.diff.aggression
          enemy.vx = -enemy.facing * 180
          break
        }
        if (distance > 85 * 0.85) {
          enemy.state = 'walk'
          enemy.vx += (enemy.facing * 130 - enemy.vx) * Math.min(1, dt * 8)
        } else {
          enemy.state = 'idle'
          enemy.vx *= Math.pow(0.02, dt)
          if (enemy.cd <= 0 && mayAttack(state)) {
            enemy.facing = sign(dx) || enemy.facing
            telegraph(state, enemy, 0.44)
          }
        }
        break
      case 'tele':
        enemy.vx *= Math.pow(0.001, dt)
        if (enemy.t <= 0) {
          enemy.state = 'attack'; enemy.t = 0.12; enemy.hitDone = false
          enemy.vx = enemy.facing * 360
          emit(state, 'enemySwing', { id: enemy.id, x: enemy.x, y: enemy.y, facing: enemy.facing, kind: enemy.type })
        }
        break
      case 'attack':
        if (!enemy.hitDone) {
          const reachBox = enemy.facing > 0
            ? { x0: enemy.x, x1: enemy.x + 85, y0: enemy.y - enemy.h - 8, y1: enemy.y + 4 }
            : { x0: enemy.x - 85, x1: enemy.x, y0: enemy.y - enemy.h - 8, y1: enemy.y + 4 }
          if (overlaps(reachBox, playerBox(player))) {
            enemy.hitDone = true
            const outcome = enemyStrike(state, enemy, 1)
            if (outcome === 'parry') break
          }
        }
        enemy.vx *= Math.pow(0.02, dt)
        if (enemy.t <= 0) { enemy.state = 'recover'; enemy.t = 0.48 }
        break
      case 'recover':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = (0.6 + state.random() * 0.9) / state.diff.aggression }
        break
      case 'hurt':
        enemy.vx *= Math.pow(0.004, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.35 }
        break
      case 'stagger':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.5 }
        break
    }
    moveGround(state, enemy, dt)
  }

  function updateHammer(state, enemy, dt) {
    const player = state.player
    const dx = player.x - enemy.x
    const distance = Math.abs(dx)
    switch (enemy.state) {
      case 'idle':
      case 'walk':
        if (!player.dead) enemy.facing = sign(dx) || enemy.facing
        if (player.dead) { enemy.vx *= 0.8; break }
        if (distance > 105 * 0.85) {
          enemy.state = 'walk'
          enemy.vx += (enemy.facing * 52 - enemy.vx) * Math.min(1, dt * 6)
        } else {
          enemy.state = 'idle'
          enemy.vx *= Math.pow(0.02, dt)
          if (enemy.cd <= 0 && mayAttack(state)) {
            enemy.facing = sign(dx) || enemy.facing
            const isSlam = state.random() < 0.5
            enemy.move = isSlam ? 'slam' : 'swing'
            telegraph(state, enemy, isSlam ? 0.72 : 0.65, isSlam)
          }
        }
        break
      case 'tele':
        enemy.vx *= Math.pow(0.001, dt)
        if (enemy.t <= 0) {
          enemy.state = 'attack'; enemy.t = 0.18; enemy.hitDone = false
          enemy.vx = enemy.facing * (enemy.move === 'slam' ? 120 : 190)
          emit(state, 'enemySwing', { id: enemy.id, x: enemy.x, y: enemy.y, facing: enemy.facing, kind: enemy.type })
          if (enemy.move === 'slam') emit(state, 'slam', { id: enemy.id, x: enemy.x, y: GROUND })
        }
        break
      case 'attack':
        if (!enemy.hitDone) {
          const reach = enemy.move === 'slam' ? 95 : 105
          const reachBox = enemy.facing > 0
            ? { x0: enemy.x, x1: enemy.x + reach, y0: GROUND - 48, y1: GROUND + 4 }
            : { x0: enemy.x - reach, x1: enemy.x, y0: GROUND - 48, y1: GROUND + 4 }
          if (overlaps(reachBox, playerBox(player))) {
            enemy.hitDone = true
            const outcome = enemyStrike(state, enemy, 1, enemy.move === 'slam' ? 'unblockable' : 'melee')
            if (outcome === 'parry') break
          }
        }
        enemy.vx *= Math.pow(0.02, dt)
        if (enemy.t <= 0) { enemy.state = 'recover'; enemy.t = 0.85 }
        break
      case 'recover':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = (0.9 + state.random() * 1.2) / state.diff.aggression }
        break
      case 'hurt':
        enemy.vx *= Math.pow(0.004, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.45 }
        break
      case 'stagger':
        enemy.vx *= Math.pow(0.01, dt)
        if (enemy.t <= 0) { enemy.state = 'idle'; enemy.cd = 0.5 }
        break
    }
    moveGround(state, enemy, dt)
  }

  function chooseAnglerMove(state, boss) {
    const player = state.player
    const distance = Math.abs(player.x - boss.x)
    const roll = state.random()
    boss.moves += 1
    if (boss.phase2 && boss.moves % 4 === 0) return startAnglerPhantom(state, boss)
    if (distance > 220) {
      if (roll < 0.42) return startAnglerIceSlam(state, boss)
      if (roll < 0.78) return startAnglerHook(state, boss)
      boss.state = 'walk'; boss.t = 1.1
      return
    }
    if (roll < 0.6) return startAnglerCombo(state, boss)
    if (roll < 0.82) return startAnglerIceSlam(state, boss)
    boss.state = 'backstep'; boss.t = 0.35
    boss.vx = -boss.facing * 390; boss.vy = -340
  }

  function startAnglerCombo(state, boss) {
    boss.state = 'combo'
    boss.combo = { i: 0, phase: 'tele' }
    boss.t = (boss.phase2 ? 0.34 : 0.42) * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'tell', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function startAnglerHook(state, boss) {
    boss.state = 'hookTell'
    boss.t = 0.55 * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'danger', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function startAnglerIceSlam(state, boss) {
    boss.state = 'slamTell'
    boss.t = 0.54 * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'tell', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function startAnglerPhantom(state, boss) {
    const player = state.player
    const [min, max] = state.lock
    const middle = (min + max) / 2
    emit(state, 'vanish', { id: boss.id, x: boss.x, y: boss.y - 36 })
    boss.x = player.x < middle ? max - 60 : min + 60
    boss.y = GROUND; boss.vy = 0; boss.vx = 0
    boss.facing = sign(player.x - boss.x) || -1
    emit(state, 'appear', { id: boss.id, x: boss.x, y: boss.y - 36 })
    boss.state = 'phantomTell'
    boss.t = 0.88 * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'danger', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function updateAngler(state, boss, dt) {
    const player = state.player
    const dx = player.x - boss.x
    const faster = boss.phase2 ? 0.82 : 1
    if (boss.state !== 'broken' && state.clock - boss.postureClock > 2) boss.posture = Math.max(0, boss.posture - 7 * dt)
    const facePlayer = () => { if (!player.dead) boss.facing = sign(dx) || boss.facing }
    switch (boss.state) {
      case 'intro':
        boss.vx = 0
        if (boss.t <= 0) { boss.state = 'idle'; boss.t = 0.4 }
        break
      case 'idle':
        facePlayer()
        boss.vx *= Math.pow(0.01, dt)
        if (boss.t <= 0 && !player.dead) chooseAnglerMove(state, boss)
        break
      case 'walk':
        facePlayer()
        boss.vx += (boss.facing * 140 - boss.vx) * Math.min(1, dt * 6)
        if (Math.abs(dx) < 120 || boss.t <= 0) startAnglerCombo(state, boss)
        break
      case 'backstep':
        if (boss.t <= 0 && boss.y >= GROUND - 0.5) { boss.vx = 0; startAnglerIceSlam(state, boss) }
        break
      case 'combo': {
        const combo = boss.combo
        if (combo.phase === 'tele') {
          boss.vx *= Math.pow(0.001, dt)
          if (boss.t <= 0) {
            combo.phase = 'active'; boss.t = 0.13; boss.hitDone = false
            boss.vx = boss.facing * 260
            emit(state, 'enemySwing', { id: boss.id, x: boss.x, y: boss.y, facing: boss.facing, kind: 'angler', step: combo.i })
          }
        } else if (combo.phase === 'active') {
          if (!boss.hitDone && overlaps(bossReachBox(boss, 115), playerBox(player))) {
            boss.hitDone = true
            enemyStrike(state, boss, 1)
            if (boss.state !== 'combo') break
          }
          boss.vx *= Math.pow(0.02, dt)
          if (boss.t <= 0) {
            combo.i += 1
            const maxSteps = boss.phase2 ? 3 : 2
            if (combo.i >= maxSteps) { boss.state = 'recover'; boss.t = 0.75; boss.combo = null }
            else {
              combo.phase = 'tele'; facePlayer()
              boss.t = (combo.i === 2 ? 0.44 : 0.32) * faster * state.diff.tele
              boss.teleTotal = boss.t
              emit(state, 'tell', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
            }
          }
        }
        break
      }
      case 'hookTell':
        boss.vx = 0
        if (boss.t <= 0) {
          facePlayer()
          boss.state = 'hook'; boss.t = 0.14
          state.projectiles.push({ id: state.nextId++, type: 'hook', owner: 'enemy', x: boss.x + boss.facing * 24, y: boss.y - 30, vx: boss.facing * 750, vy: 0, life: 0.6, source: boss.id, dead: false })
          emit(state, 'hook', { id: boss.id, x: boss.x, y: boss.y - 30 })
        }
        break
      case 'hook':
        boss.vx *= Math.pow(0.02, dt)
        if (boss.t <= 0) { boss.state = 'recover'; boss.t = 0.7 }
        break
      case 'slamTell':
        boss.vx = 0
        if (boss.t <= 0) {
          boss.state = 'slam'; boss.t = 0.15
          emit(state, 'slam', { id: boss.id, x: boss.x, y: GROUND })
          spawnIceWave(state, boss, -1, 380, 1.2)
          spawnIceWave(state, boss, 1, 380, 1.2)
        }
        break
      case 'slam':
        boss.vx = 0
        if (boss.t <= 0) { boss.state = 'recover'; boss.t = 0.8 }
        break
      case 'phantomTell':
        boss.vx = 0
        if (boss.t <= 0) {
          boss.state = 'phantom'; boss.hitDone = false; boss.t = 1.1
          boss.vx = boss.facing * 1100
          emit(state, 'phantom', { id: boss.id, x: boss.x, y: boss.y, facing: boss.facing })
        }
        break
      case 'phantom': {
        const [min, max] = state.lock
        if (!boss.hitDone && overlaps({ x0: boss.x - 24, x1: boss.x + 24, y0: GROUND - 58, y1: GROUND }, playerBox(player))) {
          boss.hitDone = true
          enemyStrike(state, boss, 1, 'unblockable')
        }
        if ((boss.facing > 0 && boss.x >= max - 62) || (boss.facing < 0 && boss.x <= min + 62) || boss.t <= 0) {
          boss.vx = 0; boss.state = 'recover'; boss.t = 0.85
        }
        break
      }
      case 'roar':
        boss.vx = 0
        if (Math.abs(dx) < 150 && !player.dead && boss.t > 1.1) { player.vx = sign(dx) * 360; player.vy = Math.min(player.vy, -200); player.onGround = false }
        if (boss.t <= 0) { boss.state = 'idle'; boss.t = 0.3 }
        break
      case 'broken':
        boss.vx = 0
        if (boss.t <= 0) { boss.state = 'recover'; boss.t = 0.3; boss.posture = 40 }
        break
      case 'recover':
        boss.vx *= Math.pow(0.01, dt)
        if (boss.t <= 0) { boss.state = 'idle'; boss.t = (0.42 + state.random() * 0.4) * (boss.phase2 ? 0.78 : 1) / state.diff.aggression }
        break
      case 'dying':
        boss.vx = 0
        break
    }
    const [min, max] = state.lock || [0, state.world.length]
    boss.x = clamp(boss.x + boss.vx * dt, min + boss.w, max - boss.w)
    if (boss.y < GROUND || boss.vy < 0) {
      boss.vy = Math.min(1100, boss.vy + GRAVITY * dt)
      boss.y = Math.min(GROUND, boss.y + boss.vy * dt)
      if (boss.y >= GROUND) boss.vy = 0
    }
  }

  // ------------------------------------------------------------------ boss

  function chooseBossMove(state, boss) {
    const player = state.player
    const distance = Math.abs(player.x - boss.x)
    const roll = state.random()
    boss.moves += 1
    if (boss.phase2 && boss.moves % 4 === 0) return startPhantom(state, boss)
    if (distance > 230) {
      if (roll < 0.42) return startWave(state, boss)
      if (roll < 0.78) return startLeap(state, boss)
      boss.state = 'walk'; boss.t = 1.2
      return
    }
    if (roll < 0.62) return startCombo(state, boss)
    if (roll < 0.82) return startLeap(state, boss)
    boss.state = 'backstep'; boss.t = 0.35
    boss.vx = -boss.facing * 420; boss.vy = -360
  }

  function startCombo(state, boss) {
    boss.state = 'combo'
    boss.combo = { i: 0, phase: 'tele' }
    boss.t = (boss.phase2 ? 0.36 : 0.44) * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'tell', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function startLeap(state, boss) {
    boss.state = 'crouch'
    boss.t = 0.48 * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'danger', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function startWave(state, boss) {
    boss.state = 'waveTell'
    boss.t = 0.58 * state.diff.tele
    boss.teleTotal = boss.t
    boss.wavesLeft = boss.phase2 ? 2 : 1
    emit(state, 'tell', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function startPhantom(state, boss) {
    const player = state.player
    const [min, max] = state.lock
    const middle = (min + max) / 2
    emit(state, 'vanish', { id: boss.id, x: boss.x, y: boss.y - 36 })
    boss.x = player.x < middle ? max - 60 : min + 60
    boss.y = GROUND; boss.vy = 0; boss.vx = 0
    boss.facing = sign(player.x - boss.x) || -1
    emit(state, 'appear', { id: boss.id, x: boss.x, y: boss.y - 36 })
    boss.state = 'phantomTell'
    boss.t = 0.95 * state.diff.tele
    boss.teleTotal = boss.t
    emit(state, 'danger', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
  }

  function bossReachBox(boss, reach) {
    return boss.facing > 0
      ? { x0: boss.x, x1: boss.x + reach, y0: boss.y - boss.h - 6, y1: boss.y + 4 }
      : { x0: boss.x - reach, x1: boss.x, y0: boss.y - boss.h - 6, y1: boss.y + 4 }
  }

  function spawnWave(state, boss, direction, speed, life) {
    state.projectiles.push({ id: state.nextId++, type: 'wave', owner: 'enemy', x: boss.x + direction * 30, y: GROUND, vx: direction * speed, vy: 0, life, source: boss.id, dead: false })
  }

  function updateBoss(state, boss, dt) {
    const player = state.player
    const dx = player.x - boss.x
    const faster = boss.phase2 ? 0.82 : 1
    if (boss.state !== 'broken' && state.clock - boss.postureClock > 2) boss.posture = Math.max(0, boss.posture - 7 * dt)
    const facePlayer = () => { if (!player.dead) boss.facing = sign(dx) || boss.facing }
    switch (boss.state) {
      case 'intro':
        boss.vx = 0
        if (boss.t <= 0) { boss.state = 'idle'; boss.t = 0.4 }
        break
      case 'idle':
        facePlayer()
        boss.vx *= Math.pow(0.01, dt)
        if (boss.t <= 0 && !player.dead) chooseBossMove(state, boss)
        break
      case 'walk':
        facePlayer()
        boss.vx += (boss.facing * 150 - boss.vx) * Math.min(1, dt * 6)
        if (Math.abs(dx) < 120 || boss.t <= 0) startCombo(state, boss)
        break
      case 'backstep':
        if (boss.t <= 0 && boss.y >= GROUND - 0.5) { boss.vx = 0; startWave(state, boss) }
        break
      case 'combo': {
        const combo = boss.combo
        if (combo.phase === 'tele') {
          boss.vx *= Math.pow(0.001, dt)
          if (boss.t <= 0) {
            combo.phase = 'active'; boss.t = 0.13; boss.hitDone = false
            boss.vx = boss.facing * 300
            emit(state, 'enemySwing', { id: boss.id, x: boss.x, y: boss.y, facing: boss.facing, kind: 'boss', step: combo.i })
          }
        } else if (combo.phase === 'active') {
          if (!boss.hitDone && overlaps(bossReachBox(boss, 100), playerBox(player))) {
            boss.hitDone = true
            enemyStrike(state, boss, 1)
            if (boss.state !== 'combo') break
          }
          boss.vx *= Math.pow(0.02, dt)
          if (boss.t <= 0) {
            combo.i += 1
            if (combo.i >= 3) { boss.state = 'recover'; boss.t = 0.8; boss.combo = null }
            else {
              combo.phase = 'tele'; facePlayer()
              boss.t = (combo.i === 2 ? 0.5 : 0.34) * faster * state.diff.tele
              boss.teleTotal = boss.t
              emit(state, 'tell', { id: boss.id, x: boss.x, y: boss.y - boss.h, facing: boss.facing })
            }
          }
        }
        break
      }
      case 'crouch':
        facePlayer(); boss.vx = 0
        if (boss.t <= 0) {
          const [min, max] = state.lock
          const target = clamp(player.x, min + 50, max - 50)
          boss.vx = (target - boss.x) / 0.72
          boss.vy = -GRAVITY * 0.72 / 2
          boss.state = 'leap'; boss.t = 0.72
          emit(state, 'leap', { id: boss.id, x: boss.x, y: boss.y })
        }
        break
      case 'leap':
        if (boss.y >= GROUND - 0.5 && boss.vy >= 0 && boss.t < 0.5) {
          boss.vx = 0
          boss.state = 'recover'; boss.t = 0.85
          emit(state, 'slam', { id: boss.id, x: boss.x, y: GROUND })
          const slam = { x0: boss.x - 82, x1: boss.x + 82, y0: GROUND - 50, y1: GROUND + 4 }
          if (overlaps(slam, playerBox(player))) enemyStrike(state, boss, 1, 'unblockable')
          spawnWave(state, boss, -1, 360, 0.95)
          spawnWave(state, boss, 1, 360, 0.95)
        }
        break
      case 'waveTell':
        boss.vx = 0
        if (boss.t <= 0) {
          facePlayer()
          spawnWave(state, boss, boss.facing, 430, 2.2)
          emit(state, 'wave', { id: boss.id, x: boss.x, y: GROUND, facing: boss.facing })
          boss.wavesLeft -= 1
          if (boss.wavesLeft > 0) { boss.t = 0.42; boss.teleTotal = 0.42 }
          else { boss.state = 'recover'; boss.t = 0.6 }
        }
        break
      case 'phantomTell':
        boss.vx = 0
        if (boss.t <= 0) {
          boss.state = 'phantom'; boss.hitDone = false; boss.t = 1.2
          boss.vx = boss.facing * 1150
          emit(state, 'phantom', { id: boss.id, x: boss.x, y: boss.y, facing: boss.facing })
        }
        break
      case 'phantom': {
        const [min, max] = state.lock
        if (!boss.hitDone && overlaps({ x0: boss.x - 24, x1: boss.x + 24, y0: GROUND - 58, y1: GROUND }, playerBox(player))) {
          boss.hitDone = true
          enemyStrike(state, boss, 1, 'unblockable')
        }
        if ((boss.facing > 0 && boss.x >= max - 62) || (boss.facing < 0 && boss.x <= min + 62) || boss.t <= 0) {
          boss.vx = 0; boss.state = 'recover'; boss.t = 0.9
        }
        break
      }
      case 'roar':
        boss.vx = 0
        if (Math.abs(dx) < 150 && !player.dead && boss.t > 1.1) { player.vx = sign(dx) * 360; player.vy = Math.min(player.vy, -200); player.onGround = false }
        if (boss.t <= 0) { boss.state = 'idle'; boss.t = 0.3 }
        break
      case 'broken':
        boss.vx = 0
        if (boss.t <= 0) { boss.state = 'recover'; boss.t = 0.3; boss.posture = 40 }
        break
      case 'recover':
        boss.vx *= Math.pow(0.01, dt)
        if (boss.t <= 0) { boss.state = 'idle'; boss.t = (0.45 + state.random() * 0.4) * (boss.phase2 ? 0.8 : 1) / state.diff.aggression }
        break
      case 'dying':
        boss.vx = 0
        break
    }
    const [min, max] = state.lock || [0, state.world.length]
    boss.x = clamp(boss.x + boss.vx * dt, min + boss.w, max - boss.w)
    if (boss.y < GROUND || boss.vy < 0) {
      boss.vy = Math.min(1100, boss.vy + GRAVITY * dt)
      boss.y = Math.min(GROUND, boss.y + boss.vy * dt)
      if (boss.y >= GROUND) boss.vy = 0
    }
  }

  function updateEnemies(state, dt) {
    for (const enemy of state.enemies) {
      if (!enemy.alive) continue
      if (enemy.delay > 0) {
        enemy.delay -= dt
        if (enemy.delay <= 0) { enemy.delay = 0; emit(state, 'spawn', { id: enemy.id, kind: enemy.type, x: enemy.x, y: enemy.y }) }
        continue
      }
      enemy.t -= dt
      enemy.cd -= dt
      enemy.flash = Math.max(0, enemy.flash - dt)
      if (enemy.state === 'spawn') {
        if (enemy.t <= 0) {
          if (enemy.type === 'boss' || enemy.type === 'angler') { enemy.state = 'intro'; enemy.t = 2.2; emit(state, 'bossIntro', { id: enemy.id, kind: enemy.type, x: enemy.x, y: enemy.y }) }
          else enemy.state = 'idle'
        }
        continue
      }
      if (enemy.type === 'boss') {
        if (enemy.state === 'dying') {
          if (enemy.t <= 0) { enemy.alive = false; enemy.state = 'dead'; emit(state, 'kill', { id: enemy.id, kind: 'boss', x: enemy.x, y: enemy.y, facing: enemy.facing, how: 'boss' }) }
          continue
        }
        updateBoss(state, enemy, dt)
      } else if (enemy.type === 'angler') {
        if (enemy.state === 'dying') {
          if (enemy.t <= 0) { enemy.alive = false; enemy.state = 'dead'; emit(state, 'kill', { id: enemy.id, kind: 'angler', x: enemy.x, y: enemy.y, facing: enemy.facing, how: 'boss' }) }
          continue
        }
        updateAngler(state, enemy, dt)
      } else if (enemy.type === 'spearman') updateSpearman(state, enemy, dt)
      else if (enemy.type === 'ronin') updateRonin(state, enemy, dt)
      else if (enemy.type === 'hammer') updateHammer(state, enemy, dt)
      else if (enemy.type === 'crow') updateCrow(state, enemy, dt)
      else if (enemy.type === 'archer') updateArcher(state, enemy, dt)
      else if (enemy.type === 'dummy') updateDummy(state, enemy)
      else updateMelee(state, enemy, dt)
    }
  }

  function updateProjectiles(state, dt) {
    const player = state.player
    for (const shot of state.projectiles) {
      if (shot.dead) continue
      shot.life -= dt
      shot.x += shot.vx * dt
      if (shot.owner === 'enemy' && (shot.type === 'arrow' || shot.type === 'dart')) shot.x += windSpeed(state, shot.x) * 0.5 * dt
      if (shot.type === 'arrow') { shot.vy += ARROW_GRAVITY * dt; shot.y += shot.vy * dt }
      if (shot.life <= 0 || shot.y > GROUND + 4 || shot.x < 0 || shot.x > state.world.length) {
        if (shot.type === 'arrow' && shot.y > GROUND - 4) emit(state, 'stick', { x: shot.x, y: GROUND, vx: shot.vx, vy: shot.vy })
        shot.dead = true
        continue
      }
      if (shot.owner === 'enemy') {
        const box = (shot.type === 'wave' || shot.type === 'iceWave')
          ? { x0: shot.x - 18, x1: shot.x + 18, y0: GROUND - 36, y1: GROUND }
          : { x0: shot.x - 8, x1: shot.x + 8, y0: shot.y - 6, y1: shot.y + 6 }
        if (!player.dead && overlaps(box, playerBox(player))) {
          const action = player.action
          const facing = sign(-shot.vx) === player.facing
          if (shot.type === 'hook') {
            if (player.iframes <= 0 && !state.strike) {
              shot.dead = true
              hurtPlayer(state, 1, shot.x)
              player.vx = -player.facing * 340
            }
          } else if (action?.type === 'slash' && facing && state.clock - player.slashStartClock <= state.diff.parry) {
            deflect(state, shot)
          } else if (player.iframes <= 0 && !state.strike) {
            shot.dead = true
            hurtPlayer(state, 1, shot.x - shot.vx)
          }
        }
      } else {
        for (const ice of state.icicles) {
          if (ice.state === 'hang' && overlaps({ x0: shot.x - 8, x1: shot.x + 8, y0: shot.y - 6, y1: shot.y + 6 }, iceBox(ice))) dropIcicle(state, ice)
        }
        for (const enemy of state.enemies) {
          if (!enemy.alive || enemy.delay > 0 || enemy.state === 'spawn') continue
          if (overlaps({ x0: shot.x - 8, x1: shot.x + 8, y0: shot.y - 6, y1: shot.y + 6 }, enemyBox(enemy))) {
            shot.dead = true
            hitEnemy(state, enemy, 3, 'arrow')
            if (!enemy.alive) addScore(state, 80)
            break
          }
        }
      }
    }
    state.projectiles = state.projectiles.filter((shot) => !shot.dead)
  }

  function updateStrike(state, dt) {
    const strike = state.strike
    strike.t += dt
    if (!strike.applied && strike.t >= 0.55) {
      strike.applied = true
      const player = state.player
      for (const shot of state.projectiles) if (shot.owner === 'enemy') shot.dead = true
      for (const enemy of state.enemies) {
        if (!enemy.alive || enemy.delay > 0 || enemy.state === 'spawn' || Math.abs(enemy.x - player.x) > 470) continue
        hitEnemy(state, enemy, 4, 'strike')
      }
      for (const ice of state.icicles) if (ice.state === 'hang' && Math.abs(ice.x - player.x) <= 470) dropIcicle(state, ice)
      emit(state, 'strikeHit', { x: player.x, y: player.y })
    }
    if (strike.t >= 1.15) state.strike = null
  }

  // ------------------------------------------------------------ terrain: icicles, thin ice, reeds

  function iceBox(ice) { return { x0: ice.x - 9, x1: ice.x + 9, y0: ice.top, y1: ice.y } }

  function dropIcicle(state, ice) {
    if (ice.state !== 'hang') return
    ice.state = 'fall'
    emit(state, 'icicleFall', { x: ice.x, y: ice.y, id: ice.id })
  }

  function updateIcicles(state, dt) {
    const player = state.player
    for (const ice of state.icicles) {
      if (ice.state !== 'fall') continue
      ice.vy = Math.min(900, ice.vy + GRAVITY * dt)
      ice.y += ice.vy * dt
      const box = { x0: ice.x - 16, x1: ice.x + 16, y0: ice.y - 30, y1: ice.y + 6 }
      for (const enemy of state.enemies) {
        if (!enemy.alive || enemy.delay > 0 || enemy.state === 'spawn' || enemy.state === 'dying' || ice.hits.includes(enemy.id)) continue
        if (!overlaps(box, enemyBox(enemy))) continue
        ice.hits.push(enemy.id)
        hitEnemy(state, enemy, 3, 'icicle')
        emit(state, 'iceHit', { x: ice.x, y: enemy.y - enemy.h / 2 })
      }
      if (!ice.hitPlayer && !player.dead && overlaps(box, playerBox(player))) {
        ice.hitPlayer = true
        if (player.iframes <= 0 && !state.strike) hurtPlayer(state, 1, ice.x)
      }
      if (ice.y >= GROUND) {
        ice.state = 'gone'
        emit(state, 'iceShatter', { x: ice.x, y: GROUND })
      }
    }
  }

  const CRACK_WIDTH = 230, CRACK_STAND = 1.0, CRACK_TELE = 0.8, CRACK_OPEN = 3

  /** Standing still on 薄冰 cracks it open under the player; a rooted enemy beside you falls in with you. */
  function updateThinIce(state, dt) {
    const player = state.player
    const grounded = !player.dead && player.onGround && player.y >= GROUND - 1
    const standing = grounded && onThinIce(state, player.x) && Math.abs(player.vx) < 90 && !state.strike
    player.standT = standing ? player.standT + dt : Math.max(0, player.standT - dt * 2)
    if (player.standT >= CRACK_STAND && state.clock >= state.nextCrack && !state.cracks.some((crack) => crack.state === 'tele')) {
      player.standT = 0
      const tele = CRACK_TELE * state.diff.tele
      state.nextCrack = state.clock + tele + 1.2
      state.cracks.push({ id: state.nextId++, x: player.x, w: CRACK_WIDTH, state: 'tele', t: 0, tele })
      emit(state, 'crackTell', { x: player.x, w: CRACK_WIDTH, tele })
      prompt(state, 'crack')
    }
    for (const crack of state.cracks) {
      crack.t += dt
      if (crack.state === 'tele' && crack.t >= crack.tele) {
        crack.state = 'open'; crack.t = 0
        emit(state, 'crackBurst', { x: crack.x, w: crack.w })
        const half = crack.w / 2
        if (grounded && Math.abs(player.x - crack.x) <= half && player.iframes <= 0 && !state.strike) hurtPlayer(state, 1, crack.x)
        for (const enemy of state.enemies) {
          if (!enemy.alive || enemy.delay > 0 || enemy.state === 'spawn' || enemy.state === 'dying') continue
          if (enemy.type === 'crow' || enemy.type === 'boss' || enemy.type === 'angler' || enemy.y < GROUND - 4 || Math.abs(enemy.x - crack.x) > half) continue
          hitEnemy(state, enemy, 4, 'crack')
          if (enemy.alive) {
            enemy.state = 'stagger'; enemy.openedAt = state.clock; enemy.t = 1.4; enemy.vx = 0
            emit(state, 'stagger', { id: enemy.id, x: enemy.x, y: enemy.y - enemy.h })
          }
        }
      } else if (crack.state === 'open' && crack.t >= CRACK_OPEN) crack.state = 'gone'
    }
    state.cracks = state.cracks.filter((crack) => crack.state !== 'gone')
  }

  /** Crouching still in 芦丛 for a moment hides the player; an enemy at arm's length, or too long a stay, gives it away. */
  function updateCover(state, dt) {
    const player = state.player
    const spotted = state.enemies.some((enemy) => enemy.alive && enemy.delay <= 0 && enemy.state !== 'spawn' && enemy.type !== 'crow' && Math.abs(enemy.x - player.x) < 48 && Math.abs(enemy.y - player.y) < 40)
    const eligible = !player.dead && player.onGround && inReeds(state, player.x) && !player.action && player.hurt <= 0 && Math.abs(player.vx) < 70 && !spotted && player.spottedT <= 0
    player.spottedT = Math.max(0, player.spottedT - dt)
    player.coverT = eligible ? Math.min(0.6, player.coverT + dt) : 0
    const hidden = player.coverT >= 0.35
    player.hideT = hidden ? player.hideT + dt : 0
    if (hidden && player.hideT > 4) { player.spottedT = 1.5; player.hideT = 0; player.coverT = 0 }
    const now = player.coverT >= 0.35
    if (now !== player.hidden) emit(state, now ? 'hide' : 'reveal', { x: player.x, y: player.y })
    player.hidden = now
  }

  function updateEncounters(state, dt) {
    const player = state.player
    if (!state.lock) {
      const encounter = state.plan[state.encounter]
      if (encounter && player.x >= encounter.trigger) {
        // Inset arenas so both ink curtains stay on a typical phone screen with some margin.
        state.lock = [encounter.lock[0] + 40, encounter.lock[1] - 40]
        state.wave = 0
        emit(state, 'lock', { id: encounter.id, x0: state.lock[0], x1: state.lock[1] })
        for (const spec of encounter.waves[0]) spawnEnemy(state, spec)
        if (encounter.id === 'gate') prompt(state, 'slash')
      }
      return
    }
    const encounter = state.plan[state.encounter]
    if (!encounter || state.enemies.some((enemy) => enemy.alive)) return
    if (state.waveDelay > 0) {
      state.waveDelay -= dt
      if (state.waveDelay > 0) return
      for (const spec of encounter.waves[state.wave]) spawnEnemy(state, spec)
      return
    }
    if (state.wave + 1 < encounter.waves.length) {
      state.wave += 1
      state.waveDelay = 0.7
      emit(state, 'wave2', { id: encounter.id })
      return
    }
    state.lock = null
    state.encounter += 1
    addScore(state, 100 * state.encounter)
    emit(state, 'clear', { id: encounter.id })
    if (state.encounter >= state.plan.length) finish(state, 'clear')
  }

  function finish(state, reason) {
    if (state.phase === 'over') return
    const player = state.player
    const level = state.level
    state.phase = 'over'
    state.reason = reason
    if (reason === 'clear') {
      state.bonus = {
        time: Math.max(0, Math.round(level.par - state.time) * 6),
        hp: player.hp * 150,
        flawless: player.damageTaken === 0 ? (level.scroll ? 1500 : level.boss ? 1000 : 600) : 0,
      }
      player.score += state.bonus.time + state.bonus.hp + state.bonus.flawless
    }
    emit(state, 'over', { reason })
  }

  function updateFlags(state) {
    const player = state.player
    const areas = state.world.areas
    const [from, to] = state.level.range
    for (const entry of state.world.prompts) {
      if (entry.local && (entry.x < from - 150 || entry.x > to)) continue
      if (player.x >= entry.x) prompt(state, entry.key)
    }
    let area = state.area
    while (area + 1 < areas.length && player.x >= areas[area + 1].x) area += 1
    if (area !== state.area) { state.area = area; emit(state, 'area', { index: area, name: areas[area].name, sub: areas[area].sub }) }
  }

  // ------------------------------------------------------------ seals & grade

  /** Current value of the 幅's 技 challenge, and whether it is met right now. */
  function challenge(state) {
    const goal = state.level.challenge
    const value = goal.stat === 'time' ? Math.floor(state.time) : state.player[goal.stat] || 0
    return { value, goal: goal.goal, met: goal.at === 'most' ? value <= goal.goal : value >= goal.goal, text: goal.text, stat: goal.stat }
  }

  /** Seals are a bitmask (1 过 通关, 2 全 少伤, 4 技 绝技); grade 0 means not cleared, 1–4 能 / 妙 / 神 / 逸品. */
  function evaluate(state) {
    const cleared = state.reason === 'clear'
    if (!cleared) return { seals: 0, grade: 0 }
    let seals = 1
    if (state.player.damageTaken <= state.level.hurt) seals |= 2
    if (challenge(state).met) seals |= 4
    const grade = 1 + state.level.grades.filter((score) => state.player.score >= score).length
    return { seals, grade }
  }

  function step(state, input = {}, dt = 1 / 60) {
    state.events = []
    if (state.phase !== 'playing') return state.events
    const player = state.player
    if (input.jumpPressed) player.jumpBuffer = 0.13
    if (input.slashPressed) player.slashBuffer = 0.17
    if (input.dashPressed) player.dashBuffer = 0.13
    if (input.inkPressed) player.inkBuffer = 0.13
    player.jumpHeld = Boolean(input.jump)
    state.time += dt
    player.jumpBuffer -= dt; player.slashBuffer -= dt; player.dashBuffer -= dt; player.inkBuffer -= dt
    if (state.hitstop > 0) { state.hitstop -= dt; return state.events }
    let scale = 1
    if (state.slow > 0) { state.slow -= dt; scale = state.slowScale }
    const sdt = dt * scale
    state.clock += sdt
    if (state.strike) {
      updateStrike(state, dt)
      return state.events
    }
    updateWind(state)
    updateCover(state, sdt)
    updatePlayer(state, input, sdt)
    updateEnemies(state, sdt)
    updateProjectiles(state, sdt)
    updateIcicles(state, sdt)
    updateThinIce(state, sdt)
    state.enemies = state.enemies.filter((enemy) => enemy.alive || enemy.state === 'dying')
    updateEncounters(state, dt)
    updateFlags(state)
    if (player.dead && player.deadT > 1.5) finish(state, 'death')
    return state.events
  }

  function result(state) {
    const player = state.player
    const { seals, grade } = evaluate(state)
    return {
      level: state.level.id, code: state.level.code, score: player.score, kills: player.kills, maxCombo: player.maxCombo,
      parries: player.parries, cleared: state.reason === 'clear', seconds: Math.floor(state.time), executions: player.executions,
      damageTaken: player.damageTaken, seals, grade,
    }
  }

  globalThis.XiabanInkBlade = {
    GROUND, GRAVITY, STAGES, ENEMY, DIFFICULTIES,
    createGame, step, result, evaluate, challenge, spawnEnemy, slashBox, enemyBox, overlaps, windSpeed,
  }
})()
