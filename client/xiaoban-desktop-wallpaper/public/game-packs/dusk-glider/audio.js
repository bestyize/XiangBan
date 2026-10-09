/* 暮色滑翔 · synthesized sound: soft wind, a kalimba music bed, a warm pad and gentle chimes. No audio files. */
(() => {
  'use strict'

  const PENTATONIC = [0, 2, 4, 7, 9]
  const ROOT = 261.63
  const MASTER = 0.7
  const EMPTY = {}
  // Slow chord cycle (Hz), three voices each: warm, open, never tense.
  const CHORDS = [
    [130.81, 196, 329.63],
    [110, 164.81, 261.63],
    [174.61, 261.63, 329.63],
    [196, 293.66, 392],
  ]
  const PAD_TYPES = ['triangle', 'sine', 'triangle']

  let ac = null, master = null, musicBus = null, wet = null, noise = null, suspendTimer = 0
  let windFilter = null, windGain = null, rumbleGain = null, rumbleFilter = null, seaGain = null
  let pads = null, padFilter = null, padGain = null
  let enabled = false, lastParams = 0, nextNote = 0, nextSpark = 0, nextChord = 0, chordIndex = 0, melody = 2

  const clamp = (v, lo, hi) => (v < lo ? lo : v > hi ? hi : v)
  // Major pentatonic step (can be negative) above a root frequency.
  const pent = (root, step) => root * Math.pow(2, (PENTATONIC[((step % 5) + 5) % 5] + Math.floor(step / 5) * 12) / 12)

  function loopNoise(filterType, freq, q) {
    const src = ac.createBufferSource(); src.buffer = noise; src.loop = true
    const filter = ac.createBiquadFilter(); filter.type = filterType; filter.frequency.value = freq; filter.Q.value = q
    src.connect(filter)
    src.start(0, Math.random() * 1.5)
    return filter
  }

  function buildReverb() {
    const length = Math.floor(ac.sampleRate * 2.8)
    const impulse = ac.createBuffer(2, length, ac.sampleRate)
    for (let c = 0; c < 2; c++) {
      const data = impulse.getChannelData(c)
      let lp = 0
      for (let i = 0; i < length; i++) {
        lp += (Math.random() * 2 - 1 - lp) * 0.4
        data[i] = lp * Math.pow(1 - i / length, 2.4)
      }
    }
    return impulse
  }

  function buildLayers() {
    // Wind: band-passed noise, opened up by speed.
    windFilter = loopNoise('bandpass', 300, 0.9)
    windGain = ac.createGain(); windGain.gain.value = 0
    windFilter.connect(windGain); windGain.connect(master)

    // Ground roll: low noise with a slow wobble.
    rumbleFilter = loopNoise('lowpass', 160, 0.7)
    const rollMod = ac.createGain(); rollMod.gain.value = 0.6
    rumbleGain = ac.createGain(); rumbleGain.gain.value = 0
    rumbleFilter.connect(rollMod); rollMod.connect(rumbleGain); rumbleGain.connect(master)
    const roll = ac.createOscillator(), rollDepth = ac.createGain()
    roll.frequency.value = 5.5; rollDepth.gain.value = 0.35
    roll.connect(rollDepth); rollDepth.connect(rollMod.gain); roll.start()

    // Sea wash: low noise swelling in and out.
    const seaFilter = loopNoise('lowpass', 700, 0.5)
    const seaMod = ac.createGain(); seaMod.gain.value = 0.65
    seaGain = ac.createGain(); seaGain.gain.value = 0
    seaFilter.connect(seaMod); seaMod.connect(seaGain); seaGain.connect(master)
    const swell = ac.createOscillator(), swellAmp = ac.createGain(), swellFreq = ac.createGain()
    swell.frequency.value = 0.11; swellAmp.gain.value = 0.35; swellFreq.gain.value = 320
    swell.connect(swellAmp); swellAmp.connect(seaMod.gain)
    swell.connect(swellFreq); swellFreq.connect(seaFilter.frequency); swell.start()

    // Pad: three voices, each doubled with a slight detune.
    padFilter = ac.createBiquadFilter(); padFilter.type = 'lowpass'; padFilter.frequency.value = 1500; padFilter.Q.value = 0.3
    padGain = ac.createGain(); padGain.gain.value = 0
    padFilter.connect(padGain); padGain.connect(musicBus)
    pads = []
    for (let i = 0; i < 6; i++) {
      const osc = ac.createOscillator()
      osc.type = PAD_TYPES[i % 3]; osc.detune.value = i < 3 ? -5 : 6
      osc.frequency.value = CHORDS[0][i % 3]
      osc.connect(padFilter); osc.start(); pads.push(osc)
    }
  }

  function teardown() {
    if (suspendTimer && typeof clearTimeout === 'function') clearTimeout(suspendTimer)
    suspendTimer = 0
    ac = master = musicBus = wet = noise = null
    windFilter = windGain = rumbleGain = rumbleFilter = seaGain = null
    pads = padFilter = padGain = null
  }

  function init() {
    if (ac) return true
    const Ctor = globalThis.AudioContext || globalThis.webkitAudioContext
    if (!Ctor) return false
    try {
      ac = new Ctor()
      const limiter = ac.createDynamicsCompressor()
      limiter.threshold.value = -12; limiter.knee.value = 10; limiter.ratio.value = 10
      limiter.attack.value = 0.004; limiter.release.value = 0.25
      master = ac.createGain(); master.gain.value = 0
      master.connect(limiter); limiter.connect(ac.destination)
      const reverb = ac.createConvolver(); reverb.buffer = buildReverb()
      wet = ac.createGain(); wet.gain.value = 0.5
      wet.connect(reverb); reverb.connect(master)
      musicBus = ac.createGain(); musicBus.gain.value = 0.8
      const musicSend = ac.createGain(); musicSend.gain.value = 0.55
      musicBus.connect(master); musicBus.connect(musicSend); musicSend.connect(wet)
      noise = ac.createBuffer(1, Math.floor(ac.sampleRate * 2), ac.sampleRate)
      const data = noise.getChannelData(0)
      for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1
      buildLayers()
      return true
    } catch { teardown(); return false }
  }

  function out(node, send = 0, bus = master) {
    node.connect(bus)
    if (send) { const g = ac.createGain(); g.gain.value = send; node.connect(g); g.connect(wet) }
  }

  function envelope(param, at, peak, attack, decay) {
    param.setValueAtTime(0.0001, at)
    param.exponentialRampToValueAtTime(Math.max(0.0002, peak), at + attack)
    param.exponentialRampToValueAtTime(0.0001, at + attack + decay)
  }

  function tone(freq, { type = 'sine', dur = 0.2, vol = 0.1, delay = 0, bend = 1, send = 0.3, attack = 0.004, bus = master } = {}) {
    const at = ac.currentTime + delay
    const osc = ac.createOscillator(), gain = ac.createGain()
    osc.type = type
    osc.frequency.setValueAtTime(freq, at)
    if (bend !== 1) osc.frequency.exponentialRampToValueAtTime(Math.max(20, freq * bend), at + attack + dur)
    envelope(gain.gain, at, vol, attack, dur)
    osc.connect(gain); out(gain, send, bus)
    osc.start(at); osc.stop(at + attack + dur + 0.05)
  }

  function hiss({ dur = 0.15, vol = 0.1, freq = 1200, to = freq, q = 1, type = 'bandpass', delay = 0, send = 0.15, attack = 0.006 } = {}) {
    const at = ac.currentTime + delay
    const src = ac.createBufferSource(); src.buffer = noise
    const filter = ac.createBiquadFilter(); filter.type = type; filter.Q.value = q
    filter.frequency.setValueAtTime(freq, at)
    filter.frequency.exponentialRampToValueAtTime(Math.max(40, to), at + attack + dur)
    const gain = ac.createGain()
    envelope(gain.gain, at, vol, attack, dur)
    src.connect(filter); filter.connect(gain); out(gain, send)
    src.start(at, Math.random() * 1.2); src.stop(at + attack + dur + 0.05)
  }

  // Kalimba / music-box tine: a round fundamental plus a short metallic partial.
  function kalimba(freq, vol = 0.06, delay = 0, dark = 0, music = false) {
    const bus = music ? musicBus : master, send = music ? 0 : 0.5
    tone(freq, { dur: 1.5 + (1 - dark) * 0.6, vol, delay, send, attack: 0.003, bus })
    tone(freq * 5.42, { dur: 0.2, vol: vol * 0.14 * (1 - dark * 0.7), delay, send, attack: 0.002, bus })
  }

  function chime(freq, vol = 0.07, dur = 1.1, delay = 0) {
    for (const [ratio, v] of [[1, 1], [2.01, 0.38], [3.02, 0.14]]) tone(freq * ratio, { dur: dur / Math.sqrt(ratio), vol: vol * v, delay, send: 0.55, attack: 0.002 })
  }

  const SOUNDS = {
    ui: () => { tone(660, { type: 'triangle', dur: 0.14, vol: 0.05 }); tone(990, { dur: 0.1, vol: 0.018, delay: 0.01 }) },
    launch: (o) => {
      const k = clamp(((o.speed || 20) - 5) / 55, 0, 1)
      hiss({ dur: 0.6 + k * 0.3, vol: 0.1 + k * 0.08, freq: 500 + k * 600, to: 2400 + k * 2600, q: 0.8, attack: 0.12, send: 0.3 })
      tone(330, { dur: 0.45, vol: 0.03, bend: 1.8, attack: 0.08 })
    },
    land: () => { tone(95, { dur: 0.22, vol: 0.16, bend: 0.6, send: 0.1 }); hiss({ dur: 0.12, vol: 0.08, freq: 500, to: 140, type: 'lowpass' }) },
    perfect: (o) => {
      const c = clamp(Math.round(o.chain || 1), 1, 14), f = pent(523.25, Math.min(c - 1, 9))
      chime(f, 0.08)
      chime(f * 1.5, 0.03, 0.9, 0.07)
      if (c >= 3) chime(f * 2, 0.025, 0.8, 0.14)
    },
    thud: () => {
      tone(70, { dur: 0.3, vol: 0.2, bend: 0.55, send: 0.1 })
      hiss({ dur: 0.18, vol: 0.12, freq: 350, to: 100, type: 'lowpass' })
      tone(140, { type: 'triangle', dur: 0.14, vol: 0.05, bend: 0.6, delay: 0.09 })
    },
    splash: () => {
      hiss({ dur: 0.5, vol: 0.16, freq: 1800, to: 500, q: 0.6, attack: 0.01, send: 0.2 })
      hiss({ dur: 0.3, vol: 0.06, freq: 3000, to: 1500, type: 'highpass' })
      tone(140, { dur: 0.22, vol: 0.12, bend: 1.9, delay: 0.04 })
    },
    skip: () => {
      hiss({ dur: 0.09, vol: 0.09, freq: 3200, to: 1800, q: 1.2 })
      tone(900, { dur: 0.07, vol: 0.035, bend: 1.4 })
      tone(260, { dur: 0.1, vol: 0.04, bend: 1.5, delay: 0.02 })
    },
    flap: () => {
      hiss({ dur: 0.07, vol: 0.035, freq: 1400, to: 2600, q: 1.5, send: 0.1 })
      hiss({ dur: 0.06, vol: 0.025, freq: 1700, to: 2900, q: 1.5, delay: 0.055, send: 0.1 })
    },
    cloud: () => {
      for (let i = 0; i < 6; i++) tone(pent(1046.5, i + 1), { dur: 0.5, vol: 0.02, delay: i * 0.05, send: 0.6, attack: 0.003 })
      hiss({ dur: 0.5, vol: 0.03, freq: 5000, to: 8000, type: 'highpass', attack: 0.1, send: 0.5 })
    },
    island: () => {
      [523.25, 659.25, 783.99, 1046.5].forEach((f, i) => kalimba(f, 0.07, i * 0.14))
      tone(261.63, { dur: 1.6, vol: 0.03, attack: 0.2, send: 0.6 })
      chime(1568, 0.025, 1.6, 0.44)
    },
    fever: () => {
      hiss({ dur: 1.1, vol: 0.09, freq: 800, to: 6000, q: 1.5, attack: 0.6, send: 0.4 })
      tone(523.25, { dur: 1, vol: 0.04, bend: 2, attack: 0.5, send: 0.5 })
      tone(783.99, { dur: 1, vol: 0.03, bend: 2, attack: 0.5, send: 0.5 })
      chime(2093, 0.03, 1.4, 0.9)
    },
    feverEnd: () => {
      tone(880, { dur: 0.9, vol: 0.04, bend: 0.5, attack: 0.02, send: 0.5 })
      hiss({ dur: 0.8, vol: 0.04, freq: 4000, to: 700, q: 0.8, attack: 0.05, send: 0.4 })
    },
    night: () => {
      for (const f of [110, 164.81, 220, 329.63]) tone(f, { dur: 3, vol: 0.045, attack: 1.2, send: 0.6 })
    },
    over: () => {
      [783.99, 659.25, 587.33, 523.25].forEach((f, i) => kalimba(f, 0.07, i * 0.34 + (i === 3 ? 0.1 : 0), 0.2))
      chime(261.63, 0.04, 2.4, 1.1)
      tone(196, { dur: 2.6, vol: 0.03, attack: 0.4, delay: 1.0, send: 0.6 })
    },
  }

  function play(name, opts) {
    const sound = SOUNDS[name]
    if (!enabled || !ac || ac.state !== 'running' || !sound) return
    try { sound(opts || EMPTY) } catch { /* Audio failures never interrupt play. */ }
  }

  // Generative bed: a slow pentatonic kalimba line that thins and darkens at night, plus fever sparkles.
  function scheduleMusic(now, night, fever) {
    if (now >= nextNote) {
      if (now > nextNote + 1) nextNote = now
      if (Math.random() < 0.8 - night * 0.5) {
        melody = clamp(melody + Math.round((Math.random() - 0.5) * 4), -4, 6)
        const step = melody - Math.round(night * 5)
        kalimba(pent(ROOT, step), 0.04, 0, night, true)
        if (Math.random() < 0.25 * (1 - night)) kalimba(pent(ROOT, step + 2), 0.025, 0.02, night, true)
      }
      nextNote += (0.6 + night * 0.45) * (Math.random() < 0.35 ? 2 : 1) * (0.9 + Math.random() * 0.2)
    }
    if (now >= nextChord) {
      const chord = CHORDS[chordIndex++ % CHORDS.length]
      for (let i = 0; i < 6; i++) pads[i].frequency.setTargetAtTime(chord[i % 3], now, 1.5)
      if (night < 0.8) kalimba(chord[0] / 2, 0.035, 0, night, true)
      nextChord = now + 9 + night * 4
    }
    if (!fever) nextSpark = now
    else if (now >= nextSpark) {
      const f = pent(ROOT * 2, (Math.random() * 8) | 0)
      tone(f, { dur: 0.55, vol: 0.018, send: 0, attack: 0.003, bus: musicBus })
      tone(f * 2, { dur: 0.25, vol: 0.006, send: 0, attack: 0.002, bus: musicBus })
      nextSpark = now + 0.14 + Math.random() * 0.22
    }
  }

  // Called every frame: cheap, allocation-free. Continuous params are retargeted ~20 times a second.
  function update(s) {
    if (!enabled || !ac || ac.state !== 'running' || !s) return
    const now = ac.currentTime
    const night = clamp(+s.night || 0, 0, 1)
    scheduleMusic(now, night, !!s.fever)
    if (now - lastParams < 0.05) return
    lastParams = now
    const sp = clamp((+s.speed || 0) / 60, 0, 1), air = !!s.air, playing = !!s.playing
    const run = playing ? 1 : 0
    windGain.gain.setTargetAtTime((0.04 + 0.42 * sp) * (air ? 1 : 0.6) * (playing ? 1 : 0.15), now, 0.15)
    windFilter.frequency.setTargetAtTime(250 + 1800 * sp + (air ? 400 : 0), now, 0.2)
    const rolling = s.hold && !air && playing
    rumbleGain.gain.setTargetAtTime(rolling ? 0.7 * (0.3 + 0.7 * sp) : 0, now, 0.25)
    rumbleFilter.frequency.setTargetAtTime(120 + 300 * sp, now, 0.25)
    seaGain.gain.setTargetAtTime(s.water && playing ? 0.22 : 0, now, 0.5)
    padGain.gain.setTargetAtTime(0.012 + 0.004 * night, now, 1)
    padFilter.frequency.setTargetAtTime(1500 - 900 * night, now, 1)
    musicBus.gain.setTargetAtTime((0.75 + 0.25 * run) * (1 - night * 0.2), now, 0.8)
  }

  function setEnabled(value) {
    enabled = !!value
    if (enabled) {
      if (!init()) { enabled = false; return }
      if (suspendTimer && typeof clearTimeout === 'function') clearTimeout(suspendTimer)
      suspendTimer = 0
      if (ac.state !== 'running') void Promise.resolve(ac.resume()).catch(() => {})
      const now = ac.currentTime
      master.gain.setTargetAtTime(MASTER, now, 0.15)
      nextNote = now + 0.5; nextChord = now + 0.1; nextSpark = now; lastParams = 0
    } else if (ac) {
      master.gain.setTargetAtTime(0, ac.currentTime, 0.08)
      if (typeof setTimeout === 'function') {
        suspendTimer = setTimeout(() => {
          suspendTimer = 0
          if (!enabled && ac && ac.state === 'running') void Promise.resolve(ac.suspend()).catch(() => {})
        }, 500)
      }
    }
  }

  function resume() {
    if (enabled && ac && ac.state !== 'running' && ac.state !== 'closed') void Promise.resolve(ac.resume()).catch(() => {})
  }

  function close() {
    enabled = false
    const old = ac
    teardown()
    if (old) void Promise.resolve(old.close()).catch(() => {})
  }

  globalThis.XiabanGliderAudio = { play, update, setEnabled, resume, close, get enabled() { return enabled } }
})()
