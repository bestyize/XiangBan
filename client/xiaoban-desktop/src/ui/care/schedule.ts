import { newCareRule, type CareConfig, type CareRule } from '../../care-model';

const pad = (value: number) => String(value).padStart(2, '0');
const clock = (date: Date) => `${pad(date.getHours())}:${pad(date.getMinutes())}`;
const monthDay = (date: Date) => `${date.getMonth() + 1}/${date.getDate()} ${clock(date)}`;

function parseClock(value: string | undefined): [number, number] | null {
  if (!value) return null;
  const [hour, minute] = value.split(':').map(Number);
  if (!Number.isFinite(hour) || !Number.isFinite(minute) || hour < 0 || hour > 23 || minute < 0 || minute > 59) return null;
  return [hour, minute];
}

export function formatLocalDateTime(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${clock(date)}`;
}

/**
 * Computes the next local trigger Date for a rule, or null if paused/finished/rest/invalid.
 */
export function nextRunDate(rule: CareRule, now = new Date()): Date | null {
  if (!rule.enabled || rule.kind === 'rest') return null;
  const floorMs = Math.max(now.getTime(), rule.notBefore * 1000);
  if (rule.repeat === 'once') {
    const at = new Date(rule.onceAt);
    if (!rule.onceAt || Number.isNaN(at.getTime())) return null;
    return at.getTime() > now.getTime() ? (at.getTime() < floorMs ? new Date(floorMs) : at) : null;
  }
  if (rule.repeat === 'interval') {
    const start = parseClock(rule.windowStart || '09:00');
    const end = parseClock(rule.windowEnd || '18:00');
    const step = Math.max(15, Math.min(360, Math.round(rule.intervalEveryMinutes ?? 90)));
    if (!start || !end) return null;
    const startMins = start[0] * 60 + start[1];
    const endMins = end[0] * 60 + end[1];
    if (startMins >= endMins) return null;
    for (let offset = 0; offset < 8; offset++) {
      const base = new Date(now.getFullYear(), now.getMonth(), now.getDate() + offset, 0, 0, 0, 0);
      const weekday = base.getDay() === 0 ? 7 : base.getDay();
      if (rule.weekdays.length && !rule.weekdays.includes(weekday)) continue;
      for (let mins = startMins; mins <= endMins; mins += step) {
        const candidate = new Date(base.getFullYear(), base.getMonth(), base.getDate(), Math.floor(mins / 60), mins % 60);
        if (candidate.getTime() > floorMs) return candidate;
      }
    }
    return null;
  }
  const parsed = parseClock(rule.time);
  if (!parsed) return null;
  for (let offset = 0; offset < 8; offset++) {
    const day = new Date(now.getFullYear(), now.getMonth(), now.getDate() + offset, parsed[0], parsed[1]);
    if (day.getTime() <= now.getTime()) continue;
    const weekday = day.getDay() === 0 ? 7 : day.getDay();
    if (rule.repeat === 'weekdays' && !rule.weekdays.includes(weekday)) continue;
    return day;
  }
  return null;
}

export function isRuleCompleted(rule: CareRule, now = new Date()): boolean {
  if (rule.kind === 'rest' || rule.repeat !== 'once' || !rule.onceAt) return false;
  const at = new Date(rule.onceAt);
  return !Number.isNaN(at.getTime()) && at.getTime() <= now.getTime();
}

export function isQuietNow(config: CareConfig, now = new Date()): boolean {
  if (!config.quietEnabled) return false;
  const start = parseClock(config.quietStart);
  const end = parseClock(config.quietEnd);
  if (!start || !end) return false;
  const cur = now.getHours() * 60 + now.getMinutes();
  const s = start[0] * 60 + start[1];
  const e = end[0] * 60 + end[1];
  if (s === e) return true;
  return s < e ? cur >= s && cur < e : cur >= s || cur < e;
}

export function formatCountdown(target: Date, now = new Date()): string {
  const diffMin = Math.max(1, Math.round((target.getTime() - now.getTime()) / 60_000));
  if (diffMin < 60) return `${diffMin} 分钟后`;
  const hours = Math.floor(diffMin / 60);
  const mins = diffMin % 60;
  if (hours < 24) return mins ? `${hours} 小时 ${mins} 分后` : `${hours} 小时后`;
  const days = Math.floor(hours / 24);
  return days === 1 ? '明天' : `${days} 天后`;
}

/**
 * Short label for the next time a rule will show, used as the badge on a reminder row.
 * It only reads the rule itself (no timers), so it is an estimate: quiet hours and activity-based
 * rest reminders are not simulated here.
 */
export function nextRunLabel(rule: CareRule, now = new Date()): string {
  if (!rule.enabled) return '已暂停';
  if (rule.kind === 'rest') return `累计 ${rule.intervalMinutes} 分钟`;
  if (rule.notBefore * 1000 > now.getTime()) return `稍后 ${monthDay(new Date(rule.notBefore * 1000))}`;
  if (rule.repeat === 'once') {
    const at = new Date(rule.onceAt);
    if (!rule.onceAt || Number.isNaN(at.getTime())) return '未设置';
    return at.getTime() > now.getTime() ? monthDay(at) : '已完成';
  }
  const next = nextRunDate(rule, now);
  if (!next) return '未设置';
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const nextDayStart = new Date(next.getFullYear(), next.getMonth(), next.getDate()).getTime();
  const dayOffset = Math.round((nextDayStart - todayStart) / 86_400_000);
  if (dayOffset === 0) return `今天 ${clock(next)}`;
  if (dayOffset === 1) return `明天 ${clock(next)}`;
  return monthDay(next);
}

export interface QuickParseResult {
  rule: CareRule;
  label: string;
  stickyHint: boolean;
}

/**
 * Parses a short Chinese natural-language reminder/task string into a valid CareRule draft.
 */
export function parseQuickCareInput(raw: string, isTask: boolean, config: CareConfig, now = new Date()): QuickParseResult | null {
  const text = raw.trim();
  if (!text) return null;
  const base = newCareRule(isTask ? 'agent' : 'custom');

  if (isTask) {
    const mTime = text.match(/(\d{1,2})[:点](\d{2})?/);
    const hh = mTime ? pad(Math.min(23, Number(mTime[1]))) : '09';
    const mm = mTime?.[2] ? pad(Math.min(59, Number(mTime[2]))) : (mTime ? '00' : '15');
    const isFriday = text.includes('周五');
    const isWorkday = text.includes('工作日');
    base.time = `${hh}:${mm}`;
    base.repeat = (isFriday || isWorkday) ? 'weekdays' : 'daily';
    base.weekdays = isFriday ? [5] : isWorkday ? [1, 2, 3, 4, 5] : [];
    base.text = text;
    base.sound = config.sound;
    base.voice = config.voice;
    base.sticky = false;
    return {
      rule: base,
      label: isFriday ? `每周五 ${base.time}` : isWorkday ? `工作日 ${base.time}` : `每天 ${base.time}`,
      stickyHint: false,
    };
  }

  const mAfterMin = text.match(/(\d+)\s*分钟后/);
  if (mAfterMin) {
    const mins = Math.max(1, Math.min(1440, Number(mAfterMin[1])));
    const target = new Date(now.getTime() + mins * 60_000);
    base.repeat = 'once';
    base.onceAt = formatLocalDateTime(target);
    base.time = clock(target);
    base.text = text.replace(mAfterMin[0], '').trim() || text;
    base.sound = config.sound || config.onceSound;
    base.voice = config.voice || config.onceVoice;
    base.sticky = true;
    return { rule: base, label: `${mins} 分钟后 · 仅一次`, stickyHint: true };
  }

  const mAfterHour = text.match(/(\d+)\s*小时后/);
  if (mAfterHour) {
    const hours = Math.max(1, Math.min(72, Number(mAfterHour[1])));
    const target = new Date(now.getTime() + hours * 3600_000);
    base.repeat = 'once';
    base.onceAt = formatLocalDateTime(target);
    base.time = clock(target);
    base.text = text.replace(mAfterHour[0], '').trim() || text;
    base.sound = config.sound || config.onceSound;
    base.voice = config.voice || config.onceVoice;
    base.sticky = true;
    return { rule: base, label: `${hours} 小时后 · 仅一次`, stickyHint: true };
  }

  const mEvery = text.match(/每\s*(\d+)\s*分钟/);
  if (mEvery) {
    const every = Math.max(15, Math.min(360, Number(mEvery[1])));
    const workdays = text.includes('工作日');
    base.repeat = 'interval';
    base.intervalEveryMinutes = every;
    base.windowStart = '09:00';
    base.windowEnd = '18:00';
    base.weekdays = workdays ? [1, 2, 3, 4, 5] : [];
    base.text = text.replace(mEvery[0], '').replace(/工作日/g, '').trim() || text;
    base.sound = config.sound;
    base.voice = config.voice;
    base.sticky = false;
    return { rule: base, label: `${workdays ? '工作日' : '每天'} 09:00–18:00 · 每 ${every} 分钟`, stickyHint: false };
  }

  const mClock = text.match(/(明天|今晚|今天|每天|工作日)?\s*(\d{1,2})[:点](\d{2})?/);
  if (mClock) {
    const hhNum = Math.min(23, Number(mClock[2]));
    const mmNum = mClock[3] ? Math.min(59, Number(mClock[3])) : 0;
    const prefix = mClock[1] || '';
    const hhmm = `${pad(hhNum)}:${pad(mmNum)}`;
    base.time = hhmm;
    base.text = text.replace(mClock[0], '').trim() || text;
    if (prefix === '明天' || prefix === '今晚' || prefix === '今天') {
      const target = new Date(now.getFullYear(), now.getMonth(), now.getDate() + (prefix === '明天' ? 1 : 0), hhNum, mmNum);
      if (target.getTime() <= now.getTime()) target.setDate(target.getDate() + 1);
      base.repeat = 'once';
      base.onceAt = formatLocalDateTime(target);
      base.sound = config.sound || config.onceSound;
      base.voice = config.voice || config.onceVoice;
      base.sticky = true;
      return { rule: base, label: `${prefix} ${hhmm} · 仅一次`, stickyHint: true };
    }
    if (prefix === '工作日') {
      base.repeat = 'weekdays';
      base.weekdays = [1, 2, 3, 4, 5];
      base.sound = config.sound;
      base.voice = config.voice;
      base.sticky = false;
      return { rule: base, label: `工作日 ${hhmm}`, stickyHint: false };
    }
    base.repeat = 'daily';
    base.sound = config.sound;
    base.voice = config.voice;
    base.sticky = false;
    return { rule: base, label: `每天 ${hhmm}`, stickyHint: false };
  }

  const fallback = new Date(now.getTime() + 60 * 60_000);
  base.repeat = 'once';
  base.onceAt = formatLocalDateTime(fallback);
  base.time = clock(fallback);
  base.text = text;
  base.sound = config.sound || config.onceSound;
  base.voice = config.voice || config.onceVoice;
  base.sticky = true;
  return { rule: base, label: '1 小时后 · 仅一次', stickyHint: true };
}
