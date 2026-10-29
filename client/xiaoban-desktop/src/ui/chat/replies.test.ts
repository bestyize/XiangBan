import { expect, it } from 'vitest';
import { replyBlocks } from './replies.ts';
import type { ChatItem } from './types.ts';

it('collects interleaved tools beneath each reply without changing stored events or hiding notices', () => {
  const items: ChatItem[] = [
    { kind: 'user', id: 'u1', text: '第一轮', images: [] },
    { kind: 'tool', id: 't1', name: 'shell', arguments: '{}', output: '', status: 'done' },
    { kind: 'assistant', id: 'a1', text: '回复', streaming: false },
    { kind: 'notice', id: 'n1', text: '需重试', tone: 'error' },
    { kind: 'tool', id: 't2', name: 'shell', arguments: '{}', output: '', status: 'error' },
    { kind: 'user', id: 'u2', text: '', images: ['data:image/png;base64,AA=='] },
    { kind: 'tool', id: 't3', name: 'get_app_state', arguments: '{}', output: '', status: 'running' },
  ];
  const before = JSON.stringify(items);
  const blocks = replyBlocks(items);
  expect(blocks.map(block => block.calls.map(call => call.id))).toEqual([[], ['t1', 't2'], [], ['t3']]);
  expect(blocks[1].items.map(item => item.id)).toEqual(['a1', 'n1']);
  expect(JSON.stringify(items)).toBe(before);
  // A reply that begins with tools keeps its component identity as prose streams in later.
  expect(replyBlocks([...items, { kind: 'assistant', id: 'a2', text: '下一轮回复', streaming: true }]).at(-1)?.id).toBe(blocks.at(-1)?.id);
});
