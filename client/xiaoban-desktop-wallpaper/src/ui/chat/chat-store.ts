import type { PendingAttachment } from './attachments.ts';
import type { Mention, MentionItem } from './mentions.ts';
import type { MessageQueue, SessionModel } from './types.ts';
import { modelFailure } from './model-errors.ts';
import { reactive, ref, watch } from 'vue';
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
  queues: Record<string, MessageQueue>;
  readonly queue: MessageQueue | null;
  enqueue(text: string, attachments: PendingAttachment[], mentions: Mention[], mode?: 'queue' | 'steer' | 'replace'): Promise<boolean>;
  updateQueue(action: 'pause' | 'resume' | 'remove' | 'edit' | 'reorder' | 'send_now', id?: string, options?: { text?: string; order?: string[]; removeImages?: number[]; removeFiles?: string[] }, sessionId?: string): Promise<boolean>;
  drafts: Record<string, ComposerDraft>;
  readonly draft: ComposerDraft;
  draftFor(id: string | null): ComposerDraft;
  pendingModel: SessionModel | null;
  pendingContextTokens: number | null;
  readonly model: SessionModel | null;
  setModel(model: SessionModel, sessionId?: string | null): Promise<boolean>;
  readonly sending: boolean;
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
  /** Stop the current run and wait for its terminal event before sending a plan revision. */
  revisePlan(instruction: string): Promise<boolean>;
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
  setContextSize(tokens: number, sessionId?: string | null): Promise<boolean>;
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

export interface ComposerDraft {
  text: string;
  attachments: PendingAttachment[];
  skills: { name: string; title: string }[];
  connectors: MentionItem[];
}

/** Sent by the host around an automatic compaction (`chat-compacting`). */
interface CompactingEvent { session: string; state: 'start' | 'done' | 'failed'; summary?: string; message?: string }

export const CONTINUE_TEXT = '请接着上面的内容继续写完。';
const STALE = '这个确认已经结束';

