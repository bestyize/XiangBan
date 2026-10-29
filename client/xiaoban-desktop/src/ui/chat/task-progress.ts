import type { ReplyBlock, ToolCall } from './replies.ts';

export interface TaskStep { content: string; status: 'pending' | 'in_progress' | 'completed'; evidence: string }
export interface TaskPlan { steps: TaskStep[]; handoff: string; callId: string }
export type TaskState = 'running' | 'waiting' | 'approval' | 'blocked' | 'paused' | 'completed' | 'partial' | 'failed';
export interface TaskProgress {
  plan: TaskPlan; state: TaskState; completed: number; current: number; message: string; canResume: boolean;
}

/** Only an acknowledged todo call changes the displayed plan. Proposed or failed updates do not. */
export function taskPlan(calls: ToolCall[]): TaskPlan | null {
  for (let index = calls.length - 1; index >= 0; index--) {
    const call = calls[index];
    if (call.name !== 'todo' || call.status !== 'done' || !call.output.startsWith('Todo list updated:')) continue;
    try {
      const args: unknown = JSON.parse(call.arguments);
      if (!args || typeof args !== 'object' || !('todos' in args) || !Array.isArray(args.todos) || args.todos.length > 50) continue;
      const steps: TaskStep[] = [];
      let valid = true;
      for (const item of args.todos) {
        if (!item || typeof item !== 'object' || typeof item.content !== 'string' || !item.content.trim()
          || [...item.content.trim()].length > 300 || !['pending', 'in_progress', 'completed'].includes(item.status)
          || (item.evidence !== undefined && (typeof item.evidence !== 'string' || [...item.evidence].length > 1000))) { valid = false; break; }
        steps.push({ content: item.content.trim(), status: item.status, evidence: item.evidence ?? '' });
      }
      const handoff = 'handoff' in args ? args.handoff : '';
      if (!valid || steps.filter(step => step.status === 'in_progress').length > 1
        || typeof handoff !== 'string' || [...handoff].length > 2000) continue;
      return steps.length ? { steps, handoff, callId: call.id } : null;
    } catch { /* An incomplete or malformed call is never a plan checkpoint. */ }
  }
  return null;
}

function operation(call: ToolCall): string {
  const canonical = (value: unknown): unknown => Array.isArray(value) ? value.map(canonical)
    : value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).sort(([a], [b]) => a.localeCompare(b)).map(([k,v]) => [k, canonical(v)])) : value;
  try { return `${call.name}:${JSON.stringify(canonical(JSON.parse(call.arguments)))}`; }
  catch { return `${call.name}:${call.arguments}`; }
}

export function unresolvedCalls(calls: ToolCall[]): ToolCall[] {
  const unresolved = new Map<string, ToolCall>();
  for (const call of calls) {
    if (call.name === 'todo') continue;
    if (call.status === 'error') unresolved.set(operation(call), call);
    else if (call.status === 'done') unresolved.delete(operation(call));
  }
  return [...unresolved.values()];
}

export function taskProgress(block: ReplyBlock, active: boolean, waiting: 'question' | 'approval' | null = null, status = ''): TaskProgress | null {
  const plan = taskPlan(block.calls);
  if (!plan) return null;
  const completed = plan.steps.filter(step => step.status === 'completed').length;
  const working = plan.steps.findIndex(step => step.status === 'in_progress');
  const current = working >= 0 ? working : plan.steps.findIndex(step => step.status === 'pending');
  const notice = [...block.items].reverse().find(item => item.kind === 'notice' && (item.tone !== 'info' || item.canContinue || item.text === '已停止'));
  const unresolved = unresolvedCalls(block.calls);
  const outcome = [...block.items].reverse().find(item => item.kind === 'notice' && item.taskStatus);
  const taskStatus = outcome?.kind === 'notice' ? outcome.taskStatus : undefined;
  let state: TaskState;
  if (waiting) state = waiting === 'question' ? 'waiting' : 'approval';
  else if (active) state = 'running';
  else if (taskStatus === 'cancelled' || notice?.text === '已停止') state = 'paused';
  else if (taskStatus === 'partial') state = 'partial';
  else if (taskStatus === 'failed') state = 'failed';
  else if (notice || unresolved.length > 0 || block.calls.some(call => call.status === 'running')) state = 'blocked';
  else state = completed === plan.steps.length ? 'completed' : 'paused';
  const message = state === 'waiting' ? '回答下方问题后，小伴会继续执行。'
    : state === 'approval' ? '请在下方确认本次操作，计划不需要另行审批。'
    : state === 'blocked' || state === 'partial' || state === 'failed' ? (notice?.kind === 'notice' ? notice.text : '有操作未完成，可展开执行详情查看原因。')
    : state === 'paused' ? '已暂停，当前进度已保留。'
    : state === 'completed' ? '所有步骤已完成'
    : status || (current < 0 ? '正在汇总结果与交付文件…' : '按计划继续执行');
  const canResume = !active && state !== 'completed' && (!notice || notice.kind === 'notice' && (notice.text === '已停止' || !!notice.canContinue));
  return { plan, state, completed, current, message, canResume };
}
