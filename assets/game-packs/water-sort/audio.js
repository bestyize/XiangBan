(() => {
  'use strict';

  // Original eight-bar café loop. Everything is synthesized locally, including the water noise.
  const tempo = 92;
  const beat = 60 / tempo;
  const chords = [
    [38, 57, 61, 64, 66], [35, 57, 62, 66, 69],
    [40, 55, 59, 62, 66], [45, 55, 61, 64, 69],
    [42, 57, 61, 64, 69], [43, 57, 59, 62, 66],
    [40, 55, 59, 62, 67], [45, 55, 61, 66, 71],
  ];
  const melody = [[74, 76, 78], [76, 74, 69], [71, 74, 78], [76, 73, 69],
    [73, 76, 81], [78, 76, 74], [71, 74, 79], [76, 73, 71]];
  let ctx, master, musicBus, noiseBuffer, timer, nextBeat, step;
  let settings = { effects: false, music: false, active: false };
  let unlocked = false;
  const frequency = midi => 440 * 2 ** ((midi - 69) / 12);

  function tone(bus, midi, time, length, volume, wave = 'sine') {
    const osc = ctx.createOscillator();
    const envelope = ctx.createGain();
    osc.type = wave;
    osc.frequency.value = frequency(midi);
    envelope.gain.setValueAtTime(.0001, time);
    envelope.gain.exponentialRampToValueAtTime(volume, time + .018);
    envelope.gain.exponentialRampToValueAtTime(.0001, time + length);
    osc.connect(envelope).connect(bus);
    osc.start(time);
    osc.stop(time + length + .01);
  }

  function noise(bus, time, length, volume, pitch, type = 'bandpass') {
    const source = ctx.createBufferSource();
    const filter = ctx.createBiquadFilter();
    const envelope = ctx.createGain();
    source.buffer = noiseBuffer;
    filter.type = type;
    filter.frequency.setValueAtTime(pitch, time);
    filter.frequency.exponentialRampToValueAtTime(pitch * .65, time + length);
    envelope.gain.setValueAtTime(.0001, time);
    envelope.gain.exponentialRampToValueAtTime(volume, time + .02);
    envelope.gain.exponentialRampToValueAtTime(.0001, time + length);
    source.connect(filter).connect(envelope).connect(bus);
    source.start(time, .06, length + .01);
  }

  function schedule() {
    if (ctx.state !== 'running') return;
    if (nextBeat < ctx.currentTime) nextBeat = ctx.currentTime + .025;
    while (nextBeat < ctx.currentTime + .2) {
      const bar = Math.floor(step / 8) % chords.length;
      const eighth = step % 8;
      const t = nextBeat + (eighth % 2 ? beat * .065 : 0);
      const [root, ...voicing] = chords[bar];
      if (eighth === 0 || eighth === 4) tone(musicBus, root + (eighth === 4 ? 7 : 0), t, beat * 1.3, .12);
      if (eighth === 0 || eighth === 5) {
        voicing.forEach((midi, i) => tone(musicBus, midi, t + i * .016, beat * 1.8, .04, 'triangle'));
      }
      if (eighth === 1 || eighth === 3 || eighth === 6) {
        const note = melody[bar][[1, 3, 6].indexOf(eighth)];
        tone(musicBus, note, t, beat * .85, .055);
      }
      if (eighth === 2 || eighth === 6) noise(musicBus, t, .105, .06, 2300, 'highpass');
      step++;
      nextBeat += beat / 2;
    }
  }

  function sync() {
    if (!ctx) return;
    if (settings.active && (settings.effects || settings.music) && unlocked) {
      if (ctx.state === 'suspended') ctx.resume().catch(() => {});
    } else if (ctx.state === 'running') {
      ctx.suspend().catch(() => {});
    }
    const shouldPlay = settings.active && settings.music && unlocked;
    musicBus.gain.setTargetAtTime(shouldPlay ? .17 : 0, ctx.currentTime, .035);
    if (shouldPlay && timer === undefined) {
      step = 0;
      nextBeat = ctx.currentTime + .06;
      timer = setInterval(schedule, 65);
    } else if (!shouldPlay && timer !== undefined) {
      clearInterval(timer);
      timer = undefined;
    }
  }

  function unlock() {
    if (!settings.active || (!settings.effects && !settings.music)) return;
    if (unlocked && ctx?.state === 'running') return;
    if (!ctx) {
      const Audio = globalThis.AudioContext || globalThis.webkitAudioContext;
      if (!Audio) return;
      try {
        ctx = new Audio();
        master = ctx.createGain();
        master.gain.value = .72;
        master.connect(ctx.destination);
        musicBus = ctx.createGain();
        musicBus.gain.value = 0;
        musicBus.connect(master);
        noiseBuffer = ctx.createBuffer(1, ctx.sampleRate, ctx.sampleRate);
        const samples = noiseBuffer.getChannelData(0);
        for (let i = 0; i < samples.length; i++) samples[i] = Math.random() * 2 - 1;
      } catch (_) {
        ctx?.close().catch(() => {});
        ctx = undefined;
        return;
      }
    }
    unlocked = true;
    sync();
  }

  function play(kind, length = .2) {
    if (!settings.active || !settings.effects || ctx?.state !== 'running') return;
    const t = ctx.currentTime + .005;
    if (kind === 'pour') {
      noise(master, t, length, .20, 1850);
      tone(master, 55, t + .025, .16, .033);
      tone(master, 62, t + length * .55, .13, .033);
      return;
    }
    const notes = {
      select: [81], invalid: [48], undo: [76, 69], cup: [69, 76],
      complete: [69, 76, 81], win: [62, 69, 73, 78, 81],
    }[kind];
    if (!notes) return;
    const spaced = kind === 'win' ? .072 : .048;
    notes.forEach((midi, i) => tone(master, midi, t + i * spaced, kind === 'win' ? .46 : .15,
      kind === 'select' || kind === 'invalid' ? .036 : .071));
  }

  globalThis.PourJoyAudio = Object.freeze({
    configure(next) { settings = next; sync(); },
    unlock,
    play,
  });
})();