export function createChatStore(deps: ChatDeps): ChatStore {
  let selectionRevision = 0;
  const draftSending = ref(false);
  const queueSending = reactive(new Set<string>());
  const queueAttempts = new Map<string, { payload: string; id: string }>();
  const defaultPermissionMode = ref<'ask' | 'auto'>('ask');
  const pendingPermissionMode = ref<'ask' | 'auto'>('ask');
  let contextRevision = 0;
  const connectorWrites = new Map<string, Promise<unknown>>();
  const modelChanges = reactive(new Set<string>());
  const defaultConfigured = ref(false);
  const modelRevisions = new Map<string, number>();
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
  let revisingPlan = false;
  interface Outcome {runId:string;status:string;validator:string;evidence:string;sessionId?:string;lastToolCallId?:string;verifiedAtMs?:number|null}
  interface Snapshot { outcomes?:Outcome[]; outcome?:Outcome; pending?: PendingSnapshot; messages: WireMessage[]; runId?: string; sequence?: number; events?: ChatEventPayload[]; running?: boolean; ready?: boolean; warning?: string; interrupted?: boolean }


  const store: ChatStore = reactive({
    drafts: {} as Record<string, ComposerDraft>,
    get draft() { return store.draftFor(store.currentId); },
    draftFor(id: string | null) {
      const key = id ?? '$new';
      store.drafts[key] ??= { text: '', attachments: [], skills: [], connectors: (store.sessions.find(s => s.id === id)?.connectors ?? []).map(id => ({ kind: 'connector' as const, id, name: id, description: '' })) };
      return store.drafts[key];
    },
    pendingModel: null as SessionModel | null,
    pendingContextTokens: null as number | null,
    get model() { return store.currentId ? store.sessions.find(s => s.id === store.currentId)?.model ?? null : store.pendingModel; },
    get sending() { return draftSending.value || queueSending.has(store.currentId ?? '$new') || modelChanges.has(store.currentId ?? '$new'); },
    async setModel(model: SessionModel, sessionId: string | null = store.currentId) {
      const key = sessionId ?? '$new';
      const revision = (modelRevisions.get(key) ?? 0) + 1;
      modelRevisions.set(key, revision);
      const draft = sessionId ? null : store.draftFor(null);
      modelChanges.add(key);
      try {
        const saved = await deps.invoke<SessionModel>('chat_set_model', { sessionId, selection: model });
        if (disposed || modelRevisions.get(key) !== revision) return false;
        if (sessionId) {
          const session = store.sessions.find(s => s.id === sessionId);
          if (session) session.model = saved;
        } else if (store.draftFor(null) === draft) { store.pendingModel = saved; }
        if (store.currentId === sessionId) void store.refreshContext();
        return true;
      } catch (error) { deps.notify(error, 'error'); return false; }
      finally { if (modelRevisions.get(key) === revision) modelChanges.delete(key); }
    },
    ready: false,
    get configured() { return Boolean(store.model) || defaultConfigured.value; },
    set configured(value: boolean) { defaultConfigured.value = value; },
    workspace: '',
    recentWorkspaces: [] as RecentWorkspace[],
    pendingWorkspace: '',
    pendingExpert: null as ExpertView | null,
    get currentExpert() {
      const bound = store.currentId ? store.sessions.find(s => s.id === store.currentId)?.expert : null;
      if (bound) return { plugin: bound.plugin, name: bound.name, profession: bound.profession, agent: bound.agent, workflow: bound.workflow, hasAvatar: !bound.workflow };
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
    get permissionMode(): 'ask' | 'auto' { return store.currentId ? store.sessions.find(s => s.id === store.currentId)?.permissionMode ?? 'ask' : pendingPermissionMode.value; },
    set permissionMode(mode: 'ask' | 'auto') {
      const session = store.currentId ? store.sessions.find(s => s.id === store.currentId) : null;
      if (session) session.permissionMode = mode;
      else pendingPermissionMode.value = mode;
    },
    hideToolCalls: false,
    async setPermissionMode(mode: 'ask' | 'auto') {
      const id = store.currentId;
      const before = store.permissionMode;
      store.permissionMode = mode;
      if (store.currentId) {
        const s = store.sessions.find(item => item.id === store.currentId);
        if (s) s.permissionMode = mode;
      }
      if (!deps.desktop || !id) return;
      try {
        const args = { sessionId: id, mode };
        await deps.invoke('chat_set_permission_mode', args);
      } catch (error) {
        const session = store.sessions.find(s => s.id === id);
        if (session?.permissionMode === mode) session.permissionMode = before;
        deps.notify(error, 'error');
      }
    },
    queues: {} as Record<string, MessageQueue>,
    get queue() { return store.currentId ? store.queues[store.currentId] ?? null : null; },
    async enqueue(text, attachments, mentions, mode = 'queue') {
      const session = store.currentId;
      if (!deps.desktop || !session || queueSending.has(session)) return false;
      queueSending.add(session);
      try {
        const payload = JSON.stringify([text, attachments, mentions, mode]);
        const uniqueId = typeof globalThis.crypto?.randomUUID === 'function' ? globalThis.crypto.randomUUID() : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
        if (queueAttempts.get(session)?.payload !== payload) queueAttempts.set(session, { payload, id: `q-${uniqueId}` });
        const requestId = queueAttempts.get(session)!.id;
        const images = attachments.filter((a): a is string => typeof a === 'string');
        const files = attachments.filter(a => typeof a !== 'string').map(({ name, dataUrl }) => ({ name, dataUrl }));
        const queue = await deps.invoke<MessageQueue>('chat_enqueue', { sessionId: session, text, mode: mode === 'replace' ? 'queue' : mode, stopNow: mode === 'replace', requestId, images, attachments: files, mentions });
        applyQueue(session, queue);
        queueAttempts.delete(session);
        sent.add(session);
        return true;
      } catch (error) { deps.notify(error, 'error'); return false; }
      finally { queueSending.delete(session); }
    },
    async updateQueue(action, id, options = {}, sessionId) {
      const session = sessionId ?? store.currentId;
      if (!deps.desktop || !session) return false;
      try {
        const queue = await deps.invoke<MessageQueue>('chat_queue_update', { sessionId: session, action, id, ...options });
        applyQueue(session, queue); return true;
      } catch (error) { deps.notify(error, 'error'); return false; }
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
            deps.listen<{session: string; queue: MessageQueue}>('chat-queue-changed', payload => { if (!disposed) applyQueue(payload.session, payload.queue); }),
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
                  defaultPermissionMode.value = payload.mode;
                  if (!store.currentId) pendingPermissionMode.value = payload.mode;
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
        if (overview.permissionMode) defaultPermissionMode.value = overview.permissionMode;
        if (typeof overview.hideToolCalls === 'boolean') store.hideToolCalls = overview.hideToolCalls;
        store.sessions = overview.sessions;
        for (const [session, queue] of Object.entries(overview.queues ?? {})) applyQueue(session, queue);
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
      queueAttempts.clear();
    },

    current() {
      return store.currentId ? store.views[store.currentId] ?? null : null;
    },

    async select(id) {
      ++selectionRevision;
      deps.onActivity?.();
      store.currentId = id;
      void store.refreshContext();
      if (!id) {
        if (draftSending.value) { delete store.drafts.$new; store.pendingModel = null; store.pendingContextTokens = null; }
        pendingPermissionMode.value = defaultPermissionMode.value; return;
      }
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
      const revision = selectionRevision;
      const ownerDraft = store.draft;
      const pendingWorkspace = store.pendingWorkspace;
      const pendingExpert = store.pendingExpert;
      const pendingModel = store.pendingModel;
      const pendingContext = store.pendingContextTokens;
      if (!id && draftSending.value) return false;
      let created = false;
      try {
        if (!id) {
          draftSending.value = true;
          // The host runs each conversation in the mode saved with it, so a new one must be created in the
          // mode the top bar shows now; otherwise "完全信任" would be displayed but not applied.
          const summary = await deps.invoke<SessionSummary>('chat_new_session', {
            permissionMode: store.permissionMode, ...(store.pendingWorkspace ? { workspace: store.pendingWorkspace } : {}),
            ...(store.pendingExpert ? { expert: store.pendingExpert.workflow ? { workflow: store.pendingExpert.workflow } : { plugin: store.pendingExpert.plugin, ...(store.pendingExpert.agent ? { agent: store.pendingExpert.agent } : {}) } } : {}),
            ...(store.pendingModel ? { model: store.pendingModel } : {}),
            ...(store.pendingContextTokens ? { contextTokens: store.pendingContextTokens } : {}),
            ...(ownerDraft.connectors.length ? { connectors: ownerDraft.connectors.map(c => c.id) } : {}),
          });
          if (store.pendingWorkspace === pendingWorkspace) store.pendingWorkspace = '';
          if (store.pendingExpert === pendingExpert) store.pendingExpert = null;
          id = summary.id;
          created = true;
          upsert(summary);
          store.views[id] = newView(id, true);
          store.drafts[id] = ownerDraft;
          if (selectionRevision === revision && store.currentId === null) store.currentId = id;
          if (store.drafts.$new === ownerDraft) delete store.drafts.$new;
          if (store.pendingModel === pendingModel) store.pendingModel = null;
          if (store.pendingContextTokens === pendingContext) store.pendingContextTokens = null;
          if (selectionRevision === revision) pendingPermissionMode.value = defaultPermissionMode.value;
        }
      } catch (error) {
        deps.notify(error, 'error');
        return false;
      } finally {
        draftSending.value = false;
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
          void deps.invoke<SessionSummary[]>('chat_delete_session', { sessionId }).then(() => { if (!disposed) store.sessions = store.sessions.filter(s => s.id !== sessionId); }).catch(() => {});
          delete store.views[sessionId];
          if (!store.drafts.$new || (store.currentId === sessionId && selectionRevision === revision)) {
            store.drafts.$new = ownerDraft;
            store.pendingModel = pendingModel; store.pendingContextTokens = pendingContext;
            store.pendingExpert = pendingExpert; store.pendingWorkspace = pendingWorkspace;
          }
          if (store.currentId === sessionId && selectionRevision === revision) {
            store.currentId = null;
          }
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

    async revisePlan(instruction) {
      const id = store.currentId, view = store.current();
      if (!id || !view || !instruction.trim() || revisingPlan) return false;
      const run = activeRun.get(id);
      revisingPlan = true;
      try {
        if (view.running) {
          // A cancellation request returning does not mean native tools have finished stopping.
          await deps.invoke('chat_cancel', { sessionId: id });
          if (view.running) await new Promise<void>((resolve, reject) => {
            const stop = watch(() => view.running, running => {
              if (!running) { clearTimeout(timeout); stop(); resolve(); }
            }, { flush: 'sync' });
            const timeout = setTimeout(() => { stop(); reject(new Error('任务尚未停止，修改要求已保留，请稍后再试。')); }, 5000);
          });
        }
        if (disposed || store.currentId !== id || store.views[id] !== view || activeRun.get(id) !== run) {
          deps.notify('会话或运行已切换，修改要求尚未发送。', 'info');
          return false;
        }
        return await store.send(`请调整当前任务的后续计划：\n${instruction.trim()}\n保留已经完成的进展，先核对当前状态和执行依据，再用 todo 更新后续步骤并继续。结果不明的操作先检查，不要直接重放；原有授权范围不变。`);
      } catch (error) { deps.notify(error, 'error'); return false; }
      finally { revisingPlan = false; }
    },

    async remove(id) {
      if (id === store.currentId) deps.onActivity?.();
      try {
        const sessions = await deps.invoke<SessionSummary[]>('chat_delete_session', { sessionId: id });
        store.sessions = sessions ?? store.sessions.filter(s => s.id !== id);
        delete store.views[id];
        delete store.queues[id];
        queueAttempts.delete(id);
        if (store.currentId === id) store.currentId = null;
      } catch (error) {
        deps.notify(error, 'error');
      }
    },

    async refreshContext() {
      if (!deps.desktop) return;
      const id = store.currentId;
      try {
        const revision = ++contextRevision;
        const info = await deps.invoke<ContextInfo>('chat_context', { sessionId: id, ...(!id && store.pendingModel ? { model: store.pendingModel } : {}), ...(!id && store.pendingContextTokens ? { contextTokens: store.pendingContextTokens } : {}) });
        if (!disposed && store.currentId === id && revision === contextRevision && info) store.context = info;
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

    async setContextSize(tokens, sessionId: string | null = store.currentId) {
      if (!deps.desktop) return false;
      const session = store.sessions.find(s => s.id === sessionId);
      const before = session?.contextTokens ?? store.pendingContextTokens;
      if (sessionId && !session) return false;
      if (session) session.contextTokens = tokens; else store.pendingContextTokens = tokens;
      if (store.currentId === sessionId) store.context = { ...store.context, limit: tokens };
      try {
        if (sessionId) await deps.invoke('chat_set_session_options', { sessionId, contextTokens: tokens });
      } catch (error) {
        deps.notify(error, 'error');
        if (session?.contextTokens === tokens) session.contextTokens = before;
        if (!sessionId && store.pendingContextTokens === tokens) store.pendingContextTokens = before;
        return false;
      } finally {
        if (store.currentId === sessionId) await store.refreshContext();
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

  function applyQueue(session: string, queue: MessageQueue) {
    if (disposed || !queue || queue.revision < (store.queues[session]?.revision ?? -1)) return;
    store.queues[session] = queue;
  }

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

  function applyOutcome(session: string, outcome: Outcome) {
    const v = viewOf(session, true);
    if (outcome.sessionId && outcome.sessionId !== session) return;
    const id = `outcome:${outcome.runId}`;
    const existing = v.items.findIndex(item => item.id === id);
    const old = existing >= 0 ? v.items[existing] : undefined;
    // A late trace-only event must not undo explicit verification.
    if (old?.kind === 'notice' && old.taskStatus === 'succeeded'
      && (outcome.status !== 'succeeded' || (old.verifiedAtMs ?? 0) > (outcome.verifiedAtMs ?? 0))) return;
    const anchors = outcome.lastToolCallId ? v.items.map((item, index) => item.kind === 'tool' && item.id === outcome.lastToolCallId ? index : -1).filter(index => index >= 0) : [];
    if (existing < 0 && outcome.lastToolCallId && anchors.length !== 1) return;
    if (existing < 0 && !outcome.lastToolCallId && (!v.items.some(i => i.kind === 'tool')
      || activeRun.has(session) && activeRun.get(session) !== outcome.runId)) return;
    const partial = outcome.status === 'partial', failed = outcome.status === 'failed';
    const verified = outcome.status === 'succeeded';
    const status = ['succeeded','failed','partial','cancelled','unknown'].includes(outcome.status)
      ? outcome.status as 'succeeded'|'failed'|'partial'|'cancelled'|'unknown' : 'unknown';
    const notice: ChatItem = { kind: 'notice', id, taskStatus: status, tone: partial || failed ? 'warning' : 'info',
      text: partial ? '仍有未解决的操作失败，请核对结果后继续。' : failed ? '本次执行失败，已保留当前进度。'
        : verified ? outcome.validator === 'expected_file_sha256' ? '指定文件已通过 SHA256 验收。' : '任务已通过指定验收。'
        : outcome.status === 'cancelled' ? '已停止' : '执行已结束，任务结果尚未独立验收。',
      verifiedAtMs: outcome.verifiedAtMs ?? undefined,
      canContinue: partial || failed, details: outcome.evidence, detailsLabel: '查看执行依据' };
    if (existing >= 0) v.items.splice(existing, 1, notice);
    else {
      const nextUser = anchors.length ? v.items.findIndex((item, index) => index > anchors[0] && item.kind === 'user') : -1;
      v.items.splice(nextUser < 0 ? v.items.length : nextUser, 0, notice);
    }
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
        view.usage = undefined;
        view.status = '';
        for (const payload of snapshot.events ?? []) applyEvent(view, payload.event);
        if(snapshot.pending) applyPending(id,snapshot.pending);
        if (snapshot.runId) activeRun.set(id, snapshot.runId);
        for (const outcome of snapshot.outcomes ?? []) applyOutcome(id, outcome);
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

  let syncingConnectors = false;
  // A host update is authoritative; it must never look like a local composer edit.
  unsubscribers.push(watch(() => store.sessions, sessions => {
    syncingConnectors = true;
    try {
      for (const session of sessions) {
        const draft = store.drafts[session.id];
        if (!draft || session.connectors === undefined || connectorWrites.has(session.id)) continue;
        draft.connectors = (session.connectors ?? []).map(id => draft.connectors.find(item => item.id === id)
          ?? { kind: 'connector' as const, id, name: id, description: '' });
      }
    } finally { syncingConnectors = false; }
  }, { flush: 'sync' }));
  // Observe draft data only, including background conversations. Creating a draft is not an edit.
  unsubscribers.push(watch(() => Object.entries(store.drafts).map(([id, draft]) => [id, draft.connectors.map(c => c.id)] as const), (drafts, before) => {
    if (syncingConnectors || disposed || !deps.desktop) return;
    const previousDrafts = new Map(before);
    for (const [id, ids] of drafts) {
      const old = previousDrafts.get(id);
      if (id === '$new' || !old || JSON.stringify(old) === JSON.stringify(ids)) continue;
      const session = store.sessions.find(s => s.id === id);
      if (!session || JSON.stringify(session.connectors ?? []) === JSON.stringify(ids)) continue;
      session.connectors = [...ids];
      const previous = connectorWrites.get(id) ?? Promise.resolve();
      const write = previous.catch(() => {}).then(() => deps.invoke('chat_set_session_options', { sessionId: id, connectors: [...ids] }))
        .catch(error => deps.notify(error, 'error'));
      connectorWrites.set(id, write);
      void write.finally(() => { if (connectorWrites.get(id) === write) connectorWrites.delete(id); });
    }
  }, { flush: 'sync' }));
  return store;
}
