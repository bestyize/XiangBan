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
    spearman: { thigh: 14, shin: 14, torso: 22, upper: 12, fore: 11, head: 5.6, limbs: [6.2, 5.2, 5.2, 4.2], blade: 0, color: '#1b1d22', robe: 'armor', hat: 'helmet', spear: true, grip: { span: 13 } },
    hammer: { thigh: 16, shin: 15, torso: 26, upper: 14, fore: 13, head: 7.2, limbs: [8.5, 7.2, 7.5, 6.2], blade: 0, color: '#202228', robe: 'vest', headStyle: 'blob', hammer: true, grip: { span: 11 } },
    soldier: { thigh: 13, shin: 13, torso: 20, upper: 10.5, fore: 10, head: 5.5, limbs: [5.6, 4.6, 4.6, 3.8], blade: 30, bladeW: 3.4, color: '#202226', robe: 'armor', hat: 'kettle', headStyle: 'plain' },
    footman: { thigh: 13, shin: 13, torso: 20, upper: 10.5, fore: 10, head: 5.5, limbs: [5.6, 4.6, 4.6, 3.8], blade: 31, bladeW: 4.4, color: '#1c1e22', robe: 'armor', hat: 'cavhelm', headStyle: 'plain', sabre: true, cape: 18 },
    bomber: { thigh: 12.5, shin: 12.5, torso: 19, upper: 10, fore: 9.5, head: 5.4, limbs: [5.4, 4.5, 4.6, 3.8], blade: 0, color: '#25262a', robe: 'padded', hat: 'wrap', headStyle: 'plain', pot: true, satchel: true },
    pavise: { thigh: 14, shin: 13, torso: 22, upper: 12, fore: 11, head: 5.6, limbs: [6.4, 5.4, 5.6, 4.6], blade: 0, color: '#1e2024', robe: 'armor', hat: 'kettle', headStyle: 'plain', pavise: true, shortSpear: true },
    general: { thigh: 18, shin: 18, torso: 26, upper: 14, fore: 13, head: 6.6, limbs: [8.2, 6.8, 7.2, 5.8], blade: 0, color: '#0d0e10', robe: 'lamellar', hat: 'warhelm', headStyle: 'plain', broadsword: true, backFlags: true, cape: 28, grip: { span: 15 } },
    angler: { thigh: 16, shin: 16, torso: 24, upper: 12, fore: 11, head: 6.2, limbs: [6.2, 5.2, 5.4, 4.4], blade: 0, color: '#121316', robe: 'straw', hat: 'douli', beard: true, rod: true, grip: { span: 12 } },
    pirate: { thigh: 13, shin: 13, torso: 20, upper: 11, fore: 10, head: 5.6, limbs: [5.8, 4.7, 4.8, 3.8], blade: 32, bladeW: 4, color: '#202a2d', robe: 'rags', hat: null, headStyle: 'plain', sabre: true, seaRaider: true },
    hooker: { thigh: 14, shin: 13, torso: 22, upper: 12, fore: 11, head: 5.8, limbs: [6.2, 5, 5.3, 4.2], blade: 0, color: '#16272b', robe: 'straw', hat: null, hook: true, seaRaider: true, grip: { span: 13 } },
    lantern: {thigh:15,shin:14,torso:24,upper:12,fore:12,head:5.2,limbs:[5.3,4.4,4.1,3.6],blade:36,bladeW:3.5,color:'#242823',robe:'long',headStyle:'plain',city:true},
    mirror: {thigh:14,shin:14,torso:23,upper:12,fore:11,head:5.3,limbs:[4.8,4,3.6,3.2],blade:25,bladeW:2.3,color:'#242928',robe:'long',headStyle:'plain',city:true},
    watchman: {thigh:16,shin:16,torso:27,upper:13,fore:12,head:6.5,limbs:[7.7,6,6,5],blade:0,color:'#33382d',robe:'armor',spear:true,grip:{span:14},city:true},
    judge: {thigh:19,shin:18,torso:30,upper:15,fore:14,head:6.2,limbs:[6.7,5.6,5.4,4.7],blade:0,color:'#1b211b',robe:'long',headStyle:'plain',city:true},
    umbrella:{thigh:14,shin:14,torso:22,upper:12,fore:11,head:5.5,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true},
    monk:{thigh:16,shin:15,torso:24,upper:13,fore:12,head:5.8,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true,grip:{span:13}},
    disciple:{thigh:13,shin:13,torso:19,upper:11,fore:10,head:5.2,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true},
    rope:{thigh:14,shin:13,torso:20,upper:11,fore:11,head:5.2,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true},
    crossbow:{thigh:14,shin:14,torso:21,upper:12,fore:11,head:5.4,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true},
    axeman:{thigh:16,shin:15,torso:26,upper:14,fore:13,head:6.2,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true,grip:{span:13}},
    yanhui:{thigh:16,shin:16,torso:24,upper:13,fore:12,head:5.8,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true},
    shizhen:{thigh:18,shin:17,torso:28,upper:14,fore:13,head:6.5,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true,grip:{span:13}},
    yue:{thigh:18,shin:18,torso:27,upper:14,fore:13,head:6.2,limbs:[6,5,5,4],blade:0,color:'#292e28',robe:'robe',summit:true,grip:{span:13}},
  }

  const POSE_KEYS = ['lean', 'crouch', 'l1', 'b1', 'l2', 'b2', 'ou', 'of', 'sw', 'reach', 'head', 'rot', 'bob', 'draw', 'air', 'flutter', 'wrist', 'one', 'flag', 'pot', 'sh', 'cast']

  function basePose() {
    return { lean: 0.06, crouch: 1, l1: 0.18, b1: 0.3, l2: -0.16, b2: 0.22, ou: 0.1, of: -0.4, sw: 2.45, reach: 0.75, head: 0, rot: 0, bob: 0, draw: 0, air: 0, flutter: 0, wrist: 0, one: 0, flag: 0, pot: 0, sh: 0, cast: 0 }
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
      if (p.vy < 0) Object.assign(t, { lean: 0.14, l1: 0.75, b1: 1.35, l2: -0.12, b2: 0.85, sw: 3.0, reach: 0.85, ou: -0.5, of: -1, wrist: -0.35 })
      else Object.assign(t, { lean: 0.05, l1: 0.35, b1: 0.75, l2: -0.32, b2: 0.55, sw: 2.6, reach: 0.85, ou: 2.1, of: 2.5, wrist: -0.5 })
      if (anim.flip > 0) t.rot = (1 - anim.flip) * TAU
      if (anim.parry > 0) { t.sw = -1.25; t.reach = 1 }
      return t
    }
    if (speed > 25) {
      runLegs(t, anim.phase, clamp(speed / 240, 0.35, 1))
      t.lean = 0.24 + t.flutter * 0.12
      t.crouch = 2
      t.sw = 2.55; t.reach = 0.85; t.wrist = -0.42
      t.ou = -Math.sin(anim.phase) * 0.9; t.of = t.ou - 0.9
    } else {
      const breathe = Math.sin(time * 1.8)
      t.lean = 0.06 + breathe * 0.02
      t.crouch = 1.2 + breathe * 0.6
      t.sw = 0.95; t.reach = 0.9; t.wrist = -0.28
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
    if (KINDS[e.type]?.summit) return globalThis.XiabanInkMountainFigures.pose(e,t,anim,time,progress,speed)
    if (KINDS[e.type]?.city) return cityPose(e,t,anim,time,progress,speed)
    if (e.type === 'archer') { t.lean = 0.08; t.ou = 1.5; t.of = 1.52; t.sw = 0 }
    if (e.type === 'shield') { t.ou = 0.95; t.of = 1.45; t.lean = 0.22 }
    if (e.type === 'spearman') { t.sw = -0.15; t.reach = 1.1; t.lean = 0.28 }
    if (e.type === 'hammer') { t.sw = -0.6; t.reach = 0.95; t.lean = 0.25; t.crouch = 4 }
    if (e.type === 'ronin') { t.lean = 0.42; t.crouch = 3.5; t.sw = 1.2; t.wrist = -0.35 }
    if (e.type === 'puppet' || e.type === 'shield') t.wrist = -0.25
    if (e.type === 'soldier' || e.type === 'footman') { t.wrist = -0.3; t.lean = 0.3 }
    if (e.type === 'pirate') { t.wrist = -0.45; t.lean = 0.4; t.sw = 1.1 }
    if (e.type === 'hooker') { t.lean = 0.2; t.sw = -0.25; t.reach = 1.15; t.crouch = 4 }
    if (e.type === 'bomber') { t.lean = 0.22; t.sw = 0.9; t.reach = 0.85; t.ou = 1.2; t.of = 1.0; t.pot = 1 }
    if (e.type === 'pavise') {
      t.lean = 0.14; t.crouch = 5; t.sw = 0.6; t.reach = 0.75; t.wrist = -1.5
      Object.assign(t, { l1: 0.38, b1: 0.45, l2: -0.32, b2: 0.28 })
      return paviseState(e, t, anim, time, progress, speed)
    }
    if (e.type === 'pirate' || e.type === 'hooker') return seaRaiderPose(e, t, anim, time, progress, speed)
    if (e.type === 'bomber') return bomberState(e, t, anim, time, progress, speed)
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

  function cityPose(e,t,anim,time,q,speed) {
    const judge=e.type==='judge',guard=e.type==='watchman'
    t.lean=judge?0.02:guard?0.1:0.18;t.crouch=1;t.sw=judge?0.65:guard?-0.15:1.2;t.ou=-0.5;t.of=-0.7;t.reach=1
    if(e.type==='mirror'){t.ou=-1.5;t.of=-.8;t.lean=.1}
    if(e.state==='walk'||(e.state==='idle'&&speed>10))runLegs(t,anim.phase,Math.min(.65,speed/170))
    if(e.state==='tele'||e.state==='judgeTell'){t.sw=judge?-2.55:guard?-.2:-2.2;t.lean=-.15;t.crouch=3+q*3;t.reach=guard?1.3:1;t.l1=.5;t.l2=-.4;t.ou=-.7;t.of=-1.1}
    if(e.state==='attack'||e.state==='judgeAttack'){t.sw=guard?0:judge&&e.judgeStage<3?-.12:.75;t.lean=.63;t.crouch=5;t.reach=1.3;t.l1=.8;t.l2=-.65;anim.snap=true}
    if(e.state==='judgeWrite'){t.sw=-1.7+q*2.2;t.lean=.05+q*.35;t.ou=-1.9;t.of=-2.2;t.l1=.45;t.l2=-.4;t.crouch=3;t.reach=1.25;anim.snap=true}
    if(e.state==='judgeMirror'){t.sw=-1.1;t.ou=-2.2;t.of=-2.6;t.lean=-.1}
    if(e.state==='recover'){t.sw=1.05;t.lean=.45;t.crouch=5;t.ou=.4;t.of=.9}
    if(e.state==='hurt'||e.state==='stagger'||e.state==='broken'){t.lean=-.48;t.crouch=e.state==='broken'?14:8;t.sw=-2.5;t.head=-.4;t.ou=2;t.of=2.6;t.l1=.6;t.l2=-.3}
    if(e.state==='roar'){t.sw=-2;t.ou=-2.4;t.of=-2.8;t.lean=-.15}
    return t
  }

  function seaRaiderPose(e, t, anim, time, q, speed) {
    const hook=e.type==='hooker'
    t.sw=hook?-0.45:0.9;t.ou=-0.8;t.of=-1.4;t.lean=hook?0.12:0.3;t.crouch=hook?2:5
    if(e.state==='walk') {runLegs(t,anim.phase,clamp(speed/160,0.3,0.8));t.ou=-Math.sin(anim.phase)*0.6;t.of=t.ou-0.9}
    if(e.state==='tele') {
      t.sw=hook?-1.3+q*0.3:e.chain?1.9:-2.5
      t.ou=e.chain?-1.9:0.8;t.of=e.chain?-2.6:1.4;t.lean=-0.18;t.crouch=5+q*4
      t.l1=0.6;t.l2=-0.5;t.b1=0.55;t.b2=0.2;t.reach=1
      t.cast=hook?0.15+q*0.25:0
    } else if(e.state==='attack') {
      t.sw=hook?-0.03:e.chain?-0.65:1.3;t.lean=0.7;t.crouch=6;t.reach=1.25
      t.l1=0.9;t.l2=-0.8;t.b1=0.55;t.b2=0.15;t.ou=-0.7;t.of=-1.3;t.cast=hook?1:0;anim.snap=true
    } else if(e.state==='recover') {t.lean=0.65;t.sw=0.8;t.crouch=6;t.ou=0.8;t.of=1.7}
    else if(e.state==='hurt'||e.state==='stagger') {t.lean=-0.6;t.crouch=7;t.sw=-2.2;t.ou=2.3;t.of=2.8;t.l1=0.5;t.l2=-0.4;t.head=-0.3}
    else if(e.state==='spawn') {t.lean=0.65;t.crouch=12}
    return t
  }

  function bomberState(e, t, anim, time, progress, speed) {
    switch (e.state) {
      case 'spawn': t.crouch = 8; t.lean = 0.5; break
      case 'walk': runLegs(t, anim.phase, clamp(speed / 150, 0.4, 0.9)); t.lean = 0.3; t.crouch = 3.5; break
      case 'hop': t.air = 1; Object.assign(t, { l1: 0.7, b1: 1.3, l2: -0.1, b2: 0.9, lean: -0.1 }); break
      case 'tele':
        t.sw = lerp(0.9, -2.35, ease(progress * 1.3)); t.reach = 1; t.lean = lerp(0.3, -0.2, ease(progress)); t.crouch = 3 + progress * 3
        t.ou = 1.4; t.of = 1.25
        Object.assign(t, { l1: 0.5, b1: 0.45, l2: -0.5, b2: 0.2 })
        break
      case 'recover':
        t.sw = 1.15; t.reach = 1.25; t.lean = 0.66; t.crouch = 5; t.pot = 0; t.ou = -0.5; t.of = -0.9
        Object.assign(t, { l1: 0.7, b1: 0.4, l2: -0.6, b2: 0.15 })
        break
      case 'hurt': t.lean = -0.38; t.sw = -1.8; t.ou = 2; t.of = 2.4; t.crouch = 2; t.head = -0.3; t.pot = 0; break
      case 'stagger':
        t.lean = -0.55 + Math.sin(time * 9) * 0.06; t.sw = -2.7; t.reach = 1; t.ou = 2.6; t.of = 3; t.crouch = 5; t.head = -0.45; t.pot = 0
        Object.assign(t, { l1: 0.5, b1: 0.6, l2: -0.35, b2: 0.3 })
        break
    }
    return t
  }

  // Pavise bearer: the shield stands braced; `sh` swings it open, `wrist` keeps the spear axis independent of the arm.
  function paviseState(e, t, anim, time, progress, speed) {
    switch (e.state) {
      case 'spawn': t.crouch = 8; t.lean = 0.4; break
      case 'walk': runLegs(t, anim.phase, clamp(speed / 150, 0.3, 0.6)); t.lean = 0.2; break
      case 'tele': {
        const q = ease(progress * 2)
        t.sw = lerp(0.6, 2.7, q); t.reach = lerp(0.75, 0.9, q); t.wrist = (lerp(-0.7, 0, q)) - t.sw; t.lean = lerp(0.14, -0.02, q); t.crouch = 5 + progress * 2
        Object.assign(t, { l1: 0.5, b1: 0.45, l2: -0.45, b2: 0.25 })
        break
      }
      case 'attack':
        t.sw = 0.12; t.reach = 1.4; t.wrist = -0.12; t.lean = 0.45; t.crouch = 6; anim.snap = true
        Object.assign(t, { l1: 0.75, b1: 0.4, l2: -0.7, b2: 0.1 })
        break
      case 'recover': t.sw = 0.5; t.reach = 1.1; t.wrist = -0.55; t.lean = 0.3; t.crouch = 5; break
      case 'hurt': t.lean = -0.3; t.sh = 0.25; t.sw = 0.2; t.wrist = -0.9; t.head = -0.25; break
      case 'stagger':
        t.sh = 1; t.lean = -0.5 + Math.sin(time * 9) * 0.05; t.sw = -2.5; t.reach = 1; t.wrist = 0.5; t.ou = 2.7; t.of = 3.0; t.crouch = 5; t.head = -0.4
        Object.assign(t, { l1: 0.5, b1: 0.6, l2: -0.35, b2: 0.3 })
        break
    }
    return t
  }

  function generalPose(e, anim, time) {
    const t = basePose()
    const speed = Math.abs(e.vx)
    t.flutter = clamp(speed / 260, 0, 1)
    anim.snap = false
    const progress = e.teleTotal ? 1 - Math.max(0, e.t) / e.teleTotal : 0
    const breathe = Math.sin(time * 1.4)
    t.lean = 0.12; t.sw = -0.3; t.reach = 0.95; t.ou = -0.3; t.of = -0.6; t.crouch = 3; t.wrist = 0
    switch (e.state) {
      case 'spawn':
      case 'intro':
        t.one = 1; t.sw = 0.9; t.reach = 0.55; t.wrist = -2.3; t.lean = 0.02 + breathe * 0.02; t.ou = 0.15; t.of = -0.2
        break
      case 'idle':
        t.crouch = 4 + breathe; t.sw = -0.3 + breathe * 0.04; t.lean = 0.16
        Object.assign(t, { l1: 0.45, b1: 0.4, l2: -0.4, b2: 0.15 })
        break
      case 'walk':
        runLegs(t, anim.phase, 0.65); t.lean = 0.26; t.sw = -0.3
        break
      case 'combo': {
        const step = e.combo ? e.combo.i : 0
        const raised = [-2.5, 2.9, -1.95][step]
        const swing = [1.0, -0.25, 1.3][step]
        if (e.combo?.phase === 'tele') { t.sw = lerp(-0.3, raised, ease(progress * 1.5)); t.reach = 1; t.lean = -0.1; t.crouch = 5 }
        else { t.sw = swing; t.reach = 1.05; t.lean = 0.55; t.crouch = 7; anim.snap = true }
        Object.assign(t, { l1: 0.7, b1: 0.45, l2: -0.7, b2: 0.12 })
        break
      }
      case 'flagTell':
        t.one = 1; t.flag = ease(progress * 2.2); t.sw = 0.8; t.reach = 0.8; t.wrist = -2.2; t.lean = -0.1; t.crouch = 3
        t.ou = 2.9; t.of = 3.0
        break
      case 'rushTell':
        t.crouch = 7 + progress * 7; t.lean = 0.55; t.sw = 2.7; t.wrist = 0.3; t.reach = 1
        Object.assign(t, { l1: 0.9, b1: 1.4, l2: -0.3, b2: 1.1 })
        break
      case 'rush':
        t.lean = 0.95; t.crouch = 8; t.sw = 0.05; t.reach = 1.4; t.flutter = 1; anim.snap = true
        Object.assign(t, { l1: 1.2, b1: 0.5, l2: -1.1, b2: 0.15 })
        break
      case 'backstep':
        t.air = 1; t.lean = -0.3; t.sw = -1.2; t.reach = 1
        break
      case 'roar':
        t.one = 1; t.lean = -0.4; t.sw = -2.0; t.wrist = 0; t.reach = 0.9; t.ou = 2.8; t.of = 3.1; t.head = -0.4; t.crouch = 3
        break
      case 'broken':
        t.one = 1; t.lean = 0.85; t.crouch = 16; t.sw = 0.6; t.wrist = -0.4; t.reach = 1; t.ou = 0.6; t.of = 0.4; t.head = 0.3
        Object.assign(t, { l1: 1.2, b1: 2.3, l2: -0.2, b2: 1.6 })
        break
      case 'recover':
        t.lean = 0.4; t.sw = 0.7; t.crouch = 6
        break
      case 'dying':
        t.one = 1; t.lean = 0.9; t.crouch = 18; t.sw = 0.5; t.wrist = -0.4; t.ou = 0.5; t.of = 0.2; t.head = 0.5
        Object.assign(t, { l1: 1.3, b1: 2.4, l2: -0.15, b2: 1.7 })
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
    anim.phase += speed * dt * (kind === 'boss' || kind === 'angler' || kind === 'general' ? 0.06 : 0.085)
    anim.flip = Math.max(0, anim.flip - dt / 0.42)
    anim.parry = Math.max(0, anim.parry - dt)
    const target = kind === 'player' ? playerPose(entity, anim, time, rules) : kind === 'general' ? generalPose(entity, anim, time) : (kind === 'boss' || kind === 'angler') ? bossPose(entity, anim, time) : enemyPose(entity, anim, time)
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
    const offShoulder = [shoulder[0] - fwd[0] * 1.5, shoulder[1]]
    let offElbow = [shoulder[0] - fwd[0] * 1.5 + ox1 * spec.upper, shoulder[1] + oy1 * spec.upper]
    const [ox2, oy2] = dir(pose.of)
    let offHand = [offElbow[0] + ox2 * spec.fore, offElbow[1] + oy2 * spec.fore]
    // One-handed blades hang from a bent wrist; spears, hammers and rods run through both fists.
    const angle = pose.sw + (pose.wrist || 0)
    const axis = [Math.cos(angle), Math.sin(angle)]
    let rear = null
    if (spec.grip && !(pose.one > 0.5)) {
      rear = [hx - axis[0] * spec.grip.span, hy - axis[1] * spec.grip.span]
      const [bx, by, rx, ry] = ik(offShoulder[0], offShoulder[1], rear[0], rear[1], spec.upper, spec.fore)
      offElbow = [bx, by]; offHand = [rx, ry]
    }
    return {
      hip, neck, shoulder, head, headTilt, up, fwd, front: leg(pose.l1, pose.b1), back: leg(pose.l2, pose.b2),
      elbow: [ex, ey], hand: [hx, hy], offElbow, offHand, offShoulder, swordDir, axis,
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
    const hemDepth = long ? 30 : spec.robe === 'lamellar' ? 21 : spec.robe === 'hood' ? 13 : 14
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
    } else if (spec.robe === 'armor' || spec.robe === 'lamellar') {
      ctx.quadraticCurveTo((hf[0] + hb[0]) / 2, hip[1] + hemDepth, hb[0], hb[1])
      ctx.quadraticCurveTo(wb[0] - 3, (wb[1] + hb[1]) / 2, wb[0], wb[1])
      ctx.quadraticCurveTo(sb[0] - 1.5, (sb[1] + wb[1]) / 2, sb[0], sb[1])
      ctx.closePath(); ctx.fill()
      ctx.strokeStyle = 'rgba(235,225,210,0.32)'; ctx.lineWidth = 0.8
      for (let y = sf[1] + 6; y < hf[1]; y += 5) {
        ctx.beginPath(); ctx.moveTo(sb[0] + 2, y); ctx.lineTo(sf[0] - 2, y); ctx.stroke()
      }
    } else if (spec.robe === 'vest' || spec.robe === 'padded') {
      ctx.quadraticCurveTo((hf[0] + hb[0]) / 2, hip[1] + hemDepth - 4, hb[0], hb[1] - 4)
      ctx.quadraticCurveTo(wb[0] - 2, (wb[1] + hb[1]) / 2, wb[0], wb[1])
      ctx.quadraticCurveTo(sb[0] - 1, (sb[1] + wb[1]) / 2, sb[0], sb[1])
      ctx.closePath(); ctx.fill()
      if (spec.robe === 'padded') {
        ctx.strokeStyle = 'rgba(235,225,210,0.26)'; ctx.lineWidth = 0.7
        for (let k = 0; k < 4; k++) { const y = sf[1] + 4 + k * 5; ctx.beginPath(); ctx.moveTo(sb[0] + 2, y); ctx.lineTo(sf[0] - 1, y + 0.6); ctx.stroke() }
        ctx.beginPath(); ctx.moveTo(lerp(sb[0], sf[0], 0.5), sf[1] + 3); ctx.lineTo(lerp(hb[0], hf[0], 0.5), hf[1] - 2); ctx.stroke()
      }
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
    if (spec.headStyle && !look.flash && !LATE_HATS[spec.hat]) eyes(ctx, hx + r * 0.42, hy - r * 0.1, look.eyeGlow || 0, spec.head)
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

  const LATE_HATS = { kettle: 1, cavhelm: 1, wrap: 1, warhelm: 1 }

  // Soldier helmets, the rider's dented helm, the bomber's headwrap and the general's war helm. Eyes are drawn afterwards, on top.
  function drawHat2(ctx, spec, s, pose, time, look, seed) {
    const [hx, hy] = s.head
    const r = spec.head
    const fl = pose.flutter
    ctx.save(); ctx.translate(hx, hy); ctx.rotate(s.headTilt * 0.7)
    const flash = look.flash
    const metal = flash ? look.color : spec.hat === 'warhelm' ? '#17181b' : '#2a2c31'
    ctx.fillStyle = metal; ctx.strokeStyle = metal
    const pale = 'rgba(239,231,212,0.34)'
    if (spec.hat === 'wrap') {
      ctx.fillStyle = flash ? look.color : '#33343a'
      ctx.beginPath(); ctx.ellipse(0.5, -r * 0.95, r * 1.28, r * 0.78, -0.08, 0, TAU); ctx.fill()
      ctx.beginPath(); ctx.ellipse(1.5, -r * 1.5, r * 0.8, r * 0.55, 0.1, 0, TAU); ctx.fill()
      const wave = Math.sin(time * 8 + seed) * (1.5 + fl * 2)
      ctx.beginPath(); ctx.moveTo(-r * 0.9, -r * 1.1)
      ctx.quadraticCurveTo(-r * 2 - fl * 5, -r * 0.6 + wave, -r * 2.8 - fl * 9, r * 0.9 + wave * 1.4)
      ctx.quadraticCurveTo(-r * 1.8 - fl * 4, -r * 0.1, -r * 0.7, -r * 0.3); ctx.closePath(); ctx.fill()
      if (!flash) {
        ctx.strokeStyle = 'rgba(235,225,210,0.34)'; ctx.lineWidth = 0.7
        for (let i = 0; i < 3; i++) { ctx.beginPath(); ctx.moveTo(-r * 1.1 + i * 3.2, -r * 1.5 + i * 0.8); ctx.quadraticCurveTo(0, -r * 1.2 + i * 1.4, r * 1.2, -r * 0.8 + i * 1.6); ctx.stroke() }
      }
    } else {
      const big = spec.hat === 'warhelm' ? 1.18 : 1
      const dome = r * 1.14 * big, cy = -r * 0.6
      ctx.beginPath(); ctx.arc(0, cy, dome, Math.PI, 0); ctx.fill()
      ctx.beginPath(); ctx.roundRect(-dome - 0.6, cy - 0.4, dome * 2 + 1.2, 2.3, 1); ctx.fill()
      // neck guard
      const len = spec.hat === 'warhelm' ? 13 : 7
      ctx.beginPath(); ctx.moveTo(-dome * 0.95, cy + 1); ctx.lineTo(-dome - 2.5 - fl * 2, cy + len * 0.6); ctx.lineTo(-dome * 0.4, cy + len); ctx.lineTo(-dome * 0.35, cy + 2); ctx.closePath(); ctx.fill()
      if (spec.hat === 'warhelm') {
        ctx.beginPath(); ctx.moveTo(dome * 0.8, cy + 1); ctx.lineTo(dome * 0.35, cy + 8); ctx.lineTo(dome * 0.1, cy + 2); ctx.closePath(); ctx.fill()
      }
      ctx.beginPath(); ctx.arc(0, cy - dome - 0.8, 1.6, 0, TAU); ctx.fill()
      if (!flash) {
        ctx.strokeStyle = pale; ctx.lineWidth = 0.6
        ctx.beginPath(); ctx.arc(0, cy, dome - 1.4, Math.PI * 1.1, Math.PI * 1.85); ctx.stroke()
        if (spec.hat === 'cavhelm') { ctx.beginPath(); ctx.moveTo(-1.5, cy - dome + 0.6); ctx.lineTo(0.8, cy - dome + 3.4); ctx.lineTo(-0.4, cy - dome + 5.6); ctx.stroke() }
      }
      const top = cy - dome - 1
      if (spec.hat === 'cavhelm') {
        ctx.strokeStyle = flash ? look.color : '#1a1b1e'; ctx.lineWidth = 1.6
        for (let i = 0; i < 4; i++) {
          const w = Math.sin(time * 9 + i + seed) * (1 + fl * 2)
          ctx.beginPath(); ctx.moveTo(0, top); ctx.quadraticCurveTo(-3 - i, top - 4 - fl, -6 - i * 1.8 - fl * 5, top + 1 + i * 1.6 + w); ctx.stroke()
        }
      } else if (spec.hat === 'warhelm') {
        ctx.fillStyle = flash ? look.color : '#2a2c31'
        ctx.beginPath(); ctx.roundRect(-2, top - 4, 4, 5, 1); ctx.fill()
        if (!flash) {
          ctx.fillStyle = CINNABAR; ctx.strokeStyle = CINNABAR
          const sway = Math.sin(time * 5 + seed) * 1.5, back = 7 + fl * 10
          ctx.beginPath(); ctx.moveTo(-2, top - 3)
          ctx.quadraticCurveTo(-3, top - 22, 4 - back * 0.3, top - 30 + sway)
          ctx.quadraticCurveTo(-back * 0.5, top - 20, -back, top - 16 + sway * 1.5)
          ctx.quadraticCurveTo(-back * 0.35, top - 12, 2.5, top - 3); ctx.closePath(); ctx.fill()
          ctx.lineWidth = 1
          for (let i = 0; i < 4; i++) { ctx.beginPath(); ctx.moveTo(0, top - 5); ctx.quadraticCurveTo(-6 - i * 2, top - 18 + i * 3, -back - 4 - i * 2 - fl * 4, top - 12 + i * 4 + sway); ctx.stroke() }
        }
      }
    }
    ctx.restore()
  }

  function drawHat(ctx, spec, s, pose, time, look, seed) {
    if (LATE_HATS[spec.hat]) return drawHat2(ctx, spec, s, pose, time, look, seed)
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

  const WRAP = 'rgba(232,224,204,0.82)'

  function fist(ctx, p, r, look, color) {
    ctx.save()
    ctx.fillStyle = color
    ctx.beginPath(); ctx.arc(p[0], p[1], r, 0, TAU); ctx.fill()
    ctx.restore()
  }

  // Pale cloth binding on a haft, just ahead of a fist.
  function binding(ctx, p, axis, look, width, at = 3.2, len = 4.2) {
    if (look.flash) return
    ctx.save()
    ctx.strokeStyle = WRAP; ctx.lineWidth = width
    ctx.lineCap = 'butt'
    ctx.beginPath(); ctx.moveTo(p[0] + axis[0] * at, p[1] + axis[1] * at); ctx.lineTo(p[0] + axis[0] * (at + len), p[1] + axis[1] * (at + len)); ctx.stroke()
    ctx.restore()
  }

  function drawBlade(ctx, spec, s, look, out) {
    if (!spec.blade) return
    const [hx, hy] = s.hand
    const [dx, dy] = s.axis
    const len = spec.blade, w = spec.bladeW
    const nx = -dy, ny = dx
    let tip = [hx + dx * len, hy + dy * len]
    out.tip = tip
    ctx.save()
    // Hilt: a hand-and-a-half grip with its pommel behind the fist and a wrapped middle.
    ctx.strokeStyle = look.flash ? look.color : '#46392d'; ctx.lineWidth = 2.5
    ctx.beginPath(); ctx.moveTo(hx - dx * 5, hy - dy * 5); ctx.lineTo(hx + dx * 2.6, hy + dy * 2.6); ctx.stroke()
    if (!look.flash) {
      ctx.strokeStyle = WRAP; ctx.lineWidth = 0.8
      for (let k = -4.6; k <= -2; k += 1.5) { ctx.beginPath(); ctx.moveTo(hx + dx * k + nx * 1.3, hy + dy * k + ny * 1.3); ctx.lineTo(hx + dx * (k + 1.1) - nx * 1.3, hy + dy * (k + 1.1) - ny * 1.3); ctx.stroke() }
    }
    ctx.strokeStyle = look.color || spec.color
    ctx.lineWidth = 1.5
    ctx.beginPath(); ctx.moveTo(hx + dx * 2.8 + nx * 3.6, hy + dy * 2.8 + ny * 3.6); ctx.lineTo(hx + dx * 2.8 - nx * 3.6, hy + dy * 2.8 - ny * 3.6); ctx.stroke()
    ctx.fillStyle = look.color || spec.color
    ctx.beginPath()
    if (spec.sabre) {
      // curved sabre: spine bows up, edge bellies, point swings forward
      const q = (u, v) => [hx + dx * u + nx * v, hy + dy * u + ny * v]
      out.tip = tip = q(len + 1, 3.6)
      ctx.moveTo(...q(3.4, -w / 2))
      ctx.quadraticCurveTo(...q(len * 0.55, -w / 2 - 3.2), ...tip)
      ctx.quadraticCurveTo(...q(len * 0.6, w * 0.7 + 1), ...q(3.4, w / 2))
    } else {
      ctx.moveTo(hx + dx * 3.4 + nx * w / 2, hy + dy * 3.4 + ny * w / 2)
      ctx.lineTo(hx + dx * len * 0.86 + nx * w * 0.45, hy + dy * len * 0.86 + ny * w * 0.45)
      ctx.lineTo(tip[0], tip[1])
      ctx.lineTo(hx + dx * 3.4 - nx * w / 2, hy + dy * 3.4 - ny * w / 2)
    }
    ctx.closePath(); ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = look.edge || 'rgba(239,231,212,0.75)'
      ctx.lineWidth = 0.55
      ctx.beginPath(); ctx.moveTo(hx + dx * 5 + nx * w * 0.18, hy + dy * 5 + ny * w * 0.18); ctx.lineTo(hx + dx * len * 0.9, hy + dy * len * 0.9); ctx.stroke()
    }
    if (spec.mask && !look.flash) {
      ctx.strokeStyle = CINNABAR; ctx.lineWidth = 1.2
      ctx.beginPath(); ctx.moveTo(hx - dx * 5, hy - dy * 5); ctx.quadraticCurveTo(hx - dx * 6, hy - dy * 6 + 5, hx - dx * 5 - 2, hy - dy * 5 + 10); ctx.stroke()
    }
    fist(ctx, [hx - dx * 0.5, hy - dy * 0.5], Math.max(2.1, spec.limbs[3] * 0.62), look, look.color || spec.color)
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

  // Two-handed weapons are drawn before the arms so both fists sit on top of the haft.
  function drawSpear(ctx, spec, s, look, out) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.axis
    const len = 76, shaftLen = 58, butt = spec.grip.span + 4
    const nx = -dy, ny = dx
    const tip = [hx + dx * len, hy + dy * len]
    out.tip = tip
    ctx.save()
    ctx.strokeStyle = look.flash ? look.color : '#3a2d22'; ctx.lineWidth = 2.5
    ctx.beginPath(); ctx.moveTo(hx - dx * butt, hy - dy * butt); ctx.lineTo(hx + dx * shaftLen, hy + dy * shaftLen); ctx.stroke()
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
    const [dx, dy] = s.axis
    const len = 38, butt = spec.grip.span + 4
    const headX = hx + dx * len, headY = hy + dy * len
    out.tip = [headX + dx * 10, headY + dy * 10]
    ctx.save()
    ctx.strokeStyle = look.flash ? look.color : '#3b3128'; ctx.lineWidth = 3.4
    ctx.beginPath(); ctx.moveTo(hx - dx * butt, hy - dy * butt); ctx.lineTo(headX, headY); ctx.stroke()
    ctx.translate(headX, headY)
    ctx.rotate(Math.atan2(dy, dx))
    ctx.fillStyle = look.flash ? look.color : '#43464d'
    ctx.beginPath(); ctx.roundRect(-5, -14, 16, 28, 2.5); ctx.fill()
    if (!look.flash) {
      ctx.fillStyle = 'rgba(30,31,36,0.55)'
      ctx.fillRect(-5, -14, 3, 28)
      ctx.strokeStyle = 'rgba(235,228,215,0.32)'; ctx.lineWidth = 0.8
      ctx.beginPath(); ctx.moveTo(1, -12.5); ctx.lineTo(9.5, -12.5); ctx.moveTo(1, 12.5); ctx.lineTo(9.5, 12.5); ctx.stroke()
    }
    ctx.restore()
  }

  function drawRod(ctx, spec, s, look, out) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.axis
    const rodLen = 62, butt = spec.grip.span + 3
    const tipX = hx + dx * rodLen - dy * 7, tipY = Math.min(1, hy + dy * rodLen + dx * 7)
    out.tip = [tipX + dx * 40, Math.min(2, tipY + dy * 20)]
    ctx.save()
    ctx.strokeStyle = look.flash ? look.color : '#3a4630'; ctx.lineWidth = 2
    ctx.beginPath()
    ctx.moveTo(hx - dx * butt, hy - dy * butt)
    ctx.quadraticCurveTo(hx + dx * rodLen * 0.6, hy + dy * rodLen * 0.6, tipX, tipY)
    ctx.stroke()
    ctx.strokeStyle = look.flash ? look.color : 'rgba(235,230,220,0.7)'; ctx.lineWidth = 0.8
    const hookX = out.tip[0], hookY = out.tip[1]
    ctx.beginPath(); ctx.moveTo(tipX, tipY); ctx.quadraticCurveTo((tipX + hookX) / 2 + 5, (tipY + hookY) / 2 + 8, hookX, hookY); ctx.stroke()
    ctx.fillStyle = CINNABAR
    ctx.beginPath(); ctx.arc(hookX, hookY, 2.2, 0, TAU); ctx.fill()
    ctx.restore()
  }

  // Two-handed (or one-handed when pose.one) heavy 大刀 with a wide curved blade and a cinnabar collar tassel.
  function drawBroadsword(ctx, spec, s, look, out, time) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.axis
    const nx = -dy, ny = dx
    const P = (a, b) => [hx + dx * a + nx * b, hy + dy * a + ny * b]
    const butt = spec.grip.span + 5
    out.tip = P(90, -3)
    ctx.save()
    const ink = look.color || spec.color
    ctx.strokeStyle = look.flash ? look.color : '#2d2219'; ctx.lineWidth = 3
    ctx.beginPath(); ctx.moveTo(...P(-butt, 0)); ctx.lineTo(...P(40, 0)); ctx.stroke()
    if (!look.flash) {
      ctx.strokeStyle = 'rgba(235,225,210,0.45)'; ctx.lineWidth = 3.2; ctx.lineCap = 'butt'
      ctx.beginPath(); ctx.moveTo(...P(-butt + 1, 0)); ctx.lineTo(...P(-butt + 4, 0)); ctx.stroke()
      ctx.lineCap = 'round'
    }
    ctx.fillStyle = ink
    ctx.beginPath(); ctx.arc(...P(-butt, 0), 2.4, 0, TAU); ctx.fill()
    ctx.strokeStyle = ink; ctx.lineWidth = 2.2
    ctx.beginPath(); ctx.moveTo(...P(40, -5)); ctx.lineTo(...P(40, 5)); ctx.stroke()
    ctx.fillStyle = look.flash ? look.color : '#34373d'
    ctx.beginPath()
    ctx.moveTo(...P(40, -2.4))
    ctx.lineTo(...P(84, -3.2))
    ctx.lineTo(...P(91, -3))
    ctx.quadraticCurveTo(...P(80, 15), ...P(60, 12.5))
    ctx.quadraticCurveTo(...P(48, 10), ...P(41, 3))
    ctx.closePath(); ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = 'rgba(239,231,212,0.7)'; ctx.lineWidth = 0.6
      ctx.beginPath(); ctx.moveTo(...P(88, -1)); ctx.quadraticCurveTo(...P(78, 12.4), ...P(60, 11)); ctx.stroke()
      ctx.strokeStyle = 'rgba(239,231,212,0.3)'; ctx.lineWidth = 0.7
      ctx.beginPath(); ctx.moveTo(...P(45, 0)); ctx.lineTo(...P(80, -0.5)); ctx.stroke()
      ctx.fillStyle = CINNABAR; ctx.strokeStyle = CINNABAR; ctx.lineWidth = 1.1
      const sway = Math.sin(time * 6) * 2 - 2
      ctx.beginPath(); ctx.arc(...P(39, 0), 1.7, 0, TAU); ctx.fill()
      for (let i = 0; i < 3; i++) { ctx.beginPath(); ctx.moveTo(...P(39, 0)); ctx.quadraticCurveTo(...P(35 - i * 2, 4 + i * 2), hx + dx * (34 - i * 2) + nx * (8 + i * 2) + sway, hy + dy * (34 - i * 2) + ny * (8 + i * 2) + 5 + i * 2); ctx.stroke() }
    }
    ctx.restore()
  }

  // Short spear held one-handed beside the pavise.
  function drawShortSpear(ctx, spec, s, look, out) {
    const [hx, hy] = s.hand
    const [dx, dy] = s.axis
    const nx = -dy, ny = dx
    const shaft = 46, tip = [hx + dx * (shaft + 13), hy + dy * (shaft + 13)]
    out.tip = tip
    ctx.save()
    ctx.strokeStyle = look.flash ? look.color : '#3a2d22'; ctx.lineWidth = 2.4
    ctx.beginPath(); ctx.moveTo(hx - dx * 16, hy - dy * 16); ctx.lineTo(hx + dx * shaft, hy + dy * shaft); ctx.stroke()
    if (!look.flash) {
      ctx.fillStyle = CINNABAR
      ctx.beginPath()
      ctx.moveTo(hx + dx * (shaft - 2), hy + dy * (shaft - 2))
      ctx.quadraticCurveTo(hx + dx * (shaft + 4) + nx * 4, hy + dy * (shaft + 4) + ny * 4 + 1, hx + dx * (shaft + 7), hy + dy * (shaft + 7) + 3)
      ctx.quadraticCurveTo(hx + dx * (shaft + 3) - nx * 3, hy + dy * (shaft + 3) - ny * 3, hx + dx * (shaft - 2), hy + dy * (shaft - 2)); ctx.fill()
    }
    ctx.fillStyle = look.flash ? look.color : '#e5dfd2'
    ctx.beginPath()
    ctx.moveTo(hx + dx * shaft + nx * 3.2, hy + dy * shaft + ny * 3.2); ctx.lineTo(tip[0], tip[1]); ctx.lineTo(hx + dx * shaft - nx * 3.2, hy + dy * shaft - ny * 3.2)
    ctx.closePath(); ctx.fill()
    ctx.restore()
  }

  function drawHook(ctx, s, look, out, pose, time) {
    const [hx, hy] = s.hand, [dx, dy] = s.axis
    const nx = -dy, ny = dx
    const cast=pose.cast||0, shaft=40+cast*105
    const tx=hx+dx*shaft, ty=hy+dy*shaft
    out.tip=[tx+8,ty-5]
    ctx.save();ctx.strokeStyle=look.flash?look.color:'#1d3036';ctx.lineWidth=2.3
    ctx.beginPath();ctx.moveTo(hx-dx*18,hy-dy*18);ctx.lineTo(hx+dx*27,hy+dy*27);ctx.stroke()
    // Articulated chain, deliberately slack during preparation and taut on contact.
    ctx.lineWidth=1.3
    for(let i=0;i<15;i++) {
      const q=i/14, sag=Math.sin(q*Math.PI)*(1-cast)*17
      const px=hx+dx*(25+q*(shaft-25)), py=hy+dy*(25+q*(shaft-25))+sag
      ctx.beginPath();ctx.ellipse(px,py,3,1.6,Math.atan2(dy,dx)+(i%2)*0.55,0,TAU);ctx.stroke()
    }
    ctx.fillStyle=look.flash?look.color:'#1c2b30';ctx.beginPath();ctx.moveTo(tx-5,ty-3)
    ctx.lineTo(tx+12,ty-12);ctx.lineTo(tx+9,ty+2);ctx.lineTo(tx+2,ty+13);ctx.lineTo(tx-11,ty+15);ctx.lineTo(tx-6,ty+6);ctx.lineTo(tx+1,ty+5);ctx.closePath();ctx.fill()
    ctx.strokeStyle=look.flash?look.color:'#c4d3cf';ctx.lineWidth=1.2;ctx.beginPath();ctx.moveTo(tx+12,ty-12);ctx.lineTo(tx+9,ty+2);ctx.lineTo(tx+2,ty+13);ctx.lineTo(tx-11,ty+15);ctx.stroke()
    ctx.restore()
  }

  // Clay fire-pot ~7 radius with a short fuse; returns the fuse tip (local).
  function drawPot(ctx, cx, cy, look, time) {
    ctx.save()
    const flash = look.flash
    ctx.fillStyle = flash ? look.color : '#8d6a3b'
    ctx.beginPath(); ctx.arc(cx, cy, 7, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.roundRect(cx - 3, cy - 9, 6, 4, 1.2); ctx.fill()
    if (!flash) {
      ctx.fillStyle = 'rgba(40,28,18,0.55)'
      ctx.beginPath(); ctx.arc(cx + 2, cy + 2.4, 6, -0.3, Math.PI * 0.85); ctx.lineTo(cx, cy + 1); ctx.fill()
      ctx.strokeStyle = '#1b1a1a'; ctx.lineWidth = 0.9
      ctx.beginPath(); ctx.arc(cx, cy, 7, 0, TAU); ctx.stroke()
      ctx.beginPath(); ctx.moveTo(cx - 6.4, cy - 1.5); ctx.quadraticCurveTo(cx, cy + 1.5, cx + 6.4, cy - 1.5); ctx.stroke()
      ctx.fillStyle = '#1b1a1a'; ctx.beginPath(); ctx.ellipse(cx, cy - 8.6, 2.4, 1, 0, 0, TAU); ctx.fill()
    }
    const fx = cx + 3.2, fy = cy - 14
    ctx.strokeStyle = look.flash ? look.color : '#cdbf9a'; ctx.lineWidth = 1.1
    ctx.beginPath(); ctx.moveTo(cx, cy - 8.6); ctx.quadraticCurveTo(cx + 0.5, cy - 12, fx, fy); ctx.stroke()
    ctx.restore()
    return [fx, fy]
  }

  // Standing pavise: 15 x 62, planks, pale lacquer panel with an ink beast-face, iron studs. Origin = bottom centre.
  function drawPavise(ctx, pose, look) {
    const sh = pose.sh
    const bx = 15 + sh * 9, tilt = sh * 0.85
    ctx.save()
    ctx.translate(bx, -1); ctx.rotate(tilt)
    const flash = look.flash
    ctx.fillStyle = flash ? look.color : '#3b3027'
    ctx.beginPath(); ctx.roundRect(-7.5, -62, 15, 62, 2.2); ctx.fill()
    if (!flash) {
      ctx.strokeStyle = 'rgba(20,18,16,0.7)'; ctx.lineWidth = 0.7
      for (const x of [-2.6, 2.6]) { ctx.beginPath(); ctx.moveTo(x, -61); ctx.lineTo(x, -1); ctx.stroke() }
      ctx.fillStyle = '#d8cdb0'
      ctx.beginPath(); ctx.roundRect(-5.2, -53, 10.4, 31, 2); ctx.fill()
      ctx.strokeStyle = '#17171a'; ctx.fillStyle = '#17171a'; ctx.lineWidth = 1
      // beast face: brow, slit eyes, snout and fangs
      ctx.beginPath(); ctx.moveTo(-4.4, -46.5); ctx.lineTo(0, -44.6); ctx.lineTo(4.4, -46.5); ctx.stroke()
      ctx.beginPath(); ctx.ellipse(-2.3, -43.6, 1.6, 0.8, 0.35, 0, TAU); ctx.ellipse(2.3, -43.6, 1.6, 0.8, -0.35, 0, TAU); ctx.fill()
      ctx.beginPath(); ctx.moveTo(-1.4, -40.2); ctx.lineTo(0, -38.4); ctx.lineTo(1.4, -40.2); ctx.stroke()
      ctx.lineWidth = 1.3; ctx.beginPath(); ctx.moveTo(-3.6, -35.5); ctx.quadraticCurveTo(0, -33.4, 3.6, -35.5); ctx.stroke()
      ctx.beginPath(); ctx.moveTo(-2.4, -35); ctx.lineTo(-1.8, -31.2); ctx.lineTo(-1.1, -34.6); ctx.moveTo(2.4, -35); ctx.lineTo(1.8, -31.2); ctx.lineTo(1.1, -34.6); ctx.fill()
      ctx.lineWidth = 0.8
      for (const [a, b] of [[-4.5, -50], [4.5, -50], [-4.5, -25], [4.5, -25]]) { ctx.beginPath(); ctx.moveTo(a, b); ctx.lineTo(a * 0.6, b + 2.6); ctx.stroke() }
      ctx.fillStyle = '#9a9b9a'
      for (const y of [-58.5, -15, -5.5]) for (const x of [-4.8, 4.8]) { ctx.beginPath(); ctx.arc(x, y, 0.95, 0, TAU); ctx.fill() }
      ctx.strokeStyle = 'rgba(190,190,185,0.3)'; ctx.lineWidth = 1
      ctx.beginPath(); ctx.roundRect(-7, -61.5, 14, 61, 2); ctx.stroke()
      ctx.fillStyle = '#242220'; ctx.fillRect(-7.5, -3, 15, 3)
    }
    ctx.restore()
  }

  function drawBanner(ctx, hand, facing, look, time, seed) {
    const [hx, hy] = hand
    const top = [hx + 1, hy - 44]
    ctx.save()
    ctx.strokeStyle = look.flash ? look.color : '#2d2219'; ctx.lineWidth = 2
    ctx.beginPath(); ctx.moveTo(hx - 1, hy + 9); ctx.lineTo(top[0], top[1] - 2); ctx.stroke()
    const w = 24, h = 17
    const wave = (k) => Math.sin(time * 7 + k * 1.3 + seed) * (1 + k * 1.5)
    ctx.fillStyle = look.flash ? look.color : '#e8dfc6'
    ctx.beginPath(); ctx.moveTo(top[0], top[1])
    for (let i = 1; i <= 4; i++) ctx.lineTo(top[0] - (w * i) / 4, top[1] + wave(i) * 0.6 - 1)
    for (let i = 4; i >= 0; i--) ctx.lineTo(top[0] - (w * i) / 4 + (i === 0 ? 0 : 0), top[1] + h + wave(i) * 0.6 - (i === 4 ? 3 : 0))
    ctx.closePath(); ctx.fill()
    if (!look.flash) {
      ctx.strokeStyle = '#17171a'; ctx.lineWidth = 0.9; ctx.stroke()
      ctx.save()
      ctx.translate(top[0] - w * 0.5, top[1] + h * 0.5 + wave(2) * 0.3); ctx.scale(facing, 1)
      ctx.fillStyle = '#17171a'; ctx.font = '700 12px "Kaiti SC","STKaiti","KaiTi","Songti SC",serif'; ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
      ctx.fillText('令', 0, 0.5)
      ctx.restore()
      ctx.strokeStyle = CINNABAR; ctx.lineWidth = 1
      for (let i = 0; i < 3; i++) { ctx.beginPath(); ctx.moveTo(top[0], top[1]); ctx.quadraticCurveTo(top[0] + 2 + i, top[1] - 3, top[0] + 2 + i * 1.5, top[1] - 7 - i); ctx.stroke() }
    }
    ctx.restore()
  }

  // 靠旗: four small pennants fanned out behind the shoulders.
  function drawBackFlags(ctx, s, pose, look, time, seed) {
    const ax = s.neck[0] - s.fwd[0] * 4 - s.up[0] * 4, ay = s.neck[1] - s.fwd[1] * 4 - s.up[1] * 4
    ctx.save()
    ctx.fillStyle = look.color; ctx.strokeStyle = look.color
    const fl = pose.flutter
    for (let i = 0; i < 4; i++) {
      const a = -1.75 - i * 0.28, L = 27 - i * 1.5
      const dx = Math.cos(a), dy = Math.sin(a), nx = -dy, ny = dx
      const tx = ax + dx * L, ty = ay + dy * L
      ctx.lineWidth = 1.5
      ctx.beginPath(); ctx.moveTo(ax, ay); ctx.lineTo(tx, ty); ctx.stroke()
      const sway = Math.sin(time * 6 + i * 1.1 + seed) * (1.2 + fl * 3) - fl * 5
      ctx.beginPath()
      ctx.moveTo(ax + dx * L * 0.3, ay + dy * L * 0.3)
      ctx.lineTo(tx, ty)
      ctx.quadraticCurveTo(tx - dx * 5 - nx * 4 + sway * 0.4, ty - dy * 5 - ny * 4, ax + dx * L * 0.3 - nx * 7 + sway, ay + dy * L * 0.3 - ny * 7 + 2)
      ctx.closePath(); ctx.fill()
      if (!look.flash) {
        ctx.strokeStyle = 'rgba(235,225,210,0.34)'; ctx.lineWidth = 0.6
        ctx.beginPath(); ctx.moveTo(ax + dx * L * 0.35 - nx * 2, ay + dy * L * 0.35 - ny * 2); ctx.lineTo(tx - dx * 4 - nx * 2.5, ty - dy * 4 - ny * 2.5); ctx.stroke()
        ctx.strokeStyle = look.color
      }
    }
    ctx.restore()
  }

  function drawCape(ctx, spec, s, pose, time, look, seed) {
    const len = spec.cape, fl = pose.flutter
    const wave = Math.sin(time * 8 + seed) * (1.2 + fl * 2.5)
    const top = [s.neck[0] - s.fwd[0] * 3, s.neck[1] - s.fwd[1] * 3]
    const mid = [s.hip[0] - 7 - fl * len * 0.4, s.neck[1] + (s.hip[1] - s.neck[1]) * 0.55]
    const end = [s.hip[0] - 5 - len * 0.45 - fl * len * 0.7, s.hip[1] + len * 0.55 + wave]
    const low = [s.hip[0] - 4, s.hip[1] + len * 0.5]
    ctx.save()
    if (!look.flash) ctx.fillStyle = look.color || spec.color
    ctx.beginPath(); ctx.moveTo(top[0], top[1])
    ctx.quadraticCurveTo(mid[0] - 3, mid[1], end[0], end[1])
    ctx.quadraticCurveTo((end[0] + low[0]) / 2, end[1] + 3 + wave * 0.5, low[0], low[1])
    ctx.quadraticCurveTo(s.hip[0] + 1, (low[1] + s.hip[1]) / 2, s.hip[0] + 2, s.hip[1])
    ctx.closePath(); ctx.fill()
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

  function drawSeaCostume(g,kind,s,pose,look,time) {
    const hook=kind==='hooker', [hx,hy]=s.head, [sx,sy]=s.shoulder
    g.save();g.fillStyle=look.color||'#17292e';g.strokeStyle=look.color||'#17292e'
    if(hook) {
      // Broad rope collar and a broken shell mask: a drowned salvage hunter.
      g.lineWidth=5;g.beginPath();g.arc(sx-2,sy+2,12,2.8,6.6);g.stroke()
      g.strokeStyle=look.flash?look.color:'rgba(209,188,144,0.72)';g.lineWidth=0.8
      for(let i=0;i<12;i++){const a=2.8+i*0.32;g.beginPath();g.moveTo(sx-2+Math.cos(a)*9,sy+2+Math.sin(a)*9);g.lineTo(sx-2+Math.cos(a)*14,sy+2+Math.sin(a)*14);g.stroke()}
      g.fillStyle=look.flash?look.color:'#c8c5a7';g.beginPath();g.moveTo(hx-3,hy-5);g.lineTo(hx+6,hy-7);g.lineTo(hx+9,hy+1);g.lineTo(hx+4,hy+7);g.lineTo(hx-1,hy+5);g.closePath();g.fill()
      g.strokeStyle='#203339';g.lineWidth=1;g.beginPath();g.moveTo(hx+3,hy-5);g.lineTo(hx+1,hy);g.lineTo(hx+5,hy+3);g.stroke()
      g.fillStyle=CINNABAR;g.fillRect(hx+4,hy-1,3,1.5)
      g.strokeStyle=look.flash?look.color:'#17292e';g.lineWidth=1.8
      for(let i=0;i<8;i++){const x=sx-8+i*2;g.beginPath();g.moveTo(x,sy+4);g.quadraticCurveTo(x-8,sy+20,x-12+Math.sin(time*5+i)*4,sy+34+i%3*4);g.stroke()}
    } else {
      // Top knot, split head wrap and crossed short blades distinguish the boarding duelist.
      g.beginPath();g.ellipse(hx-3,hy-8,4.5,4,-0.3,0,TAU);g.fill()
      g.strokeStyle=look.flash?look.color:'#a78a66';g.lineWidth=2.4;g.beginPath();g.moveTo(hx-6,hy-3);g.lineTo(hx+5,hy-3);g.stroke()
      g.strokeStyle=look.flash?look.color:'#20373c';g.lineWidth=2
      for(let i=0;i<2;i++){g.beginPath();g.moveTo(hx-6,hy-2);g.quadraticCurveTo(hx-16,hy+5,hx-23+Math.sin(time*7+i)*3,hy+i*6);g.stroke()}
      g.strokeStyle=look.flash?look.color:'rgba(207,218,206,0.55)';g.lineWidth=1.3;g.beginPath();g.moveTo(sx-6,sy+2);g.lineTo(s.hip[0]+4,s.hip[1]);g.stroke()
      const [x,y]=s.offHand, a=pose.of-1.7, dx=Math.cos(a),dy=Math.sin(a)
      g.fillStyle=look.flash?look.color:'#182b31';g.beginPath();g.moveTo(x,y);g.lineTo(x+dx*27-dy*4,y+dy*27+dx*4);g.quadraticCurveTo(x+dx*34-dy*9,y+dy*34+dx*9,x+dx*31,y+dy*31);g.lineTo(x+dy*2,y-dx*2);g.closePath();g.fill()
      g.strokeStyle=look.flash?look.color:'#b9cac4';g.lineWidth=1;g.beginPath();g.moveTo(x-dy*3,y+dx*3);g.lineTo(x+dx*27-dy*4,y+dy*27+dx*4);g.stroke()
    }
    g.restore()
  }

  // Draws a posed humanoid in world space. `look` = { color, alpha, flash, eyeGlow, halo }.
  function drawHumanoid(ctx, kind, pose, x, y, facing, look, time, seed = 0, extras = {}) {
    const spec = KINDS[kind]
    if ((spec.city||spec.summit) && globalThis.XiabanInkLanternFigures) return globalThis.XiabanInkLanternFigures.draw(ctx,kind,pose,x,y,facing,look,time,seed)
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
    const gripOn = !!spec.grip && !(pose.one > 0.5)
    if (spec.cape) drawCape(ctx, spec, s, pose, time, look, seed)
    if (spec.backFlags) drawBackFlags(ctx, s, pose, { ...look, color }, time, seed)
    let paviseHand = null
    if (spec.pavise) {
      const sh = pose.sh, bx = 15 + sh * 9, tilt = sh * 0.85
      const lx = -3, ly = -32
      const h = [bx + lx * Math.cos(tilt) - ly * Math.sin(tilt), -1 + lx * Math.sin(tilt) + ly * Math.cos(tilt)]
      const free = s.offHand
      const tgt = [lerp(h[0], free[0], clamp(sh * 1.4, 0, 1)), lerp(h[1], free[1], clamp(sh * 1.4, 0, 1))]
      const [ex, ey, hx2, hy2] = ik(s.offShoulder[0], s.offShoulder[1], tgt[0], tgt[1], spec.upper, spec.fore)
      paviseHand = [[ex, ey], [hx2, hy2]]
    }

    // Wet-ink halo keeps silhouettes legible over busy washes.
    if (look.halo) {
      ctx.save()
      ctx.globalAlpha *= look.halo
      ctx.fillStyle = 'rgba(239,231,212,1)'
      ctx.beginPath(); ctx.ellipse(s.hip[0], s.hip[1] - spec.torso * 0.35, spec.torso * 0.9, spec.torso * 1.35, pose.lean, 0, TAU); ctx.fill()
      ctx.restore()
      ctx.fillStyle = color
    }

    if (paviseHand) limb(ctx, [s.offShoulder, paviseHand[0], paviseHand[1]], au * 0.92)
    else if (!gripOn) limb(ctx, [s.offShoulder, s.offElbow, s.offHand], au * 0.92)
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
    if (spec.satchel) {
      ctx.save()
      ctx.fillStyle = look.flash ? look.color : '#1c1d20'
      const sx = s.hip[0] + s.fwd[0] * 3, sy = s.hip[1] + 4
      if (!look.flash) {
        ctx.strokeStyle = 'rgba(235,225,210,0.3)'; ctx.lineWidth = 1.2
        ctx.beginPath(); ctx.moveTo(s.neck[0] - s.fwd[0] * 3, s.neck[1] + 1); ctx.lineTo(sx + 1, sy - 2); ctx.stroke()
      }
      ctx.beginPath(); ctx.ellipse(sx, sy + 2, 7.5, 6.5, 0.2, 0, TAU); ctx.fill()
      if (!look.flash) {
        ctx.fillStyle = '#8d6a3b'
        for (const dx of [-3.4, 0.2, 3.6]) { ctx.beginPath(); ctx.arc(sx + dx, sy - 3.4 - (dx === 0.2 ? 1 : 0), 2.6, 0, TAU); ctx.fill() }
      }
      ctx.restore()
      ctx.fillStyle = color; ctx.strokeStyle = color
    }
    drawHead(ctx, spec, s, look, time, seed)
    if (spec.hat) drawHat(ctx, spec, s, pose, time, look, seed)
    if (spec.seaRaider) drawSeaCostume(ctx, kind, s, pose, look, time)
    if (spec.headStyle && !look.flash && LATE_HATS[spec.hat]) eyes(ctx, s.head[0] + spec.head * 0.42, s.head[1] - spec.head * 0.1, look.eyeGlow || 0, spec.head)
    if (spec.robe === 'lamellar') {
      ctx.fillStyle = look.flash ? look.color : '#191a1d'
      ctx.beginPath(); ctx.ellipse(s.shoulder[0] + 1, s.shoulder[1] + 2, 9, 6.5, -0.2, 0, TAU); ctx.fill()
      if (!look.flash) { ctx.strokeStyle = 'rgba(235,225,210,0.3)'; ctx.lineWidth = 0.7; ctx.beginPath(); ctx.arc(s.shoulder[0] + 1, s.shoulder[1] + 2, 5.5, Math.PI * 1.1, Math.PI * 1.9); ctx.stroke() }
      ctx.fillStyle = color; ctx.strokeStyle = color
    }
    if (spec.pavise) {
      ctx.save()
      const sh = pose.sh
      drawPavise(ctx, pose, look)
      ctx.restore()
      void sh
    }
    if (spec.shield) drawShield(ctx, s, look)
    if (spec.bow) {
      const pull = drawBow(ctx, s, pose, look, out)
      limb(ctx, [s.shoulder, [lerp(s.shoulder[0], pull[0], 0.5) - 2, lerp(s.shoulder[1], pull[1], 0.5) + 3], pull], af)
    } else if (gripOn) {
      if (spec.broadsword) drawBroadsword(ctx, spec, s, look, out, time)
      else if (spec.hook) drawHook(ctx, s, look, out, pose, time)
      else if (spec.spear) drawSpear(ctx, spec, s, look, out)
      else if (spec.hammer) drawHammer(ctx, spec, s, look, out)
      else drawRod(ctx, spec, s, look, out)
      ctx.fillStyle = color; ctx.strokeStyle = color
      if (spec.broadsword && pose.flag > 0.05) drawBanner(ctx, s.offHand, facing, look, time, seed)
      limb(ctx, [s.offShoulder, s.offElbow, s.offHand], au * 0.92)
      limb(ctx, [s.shoulder, s.elbow, s.hand], au)
      ctx.lineWidth = af
      ctx.beginPath(); ctx.moveTo(s.offElbow[0], s.offElbow[1]); ctx.lineTo(s.offHand[0], s.offHand[1]); ctx.stroke()
      ctx.beginPath(); ctx.moveTo(s.elbow[0], s.elbow[1]); ctx.lineTo(s.hand[0], s.hand[1]); ctx.stroke()
      if (!spec.broadsword) {
        binding(ctx, s.hand, s.axis, look, spec.hammer ? 3.8 : 2.8)
        binding(ctx, s.offHand, s.axis, look, spec.hammer ? 3.8 : 2.8)
      }
      const r = af * 0.62
      fist(ctx, s.offHand, r, look, color)
      fist(ctx, s.hand, r, look, color)
    } else {
      if (spec.broadsword) {
        if (pose.flag > 0.05) drawBanner(ctx, s.offHand, facing, look, time, seed)
        ctx.fillStyle = color; ctx.strokeStyle = color
        limb(ctx, [s.offShoulder, s.offElbow, s.offHand], au * 0.92)
        fist(ctx, s.offHand, af * 0.62, look, color)
        drawBroadsword(ctx, spec, s, look, out, time)
      } else if (spec.shortSpear) drawShortSpear(ctx, spec, s, look, out)
      ctx.fillStyle = color; ctx.strokeStyle = color
      limb(ctx, [s.shoulder, s.elbow, s.hand], au)
      if (spec.sleeves) drawSleeve(ctx, s.shoulder, s.elbow, pose, time, seed, false)
      ctx.lineWidth = af
      ctx.beginPath(); ctx.moveTo(s.elbow[0], s.elbow[1]); ctx.lineTo(s.hand[0], s.hand[1]); ctx.stroke()
      if (spec.broadsword || spec.shortSpear) fist(ctx, s.hand, af * 0.62, look, color)
      if (spec.pot && pose.pot > 0.5) {
        const c = [s.hand[0] + 1.5, s.hand[1] - 5.5]
        out.tip = drawPot(ctx, c[0], c[1], look, time)
        fist(ctx, s.hand, af * 0.7, look, color)
      }
      if (!spec.broadsword && !spec.shortSpear) drawBlade(ctx, spec, s, look, out)
    }
    if (spec.city && globalThis.XiabanInkLantern) {
      globalThis.XiabanInkLantern.costume(ctx,kind,s,pose,look,time,look.entity)
      if(kind==='judge')out.tip=[s.hand[0]+Math.cos(pose.sw)*(look.entity?.judgeStage===3?49:78),s.hand[1]+Math.sin(pose.sw)*(look.entity?.judgeStage===3?49:78)]
    }
    ctx.restore()
    const map = (p) => p && [x + p[0] * facing, y + p[1]]
    out.tip = map(out.tip)
    out.hand = map(s.hand)
    out.neck = map(s.neck)
    out.hip = map(s.hip)
    return out
  }

  // ---------------------------------------------------------------- horse & riders

  const HIP_F = [13, -31], HIP_H = [-15, -31], PIVOT = [-14, 0]
  const frac = (v) => v - Math.floor(v)

  function taper(ctx, a, b, w0, w1) {
    const dx = b[0] - a[0], dy = b[1] - a[1], l = Math.hypot(dx, dy) || 1
    const nx = -dy / l, ny = dx / l
    ctx.beginPath()
    ctx.moveTo(a[0] + nx * w0 / 2, a[1] + ny * w0 / 2); ctx.lineTo(b[0] + nx * w1 / 2, b[1] + ny * w1 / 2)
    ctx.lineTo(b[0] - nx * w1 / 2, b[1] - ny * w1 / 2); ctx.lineTo(a[0] - nx * w0 / 2, a[1] - ny * w0 / 2); ctx.closePath(); ctx.fill()
    ctx.beginPath(); ctx.arc(a[0], a[1], w0 / 2, 0, TAU); ctx.arc(b[0], b[1], w1 / 2, 0, TAU); ctx.fill()
  }

  function legIK(hip, foot, l1, l2, bendFwd) {
    let dx = foot[0] - hip[0], dy = foot[1] - hip[1]
    let d = Math.hypot(dx, dy) || 0.001
    const max = l1 + l2 - 0.05
    if (d > max) { dx *= max / d; dy *= max / d; d = max }
    const a = Math.acos(clamp((l1 * l1 + d * d - l2 * l2) / (2 * l1 * d), -1, 1))
    const base = Math.atan2(dy, dx)
    const ang = bendFwd ? base - a : base + a
    return [hip[0] + Math.cos(ang) * l1, hip[1] + Math.sin(ang) * l1]
  }

  const GAITS = {
    gallop: { duty: 0.4, off: { hf: 0, hn: 0.12, ff: 0.5, fn: 0.62 }, fwdF: 15, bckF: 9, fwdH: 11, bckH: 15, liftF: 15, liftH: 12 },
    trot: { duty: 0.5, off: { hf: 0, hn: 0.5, ff: 0.5, fn: 0 }, fwdF: 9, bckF: 7, fwdH: 7, bckH: 9, liftF: 9, liftH: 8 },
  }

  function gaitFoot(hip, front, key, st) {
    const g = GAITS[st.mode] || GAITS.trot
    const w = st.w
    const fwd = (front ? g.fwdF : g.fwdH) * w, bck = (front ? g.bckF : g.bckH) * w, lift = (front ? g.liftF : g.liftH) * w
    const u = frac(st.phase + g.off[key])
    let fx, fy = 0
    if (u < g.duty) fx = lerp(fwd, -bck, u / g.duty)
    else { const q = (u - g.duty) / (1 - g.duty); fx = lerp(-bck, fwd, q * q * (3 - 2 * q)); fy = -Math.sin(q * Math.PI) * lift }
    return [hip[0] + fx, fy]
  }

  function drawLeg(ctx, hip, foot, front, color, flash) {
    const knee = legIK(hip, foot, 16, 17, front)
    ctx.fillStyle = color; ctx.strokeStyle = color
    taper(ctx, hip, knee, front ? 6.6 : 8, front ? 3.6 : 3.8)
    // cannon bone and hoof
    const ex = foot[0] - knee[0], ey = foot[1] - knee[1], l = Math.hypot(ex, ey) || 1
    const pastern = [foot[0] - ex / l * 3, foot[1] - ey / l * 3]
    taper(ctx, knee, pastern, 3.6, 2.8)
    ctx.beginPath(); ctx.ellipse(foot[0] + ex / l * 0.5 + 0.8, foot[1] - 0.8, 3.4, 2.4, Math.atan2(ey, ex) * 0.5, 0, TAU); ctx.fill()
    if (!flash) { ctx.strokeStyle = 'rgba(235,225,210,0.16)'; ctx.lineWidth = 0.6; ctx.beginPath(); ctx.moveTo(hip[0], hip[1] + 2); ctx.lineTo(knee[0], knee[1]); ctx.stroke() }
  }

  // Paints the horse. st = { phase, mode, w, rear, paw, flow, nod, loose, riderFn }.
  // Returns a map from the (rotated) horse frame to this canvas's local space.
  function paintHorse(ctx, st, look, time, seed) {
    const flash = look.flash
    const col = look.color || '#17181b'
    const rear = st.rear || 0
    const gal = st.mode === 'gallop' ? st.w : 0
    const rock = Math.sin(st.phase * TAU * 1) * 0.045 * gal
    const ang = -rear * 0.7 + rock
    const bob = st.mode === 'stand' ? 0 : -Math.abs(Math.sin(st.phase * TAU)) * (st.mode === 'gallop' ? 3 : 1.6) * st.w
    const ca = Math.cos(ang), sa = Math.sin(ang)
    const P = (p) => {
      const dx = p[0] - PIVOT[0], dy = p[1] - PIVOT[1]
      return [PIVOT[0] + dx * ca - dy * sa, PIVOT[1] + dx * sa + dy * ca + bob]
    }
    ctx.lineCap = 'round'; ctx.lineJoin = 'round'
    if (look.halo) {
      ctx.save(); ctx.globalAlpha *= look.halo; ctx.fillStyle = 'rgba(239,231,212,1)'
      ctx.beginPath(); ctx.ellipse(2, -38, 34, 30, 0, 0, TAU); ctx.fill(); ctx.restore()
    }
    // legs
    const tuck = clamp(rear * 3, 0, 1)
    const hipF = P(HIP_F), hipH = P(HIP_H)
    const foreFoot = (key) => {
      const g = gaitFoot(HIP_F, true, key, st)
      if (st.paw && key === 'fn') { const pw = Math.max(0, Math.sin(time * 4.2 + seed)); g[0] += pw * 6; g[1] -= pw * 12 }
      const tk = [HIP_F[0] + 12, -21 - (key === 'fn' ? 3 : 0)]
      return P([lerp(g[0], tk[0], tuck), lerp(g[1], tk[1], tuck)])
    }
    const hindFoot = (key) => {
      const g = gaitFoot(HIP_H, false, key, st)
      if (rear > 0.01) g[0] = lerp(g[0], HIP_H[0] + (key === 'hn' ? 3 : -4), clamp(rear * 3, 0, 1))
      return g
    }
    const farC = flash ? col : 'rgba(17,18,21,0.82)'
    const legCol = look.color ? col : '#1a1b1e'
    ctx.globalAlpha *= 1
    drawLeg(ctx, hipH, hindFoot('hf'), false, flash ? col : '#303237', flash)
    drawLeg(ctx, hipF, foreFoot('ff'), true, flash ? col : '#303237', flash)
    void farC
    // body (rotated frame)
    ctx.save()
    ctx.translate(PIVOT[0], PIVOT[1] + bob); ctx.rotate(ang); ctx.translate(-PIVOT[0], -PIVOT[1])
    ctx.fillStyle = col; ctx.strokeStyle = col
    const flow = st.flow
    // tail
    for (let i = 0; i < 3; i++) {
      const w = Math.sin(time * 9 + i * 1.3 + seed) * (2 + flow * 4)
      ctx.lineWidth = 3.2 - i * 0.7
      ctx.beginPath(); ctx.moveTo(-22, -42 + i)
      ctx.quadraticCurveTo(-30 - flow * 8, -42 - flow * 3 + w * 0.4, -37 - flow * 17 - i * 2, -33 + (1 - flow) * 9 - i * 2 + w)
      ctx.stroke()
    }
    // wash 1: barrel, chest and rump
    ctx.beginPath(); ctx.ellipse(-1, -36, 22, 9.5, 0.03, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.ellipse(13, -37, 10.5, 11, 0, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.ellipse(-15, -38, 10.5, 10, 0, 0, TAU); ctx.fill()
    // neck and head
    const hd = [0, st.nod || 0]
    ctx.beginPath(); ctx.moveTo(6, -43)
    ctx.quadraticCurveTo(18, -58, 30 + hd[0], -62 + hd[1])
    ctx.lineTo(34 + hd[0], -53 + hd[1]); ctx.quadraticCurveTo(27, -43, 21, -29); ctx.closePath(); ctx.fill()
    const poll = [31 + hd[0], -61.5 + hd[1]], muzzle = [46 + hd[0], -49 + hd[1] + (st.headDown || 0)]
    taper(ctx, poll, muzzle, 8.2, 4.4)
    ctx.beginPath(); ctx.arc(36 + hd[0], -55 + hd[1], 4.8, 0, TAU); ctx.fill()
    ctx.beginPath(); ctx.moveTo(29 + hd[0], -63 + hd[1]); ctx.lineTo(28.5 + hd[0], -70 + hd[1]); ctx.lineTo(33 + hd[0], -64.5 + hd[1]); ctx.closePath(); ctx.fill()
    // mane
    ctx.lineWidth = 2.1
    for (let i = 0; i < 7; i++) {
      const t = i / 6
      const bx = lerp(9, 29, t) + hd[0] * t, by = lerp(-45, -63.5, Math.pow(t, 0.8)) + hd[1] * t
      const w = Math.sin(time * 8 + i * 0.9 + seed) * (1.5 + flow * 3)
      ctx.beginPath(); ctx.moveTo(bx, by)
      ctx.quadraticCurveTo(bx - 4 - flow * 5, by - 1 + w * 0.3, bx - 8 - flow * 11, by + 4 + w)
      ctx.stroke()
    }
    if (!flash) {
      // wash 2: lighter belly bloom and a wet highlight along the back
      ctx.fillStyle = 'rgba(92,88,82,0.30)'
      ctx.beginPath(); ctx.ellipse(-2, -32, 19, 5, 0.02, 0, TAU); ctx.fill()
      ctx.strokeStyle = 'rgba(235,225,210,0.17)'; ctx.lineWidth = 0.8
      ctx.beginPath(); ctx.moveTo(-22, -43); ctx.quadraticCurveTo(-5, -47, 9, -45); ctx.quadraticCurveTo(20, -58, 29, -63.5); ctx.stroke()
      // saddle cloth and saddle
      ctx.fillStyle = st.cloth || '#6b5a42'
      ctx.beginPath(); ctx.moveTo(-9, -44.5); ctx.lineTo(9, -44.5); ctx.lineTo(10.5, -29.5); ctx.lineTo(-10.5, -29.5); ctx.closePath(); ctx.fill()
      ctx.strokeStyle = 'rgba(20,18,16,0.8)'; ctx.lineWidth = 0.9
      ctx.beginPath(); ctx.moveTo(-10, -32); ctx.lineTo(10, -32); ctx.stroke()
      ctx.strokeStyle = 'rgba(235,225,210,0.5)'; ctx.lineWidth = 0.7
      ctx.beginPath(); ctx.moveTo(-9, -35); ctx.lineTo(9.4, -35); ctx.stroke()
      ctx.fillStyle = '#202125'
      ctx.beginPath(); ctx.ellipse(0, -45, 9, 3.4, 0, Math.PI, 0); ctx.fill()
      // bridle
      ctx.strokeStyle = 'rgba(235,225,210,0.42)'; ctx.lineWidth = 0.6
      ctx.beginPath(); ctx.moveTo(31 + hd[0], -62 + hd[1]); ctx.lineTo(38 + hd[0], -51 + hd[1]); ctx.moveTo(43 + hd[0], -52 + hd[1]); ctx.lineTo(37 + hd[0], -57 + hd[1]); ctx.stroke()
      // eye
      eyes(ctx, 37.5 + hd[0], -58 + hd[1], look.eyeGlow || 0, 4.2)
    }
    const reinFrom = [43 + hd[0], -51 + hd[1]]
    const map = (p) => P(p)
    if (st.riderFn) st.riderFn(ctx, map, reinFrom)
    else if (!flash) {
      // loose reins and dangling stirrup leathers
      ctx.strokeStyle = 'rgba(235,225,210,0.5)'; ctx.lineWidth = 0.8
      const sway = Math.sin(time * 9 + seed) * 2.5
      ctx.beginPath(); ctx.moveTo(reinFrom[0], reinFrom[1]); ctx.quadraticCurveTo(26, -34 + sway, 6, -44); ctx.stroke()
      ctx.beginPath(); ctx.moveTo(6, -43); ctx.lineTo(9 + sway * 0.4, -31); ctx.stroke()
      ctx.fillStyle = col; ctx.beginPath(); ctx.ellipse(9 + sway * 0.4, -30, 2.4, 1.5, 0, 0, TAU); ctx.fill()
    }
    ctx.restore()
    // near legs over the body
    drawLeg(ctx, hipH, hindFoot('hn'), false, flash ? col : legCol, flash)
    drawLeg(ctx, hipF, foreFoot('fn'), true, flash ? col : legCol, flash)
    return P
  }

  // Polearm drawn through grip point `g` at angle a. style 'lance' | 'halberd'. Returns tip.
  function drawPolearm(ctx, g, a, style, look, time) {
    const dx = Math.cos(a), dy = Math.sin(a), nx = -dy, ny = dx
    const hal = style === 'halberd'
    const back = hal ? 30 : 26, shaft = hal ? 78 : 70, head = hal ? 17 : 15
    const pt = (u, v) => [g[0] + dx * u + nx * v, g[1] + dy * u + ny * v]
    const tip = pt(shaft + head, 0)
    ctx.save()
    ctx.strokeStyle = look.flash ? look.color : '#3a2d22'; ctx.lineWidth = hal ? 2.8 : 2.4
    ctx.beginPath(); ctx.moveTo(...pt(-back, 0)); ctx.lineTo(...pt(shaft, 0)); ctx.stroke()
    if (!look.flash) {
      ctx.strokeStyle = 'rgba(235,225,210,0.5)'; ctx.lineWidth = 3; ctx.lineCap = 'butt'
      ctx.beginPath(); ctx.moveTo(...pt(2.5, 0)); ctx.lineTo(...pt(6, 0)); ctx.stroke(); ctx.lineCap = 'round'
    }
    ctx.fillStyle = look.flash ? look.color : '#e5dfd2'
    ctx.beginPath(); ctx.moveTo(...pt(shaft, 3.1)); ctx.lineTo(...tip); ctx.lineTo(...pt(shaft, -3.1)); ctx.closePath(); ctx.fill()
    if (hal) {
      // crescent blade bowing out of one side of the head
      ctx.beginPath()
      ctx.moveTo(...pt(shaft - 4, 2))
      ctx.quadraticCurveTo(...pt(shaft + 8, 17), ...pt(shaft + 1, 17.5))
      ctx.quadraticCurveTo(...pt(shaft - 2, 10), ...pt(shaft - 11, 3))
      ctx.closePath(); ctx.fill()
      ctx.beginPath(); ctx.moveTo(...pt(shaft - 4, -2)); ctx.lineTo(...pt(shaft + 3, -9)); ctx.lineTo(...pt(shaft - 9, -3)); ctx.closePath(); ctx.fill()
    }
    if (!look.flash) {
      ctx.fillStyle = CINNABAR
      const sway = Math.sin(time * 7) * 1.5
      ctx.beginPath()
      ctx.moveTo(...pt(shaft - 2, 0))
      ctx.quadraticCurveTo(pt(shaft - 6, 5)[0] + sway, pt(shaft - 6, 5)[1] + 3, pt(shaft - 14, 6)[0] + sway * 2, pt(shaft - 14, 6)[1] + 6 + sway)
      ctx.quadraticCurveTo(pt(shaft - 7, -1)[0], pt(shaft - 7, -1)[1] + 3, ...pt(shaft - 2, 0)); ctx.fill()
    }
    ctx.restore()
    return tip
  }

  function paintRider(ctx, kind, r, look, time, seed, map, reinFrom, flow) {
    const gen = kind === 'general'
    const flash = look.flash
    const col = look.color || (gen ? '#0d0e10' : '#1f2125')
    const armor = flash ? col : gen ? '#16171a' : '#25272c'
    const pale = 'rgba(235,225,210,0.3)'
    const H = [0, -44.5], len = gen ? 22 : 19.5
    const L = r.lean
    const N = [H[0] + Math.sin(L) * len, H[1] - Math.cos(L) * len]
    const Sh = [N[0] - Math.sin(L) * 2.5, N[1] + Math.cos(L) * 2.5]
    const headTilt = L * 0.5
    const hr = gen ? 6.2 : 5.4
    const HC = [N[0] + Math.sin(headTilt) * (hr + 1.2), N[1] - Math.cos(headTilt) * (hr + 1.2)]
    const out = { tip: null, head: HC }
    ctx.lineCap = 'round'; ctx.lineJoin = 'round'
    ctx.fillStyle = col; ctx.strokeStyle = col
    // back flags and cape
    if (gen && !flash) {
      for (let i = 0; i < 4; i++) {
        const a = -1.85 - i * 0.3, L2 = 24 - i * 1.5
        const dx = Math.cos(a), dy = Math.sin(a), nx = -dy, ny = dx
        const ax = Sh[0] - 3, ay = Sh[1] - 2
        const tx = ax + dx * L2, ty = ay + dy * L2
        const sway = Math.sin(time * 6 + i * 1.1 + seed) * (1.2 + flow * 3) - flow * 5
        ctx.lineWidth = 1.4; ctx.beginPath(); ctx.moveTo(ax, ay); ctx.lineTo(tx, ty); ctx.stroke()
        ctx.beginPath(); ctx.moveTo(ax + dx * L2 * 0.3, ay + dy * L2 * 0.3); ctx.lineTo(tx, ty)
        ctx.quadraticCurveTo(tx - dx * 4 - nx * 4 + sway * 0.4, ty - dy * 4 - ny * 4, ax + dx * L2 * 0.3 - nx * 7 + sway, ay + dy * L2 * 0.3 - ny * 7 + 2); ctx.closePath(); ctx.fill()
      }
    }
    const wave = Math.sin(time * 8 + seed) * (1.5 + flow * 3)
    const capeLen = gen ? 20 : 13
    ctx.beginPath(); ctx.moveTo(N[0] - 2, N[1] + 1)
    ctx.quadraticCurveTo(N[0] - 8 - flow * capeLen * 0.4, N[1] + 6, N[0] - 6 - capeLen * 0.5 - flow * capeLen * 0.8, N[1] + 12 + capeLen * 0.2 + wave)
    ctx.quadraticCurveTo(N[0] - 6 - capeLen * 0.2 - flow * capeLen * 0.5, H[1] + 6 + wave * 0.5, H[0] - 5, H[1] + 4)
    ctx.closePath(); ctx.fill()
    // rider's leg and stirrup
    ctx.lineWidth = gen ? 7 : 6
    const K = [H[0] + 9, H[1] + 7], F = [H[0] + 5, H[1] + 18]
    ctx.beginPath(); ctx.moveTo(H[0], H[1]); ctx.lineTo(K[0], K[1]); ctx.lineTo(F[0], F[1]); ctx.stroke()
    ctx.beginPath(); ctx.moveTo(F[0] - 2, F[1] + 0.5); ctx.lineTo(F[0] + 4.5, F[1] + 0.8); ctx.lineWidth = 3.4; ctx.stroke()
    // torso: armoured body
    ctx.fillStyle = armor; ctx.strokeStyle = armor
    ctx.lineWidth = gen ? 14 : 12
    ctx.beginPath(); ctx.moveTo(H[0], H[1]); ctx.lineTo(N[0], N[1]); ctx.stroke()
    ctx.beginPath(); ctx.moveTo(H[0] - 7, H[1] + 1); ctx.lineTo(H[0] + 8, H[1] + 1); ctx.lineTo(H[0] + 10.5, H[1] + 8); ctx.lineTo(H[0] - 9, H[1] + 8); ctx.closePath(); ctx.fill()
    ctx.beginPath(); ctx.ellipse(Sh[0] + 0.5, Sh[1] + 1.5, gen ? 8.5 : 7, gen ? 6 : 5, -0.2, 0, TAU); ctx.fill()
    if (!flash) {
      ctx.strokeStyle = pale; ctx.lineWidth = 0.7
      for (let k = 0; k < 4; k++) { const t = (k + 1) / 5; const px = lerp(H[0], N[0], t), py = lerp(H[1], N[1], t); ctx.beginPath(); ctx.moveTo(px - 6, py); ctx.lineTo(px + 6, py); ctx.stroke() }
      ctx.beginPath(); ctx.moveTo(H[0] - 8, H[1] + 4.5); ctx.lineTo(H[0] + 9, H[1] + 4.5); ctx.stroke()
    }
    // arms
    const aim = r.a
    const gp = [Sh[0] + 6 + Math.cos(aim) * 4, Sh[1] + 6 + Math.sin(aim) * 5]
    const upper = gen ? 11 : 10, fore = gen ? 10.5 : 9.5
    // off hand: reins or the raised command flag
    const reinHand = [Sh[0] + 14, Sh[1] + 5]
    const raised = [Sh[0] + 4, Sh[1] - 17]
    const offT = [lerp(reinHand[0], raised[0], r.flag), lerp(reinHand[1], raised[1], r.flag)]
    const [ox, oy, ohx, ohy] = ik(Sh[0], Sh[1], offT[0], offT[1], upper, fore)
    if (!flash) {
      ctx.strokeStyle = 'rgba(235,225,210,0.5)'; ctx.lineWidth = 0.8
      ctx.beginPath(); ctx.moveTo(ohx, ohy); ctx.quadraticCurveTo((ohx + reinFrom[0]) / 2, Math.max(ohy, reinFrom[1]) + 4, reinFrom[0], reinFrom[1]); ctx.stroke()
    }
    // weapon
    out.tip = drawPolearm(ctx, gp, aim, gen ? 'halberd' : 'lance', look, time)
    ctx.strokeStyle = col; ctx.fillStyle = col
    const limbW = gen ? 6.4 : 5.2
    if (r.flag > 0.04) drawBanner(ctx, [ohx, ohy], 1, look, time, seed)
    ctx.strokeStyle = armor
    const [ex, ey, hx, hy] = ik(Sh[0], Sh[1], gp[0], gp[1], upper, fore)
    ctx.lineWidth = limbW; ctx.beginPath(); ctx.moveTo(Sh[0], Sh[1]); ctx.lineTo(ex, ey); ctx.lineTo(hx, hy); ctx.stroke()
    ctx.strokeStyle = armor; ctx.lineWidth = limbW; ctx.beginPath(); ctx.moveTo(Sh[0] - 1, Sh[1]); ctx.lineTo(ox, oy); ctx.lineTo(ohx, ohy); ctx.stroke()
    ctx.fillStyle = col
    ctx.beginPath(); ctx.arc(hx, hy, 3, 0, TAU); ctx.fill(); ctx.beginPath(); ctx.arc(ohx, ohy, 2.8, 0, TAU); ctx.fill()
    // head and helmet
    ctx.fillStyle = col
    ctx.beginPath(); ctx.arc(HC[0], HC[1], hr, 0, TAU); ctx.fill()
    const metal = flash ? col : gen ? '#17181b' : '#2a2c31'
    ctx.save(); ctx.translate(HC[0], HC[1]); ctx.rotate(headTilt * 0.7)
    ctx.fillStyle = metal; ctx.strokeStyle = metal
    const dome = hr * (gen ? 1.3 : 1.14), cy = -hr * 0.6
    ctx.beginPath(); ctx.arc(0, cy, dome, Math.PI, 0); ctx.fill()
    ctx.beginPath(); ctx.roundRect(-dome - 0.6, cy - 0.4, dome * 2 + 1.2, 2.3, 1); ctx.fill()
    const nl = gen ? 12 : 7
    ctx.beginPath(); ctx.moveTo(-dome * 0.95, cy + 1); ctx.lineTo(-dome - 2.5 - flow * 3, cy + nl * 0.6); ctx.lineTo(-dome * 0.4, cy + nl); ctx.lineTo(-dome * 0.35, cy + 2); ctx.closePath(); ctx.fill()
    ctx.beginPath(); ctx.arc(0, cy - dome - 0.8, 1.6, 0, TAU); ctx.fill()
    const top = cy - dome - 1
    if (!flash) {
      ctx.strokeStyle = pale; ctx.lineWidth = 0.6
      ctx.beginPath(); ctx.arc(0, cy, dome - 1.4, Math.PI * 1.1, Math.PI * 1.85); ctx.stroke()
    }
    if (gen) {
      if (!flash) {
        ctx.fillStyle = CINNABAR; ctx.strokeStyle = CINNABAR
        const sway = Math.sin(time * 5 + seed) * 1.8, back = 8 + flow * 13
        ctx.beginPath(); ctx.moveTo(-2, top - 2)
        ctx.quadraticCurveTo(-3, top - 22, 4 - back * 0.3, top - 30 + sway)
        ctx.quadraticCurveTo(-back * 0.5, top - 20, -back, top - 16 + sway * 1.5)
        ctx.quadraticCurveTo(-back * 0.35, top - 12, 2.5, top - 2); ctx.closePath(); ctx.fill()
        ctx.lineWidth = 1
        for (let i = 0; i < 4; i++) { ctx.beginPath(); ctx.moveTo(0, top - 4); ctx.quadraticCurveTo(-6 - i * 2, top - 17 + i * 3, -back - 4 - i * 2 - flow * 5, top - 11 + i * 4 + sway); ctx.stroke() }
      }
    } else {
      ctx.strokeStyle = flash ? col : '#17181a'; ctx.lineWidth = 1.7
      for (let i = 0; i < 4; i++) {
        const w = Math.sin(time * 9 + i + seed) * (1 + flow * 2)
        ctx.beginPath(); ctx.moveTo(0, top); ctx.quadraticCurveTo(-3 - i, top - 4 - flow, -6 - i * 1.8 - flow * 6, top + 1 + i * 1.6 + w); ctx.stroke()
      }
    }
    ctx.restore()
    if (!flash) eyes(ctx, HC[0] + hr * 0.42, HC[1] - hr * 0.1, look.eyeGlow || 0, hr)
    return out
  }

  // ---- rider state -> targets
  function riderTargets(kind, e) {
    const T = { a: -1.38, rear: 0, lean: 0.03, mode: 'stand', w: 0, flag: 0, paw: true, snap: false, nod: 0, headDown: 0 }
    const p = e.teleTotal ? 1 - Math.max(0, e.t) / e.teleTotal : 0
    const st = e.state
    const gen = kind === 'general'
    switch (st) {
      case 'walk':
      case 'reposition': T.mode = 'trot'; T.w = 0.95; T.a = -1.3; T.lean = -0.04; T.paw = false; break
      case 'tele':
      case 'chargeTell': {
        const q = ease(p)
        T.paw = !gen && false
        T.rear = (gen ? 0.12 : 0.34) * Math.sin(Math.min(1, p * 1.25) * Math.PI * 0.8)
        T.a = p < 0.5 ? lerp(-0.9, -1.5, ease(p * 2)) : lerp(-1.5, -0.03, ease((p - 0.5) * 2))
        if (gen && st === 'chargeTell') { T.a = lerp(-1.3, -0.03, ease(p * 1.4)); T.paw = true; T.rear = 0 }
        T.lean = lerp(0.0, 0.3, q)
        break
      }
      case 'attack':
      case 'charge': T.mode = 'gallop'; T.w = 1; T.a = -0.03; T.lean = 0.38; T.paw = false; break
      case 'recover':
        T.rear = 0.32; T.a = -0.45; T.lean = -0.2; T.paw = false; break
      case 'hurt': T.rear = 0.22; T.a = -1.0; T.lean = -0.4; T.paw = false; T.headDown = -3; break
      case 'roar': T.rear = 0.6; T.a = -1.0; T.lean = -0.5; T.paw = false; T.nod = -3; break
      case 'broken':
      case 'dying': T.lean = 0.75; T.a = 1.2; T.paw = false; T.headDown = 8; T.nod = 4; break
      case 'flagTell': T.flag = ease(p * 2.2); T.rear = 0.2 * Math.sin(Math.min(1, p) * Math.PI); T.a = -1.3; T.lean = -0.08; T.paw = false; break
      case 'combo': {
        const step = e.combo ? e.combo.i : 0
        const raised = [-2.45, 2.7][Math.min(1, step)]
        const swing = [0.85, -0.35][Math.min(1, step)]
        if (e.combo?.phase === 'tele') { T.a = lerp(-1.3, raised, ease(p * 1.5)); T.lean = -0.1 }
        else { T.a = swing; T.lean = 0.5; T.snap = true }
        T.paw = false
        break
      }
    }
    return T
  }

  function riderState(anim, T, e, dt, facing) {
    const r = anim.r || (anim.r = { a: T.a, rear: 0, lean: 0, w: 0, flag: 0, gait: 0, dir: 1, nod: 0, headDown: 0 })
    const k = T.snap ? 1 : 1 - Math.exp(-dt * 14)
    for (const key of ['a', 'rear', 'lean', 'flag', 'nod', 'headDown']) r[key] += (T[key] - r[key]) * k
    r.w += (T.w - r.w) * (1 - Math.exp(-dt * 9))
    const vx = e.vx || 0
    if (Math.abs(vx) > 8) r.dir = vx * facing >= 0 ? 1 : -1
    const sp = Math.abs(vx)
    const stride = T.mode === 'gallop' ? 110 : 62
    const eff = T.mode === 'stand' && r.w < 0.05 ? 0 : Math.max(sp, T.mode === 'trot' ? 40 : T.mode === 'gallop' ? 200 : 0)
    r.gait += r.dir * eff * dt / stride
    r.mode = T.mode === 'stand' && r.w > 0.05 ? (r.mode || 'trot') : T.mode
    return r
  }

  function drawRider(ctx, kind, enemy, anim, x, y, facing, look, time, dt = 1 / 60) {
    const gen = kind === 'general'
    const S = gen ? 1.25 : 1
    const T = riderTargets(kind, enemy)
    const r = riderState(anim, T, enemy, dt || 1 / 60, facing)
    const seed = enemy.seed || 0
    const flow = clamp(Math.abs(enemy.vx || 0) / 320 + (r.mode === 'gallop' ? r.w * 0.5 : 0), 0, 1)
    let res = { tip: null, head: [x, y - 80 * S] }
    let fnMap = null
    ctx.save()
    ctx.translate(x, y); ctx.scale(facing * S, S)
    if (look.alpha !== undefined) ctx.globalAlpha *= look.alpha
    const st = {
      phase: r.gait, mode: r.mode || 'stand', w: r.w, rear: r.rear, paw: T.paw && r.w < 0.1, flow, nod: r.nod + (r.mode === 'gallop' ? Math.sin(r.gait * TAU) * 1.8 : 0), headDown: r.headDown,
      cloth: gen ? '#5a4a36' : '#6b5a42',
      riderFn: (c, map, rein) => { res = paintRider(c, kind, r, look, time, seed, map, rein, flow); fnMap = map },
    }
    const P = paintHorse(ctx, st, look, time, seed)
    ctx.restore()
    const toWorld = (p) => { const m = P(p); return [x + facing * S * m[0], y + S * m[1]] }
    // tip/head are in the rotated horse frame; convert
    res = { tip: res.tip && toWorld(res.tip), head: toWorld(res.head) }
    void fnMap
    return res
  }

  // Riderless horse for after the rider is thrown. phase = gallop phase (cycles).
  function drawHorse(ctx, x, y, facing, look, time, phase = 0) {
    ctx.save()
    ctx.translate(x, y); ctx.scale(facing, 1)
    if (look.alpha !== undefined) ctx.globalAlpha *= look.alpha
    paintHorse(ctx, { phase, mode: 'gallop', w: 1, rear: 0, paw: false, flow: 1, nod: Math.sin(phase * TAU) * 2, headDown: 2, riderFn: null }, look, time, 0)
    ctx.restore()
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

  // A coiled sea dragon, with a continuous tapered spine and an articulated jaw.
  function dragonSpine(control, time) {
    const pts = []
    for (let k = 0; k < control.length-1; k++) {
      const a=control[Math.max(0,k-1)], b=control[k], c=control[k+1], d=control[Math.min(control.length-1,k+2)]
      for (let j=0; j<7; j++) {
        const t=j/7, u=(k+t)/(control.length-1)
        const curve = (n) => b[n]+.5*((c[n]-a[n])*t+(2*a[n]-5*b[n]+4*c[n]-d[n])*t*t+(-a[n]+3*b[n]-3*c[n]+d[n])*t*t*t)
        pts.push({ x:curve(0)+Math.sin(time*2.3+u*9)*u*5, y:curve(1)+Math.sin(time*2.7+u*13)*u*6, w:curve(2) })
      }
    }
    const [x,y,w]=control[control.length-1]
    pts.push({x,y,w})
    pts.forEach((p,i)=>{
      const a=pts[Math.max(0,i-1)], b=pts[Math.min(pts.length-1,i+1)]
      const dx=b.x-a.x,dy=b.y-a.y,len=Math.hypot(dx,dy)||1
      p.nx=-dy/len; p.ny=dx/len; p.angle=Math.atan2(dy,dx)
    })
    return pts
  }

  function dragonRim(p, side) {
    return [p.x+p.nx*p.w*side, p.y+p.ny*p.w*side]
  }

  function drawDragonBody(g, pts, ink, look, height, time) {
    for (let i=10; i<pts.length-8; i+=5) {
      const p=pts[i], [x,y]=dragonRim(p,1)
      const size=(.54+Math.sin(i*.23)*.12)*p.w
      g.save();g.translate(x,y);g.rotate(p.angle)
      g.fillStyle=look.color||'#263735'
      g.beginPath();g.moveTo(-9,-2)
      g.bezierCurveTo(-1,3,4,size*.64,12,size)
      g.quadraticCurveTo(8,size*.27,18,-3);g.closePath();g.fill()
      if(!look.color) {
        g.strokeStyle='rgba(163,179,155,.44)';g.lineWidth=.8
        g.beginPath();g.moveTo(-6,0);g.quadraticCurveTo(4,size*.68,12,size);g.stroke()
      }
      g.restore()
    }
    let coat=ink
    if (!look.color) {
      const wash=g.createLinearGradient(0,-height-22,25,13)
      wash.addColorStop(0,'#536564');wash.addColorStop(.31,'#354d4c')
      wash.addColorStop(.68,'#233637');wash.addColorStop(1,'#1b2a2d')
      coat=wash
    }
    g.fillStyle=coat;g.beginPath()
    pts.forEach((p,i)=>{const edge=dragonRim(p,1);i?g.lineTo(...edge):g.moveTo(...edge)})
    for(let i=pts.length-1;i>=0;i--)g.lineTo(...dragonRim(pts[i],-1))
    g.closePath();g.fill()
    if(look.color)return

    g.save();g.clip()
    const belly=g.createLinearGradient(0,-height,0,9)
    belly.addColorStop(0,'rgba(178,183,145,.35)');belly.addColorStop(.65,'rgba(129,144,122,.30)')
    belly.addColorStop(1,'rgba(94,120,112,.16)')
    g.fillStyle=belly;g.beginPath()
    pts.forEach((p,i)=>{const edge=dragonRim(p,-.44);i?g.lineTo(...edge):g.moveTo(...edge)})
    for(let i=pts.length-1;i>=0;i--)g.lineTo(...dragonRim(pts[i],-1))
    g.closePath();g.fill()

    g.strokeStyle='rgba(216,220,181,.32)';g.lineWidth=1.05;g.beginPath()
    for(let i=3;i<pts.length-4;i+=5) {
      for(let j=i;j<i+4;j++) {
        const edge=dragonRim(pts[j],.92+Math.sin(j*1.2)*.07)
        j===i?g.moveTo(...edge):g.lineTo(...edge)
      }
    }
    g.stroke()
    for(let i=2;i<pts.length-2;i+=2) {
      const p=pts[i], w=p.w
      g.save();g.translate(p.x,p.y);g.rotate(p.angle)
      const size=2.3+w*.21
      g.strokeStyle='rgba(158,180,169,.30)';g.lineWidth=.7;g.beginPath()
      for(const side of [-.1,.29,.64]) {
        const dy=side*w, shift=side>.5?2:0
        g.moveTo(-size+shift,dy-size*.52)
        g.quadraticCurveTo(size*.7+shift,dy-size*.49,size+shift,dy)
        g.quadraticCurveTo(size*.43+shift,dy+size*.69,-size+shift,dy+size*.4)
      }
      g.stroke()
      g.strokeStyle='rgba(22,36,34,.41)';g.lineWidth=.85;g.beginPath()
      g.moveTo(-2,-w*.42);g.quadraticCurveTo(3,-w*.64,4,-w*.97);g.stroke()
      g.restore()
    }
    g.restore()

    // Small breaks of white water trace the tail as it disappears below the surface.
    const tail=pts[pts.length-2]
    g.save();g.lineCap='round'
    for (let i=0;i<3;i++) {
      const x=tail.x+16+i*8, y=-7+i*4+Math.sin(time*3+i)*1.8
      g.strokeStyle=`rgba(230,232,198,${.32-i*.065})`;g.lineWidth=1.3-i*.18
      g.beginPath();g.moveTo(x-22,y);g.quadraticCurveTo(x,y-4,x+27,y);g.stroke()
    }
    g.restore()
  }

  function drawDragonFace(g, ink, look, jaw, time) {
    const art=globalThis.XiabanInkArt
    const ivory=look.color||'#cfcbb0', ridge=look.color||'#788981'
    for(let i=0;i<8;i++) {
      const y=-21+i*4, wake=Math.sin(time*3.4+i)*4
      g.fillStyle=look.color||(i%2?'#304744':'#263837')
      art.ribbon(g, [[-17,y],[-33,y-3],[-48,y+1+wake],[-64-i%3*5,y+6+wake]], t=>(8-i*.43)*Math.pow(1-t,.62))
    }
    for(const [x,y,size] of [[-18,-23,1.02],[-3,-27,.72]]) {
      g.save();g.translate(x,y);g.scale(size,size)
      g.fillStyle=look.color||'#626c5e';g.beginPath();g.moveTo(-4,5)
      g.bezierCurveTo(-13,-6,-15,-15,-13,-25)
      g.quadraticCurveTo(-9,-22,-7,-14)
      g.quadraticCurveTo(-5,-24,-.7,-29)
      g.quadraticCurveTo(-4,-11,3,3);g.closePath();g.fill()
      if(!look.color) {
        g.strokeStyle='rgba(216,209,165,.53)';g.lineWidth=.85;g.beginPath()
        g.moveTo(-10,-19);g.quadraticCurveTo(-9,-9,-3,1);g.stroke()
      }
      g.restore()
    }

    g.fillStyle=look.color||'#192829';g.beginPath();g.moveTo(-23,-1)
    g.quadraticCurveTo(9,1,47,-2);g.lineTo(46,7+jaw*42)
    g.quadraticCurveTo(19,14+jaw*28,-18,13);g.closePath();g.fill()
    g.save();g.translate(-19,5);g.rotate(jaw)
    g.fillStyle=look.color||'#304441';g.beginPath();g.moveTo(-11,-4)
    g.quadraticCurveTo(7,2,30,2);g.lineTo(62,-5)
    g.quadraticCurveTo(64,-1,55,5);g.lineTo(20,12)
    g.quadraticCurveTo(-7,11,-13,1);g.closePath();g.fill()
    g.strokeStyle=ridge;g.lineWidth=.84;g.beginPath();g.moveTo(7,8)
    g.quadraticCurveTo(29,10,48,4);g.stroke()
    g.fillStyle=ivory;g.beginPath();g.moveTo(51,-2);g.quadraticCurveTo(55,-8,55,-10)
    g.lineTo(57,-4);g.closePath();g.fill();g.restore()

    let face=ink
    if (!look.color) {
      const wash=g.createLinearGradient(-2,-32,2,23)
      wash.addColorStop(0,'#586c65');wash.addColorStop(.31,'#394f4f')
      wash.addColorStop(.72,'#22363c');wash.addColorStop(1,'#202c2d');face=wash
    }
    g.fillStyle=face;g.beginPath();g.moveTo(-32,11)
    g.bezierCurveTo(-40,-2,-29,-24,-14,-26)
    g.quadraticCurveTo(-1,-34,9,-23);g.lineTo(19,-19)
    g.quadraticCurveTo(32,-16,41,-20);g.quadraticCurveTo(54,-23,59,-13)
    g.quadraticCurveTo(61,-5,51,-5);g.quadraticCurveTo(41,-6,34,-1)
    g.lineTo(16,3);g.lineTo(7,11);g.quadraticCurveTo(-16,20,-32,11);g.closePath();g.fill()
    if (!look.color) {
      g.save();g.clip()
      for (let i=0;i<13;i++) {
        const x=-29+i*5.9,y=-18+Math.sin(i*2.4)*6
        g.strokeStyle=i%3?'rgba(177,188,151,.18)':'rgba(23,36,37,.32)';g.lineWidth=i%4? .73:1.8
        g.beginPath();g.moveTo(x,y);g.lineTo(x+6,y-2);g.lineTo(x+9,y-1.8);g.stroke()
      }
      g.restore()
    }

    g.fillStyle=ivory
    for(const [x,y,size] of [[32,-1,9],[45,-4,6]]) {
      g.beginPath();g.moveTo(x-2.8,y);g.quadraticCurveTo(x+2.7,y+size*.37,x-1.3,y+size)
      g.quadraticCurveTo(x+.1,y+size*.45,x-5,y+1);g.closePath();g.fill()
    }
    if (!look.color) {
      g.strokeStyle='rgba(184,196,162,.61)';g.lineWidth=1
      g.beginPath();g.moveTo(-21,-15);g.quadraticCurveTo(-8,-30,4,-22)
      g.moveTo(16,-16);g.quadraticCurveTo(31,-12,41,-17);g.stroke()
      g.strokeStyle='rgba(156,177,159,.57)';g.lineWidth=1.2
      for(let i=0;i<3;i++) {
        g.beginPath();g.moveTo(-19+i*5,-3+i*2)
        g.quadraticCurveTo(-15+i*4,5+i*2,-25+i*6,13+i);g.stroke()
      }
      g.fillStyle='#152326';g.beginPath();g.ellipse(49,-13,3.7,1.7,-.39,0,TAU);g.fill()
      g.fillStyle='#101e24';g.beginPath();g.moveTo(-9,-18);g.lineTo(12,-13)
      g.quadraticCurveTo(1,-9,-9,-18);g.fill()
      g.strokeStyle='rgba(191,179,129,.7)';g.lineWidth=.92
      g.beginPath();g.moveTo(-10,-20);g.quadraticCurveTo(3,-18,13,-14);g.stroke()
      g.fillStyle=CINNABAR;g.beginPath();g.ellipse(2,-14.3,2.4+(look.eyeGlow||0)*.72,1.17,.21,0,TAU);g.fill()
    }
    for(let i=0;i<2;i++) {
      const wave=Math.sin(time*3.7+i*2)*7
      g.strokeStyle=i?ridge:ivory;g.lineWidth=i?1:1.4
      g.beginPath();g.moveTo(39,-3);g.bezierCurveTo(57,2+i*4,64,-12+wave-i*4,80+i*13,-9+wave-i*13);g.stroke()
    }
  }

  // Only the silhouette changes with the pose. The body behind the head remains scenery.
  function drawDragon(g,e,x,y,facing,look={},time=0) {
    const tell=e.state==='dragonTell', bite=e.state==='dragonBite', wave=e.state==='dragonWaveTell'||e.state==='dragonWave'
    const broken=e.state==='broken', recovering=e.state==='recover', diving=e.state==='dragonDive'
    if(e.state==='dragonDiveTell')return {head:null,tip:null}
    const q=e.teleTotal?clamp(1-e.t/e.teleTotal,0,1):0
    const headX=bite?40:tell?-18*q:recovering?6:12
    const headY=broken?-29:wave?-111:tell?-70-q*20:-66
    const tilt=tell?-.13*q:bite?.14:wave?-.48:broken?.18:0
    const sink=diving?(1-clamp(e.t/.6,0,1))*260:e.state==='intro'?Math.max(0,e.t/2.2)*150:e.state==='dying'?(1-clamp(e.t/2.6,0,1))*200:0
    const tip=[x+(headX+50*Math.cos(tilt)+5*Math.sin(tilt))*facing,y+headY+50*Math.sin(tilt)-5*Math.cos(tilt)+sink]
    const rise=broken?.37:wave?1.06:1
    const pts=dragonSpine([[headX-10,headY+3,17],[-39,-84*rise,20],[-76,-124*rise,24],[-126,-153*rise,29],[-169,-132*rise,28],
      [-190,-86*rise,24],[-217,-49*rise,18],[-253,-48*rise,13],[-289,-32*rise,8],[-323,8,2]],time)
    const ink=look.color||'#263b3d'
    g.save();g.translate(x,y);g.scale(facing,1)
    if(look.alpha!==undefined)g.globalAlpha*=look.alpha
    g.beginPath();g.rect(-351,-340,522,345);g.clip();g.translate(0,sink)
    drawDragonBody(g,pts,ink,look,153*rise,time)
    const p=pts[11], stroke=globalThis.XiabanInkArt
    g.fillStyle=ink
    stroke.ribbon(g,[[p.x-5,p.y+9],[p.x+9,p.y+25],[p.x+3,p.y+43],[p.x+14,p.y+49]],t=>12*(1-t)+1)
    for(let i=0;i<3;i++){
      stroke.ribbon(g,[[p.x+2,p.y+41],[p.x+13+i*4,p.y+44+i*4],[p.x+15+i*4,p.y+52+i*2]],t=>4*(1-t))
    }
    g.translate(headX,headY);g.rotate(tilt)
    const jaw=tell?.12+q*.26:bite?.065:wave?.4:broken?.22:.035+Math.sin(time*1.8)*.019
    drawDragonFace(g,ink,look,jaw,time)
    g.restore()
    return {head:[x+headX*facing,y+headY+sink],tip}
  }

  globalThis.XiabanInkFigures = {
    KINDS, CINNABAR, PAPER, createAnim, updatePose, drawHumanoid, drawDragon, drawRider, drawHorse, updateScarf, drawCrow, drawDummy, updateDummy, basePose, skeleton,
  }
})()
