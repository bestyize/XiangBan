/* 水墨剑客 · 卷七《归墨卷》角色造型与动作：墨影卒、墨影弓手、无名剑客（与主角同骨同高，纸白围巾、斗笠裂纸收尾）。 */
(() => {
  'use strict'

  const F = globalThis.XiabanInkFigures
  const PAPER = '#f4eddc'
  const INK = '#16171a'
  const TAU = Math.PI * 2
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v))
  const lerp = (a, b, t) => a + (b - a) * t
  const ease = (q) => 1 - Math.pow(1 - clamp(q, 0, 1), 3)

  if (F?.KINDS) {
    F.KINDS.shade = {
      thigh: 13, shin: 13, torso: 18, upper: 10, fore: 9.5, head: 5.2,
      limbs: [5.2, 4.3, 4, 3.4], blade: 35, bladeW: 2.3, color: '#17181b',
      robe: 'robe', hat: 'douli', returnInk: true,
    }
    F.KINDS.shadeArcher = {
      thigh: 13, shin: 13, torso: 18, upper: 10, fore: 10, head: 5.2,
      limbs: [4.6, 3.9, 3.6, 3.1], blade: 0, color: '#1b1c20',
      robe: 'hood', headStyle: 'hood', bow: true, returnInk: true,
    }
    F.KINDS.nameless = {
      thigh: 13, shin: 13, torso: 18, upper: 10, fore: 9.5, head: 5.2,
      limbs: [5.2, 4.3, 4, 3.4], blade: 35, bladeW: 2.3, color: '#111214',
      robe: 'robe', hat: 'douli', returnInk: true,
    }
  }

  function runLegs(t, phase, amount) {
    const s = Math.sin(phase), c = Math.cos(phase)
    t.l1 = s * 0.85 * amount
    t.b1 = 0.25 + Math.max(0, c) * 1.25 * amount
    t.l2 = -s * 0.85 * amount
    t.b2 = 0.25 + Math.max(0, -c) * 1.25 * amount
    t.bob = Math.abs(Math.cos(phase)) * 1.6 * amount
  }

  function pose(e, t, anim, time, progress, speed) {
    if (e.type === 'shadeArcher') {
      t.lean = 0.08; t.ou = 1.5; t.of = 1.52; t.sw = 0
      if (e.state === 'spawn') { t.crouch = 8; t.lean = 0.5 }
      else if (e.state === 'tele' || e.state === 'arrowCounter') {
        t.draw = ease(progress)
        t.lean = e.state === 'arrowCounter' ? -0.14 : 0.02
      } else if (e.state === 'hop') {
        t.air = 1; t.l1 = 0.7; t.b1 = 1.2; t.l2 = -0.1; t.b2 = 0.8
      } else if (e.state === 'hurt') {
        t.lean = -0.45; t.crouch = 4; t.head = -0.25
      } else if (e.state === 'stagger') {
        t.lean = 0.42; t.crouch = 6; t.head = 0.35
      }
      return t
    }

    // Swordsman idle mirrors the protagonist's stance (slightly leaning forward for shades).
    const breathe = Math.sin(time * 1.8 + (e.seed || 0))
    t.lean = (e.type === 'shade' ? 0.14 : 0.06) + breathe * 0.02
    t.crouch = 1.4 + breathe * 0.6
    t.sw = 0.95; t.reach = 0.9; t.wrist = -0.28
    t.ou = 0.1; t.of = -0.4

    switch (e.state) {
      case 'spawn':
        t.crouch = 7; t.lean = 0.5
        break
      case 'intro': {
        const q = clamp(1 - e.t / 0.8, 0, 1)
        t.lean = 0.04
        t.sw = lerp(2.35, 0.95, ease(clamp((q - 0.2) / 0.65, 0, 1)))
        t.reach = 0.88
        break
      }
      case 'walk':
        runLegs(t, anim.phase, clamp(speed / 200, 0.35, 0.9))
        t.lean = 0.24; t.crouch = 2; t.sw = 2.5; t.wrist = -0.4
        t.ou = -Math.sin(anim.phase) * 0.85; t.of = t.ou - 0.85
        break
      case 'tele':
      case 'riposteTele':
      case 'duelTell':
      case 'duelThrustTell': {
        const raised = e.state === 'duelThrustTell' ? -0.35 : -2.25
        t.sw = lerp(0.95, raised, ease(progress * 1.4))
        t.reach = 1
        t.lean = lerp(0.22, e.state === 'duelThrustTell' ? 0.32 : -0.12, progress)
        t.crouch = 3 + progress * 2
        Object.assign(t, { l1: 0.48, b1: 0.5, l2: -0.48, b2: 0.2 })
        break
      }
      case 'attack':
      case 'riposteAttack':
      case 'duelAttack':
      case 'duelThrustAttack':
        t.sw = e.state === 'duelThrustAttack' ? 0.08 : 0.88
        t.reach = 1; t.lean = 0.52; t.crouch = 4.5
        Object.assign(t, { l1: 0.72, b1: 0.45, l2: -0.72, b2: 0.12 })
        t.ou = -0.7; t.of = -1.3
        anim.snap = true
        break
      case 'airRiposteTele':
        t.air = 1; t.sw = -2.45; t.reach = 1; t.lean = 0.2; t.rot = -0.28 + Math.sin(time * 8) * 0.06
        Object.assign(t, { l1: 0.8, b1: 1.35, l2: 0.1, b2: 1.1 })
        break
      case 'airRiposteAttack':
        t.air = 1; t.sw = 0.98; t.reach = 1; t.lean = 0.55; t.rot = 0.18
        Object.assign(t, { l1: 0.65, b1: 0.9, l2: -0.5, b2: 0.5 })
        anim.snap = true
        break
      case 'combo': {
        const c = e.combo
        if (c?.phase === 'tele') {
          const raised = c.i === 1 ? 2.65 : -2.35
          t.sw = lerp(0.9, raised, ease(progress * 1.4))
          t.reach = 1; t.lean = lerp(0.25, -0.14, progress); t.crouch = 3.5 + progress * 2
          Object.assign(t, { l1: 0.48, b1: 0.5, l2: -0.48, b2: 0.2 })
        } else {
          t.sw = c?.i === 1 ? -1.4 : 0.95
          t.reach = 1; t.lean = 0.52; t.crouch = 4.5
          Object.assign(t, { l1: 0.74, b1: 0.4, l2: -0.74, b2: 0.1 })
          anim.snap = true
        }
        break
      }
      case 'crouch':
        t.crouch = 9; t.lean = 0.42; t.sw = 2.6; t.reach = 0.9
        break
      case 'leap':
        t.air = 1; t.lean = 0.35; t.sw = -2.3; t.reach = 1
        Object.assign(t, { l1: 0.7, b1: 1.25, l2: -0.2, b2: 0.8 })
        break
      case 'waveTell':
      case 'flagTell':
      case 'blankTell':
        t.sw = lerp(0.8, -2.65, ease(progress * 1.3))
        t.reach = 1; t.lean = -0.18; t.crouch = 4.5
        t.ou = -1.2; t.of = -1.8
        break
      case 'blankStroke':
        t.sw = 0.95; t.reach = 1; t.lean = 0.55; t.crouch = 5
        anim.snap = true
        break
      case 'rushTell':
        t.crouch = 7; t.lean = 0.65; t.sw = 2.95; t.reach = 0.95
        break
      case 'rush':
        t.lean = 0.85; t.crouch = 4; t.sw = 3.05; t.reach = 1
        Object.assign(t, { l1: 1.05, b1: 0.55, l2: -1.0, b2: 0.15 })
        anim.snap = true
        break
      case 'backstep':
        t.air = 1; t.lean = -0.25; t.sw = 2.5
        Object.assign(t, { l1: 0.5, b1: 0.9, l2: -0.3, b2: 0.6 })
        break
      case 'roar':
        t.lean = -0.22; t.crouch = 3; t.sw = -1.6; t.reach = 0.95; t.ou = -1.4; t.of = -2.0
        break
      case 'recover':
        t.lean = 0.42; t.crouch = 4; t.sw = 1.15; t.reach = 0.85
        break
      case 'hurt':
        t.lean = -0.48; t.crouch = 3.5; t.sw = -2.1; t.head = -0.28
        break
      case 'stagger':
      case 'broken':
        t.lean = 0.46; t.crouch = 7; t.sw = 1.65; t.reach = 0.75; t.head = 0.35
        Object.assign(t, { l1: 0.42, b1: 0.95, l2: -0.28, b2: 0.75 })
        break
      case 'dying':
        // Stand tall and still as the douli splits into blank paper.
        t.lean = 0.03; t.crouch = 1; t.sw = 1.1; t.reach = 0.86; t.rot = 0
        break
    }
    return t
  }

  function overlay(ctx, kind, s, pose, look, time, entity) {
    if (look.flash) return
    const e = entity || {}
    ctx.save()

    // Ink-bleed hem aura for ink shades (distinguishes them from mortal fighters).
    if (kind === 'shade' || kind === 'shadeArcher') {
      ctx.fillStyle = 'rgba(20,21,24,0.22)'
      for (let i = 0; i < 6; i++) {
        const bx = s.hip[0] - 10 + i * 4
        const lift = Math.sin(time * 5.2 + i * 1.7 + (e.seed || 0)) * 2.5
        ctx.beginPath()
        ctx.ellipse(bx, -2 + lift * 0.5, 3.2, 4.5 + (i % 2) * 2, 0.1 * (i - 2.5), 0, TAU)
        ctx.fill()
      }
      // Subtle cinnabar eyes under brim/hood.
      const eyeGlow = look.eyeGlow || 0.25
      ctx.fillStyle = '#e2412c'
      ctx.globalAlpha = 0.7 + 0.3 * eyeGlow
      ctx.beginPath()
      ctx.ellipse(s.head[0] + 2.6, s.head[1] - 0.5, 1.5, 0.95, 0, 0, TAU)
      ctx.fill()
      ctx.globalAlpha = 1
    }

    if (kind === 'shade') {
      // Broken/notched douli rim cut.
      ctx.fillStyle = PAPER
      ctx.beginPath()
      ctx.moveTo(s.head[0] - 9, s.head[1] - 5.5)
      ctx.lineTo(s.head[0] - 5.5, s.head[1] - 8.2)
      ctx.lineTo(s.head[0] - 3.8, s.head[1] - 4.4)
      ctx.closePath()
      ctx.fill()
      // Crisp paper-white flash along blade right after 拆.
      if (e.state === 'riposteTele' && e.teleTotal && e.t > e.teleTotal - 0.16) {
        const q = clamp((e.t - (e.teleTotal - 0.16)) / 0.16, 0, 1)
        const ang = pose.sw + (pose.wrist || 0)
        ctx.strokeStyle = PAPER
        ctx.globalAlpha = 0.95 * q
        ctx.lineWidth = 2.6
        ctx.beginPath()
        ctx.moveTo(s.hand[0], s.hand[1])
        ctx.lineTo(s.hand[0] + Math.cos(ang) * 35, s.hand[1] + Math.sin(ang) * 35)
        ctx.stroke()
      }
    }

    if (kind === 'shadeArcher') {
      // Translucent ink veil hanging from the hood.
      const sway = Math.sin(time * 3.8 + (e.seed || 0)) * 2
      ctx.fillStyle = 'rgba(22,24,28,0.42)'
      ctx.beginPath()
      ctx.moveTo(s.head[0] + 4, s.head[1] - 3)
      ctx.quadraticCurveTo(s.head[0] + 6 + sway * 0.4, s.head[1] + 5, s.head[0] + 3 + sway, s.head[1] + 12)
      ctx.lineTo(s.head[0] - 2 + sway * 0.6, s.head[1] + 11)
      ctx.lineTo(s.head[0] - 1, s.head[1] - 2)
      ctx.closePath()
      ctx.fill()
    }

    if (kind === 'nameless') {
      // Paper-white waist sash with fine ink outline (mirror of protagonist's cinnabar sash).
      const wf = [s.hip[0] + s.fwd[0] * 5.6, s.hip[1] + s.fwd[1] * 5.6 - 2]
      const wb = [s.hip[0] - s.fwd[0] * 6.6, s.hip[1] - s.fwd[1] * 6.6 - 2]
      ctx.strokeStyle = INK; ctx.lineWidth = 3.2
      ctx.beginPath(); ctx.moveTo(wb[0], wb[1]); ctx.lineTo(wf[0], wf[1]); ctx.stroke()
      ctx.strokeStyle = PAPER; ctx.lineWidth = 1.9
      ctx.beginPath(); ctx.moveTo(wb[0], wb[1]); ctx.lineTo(wf[0], wf[1]); ctx.stroke()
      const sway = Math.sin(time * 9) * 1.5 - pose.flutter * 5
      ctx.strokeStyle = INK; ctx.lineWidth = 2.1
      ctx.beginPath(); ctx.moveTo(wb[0], wb[1]); ctx.quadraticCurveTo(wb[0] - 4, wb[1] + 5, wb[0] - 6 + sway, wb[1] + 10); ctx.stroke()
      ctx.strokeStyle = PAPER; ctx.lineWidth = 1.1
      ctx.beginPath(); ctx.moveTo(wb[0], wb[1]); ctx.quadraticCurveTo(wb[0] - 4, wb[1] + 5, wb[0] - 6 + sway, wb[1] + 10); ctx.stroke()

      // Paper-white counter flash on 拆.
      if (e.state === 'riposteTele' && e.teleTotal && e.t > e.teleTotal - 0.16) {
        const q = clamp((e.t - (e.teleTotal - 0.16)) / 0.16, 0, 1)
        const ang = pose.sw + (pose.wrist || 0)
        ctx.strokeStyle = PAPER
        ctx.globalAlpha = 0.95 * q
        ctx.lineWidth = 2.8
        ctx.beginPath()
        ctx.moveTo(s.hand[0], s.hand[1])
        ctx.lineTo(s.hand[0] + Math.cos(ang) * 36, s.hand[1] + Math.sin(ang) * 36)
        ctx.stroke()
      }

      // Finale: douli splits open revealing a blank white sheet of paper.
      if (e.state === 'dying') {
        const crack = clamp(1 - e.t / 3.2, 0, 1)
        const hx = s.head[0], hy = s.head[1]
        // Split paper glow inside the hat.
        ctx.fillStyle = PAPER
        ctx.strokeStyle = INK; ctx.lineWidth = 0.9
        const open = 2 + crack * 9
        ctx.beginPath()
        ctx.rect(hx - open * 0.7, hy - 10 - crack * 4, open * 1.4, 14 + crack * 4)
        ctx.fill(); ctx.stroke()
        // Falling split hat halves.
        ctx.fillStyle = INK
        ctx.beginPath()
        ctx.moveTo(hx - open * 0.8, hy - 11 + crack * 5)
        ctx.lineTo(hx - 15 - crack * 6, hy - 4 + crack * 9)
        ctx.lineTo(hx - open * 0.6, hy - 3 + crack * 6)
        ctx.closePath(); ctx.fill()
        ctx.beginPath()
        ctx.moveTo(hx + open * 0.8, hy - 11 + crack * 5)
        ctx.lineTo(hx + 15 + crack * 6, hy - 4 + crack * 9)
        ctx.lineTo(hx + open * 0.6, hy - 3 + crack * 6)
        ctx.closePath(); ctx.fill()
      }
    }

    ctx.restore()
  }

  globalThis.XiabanInkReturnFigures = { pose, overlay }
})()
