import { modelFailure } from './model-errors.ts';
import type { ChatItem, KernelEvent, SessionView, WireMessage } from './types.ts';

export function newView(id: string, loaded = false): SessionView {
  return { id, items: [], loaded, running: false, approvals: [], questions: [], status: '', seq: 0 };
}

export const nextId = (view: SessionView) => `${view.id}:${++view.seq}`;

const textOf = (message: WireMessage) => (message.parts ?? []).map(part => (part.type === 'text' ? part.text : '')).join('');
const toolOutput = (output: string, structured: unknown) => structured === undefined || structured === null ? output : [output, JSON.stringify(structured, null, 2)].filter(Boolean).join('\n');

/** What the conversation shows where an older part was replaced by a summary. */
export function summaryNotice(view: SessionView, summary: string): ChatItem {
  return { kind: 'notice', id: nextId(view), tone: 'info', text: '较早的对话已压缩成摘要，小伴之后会参考摘要和最近的内容。', details: summary, detailsLabel: '查看摘要' };
}

/** The conversation as it was saved: tool results are put onto the calls they answer. */
export function itemsFromHistory(view: SessionView, messages: WireMessage[]): ChatItem[] {
  const items: ChatItem[] = [];
  const tools = new Map<string, Extract<ChatItem, { kind: 'tool' }>>();
  for (const message of messages) {
    if (message.role === 'user' && message.synthetic && message.context_summary) {
      items.push(summaryNotice(view, message.context_summary.text));
      continue;
    }
    if (message.synthetic) continue;
    if (message.role === 'user') {
      const images = (message.parts ?? []).filter(part => part.type === 'image').map(part => (part as { url: string }).url);
      const text = textOf(message);
      if (text || images.length) items.push({ kind: 'user', id: nextId(view), text, images });
    } else if (message.role === 'assistant') {
      const text = textOf(message);
      if (text.trim()) items.push({ kind: 'assistant', id: nextId(view), text, streaming: false });
      for (const call of message.tool_calls ?? []) {
        const item = { kind: 'tool' as const, id: call.id, name: call.name, arguments: call.arguments, output: '', status: 'running' as const };
        tools.set(call.id, item);
        items.push(item);
      }
    } else if (message.role === 'tool' && message.tool_call_id) {
      const item = tools.get(message.tool_call_id);
      if (item) { item.output = toolOutput(textOf(message), message.structured_content); item.status = message.is_error ? 'error' : 'done'; }
    }
  }
  // A call that never got an answer was cut short, for example by closing the app.
  for (const item of tools.values()) if (item.status === 'running') { item.status = 'error'; item.output = item.output || '没有执行完就中断了'; }
  return items;
}

/** The kernel's `MaxTurns` error (`error.rs`). */
const TURN_LIMIT = /stopped after reaching the turn limit/;

function endStreaming(view: SessionView) {
  for (const item of view.items) if (item.kind === 'assistant' && item.streaming) item.streaming = false;
}

function settle(view: SessionView) {
  endStreaming(view);
  for (const item of view.items) if (item.kind === 'tool' && item.status === 'running') { item.status = 'error'; item.output = item.output || '已中断'; }
  view.running = false;
  view.approvals = [];
  view.questions = [];
  view.status = '';
}

