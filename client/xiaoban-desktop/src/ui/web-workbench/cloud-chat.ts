import { reactive } from 'vue';
import { createChatStore, type ChatDeps, type ChatStore } from '../chat/chat-store.ts';
import type { ApprovalRequest, KernelEvent, SessionSummary } from '../chat/types.ts';
import { applyEvent, newView } from '../chat/view.ts';

export interface WebTier { id: string; label: string; default?: boolean }
export interface WebWorkbenchState { tiers: WebTier[]; tier: string; connected: boolean; sending: boolean; connectionError: string }
export type CloudRequest = (path: string, init: RequestInit) => Promise<Response>;
interface WebHost { owner: string; request: CloudRequest; notify: ChatDeps['notify'] }
type CloudEvent = (KernelEvent & {requestId?: string}) | { type: 'approval_requested'; request: ApprovalRequest } | { type: 'approval_resolved'; id: string };
type Envelope = { runId: string; event: CloudEvent };

class HttpError extends Error {
  constructor(readonly status: number, message: string) { super(message); }
}

/** The server sends one JSON data line per event. Discard an incomplete final frame on reconnect. */
async function readStream(body: ReadableStream<Uint8Array>, frame: (id: string, name: string, data: string) => void) {
  const reader = body.getReader();
  const decoder = new TextDecoder();
  let buffer = '', id = '', name = '', data = '';
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      buffer += decoder.decode(part.value, { stream: true });
      let end;
      while ((end = buffer.indexOf('\n')) >= 0) {
        const line = buffer.slice(0, end).replace(/\r$/, '');
        buffer = buffer.slice(end + 1);
        if (line === '') {
          if (data) frame(id, name, data);
          id = name = data = '';
        } else if (line.startsWith('id:')) id = line.slice(3).trim();
        else if (line.startsWith('event:')) name = line.slice(6).trim();
        else if (line.startsWith('data:')) data += (data ? '\n' : '') + line.slice(5).trimStart();
      }
    }
  } finally { reader.releaseLock(); }
}

