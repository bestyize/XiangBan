import { modelFailure } from './model-errors.ts';
import { reactive } from 'vue';
import type { ApprovalDecision, ApprovalRequest, ChatEventPayload, Overview, QuestionRequest, SessionSummary, SessionView, WireMessage } from './types.ts';
import { applyEvent, itemsFromHistory, newView, nextId } from './view.ts';

export interface ChatDeps {
  onReply?(session: string, text: string, key?: string): void;
  onActivity?(): void;
  desktop: boolean;
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  listen<T>(event: string, callback: (payload: T) => void): Promise<() => void>;
  notify(message: unknown, type?: 'success' | 'error' | 'info'): void;
}

export interface ChatStore {
  /** The first overview has been read (or there is no desktop to ask). */
  ready: boolean;
  /** A model address and name are saved, so a message can be sent. */
  configured: boolean;
  /** The folder the assistant's file and command tools work in. */
  workspace: string;
  permissionMode: 'ask' | 'auto';
  setPermissionMode(mode: 'ask' | 'auto'): Promise<void>;
  hideToolCalls: boolean;
  loadError: string;
  sessions: SessionSummary[];
  views: Record<string, SessionView>;
  /** `null` is a new conversation that has not been started yet. */
  currentId: string | null;
  /** How many confirmations and questions wait for the user, in every conversation. */
  readonly attention: number;
  init(): Promise<void>;
  /** Reads the list again, for example after the settings changed. */
  refresh(): Promise<void>;
  dispose(): void;
  select(id: string | null): Promise<void>;
  /** Returns false when nothing was sent, so the caller can give the text back to the user. */
  send(text: string, images?: string[]): Promise<boolean>;
  retry(): Promise<boolean>;
  cancel(): Promise<void>;
  remove(id: string): Promise<void>;
  answerApproval(request: ApprovalRequest, decision: ApprovalDecision): Promise<void>;
  answerQuestion(request: QuestionRequest, answer: string | null): Promise<void>;
  current(): SessionView | null;
}

export const CONTINUE_TEXT = '请接着上面的内容继续写完。';
const STALE = '这个确认已经结束';

