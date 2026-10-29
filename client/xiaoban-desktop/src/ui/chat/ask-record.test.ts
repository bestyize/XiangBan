import { describe, expect, it } from 'vitest';
import { askRecords } from './ask-record.ts';
import type { ToolCall } from './replies.ts';

const call = (over: Partial<ToolCall>): ToolCall => ({ kind: 'tool', id: 't1', name: 'ask_user', arguments: JSON.stringify({ question: '放哪里？' }), output: '', status: 'done', ...over });

describe('askRecords', () => {
  it('turns finished questions into answered and skipped records', () => {
    const records = askRecords([
      call({ id: 'a', output: 'The user answered: 桌面' }),
      call({ id: 'b', output: 'The user dismissed the question without answering.' }),
    ]);
    expect(records).toEqual([{ id: 'a', question: '放哪里？', answer: '桌面' }, { id: 'b', question: '放哪里？', answer: null }]);
  });

  it('ignores other tools, running or failed questions and broken arguments', () => {
    expect(askRecords([
      call({ name: 'read_file', output: 'The user answered: x' }),
      call({ status: 'running' }),
      call({ status: 'error', output: 'cancelled' }),
      call({ arguments: '{"quest', output: 'The user answered: x' }),
    ])).toEqual([]);
  });
});