export function createCloudChat(host: WebHost) {
  const state = reactive<WebWorkbenchState>({ tiers: [], tier: '', connected: false, sending: false, connectionError: '' });
  // Drafts, message rendering, transcript actions and per-session views are shared with the native client.
  const chat = createChatStore({ desktop: false, notify: host.notify,
    invoke: async () => { throw Error('当前操作需要桌面客户端'); }, listen: async () => () => {} });
  const originalDispose = chat.dispose;
  const lifetime = new AbortController();
  const cursors = new Map<string, string>();
  const currentRuns = new Map<string, string>();
  const terminal = new Map<string, string>();
  const pending = new Map<string, { text: string; tier: string; requestId: string; observed: boolean }>();
  const storageKey = 'xiaoban.workbench.session.' + host.owner;
  let stream: AbortController | undefined, initializing: Promise<void> | undefined;
  let refreshSequence = 0, selection = 0;
  const active = () => !lifetime.signal.aborted;

  async function response(path: string, method: string, body?: unknown, signal = lifetime.signal) {
    const res = await host.request(path, { method, signal, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    if (!res.ok) {
      const error = await res.json().catch(() => null);
      throw new HttpError(res.status, error?.message || (res.status === 401 ? '请重新登录小伴' : '工作台暂时无法连接'));
    }
    return res;
  }
  async function json<T>(path: string, method = 'GET', body?: unknown): Promise<T> {
    return (await response(path, method, body)).json();
  }
  function update(summary: SessionSummary) {
    refreshSequence++;
    chat.sessions = [summary, ...chat.sessions.filter(s => s.id !== summary.id)];
  }
  function fold(id: string, { runId, event }: Envelope) {
    const view = chat.views[id];
    if (event.type === 'approval_requested') view.approvals.push(event.request);
    else if (event.type === 'approval_resolved') view.approvals = view.approvals.filter(a => a.id !== event.id);
    else {
      applyEvent(view, event);
      if (event.type === 'user_message') {
        currentRuns.set(id, runId);
        const attempt = pending.get(id);
        if (attempt && attempt.requestId === event.requestId) attempt.observed = true;
      }
      if (['done', 'cancelled', 'error'].includes(event.type)) {
        currentRuns.delete(id);
        terminal.set(id, runId);
        view.approvals = [];
        view.hostBusy = false;
      }
    }
  }
  async function observe(id: string, signal: AbortSignal) {
    let delay = 300;
    while (!signal.aborted) {
      try {
        const res = await response('/sessions/' + id + '/events?after=' + (cursors.get(id) ?? '0'), 'GET', undefined, signal);
        if (!res.body || !res.headers.get('content-type')?.includes('text/event-stream')) throw Error('无法读取对话事件');
        await readStream(res.body, (seq, name, data) => {
          if (signal.aborted) return;
          if (name === 'ready') {
            chat.views[id].loaded = true; state.connected = true; state.connectionError = ''; delay = 300;
          } else if (/^\d+$/.test(seq) && BigInt(seq) > BigInt(cursors.get(id) ?? 0)) {
            fold(id, JSON.parse(data)); cursors.set(id, seq);
          }
        });
      } catch (error) {
        if (signal.aborted) return;
        state.connected = false;
        state.connectionError = error instanceof Error ? error.message : '正在重新连接';
        if (error instanceof HttpError && [401, 403, 404].includes(error.status)) { void chat.refresh(); return; }
        await new Promise<void>(resolve => {
          const timer = setTimeout(done, delay);
          function done() { clearTimeout(timer); signal.removeEventListener('abort', done); resolve(); }
          signal.addEventListener('abort', done, { once: true });
        });
        delay = Math.min(5000, delay * 2);
      }
    }
  }
  function choose(id: string | null) {
    selection++;
    stream?.abort();
    chat.currentId = id;
    try { sessionStorage.setItem(storageKey, id ?? ''); } catch { /* optional navigation preference */ }
    state.connectionError = ''; state.connected = !id;
    if (!id) return;
    chat.views[id] ??= newView(id);
    stream = new AbortController();
    void observe(id, stream.signal);
  }

  const operations: Partial<ChatStore> = {
    init() {
      return initializing ??= (async () => {
        const revision = selection;
        await chat.refresh();
        if (!active() || revision !== selection) return;
        let saved: string | null = null;
        try { saved = sessionStorage.getItem(storageKey); } catch { /* private browsing */ }
        choose(saved === '' ? null : chat.sessions.find(s => s.id === saved)?.id ?? chat.sessions[0]?.id ?? null);
      })();
    },
    async refresh() {
      const seq = ++refreshSequence;
      try {
        const rows = await json<SessionSummary[]>('/sessions');
        if (!active() || seq !== refreshSequence) return;
        chat.sessions = rows; chat.loadError = '';
        if (chat.currentId && !rows.some(s => s.id === chat.currentId)) choose(null);
      } catch (error) { if (active()) chat.loadError = error instanceof Error ? error.message : String(error); }
      finally { if (active()) chat.ready = true; }
    },
    async select(id) { choose(id); },
    async send(input) {
      const text = input.trim(), tier = state.tier;
      if (!text || !tier || state.sending || chat.current()?.running || !state.connected) return false;
      const id = chat.currentId || crypto.randomUUID(), revision = selection;
      const draft = chat.draft;
      const attempt = pending.get(id);
      const requestId = attempt?.text === text && attempt.tier === tier ? attempt.requestId : crypto.randomUUID();
      state.sending = true;
      try {
        if (!chat.currentId) {
          const summary = await json<SessionSummary>('/sessions', 'POST', { id });
          if (!active()) return false;
          update(summary); chat.drafts[id] = draft; delete chat.drafts.$new;
          if (revision === selection) choose(id);
        }
        const submission = { text, tier, requestId, observed: false };
        pending.set(id, submission);
        const submit = () => json<{session: SessionSummary; runId: string}>('/sessions/' + id + '/messages', 'POST', { requestId, text, tier });
        let result;
        try { result = await submit(); }
        catch (error) {
          // A network failure can lose the acknowledgement after commit. Reuse the original request id.
          if (!active() || (error instanceof HttpError && error.status < 500)) throw error;
          result = await submit();
        }
        if (!active()) return false;
        pending.delete(id); update(result.session);
        if (!submission.observed && terminal.get(id) !== result.runId) {
          currentRuns.set(id, result.runId);
          if (chat.views[id]) chat.views[id].running = true;
        }
        return true;
      } catch (error) {
        if (pending.get(id)?.observed) { pending.delete(id); if (active()) void chat.refresh(); return true; }
        if (active()) host.notify(error, 'error');
        return false;
      }
      finally { state.sending = false; }
    },
    async cancel() {
      const id = chat.currentId, run = id && currentRuns.get(id);
      if (!run) return;
      try { await json('/sessions/' + id + '/runs/' + run + '/cancel', 'POST'); }
      catch (error) { if (active()) host.notify(error, 'error'); }
    },
    async answerApproval(request, decision) {
      const run = currentRuns.get(request.session);
      if (!run || decision === 'always') return;
      try { await json('/sessions/' + request.session + '/runs/' + run + '/approvals/' + request.id, 'POST', { decision }); }
      catch (error) { if (active()) host.notify(error, 'error'); }
    },
    async rename(id, title) {
      try { const row = await json<SessionSummary>('/sessions/' + id, 'PATCH', { title }); if (active()) update(row); return true; }
      catch (error) { if (active()) host.notify(error, 'error'); return false; }
    },
    async remove(id) {
      try {
        await json('/sessions/' + id, 'DELETE');
        if (!active()) return;
        if (chat.currentId === id) choose(null);
        chat.sessions = chat.sessions.filter(s => s.id !== id);
        delete chat.views[id]; delete chat.drafts[id]; currentRuns.delete(id); terminal.delete(id); cursors.delete(id); pending.delete(id);
      } catch (error) { if (active()) host.notify(error, 'error'); }
    },
    dispose() { lifetime.abort(); stream?.abort(); originalDispose(); },
  };
  Object.assign(chat, operations);
  return { chat, state, reconnect: () => choose(chat.currentId) };
}
