/** What the host sends and accepts for the chat. Field names are the host's own (snake_case for kernel events). */

export interface SessionSummary { id: string; title: string; createdAt: number; updatedAt: number; permissionMode?: 'ask' | 'auto' }

export interface Overview {
  configured: boolean;
  sessions: SessionSummary[];
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
}

export type KernelEvent =
  | { type: 'text_delta'; text: string }
  | { type: 'tool_start'; id: string; name: string; arguments: string }
  | { type: 'tool_end'; id: string; output: string; is_error: boolean }
  | { type: 'usage'; prompt_tokens: number; completion_tokens: number }
  | { type: 'retrying'; attempt: number; delay_ms: number; message: string }
  | { type: 'truncated' }
  | { type: 'done' }
  | { type: 'cancelled' }
  | { type: 'error'; message: string };

export interface ChatEventPayload { session: string; event: KernelEvent }

export type Risk = 'read' | 'write' | 'network' | 'exec';
export interface ApprovalRequest {
  id: string;
  session: string;
  tool: string;
  risk: Risk;
  arguments: unknown;
  /** The conversation has read outside content, so this is asked even where it normally would not be. */
  tainted: boolean;
  canRemember: boolean;
}
export type ApprovalDecision = 'once' | 'always' | 'deny';

export interface QuestionRequest { id: string; session: string; question: string; options: string[] }

export type ChatItem =
  | { kind: 'user'; id: string; text: string; images: string[] }
  | { kind: 'assistant'; id: string; text: string; streaming: boolean }
  | { kind: 'tool'; id: string; name: string; arguments: string; output: string; status: 'running' | 'done' | 'error' }
  | { kind: 'notice'; id: string; tone: 'info' | 'warning' | 'error'; text: string; canContinue?: boolean };

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
