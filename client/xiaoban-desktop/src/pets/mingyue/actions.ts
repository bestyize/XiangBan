import type { ExpressionName } from '../../companion/expressions.ts';

export const actionLabels = {
  idle: '待机', walk: '散步', read: '捧书', 'bench-read': '坐着读书', write: '写字',
  listen: '听潮', look: '张望', contact: '打招呼', pet: '伸手轻抚', play: '玩游戏',
  paint: '画画', water: '浇水', swing: '荡秋千', fish: '钓鱼', bench: '坐下', sleep: '睡觉',
} as const;
export type MingyueClip = keyof typeof actionLabels;
export type PetMotion = 'auto' | MingyueClip;
export const isMingyue = (theme?: string, pet?: string) => theme === 'tide-study-mingyue' && pet === 'mingyue';
export const isPetMotion = (value: unknown): value is PetMotion =>
  value === 'auto' || (typeof value === 'string' && Object.hasOwn(actionLabels, value));

const moods: Partial<Record<ExpressionName, MingyueClip>> = {
  thinking: 'read', waiting: 'bench-read', curious: 'look', listening: 'listen',
  greeting: 'contact', speaking: 'contact', surprised: 'look', confused: 'look',
  shy: 'pet', love: 'pet', happy: 'pet', laughing: 'play', excited: 'play',
  sleepy: 'bench', sleeping: 'sleep',
};
export const clipForMood = (expression: ExpressionName): MingyueClip => moods[expression] ?? 'idle';
