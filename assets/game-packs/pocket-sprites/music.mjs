// One decoded, looping music source per AudioContext; scene changes never stack it.
export function createMusic(context, allowed) {
  let source, gain, pending;
  let duckUntil = 0;
  const sync = () => {
    if (!gain) return;
    const now = context.currentTime;
    gain.gain.cancelScheduledValues(now);
    gain.gain.setValueAtTime(gain.gain.value, now);
    gain.gain.setTargetAtTime(allowed() ? (duckUntil > now ? .022 : .1) : 0, now, .035);
    if (allowed() && duckUntil > now) gain.gain.setTargetAtTime(.1, duckUntil, .22);
  };
  return {
    sync,
    duck(seconds = .65) {
      duckUntil = Math.max(duckUntil, context.currentTime + seconds);
      sync();
    },
    load() {
      if (pending || source) return;
      pending = (async () => {
        const response = await fetch(new URL('./assets/garden-skip.mp3', import.meta.url));
        if (!response.ok) throw new Error('Music unavailable');
        const buffer = await context.decodeAudioData(await response.arrayBuffer());
        gain = context.createGain(); gain.gain.value = 0; gain.connect(context.destination);
        source = context.createBufferSource(); source.buffer = buffer;
        source.loop = true; source.loopStart = 0;
        source.loopEnd = Math.min(buffer.duration, 69.81818594104308);
        source.connect(gain); source.start(); sync();
      })().catch(() => { pending = null; });
    },
  };
}
