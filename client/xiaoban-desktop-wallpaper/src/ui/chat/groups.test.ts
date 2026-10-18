import { describe, expect, it } from 'vitest';
import { listRows, type ListRow } from './groups.ts';
import type { ChatGroup, SessionSummary } from './types.ts';

const session = (id: string, group?: string): SessionSummary => ({ id, title: id, createdAt: 1, updatedAt: 1, group });
const work: ChatGroup = { id: 'g-work', name: '工作' };
const life: ChatGroup = { id: 'g-life', name: '生活' };

const shape = (rows: ListRow[]) => rows.map(row => {
  switch (row.kind) {
    case 'group': return `group:${row.group.name}(${row.count})${row.collapsed ? '-' : ''}`;
    case 'session': return `${row.grouped ? '  ' : ''}${row.session.id}`;
    case 'empty': return `  (empty ${row.groupId})`;
    case 'label': return `label:${row.text}`;
  }
});

describe('the rows of the conversation list', () => {
  it('is just the conversations, in the host order, when there are no groups', () => {
    expect(shape(listRows([session('a'), session('b')], [], new Set()))).toEqual(['a', 'b']);
  });

  it('puts each group first with its conversations, then the rest under a label', () => {
    const rows = listRows([session('a', 'g-life'), session('b'), session('c', 'g-work'), session('d', 'g-life')], [work, life], new Set());
    expect(shape(rows)).toEqual(['group:工作(1)', '  c', 'group:生活(2)', '  a', '  d', 'label:未分组', 'b']);
  });

  it('hides the conversations of a collapsed group but still counts them', () => {
    const rows = listRows([session('a', 'g-work'), session('b', 'g-work'), session('c')], [work], new Set(['g-work']));
    expect(shape(rows)).toEqual(['group:工作(2)-', 'label:未分组', 'c']);
  });

  it('shows an empty group as a hint, and skips the label when everything is grouped', () => {
    expect(shape(listRows([session('a', 'g-life')], [work, life], new Set()))).toEqual(['group:工作(0)', '  (empty g-work)', 'group:生活(1)', '  a']);
    expect(shape(listRows([], [work], new Set(['g-work'])))).toEqual(['group:工作(0)-']);
  });

  it('treats a conversation whose group is gone as ungrouped', () => {
    const rows = listRows([session('a', 'g-deleted'), session('b', '')], [work], new Set());
    expect(shape(rows)).toEqual(['group:工作(0)', '  (empty g-work)', 'label:未分组', 'a', 'b']);
  });
});
