import { describe, expect, it } from 'vitest';
import { applyEvent, describeCall, itemsFromHistory, newView } from './view.ts';
import type { WireMessage } from './types.ts';

const text = (value: string) => [{ type: 'text' as const, text: value }];

describe('history to chat items', () => {
  it.each([
    ['verification_required', '目标尚未通过指定命令验收'],
    ['unfinished_milestones', '还有未完成的里程碑'],
    ['permission_denied', '你已拒绝本次操作'],
    ['progress_store_unavailable', '任务进度暂时无法读取'],
  ])('explains the %s pause without suggesting a model switch', (reason, expected) => {
    const view = newView('s1');
    applyEvent(view, { type: 'error', message: `run paused by continuation guard: ${reason}` });
    expect(view.items[0]).toMatchObject({ kind: 'notice', tone: 'warning', text: expect.stringContaining(expected) });
    expect(view.items[0]).not.toHaveProperty('canSwitchModel');
  });
  it('shows an exhausted run budget as a pause with a continue action', () => {
    const view = newView('s1');
    applyEvent(view, { type: 'error', message: 'run budget exceeded: tool calls' });
    expect(view.items[0]).toMatchObject({ kind: 'notice', tone: 'warning', canContinue: true });
    expect(view.items[0]).not.toHaveProperty('canSwitchModel');
  });
  it('keeps tool errors and structured results after reloading history', () => {
    const items = itemsFromHistory(newView('s1'), [
      { role: 'assistant', tool_calls: [{ id: 'c', name: 'mcp', arguments: '{}' }] },
      { role: 'tool', tool_call_id: 'c', parts: text('failed'), is_error: true, structured_content: { retryable: false } },
    ]);
    expect(items[0]).toMatchObject({ status: 'error' });
    expect(items[0]).toHaveProperty('output', 'failed\n{\n  "retryable": false\n}');
  });
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

  it('leaves out what the harness wrote, such as a nudge to keep working', () => {
    const items = itemsFromHistory(newView('s1'), [
      { role: 'user', parts: text('做个游戏') },
      { role: 'assistant', parts: text('先写世界数据：') },
      { role: 'user', parts: text('You said what you would do next but did not call any tool.'), synthetic: true },
      { role: 'assistant', parts: text('写好了') },
    ]);
    expect(items.map(i => (i.kind === 'user' || i.kind === 'assistant' ? i.text : i.kind))).toEqual(['做个游戏', '先写世界数据：', '写好了']);
  });

  it('marks a call that never got an answer as interrupted', () => {
    const items = itemsFromHistory(newView('s1'), [{ role: 'assistant', tool_calls: [{ id: 'c1', name: 'shell', arguments: '{}' }] }]);
    expect(items[0]).toMatchObject({ kind: 'tool', status: 'error', output: '没有执行完就中断了' });
  });
});

