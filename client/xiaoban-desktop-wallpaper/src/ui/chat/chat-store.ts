import type { PendingAttachment } from './attachments.ts';
import type { Mention } from './mentions.ts';
import { modelFailure } from './model-errors.ts';
import { reactive } from 'vue';
import type { ApprovalDecision, ApprovalRequest, ChatEventPayload, ChatGroup, ChatItem, ContextInfo, ExpertView, Library, Overview, QuestionRequest, RecentWorkspace, SessionSummary, SessionView, WireMessage, WorktreeInfo, WorktreeLink } from './types.ts';
import { applyEvent, checkpointNotice, interruptedNotice, itemsFromHistory, newView, nextId, summaryNotice, workspaceNotice } from './view.ts';
import { toolLabel } from './format.ts';

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
  /** The default working folder (the setting): old conversations work there, and the first new one starts there. */
  workspace: string;
  /** Folders used lately, newest first, for the folder menu. */
  recentWorkspaces: RecentWorkspace[];
  /** The folder chosen for the new conversation the page shows, before its first message; empty when none was. */
  pendingWorkspace: string;
  /** The expert summoned for the new conversation the page shows, before its first message; `null` when none was. */
  pendingExpert: ExpertView | null;
  /** The expert the open conversation is bound to, or the one summoned for the new one. */
  readonly currentExpert: ExpertView | null;
  /** The folder the open conversation works in, or a new one would. */
  readonly currentWorkspace: string;
  /** Moves the open conversation to `path`. A conversation with messages is not moved unless `how` says how:
   *  `new` starts a new conversation there, `here` moves this one and tells the model. Returns `ask` when the
   *  page should ask which, `failed` when the host refused (the reason is shown). */
  switchWorkspace(path: string, how?: 'new' | 'here'): Promise<'done' | 'ask' | 'failed'>;
  /** Moves any saved conversation (not only the open one) to `path`, as the sidebar's “移动到工作区” does. The host tells
   *  the model about the new folder; an open, loaded conversation also shows the notice. Returns whether it was moved. */
  moveToWorkspace(id: string, path: string): Promise<boolean>;
  /** The system folder picker; `null` when nothing was chosen. */
  pickWorkspace(): Promise<string | null>;
  /** Shows the open conversation's folder in Finder or Explorer. */
  revealWorkspace(): Promise<void>;
  /** The isolated copy of the folder the open conversation works in, if it is in one. */
  readonly currentWorktree: WorktreeLink | null;
  /** How the open conversation stands with its copy; `null` when there is nothing to ask (no desktop, no saved conversation). */
  worktreeInfo(): Promise<WorktreeInfo | null>;
  /** Moves the open conversation into a new isolated copy of its folder. Returns whether it worked (the reason is shown if not). */
  startWorktree(): Promise<boolean>;
  /** Ends the copy: `merge` brings its changes back into the original folder, `keep` leaves it on disk, `discard` deletes it. */
  finishWorktree(action: 'merge' | 'keep' | 'discard'): Promise<boolean>;
  permissionMode: 'ask' | 'auto';
  setPermissionMode(mode: 'ask' | 'auto'): Promise<void>;
  hideToolCalls: boolean;
  loadError: string;
  sessions: SessionSummary[];
  groups: ChatGroup[];
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
  /**
   * Returns false when nothing was sent, so the caller can give the text back to the user.
   * `note` goes to the model at the front of the message and is not shown (see `voice/interruption.ts`).
   */
  send(text: string, attachments?: PendingAttachment[], note?: string, mentions?: Mention[]): Promise<boolean>;
  /**
   * Whether this window sent the question being answered in the conversation. Both windows hear every chat event,
   * and only the one the user spoke or typed into should read the answer aloud.
   */
  sentHere(session: string): boolean;
  retry(): Promise<boolean>;
  cancel(): Promise<void>;
  remove(id: string): Promise<void>;
  /** The size of the next request in the open conversation (or of a new one), against the limit chosen in settings. */
  context: ContextInfo;
  /** The conversation that is being compacted right now, if any. */
  compactingId: string | null;
  /** Reads `context` again. Quiet on failure: the ring keeps its last numbers. */
  refreshContext(): Promise<void>;
  /** Replaces the older part of the open conversation, for the model only, with a summary. Returns whether it was done; the reason is shown when not. */
  compact(): Promise<boolean>;
  /** Saves the context limit the slider chose. The ring shows it at once; it is put back if the host refuses. */
  setContextSize(tokens: number): Promise<boolean>;
  /** The edits below return whether the host accepted them, so a dialog can stay open when it did not. The host's reason is shown to the user. */
  rename(id: string, title: string): Promise<boolean>;
  /** `null` takes the conversation out of its group. */
  move(id: string, groupId: string | null): Promise<boolean>;
  /** With `moveId`, that conversation goes into the new group in the same step. */
  createGroup(name: string, moveId?: string): Promise<boolean>;
  renameGroup(id: string, name: string): Promise<boolean>;
  /** The conversations in the group are kept. */
  removeGroup(id: string): Promise<boolean>;
  answerApproval(request: ApprovalRequest, decision: ApprovalDecision): Promise<void>;
  answerQuestion(request: QuestionRequest, answer: string | null): Promise<void>;
  /** Puts back the files the answer behind a checkpoint notice changed. Returns whether it was done. */
  undoRun(session: string, itemId: string): Promise<boolean>;
  current(): SessionView | null;
}

