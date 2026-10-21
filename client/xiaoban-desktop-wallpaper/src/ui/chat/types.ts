import type { AttachmentCard } from './attachments.ts';
/** What the host sends and accepts for the chat. Field names are the host's own (snake_case for kernel events). */

export interface SessionSummary { id: string; title: string; createdAt: number; updatedAt: number; permissionMode?: 'ask' | 'auto'; /** The group it was moved into; empty or missing when it is not in one. */ group?: string }

/** How much of the context the next request would use. The numbers are estimates, in tokens. */
export interface ContextInfo { used: number; limit: number; canCompact: boolean }

/** A folder in the list of conversations. Deleting one never deletes the conversations in it. */
export interface ChatGroup { id: string; name: string }

/** What the host sends back after it changes names or groups: the whole list, so both stay in step. */
export interface Library { sessions: SessionSummary[]; groups: ChatGroup[] }

export interface Overview {
  configured: boolean;
  sessions: SessionSummary[];
  groups?: ChatGroup[];
  running: string[];
  /** The folder the assistant's file and command tools work in. */
  workspace: string;
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
}

export type KernelEvent =
  | { type: 'text_delta'; text: string }
  | { type: 'tool_start'; id: string; name: string; arguments: string }
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
}
export type ApprovalDecision = 'once' | 'always' | 'deny';

export interface QuestionRequest { id: string; session: string; question: string; options: string[] }

export type ChatItem =
  | { kind: 'user'; id: string; text: string; images: string[]; files?: AttachmentCard[] }
  | { kind: 'assistant'; id: string; text: string; streaming: boolean }
  | { kind: 'tool'; id: string; name: string; arguments: string; output: string; status: 'running' | 'done' | 'error' }
  | { kind: 'notice'; id: string; tone: 'info' | 'warning' | 'error'; text: string; code?: 'truncated'; canContinue?: boolean; canSwitchModel?: boolean; canRetry?: boolean; details?: string; /** The label on the fold-out `details`; "错误详情" when missing. */ detailsLabel?: string };

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
  /** Counter for item ids. */
  seq: number;
}