describe('reasoning summary and answer streams', () => {
  it('restores displayable reasoning but never renders opaque provider state', () => {
    const items = itemsFromHistory(newView('s1'), [
      { role: 'assistant', parts: text('答案'), reasoning: [
        { kind: 'summary', text: '检查需求，整理结论。' },
        { kind: 'text', text: '模型明确公开的思考文本' },
      ], provider_state: { private: 'unrendered' } } as WireMessage,
    ]);
    expect(items.map(i => i.kind)).toEqual(['reasoning', 'reasoning', 'assistant']);
    expect(items[0]).toMatchObject({ text: '检查需求，整理结论。', reasoningKind: 'summary', streaming: false });
    expect(JSON.stringify(items)).not.toContain('unrendered');
  });

  it('streams reasoning incrementally, separates answer text, and closes on tool dispatch', () => {
    const view = newView('s1', true);
    view.running = true;
    applyEvent(view, { type: 'reasoning_delta', kind: 'summary', text: '分析' });
    applyEvent(view, { type: 'reasoning_delta', kind: 'summary', text: '完成' });
    expect(view.items).toHaveLength(1);
    expect(view.items[0]).toMatchObject({ kind: 'reasoning', text: '分析完成', streaming: true });
    applyEvent(view, { type: 'text_delta', text: '答案' });
    expect(view.items[0]).toMatchObject({ streaming: false });
    expect(view.items[1]).toMatchObject({ kind: 'assistant', text: '答案' });
    applyEvent(view, { type: 'tool_start', id: 't1', name: 'shell', arguments: '{}' });
    applyEvent(view, { type: 'reasoning_delta', kind: 'text', text: '下一步' });
    expect(view.items.at(-1)).toMatchObject({ kind: 'reasoning', text: '下一步' });
    applyEvent(view, { type: 'cancelled' });
    expect(view.items.at(-2)).toMatchObject({ kind: 'reasoning', streaming: false });
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

  it('drops the reasoning of an attempt that stalled when the retry starts, and keeps earlier steps', () => {
    const view = newView('s1', true);
    applyEvent(view, { type: 'tool_start', id: 't1', name: 'shell', arguments: '{}' });
    applyEvent(view, { type: 'tool_end', id: 't1', output: 'ok', is_error: false });
    applyEvent(view, { type: 'reasoning_delta', kind: 'summary', text: '先想想怎么写脚本' });
    expect(view.items.map(i => i.kind)).toEqual(['tool', 'reasoning']);
    applyEvent(view, { type: 'retrying', attempt: 1, delay_ms: 500, message: 'HTTP transport timed out' });
    expect(view.items.map(i => i.kind)).toEqual(['tool']);
    applyEvent(view, { type: 'reasoning_delta', kind: 'summary', text: '重新来' });
    expect(view.items.map(i => i.kind)).toEqual(['tool', 'reasoning']);
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

  it('shows tool cutoff recovery as transient status and finishes without a continue warning', () => {
    const view = newView('s1', true);
    view.running = true;
    applyEvent(view, { type: 'truncated' }); // Older hosts reported this before recovering.
    applyEvent(view, { type: 'tool_end', id: 'cut', output: 'Not run', is_error: true });
    expect(view.items.some(i => i.kind === 'notice')).toBe(true);
    applyEvent(view, { type: 'continuing', reason: 'tool_call_length' });
    expect(view.status).toContain('调整步骤重试');
    expect(view.running).toBe(true);
    expect(view.items.some(i => i.kind === 'notice')).toBe(false);
    applyEvent(view, { type: 'tool_end', id: 'saved', output: 'saved', is_error: false });
    applyEvent(view, { type: 'text_delta', text: '已完成' });
    applyEvent(view, { type: 'done' });
    expect(view.items.some(i => i.kind === 'notice')).toBe(false);
    expect(view.running).toBe(false);
    expect(view.status).toBe('');
  });

  it('clears an older host cutoff after a successful tool but keeps final truncations visible', () => {
    const view = newView('s1');
    applyEvent(view, { type: 'truncated' });
    applyEvent(view, { type: 'tool_end', id: 'saved', output: 'ok', is_error: false });
    expect(view.items.some(i => i.kind === 'notice')).toBe(false);
    applyEvent(view, { type: 'text_delta', text: '最后的说明' });
    applyEvent(view, { type: 'truncated' });
    applyEvent(view, { type: 'done' });
    expect(view.items.at(-1)).toMatchObject({ kind: 'notice', code: 'truncated', canContinue: true });
  });

  it('offers continue only after output recovery is exhausted', () => {
    const view = newView('s1');
    applyEvent(view, { type: 'error', message: 'run paused by continuation guard: output_length_limit' });
    expect(view.items.at(-1)).toMatchObject({ kind: 'notice', tone: 'warning', canContinue: true, text: expect.stringContaining('工具参数超过输出上限') });
    expect(view.items.at(-1)).not.toHaveProperty('canSwitchModel');
  });

  it('starts a new message when the harness keeps a stopped answer going', () => {
    const view = newView('s1', true);
    view.running = true;
    applyEvent(view, { type: 'text_delta', text: '先写世界数据：' });
    applyEvent(view, { type: 'continuing', reason: 'announced_action' });
    expect(view.status).toBe('还没做完，小伴继续处理中…');
    expect(view.running).toBe(true);
    applyEvent(view, { type: 'text_delta', text: '写好了。' });
    expect(view.items.map(i => (i.kind === 'assistant' ? i.text : i.kind))).toEqual(['先写世界数据：', '写好了。']);
    expect(view.status).toBe('');
  });

  it('offers to continue, not to switch models, when an answer hits the step limit', () => {
    const view = newView('s1', true);
    applyEvent(view, { type: 'tool_start', id: 't1', name: 'write_file', arguments: '{}' });
    applyEvent(view, { type: 'tool_end', id: 't1', output: 'ok', is_error: false });
    applyEvent(view, { type: 'error', message: 'stopped after reaching the turn limit (60)' });
    const notice = view.items[view.items.length - 1];
    expect(notice).toMatchObject({ kind: 'notice', tone: 'warning', canContinue: true });
    expect(notice).not.toHaveProperty('canSwitchModel');
  });

  it('shows the continuation guard pause reason and offers only to continue', () => {
    for (const reason of ['no_progress', 'continuation_limit']) {
      const view = newView('s1', true);
      view.running = true;
      applyEvent(view, { type: 'text_delta', text: '现在开始写：' });
      applyEvent(view, { type: 'error', message: `run paused by continuation guard: ${reason}` });
      expect(view.running).toBe(false);
      expect(view.status).toBe('');
      expect(view.items[0]).toMatchObject({ streaming: false });
      const notice = view.items.at(-1);
      expect(notice).toMatchObject({ kind: 'notice', tone: 'warning', canContinue: true });
      expect(notice).not.toHaveProperty('canRetry');
      expect(notice).not.toHaveProperty('canSwitchModel');
      if (notice?.kind === 'notice') expect(notice.text).toContain(reason === 'no_progress' ? '未取得新进展' : '续跑次数上限');
    }
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
  expect(describeCall('community_share', JSON.stringify({kind:'effect',id:'stars'}))).toBe('stars');
  expect(describeCall('community_open', JSON.stringify({target:'forum_wallpaper'}))).toBe('forum_wallpaper');
});

describe('a stored summary in the history', () => {
  const summary = '[[xiaoban-summary keep_from=2]]\n【用户目标】整理文件\n【已完成】列出了目录';

  it('is shown as a notice that unfolds to the summary, not as something the user said', () => {
    const items = itemsFromHistory(newView('s1'), [
      { role: 'user', parts: text('旧问题') }, { role: 'assistant', parts: text('旧回答') },
      { role: 'user', parts: text('最近的问题') }, { role: 'assistant', parts: text('最近的回答') },
      { role: 'user', synthetic: true, context_summary: { keep_from: 2, text: '【用户目标】整理文件\n【已完成】列出了目录' } },
      { role: 'user', parts: text('之后的问题') },
    ]);
    expect(items.map(i => i.kind)).toEqual(['user', 'assistant', 'user', 'assistant', 'notice', 'user']);
    const notice = items[4] as Extract<typeof items[number], { kind: 'notice' }>;
    expect(notice.tone).toBe('info');
    expect(notice.detailsLabel).toBe('查看摘要');
    expect(notice.details).toBe('【用户目标】整理文件\n【已完成】列出了目录');
    expect(notice.text).toContain('压缩成摘要');
    expect(items.some(i => i.kind === 'user' && i.text.includes('xiaoban-summary'))).toBe(false);
  });

  it('keeps all user-supplied marker strings as ordinary messages', () => {
    for (const said of [summary, '[[xiaoban-summary keep_from=2]] 没有换行', '我贴一下：[[xiaoban-summary keep_from=2]]\nx', '[[xiaoban-summary keep_from=]]\nx']) {
      const items = itemsFromHistory(newView('s1'), [{ role: 'user', parts: text(said) }]);
      expect(items.map(i => i.kind)).toEqual(['user']);
    }
  });
});


it('restores a file-only message from saved attachment metadata', () => {
  const items=itemsFromHistory(newView('s1'),[{role:'user',parts:[],structured_content:{attachments:[{id:'a',name:'报告.pdf',size:200,mime:'application/pdf',path:'/private/copy.pdf'}]}}]);
  expect(items).toHaveLength(1);
  expect(items[0]).toMatchObject({kind:'user',text:'',images:[],files:[{name:'报告.pdf',size:200}]});
});

describe('the tokens of an answer', () => {
  it('adds every model call, research helpers included, and puts the total on the last reply', () => {
    const view = newView('s1', true);
    view.items.push({ kind: 'user', id: 'u1', text: '做个游戏', images: [] });
    applyEvent(view, { type: 'text_delta', text: '先看看' });
    applyEvent(view, { type: 'usage', prompt_tokens: 12_000, completion_tokens: 300 });
    applyEvent(view, { type: 'usage', prompt_tokens: 3_000, completion_tokens: 200 });
    applyEvent(view, { type: 'text_delta', text: '好了' });
    applyEvent(view, { type: 'usage', prompt_tokens: 13_000, completion_tokens: 500 });
    applyEvent(view, { type: 'done' });
    const replies = view.items.filter(i => i.kind === 'assistant');
    expect(replies.at(-1)).toMatchObject({ usage: { prompt: 28_000, completion: 1_000 } });
    expect(view.usage).toBeUndefined();
  });

  it('starts again for the next answer and leaves an answer without usage alone', () => {
    const view = newView('s1', true);
    view.items.push({ kind: 'user', id: 'u1', text: '你好', images: [] });
    applyEvent(view, { type: 'text_delta', text: '你好' });
    applyEvent(view, { type: 'done' });
    expect(view.items.at(-1)).not.toHaveProperty('usage');
  });
});


it('restores interrupted reasoning and answer with an explicit continuation notice', () => {
  const view = newView('recovered', true);
  const messages: WireMessage[] = [{ role: 'assistant', parts: [{ type: 'text', text: 'answer prefix' }],
    reasoning: [{ kind: 'text', text: 'checking' }], structured_content: { interrupted: true } }];
  const items = itemsFromHistory(view, messages);
  expect(items.map(item => item.kind)).toEqual(['reasoning', 'assistant', 'notice']);
  expect(items[0]).toMatchObject({ text: 'checking', streaming: false });
  expect(items[1]).toMatchObject({ text: 'answer prefix', streaming: false });
  expect(items[2]).toMatchObject({ tone: 'warning', canContinue: true });
});

describe('reliability pause reasons', () => {
  it('requires inspecting unknown actions instead of offering blind continuation', () => {
    const view = newView('s1');
    applyEvent(view, { type: 'error', message: 'run paused by continuation guard: tool_outcome_needs_review' });
    expect(view.items[0]).toMatchObject({ kind: 'notice', tone: 'warning', canContinue: false });
    expect(view.items[0]).toHaveProperty('text', expect.stringContaining('勿直接重复执行'));
  });
  it('describes elapsed time and token budgets separately from tool count', () => {
    const view = newView('s1');
    applyEvent(view, { type: 'error', message: 'run budget exceeded: elapsed time' });
    applyEvent(view, { type: 'error', message: 'run budget exceeded: reported tokens' });
    expect(view.items[0]).toHaveProperty('text', expect.stringContaining('运行时间'));
    expect(view.items[1]).toHaveProperty('text', expect.stringContaining('模型用量'));
  });
});
