import type { ChatGroup, SessionSummary } from './types.ts';

/** One line of the conversation list, top to bottom. */
export type ListRow =
  | { kind: 'group'; group: ChatGroup; count: number; collapsed: boolean }
  | { kind: 'session'; session: SessionSummary; grouped: boolean }
  | { kind: 'empty'; groupId: string }
  | { kind: 'label'; text: string };

/**
 * Groups first, each with its conversations, then the conversations that are in no group.
 * Within a section the order is the host's (newest first). A conversation that names a group
 * which no longer exists counts as ungrouped, so nothing can disappear from the list.
 */
export function listRows(sessions: readonly SessionSummary[], groups: readonly ChatGroup[], collapsed: ReadonlySet<string>): ListRow[] {
  const known = new Set(groups.map(g => g.id));
  const loose = sessions.filter(s => !s.group || !known.has(s.group));
  const rows: ListRow[] = [];
  for (const group of groups) {
    const inside = sessions.filter(s => s.group === group.id);
    const isCollapsed = collapsed.has(group.id);
    rows.push({ kind: 'group', group, count: inside.length, collapsed: isCollapsed });
    if (isCollapsed) continue;
    if (!inside.length) rows.push({ kind: 'empty', groupId: group.id });
    for (const session of inside) rows.push({ kind: 'session', session, grouped: true });
  }
  if (groups.length && loose.length) rows.push({ kind: 'label', text: '未分组' });
  for (const session of loose) rows.push({ kind: 'session', session, grouped: false });
  return rows;
}
