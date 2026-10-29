import { expect, it } from 'vitest';
import { groupReasoning } from './reasoning-group.ts';
import type { ChatItem } from './types.ts';

const reasoning = (id: string, streaming = false): ChatItem => ({ kind: 'reasoning', id, text: id, reasoningKind: 'summary', streaming });
const answer = (id: string): ChatItem => ({ kind: 'assistant', id, text: id, streaming: false });

it('merges runs of finished reasoning into one line and leaves everything else in order', () => {
  const out = groupReasoning([reasoning('a'), reasoning('b'), reasoning('c'), answer('x'), reasoning('d')]);
  expect(out.map(item => item.kind)).toEqual(['reasoning-group', 'assistant', 'reasoning']);
  expect(out[0]).toMatchObject({ kind: 'reasoning-group', items: [{ id: 'a' }, { id: 'b' }, { id: 'c' }] });
});

it('keeps a streaming segment separate so its live preview stays visible', () => {
  const out = groupReasoning([reasoning('a'), reasoning('b'), reasoning('c', true)]);
  expect(out.map(item => item.kind)).toEqual(['reasoning-group', 'reasoning']);
});

it('never groups a lone segment', () => {
  expect(groupReasoning([reasoning('a'), answer('x')]).map(item => item.kind)).toEqual(['reasoning', 'assistant']);
});
