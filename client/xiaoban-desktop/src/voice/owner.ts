/**
 * The console's conversation and the floating assistant are separate windows, each with its own microphone,
 * recogniser and voice, and both hear every chat event. Left alone they listen to the same words, answer them
 * twice and talk over each other. The microphone and the voice belong to one window at a time: the one that
 * started using them last. When another window takes over, this one stops listening and speaking at once.
 */
const EVENT = 'voice-owner';

export interface VoiceOwner {
  /** This window is about to use the microphone or the voice: the others stop theirs. */
  claim(): void;
  dispose(): void;
}
export interface VoiceOwnerDeps {
  /** This window's name. */
  me: string;
  emit(event: string, payload: unknown): Promise<unknown> | void;
  listen<T>(event: string, callback: (payload: T) => void): Promise<() => void>;
  /** Another window took over while this one had the voice. */
  onLost(): void;
}

export function createVoiceOwner(deps: VoiceOwnerDeps): VoiceOwner {
  let holder: string | null = null, disposed = false, off: (() => void) | undefined;
  void deps.listen<{ owner?: string }>(EVENT, ({ owner }) => {
    if (disposed || typeof owner !== 'string' || owner === deps.me) return;
    const lost = holder === deps.me;
    holder = owner;
    if (lost) deps.onLost();
  }).then(stop => { if (disposed) stop(); else off = stop; }, () => {});
  return {
    claim() {
      if (disposed || holder === deps.me) return;
      holder = deps.me;
      try { void Promise.resolve(deps.emit(EVENT, { owner: deps.me })).catch(() => {}); } catch { /* a window that cannot tell the others still works alone */ }
    },
    dispose() { disposed = true; off?.(); },
  };
}
