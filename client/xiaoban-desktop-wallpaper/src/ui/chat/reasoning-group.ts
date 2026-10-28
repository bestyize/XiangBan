import type { ChatItem } from './types.ts';

type ReasoningItem = Extract<ChatItem, { kind: 'reasoning' }>;
/** Several finished thinking segments in a row, shown as one line. */
export interface ReasoningGroup { kind: 'reasoning-group'; id: string; items: ReasoningItem[] }

/**
 * Merges runs of two or more finished reasoning items into one group. A segment still streaming stays on its own
 * so its live preview remains visible, and anything between segments (an answer, a notice) keeps the order.
 */
export function groupReasoning(items: ChatItem[]): (ChatItem | ReasoningGroup)[] {
  const out: (ChatItem | ReasoningGroup)[] = [];
  let run: ReasoningItem[] = [];
  const flush = () => {
    if (run.length > 1) out.push({ kind: 'reasoning-group', id: `${run[0]!.id}+${run.length}`, items: run });
    else out.push(...run);
    run = [];
  };
  for (const item of items) {
    if (item.kind === 'reasoning' && !item.streaming) run.push(item);
    else { flush(); out.push(item); }
  }
  flush();
  return out;
}
