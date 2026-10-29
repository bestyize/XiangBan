import { reactive } from 'vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

/** What `community_prepare` reports about a checked pack. Nothing has been changed on this machine at this point. */
export interface PackPreview {
  token: string;
  packId: number;
  kind: string;
  title: string;
  version: string;
  trustLevel: number;
  canApply: boolean;
  size: number;
  sha256: string;
  items: { title: string; detail?: string | null; schedule?: { repeat?: string; time?: string; weekdays?: number[]; intervalMinutes?: number } }[];
  startsDisabled: boolean;
}

export interface ApplyRequest { packId: number; version?: number | null; sha256?: string | null }

export type ApplyState =
  | { phase: 'idle' }
  | { phase: 'loading'; request: ApplyRequest }
  | { phase: 'confirm'; preview: PackPreview }
  | { phase: 'applying'; preview: PackPreview }
  | { phase: 'done'; message: string }
  | { phase: 'error'; message: string };

/** One shared flow for the link dialog and the community page: check first, apply only after the user says yes. */
export const applyFlow = reactive<{ state: ApplyState }>({ state: { phase: 'idle' } });

const text = (error: unknown) => (error instanceof Error ? error.message : typeof error === 'string' ? error : '操作失败，请稍后重试');

export async function startApply(request: ApplyRequest) {
  applyFlow.state = { phase: 'loading', request };
  try {
    const preview = await invoke<PackPreview>('community_prepare', {
      packId: request.packId, version: request.version ?? null, sha256: request.sha256 ?? null,
    });
    applyFlow.state = { phase: 'confirm', preview };
  } catch (error) {
    applyFlow.state = { phase: 'error', message: text(error) };
  }
}

/** What the console does with a template that is handed over rather than applied. */
export interface ApplyHandlers {
  task(prompt: string): Promise<void> | void;
  goal(goal: NonNullable<ControlStore['goalTemplate']>): Promise<void> | void;
}

interface ApplyResult {
  status: 'applied' | 'task' | 'goal';
  message: string;
  task?: { prompt: string };
  goal?: NonNullable<ControlStore['goalTemplate']>;
}

export async function confirmApply(handlers?: ApplyHandlers): Promise<string | null> {
  const current = applyFlow.state;
  if (current.phase !== 'confirm') return null;
  applyFlow.state = { phase: 'applying', preview: current.preview };
  try {
    const result = await invoke<ApplyResult>('community_apply', { token: current.preview.token });
    if (result.status === 'task' && result.task) await handlers?.task(result.task.prompt);
    if (result.status === 'goal' && result.goal) await handlers?.goal(result.goal);
    applyFlow.state = { phase: 'done', message: result.message };
    return result.message;
  } catch (error) {
    applyFlow.state = { phase: 'error', message: text(error) };
    return null;
  }
}

/** 桌面端现在能应用的类型。 */
export const SUPPORTED_KINDS = ['effect', 'reminder', 'routine', 'task', 'goal'];

/**
 * 复刻方案：把方案里的内容逐项走同一条“下载 → 校验 → 展示 → 确认”流程，一次只处理一项，
 * 每一项都要用户点头；桌面端还不支持的类型跳过，下架的内容会在那一步报错，不影响后面的。
 */
export const schemeFlow = reactive<{ title: string; queue: ApplyRequest[]; total: number; skipped: number }>({ title: '', queue: [], total: 0, skipped: 0 });

export interface SchemeItem { pack_id: number; kind: string; version?: number }

export function startScheme(title: string, items: SchemeItem[]): number {
  const supported = items.filter(i => SUPPORTED_KINDS.includes(i.kind));
  trackEvent('desktop_scheme_replicate');
  schemeFlow.title = title;
  schemeFlow.total = supported.length;
  schemeFlow.skipped = items.length - supported.length;
  schemeFlow.queue = supported.map(i => ({ packId: i.pack_id, version: i.version ?? null }));
  nextInScheme();
  return supported.length;
}

function nextInScheme() {
  const next = schemeFlow.queue.shift();
  if (next) void startApply(next);
}

/** 第几项 / 共几项，不在复刻中返回空。 */
export function schemeProgress(): string {
  if (!schemeFlow.total || applyFlow.state.phase === 'idle') return '';
  return `方案「${schemeFlow.title}」第 ${schemeFlow.total - schemeFlow.queue.length} / ${schemeFlow.total} 项`;
}

export function stopScheme() {
  schemeFlow.queue = [];
  schemeFlow.total = 0;
  applyFlow.state = { phase: 'idle' };
}

/** 匿名统计（失败不影响任何操作）：只报事件名和内容类型。 */
export function trackEvent(name: string, dim?: string) {
  void (async () => { try { await invoke('community_track', { name, dim: dim ?? null }); } catch { /* 统计失败不影响任何操作 */ } })();
}

export function closeApply() {
  // 在确认窗口里取消（没点应用就关掉）算一次“取消”。
  if (applyFlow.state.phase === 'confirm') trackEvent('desktop_apply_cancel', applyFlow.state.preview.kind);
  applyFlow.state = { phase: 'idle' };
  if (schemeFlow.queue.length) nextInScheme();
  else schemeFlow.total = 0;
}

export const KIND_LABEL: Record<string, string> = {
  effect: '特效', reminder: '提醒', task: '任务模板', routine: '例行事项', goal: '目标',
  widget: '组件', game: '游戏', wallpaper: '壁纸', skill: '技能', expert: '专家', workflow: '工作流',
};
export const TRUST_LABEL = ['官方', '已审核', '未审核'];

const WEEKDAYS = ['一', '二', '三', '四', '五', '六', '日'];
export function scheduleText(s?: PackPreview['items'][number]['schedule']): string {
  if (!s) return '';
  if (s.intervalMinutes && !s.time) return `每 ${s.intervalMinutes} 分钟`;
  if (s.repeat === 'weekdays' && s.weekdays?.length) return `每周${s.weekdays.map(d => WEEKDAYS[d - 1] ?? d).join('、')} ${s.time ?? ''}`.trim();
  if (s.repeat === 'once') return `一次 ${s.time ?? ''}`.trim();
  if (s.repeat === 'daily') return `每天 ${s.time ?? ''}`.trim();
  return s.time ?? '';
}
