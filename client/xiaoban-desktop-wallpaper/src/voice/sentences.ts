import { speechText } from '../ui/speech/text.ts';

/** Characters that end a sentence worth speaking on its own. */
const ENDS = new Set(['。', '！', '？', '!', '?', '；', ';', '…', '\n']);
/** Where a long sentence without an end may be cut so the first words are not held back. */
const PAUSES = new Set(['，', ',', '、', '：', ':']);
/** A clause longer than this is cut at the next pause instead of waiting for the sentence end. */
const LONG = 48;
/** The first sentence is what the listener waits for, so it is cut at a pause much sooner. */
const FIRST_LONG = 8;

/**
 * Cuts a streamed Markdown answer into sentences that can be synthesised one by one.
 * Feed it the whole text so far each time it grows; it returns only the sentences
 * completed since the last call. Code blocks are skipped and Markdown syntax removed.
 */
export class SentenceChunker {
  private seen = 0;
  private pending = '';
  private line = '';
  private fence = false;
  private spoken = 0;

  push(full: string): string[] {
    if (full.length <= this.seen) return [];
    const added = full.slice(this.seen);
    this.seen = full.length;
    const out: string[] = [];
    for (let i = 0; i < added.length; i++) {
      const ch = added[i];
      this.line += ch;
      if (ch === '\n') {
        const fenceLine = this.line.trimStart().startsWith('```');
        if (fenceLine) {
          if (!this.fence) this.pending = this.pending.slice(0, Math.max(0, this.pending.length - (this.line.length - 1)));
          this.fence = !this.fence;
          this.line = '';
          continue;
        }
        this.line = '';
      }
      if (this.fence) continue;
      this.pending += ch;
      const next = added[i + 1];
      const end = ENDS.has(ch) || (ch === '.' && (next === ' ' || next === '\n'));
      const pause = PAUSES.has(ch) && this.pending.length >= (this.spoken ? LONG : FIRST_LONG);
      if (end || pause) this.take(out);
    }
    return out;
  }

  /** The answer is complete: whatever is left is the last sentence. */
  flush(): string[] {
    const out: string[] = [];
    if (!this.fence) this.take(out, true);
    this.pending = ''; this.line = '';
    return out;
  }

  private take(out: string[], last = false) {
    const text = speechText(this.pending).replace(/\|/g, '，').trim();
    // Too little to say on its own ("1." of a list, a lone emoji): keep it for the next sentence.
    if (!last && text.length < 2 && !/\n$/.test(this.pending)) return;
    if (text) { out.push(text); this.spoken++; }
    this.pending = '';
  }
}
