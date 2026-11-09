/* 水墨剑客 · synthesized sound: steel, ink, wind and a generative guqin-like pentatonic line. */
(() => {
  'use strict'

  const SCALE = [146.83, 164.81, 185, 220, 246.94, 293.66, 329.63, 369.99, 440, 493.88, 587.33]
  let ac = null, master = null, wet = null, noise = null, rain = null, rainGain = null, wind = null, windGain = null, windFilter = null
  // Volume 3 (烽火卷): desolate frontier mode, 羽调-flavoured minor pentatonic A C D E G over two octaves.
  const FRONTIER = [110, 130.81, 146.83, 164.81, 196, 220, 261.63, 293.66, 329.63, 392, 440]
  let fire = null, fireGain = null, sandSrc = null, sandGain = null, sandFilter = null
  let nextCrackle = 0, nextDrone = 0
  let enabled = true, nextNote = 0, nextDrum = 0, nextChime = 0, nextSurf = 0, lastNote = 5

  function init() {
    if (ac) return true
    const Audio = window.AudioContext || window.webkitAudioContext
    if (!Audio) return false
    try {
      ac = new Audio()
      const compressor = ac.createDynamicsCompressor()
      compressor.threshold.value = -16; compressor.ratio.value = 4
      master = ac.createGain(); master.gain.value = 0.75
      master.connect(compressor); compressor.connect(ac.destination)
      const reverb = ac.createConvolver()
      const length = Math.floor(ac.sampleRate * 2.4)
      const impulse = ac.createBuffer(2, length, ac.sampleRate)
      for (let c = 0; c < 2; c++) {
        const data = impulse.getChannelData(c)
        for (let i = 0; i < length; i++) data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / length, 2.6)
      }
      reverb.buffer = impulse
      wet = ac.createGain(); wet.gain.value = 0.42
      wet.connect(reverb); reverb.connect(master)
      noise = ac.createBuffer(1, ac.sampleRate, ac.sampleRate)
      const data = noise.getChannelData(0)
      for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1
      return true
    } catch { ac = null; return false }
  }

  function out(node, send = 0) {
    node.connect(master)
    if (send) { const g = ac.createGain(); g.gain.value = send; node.connect(g); g.connect(wet) }
  }

  function envelope(param, at, peak, attack, decay) {
    param.setValueAtTime(0.0001, at)
    param.exponentialRampToValueAtTime(Math.max(0.0002, peak), at + attack)
    param.exponentialRampToValueAtTime(0.0001, at + attack + decay)
  }

  function tone(freq, { type = 'sine', dur = 0.2, vol = 0.1, delay = 0, bend = 1, send = 0.2, attack = 0.004 } = {}) {
    const at = ac.currentTime + delay
    const osc = ac.createOscillator(), gain = ac.createGain()
    osc.type = type
    osc.frequency.setValueAtTime(freq, at)
    if (bend !== 1) osc.frequency.exponentialRampToValueAtTime(freq * bend, at + dur)
    envelope(gain.gain, at, vol, attack, dur)
    osc.connect(gain); out(gain, send)
    osc.start(at); osc.stop(at + attack + dur + 0.05)
  }

  function hiss({ dur = 0.15, vol = 0.1, freq = 1200, to = freq, q = 1, type = 'bandpass', delay = 0, send = 0.1 } = {}) {
    const at = ac.currentTime + delay
    const src = ac.createBufferSource(); src.buffer = noise
    const filter = ac.createBiquadFilter(); filter.type = type; filter.Q.value = q
    filter.frequency.setValueAtTime(freq, at)
    filter.frequency.exponentialRampToValueAtTime(Math.max(40, to), at + dur)
    const gain = ac.createGain()
    envelope(gain.gain, at, vol, 0.006, dur)
    src.connect(filter); filter.connect(gain); out(gain, send)
    src.start(at, Math.random() * 0.5); src.stop(at + dur + 0.05)
  }

  function bell(freq, vol = 0.07, dur = 0.9, delay = 0) {
    for (const [ratio, v] of [[1, 1], [2.76, 0.5], [5.4, 0.28], [8.93, 0.14]]) tone(freq * ratio, { dur: dur / Math.sqrt(ratio), vol: vol * v, delay, send: 0.5, attack: 0.002 })
  }

  function drum(freq = 80, vol = 0.3, delay = 0) {
    tone(freq, { dur: 0.45, vol, bend: 0.45, delay, send: 0.25, attack: 0.003 })
    hiss({ dur: 0.08, vol: vol * 0.35, freq: 400, to: 120, type: 'lowpass', delay })
  }

  function pluck(freq, vol = 0.05, delay = 0) {
    const at = ac.currentTime + delay
    const osc = ac.createOscillator(), osc2 = ac.createOscillator(), gain = ac.createGain(), filter = ac.createBiquadFilter()
    osc.type = 'triangle'; osc2.type = 'sine'
    osc.frequency.setValueAtTime(freq * 1.012, at); osc.frequency.exponentialRampToValueAtTime(freq, at + 0.12)
    osc.frequency.setValueAtTime(freq, at + 0.5); osc.frequency.linearRampToValueAtTime(freq * 0.994, at + 1.2)
    osc2.frequency.setValueAtTime(freq * 2, at)
    filter.type = 'lowpass'; filter.frequency.setValueAtTime(2600, at); filter.frequency.exponentialRampToValueAtTime(500, at + 1.6)
    envelope(gain.gain, at, vol, 0.004, 2.4)
    const g2 = ac.createGain(); g2.gain.value = 0.25
    osc.connect(filter); osc2.connect(g2); g2.connect(filter); filter.connect(gain); out(gain, 0.55)
    osc.start(at); osc2.start(at); osc.stop(at + 2.6); osc2.stop(at + 2.6)
  }

  // One gallop beat cluster: three quick low thuds with slight randomness.
  function hoofCluster(delay = 0, vol = 0.1) {
    for (let i = 0; i < 3; i++) {
      const d = delay + i * (0.045 + Math.random() * 0.012), v = vol * (i === 0 ? 1 : 0.7) * (0.85 + Math.random() * 0.3)
      tone(95 + Math.random() * 30, { dur: 0.09, vol: v, bend: 0.5, delay: d, send: 0.08, attack: 0.002 })
      hiss({ dur: 0.05, vol: v * 0.45, freq: 900 + Math.random() * 400, to: 250, type: 'lowpass', delay: d, send: 0.05 })
    }
  }

  // Horse whinny: vibrato sawtooth/square pair, pitch fall, through a bandpass.
  function neighAt(delay = 0, vol = 0.05, pitch = 1, send = 0.3) {
    const at = ac.currentTime + delay, dur = 0.65
    const filter = ac.createBiquadFilter(); filter.type = 'bandpass'; filter.Q.value = 2.2
    filter.frequency.setValueAtTime(1100 * pitch, at); filter.frequency.exponentialRampToValueAtTime(700 * pitch, at + dur)
    const gain = ac.createGain(); envelope(gain.gain, at, vol, 0.05, dur)
    const lfo = ac.createOscillator(), lfoGain = ac.createGain()
    lfo.frequency.value = 22; lfoGain.gain.value = 18 * pitch
    lfo.connect(lfoGain)
    for (const type of ['sawtooth', 'square']) {
      const osc = ac.createOscillator(); osc.type = type
      osc.frequency.setValueAtTime(560 * pitch, at)
      osc.frequency.exponentialRampToValueAtTime(900 * pitch, at + 0.12)
      osc.frequency.exponentialRampToValueAtTime(420 * pitch, at + dur)
      lfoGain.connect(osc.frequency)
      osc.connect(filter); osc.start(at); osc.stop(at + dur + 0.1)
    }
    filter.connect(gain); out(gain, send)
    lfo.start(at); lfo.stop(at + dur + 0.1)
  }

  const SOUNDS = {
    stonestep: () => {tone(140,{dur:.045,vol:.03,bend:.5});hiss({dur:.045,vol:.025,freq:450,to:120,type:'lowpass'})},
    mirror: () => { bell(784,0.025,1.1);bell(1174,0.018,0.8,0.06) },
    glass: () => { for(let i=0;i<6;i++)bell(1200+i*310,0.018,.35,i*.018);hiss({dur:.16,vol:.07,freq:4600,to:1800}) },
    paper: () => hiss({dur:.2,vol:.09,freq:2700,to:700,q:.6}),
    brush: () => { hiss({dur:.3,vol:.12,freq:1000,to:130});tone(98,{dur:.15,vol:.06,bend:.6}) },
    chain: () => { for(let i=0;i<5;i++) bell(960+i*210,0.024,0.12,i*0.035); hiss({dur:0.2,vol:0.07,freq:2700,to:900}) },
    rigging: () => { tone(180,{type:'triangle',dur:0.22,vol:0.06,bend:0.4});hiss({dur:0.14,vol:0.09,freq:1400,to:300}) },
    slash: () => { hiss({ dur: 0.14, vol: 0.12, freq: 2600, to: 700, q: 0.9 }); tone(220, { dur: 0.08, vol: 0.02, bend: 0.6 }) },
    heavy: () => { hiss({ dur: 0.2, vol: 0.16, freq: 1800, to: 380, q: 0.8 }); tone(130, { dur: 0.14, vol: 0.05, bend: 0.5 }) },
    hit: () => { hiss({ dur: 0.09, vol: 0.16, freq: 900, to: 300, type: 'lowpass' }); tone(150, { dur: 0.12, vol: 0.12, bend: 0.4 }) },
    parry: () => { bell(1250, 0.09, 1.1); hiss({ dur: 0.05, vol: 0.12, freq: 5000, to: 3000, type: 'highpass' }) },
    block: () => { bell(520, 0.05, 0.3); hiss({ dur: 0.05, vol: 0.08, freq: 1500, to: 900 }) },
    execute: () => { drum(70, 0.4); bell(880, 0.07, 1.4, 0.02); hiss({ dur: 0.35, vol: 0.12, freq: 3000, to: 200, q: 0.6 }) },
    hurt: () => { tone(110, { dur: 0.25, vol: 0.18, bend: 0.5, type: 'triangle' }); hiss({ dur: 0.12, vol: 0.12, freq: 600, to: 150, type: 'lowpass' }) },
    jump: () => hiss({ dur: 0.1, vol: 0.05, freq: 900, to: 2200, q: 0.7 }),
    land: () => hiss({ dur: 0.07, vol: 0.06, freq: 400, to: 120, type: 'lowpass' }),
    dash: () => hiss({ dur: 0.18, vol: 0.1, freq: 600, to: 3200, q: 0.6 }),
    tell: () => bell(2350, 0.035, 0.35),
    danger: () => { bell(330, 0.08, 1.4); tone(165, { dur: 0.8, vol: 0.05, send: 0.5 }) },
    caw: () => { tone(760, { type: 'sawtooth', dur: 0.16, vol: 0.025, bend: 0.7, send: 0.3 }); tone(740, { type: 'sawtooth', dur: 0.12, vol: 0.02, bend: 0.75, delay: 0.17, send: 0.3 }) },
    shoot: () => { tone(260, { type: 'triangle', dur: 0.12, vol: 0.06, bend: 0.7 }); hiss({ dur: 0.1, vol: 0.05, freq: 3000, to: 1500 }) },
    kill: () => { hiss({ dur: 0.4, vol: 0.12, freq: 1200, to: 150, type: 'lowpass', send: 0.3 }); tone(98, { dur: 0.3, vol: 0.1, bend: 0.6 }) },
    spawn: () => hiss({ dur: 0.5, vol: 0.05, freq: 300, to: 1400, q: 2, send: 0.4 }),
    strike: () => { hiss({ dur: 0.55, vol: 0.18, freq: 200, to: 4000, q: 0.5, send: 0.4 }); drum(55, 0.45, 0.55); bell(587, 0.08, 2, 0.55); bell(880, 0.05, 2, 0.6) },
    slam: () => { drum(48, 0.5); hiss({ dur: 0.3, vol: 0.14, freq: 500, to: 80, type: 'lowpass' }) },
    wave: () => hiss({ dur: 0.6, vol: 0.1, freq: 500, to: 2000, q: 0.6, send: 0.3 }),
    roar: () => { drum(60, 0.4); tone(98, { type: 'sawtooth', dur: 1, vol: 0.04, bend: 0.8, send: 0.6 }) },
    gong: () => { bell(196, 0.12, 2.6); drum(55, 0.35) },
    victory: () => { [293.66, 369.99, 440, 587.33].forEach((f, i) => pluck(f, 0.08, i * 0.16)); bell(1174, 0.05, 2.4, 0.7) },
    defeat: () => { [220, 185, 146.83].forEach((f, i) => pluck(f, 0.07, i * 0.3)) },
    snowstep: () => {
      hiss({ dur: 0.06, vol: 0.05, freq: 3600, to: 2200, q: 0.9, type: 'bandpass', send: 0.05 })
      hiss({ dur: 0.1, vol: 0.035, freq: 700, to: 260, type: 'lowpass', delay: 0.025, send: 0.05 })
    },
    iceslide: () => { hiss({ dur: 0.22, vol: 0.045, freq: 5200, to: 6800, q: 4, send: 0.2 }); tone(1900, { dur: 0.12, vol: 0.006, bend: 1.15, send: 0.3 }) },
    icecrack: () => {
      hiss({ dur: 0.07, vol: 0.12, freq: 6500, to: 3000, type: 'highpass', send: 0.3 })
      tone(2400, { type: 'square', dur: 0.09, vol: 0.018, bend: 0.3, send: 0.4 })
      tone(1500, { type: 'triangle', dur: 0.18, vol: 0.02, bend: 0.5, delay: 0.05, send: 0.5 })
      bell(3100, 0.03, 0.6, 0.04)
    },
    icicrack: () => { hiss({ dur: 0.06, vol: 0.1, freq: 7000, to: 3500, type: 'highpass', send: 0.3 }); tone(2900, { type: 'square', dur: 0.05, vol: 0.012, bend: 0.4, send: 0.3 }) },
    iceshatter: () => { bell(3000, 0.03, 0.4); bell(3900, 0.022, 0.35, 0.05); hiss({ dur: 0.22, vol: 0.06, freq: 6500, to: 2500, type: 'highpass', send: 0.3 }) },
    creak: () => {
      tone(92, { type: 'sawtooth', dur: 0.7, vol: 0.028, bend: 1.5, send: 0.4 })
      hiss({ dur: 0.7, vol: 0.04, freq: 300, to: 900, q: 3, send: 0.3 })
      for (const d of [0.18, 0.34, 0.5]) hiss({ dur: 0.04, vol: 0.05, freq: 5200, to: 3000, type: 'highpass', delay: d, send: 0.2 })
    },
    hide: () => hiss({ dur: 0.28, vol: 0.035, freq: 2000, to: 900, q: 0.8 }),
    ambush: () => { hiss({ dur: 0.14, vol: 0.14, freq: 3000, to: 600, q: 0.9 }); drum(110, 0.18); bell(1500, 0.05, 0.7, 0.04) },
    gust: () => hiss({ dur: 1.7, vol: 0.085, freq: 300, to: 1400, q: 0.5, send: 0.5 }),
    ui: () => pluck(440, 0.06),
    ink: () => bell(1760, 0.05, 0.8),
    hoof: () => hoofCluster(0, 0.1),
    charge: () => {
      let t = 0, gap = 0.2
      for (let i = 0; i < 6; i++) { hoofCluster(t, 0.07 + i * 0.012); t += gap; gap *= 0.82 }
      hiss({ dur: 0.9, vol: 0.07, freq: 300, to: 1600, q: 0.8, send: 0.2 })
    },
    neigh: () => neighAt(0, 0.05, 1, 0.3),
    unhorse: () => {
      tone(80, { dur: 0.3, vol: 0.18, bend: 0.4, send: 0.15 })
      hiss({ dur: 0.18, vol: 0.14, freq: 700, to: 150, type: 'lowpass' })
      for (let i = 0; i < 6; i++) hiss({ dur: 0.04 + Math.random() * 0.03, vol: 0.05 + Math.random() * 0.03, freq: 3500 + Math.random() * 3000, to: 2500, q: 3, delay: 0.04 + i * 0.045 + Math.random() * 0.03, send: 0.2 })
      neighAt(0.3, 0.025, 0.9, 0.7)
    },
    volleyTell: () => {
      for (let i = 0; i < 14; i++) {
        const d = Math.random() * 0.8, f = 180 + Math.random() * 200
        tone(f, { type: 'triangle', dur: 0.1, vol: 0.016, bend: 0.6, delay: d, send: 0.3 })
        hiss({ dur: 0.04, vol: 0.02, freq: 2500 + Math.random() * 1500, to: 1500, delay: d, send: 0.2 })
      }
      hiss({ dur: 1, vol: 0.07, freq: 900, to: 5200, q: 3, delay: 0.1, send: 0.4 })
    },
    volleyHit: () => {
      for (let i = 0; i < 18; i++) {
        const d = Math.random() * 0.4, wood = Math.random() < 0.4
        hiss({ dur: 0.04, vol: 0.05 + Math.random() * 0.04, freq: wood ? 1600 : 500, to: wood ? 900 : 200, q: wood ? 2 : 0.8, type: wood ? 'bandpass' : 'lowpass', delay: d, send: 0.1 })
        if (i % 3 === 0) tone(120 + Math.random() * 60, { dur: 0.06, vol: 0.04, bend: 0.5, delay: d, send: 0.1 })
      }
    },
    potThrow: () => {
      hiss({ dur: 0.3, vol: 0.07, freq: 500, to: 1800, q: 0.7 })
      hiss({ dur: 0.3, vol: 0.035, freq: 6500, to: 4500, type: 'highpass', send: 0.15 })
    },
    potBreak: () => {
      hiss({ dur: 0.08, vol: 0.14, freq: 4500, to: 2500, type: 'highpass', send: 0.3 })
      for (let i = 0; i < 5; i++) tone(1800 + Math.random() * 2200, { type: 'square', dur: 0.04, vol: 0.012, bend: 0.5, delay: 0.01 + i * 0.03, send: 0.2 })
      hiss({ dur: 0.1, vol: 0.1, freq: 900, to: 300, delay: 0.03 })
      hiss({ dur: 0.4, vol: 0.09, freq: 200, to: 900, q: 0.6, delay: 0.1, send: 0.3 })
      tone(70, { dur: 0.3, vol: 0.1, bend: 0.6, delay: 0.1, send: 0.2 })
    },
    ignite: () => {
      hiss({ dur: 0.7, vol: 0.09, freq: 300, to: 2400, q: 0.7, send: 0.3 })
      for (let i = 0; i < 10; i++) hiss({ dur: 0.02, vol: 0.04 + Math.random() * 0.03, freq: 3000 + Math.random() * 3500, to: 2000, q: 2, delay: 0.1 + Math.random() * 0.6, send: 0.1 })
    },
    burn: () => {
      hiss({ dur: 0.3, vol: 0.07, freq: 6000, to: 3500, type: 'highpass', send: 0.1 })
      tone(70, { dur: 0.18, vol: 0.1, bend: 0.5, send: 0.1 })
    },
    horn: () => {
      const at = ac.currentTime, dur = 1.6
      const filter = ac.createBiquadFilter(); filter.type = 'lowpass'; filter.Q.value = 2
      filter.frequency.setValueAtTime(350, at); filter.frequency.exponentialRampToValueAtTime(1100, at + 0.7)
      filter.frequency.exponentialRampToValueAtTime(500, at + dur)
      const gain = ac.createGain()
      gain.gain.setValueAtTime(0.0001, at)
      gain.gain.exponentialRampToValueAtTime(0.09, at + 0.5)
      gain.gain.setValueAtTime(0.09, at + 1.1)
      gain.gain.exponentialRampToValueAtTime(0.0001, at + dur)
      for (const detune of [-9, 0, 8]) {
        const osc = ac.createOscillator(); osc.type = 'sawtooth'
        osc.frequency.setValueAtTime(82 * 0.9, at); osc.frequency.exponentialRampToValueAtTime(82, at + 0.4)
        osc.detune.value = detune
        osc.connect(filter); osc.start(at); osc.stop(at + dur + 0.05)
      }
      filter.connect(gain); out(gain, 0.6)
      hiss({ dur: 1.2, vol: 0.012, freq: 500, to: 900, q: 2, delay: 0.1, send: 0.4 })
    },
    formationBreak: () => {
      for (let i = 0; i < 7; i++) {
        const d = i * 0.05 + Math.random() * 0.03
        hiss({ dur: 0.07, vol: 0.09, freq: 700 + Math.random() * 700, to: 300, q: 3, delay: d, send: 0.15 })
        tone(150 + Math.random() * 100, { type: 'triangle', dur: 0.08, vol: 0.06, bend: 0.6, delay: d, send: 0.15 })
      }
      bell(660, 0.04, 0.8, 0.12)
    },
    flag: () => {
      for (const [d, v] of [[0, 0.07], [0.09, 0.05], [0.17, 0.06], [0.3, 0.03]]) hiss({ dur: 0.07, vol: v, freq: 1200, to: 400, q: 0.8, delay: d, send: 0.1 })
    },
    sandstep: () => {
      hiss({ dur: 0.07, vol: 0.03, freq: 2800, to: 1800, q: 0.7, send: 0.03 })
      hiss({ dur: 0.09, vol: 0.02, freq: 600, to: 250, type: 'lowpass', delay: 0.02, send: 0.03 })
    },
    dust: () => {
      const at = ac.currentTime
      const src = ac.createBufferSource(); src.buffer = noise
      const filter = ac.createBiquadFilter(); filter.type = 'bandpass'; filter.Q.value = 0.6
      filter.frequency.setValueAtTime(700, at); filter.frequency.exponentialRampToValueAtTime(2200, at + 0.6); filter.frequency.exponentialRampToValueAtTime(900, at + 1.2)
      const gain = ac.createGain()
      gain.gain.setValueAtTime(0.0001, at)
      gain.gain.exponentialRampToValueAtTime(0.07, at + 0.5)
      gain.gain.exponentialRampToValueAtTime(0.0001, at + 1.2)
      src.connect(filter); filter.connect(gain); out(gain, 0.2)
      src.start(at, Math.random() * 0.5); src.stop(at + 1.25)
    },
  }

  function play(name) {
    if (!enabled || !ac || ac.state !== 'running' || !SOUNDS[name]) return
    try { SOUNDS[name]() } catch { /* Audio failures never interrupt play. */ }
  }

  // Ambient bed: sparse plucks, faster with drums in combat, rain noise in the storm.
  function update(mood) {
    if (!enabled || !ac || ac.state !== 'running') return
    const now = ac.currentTime
    if (now >= nextNote) {
      const scale = mood.volume === 3 ? FRONTIER : SCALE
      lastNote = Math.max(0, Math.min(scale.length - 1, lastNote + Math.round((Math.random() - 0.5) * 4)))
      pluck(scale[lastNote], mood.combat ? 0.035 : 0.045)
      if (Math.random() < 0.3) pluck(scale[Math.max(0, lastNote - 3)] / 2, 0.03, 0.02)
      if (mood.volume === 3 && now >= nextDrone) {
        pluck(Math.random() < 0.5 ? 55 : 82.41, 0.04, 0.05)
        nextDrone = now + 6 + Math.random() * 6
      }
      nextNote = now + (mood.combat ? 0.55 + Math.random() * 0.7 : 1.3 + Math.random() * 1.8)
    }
    if (mood.combat && mood.war && now >= nextDrum) {
      drum(mood.boss ? 56 : 64, mood.boss ? 0.17 : 0.1)
      drum(mood.boss ? 56 : 64, mood.boss ? 0.1 : 0.06, 0.2)
      if (mood.boss) drum(70, 0.08, 0.36)
      nextDrum = now + (mood.boss ? 0.6 : 0.95)
    } else if (mood.combat && now >= nextDrum) { drum(mood.boss ? 62 : 74, mood.boss ? 0.16 : 0.08); nextDrum = now + (mood.boss ? 0.62 : 1.1) }
    if (!rain) {
      rain = ac.createBufferSource(); rain.buffer = noise; rain.loop = true
      const filter = ac.createBiquadFilter(); filter.type = 'bandpass'; filter.frequency.value = 2400; filter.Q.value = 0.5
      rainGain = ac.createGain(); rainGain.gain.value = 0
      rain.connect(filter); filter.connect(rainGain); rainGain.connect(master); rain.start()
    }
    rainGain.gain.setTargetAtTime(mood.rain * 0.05, now, 0.6)
    if (!wind) {
      wind = ac.createBufferSource(); wind.buffer = noise; wind.loop = true
      windFilter = ac.createBiquadFilter(); windFilter.type = 'bandpass'; windFilter.frequency.value = 420; windFilter.Q.value = 0.7
      windGain = ac.createGain(); windGain.gain.value = 0
      wind.connect(windFilter); windFilter.connect(windGain); windGain.connect(master); wind.start()
    }
    const gale = mood.gale || 0, snow = mood.snow || 0, sea = mood.sea || 0
    windGain.gain.setTargetAtTime(snow * 0.014 + gale * 0.06 + sea * 0.025, now, 0.8)
    windFilter.frequency.setTargetAtTime(380 + gale * 520 + sea * 130 + Math.sin(now * 0.7) * 120 * (0.4 + gale + sea), now, 0.4)
    if (sea > 0 && now >= nextSurf) {
      hiss({ dur: 1.3, vol: 0.018 + sea * 0.02, freq: 280, to: 1100, q: 0.5, send: 0.25 })
      nextSurf = now + 3.2 + Math.random() * 1.7
    }
    if (snow > 0.1 && now >= nextChime) {
      bell(SCALE[2 + Math.floor(Math.random() * 6)] * 2, 0.012 + 0.01 * snow, 2.2)
      nextChime = now + 5 + Math.random() * 8
    }
    const fireLevel = Math.max(0, Math.min(1, mood.fire || 0)), sandLevel = Math.max(0, Math.min(1, mood.sand || 0))
    if (fireLevel > 0 && !fire) {
      fire = ac.createBufferSource(); fire.buffer = noise; fire.loop = true
      const filter = ac.createBiquadFilter(); filter.type = 'bandpass'; filter.frequency.value = 1500; filter.Q.value = 0.4
      fireGain = ac.createGain(); fireGain.gain.value = 0
      fire.connect(filter); filter.connect(fireGain); fireGain.connect(master); fire.start()
    }
    if (fire) fireGain.gain.setTargetAtTime(fireLevel * 0.03, now, 0.5)
    if (fireLevel > 0.02 && now >= nextCrackle) {
      hiss({ dur: 0.02 + Math.random() * 0.03, vol: 0.015 + 0.05 * fireLevel * Math.random(), freq: 2500 + Math.random() * 4000, to: 1500, q: 1.5, send: 0.05 })
      nextCrackle = now + (0.38 - 0.32 * fireLevel) * (0.3 + Math.random() * 1.4)
    }
    if (sandLevel > 0 && !sandSrc) {
      sandSrc = ac.createBufferSource(); sandSrc.buffer = noise; sandSrc.loop = true
      sandFilter = ac.createBiquadFilter(); sandFilter.type = 'bandpass'; sandFilter.frequency.value = 600; sandFilter.Q.value = 0.8
      sandGain = ac.createGain(); sandGain.gain.value = 0
      sandSrc.connect(sandFilter); sandFilter.connect(sandGain); sandGain.connect(master); sandSrc.start()
    }
    if (sandSrc) {
      sandGain.gain.setTargetAtTime(sandLevel * 0.045, now, 0.8)
      sandFilter.frequency.setTargetAtTime(500 + sandLevel * 600 + Math.sin(now * 0.9) * 180 * sandLevel, now, 0.4)
    }
  }

  function setEnabled(value) {
    enabled = value
    if (value) resume()
    if (!value && rainGain) rainGain.gain.setTargetAtTime(0, ac.currentTime, 0.1)
    if (!value && windGain) windGain.gain.setTargetAtTime(0, ac.currentTime, 0.1)
    if (!value && fireGain) fireGain.gain.setTargetAtTime(0, ac.currentTime, 0.1)
    if (!value && sandGain) sandGain.gain.setTargetAtTime(0, ac.currentTime, 0.1)
  }

  function resume() { if (enabled && init() && ac.state === 'suspended') void ac.resume().catch(() => {}) }
  function close() { if (ac) void ac.close().catch(() => {}) }

  globalThis.XiabanInkAudio = { play, update, setEnabled, resume, close, get enabled() { return enabled } }
})()
