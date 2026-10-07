/* 水墨剑客 · procedural ink figures: skeletal poses, brushed silhouettes, cloth and scarf motion. */
(() => {
  'use strict'

  const PAPER = '#efe7d4'
  const CINNABAR = '#b8322a'
  const EMBER = '#e2412c'
  const TAU = Math.PI * 2
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v))
  const lerp = (a, b, t) => a + (b - a) * t

  const KINDS = {
    player: { thigh: 13, shin: 13, torso: 18, upper: 10, fore: 9.5, head: 5.2, limbs: [5.2, 4.3, 4, 3.4], blade: 35, bladeW: 2.3, color: '#141416', robe: 'robe', hat: 'douli' },
    puppet: { thigh: 12, shin: 12, torso: 18, upper: 10, fore: 9.5, head: 6, limbs: [5.6, 4.6, 4.6, 3.8], blade: 30, bladeW: 3.6, color: '#30343a', robe: 'rags', headStyle: 'blob' },
    shield: { thigh: 13, shin: 12, torso: 21, upper: 11, fore: 10, head: 6.6, limbs: [7.2, 6.2, 6.4, 5.2], blade: 27, bladeW: 5.5, color: '#2b2e33', robe: 'rags', headStyle: 'blob', shield: true },
    archer: { thigh: 13, shin: 13, torso: 18, upper: 10, fore: 10, head: 5.2, limbs: [4.6, 3.9, 3.6, 3.1], blade: 0, color: '#33373d', robe: 'hood', headStyle: 'hood', bow: true },
    boss: { thigh: 17, shin: 17, torso: 25, upper: 13, fore: 12, head: 6.2, limbs: [7, 5.8, 6, 4.8], blade: 56, bladeW: 3.2, color: '#0f1012', robe: 'long', hat: 'veil', mask: true, sleeves: true },
    ronin: { thigh: 13, shin: 13, torso: 18, upper: 10, fore: 9.5, head: 5.6, limbs: [5.4, 4.4, 4.2, 3.6], blade: 32, bladeW: 3.2, color: '#24262b', robe: 'straw', hat: 'douli' },
    spearman: { thigh: 14, shin: 14, torso: 22, upper: 12, fore: 11, head: 5.6, limbs: [6.2, 5.2, 5.2, 4.2], blade: 0, color: '#1b1d22', robe: 'armor', hat: 'helmet', spear: true },
    hammer: { thigh: 16, shin: 15, torso: 26, upper: 14, fore: 13, head: 7.2, limbs: [8.5, 7.2, 7.5, 6.2], blade: 0, color: '#202228', robe: 'vest', headStyle: 'blob', hammer: true },
    angler: { thigh: 16, shin: 16, torso: 24, upper: 12, fore: 11, head: 6.2, limbs: [6.2, 5.2, 5.4, 4.4], blade: 0, color: '#121316', robe: 'straw', hat: 'douli', beard: true, rod: true },
  }

  const POSE_KEYS = ['lean', 'crouch', 'l1', 'b1', 'l2', 'b2', 'ou', 'of', 'sw', 'reach', 'head', 'rot', 'bob', 'draw', 'air', 'flutter']

  function basePose() {
    return { lean: 0.06, crouch: 1, l1: 0.18, b1: 0.3, l2: -0.16, b2: 0.22, ou: 0.1, of: -0.4, sw: 2.45, reach: 0.75, head: 0, rot: 0, bob: 0, draw: 0, air: 0, flutter: 0 }
  }

  function createAnim(entity) {
    return { pose: basePose(), phase: (entity.seed || 0) * 3, flip: 0, parry: 0, wobble: 0, wobbleV: 0, scarf: null, speed: 0, snap: false }
  }

  function runLegs(target, phase, amount) {
    const s = Math.sin(phase), c = Math.cos(phase)
    target.l1 = s * 0.85 * amount
    target.b1 = 0.25 + Math.max(0, c) * 1.25 * amount
    target.l2 = -s * 0.85 * amount
    target.b2 = 0.25 + Math.max(0, -c) * 1.25 * amount
    target.bob = Math.abs(Math.cos(phase)) * 1.6 * amount
  }

  const ease = (q) => 1 - Math.pow(1 - clamp(q, 0, 1), 3)

  // ---------------------------------------------------------------- poses

  function playerPose(p, anim, time, rules) {
    const t = basePose()
    const speed = Math.abs(p.vx)
    t.flutter = clamp(speed / 240, 0, 1)
    if (p.dead) {
      Object.assign(t, { lean: -0.2, crouch: 6, l1: 0.6, b1: 1.2, l2: 0.1, b2: 0.9, ou: 2.4, of: 2.8, sw: 0.4, reach: 1, rot: -Math.min(1, p.deadT * 1.6) * 1.45 })
      return t
    }
    const action = p.action
    if (action?.type === 'slash') {
      const stage = rules.STAGES[action.stage]
      const q = (action.t - stage.a0) / (stage.a1 - stage.a0)
      const [a0, a1] = stage.arc
      t.sw = q <= 0 ? a0 - 0.25 * (1 - action.t / stage.a0) : lerp(a0, a1, ease(q)) + (q > 1 ? Math.sin(Math.min(1, (q - 1) * 0.5) * Math.PI) * 0.08 : 0)
      t.reach = 1
      t.lean = (action.stage === 2 ? 0.48 : 0.32) * ease(q + 0.3)
      t.crouch = action.stage === 2 ? 5 : 3
      Object.assign(t, action.air ? { l1: 0.75, b1: 1.3, l2: 0.05, b2: 1.05 } : { l1: 0.62, b1: 0.55, l2: -0.6, b2: 0.15 })
      t.ou = -0.7; t.of = -1.3
      anim.snap = true
      return t
    }
    anim.snap = false
    if (action?.type === 'dash') {
      Object.assign(t, { lean: 0.85, crouch: 4, l1: 1.1, b1: 0.6, l2: -1.05, b2: 0.15, sw: 3.05, reach: 0.95, ou: -1.3, of: -1.5 })
      return t
    }
    if (p.hurt > 0) {
      Object.assign(t, { lean: -0.5, crouch: 3, l1: 0.45, b1: 0.5, l2: -0.25, b2: 0.35, sw: -2.2, reach: 0.9, ou: 2.6, of: 2.2, head: -0.3 })
      return t
    }
    if (!p.onGround) {
      t.air = 1
      if (p.vy < 0) Object.assign(t, { lean: 0.14, l1: 0.75, b1: 1.35, l2: -0.12, b2: 0.85, sw: 3.3, reach: 0.85, ou: -0.5, of: -1 })
      else Object.assign(t, { lean: 0.05, l1: 0.35, b1: 0.75, l2: -0.32, b2: 0.55, sw: 2.85, reach: 0.85, ou: 2.1, of: 2.5 })
      if (anim.flip > 0) t.rot = (1 - anim.flip) * TAU
      if (anim.parry > 0) { t.sw = -1.25; t.reach = 1 }
      return t
    }
    if (speed > 25) {
      runLegs(t, anim.phase, clamp(speed / 240, 0.35, 1))
      t.lean = 0.24 + t.flutter * 0.12
      t.crouch = 2
      t.sw = 2.78; t.reach = 0.85
      t.ou = -Math.sin(anim.phase) * 0.9; t.of = t.ou - 0.9
    } else {
      const breathe = Math.sin(time * 1.8)
      t.lean = 0.06 + breathe * 0.02
      t.crouch = 1.2 + breathe * 0.6
    }
    if (anim.parry > 0) { t.sw = -1.25; t.reach = 1; t.lean = -0.12 }
    return t
  }

  function telePose(t, progress, raised = -2.35) {
    t.sw = lerp(0.9, raised, ease(progress * 1.4))
    t.reach = 1
    t.lean = lerp(0.3, -0.12, progress)
    t.crouch = 3 + progress * 2
    Object.assign(t, { l1: 0.45, b1: 0.5, l2: -0.45, b2: 0.2 })
  }

  function enemyPose(e, anim, time) {
    const t = basePose()
    const speed = Math.abs(e.vx)
    t.flutter = clamp(speed / 200, 0, 1)
    anim.snap = false
    t.lean = 0.32; t.sw = 0.9; t.reach = 0.85; t.ou = 0.4; t.of = 0.1; t.crouch = 3
    const progress = e.teleTotal ? 1 - Math.max(0, e.t) / e.teleTotal : 0
    if (e.type === 'archer') { t.lean = 0.08; t.ou = 1.5; t.of = 1.52; t.sw = 0 }
    if (e.type === 'shield') { t.ou = 0.95; t.of = 1.45; t.lean = 0.22 }
    if (e.type === 'spearman') { t.sw = -0.15; t.reach = 1.1; t.lean = 0.28 }
    if (e.type === 'hammer') { t.sw = -0.6; t.reach = 0.95; t.lean = 0.25; t.crouch = 4 }
    if (e.type === 'ronin') { t.lean = 0.42; t.crouch = 3.5 }
    switch (e.state) {
      case 'spawn':
        t.crouch = 8; t.lean = 0.6
        break
      case 'walk':
        runLegs(t, anim.phase, clamp(speed / 150, 0.3, 0.8))
        t.lean += 0.06
        if (e.type === 'hammer') t.bob = Math.abs(Math.cos(anim.phase)) * 2.8
        break
      case 'tele':
        if (e.type === 'archer') { t.draw = ease(progress); t.lean = 0.02 }
        else if (e.type === 'spearman') {
          if (e.move === 'thrust') { t.sw = -0.05; t.reach = 1.25; t.lean = 0.6; t.crouch = 5 }
          else { t.sw = lerp(0.8, -2.4, ease(progress * 1.4)); t.reach = 1.1; t.lean = 0.1 }
        } else if (e.type === 'hammer') {
          t.sw = lerp(-0.5, -2.85, ease(progress * 1.5)); t.reach = 1.1; t.crouch = 6; t.lean = -0.25
        } else {
          telePose(t, progress, e.type === 'shield' ? -2.6 : -2.35)
        }
        break
      case 'attack':
        if (e.type === 'spearman') {
          if (e.move === 'thrust') { t.sw = 0.05; t.reach = 1.45; t.lean = 0.8; t.crouch = 6 }
          else { t.sw = 1.2; t.reach = 1.25; t.lean = 0.55; t.crouch = 4 }
        } else if (e.type === 'hammer') {
          t.sw = 1.25; t.reach = 1.2; t.lean = 0.75; t.crouch = 8
        } else {
          t.sw = 0.85; t.reach = 1; t.lean = 0.55; t.crouch = 5
          Object.assign(t, { l1: 0.75, b1: 0.4, l2: -0.75, b2: 0.1 })
        }
        anim.snap = true
        break
      case 'recover':
        t.sw = 0.95; t.lean = 0.6; t.crouch = 5
        if (e.type === 'archer') { t.lean = 0.05; t.draw = 0 }
        if (e.type === 'spearman') { t.sw = -0.1; t.reach = 0.95 }
        if (e.type === 'hammer') { t.sw = 0.8; t.crouch = 6 }
        break
      case 'hurt':
        t.lean = -0.38; t.sw = -1.8; t.ou = 2; t.of = 2.4; t.crouch = 2; t.head = -0.3
        break
      case 'stagger':
        t.lean = -0.55 + Math.sin(time * 9) * 0.06; t.sw = -2.7; t.reach = 1; t.ou = 2.6; t.of = 3; t.crouch = 5; t.head = -0.45
        Object.assign(t, { l1: 0.5, b1: 0.6, l2: -0.35, b2: 0.3 })
        break
      case 'hop':
        t.air = 1; Object.assign(t, { l1: 0.7, b1: 1.3, l2: -0.1, b2: 0.9, lean: -0.1 })
        break
    }
    return t
  }

  function bossPose(e, anim, time) {
    if (e.type === 'angler') return anglerPose(e, anim, time)
    const t = basePose()
    const speed = Math.abs(e.vx)
    t.flutter = clamp(speed / 260, 0, 1)
    anim.snap = false
    const progress = e.teleTotal ? 1 - Math.max(0, e.t) / e.teleTotal : 0
    t.lean = 0.12; t.sw = 0.5; t.reach = 0.9; t.ou = -0.3; t.of = -0.6; t.crouch = 2
    const breathe = Math.sin(time * 1.4)
    switch (e.state) {
      case 'spawn':
      case 'intro':
        t.sw = 1.35; t.reach = 0.7; t.lean = 0.02 + breathe * 0.02; t.ou = 0.1; t.of = 0.3
        break
      case 'idle':
        t.crouch = 3 + breathe; t.sw = 0.35 + breathe * 0.05; t.lean = 0.18
        Object.assign(t, { l1: 0.4, b1: 0.4, l2: -0.38, b2: 0.15 })
        break
      case 'walk':
        runLegs(t, anim.phase, 0.7); t.lean = 0.3; t.sw = 2.6
        break
      case 'combo': {
        const step = e.combo ? e.combo.i : 0
        const raised = [-2.4, 2.75, -2.9][step]
        const swing = [0.9, -1.3, 1.25][step]
        if (e.combo?.phase === 'tele') {
          t.sw = lerp(t.sw, raised, ease(progress * 1.5)); t.reach = 1; t.lean = -0.08; t.crouch = 4
        } else {
          t.sw = swing; t.reach = 1; t.lean = 0.55; t.crouch = 6; anim.snap = true
        }
        Object.assign(t, { l1: 0.7, b1: 0.45, l2: -0.7, b2: 0.12 })
        break
      }
      case 'crouch':
        t.crouch = 7 + progress * 8; t.lean = 0.5; t.sw = -2.6; t.reach = 1
        Object.assign(t, { l1: 0.9, b1: 1.6, l2: -0.3, b2: 1.3 })
        break
      case 'leap':
        t.air = 1; t.sw = e.vy < 0 ? -2.9 : 1.4; t.reach = 1; t.lean = 0.2
        Object.assign(t, { l1: 0.8, b1: 1.5, l2: 0, b2: 1.2 })
        break
      case 'waveTell':
        t.sw = lerp(0.6, 2.95, ease(progress * 1.3)); t.reach = 1; t.lean = 0.35; t.crouch = 6
        Object.assign(t, { l1: 0.7, b1: 0.55, l2: -0.75, b2: 0.15 })
        break
      case 'phantomTell':
        t.sw = 3.05; t.reach = 0.8; t.lean = 0.72; t.crouch = 9
        Object.assign(t, { l1: 0.95, b1: 1.2, l2: -0.85, b2: 0.3 })
        break
      case 'phantom':
        t.sw = 3.1; t.reach = 1; t.lean = 0.9; t.crouch = 7; t.flutter = 1
        Object.assign(t, { l1: 1.2, b1: 0.5, l2: -1.1, b2: 0.15 })
        anim.snap = true
        break
      case 'backstep':
        t.air = 1; t.lean = -0.25; t.sw = 2.6
        break
      case 'roar':
        t.lean = -0.4; t.sw = -2.2; t.reach = 1; t.ou = 2.8; t.of = 3.1; t.head = -0.4; t.crouch = 3
        break
      case 'broken':
        t.lean = 0.85; t.crouch = 16; t.sw = 0.25; t.reach = 1; t.ou = 0.6; t.of = 0.4; t.head = 0.3
        Object.assign(t, { l1: 1.2, b1: 2.3, l2: -0.2, b2: 1.6 })
        break
      case 'recover':
        t.lean = 0.42; t.sw = 0.85; t.crouch = 6
        break
      case 'dying':
        t.lean = 0.9; t.crouch = 18; t.sw = 0.2; t.ou = 0.5; t.of = 0.2; t.head = 0.5
        Object.assign(t, { l1: 1.3, b1: 2.4, l2: -0.15, b2: 1.7 })
        break
    }
    return t
  }

  function anglerPose(e, anim, time) {
    const t = basePose()
    const speed = Math.abs(e.vx)
    t.flutter = clamp(speed / 240, 0, 1)
    anim.snap = false
    const progress = e.teleTotal ? 1 - Math.max(0, e.t) / e.teleTotal : 0
    t.lean = 0.14; t.sw = 0.6; t.reach = 0.95; t.ou = -0.2; t.of = -0.5; t.crouch = 2
    const breathe = Math.sin(time * 1.5)
    switch (e.state) {
      case 'spawn':
      case 'intro':
        t.sw = 1.1; t.reach = 0.85; t.lean = 0.05 + breathe * 0.03
        break
      case 'idle':
        t.crouch = 3 + breathe; t.sw = 0.45 + breathe * 0.08; t.lean = 0.16
        Object.assign(t, { l1: 0.38, b1: 0.38, l2: -0.36, b2: 0.15 })
        break
      case 'walk':
        runLegs(t, anim.phase, 0.72); t.lean = 0.32; t.sw = 0.8
        break
      case 'combo': {
        const step = e.combo ? e.combo.i : 0
        const whip = [0.85, -1.2, 1.15][step]
        if (e.combo?.phase === 'tele') {
          t.sw = lerp(t.sw, -2.2, ease(progress * 1.5)); t.reach = 1.1; t.lean = -0.1; t.crouch = 4
        } else {
          t.sw = whip; t.reach = 1.25; t.lean = 0.58; t.crouch = 5; anim.snap = true
        }
        break
      }
      case 'hookTell':
        t.sw = -1.95; t.reach = 1.1; t.lean = -0.2; t.crouch = 5
        break
      case 'hook':
        t.sw = 0.05; t.reach = 1.45; t.lean = 0.75; t.crouch = 6; anim.snap = true
        break
      case 'slamTell':
        t.sw = -2.7; t.reach = 1; t.crouch = 7; t.lean = 0.45
        break
      case 'slam':
        t.sw = 1.2; t.reach = 1.1; t.crouch = 8; t.lean = 0.7; anim.snap = true
        break
      case 'phantomTell':
        t.sw = -1.5; t.reach = 1.2; t.crouch = 5; t.lean = 0.5
        break
      case 'phantom':
        t.sw = 0.3; t.reach = 1.1; t.lean = 0.85; t.crouch = 6; t.flutter = 1; anim.snap = true
        break
      case 'roar':
        t.lean = -0.4; t.sw = -2.3; t.reach = 1; t.ou = 2.6; t.of = 2.9; t.head = -0.4; t.crouch = 3
        break
      case 'broken':
        t.lean = 0.8; t.crouch = 15; t.sw = 0.3; t.reach = 1; t.head = 0.3
        Object.assign(t, { l1: 1.1, b1: 2.1, l2: -0.2, b2: 1.5 })
        break
      case 'recover':
        t.lean = 0.38; t.sw = 0.75; t.crouch = 5
        break
      case 'dying':
        t.lean = 0.85; t.crouch = 18; t.sw = 0.2; t.head = 0.5
        break
    }
    return t
  }

  function updatePose(kind, entity, anim, dt, time, rules) {
    const speed = Math.abs(entity.vx)
    anim.phase += speed * dt * (kind === 'boss' || kind === 'angler' ? 0.06 : 0.085)
    anim.flip = Math.max(0, anim.flip - dt / 0.42)
    anim.parry = Math.max(0, anim.parry - dt)
    const target = kind === 'player' ? playerPose(entity, anim, time, rules) : (kind === 'boss' || kind === 'angler') ? bossPose(entity, anim, time) : enemyPose(entity, anim, time)
    const rate = anim.snap ? 1 : 1 - Math.exp(-dt * 16)
    for (const key of POSE_KEYS) {
      if (key === 'rot' && (anim.flip > 0 || entity.dead)) { anim.pose.rot = target.rot; continue }
      if (key === 'rot' && Math.abs(anim.pose.rot) > Math.PI) anim.pose.rot = 0
      anim.pose[key] += (target[key] - anim.pose[key]) * rate
    }
    return anim.pose
  }

  // ---------------------------------------------------------------- geometry

  const dir = (a) => [Math.sin(a), Math.cos(a)]

  function ik(sx, sy, tx, ty, l1, l2) {
    let dx = tx - sx, dy = ty - sy
    let d = Math.hypot(dx, dy) || 0.001
    const max = l1 + l2 - 0.05
    if (d > max) { dx *= max / d; dy *= max / d; d = max }
    const a = Math.acos(clamp((l1 * l1 + d * d - l2 * l2) / (2 * l1 * d), -1, 1))
    const base = Math.atan2(dy, dx)
    // Elbow bends downward/outward.
    const angle = base + (Math.cos(base) >= 0 ? a : -a)
    return [sx + Math.cos(angle) * l1, sy + Math.sin(angle) * l1, sx + dx, sy + dy]
  }

  function skeleton(spec, pose) {
    const legH = (a, b) => spec.thigh * Math.cos(a) + spec.shin * Math.cos(a - b)
    const hipY = pose.air > 0.5
      ? -(spec.thigh + spec.shin) * 0.95
      : -Math.max(legH(pose.l1, pose.b1), legH(pose.l2, pose.b2)) + pose.crouch * 0.5 + pose.bob
    const hip = [0, hipY]
    const leg = (a, b) => {
      const [x1, y1] = dir(a)
      const knee = [hip[0] + x1 * spec.thigh, hip[1] + y1 * spec.thigh]
      const [x2, y2] = dir(a - b)
      const foot = [knee[0] + x2 * spec.shin, knee[1] + y2 * spec.shin]
      return { knee, foot }
    }
    const up = [Math.sin(pose.lean), -Math.cos(pose.lean)]
    const fwd = [Math.cos(pose.lean), Math.sin(pose.lean)]
    const neck = [hip[0] + up[0] * spec.torso, hip[1] + up[1] * spec.torso]
    const shoulder = [neck[0] - up[0] * 2.5, neck[1] - up[1] * 2.5]
    const headTilt = pose.lean * 0.55 + pose.head
    const head = [neck[0] + Math.sin(headTilt) * (spec.head + 1.5), neck[1] - Math.cos(headTilt) * (spec.head + 1.5)]
    const swordDir = [Math.cos(pose.sw), Math.sin(pose.sw)]
    const armLen = (spec.upper + spec.fore) * pose.reach * 0.92
    const target = [shoulder[0] + swordDir[0] * armLen * 0.85 + fwd[0] * 2, shoulder[1] + swordDir[1] * armLen * 0.85 + 3]
    const [ex, ey, hx, hy] = ik(shoulder[0], shoulder[1], target[0], target[1], spec.upper, spec.fore)
    const [ox1, oy1] = dir(pose.ou)
    const offElbow = [shoulder[0] - fwd[0] * 1.5 + ox1 * spec.upper, shoulder[1] + oy1 * spec.upper]
    const [ox2, oy2] = dir(pose.of)
    const offHand = [offElbow[0] + ox2 * spec.fore, offElbow[1] + oy2 * spec.fore]
    return {
      hip, neck, shoulder, head, headTilt, up, fwd, front: leg(pose.l1, pose.b1), back: leg(pose.l2, pose.b2),
      elbow: [ex, ey], hand: [hx, hy], offElbow, offHand, swordDir,
    }
  }

  // ---------------------------------------------------------------- drawing

  function limb(ctx, points, width) {
    ctx.lineWidth = width
    ctx.beginPath(); ctx.moveTo(points[0][0], points[0][1])
    for (let i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1])
    ctx.stroke()
  }

  function drawRobe(ctx, spec, s, pose, time, seed) {
    const { hip, neck, up, fwd } = s
    const flutter = pose.flutter
    const wave = Math.sin(time * 11 + seed) * (0.8 + flutter * 1.6)
    const long = spec.robe === 'long' ? 1 : 0
    const hemDepth = long ? 30 : spec.robe === 'hood' ? 13 : 14
    const backX = -9 - flutter * (long ? 16 : 9) - long * 3
    const frontX = 7 + Math.max(0, Math.sin(pose.l1)) * 7 + long * 4
    const sf = [neck[0] + fwd[0] * 4.6 - up[0] * 2, neck[1] + fwd[1] * 4.6 - up[1] * 2]
    const sb = [neck[0] - fwd[0] * 5.2 - up[0] * 1, neck[1] - fwd[1] * 5.2 - up[1] * 1]
    const wf = [hip[0] + fwd[0] * 5.5, hip[1] + fwd[1] * 5.5]
    const wb = [hip[0] - fwd[0] * 6.5, hip[1] - fwd[1] * 6.5]
    const hf = [hip[0] + frontX, hip[1] + hemDepth - 2]
    const hb = [hip[0] + backX, hip[1] + hemDepth - 4 + wave - flutter * 4]
    ctx.beginPath()
    ctx.moveTo(sb[0], sb[1])
    ctx.quadraticCurveTo(neck[0], neck[1] - 2, sf[0], sf[1])
    ctx.quadraticCurveTo(wf[0] + 1.5, wf[1] - 4, wf[0], wf[1])
    ctx.quadraticCurveTo(hf[0] - 1, (wf[1] + hf[1]) / 2, hf[0], hf[1])
    if (spec.robe === 'straw') {
      ctx.quadraticCurveTo((hf[0] + hb[0]) / 2, hip[1] + hemDepth + 4, hb[0], hb[1])
      ctx.quadraticCurveTo(wb[0] - 3, (wb[1] + hb[1]) / 2, wb[0], wb[1])
      ctx.quadraticCurveTo(sb[0] - 1.5, (sb[1] + wb[1]) / 2, sb[0], sb[1])
      ctx.closePath(); ctx.fill()
      for (let i = 0; i < 8; i++) {
        const t = i / 7
        const sx = lerp(sf[0] - 2, sb[0] - 4, t), sy = lerp(sf[1] + 5, sb[1] + 5, t)
        const ex = sx - 5 - flutter * 7 - (i % 3) * 3, ey = sy + 16 + (i % 2) * 5
        ctx.strokeStyle = 'rgba(20,20,22,0.82)'; ctx.lineWidth = 1.3
        ctx.beginPath(); ctx.moveTo(sx, sy); ctx.lineTo(ex, ey); ctx.stroke()
      }
    } else if (spec.robe === 'armor') {
      ctx.quadraticCurveTo((hf[0] + hb[0]) / 2, hip[1] + hemDepth, hb[0], hb[1])
      ctx.quadraticCurveTo(wb[0] - 3, (wb[1] + hb[1]) / 2, wb[0], wb[1])
      ctx.quadraticCurveTo(sb[0] - 1.5, (sb[1] + wb[1]) / 2, sb[0], sb[1])
      ctx.closePath(); ctx.fill()
      ctx.strokeStyle = 'rgba(235,225,210,0.32)'; ctx.lineWidth = 0.8
      for (let y = sf[1] + 6; y < hf[1]; y += 5) {
        ctx.beginPath(); ctx.moveTo(sb[0] + 2, y); ctx.lineTo(sf[0] - 2, y); ctx.stroke()
      }
    } else if (spec.robe === 'vest') {
      ctx.quadraticCurveTo((hf[0] + hb[0]) / 2, hip[1] + hemDepth - 4, hb[0], hb[1] - 4)
      ctx.quadraticCurveTo(wb[0] - 2, (wb[1] + hb[1]) / 2, wb[0], wb[1])
      ctx.quadraticCurveTo(sb[0] - 1, (sb[1] + wb[1]) / 2, sb[0], sb[1])
      ctx.closePath(); ctx.fill()
    } else if (spec.robe === 'rags' || spec.robe === 'long') {
      const teeth = spec.robe === 'long' ? 7 : 5
      for (let i = 1; i <= teeth; i++) {
        const t = i / teeth
        const x = lerp(hf[0], hb[0], t), y = lerp(hf[1], hb[1], t)
        const tooth = (i % 2 ? 1 : -1) * (spec.robe === 'long' ? 2.5 : 4) + Math.sin(time * 9 + i + seed) * flutter * 2
        ctx.lineTo(x - (hb[0] - hf[0]) / teeth / 2, y + tooth)
        ctx.lineTo(x, y)
      }
    } else {
      ctx.quadraticCurveTo((hf[0] + hb[0]) / 2, hip[1] + hemDepth + 2 + wave * 0.5, hb[0], hb[1])
    }
    ctx.quadraticCurveTo(wb[0] - 3 - flutter * 3, (wb[1] + hb[1]) / 2, wb[0], wb[1])
    ctx.quadraticCurveTo(sb[0] - 1.5, (sb[1] + wb[1]) / 2, sb[0], sb[1])
    ctx.closePath(); ctx.fill()
  }

  function drawHead(ctx, spec, s, look, time, seed) {
    const [hx, hy] = s.head
    const r = spec.head
    ctx.beginPath(); ctx.arc(hx, hy, r, 0, TAU); ctx.fill()
    if (spec.headStyle === 'blob') {
      for (let i = 0; i < 5; i++) {
        const a = -Math.PI * 0.9 + i * 0.42 + Math.sin(seed + i) * 0.2
        const l = r + 2.5 + ((seed * 7 + i * 3) % 3)
        ctx.beginPath()
        ctx.moveTo(hx + Math.cos(a - 0.3) * r * 0.8, hy + Math.sin(a - 0.3) * r * 0.8)
        ctx.lineTo(hx + Math.cos(a) * l, hy + Math.sin(a) * l)
        ctx.lineTo(hx + Math.cos(a + 0.3) * r * 0.8, hy + Math.sin(a + 0.3) * r * 0.8)
        ctx.fill()
      }
    }
    if (spec.headStyle === 'hood') {
      ctx.beginPath()
      ctx.moveTo(hx + r * 0.9, hy - r * 0.6)
      ctx.quadraticCurveTo(hx - r * 0.4, hy - r * 1.9, hx - r * 2.6, hy + r * 0.6)
      ctx.quadraticCurveTo(hx - r * 1.2, hy + r * 0.2, hx - r * 0.4, hy + r)
      ctx.closePath(); ctx.fill()
    }
    if (spec.beard) {
      ctx.fillStyle = '#f0ebe0'
      ctx.beginPath()
      ctx.moveTo(hx - r * 0.3, hy + r * 0.5)
      ctx.quadraticCurveTo(hx + r * 0.2, hy + r * 2.2, hx + r * 0.8, hy + r * 2.6)
      ctx.quadraticCurveTo(hx + r * 1.1, hy + r * 1.3, hx + r * 0.6, hy + r * 0.2)
      ctx.closePath(); ctx.fill()
    }
    if (spec.headStyle && !look.flash) eyes(ctx, hx + r * 0.42, hy - r * 0.1, look.eyeGlow || 0, spec.head)
  }

  function eyes(ctx, x, y, glow, size) {
    ctx.save()
    ctx.fillStyle = 'rgba(226,65,44,' + (0.12 + glow * 0.22) + ')'
    ctx.beginPath(); ctx.ellipse(x - size * 0.2, y, size * (0.42 + glow * 0.22), size * (0.26 + glow * 0.12), -0.2, 0, TAU); ctx.fill()
    ctx.fillStyle = EMBER
    ctx.beginPath(); ctx.ellipse(x, y, size * 0.22, size * 0.14, -0.2, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.ellipse(x - size * 0.48, y + size * 0.06, size * 0.15, size * 0.11, -0.2, 0, TAU); ctx.fill()
    ctx.restore()
  }

  function drawHat(ctx, spec, s, pose, time, look, seed) {
    const [hx, hy] = s.head
    if (spec.hat === 'helmet') {
      ctx.save(); ctx.translate(hx, hy - spec.head * 0.65); ctx.rotate(s.headTilt * 0.7)
      ctx.fillStyle = look.flash ? look.color : '#22252a'
      ctx.beginPath(); ctx.arc(0, 0, spec.head * 1.15, Math.PI, 0); ctx.fill()
      ctx.fillRect(-spec.head * 1.15, -1, spec.head * 2.3, 5)
      if (!look.flash) {
        ctx.fillStyle = CINNABAR
        ctx.beginPath(); ctx.moveTo(0, -spec.head * 1.15); ctx.quadraticCurveTo(-4, -spec.head * 2.1, -8, -spec.head * 2.5); ctx.quadraticCurveTo(0, -spec.head * 1.7, 0, -spec.head * 1.15); ctx.fill()
      }
      ctx.restore()
      return
    }
    const big = spec.hat === 'veil' ? 1.35 : 1
    ctx.save()
    ctx.translate(hx, hy - spec.head * 0.55)
    ctx.rotate(s.headTilt * 0.8)
    ctx.scale(big, big)
    if (spec.mask) {
      ctx.save()
      ctx.fillStyle = look.flash ? look.color : '#ece5d3'
      ctx.beginPath(); ctx.ellipse(1.5, 5.2, 3.9, 5, 0, 0, TAU); ctx.fill()
      if (!look.flash) {
        ctx.strokeStyle = CINNABAR; ctx.lineWidth = 0.9
        ctx.beginPath(); ctx.moveTo(0.4, 3.6); ctx.lineTo(3.1, 2.8); ctx.stroke()
        ctx.beginPath(); ctx.moveTo(-2.6, 3.4); ctx.lineTo(-0.9, 3.7); ctx.stroke()
        ctx.lineWidth = 0.7
        ctx.beginPath(); ctx.moveTo(-0.5, 8); ctx.quadraticCurveTo(1.5, 8.8, 3.5, 7.6); ctx.stroke()
        if (look.eyeGlow) {
          ctx.fillStyle = 'rgba(226,65,44,' + look.eyeGlow * 0.5 + ')'
          ctx.beginPath(); ctx.arc(1.8, 3.2, 3, 0, TAU); ctx.fill()
        }
      }
      ctx.restore()
    }
    ctx.beginPath()
    ctx.moveTo(-16, 2.6)
    ctx.quadraticCurveTo(-6, -2.5, 0, -7)
    ctx.quadraticCurveTo(6, -2.5, 16.5, 2.4)
    ctx.quadraticCurveTo(0, 4.4, -16, 2.6)
    ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = 'rgba(239,231,212,0.32)'; ctx.lineWidth = 0.55
      ctx.beginPath(); ctx.moveTo(-9, -0.6); ctx.quadraticCurveTo(0, -3.4, 9.5, -0.8); ctx.stroke()
      ctx.beginPath(); ctx.moveTo(-4, -4); ctx.quadraticCurveTo(0, -5.4, 4, -4.2); ctx.stroke()
    }
    if (spec.hat === 'veil') {
      ctx.globalAlpha *= 0.82
      for (let i = 0; i < 9; i++) {
        const x = -14 + i * 3.5
        if (x > -2 && x < 6) continue
        const sway = Math.sin(time * 4 + i * 0.9 + seed) * (1 + pose.flutter * 2.5) - pose.flutter * 4
        const len = 13 + ((i * 7) % 5)
        ctx.beginPath(); ctx.moveTo(x - 1.2, 2.8); ctx.lineTo(x + sway - 0.8, 2.8 + len); ctx.lineTo(x + sway + 0.8, 2.8 + len - 1); ctx.lineTo(x + 1.2, 2.8); ctx.fill()
      }
    }
    ctx.restore()
  }

  function drawBlade(ctx, spec, s, look, out) {
    if (!spec.blade) return
    const [hx, hy] = s.hand
    const [dx, dy] = s.swordDir
    const len = spec.blade, w = spec.bladeW
    const nx = -dy, ny = dx
    const tip = [hx + dx * len, hy + dy * len]
    out.tip = tip
    ctx.save()
    ctx.lineWidth = 1.6
    ctx.beginPath(); ctx.moveTo(hx - dx * 5, hy - dy * 5); ctx.lineTo(hx + dx * 1.5, hy + dy * 1.5); ctx.stroke()
    ctx.lineWidth = 1.4
    ctx.beginPath(); ctx.moveTo(hx + dx * 1.6 + nx * 3.4, hy + dy * 1.6 + ny * 3.4); ctx.lineTo(hx + dx * 1.6 - nx * 3.4, hy + dy * 1.6 - ny * 3.4); ctx.stroke()
    ctx.beginPath()
    ctx.moveTo(hx + dx * 2 + nx * w / 2, hy + dy * 2 + ny * w / 2)
    ctx.lineTo(hx + dx * len * 0.86 + nx * w * 0.45, hy + dy * len * 0.86 + ny * w * 0.45)
    ctx.lineTo(tip[0], tip[1])
    ctx.lineTo(hx + dx * 2 - nx * w / 2, hy + dy * 2 - ny * w / 2)
    ctx.closePath(); ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = look.edge || 'rgba(239,231,212,0.75)'
      ctx.lineWidth = 0.55
      ctx.beginPath(); ctx.moveTo(hx + dx * 4 + nx * w * 0.18, hy + dy * 4 + ny * w * 0.18); ctx.lineTo(hx + dx * len * 0.9, hy + dy * len * 0.9); ctx.stroke()
    }
    if (spec.mask && !look.flash) {
      ctx.strokeStyle = CINNABAR; ctx.lineWidth = 1.2
      ctx.beginPath(); ctx.moveTo(hx - dx * 5, hy - dy * 5); ctx.quadraticCurveTo(hx - dx * 7, hy - dy * 7 + 5, hx - dx * 6 - 2, hy - dy * 6 + 9); ctx.stroke()
    }
    ctx.restore()
  }

  function drawShield(ctx, s, look) {
    const [hx, hy] = s.offHand
    ctx.save()
    ctx.translate(hx + 2, hy - 2)
    ctx.rotate(0.06)
    ctx.beginPath(); ctx.roundRect(-5.5, -24, 11, 46, 3); ctx.fill()
    if (!look.flash) {
      ctx.fillStyle = '#5f5c55'
      ctx.beginPath(); ctx.roundRect(-3.8, -22, 7.6, 42, 2); ctx.fill()
      ctx.strokeStyle = 'rgba(20,20,22,0.7)'; ctx.lineWidth = 0.7
      for (const y of [-12, -1, 10]) { ctx.beginPath(); ctx.moveTo(-3.8, y); ctx.lineTo(3.8, y + 0.6); ctx.stroke() }
      ctx.fillStyle = '#e9e0c8'; ctx.fillRect(-1.6, -18, 3.2, 12)
      ctx.strokeStyle = CINNABAR; ctx.lineWidth = 0.7
      ctx.beginPath(); ctx.moveTo(0, -16.5); ctx.lineTo(0, -7.5); ctx.moveTo(-1, -14); ctx.lineTo(1, -13); ctx.moveTo(-1, -11); ctx.lineTo(1, -10); ctx.stroke()
    }
    ctx.restore()
  }

  function drawBow(ctx, s, pose, look, out) {
    const [bx, by] = s.offHand
    const draw = pose.draw
    const top = [bx - 4, by - 19], bottom = [bx - 4, by + 19]
    ctx.save()
    ctx.lineWidth = 2.1
    ctx.beginPath(); ctx.moveTo(top[0], top[1]); ctx.quadraticCurveTo(bx + 9, by, bottom[0], bottom[1]); ctx.stroke()
    const pull = [bx - 6 - draw * 14, by]
    ctx.lineWidth = 0.5
    ctx.strokeStyle = look.flash ? look.color : 'rgba(60,62,66,0.9)'
    ctx.beginPath(); ctx.moveTo(top[0], top[1]); ctx.lineTo(pull[0], pull[1]); ctx.lineTo(bottom[0], bottom[1]); ctx.stroke()
    if (draw > 0.05) {
      ctx.strokeStyle = look.color; ctx.lineWidth = 0.9
      ctx.beginPath(); ctx.moveTo(pull[0], pull[1]); ctx.lineTo(bx + 10, by); ctx.stroke()
      ctx.fillStyle = CINNABAR
      ctx.beginPath(); ctx.moveTo(bx + 13, by); ctx.lineTo(bx + 8.5, by - 1.6); ctx.lineTo(bx + 8.5, by + 1.6); ctx.fill()
      out.tip = [bx + 13, by]
    }
    ctx.restore()
    return pull
  }

  function drawSpear(ctx, spec, s, look, out) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.swordDir
    const len = 74, shaftLen = 56
    const nx = -dy, ny = dx
    const tip = [hx + dx * len, hy + dy * len]
    out.tip = tip
    ctx.save()
    ctx.strokeStyle = '#28201a'; ctx.lineWidth = 2.4
    ctx.beginPath(); ctx.moveTo(hx - dx * 16, hy - dy * 16); ctx.lineTo(hx + dx * shaftLen, hy + dy * shaftLen); ctx.stroke()
    if (!look.flash) {
      ctx.fillStyle = CINNABAR
      ctx.beginPath()
      ctx.moveTo(hx + dx * (shaftLen - 2), hy + dy * (shaftLen - 2))
      ctx.quadraticCurveTo(hx + dx * (shaftLen + 6) + nx * 5, hy + dy * (shaftLen + 6) + ny * 5, hx + dx * (shaftLen + 10), hy + dy * (shaftLen + 10))
      ctx.quadraticCurveTo(hx + dx * (shaftLen + 4) - nx * 4, hy + dy * (shaftLen + 4) - ny * 4, hx + dx * (shaftLen - 2), hy + dy * (shaftLen - 2))
      ctx.fill()
    }
    ctx.fillStyle = look.flash ? look.color : '#e5dfd2'
    ctx.beginPath()
    ctx.moveTo(hx + dx * shaftLen + nx * 3.5, hy + dy * shaftLen + ny * 3.5)
    ctx.lineTo(tip[0], tip[1])
    ctx.lineTo(hx + dx * shaftLen - nx * 3.5, hy + dy * shaftLen - ny * 3.5)
    ctx.closePath(); ctx.fill()
    ctx.restore()
  }

  function drawHammer(ctx, spec, s, look, out) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.swordDir
    const len = 44
    const headX = hx + dx * len, headY = hy + dy * len
    out.tip = [headX + dx * 14, headY + dy * 14]
    ctx.save()
    ctx.strokeStyle = '#322c26'; ctx.lineWidth = 3.6
    ctx.beginPath(); ctx.moveTo(hx - dx * 10, hy - dy * 10); ctx.lineTo(headX, headY); ctx.stroke()
    ctx.translate(headX, headY)
    const angle = Math.atan2(dy, dx)
    ctx.rotate(angle)
    ctx.fillStyle = look.flash ? look.color : '#40434a'
    ctx.beginPath(); ctx.roundRect(-6, -18, 22, 36, 3); ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = 'rgba(235,228,215,0.3)'; ctx.lineWidth = 0.8
      ctx.beginPath(); ctx.moveTo(2, -16); ctx.lineTo(14, -16); ctx.moveTo(2, 16); ctx.lineTo(14, 16); ctx.stroke()
    }
    ctx.restore()
  }

  function drawRod(ctx, spec, s, look, out) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.swordDir
    const rodLen = 58
    const tipX = hx + dx * rodLen - dy * 6, tipY = hy + dy * rodLen + dx * 6
    out.tip = [tipX + dx * 40, tipY + dy * 20]
    ctx.save()
    ctx.strokeStyle = '#2b3322'; ctx.lineWidth = 1.8
    ctx.beginPath()
    ctx.moveTo(hx - dx * 6, hy - dy * 6)
    ctx.quadraticCurveTo(hx + dx * rodLen * 0.6, hy + dy * rodLen * 0.6, tipX, tipY)
    ctx.stroke()
    ctx.strokeStyle = look.flash ? look.color : 'rgba(235,230,220,0.7)'; ctx.lineWidth = 0.8
    const hookX = out.tip[0], hookY = out.tip[1]
    ctx.beginPath(); ctx.moveTo(tipX, tipY); ctx.quadraticCurveTo((tipX + hookX) / 2 + 5, (tipY + hookY) / 2 + 8, hookX, hookY); ctx.stroke()
    ctx.fillStyle = CINNABAR
    ctx.beginPath(); ctx.arc(hookX, hookY, 2.2, 0, TAU); ctx.fill()
    ctx.restore()
  }

  function drawSleeve(ctx, from, to, pose, time, seed, back) {
    const flutter = pose.flutter
    const dx = to[0] - from[0], dy = to[1] - from[1]
    const sway = Math.sin(time * 5 + seed + (back ? 1 : 0)) * 1.5
    ctx.beginPath()
    ctx.moveTo(from[0], from[1] - 1.5)
    ctx.lineTo(to[0] - dx * 0.1, to[1] - 2.5)
    ctx.quadraticCurveTo(to[0] - 3 - flutter * 6, to[1] + 9 + sway, to[0] - 9 - flutter * 12, to[1] + 12 + sway - flutter * 4)
    ctx.quadraticCurveTo(from[0] - 2 - flutter * 4, from[1] + 10, from[0] - 1, from[1] + 2)
    ctx.closePath(); ctx.fill()
    void dy
  }

  // Draws a posed humanoid in world space. `look` = { color, alpha, flash, eyeGlow, halo }.
  function drawHumanoid(ctx, kind, pose, x, y, facing, look, time, seed = 0, extras = {}) {
    const spec = KINDS[kind]
    const s = skeleton(spec, pose)
    const out = { tip: null, head: [x + s.head[0] * facing, y + s.head[1]], hand: null }
    ctx.save()
    ctx.translate(x, y)
    ctx.scale(facing, 1)
    if (look.alpha !== undefined) ctx.globalAlpha *= look.alpha
    ctx.lineCap = 'round'; ctx.lineJoin = 'round'
    const color = look.color || spec.color
    if (extras.scarf) drawScarf(ctx, extras.scarf, x, y, facing, look)
    if (pose.rot) {
      const pivotY = s.hip[1] + 4
      ctx.translate(0, pivotY); ctx.rotate(pose.rot); ctx.translate(0, -pivotY)
    }
    ctx.fillStyle = color; ctx.strokeStyle = color
    const [lu, lf, au, af] = spec.limbs

    // Wet-ink halo keeps silhouettes legible over busy washes.
    if (look.halo) {
      ctx.save()
      ctx.globalAlpha *= look.halo
      ctx.fillStyle = 'rgba(239,231,212,1)'
      ctx.beginPath(); ctx.ellipse(s.hip[0], s.hip[1] - spec.torso * 0.35, spec.torso * 0.9, spec.torso * 1.35, pose.lean, 0, TAU); ctx.fill()
      ctx.restore()
      ctx.fillStyle = color
    }

    limb(ctx, [s.shoulder, s.offElbow, s.offHand], au * 0.92)
    if (spec.sleeves) drawSleeve(ctx, s.shoulder, s.offElbow, pose, time, seed, true)
    limb(ctx, [s.hip, s.back.knee, s.back.foot], lu)
    ctx.lineWidth = lf
    ctx.beginPath(); ctx.moveTo(s.back.foot[0] - 1.5, s.back.foot[1]); ctx.lineTo(s.back.foot[0] + 3.5, s.back.foot[1] + 0.3); ctx.stroke()
    limb(ctx, [s.hip, s.front.knee, s.front.foot], lu)
    ctx.beginPath(); ctx.moveTo(s.front.foot[0] - 1.5, s.front.foot[1]); ctx.lineTo(s.front.foot[0] + 4, s.front.foot[1] + 0.3); ctx.stroke()
    void lf
    drawRobe(ctx, spec, s, pose, time, seed)
    if (kind === 'player' && !look.flash) {
      ctx.save()
      ctx.strokeStyle = CINNABAR; ctx.lineWidth = 2.1
      const wf = [s.hip[0] + s.fwd[0] * 5.6, s.hip[1] + s.fwd[1] * 5.6 - 2], wb = [s.hip[0] - s.fwd[0] * 6.6, s.hip[1] - s.fwd[1] * 6.6 - 2]
      ctx.beginPath(); ctx.moveTo(wb[0], wb[1]); ctx.lineTo(wf[0], wf[1]); ctx.stroke()
      ctx.lineWidth = 1.2
      const sway = Math.sin(time * 9) * 1.5 - pose.flutter * 5
      ctx.beginPath(); ctx.moveTo(wb[0], wb[1]); ctx.quadraticCurveTo(wb[0] - 4, wb[1] + 5, wb[0] - 6 + sway, wb[1] + 10); ctx.stroke()
      ctx.restore()
    }
    drawHead(ctx, spec, s, look, time, seed)
    if (spec.hat) drawHat(ctx, spec, s, pose, time, look, seed)
    if (spec.shield) drawShield(ctx, s, look)
    if (spec.bow) {
      const pull = drawBow(ctx, s, pose, look, out)
      limb(ctx, [s.shoulder, [lerp(s.shoulder[0], pull[0], 0.5) - 2, lerp(s.shoulder[1], pull[1], 0.5) + 3], pull], af)
    } else {
      limb(ctx, [s.shoulder, s.elbow, s.hand], au)
      if (spec.sleeves) drawSleeve(ctx, s.shoulder, s.elbow, pose, time, seed, false)
      ctx.lineWidth = af
      ctx.beginPath(); ctx.moveTo(s.elbow[0], s.elbow[1]); ctx.lineTo(s.hand[0], s.hand[1]); ctx.stroke()
      if (spec.spear) drawSpear(ctx, spec, s, look, out)
      else if (spec.hammer) drawHammer(ctx, spec, s, look, out)
      else if (spec.rod) drawRod(ctx, spec, s, look, out)
      else drawBlade(ctx, spec, s, look, out)
    }
    ctx.restore()
    const map = (p) => p && [x + p[0] * facing, y + p[1]]
    out.tip = map(out.tip)
    out.hand = map(s.hand)
    out.neck = map(s.neck)
    out.hip = map(s.hip)
    return out
  }

  // ---------------------------------------------------------------- scarf

  function updateScarf(anim, anchorX, anchorY, facing, vx, dt, time, wind) {
    const count = 9, length = 5.6
    if (!anim.scarf) {
      anim.scarf = []
      for (let i = 0; i < count; i++) anim.scarf.push({ x: anchorX - facing * i * length, y: anchorY + i * 1.5, px: anchorX - facing * i * length, py: anchorY + i * 1.5 })
    }
    const pts = anim.scarf
    pts[0].x = anchorX; pts[0].y = anchorY; pts[0].px = anchorX; pts[0].py = anchorY
    const step = Math.min(dt, 1 / 30)
    for (let i = 1; i < count; i++) {
      const p = pts[i]
      const vxp = (p.x - p.px) * 0.9, vyp = (p.y - p.py) * 0.9
      p.px = p.x; p.py = p.y
      const gust = Math.sin(time * 3.3 + i * 0.7) * 30 + wind
      p.x += vxp * 0.92 + gust * step * step * 22
      p.y += vyp * 0.92 + (240 + Math.sin(time * 5 + i) * 60) * step * step * 9
    }
    // Follow-the-leader pass keeps every link at its rest length, however strong the gust.
    for (let i = 1; i < count; i++) {
      const a = pts[i - 1], b = pts[i]
      const dx = b.x - a.x, dy = b.y - a.y, d = Math.hypot(dx, dy) || 0.001
      b.x = a.x + dx / d * length; b.y = a.y + dy / d * length
    }
    void vx; void facing
  }

  function drawScarf(ctx, pts, x, y, facing, look) {
    if (look.flash) return
    // Points are in world space; undo the figure's local transform.
    ctx.save()
    ctx.scale(facing, 1)
    ctx.translate(-x, -y)
    ctx.fillStyle = CINNABAR
    const left = [], right = []
    for (let i = 0; i < pts.length; i++) {
      const a = pts[Math.max(0, i - 1)], b = pts[Math.min(pts.length - 1, i + 1)]
      const dx = b.x - a.x, dy = b.y - a.y, l = Math.hypot(dx, dy) || 1
      const w = lerp(2.2, 0.5, i / (pts.length - 1)) * (1 + Math.sin(i * 1.7) * 0.15)
      left.push([pts[i].x - dy / l * w, pts[i].y + dx / l * w])
      right.push([pts[i].x + dy / l * w, pts[i].y - dx / l * w])
    }
    ctx.beginPath(); ctx.moveTo(left[0][0], left[0][1])
    for (const p of left) ctx.lineTo(p[0], p[1])
    for (let i = right.length - 1; i >= 0; i--) ctx.lineTo(right[i][0], right[i][1])
    ctx.closePath(); ctx.fill()
    ctx.restore()
  }

  // ---------------------------------------------------------------- crow & dummy

  function drawCrow(ctx, crow, x, y, look, time) {
    const facing = crow.facing || 1
    ctx.save()
    ctx.translate(x, y)
    ctx.scale(facing, 1)
    if (look.alpha !== undefined) ctx.globalAlpha *= look.alpha
    const color = look.color || '#16171a'
    ctx.fillStyle = color; ctx.strokeStyle = color
    let body = 0, wing = Math.sin(time * 15 + (crow.seed || 0) * 4)
    let spread = 1
    if (crow.state === 'tele') { wing = -1 + Math.sin(time * 40) * 0.15; spread = 1.3 }
    if (crow.state === 'dive') { body = Math.atan2(crow.vy, Math.abs(crow.vx) + 1) * 0.8; wing = 0.9; spread = 0.55 }
    ctx.rotate(body)
    const wingPath = (lift, scale) => {
      const tipX = -10 * scale, tipY = -20 * lift * scale * spread
      ctx.beginPath()
      ctx.moveTo(3, -1)
      ctx.quadraticCurveTo(-2, tipY * 0.6 - 2, tipX, tipY)
      ctx.lineTo(tipX + 2.5, tipY + 4 * Math.sign(lift || 1))
      ctx.lineTo(tipX - 1, tipY + 6 * Math.sign(lift || 1))
      ctx.lineTo(tipX + 3, tipY + 8 * Math.sign(lift || 1))
      ctx.quadraticCurveTo(-4, 1, -6, 1)
      ctx.closePath(); ctx.fill()
    }
    ctx.save(); ctx.globalAlpha *= 0.6; wingPath(wing * 0.85, 0.9); ctx.restore()
    ctx.beginPath(); ctx.ellipse(0, 0, 10, 5, 0, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.arc(8.5, -2.5, 3.8, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.moveTo(11, -3.5); ctx.lineTo(17, -1.2); ctx.lineTo(11, -0.6); ctx.fill()
    ctx.beginPath(); ctx.moveTo(-8, -1); ctx.lineTo(-17, -4); ctx.lineTo(-18, 1); ctx.lineTo(-16, 4); ctx.lineTo(-8, 2.5); ctx.fill()
    wingPath(wing, 1)
    if (!look.flash) {
      const glow = crow.state === 'tele' ? 1 : 0.3
      ctx.fillStyle = 'rgba(226,65,44,' + (0.25 + glow * 0.35) + ')'
      ctx.beginPath(); ctx.arc(9.6, -3.4, 1.8 + glow * 1.8, 0, TAU); ctx.fill()
      ctx.fillStyle = EMBER
      ctx.beginPath(); ctx.arc(9.6, -3.4, 0.9, 0, TAU); ctx.fill()
    }
    ctx.restore()
    return { tip: [x + 15 * facing, y - 2], head: [x, y] }
  }

  function drawDummy(ctx, dummy, x, y, look, anim, time) {
    ctx.save()
    ctx.translate(x, y)
    ctx.rotate(anim.wobble * 0.5)
    if (look.alpha !== undefined) ctx.globalAlpha *= look.alpha
    const color = look.color || '#3b3832'
    ctx.strokeStyle = color; ctx.fillStyle = color; ctx.lineCap = 'round'
    ctx.lineWidth = 3.2
    ctx.beginPath(); ctx.moveTo(0, 2); ctx.lineTo(0, -46); ctx.stroke()
    ctx.lineWidth = 2.4
    ctx.beginPath(); ctx.moveTo(-15, -35); ctx.quadraticCurveTo(0, -38, 15, -34); ctx.stroke()
    ctx.fillStyle = look.flash ? color : '#5b5345'
    ctx.beginPath(); ctx.ellipse(0, -28, 8, 13, 0, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.arc(0, -47, 6, 0, TAU); ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = 'rgba(30,28,24,0.7)'; ctx.lineWidth = 0.6
      for (let i = -3; i <= 3; i++) {
        ctx.beginPath(); ctx.moveTo(i * 2, -40); ctx.quadraticCurveTo(i * 2.6, -28, i * 1.6, -15); ctx.stroke()
      }
      for (const sx of [-1, 1]) for (let i = 0; i < 3; i++) {
        ctx.beginPath(); ctx.moveTo(sx * 15, -35 + i * 1.2); ctx.lineTo(sx * (19 + i), -31 + i * 2.5); ctx.stroke()
      }
      ctx.lineWidth = 0.9; ctx.beginPath(); ctx.moveTo(-6, -40); ctx.lineTo(6, -40); ctx.moveTo(-6, -17); ctx.lineTo(6, -17); ctx.stroke()
      ctx.fillStyle = '#ebe1c8'; ctx.fillRect(-2.8, -33, 5.6, 11)
      ctx.strokeStyle = CINNABAR; ctx.lineWidth = 0.8
      ctx.beginPath(); ctx.moveTo(0, -31.5); ctx.lineTo(0, -23.5); ctx.moveTo(-1.6, -29); ctx.lineTo(1.6, -28); ctx.moveTo(-1.6, -26); ctx.lineTo(1.6, -25); ctx.stroke()
    }
    ctx.restore()
    void time
    return { tip: null, head: [x, y - 47] }
  }

  function updateDummy(anim, dt) {
    anim.wobbleV += (-anim.wobble * 90 - anim.wobbleV * 6) * dt
    anim.wobble += anim.wobbleV * dt
  }

  globalThis.XiabanInkFigures = {
    KINDS, CINNABAR, PAPER, createAnim, updatePose, drawHumanoid, updateScarf, drawCrow, drawDummy, updateDummy, basePose, skeleton,
  }
})()
