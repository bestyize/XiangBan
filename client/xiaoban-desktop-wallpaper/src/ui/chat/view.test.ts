import { describe, expect, it } from 'vitest';
import { applyEvent, describeCall, itemsFromHistory, newView } from './view.ts';
import type { WireMessage } from './types.ts';

const text = (value: string) => [{ type: 'text' as const, text: value }];

describe('history to chat items', () => {
  it('shows user and assistant text and puts each tool result on its call', () => {
    const history: WireMessage[] = [
      { role: 'user', parts: text('整理下载文件夹') },
      { role: 'assistant', parts: text('先看一下里面有什么'), tool_calls: [{ id: 'c1', name: 'list_dir', arguments: '{}' }, { id: 'c2', name: 'shell', arguments: '{"command":"ls"}' }] },
      { role: 'tool', parts: text('a.txt\nb.txt'), tool_call_id: 'c2' },
      { role: 'tool', parts: text('x'), tool_call_id: 'c1' },
      { role: 'assistant', parts: text('整理好了') },
    ];
    const items = itemsFromHistory(newView('s1'), history);
    expect(items.map(i => i.kind)).toEqual(['user', 'assistant', 'tool', 'tool', 'assistant']);
    expect(items[2]).toMatchObject({ id: 'c1', name: 'list_dir', output: 'x', status: 'done' });
    expect(items[3]).toMatchObject({ id: 'c2', output: 'a.txt\nb.txt' });
  });

  it('skips empty assistant turns, keeps images, and ignores stray results and system messages', () => {
    const items = itemsFromHistory(newView('s1'), [
      { role: 'system', parts: text('hidden') },
      { role: 'user', parts: [{ type: 'image', url: 'data:x' }, { type: 'text', text: '看这个' }] },
      { role: 'user', parts: [{ type: 'image', url: 'data:y' }] },
      { role: 'assistant', parts: [] },
      { role: 'assistant', parts: text('   ') },
      { role: 'tool', parts: text('orphan'), tool_call_id: 'nope' },
      { role: 'tool', parts: text('no id') },
    ]);
    expect(items).toEqual([
      { kind: 'user', id: 's1:1', text: '看这个', images: ['data:x'] },
      { kind: 'user', id: 's1:2', text: '', images: ['data:y'] },
    ]);
  });

  it('marks a call that never got an answer as interrupted', () => {
    const items = itemsFromHistory(newView('s1'), [{ role: 'assistant', tool_calls: [{ id: 'c1', name: 'shell', arguments: '{}' }] }]);
    expect(items[0]).toMatchObject({ kind: 'tool', status: 'error', output: '没有执行完就中断了' });
  });
});

