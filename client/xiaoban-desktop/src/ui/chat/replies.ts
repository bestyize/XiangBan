import type { ChatItem } from './types.ts';

export type ToolCall = Extract<ChatItem, { kind: 'tool' }>;
export interface ReplyBlock {
  id: string;
  items: Exclude<ChatItem, ToolCall>[];
  calls: ToolCall[];
}

/** Keep stored events in order; collect each user turn's tools beneath its reply for display. */
export function replyBlocks(items: ChatItem[]): ReplyBlock[] {
  const blocks: ReplyBlock[] = [];
  let reply: ReplyBlock | undefined;
  for (const item of items) {
    if (item.kind === 'user') {
      blocks.push({ id: item.id, items: [item], calls: [] });
      reply = undefined;
      continue;
    }
    if (!reply) {
      reply = { id: `reply-${item.id}`, items: [], calls: [] };
      blocks.push(reply);
    }
    if (item.kind === 'tool') reply.calls.push(item);
    else reply.items.push(item);
  }
  return blocks;
}
