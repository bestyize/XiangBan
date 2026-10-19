import { dateKey } from './calendar.ts';
import type { FlowerData } from './model.ts';

/** The six key frames of the white flower, from first sprout to a full cluster. */
export const FLOWER_STAGES = [
  { id: 'sprout', label: '初生', note: '两片小叶刚刚探出泥土' },
  { id: 'leafy', label: '舒展', note: '叶子一片片舒展开来' },
  { id: 'bud', label: '含苞', note: '悄悄含起一个小小的花苞' },
  { id: 'open', label: '初绽', note: '第一朵小白花开了' },
  { id: 'bloom', label: '盛放', note: '三朵花并肩开放' },
  { id: 'garden', label: '满庭', note: '小白花开满了枝头' },
] as const;

/** Total waterings needed to reach each stage. The last one is also the cap. */
export const FLOWER_THRESHOLDS = [0, 3, 7, 12, 18, 25] as const;
export const FLOWER_MAX_WATER = FLOWER_THRESHOLDS[FLOWER_THRESHOLDS.length - 1];

export const emptyFlower = (now = new Date()): FlowerData => ({ water: 0, since: dateKey(now) });

/** Index of the current stage for a watering count. */
export function flowerStage(water: number) {
  const value = Number.isFinite(water) ? Math.floor(water) : 0;
  let stage = 0;
  for (let index = 0; index < FLOWER_THRESHOLDS.length; index++) if (value >= FLOWER_THRESHOLDS[index]) stage = index;
  return stage;
}

/** Waterings still needed to reach the next stage; zero once it is in full bloom. */
export function waterUntilNext(water: number) {
  const next = FLOWER_THRESHOLDS[flowerStage(water) + 1];
  return next === undefined ? 0 : Math.max(0, next - Math.max(0, Math.floor(water)));
}

/** How far through the whole growth the flower is, 0..1. */
export function flowerProgress(water: number) {
  return Math.min(1, Math.max(0, Math.floor(water)) / FLOWER_MAX_WATER);
}

/** Whole days since planting, starting at 1 on the day it was planted. */
export function flowerDays(since: string, now = new Date()) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(since)) return 1;
  const [year, month, day] = since.split('-').map(Number);
  const planted = Date.UTC(year, month - 1, day);
  const today = Date.UTC(now.getFullYear(), now.getMonth(), now.getDate());
  return Math.max(1, Math.floor((today - planted) / 86400000) + 1);
}

export function validateFlower(data: FlowerData) {
  if (!data || !Number.isInteger(data.water) || data.water < 0 || data.water > 999) throw new Error('小白花的浇水记录无效');
  if (typeof data.since !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(data.since) || data.since < '1900-01-01' || data.since > '2100-12-31') throw new Error('小白花的种植日期无效');
  const [year, month, day] = data.since.split('-').map(Number);
  const date = new Date(year, month - 1, day, 12);
  if (dateKey(date) !== data.since) throw new Error('小白花的种植日期无效');
}