describe('events while an answer is running', () => {
  it('streams text into one growing message and starts a new one after a tool', () => {
    const view = newView('s1', true);
    view.running = true;
    applyEvent(view, { type: 'text_delta', text: '好的，' });
    applyEvent(view, { type: 'text_delta', text: '我来看看' });
    expect(view.items).toHaveLength(1);
    expect(view.items[0]).toMatchObject({ kind: 'assistant', text: '好的，我来看看', streaming: true });
    applyEvent(view, { type: 'tool_start', id: 't1', name: 'shell', arguments: '{"command":"ls"}' });
    expect(view.items[0]).toMatchObject({ streaming: false });
    expect(view.items[1]).toMatchObject({ kind: 'tool', status: 'running' });
    applyEvent(view, { type: 'tool_end', id: 't1', output: 'a\nb', is_error: false });
    expect(view.items[1]).toMatchObject({ status: 'done', output: 'a\nb' });
    applyEvent(view, { type: 'text_delta', text: '有两个文件' });
    expect(view.items[2]).toMatchObject({ kind: 'assistant', text: '有两个文件', streaming: true });
    applyEvent(view, { type: 'done' });
    expect(view.items[2]).toMatchObject({ streaming: false });
    expect(view.running).toBe(false);
  });

  it('shows a failed tool as an error and tolerates an end without a start', () => {
    const view = newView('s1', true);
    applyEvent(view, { type: 'tool_start', id: 't1', name: 'shell', arguments: '{}' });
    applyEvent(view, { type: 'tool_end', id: 't1', output: 'boom', is_error: true });
    applyEvent(view, { type: 'tool_end', id: 'lost', output: 'late', is_error: false });
    expect(view.items.map(i => i.kind === 'tool' && i.status)).toEqual(['error', 'done']);
  });

  it('shows a retry while it lasts and clears it once the answer flows', () => {
    const view = newView('s1', true);
    applyEvent(view, { type: 'retrying', attempt: 2, delay_ms: 1000, message: 'busy' });
    expect(view.status).toContain('第 2 次重试');
    applyEvent(view, { type: 'text_delta', text: 'x' });
    expect(view.status).toBe('');
  });

  it('ends cleanly on done, cancel and error, dropping any open questions and showing the outcome', () => {
    for (const [event, tone, text] of [
      [{ type: 'done' }, undefined, undefined],
      [{ type: 'cancelled' }, 'info', '已停止'],
      [{ type: 'error', message: '模型不可用' }, 'error', '暂时无法完成回复，可以重试或切换模型。'],
    ] as const) {
      const view = newView('s1', true);
      view.running = true;
      view.status = 'x';
      view.approvals.push({ id: 'a1', session: 's1', tool: 'shell', risk: 'exec', arguments: {}, tainted: false, canRemember: false });
      view.questions.push({ id: 'q1', session: 's1', question: '?', options: [] });
      applyEvent(view, { type: 'tool_start', id: 't', name: 'shell', arguments: '{}' });
      applyEvent(view, event);
      expect(view).toMatchObject({ running: false, approvals: [], questions: [], status: '' });
      const tool = view.items.find(i => i.kind === 'tool');
      expect(tool).toMatchObject({ status: 'error', output: '已中断' });
      const notice = view.items[view.items.length - 1];
      if (tone) expect(notice).toMatchObject({ kind: 'notice', tone, text });
      else expect(notice.kind).toBe('tool');
    }
  });

  it('offers to continue after a cut-off answer', () => {
    const view = newView('s1', true);
    applyEvent(view, { type: 'text_delta', text: '写到一半' });
    applyEvent(view, { type: 'truncated' });
    applyEvent(view, { type: 'done' });
    expect(view.items[0]).toMatchObject({ streaming: false });
    expect(view.items[1]).toMatchObject({ kind: 'notice', tone: 'warning', canContinue: true });
  });

  it('gives every item of a conversation its own key', () => {
    const view = newView('s1', true);
    for (let i = 0; i < 3; i++) { applyEvent(view, { type: 'text_delta', text: 'a' }); applyEvent(view, { type: 'tool_start', id: `t${i}`, name: 'x', arguments: '{}' }); }
    const ids = view.items.map(i => i.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe('tool call labels', () => {
  it('picks the most telling argument and keeps it short', () => {
    expect(describeCall('shell', '{"command":"ls -la\\npwd"}')).toBe('ls -la');
    expect(describeCall('read_file', '{"path":"notes/a.md","offset":1}')).toBe('notes/a.md');
    expect(describeCall('web_fetch', '{"url":"https://example.com"}')).toBe('https://example.com');
    expect(describeCall('wallpaper_apply', '{"id":"w2"}')).toBe('w2');
    expect(describeCall('shell', `{"command":"${'x'.repeat(200)}"}`)).toHaveLength(91);
    expect(describeCall('time_now', '{}')).toBe('');
    expect(describeCall('shell', '{"command": "unfinis')).toBe('');
    expect(describeCall('shell', 'null')).toBe('');
    expect(describeCall('shell', '[1,2]')).toBe('');
  });
});



it('summarizes authored resources by title or pack name', () => {
  expect(describeCall('wallpaper_create', JSON.stringify({title:'星空',html:'<html></html>'}))).toBe('星空');
  expect(describeCall('effect_create', JSON.stringify({pack:{id:'stars',name:'繁星'}}))).toBe('繁星');
  expect(describeCall('game_create', JSON.stringify({id:'star-game',name:'摘星星',files:[]}))).toBe('摘星星');
  expect(describeCall('resource_read', JSON.stringify({kind:'wallpaper',id:'my-snow'}))).toBe('my-snow');
  expect(describeCall('resource_edit', JSON.stringify({kind:'widget',id:'my-clock',changes:[]}))).toBe('my-clock');
  expect(describeCall('resource_restore', JSON.stringify({kind:'game',id:'my-game'}))).toBe('my-game');
});
