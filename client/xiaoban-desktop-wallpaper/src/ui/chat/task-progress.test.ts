import { describe, expect, it } from 'vitest';
import { taskPlan, taskProgress } from './task-progress.ts';
import { applyEvent, itemsFromHistory, newView } from './view.ts';
import { replyBlocks, type ToolCall } from './replies.ts';
import type { WireMessage } from './types.ts';

const todos = [
  { content: '读取资料', status: 'completed', evidence: '已读取 notes.md' },
  { content: '生成周报', status: 'in_progress' },
  { content: '检查交付文件', status: 'pending' },
];
function checkpoint(id = 'p1', entries = todos, handoff = ''): ToolCall {
  return { kind: 'tool', id, name: 'todo', arguments: JSON.stringify({ todos: entries, handoff }), output: 'Todo list updated:', status: 'done' };
}
const block = (calls = [checkpoint()]) => ({ id: 'b1', calls, items: [] });

describe('accepted task plans', () => {
  it('keeps the last committed checkpoint while a replacement is running, failed or malformed', () => {
    const original = checkpoint();
    for (const bad of [
      { ...checkpoint('p2'), status: 'running' as const },
      { ...checkpoint('p2'), status: 'error' as const },
      { ...checkpoint('p2'), output: 'not acknowledged' },
      { ...checkpoint('p2'), arguments: '{bad' },
      checkpoint('p2', [{ content: 'x', status: 'in_progress' }, { content: 'y', status: 'in_progress' }]),
      { ...checkpoint('p2'), arguments: JSON.stringify({ todos: [{ content: '<img onerror=boom>', status: 'unknown' }] }) },
    ]) expect(taskPlan([original, bad])?.callId).toBe('p1');
    expect(taskPlan([original, checkpoint('clear', [])])).toBeNull();
    expect(taskPlan([])).toBeNull();
  });

  it('updates one card from acknowledged events and restores the same evidence from history', () => {
    const view = newView('s1');
    const first = checkpoint();
    applyEvent(view, { type: 'tool_start', id: first.id, name: first.name, arguments: first.arguments });
    expect(taskPlan(replyBlocks(view.items)[0].calls)).toBeNull();
    applyEvent(view, { type: 'tool_end', id: first.id, output: first.output, is_error: false });
    const completed = checkpoint('p2', todos.map(t => ({ ...t, status: 'completed', evidence: '核对通过' })));
    applyEvent(view, { type: 'tool_start', id: completed.id, name: completed.name, arguments: completed.arguments });
    applyEvent(view, { type: 'tool_end', id: completed.id, output: completed.output, is_error: false });
    applyEvent(view, { type: 'done' });
    const blocks = replyBlocks(view.items);
    expect(blocks).toHaveLength(1);
    expect(taskProgress(blocks[0], false)).toMatchObject({ state: 'completed', completed: 3 });
    const history: WireMessage[] = [
      { role: 'assistant', tool_calls: [{ id: first.id, name: first.name, arguments: first.arguments }] },
      { role: 'tool', tool_call_id: first.id, parts: [{ type: 'text', text: first.output }] },
      { role: 'assistant', tool_calls: [{ id: completed.id, name: completed.name, arguments: completed.arguments }] },
      { role: 'tool', tool_call_id: completed.id, parts: [{ type: 'text', text: completed.output }] },
    ];
    const restored = replyBlocks(itemsFromHistory(newView('s1'), history))[0];
    expect(taskPlan(restored.calls)).toEqual(taskPlan(blocks[0].calls));
  });

  it('does not tick unfinished steps when the run ends, fails or is cancelled', () => {
    expect(taskProgress(block(), false)).toMatchObject({ state: 'paused', completed: 1, canResume: true });
    expect(taskProgress(block(), true, 'question')).toMatchObject({ state: 'waiting', completed: 1 });
    expect(taskProgress(block(), true, 'approval')).toMatchObject({ state: 'approval', completed: 1 });
    const view = newView('s1');
    view.items = [checkpoint()];
    applyEvent(view, { type: 'error', message: 'run paused by continuation guard: tool_outcome_needs_review' });
    expect(taskProgress(replyBlocks(view.items)[0], false)).toMatchObject({ state: 'blocked', completed: 1, canResume: false });
    const stopped = newView('s2'); stopped.items = [checkpoint()];
    applyEvent(stopped, { type: 'cancelled' });
    expect(taskProgress(replyBlocks(stopped.items)[0], false)).toMatchObject({ state: 'paused', completed: 1 });
  });

  it('keeps a completed plan running until execution settles and does not hide a later failure', () => {
    const plan = checkpoint('all', todos.map(t => ({ ...t, status: 'completed' })));
    expect(taskProgress(block([plan]), true)?.state).toBe('running');
    const failed: ToolCall = { kind: 'tool', id: 'f', name: 'shell', arguments: '{}', output: 'failed', status: 'error' };
    expect(taskProgress(block([plan, failed]), false)?.state).toBe('blocked');
    expect(taskProgress(block([failed, plan]), false)?.state).toBe('blocked');
    const recovered = { ...failed, id: 'ok', status: 'done' as const };
    expect(taskProgress(block([failed, recovered, plan]), false)?.state).toBe('completed');
  });
});

it('keeps an unrelated success from repairing an earlier failed operation', () => {
  const plan=checkpoint('all',todos.map(t=>({...t,status:'completed'})));
  const failed:ToolCall={kind:'tool',id:'a',name:'write_file',arguments:'{"path":"a.txt"}',output:'failed',status:'error'};
  const read:ToolCall={kind:'tool',id:'b',name:'read_file',arguments:'{"path":"b.txt"}',output:'ok',status:'done'};
  expect(taskProgress(block([failed,read,plan]),false)?.state).toBe('blocked');
});
it('retains backend partial outcomes and distinguishes an unverified plan', () => {
  const plan=checkpoint('all',todos.map(t=>({...t,status:'completed'})));
  const b={...block([plan]),items:[{kind:'notice' as const,id:'outcome',tone:'info' as const,text:'待核验',taskStatus:'unknown' as const}]};
  expect(taskProgress(b,false)?.state).toBe('unverified');
  expect(taskProgress({...b,items:[{...b.items[0]!,taskStatus:'partial' as const}]},false)?.state).toBe('blocked');
});
