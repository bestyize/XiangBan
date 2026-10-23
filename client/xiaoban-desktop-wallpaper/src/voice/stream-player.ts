import { SentenceChunker } from './sentences.ts';

/** One clip being played; `stop` ends it early and settles `done`. */
export interface Clip { done: Promise<void>; stop(): void }

export interface StreamPlayerDeps {
  /** Turns one sentence into audio. Runs ahead of playback, so several may be in flight. */
  synthesize(text: string): Promise<ArrayBuffer | Uint8Array>;
  /** Starts playing audio and reports when it has ended. */
  play(audio: ArrayBuffer | Uint8Array): Promise<Clip>;
  /** Sentences synthesised ahead of the one playing. */
  ahead?: number;
  /** Awaited before the first sentence is played, e.g. to let a short acknowledgement finish instead of talking over it. */
  beforeFirst?(): Promise<void> | void;
}

export interface StreamPlayerEvents {
  /** The first sentence started playing. */
  onStart?(): void;
  /** A sentence started playing. */
  onSentence?(text: string): void;
  /** A sentence could not be synthesised or played; the rest carries on. */
  onError?(error: unknown): void;
}

export interface StreamPlayer {
  /** The whole answer so far (Markdown); new complete sentences are queued. */
  push(full: string): void;
  /** The answer is complete. Resolves once everything has been played, or at once when stopped. */
  end(full?: string): Promise<void>;
  stop(): void;
  /** Sentences queued, synthesising or playing, and the answer not yet ended or not yet played out. */
  readonly active: boolean;
  /** A sentence is audible right now. */
  readonly playing: boolean;
  /** What has been played so far, for telling the assistant's own voice from the user's. */
  readonly spoken: string;
}

interface Item { text: string; audio?: Promise<ArrayBuffer | Uint8Array | Error> }

/**
 * Reads an answer while it is still being written: each complete sentence is synthesised as soon as it
 * appears (a little ahead of playback) and played in order, so the first words are heard after one sentence
 * instead of after the whole answer.
 */
export function createStreamPlayer(deps: StreamPlayerDeps, events: StreamPlayerEvents = {}): StreamPlayer {
  const ahead = Math.max(1, deps.ahead ?? 2);
  const chunker = new SentenceChunker();
  const items: Item[] = [];
  let index = 0, busy = false, ended = false, stopped = false, started = false, playing = false;
  let spoken = '', clip: Clip | null = null;
  let finish!: () => void;
  const finished = new Promise<void>(resolve => { finish = resolve; });

  function enqueue(sentences: string[]) {
    if (stopped) return;
    for (const text of sentences) items.push({ text });
    pump();
  }
  function pump() {
    if (stopped) return;
    for (let i = index; i < Math.min(items.length, index + 1 + ahead); i++) {
      const item = items[i];
      item.audio ??= deps.synthesize(item.text).catch((error: unknown) => (error instanceof Error ? error : new Error(String(error))));
    }
    if (!busy && index < items.length) void playNext();
    else if (!busy && ended) finish();
  }
  async function playNext() {
    busy = true;
    const item = items[index];
    try {
      const audio = await item.audio!;
      if (stopped) return;
      if (audio instanceof Error) { events.onError?.(audio); return; }
      if (!started) { await deps.beforeFirst?.(); if (stopped) return; }
      clip = await deps.play(audio);
      if (stopped) { clip.stop(); return; }
      playing = true; spoken += item.text;
      if (!started) { started = true; events.onStart?.(); }
      events.onSentence?.(item.text);
      await clip.done;
    } catch (error) { if (!stopped) events.onError?.(error); }
    finally {
      playing = false; clip = null; index++; busy = false;
      if (stopped) finish(); else pump();
    }
  }
  return {
    push(full) { if (!ended) enqueue(chunker.push(full)); },
    end(full) {
      if (!ended && !stopped) {
        if (full !== undefined) enqueue(chunker.push(full));
        enqueue(chunker.flush());
        ended = true;
        pump();
      }
      return finished;
    },
    stop() {
      if (stopped) return;
      stopped = true; ended = true; playing = false;
      clip?.stop();
      if (!busy) finish();
    },
    get active() { return !stopped && (!ended || index < items.length || busy); },
    get playing() { return playing; },
    get spoken() { return spoken; },
  };
}
