const CLIPS = ['nice', 'good', 'great', 'wonderful'];

export function rewardFor(events, status) {
  if (status === 'won') return 'wonderful';
  if (status === 'lost') return null;
  const waves = events.filter(event => event.type === 'clear');
  if (!waves.length) return null;
  const chain = Math.max(...waves.map(event => event.chain));
  if (chain >= 4) return 'wonderful';
  if (chain >= 3 || waves.some(event => event.combo || event.created.some(({tile}) => tile.special === 'rainbow'))) return 'great';
  if (chain >= 2 || waves.some(event => event.created.some(({tile}) => tile.special === 'bomb'))) return 'good';
  if (waves.some(event => event.created.length || event.cleared.length >= 6)) return 'nice';
  return null;
}

export function createRewards(context, allowed, duck) {
  const buffers = new Map();
  let pending, active, lastAt = -Infinity, lastTier = -1;
  const stop = () => {
    if (!active) return;
    const current = active; active = null;
    current.source.stop(); current.source.disconnect(); current.gain.disconnect();
  };
  return {
    stop,
    load() {
      if (pending) return pending;
      pending = Promise.all(CLIPS.map(async name => {
        try {
          const response = await fetch(new URL(`./assets/voice-${name}.mp3`, import.meta.url));
          if (!response.ok) return;
          buffers.set(name, await context.decodeAudioData(await response.arrayBuffer()));
        } catch { /* An unavailable clip must never interrupt gameplay. */ }
      }));
      return pending;
    },
    play(name, victory = false) {
      const tier = CLIPS.indexOf(name), buffer = buffers.get(name);
      if (tier < 0 || !buffer || !allowed() || context.state !== 'running') return false;
      const now = context.currentTime;
      if (!victory && (active || (now - lastAt < 2.4 && tier <= lastTier))) return false;
      // Victory may replace an older word; nothing is queued for later playback.
      stop();
      const source = context.createBufferSource(), gain = context.createGain();
      source.buffer = buffer; gain.gain.value = .52;
      source.connect(gain); gain.connect(context.destination);
      const current = {source, gain}; active = current;
      source.onended = () => {
        if (active === current) { active = null; source.disconnect(); gain.disconnect(); }
      };
      duck(buffer.duration + .25);
      lastAt = now; lastTier = tier; source.start();
      return true;
    },
  };
}