/** Sent by the host around an automatic compaction (`chat-compacting`). */
interface CompactingEvent { session: string; state: 'start' | 'done' | 'failed'; summary?: string; message?: string }

export const CONTINUE_TEXT = '请接着上面的内容继续写完。';
const STALE = '这个确认已经结束';

export function createChatStore(deps: ChatDeps): ChatStore {
  const unsubscribers: (() => void)[] = [];
  let starting: Promise<void> | null = null;
  let disposed = false;
  const interruptions: Record<string, ChatItem> = {};
  let refreshedAfterErrorAt = 0;
  let supportsSnapshots = false;
  let supportsPending = false;
  const revisions = new Map<string,number>();
  interface PendingSnapshot {revision:number;approvals:ApprovalRequest[];questions:QuestionRequest[]}
  const pendingEvents: Record<string, ChatEventPayload[]> = {};
  const loading = new Map<string, Promise<void>>();
  const activeRun = new Map<string, string>();
  const cursors = new Map<string, number>();
  interface Outcome {runId:string;status:string;validator:string;evidence:string}
  interface Snapshot { outcome?:Outcome; pending?: PendingSnapshot; messages: WireMessage[]; runId?: string; sequence?: number; events?: ChatEventPayload[]; running?: boolean; ready?: boolean; warning?: string; interrupted?: boolean }


  const store: ChatStore = reactive({
    ready: false,
    configured: false,
    workspace: '',
    recentWorkspaces: [] as RecentWorkspace[],
    pendingWorkspace: '',
    pendingExpert: null as ExpertView | null,
    get currentExpert() {
      const bound = store.currentId ? store.sessions.find(s => s.id === store.currentId)?.expert : null;
      if (bound) return { plugin: bound.plugin, name: bound.name, profession: bound.profession, agent: bound.agent, hasAvatar: true };
      return store.currentId ? null : store.pendingExpert;
    },
    get currentWorkspace() {
      if (store.currentId) return store.sessions.find(s => s.id === store.currentId)?.workspace || store.workspace;
      return store.pendingWorkspace || store.recentWorkspaces.find(w => w.exists)?.path || store.workspace;
    },
    async switchWorkspace(path: string, how?: 'new' | 'here') {
      deps.onActivity?.();
      const id = store.currentId;
      const view = id ? store.views[id] : null;
      // No conversation yet: the folder is used when the first message creates it.
      if (!id) { store.pendingWorkspace = path; void store.refreshContext(); return 'done'; }
      if (view?.running) { deps.notify('这个对话还在回答，请等它结束或先停止，再切换工作区', 'error'); return 'failed'; }
      const used = !!view?.items.length;
      if (used && !how) return 'ask';
      try {
        if (used && how === 'new') {
          const summary = await deps.invoke<SessionSummary>('chat_new_session', { permissionMode: store.permissionMode, workspace: path });
          upsert(summary);
          store.views[summary.id] = newView(summary.id, true);
          store.currentId = summary.id;
          store.pendingWorkspace = '';
          void store.refreshContext();
          return 'done';
        }
        const result = await deps.invoke<{ session: SessionSummary; wide: boolean; recentWorkspaces: RecentWorkspace[] }>('chat_set_workspace', { sessionId: id, workspace: path });
        store.sessions = store.sessions.map(s => s.id === result.session.id ? result.session : s);
        store.recentWorkspaces = result.recentWorkspaces ?? store.recentWorkspaces;
        if (used && view) view.items.push(workspaceNotice(view, path));
        if (result.wide) deps.notify('这个文件夹范围很大（主目录或磁盘根目录），小伴在里面删除文件前都会先问你', 'info');
        void store.refreshContext();
        return 'done';
      } catch (error) {
        deps.notify(error, 'error');
        return 'failed';
      }
    },
    async moveToWorkspace(id: string, path: string) {
      if (!deps.desktop) return false;
      const view = store.views[id];
      if (view?.running) { deps.notify('这个对话还在回答，请等它结束或先停止，再切换工作区', 'error'); return false; }
      const before = store.sessions.find(s => s.id === id)?.workspace;
      if (before && before === path) return true;
      try {
        const result = await deps.invoke<{ session: SessionSummary; wide: boolean; recentWorkspaces: RecentWorkspace[] }>('chat_set_workspace', { sessionId: id, workspace: path });
        if (disposed) return false;
        store.sessions = store.sessions.map(s => s.id === result.session.id ? result.session : s);
        store.recentWorkspaces = result.recentWorkspaces ?? store.recentWorkspaces;
        if (view?.items.length) view.items.push(workspaceNotice(view, path));
        if (result.wide) deps.notify('这个文件夹范围很大（主目录或磁盘根目录），小伴在里面删除文件前都会先问你', 'info');
        if (id === store.currentId) void store.refreshContext();
        return true;
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      }
    },
    async pickWorkspace() {
      if (!deps.desktop) return null;
      try {
        return await deps.invoke<string | null>('pick_workspace') ?? null;
      } catch (error) {
        deps.notify(error, 'error');
        return null;
      }
    },
    get currentWorktree() {
      return store.currentId ? store.sessions.find(s => s.id === store.currentId)?.worktree ?? null : null;
    },
    async worktreeInfo() {
      const id = store.currentId;
      if (!deps.desktop || !id) return null;
      try {
        return await deps.invoke<WorktreeInfo>('chat_worktree_status', { sessionId: id });
      } catch (error) {
        deps.notify(error, 'error');
        return null;
      }
    },
    async startWorktree() {
      const id = store.currentId;
      if (!deps.desktop || !id) { deps.notify('先发一条消息开始对话，再切换到独立副本', 'info'); return false; }
      if (store.views[id]?.running) { deps.notify('这个对话还在回答，请等它结束或先停止', 'error'); return false; }
      try {
        const result = await deps.invoke<{ session: SessionSummary }>('chat_worktree_start', { sessionId: id });
        if (disposed) return false;
        store.sessions = store.sessions.map(s => s.id === result.session.id ? result.session : s);
        const view = store.views[id];
        if (view?.items.length) view.items.push({ kind: 'notice', id: nextId(view), tone: 'info', text: `已切换到独立副本（分支 ${result.session.worktree?.branch ?? ''}）。之后的改动先留在副本里，不会影响原文件夹，直到你选择合并。` });
        if (id === store.currentId) void store.refreshContext();
        return true;
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      }
    },
    async finishWorktree(action: 'merge' | 'keep' | 'discard') {
      const id = store.currentId;
      if (!deps.desktop || !id) return false;
      if (store.views[id]?.running) { deps.notify('这个对话还在回答，请等它结束或先停止', 'error'); return false; }
      try {
        const result = await deps.invoke<{ session: SessionSummary; message: string }>('chat_worktree_finish', { sessionId: id, action });
        if (disposed) return false;
        store.sessions = store.sessions.map(s => s.id === result.session.id ? result.session : s);
        const view = store.views[id];
        if (view?.items.length) view.items.push({ kind: 'notice', id: nextId(view), tone: 'info', text: `${result.message}。对话回到原文件夹继续。` });
        deps.notify(result.message, 'success');
        if (id === store.currentId) void store.refreshContext();
        return true;
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      }
    },
    async revealWorkspace() {
      try {
        await deps.invoke('chat_reveal_workspace', store.currentId ? { sessionId: store.currentId } : {});
      } catch (error) {
        deps.notify(error, 'error');
      }
    },
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
    groups: [] as ChatGroup[],
    context: { used: 0, limit: 256_000, canCompact: false } as ContextInfo,
    compactingId: null as string | null,
    views: {} as Record<string, SessionView>,
    currentId: null as string | null,
    sentHere(session) { return sent.has(session); },
    get attention() {
      return Object.values(store.views).reduce((sum, view) => sum + view.approvals.length + view.questions.length, 0);
    },

    init() {
      starting ??= (async () => {
        if (!deps.desktop) { store.ready = true; return; }
        try {
          const listeners = await Promise.all([
            deps.listen<ChatEventPayload>('chat-event', payload => { if (!disposed) receive(payload); }),
            deps.listen<ApprovalRequest>('chat-approval', request => { if (!disposed && !supportsPending) addPending(viewOf(request.session, true).approvals, request); }),
            deps.listen<QuestionRequest>('chat-question', request => { if (!disposed && !supportsPending) addPending(viewOf(request.session, true).questions, request); }),
            deps.listen<{session:string;pending:PendingSnapshot}>('chat-pending-changed', p=>{if(!disposed) applyPending(p.session,p.pending);}),
            deps.listen<{session:string;outcome:Outcome}>('chat-task-outcome',p=>{if(!disposed)applyOutcome(p.session,p.outcome);}),
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
            deps.listen<CompactingEvent>('chat-compacting', payload => { if (!disposed) onCompacting(payload); }),
            // Sent while the conversation loads, before its history replaces the items: kept until then.
            deps.listen<{ session: string; runId: string; tools: string[] }>('chat-run-interrupted', ({ session, tools }) => {
              if (disposed) return;
              const view = viewOf(session, true);
              const notice = interruptedNotice(view, tools ?? [], toolLabel);
              if (view.loaded) view.items.push(notice); else interruptions[session] = notice;
            }),
            deps.listen<{ session: string; runId: string; files: string[] }>('chat-checkpoint', ({ session, runId, files }) => {
              if (disposed || !files.length) return;
              const view = viewOf(session, true);
              view.items.push(checkpointNotice(view, runId, files));
            }),
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
        supportsSnapshots = overview.snapshotProtocol === 1;
      supportsPending = overview.pendingProtocol === 1;
        store.configured = overview.configured;
        store.workspace = overview.workspace;
        store.recentWorkspaces = overview.recentWorkspaces ?? [];
        if (overview.permissionMode) store.permissionMode = overview.permissionMode;
        if (typeof overview.hideToolCalls === 'boolean') store.hideToolCalls = overview.hideToolCalls;
        store.sessions = overview.sessions;
        store.groups = overview.groups ?? [];
        for (const id of overview.running) viewOf(id, false).running = true;
        store.loadError = '';
        void store.refreshContext();
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
      void store.refreshContext();
      if (!id) return;
      const curSession = store.sessions.find(s => s.id === id);
      if (curSession?.permissionMode) {
        store.permissionMode = curSession.permissionMode;
      }
      if (!viewOf(id, false).loaded) await loadView(id);
    },

    async send(text, attachments = [], note = '', mentions = []) {
      const images = attachments.filter((item): item is string => typeof item === "string");
      const files = attachments.filter(item => typeof item !== "string");
      deps.onActivity?.();
      if (!deps.desktop) { deps.notify('请在桌面应用中使用聊天', 'error'); return false; }
      let id = store.currentId;
      let created = false;
      try {
        if (!id) {
          // The host runs each conversation in the mode saved with it, so a new one must be created in the
          // mode the top bar shows now; otherwise "完全信任" would be displayed but not applied.
          const summary = await deps.invoke<SessionSummary>('chat_new_session', {
            permissionMode: store.permissionMode, ...(store.pendingWorkspace ? { workspace: store.pendingWorkspace } : {}),
            ...(store.pendingExpert ? { expert: { plugin: store.pendingExpert.plugin, ...(store.pendingExpert.agent ? { agent: store.pendingExpert.agent } : {}) } } : {}),
          });
          store.pendingWorkspace = '';
          store.pendingExpert = null;
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
      const item = { kind: 'user' as const, id: nextId(view), text, images: [...images], ...(files.length ? { files: files.map(({ name, size, mime }) => ({ name, size, mime })) } : {}) };
      view.items.push(item);
      view.running = true;
      view.status = '';
      try {
        // Images are left out entirely when there are none, so a plain message keeps its old shape.
        const args = { sessionId: id, text: note ? note + text : text, ...(images.length ? { images } : {}), ...(files.length ? { attachments: files.map(({name,dataUrl}) => ({name,dataUrl})) } : {}), ...(mentions.length ? { mentions: mentions.map(({kind,id}) => ({kind,id})) } : {}) };
        sent.add(id);
        upsert(await deps.invoke<SessionSummary>('chat_send', args));
        return true;
      } catch (error) {
        // The host refused before starting, so nothing of this message happened.
        sent.delete(id);
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
        sent.add(id);
        upsert(await deps.invoke<SessionSummary>('chat_send', { sessionId: id, text: '', retry: true }));
        // Keep the original user message and drop only the now-resolved failure notice.
        view.items = view.items.filter(i => i.id !== previousFailure);
        return true;
      } catch (e) { sent.delete(id); view.running = false; deps.notify(e, 'error'); return false; }
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

    async refreshContext() {
      if (!deps.desktop) return;
      const id = store.currentId;
      try {
        const info = await deps.invoke<ContextInfo>('chat_context', { sessionId: id });
        if (!disposed && store.currentId === id && info) store.context = info;
      } catch {
        // A failed estimate is not worth a warning; the ring keeps its last numbers.
      }
    },

    async compact() {
      const id = store.currentId;
      const view = store.current();
      if (!deps.desktop || !id || !view || view.running || store.compactingId === id) return false;
      store.compactingId = id;
      try {
        const info = await deps.invoke<ContextInfo>('chat_compact', { sessionId: id });
        if (!disposed && store.currentId === id && info) store.context = info;
        deps.notify('较早的对话已压缩成摘要', 'success');
        try {
          // Show the summary where it now sits in the saved conversation, as reopening the conversation would.
          const messages = await deps.invoke<WireMessage[]>('chat_load_session', { sessionId: id });
          if (!disposed && !view.running) view.items = itemsFromHistory(view, messages ?? []);
        } catch {
          // The summary is in force either way; it appears when the conversation is next opened.
        }
        return true;
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      } finally {
        if (store.compactingId === id) store.compactingId = null;
      }
    },

    async setContextSize(tokens) {
      if (!deps.desktop) return false;
      const before = store.context.limit;
      store.context = { ...store.context, limit: tokens };
      try {
        await deps.invoke('set_context_tokens', { tokens });
      } catch (error) {
        deps.notify(error, 'error');
        if (!disposed) store.context = { ...store.context, limit: before };
        return false;
      } finally {
        await store.refreshContext();
      }
      return true;
    },

    rename: (id, title) => edit('chat_rename_session', { sessionId: id, title }),
    move: (id, groupId) => edit('chat_move_session', { sessionId: id, groupId }),
    createGroup: (name, moveId) => edit('chat_create_group', moveId ? { name, sessionId: moveId } : { name }),
    renameGroup: (id, name) => edit('chat_rename_group', { groupId: id, name }),
    removeGroup: id => edit('chat_delete_group', { groupId: id }),

    async answerApproval(request, decision) {
      const view = store.views[request.session];
      if (view) view.approvals = view.approvals.filter(a => a.id !== request.id);
      try {
        await deps.invoke('chat_answer_approval', { id: request.id, decision });
      } catch (error) {
        if (String(error).includes(STALE)) return;
        // The run is still waiting on this answer: put the card back so it can be answered again.
        if (view) addPending(view.approvals, request);
        deps.notify(error, 'error');
      }
    },

    async undoRun(session, itemId) {
      const view = store.views[session];
      const item = view?.items.find(i => i.id === itemId);
      if (!view || !item || item.kind !== 'notice' || !item.undoRun || view.running) return false;
      try {
        const report = await deps.invoke<{ restored: string[]; removed: string[]; failed: [string, string][] }>('chat_undo_run', { sessionId: session, runId: item.undoRun });
        item.undoRun = undefined;
        const done = report.restored.length + report.removed.length;
        item.text = `这轮回答的 ${done} 个文件改动已撤销。`;
        if (report.failed.length) {
          item.tone = 'warning';
          item.text += `另有 ${report.failed.length} 个没能恢复。`;
          item.details = report.failed.map(([path, reason]) => `${path}：${reason}`).join('\n');
          item.detailsLabel = '查看原因';
        }
        deps.notify(item.text, report.failed.length ? 'info' : 'success');
        return true;
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      }
    },

    async answerQuestion(request, answer) {
      const view = store.views[request.session];
      if (view) view.questions = view.questions.filter(q => q.id !== request.id);
      try {
        await deps.invoke('chat_answer_question', { id: request.id, answer });
      } catch (error) {
        if (String(error).includes('这个问题已经结束')) return;
        if (view) addPending(view.questions, request);
        deps.notify(error, 'error');
      }
    },
  }) as ChatStore;

  function receive(payload: ChatEventPayload) {
    const { session, runId, sequence } = payload;
    const key = `${session}/${runId ?? ''}`;
    if (runId && sequence !== undefined && sequence <= (cursors.get(key) ?? 0)) return;
    const view = viewOf(session, !runId); // legacy hosts have no cursor/snapshot contract
    if (!runId) {
      present(view, payload);
      if (!view.loaded) { (pendingEvents[session] ??= []).push(payload); void loadView(session); }
      return;
    }
    if (!view.loaded || (runId && activeRun.get(session) !== runId)) {
      (pendingEvents[session] ??= []).push(payload);
      void loadView(session);
      return;
    }
    if (runId && sequence !== undefined) cursors.set(key, sequence);
    present(view, payload);
  }

  /** Conversations whose current answer was asked from this window. */
  const sent = new Set<string>();
  function present(view: SessionView, { session, event }: ChatEventPayload) {
    applyEvent(view, event);
    if (event.type === 'done' && store.currentId === session) {
      const last = view.items.at(-1);
      if (last?.kind === 'assistant' && last.text.trim()) deps.onReply?.(session, last.text, last.id);
    }
    if ((event.type === 'cancelled' || event.type === 'error') && store.currentId === session) deps.onActivity?.();
    if (event.type === 'done' || event.type === 'cancelled' || event.type === 'error') queueMicrotask(() => sent.delete(session));
    if ((event.type === 'done' || event.type === 'cancelled' || event.type === 'error') && store.currentId === session) void store.refreshContext();
    if (event.type === 'error' && modelFailure(event.message).refresh && Date.now() - refreshedAfterErrorAt > 300000) {
      refreshedAfterErrorAt = Date.now();
      void deps.invoke('refresh_ai_providers', { force: true }).catch(() => {});
    }
  }

  function applyOutcome(session:string,outcome:Outcome){
    // An unverified outcome is internal bookkeeping, not a user-facing warning.
    // Actual errors and uncertain operations have their own execution events.
    if(outcome.status!=='succeeded'||outcome.validator!=='expected_file_sha256')return;
    const v=viewOf(session,true);if(!v.items.some(i=>i.kind==='tool'))return;
    const id=`outcome:${outcome.runId}`;if(v.items.some(i=>i.id===id))return;
    v.items.push({kind:'notice',id,tone:'info',text:'指定文件已通过 SHA256 验收。',details:outcome.evidence,detailsLabel:'查看执行依据'});
  }

  function applyPending(id:string,p:PendingSnapshot) {
    if(p.revision < (revisions.get(id) ?? -1)) return;
    revisions.set(id,p.revision);const v=viewOf(id,true);
    v.approvals=p.approvals.filter(x=>x.session===id);v.questions=p.questions.filter(x=>x.session===id);
  }

  async function loadView(id: string): Promise<void> {
    const existing = loading.get(id);
    if (existing) return existing;
    const beforeRequest = new Set(pendingEvents[id] ?? []);
    const task = (async () => {
      const view = viewOf(id, false);
      try {
        let snapshot: Snapshot | undefined;
        try { if (supportsSnapshots) snapshot = await deps.invoke<Snapshot>('chat_session_snapshot', { sessionId: id }); }
        catch (error) {
          // Only an old host lacking the command uses the legacy path.
          if (!/unknown|not found|unexpected chat_session_snapshot/i.test(String(error))) throw error;
        }
        if (!snapshot || !Array.isArray(snapshot.messages)) {
          snapshot = { messages: await deps.invoke<WireMessage[]>('chat_load_session', { sessionId: id }) ?? [] };
        }
        if (disposed || !store.views[id]) return;
        view.items = itemsFromHistory(view, snapshot.messages);
        if(snapshot.pending) applyPending(id,snapshot.pending);
        view.usage = undefined;
        view.status = '';
        for (const payload of snapshot.events ?? []) applyEvent(view, payload.event);
        if(snapshot.outcome)applyOutcome(id,snapshot.outcome);
        view.loaded = snapshot.ready !== false;
        if (snapshot.runId) {
          activeRun.set(id, snapshot.runId);
          cursors.set(`${id}/${snapshot.runId}`, snapshot.sequence ?? 0);
        }
        if (typeof snapshot.running === 'boolean') {
          view.running = snapshot.running;
          if (!snapshot.running) {
            for (const item of view.items) if (item.kind === 'assistant' || item.kind === 'reasoning') item.streaming = false;
          }
        }
        if (snapshot.interrupted && !interruptions[id]) view.items.push({ kind: 'notice', id: nextId(view), tone: 'warning', text: '上次回答在中途中断，生成的内容已保存。', canContinue: true });
        const interrupted = interruptions[id];
        if (interrupted) { view.items.push(interrupted); delete interruptions[id]; }
        if (snapshot.warning) deps.notify(snapshot.warning, 'info');
        const queued = (pendingEvents[id] ?? []).sort((a, b) => a.runId === b.runId ? (a.sequence ?? 0) - (b.sequence ?? 0) : 0);
        pendingEvents[id] = [];
        for (const payload of queued) {
          if (payload.runId && snapshot.runId && payload.runId !== snapshot.runId) {
            // Events received before this request are already represented by the
            // snapshot or an older persisted run. A newer run arriving during the
            // request requires one fresh snapshot instead of appending blindly.
            if (!beforeRequest.has(payload)) pendingEvents[id].push(payload);
          } else if (!payload.runId || payload.sequence === undefined || payload.sequence > (cursors.get(`${id}/${payload.runId}`) ?? snapshot.sequence ?? 0)) {
            if (payload.runId && payload.sequence !== undefined) cursors.set(`${id}/${payload.runId}`, payload.sequence);
            if (payload.runId) present(view, payload); else applyEvent(view, payload.event);
          }
        }
      } catch (error) {
        pendingEvents[id] = [];
        if (!disposed) deps.notify(error, 'error');
      }
    })();
    loading.set(id, task);
    try { await task; } finally {
      loading.delete(id);
      if (!disposed && pendingEvents[id]?.length) void loadView(id);
    }
  }

  /** A live delta never proves that persisted history has been loaded. */
  function viewOf(id: string, loaded: boolean): SessionView {
    // Read it back after storing it: the value of an assignment is the plain object, and changes to
    // that would never reach the page. Only what comes out of the reactive store is watched.
    if (!store.views[id]) store.views[id] = newView(id, loaded);
    return store.views[id];
  }

  /** Runs one of the host's list edits and takes the whole list it sends back. */
  async function edit(command: string, args: Record<string, unknown>): Promise<boolean> {
    if (!deps.desktop) return false;
    try {
      const library = await deps.invoke<Library>(command, args);
      if (disposed) return false;
      store.sessions = library.sessions;
      store.groups = library.groups;
      return true;
    } catch (error) {
      deps.notify(error, 'error');
      return false;
    }
  }

  /** The host compacts a conversation by itself once it is nearly full; the ring shows that like a compaction asked for. */
  function onCompacting(event: CompactingEvent) {
    if (event.state === 'start') {
      store.compactingId = event.session;
      return;
    }
    if (store.compactingId === event.session) store.compactingId = null;
    if (event.state === 'done') {
      const view = store.views[event.session];
      if (view && event.summary) view.items.push(summaryNotice(view, event.summary));
    } else {
      deps.notify(`自动压缩没有成功：${event.message ?? '原因不明'}。较早的内容仍会在超出上限时被省略。`, 'info');
    }
    if (store.currentId === event.session) void store.refreshContext();
  }

  function upsert(summary: SessionSummary) {
    store.sessions = [summary, ...store.sessions.filter(s => s.id !== summary.id)];
  }

  function addPending<T extends { id: string }>(list: T[], request: T) {
    if (!list.some(item => item.id === request.id)) list.push(request);
  }

  return store;
}
