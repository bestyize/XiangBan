/* 水墨剑客 · synthesized sound: steel, ink, wind and a generative guqin-like pentatonic line. */
(() => {
  'use strict'

  const SCALE = [146.83, 164.81, 185, 220, 246.94, 293.66, 329.63, 369.99, 440, 493.88, 587.33]
  let ac = null, master = null, wet = null, noise = null, rain = null, rainGain = null, wind = null, windGain = null, windFilter = null
  let enabled = false, nextNote = 0, nextDrum = 0, nextChime = 0, lastNote = 5

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

  const SOUNDS = {
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
      lastNote = Math.max(0, Math.min(SCALE.length - 1, lastNote + Math.round((Math.random() - 0.5) * 4)))
      pluck(SCALE[lastNote], mood.combat ? 0.035 : 0.045)
      if (Math.random() < 0.3) pluck(SCALE[Math.max(0, lastNote - 3)] / 2, 0.03, 0.02)
      nextNote = now + (mood.combat ? 0.55 + Math.random() * 0.7 : 1.3 + Math.random() * 1.8)
    }
    if (mood.combat && now >= nextDrum) { drum(mood.boss ? 62 : 74, mood.boss ? 0.16 : 0.08); nextDrum = now + (mood.boss ? 0.62 : 1.1) }
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
    const gale = mood.gale || 0, snow = mood.snow || 0
    windGain.gain.setTargetAtTime(snow * 0.014 + gale * 0.06, now, 0.8)
    windFilter.frequency.setTargetAtTime(380 + gale * 520 + Math.sin(now * 0.7) * 120 * (0.4 + gale), now, 0.4)
    if (snow > 0.1 && now >= nextChime) {
      bell(SCALE[2 + Math.floor(Math.random() * 6)] * 2, 0.012 + 0.01 * snow, 2.2)
      nextChime = now + 5 + Math.random() * 8
    }
  }

  function setEnabled(value) {
    enabled = value
    if (value && init() && ac.state === 'suspended') void ac.resume().catch(() => {})
    if (!value && rainGain) rainGain.gain.setTargetAtTime(0, ac.currentTime, 0.1)
    if (!value && windGain) windGain.gain.setTargetAtTime(0, ac.currentTime, 0.1)
  }

  function resume() { if (enabled && ac?.state === 'suspended') void ac.resume().catch(() => {}) }
  function close() { if (ac) void ac.close().catch(() => {}) }

  globalThis.XiabanInkAudio = { play, update, setEnabled, resume, close, get enabled() { return enabled } }
})()