export function createChatStore(deps: ChatDeps): ChatStore {
  const unsubscribers: (() => void)[] = [];
  let starting: Promise<void> | null = null;
  let disposed = false;
  let refreshedAfterErrorAt = 0;

  const store: ChatStore = reactive({
    ready: false,
    configured: false,
    workspace: '',
    permissionMode: 'ask' as 'ask' | 'auto',
    hideToolCalls: false,
    async setPermissionMode(mode: 'ask' | 'auto') {
      store.permissionMode = mode;
      if (store.currentId) {
        const s = store.sessions.find(item => item.id === store.currentId);
        if (s) s.permissionMode = mode;
      }
      if (!deps.desktop) return;
      try {
        const args = store.currentId ? { sessionId: store.currentId, mode } : { mode };
        await deps.invoke('chat_set_permission_mode', args);
      } catch (error) {
        deps.notify(error, 'error');
      }
    },
    loadError: '',
    sessions: [] as SessionSummary[],
    views: {} as Record<string, SessionView>,
    currentId: null as string | null,
    get attention() {
      return Object.values(store.views).reduce((sum, view) => sum + view.approvals.length + view.questions.length, 0);
    },

    init() {
      starting ??= (async () => {
        if (!deps.desktop) { store.ready = true; return; }
        try {
          const listeners = await Promise.all([
            deps.listen<ChatEventPayload>('chat-event', ({ session, event }) => { if (!disposed) {
              const view = viewOf(session, true);
              applyEvent(view, event);
              if (event.type === 'done' && store.currentId === session) {
                const last = view.items.at(-1);
                if (last?.kind === 'assistant' && last.text.trim()) deps.onReply?.(session, last.text, last.id);
              }
              if ((event.type === 'cancelled' || event.type === 'error') && store.currentId === session) deps.onActivity?.();
              if (event.type === 'error' && modelFailure(event.message).refresh && Date.now() - refreshedAfterErrorAt > 300000) {
                refreshedAfterErrorAt = Date.now();
                void deps.invoke('refresh_ai_providers', { force: true }).catch(() => {});
              }
            } }),
            deps.listen<ApprovalRequest>('chat-approval', request => { if (!disposed) addPending(viewOf(request.session, true).approvals, request); }),
            deps.listen<QuestionRequest>('chat-question', request => { if (!disposed) addPending(viewOf(request.session, true).questions, request); }),
            deps.listen<SessionSummary[]>('chat-sessions-changed', sessions => { if (!disposed) store.sessions = sessions; }),
            deps.listen<{ sessionId?: string; mode: 'ask' | 'auto' }>('chat-permission-mode-changed', payload => {
              if (!disposed) {
                if (payload.sessionId) {
                  const s = store.sessions.find(item => item.id === payload.sessionId);
                  if (s) s.permissionMode = payload.mode;
                  if (store.currentId === payload.sessionId) {
                    store.permissionMode = payload.mode;
                  }
                } else {
                  store.permissionMode = payload.mode;
                }
              }
            }),
            deps.listen<{ enabled: boolean }>('chat-hide-tool-calls-changed', payload => { if (!disposed) store.hideToolCalls = payload.enabled; }),
          ]);
          if (disposed) { listeners.forEach(stop => stop()); return; }
          unsubscribers.push(...listeners);
        } catch (error) {
          store.loadError = String(error);
        }
        await store.refresh();
      })();
      return starting;
    },

    async refresh() {
      if (!deps.desktop) return;
      try {
        const overview = await deps.invoke<Overview | undefined>('chat_overview');
        if (disposed || !overview) return;
        store.configured = overview.configured;
        store.workspace = overview.workspace;
        if (overview.permissionMode) store.permissionMode = overview.permissionMode;
        if (typeof overview.hideToolCalls === 'boolean') store.hideToolCalls = overview.hideToolCalls;
        store.sessions = overview.sessions;
        for (const id of overview.running) viewOf(id, true).running = true;
        store.loadError = '';
      } catch (error) {
        store.loadError = String(error instanceof Error ? error.message : error);
      } finally {
        if (!disposed) store.ready = true;
      }
    },

    dispose() {
      disposed = true;
      unsubscribers.splice(0).forEach(stop => stop());
    },

    current() {
      return store.currentId ? store.views[store.currentId] ?? null : null;
    },

    async select(id) {
      deps.onActivity?.();
      store.currentId = id;
      if (!id) return;
      const curSession = store.sessions.find(s => s.id === id);
      if (curSession?.permissionMode) {
        store.permissionMode = curSession.permissionMode;
      }
      const view = viewOf(id, false);
      if (view.loaded) return;
      try {
        const messages = await deps.invoke<WireMessage[]>('chat_load_session', { sessionId: id });
        if (view.loaded) return;
        view.items = itemsFromHistory(view, messages ?? []);
        view.loaded = true;
      } catch (error) {
        deps.notify(error, 'error');
      }
    },

    async send(text, images = []) {
      deps.onActivity?.();
      if (!deps.desktop) { deps.notify('请在桌面应用中使用聊天', 'error'); return false; }
      let id = store.currentId;
      let created = false;
      try {
        if (!id) {
          // The host runs each conversation in the mode saved with it, so a new one must be created in the
          // mode the top bar shows now; otherwise "完全信任" would be displayed but not applied.
          const summary = await deps.invoke<SessionSummary>('chat_new_session', { permissionMode: store.permissionMode });
          id = summary.id;
          created = true;
          upsert(summary);
          store.views[id] = newView(id, true);
          store.currentId = id;
        }
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      }
      const view = viewOf(id, true);
      if (view.running) return false;
      const item = { kind: 'user' as const, id: nextId(view), text, images: [...images] };
      view.items.push(item);
      view.running = true;
      view.status = '';
      try {
        // Images are left out entirely when there are none, so a plain message keeps its old shape.
        const args = images.length ? { sessionId: id, text, images } : { sessionId: id, text };
        upsert(await deps.invoke<SessionSummary>('chat_send', args));
        return true;
      } catch (error) {
        // The host refused before starting, so nothing of this message happened.
        view.items = view.items.filter(i => i.id !== item.id);
        view.running = false;
        deps.notify(error, 'error');
        if (created && !view.items.length) {
          const sessionId = id;
          void deps.invoke<SessionSummary[]>('chat_delete_session', { sessionId }).then(sessions => { if (!disposed && sessions) store.sessions = sessions; }).catch(() => {});
          delete store.views[sessionId];
          store.currentId = null;
        }
        return false;
      }
    },

    async retry() {
      deps.onActivity?.();
      const id = store.currentId, view = store.current();
      if (!id || !view || view.running) return false;
      view.running = true; view.status = '';
      const previousFailure = [...view.items].reverse().find(i => i.kind === 'notice' && i.canRetry)?.id;
      try {
        upsert(await deps.invoke<SessionSummary>('chat_send', { sessionId: id, text: '', retry: true }));
        // Keep the original user message and drop only the now-resolved failure notice.
        view.items = view.items.filter(i => i.id !== previousFailure);
        return true;
      } catch (e) { view.running = false; deps.notify(e, 'error'); return false; }
    },

    async cancel() {
      deps.onActivity?.();
      const id = store.currentId;
      if (id) await deps.invoke('chat_cancel', { sessionId: id }).catch(error => deps.notify(error, 'error'));
    },

    async remove(id) {
      if (id === store.currentId) deps.onActivity?.();
      try {
        const sessions = await deps.invoke<SessionSummary[]>('chat_delete_session', { sessionId: id });
        store.sessions = sessions ?? store.sessions.filter(s => s.id !== id);
        delete store.views[id];
        if (store.currentId === id) store.currentId = null;
      } catch (error) {
        deps.notify(error, 'error');
      }
    },

    async answerApproval(request, decision) {
      const view = store.views[request.session];
      if (view) view.approvals = view.approvals.filter(a => a.id !== request.id);
      try {
        await deps.invoke('chat_answer_approval', { id: request.id, decision });
      } catch (error) {
        if (!String(error).includes(STALE)) deps.notify(error, 'error');
      }
    },

    async answerQuestion(request, answer) {
      const view = store.views[request.session];
      if (view) view.questions = view.questions.filter(q => q.id !== request.id);
      try {
        await deps.invoke('chat_answer_question', { id: request.id, answer });
      } catch (error) {
        if (!String(error).includes('这个问题已经结束')) deps.notify(error, 'error');
      }
    },
  }) as ChatStore;

  /** The view of a conversation; one made for a live event counts as loaded because its history is the event stream. */
  function viewOf(id: string, loaded: boolean): SessionView {
    // Read it back after storing it: the value of an assignment is the plain object, and changes to
    // that would never reach the page. Only what comes out of the reactive store is watched.
    if (!store.views[id]) store.views[id] = newView(id, loaded);
    return store.views[id];
  }

  function upsert(summary: SessionSummary) {
    store.sessions = [summary, ...store.sessions.filter(s => s.id !== summary.id)];
  }

  function addPending<T extends { id: string }>(list: T[], request: T) {
    if (!list.some(item => item.id === request.id)) list.push(request);
  }

  return store;
}
