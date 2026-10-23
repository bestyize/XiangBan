/** Short sounds of "I heard you", played while the answer is still being thought out. */
export const ACK_PHRASES = ['好的', '嗯，我看看', '我想想'] as const;
/** Nothing is said before this long after the question is sent; a quick answer needs no acknowledgement. */
export const ACK_DELAY_MS = 700;

/** One phrase, never the one used last. `pick` returns a number in [0, 1). */
export function chooseAck<T>(clips: readonly T[], last: T | undefined, pick: () => number = Math.random): T | undefined {
  const options = clips.length > 1 ? clips.filter(clip => clip !== last) : clips;
  return options.length ? options[Math.min(options.length - 1, Math.floor(pick() * options.length))] : undefined;
}
