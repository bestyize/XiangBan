export type Tone = 'ok' | 'warn' | 'bad' | 'info' | 'idle';

export const JOB_STATUS: Record<string, [string, Tone]> = {
  queued: ['排队中', 'idle'],
  running: ['进行中', 'info'],
  succeeded: ['成功', 'ok'],
  failed: ['失败', 'bad'],
  cancelled: ['已取消', 'idle'],
};
export const ITEM_STATUS: Record<string, [string, Tone]> = {
  draft: ['草稿', 'idle'],
  review: ['待审', 'warn'],
  published: ['已上架', 'ok'],
  offline: ['已下架', 'bad'],
};
export const STAGED_STATUS: Record<string, [string, Tone]> = {
  pending: ['待处理', 'warn'],
  imported: ['已入库', 'ok'],
  ignored: ['已忽略', 'idle'],
};
export const CHANGE: Record<string, [string, Tone]> = {
  new: ['新增', 'ok'],
  changed: ['有更新', 'info'],
  failed: ['入库失败', 'bad'],
};
export const SOURCE_LABEL: Record<string, string> = { official: '官方', community: '社区', enterprise: '企业' };
export const ORIGIN_LABEL: Record<string, string> = { skillhub: 'SkillHub', modelscope: '魔搭', manual: '手动' };
export const TRIGGER_LABEL: Record<string, string> = { manual: '手动', schedule: '定时' };
export const INTERVALS: [number, string][] = [
  [0, '仅手动'], [60, '每 1 小时'], [180, '每 3 小时'], [360, '每 6 小时'], [720, '每 12 小时'], [1440, '每天'], [10080, '每周'],
];
export const AUTO_IMPORT: [string, string][] = [['none', '仅进暂存区'], ['draft', '自动入库为草稿'], ['published', '自动入库并上架']];

export function intervalLabel(minutes: number): string {
  return INTERVALS.find(([m]) => m === minutes)?.[1] ?? `每 ${minutes} 分钟`;
}
