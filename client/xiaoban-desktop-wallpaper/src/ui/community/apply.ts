import { reactive } from 'vue';
import { invoke } from '../../bridge.ts';

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

export async function confirmApply(): Promise<string | null> {
  const current = applyFlow.state;
  if (current.phase !== 'confirm') return null;
  applyFlow.state = { phase: 'applying', preview: current.preview };
  try {
    const result = await invoke<{ message: string }>('community_apply', { token: current.preview.token });
    applyFlow.state = { phase: 'done', message: result.message };
    return result.message;
  } catch (error) {
    applyFlow.state = { phase: 'error', message: text(error) };
    return null;
  }
}

export function closeApply() { applyFlow.state = { phase: 'idle' }; }

export const KIND_LABEL: Record<string, string> = {
  effect: '特效', reminder: '提醒', task: '定时任务', routine: '例行事项', goal: '目标',
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
