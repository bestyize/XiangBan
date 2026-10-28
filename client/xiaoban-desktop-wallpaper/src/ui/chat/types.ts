import type { AttachmentCard } from './attachments.ts';
import type { InstalledConnector, MarketConnector } from '../ai-store/connector-plugins.ts';
import type { AiReplyPreference, AiReasoningEffort } from '../../bridge.ts';
export interface SessionModel { providerId: string; model: string; replyPreference?: AiReplyPreference; reasoningEfforts?: AiReasoningEffort[] | null }
/** What the host sends and accepts for the chat. Field names are the host's own (snake_case for kernel events). */

/** The installed expert a conversation was started with (a WorkBuddy-style agent persona). */
export interface ExpertBinding { plugin: string; agent: string; name: string; profession: string; workflow?: string }
/** An installed expert as the pickers and the new-task page show it. */
export interface ExpertView {
  plugin: string; name: string; profession: string; description?: string; agent?: string | null;
  workflow?: string;
  quickPrompts?: string[]; tags?: string[]; hasAvatar?: boolean; kind?: 'agent' | 'team';
}
export interface SessionSummary { model?: SessionModel | null; contextTokens?: number | null; connectors?: string[]; expert?: ExpertBinding | null; id: string; title: string; createdAt: number; updatedAt: number; permissionMode?: 'ask' | 'auto'; /** The group it was moved into; empty or missing when it is not in one. */ group?: string; /** The folder it works in; empty or missing for an old conversation, which uses the default folder. */ workspace?: string; companionId?: string; companionName?: string; companionCursor?: number; /** Set while it works in an isolated copy of its folder; `workspace` is then the copy. */ worktree?: WorktreeLink | null }

/** A working folder in the chat page's folder menu. `wide` is a home folder or a disk's root. */
/** The isolated git copy a conversation works in (see `worktree.rs`): `origin` is where the user's own files are. */
export interface WorktreeLink { origin: string; path: string; branch: string; base: string }
/** How the open conversation stands with its copy: `active`, or whether it could make one. */
export type WorktreeInfo =
  | { active: true; link: WorktreeLink; status: { exists: boolean; branch: string; ahead: number; changed: string[]; changedCount: number; originBranch: string; originClean: boolean } }
  | { active: false; available: boolean; reason?: string };
export interface RecentWorkspace { path: string; name: string; exists: boolean; wide: boolean }

/** How much of the context the next request would use. The numbers are estimates, in tokens. */
export interface ContextInfo { used: number; limit: number; canCompact: boolean }

/** A folder in the list of conversations. Deleting one never deletes the conversations in it. */
export interface ChatGroup { id: string; name: string }

/** What the host sends back after it changes names or groups: the whole list, so both stay in step. */
export interface Library { sessions: SessionSummary[]; groups: ChatGroup[] }

export interface QueueEntry { id: string; text: string; mode: 'queue' | 'steer'; dispatching: boolean; images: number; files: AttachmentCard[] }
export interface MessageQueue { enqueuedId?: string; revision: number; paused: boolean; reason: string; items: QueueEntry[] }
export interface Overview {
  queueProtocol?: number;
  queues?: Record<string, MessageQueue>;
  pendingProtocol?: number;
  snapshotProtocol?: number;
  configured: boolean;
  sessions: SessionSummary[];
  groups?: ChatGroup[];
  running: string[];
  /** The default working folder (the setting), for old conversations and the first new one. */
  workspace: string;
  /** Folders used lately, newest first. */
  recentWorkspaces?: RecentWorkspace[];
  permissionMode: 'ask' | 'auto';
  hideToolCalls: boolean;
}

export type WirePart = { type: 'text'; text: string } | { type: 'image'; url: string };
export interface WireToolCall { id: string; name: string; arguments: string }
/** A saved message, as the kernel stores it. */
export interface WireMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  parts?: WirePart[];
  tool_calls?: WireToolCall[];
  tool_call_id?: string | null;
  /** Written by the assistant's harness (for example a nudge to keep working), not by the user. */
  synthetic?: boolean;
  is_error?: boolean;
  structured_content?: unknown;
  context_summary?: { keep_from: number; text: string };
  /** Only displayable model-provided content; opaque provider_state is never shown. */
  reasoning?: { kind: 'summary' | 'text'; text: string }[];
}

