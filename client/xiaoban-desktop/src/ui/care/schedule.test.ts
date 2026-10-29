import { describe, expect, it } from 'vitest';
import { newCareRule } from '../../care-model';
import { nextRunLabel } from './schedule';

const at = (y: number, m: number, d: number, h = 0, min = 0) => new Date(y, m - 1, d, h, min);

describe('nextRunLabel', () => {
  it('shows today or tomorrow for a daily reminder', () => {
    const rule = { ...newCareRule('custom'), time: '15:00' };
    expect(nextRunLabel(rule, at(2026, 10, 22, 10))).toBe('今天 15:00');
    expect(nextRunLabel(rule, at(2026, 10, 22, 16))).toBe('明天 15:00');
  });
  it('skips days that are not selected for weekday reminders', () => {
    // 2026-10-22 is a Thursday; only Monday (1) is selected, so the next one is 10-26.
    const rule = { ...newCareRule('custom'), repeat: 'weekdays' as const, weekdays: [1], time: '09:00' };
    expect(nextRunLabel(rule, at(2026, 10, 22, 10))).toBe('10/26 09:00');
  });
  it('reports paused, finished and snoozed rules', () => {
    const rule = { ...newCareRule('custom'), repeat: 'once' as const, onceAt: '2026-10-21T09:00' };
    expect(nextRunLabel({ ...rule, enabled: false }, at(2026, 10, 22))).toBe('已暂停');
    expect(nextRunLabel(rule, at(2026, 10, 22))).toBe('已完成');
    const snoozed = { ...newCareRule('custom'), notBefore: at(2026, 10, 22, 11).getTime() / 1000 };
    expect(nextRunLabel(snoozed, at(2026, 10, 22, 10))).toBe('稍后 10/22 11:00');
  });
  it('describes rest reminders by accumulated usage', () => {
    expect(nextRunLabel(newCareRule('rest'), at(2026, 10, 22))).toBe('累计 60 分钟');
  });
});