/** Folds one event of a running answer into the conversation shown. */
export function applyEvent(view: SessionView, event: KernelEvent): void {
  switch (event.type) {
    case 'text_delta': {
      view.status = '';
      const last = view.items[view.items.length - 1];
      if (last?.kind === 'assistant' && last.streaming) last.text += event.text;
      else view.items.push({ kind: 'assistant', id: nextId(view), text: event.text, streaming: true });
      break;
    }
    case 'tool_start':
      view.status = '';
      endStreaming(view);
      view.items.push({ kind: 'tool', id: event.id, name: event.name, arguments: event.arguments, output: '', status: 'running' });
      break;
    case 'tool_dispatched':
      // The existing tool card remains running; dispatch is an execution-log boundary.
      break;
    case 'tool_end': {
      const item = view.items.find((i): i is Extract<ChatItem, { kind: 'tool' }> => i.kind === 'tool' && i.id === event.id);
      const output = toolOutput(event.output, event.structured_content);
      if (item) { item.output = output; item.status = event.is_error ? 'error' : 'done'; }
      else view.items.push({ kind: 'tool', id: event.id, name: '', arguments: '', output, status: event.is_error ? 'error' : 'done' });
      break;
    }
    case 'retrying':
      view.status = `连接不太稳，正在第 ${event.attempt} 次重试…`;
      break;
    case 'truncated':
      endStreaming(view);
      view.items.push({ kind: 'notice', id: nextId(view), tone: 'warning', text: '回答太长，被截断了。', canContinue: true });
      break;
    case 'continuing':
      // The next reply is a new message, not more of the one that just stopped.
      endStreaming(view);
      view.status = '还没做完，小伴继续处理中…';
      break;
    case 'done':
      settle(view);
      break;
    case 'cancelled':
      settle(view);
      view.items.push({ kind: 'notice', id: nextId(view), tone: 'info', text: '已停止' });
      break;
    case 'error': {
      const lastUser = view.items.map(i => i.kind).lastIndexOf('user');
      const partial = view.items.slice(lastUser + 1).some(i => i.kind === 'tool' || i.kind === 'assistant');
      settle(view);
      if (/^run budget exceeded:/.test(event.message)) {
        view.items.push({ kind: 'notice', id: nextId(view), tone: 'warning', text: '本轮操作次数已达到上限，已暂停。点“继续”可以从已保存的进展接着处理。', canContinue: true, details: event.message });
        break;
      }
      if (/^run paused by continuation guard:/.test(event.message)) {
        const text = event.message.endsWith(': no_progress')
          ? '连续多次未取得新进展，小伴已暂停。点“继续”可以接着尝试。'
          : '已达到自动续跑次数上限，小伴已暂停。点“继续”可以接着做。';
        view.items.push({ kind: 'notice', id: nextId(view), tone: 'warning', text, canContinue: true, details: event.message });
        break;
      }
      if (TURN_LIMIT.test(event.message)) {
        // Not a model failure: the work so far is saved and can simply go on.
        view.items.push({ kind: 'notice', id: nextId(view), tone: 'warning', text: '这一轮步骤太多，先停在这里。点“继续”让小伴接着做。',
          canContinue: true, details: event.message });
        break;
      }
      view.items.push({ kind: 'notice', id: nextId(view), tone: 'error', text: modelFailure(event.message).message,
        canSwitchModel: true, canRetry: lastUser >= 0 && !partial, details: event.message });
      break;
    }
    case 'usage':
      break;
  }
}

/** A short label for a tool call's arguments, shown on the collapsed card. */
export function describeCall(name: string, argumentsJson: string): string {
  let args: Record<string, unknown> = {};
  try { const parsed = JSON.parse(argumentsJson); if (parsed && typeof parsed === 'object') args = parsed as Record<string, unknown>; } catch { /* half-written arguments */ }
  const pick = (...keys: string[]) => keys.map(key => args[key]).find((value): value is string => typeof value === 'string' && value.trim() !== '');
  const pack = args.pack && typeof args.pack === 'object' ? args.pack as Record<string, unknown> : {};
  const packName = typeof pack.name === 'string' ? pack.name : typeof pack.id === 'string' ? pack.id : '';
  const text = name === 'shell' ? pick('command')
    : name === 'effect_create' ? packName
    : name === 'community_share' ? pick('id')
    : name === 'community_open' ? pick('target')
    : pick('command', 'path', 'url', 'query', 'pattern', 'kind', 'title', 'name', 'id', 'text', 'question', 'expression', 'skin', 'theme', 'gameId');
  const line = (text ?? '').split('\n')[0].trim();
  return line.length > 90 ? `${line.slice(0, 90)}…` : line;
}