export type KernelEvent =
  | { type: 'user_message'; message: WireMessage; initial: boolean }
  | { type: 'text_delta'; text: string }
  | { type: 'reasoning_delta'; kind: 'summary' | 'text'; text: string }
  | { type: 'tool_start'; id: string; name: string; arguments: string }
  | { type: 'tool_waiting'; id: string; resource: string; owner_session?: string | null; waiting: boolean }
  | { type: 'tool_dispatched'; id: string }
  | { type: 'tool_end'; id: string; output: string; is_error: boolean; parts?: WirePart[]; structured_content?: unknown }
  | { type: 'usage'; prompt_tokens: number; completion_tokens: number }
  | { type: 'retrying'; attempt: number; delay_ms: number; message: string }
  | { type: 'truncated' }
  /** The model stopped with work left; the host asked it to go on. */
  | { type: 'continuing'; reason: string }
  | { type: 'done' }
  | { type: 'cancelled' }
  | { type: 'error'; message: string };

export interface ChatEventPayload { session: string; event: KernelEvent; runId?: string; sequence?: number; timestampMs?: number }

export type Risk = 'read' | 'write' | 'network' | 'exec';
export interface ApprovalRequest {
  id: string;
  session: string;
  tool: string;
  risk: Risk;
  arguments: unknown;
  /** The conversation has read outside content, so this is asked even where it normally would not be. */
  tainted: boolean;
  /** Why this was asked even in full trust: a dangerous operation. Absent or null for an ordinary request. */
  danger?: string | null;
  canRemember: boolean;
  /** The exact outside-workspace path must be approved for this operation only. */
  externalPath?: boolean;
  /** For a command: the kind of command "always" would allow in this conversation, e.g. `npm test`. */
  rememberAs?: string | null;
}
export type ApprovalDecision = 'once' | 'always' | 'deny';

export interface ConnectorSuggestion {
  reference: string; name: string; description: string;
  installed?: InstalledConnector;
  namespace?: string; slug?: string; version?: string; connector?: MarketConnector;
}
export interface QuestionRequest { id: string; session: string; question: string; options: string[]; connectors?: ConnectorSuggestion[] }

export type ChatItem =
  | { kind: 'user'; delivery?: 'steer' | 'queue'; id: string; text: string; images: string[]; files?: AttachmentCard[] }
  | { kind: 'reasoning'; id: string; text: string; reasoningKind: 'summary' | 'text'; streaming: boolean }
  | { kind: 'assistant'; id: string; text: string; streaming: boolean; /** Tokens of the whole answer, research helpers included. */ usage?: TokenUsage }
  | { kind: 'tool'; id: string; name: string; arguments: string; output: string; status: 'running' | 'done' | 'error'; waitingFor?: { resource: string; ownerSession?: string } }
  | { kind: 'notice'; id: string; tone: 'info' | 'warning' | 'error'; text: string; code?: 'truncated'; taskStatus?: 'succeeded' | 'failed' | 'partial' | 'cancelled' | 'unknown'; verifiedAtMs?: number; canContinue?: boolean; canSwitchModel?: boolean; canRetry?: boolean; details?: string; /** The label on the fold-out `details`; "错误详情" when missing. */ detailsLabel?: string; /** The answer whose file changes this notice can undo. */ undoRun?: string };

export interface SessionView {
  id: string;
  items: ChatItem[];
  /** The history was read from disk (or the conversation is new). */
  loaded: boolean;
  running: boolean;
  approvals: ApprovalRequest[];
  questions: QuestionRequest[];
  /** A short line about what is happening, such as a retry. Empty when nothing needs saying. */
  status: string;
  /** The retry the host is making now (its number), until the model answers; the page adds how long it has waited. */
  retry?: number;
  /** When the last event of the running answer arrived (ms): silence is measured from here. */
  activityAt?: number;
  /** Counter for item ids. */
  seq: number;
  /** Tokens of the answer running now; put on its last reply when it ends. */
  usage?: TokenUsage;
}

export interface TokenUsage { prompt: number; completion: number }
