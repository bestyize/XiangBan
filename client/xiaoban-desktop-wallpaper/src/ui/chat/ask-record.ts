import type { ToolCall } from './replies.ts';

export interface AskRecord { id: string; question: string; answer: string | null }

const ANSWERED = 'The user answered: ';
const DISMISSED = 'The user dismissed the question';

/** The questions the user has settled in this reply, read from the finished `ask_user` calls. */
export function askRecords(calls: ToolCall[]): AskRecord[] {
  const records: AskRecord[] = [];
  for (const call of calls) {
    if (call.name !== 'ask_user' || call.status !== 'done') continue;
    let question = '';
    try { const text = (JSON.parse(call.arguments) as { question?: unknown }).question; question = typeof text === 'string' ? text.trim() : ''; } catch { /* a half-written call has no question to show */ }
    if (!question) continue;
    if (call.output.startsWith(ANSWERED)) records.push({ id: call.id, question, answer: call.output.slice(ANSWERED.length).trim() });
    else if (call.output.startsWith(DISMISSED)) records.push({ id: call.id, question, answer: null });
  }
  return records;
}
