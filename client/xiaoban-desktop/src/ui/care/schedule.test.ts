import { describe, expect, it } from 'vitest';
import { newCareRule, type CareConfig } from '../../care-model';
import { nextRunLabel, parseQuickCareInput } from './schedule';

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
  it('computes the next slot for window interval reminders', () => {
    const rule = {
      ...newCareRule('custom'),
      repeat: 'interval' as const,
      intervalEveryMinutes: 90,
      windowStart: '09:00',
      windowEnd: '18:00',
      weekdays: [1, 2, 3, 4, 5],
    };
    expect(nextRunLabel(rule, at(2026, 10, 22, 10, 0))).toBe('今天 10:30');
    expect(nextRunLabel(rule, at(2026, 10, 22, 18, 5))).toBe('明天 09:00');
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

describe('parseQuickCareInput', () => {
  it('parses relative minutes and windowed intervals cleanly', () => {
    const config: CareConfig = {
      enabled: true, sound: false, onceSound: true, voice: false, onceVoice: true,
      quietEnabled: false, quietStart: '22:00', quietEnd: '08:00',
      weatherSource: { provider: 'open-meteo', forecastUrl: '', geocodingUrl: '' },
    };
    const now = at(2026, 10, 22, 10, 0);
    const once = parseQuickCareInput('30分钟后 关火', false, config, now)!;
    expect(once.rule.repeat).toBe('once');
    expect(once.rule.onceAt).toBe('2026-10-22T10:30');
    expect(once.rule.text).toBe('关火');
    expect(once.rule.sticky).toBe(true);

    const interval = parseQuickCareInput('工作日每90分钟 喝水', false, config, now)!;
    expect(interval.rule.repeat).toBe('interval');
    expect(interval.rule.intervalEveryMinutes).toBe(90);
    expect(interval.rule.weekdays).toEqual([1, 2, 3, 4, 5]);
    expect(interval.rule.text).toBe('喝水');
  });
});
