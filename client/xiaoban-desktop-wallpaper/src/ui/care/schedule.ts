import type { CareRule } from '../../care-model';

const pad = (value: number) => String(value).padStart(2, '0');
const clock = (date: Date) => `${pad(date.getHours())}:${pad(date.getMinutes())}`;
const monthDay = (date: Date) => `${date.getMonth() + 1}/${date.getDate()} ${clock(date)}`;

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
  const [hour, minute] = rule.time.split(':').map(Number);
  if (!Number.isFinite(hour) || !Number.isFinite(minute)) return '未设置';
  for (let offset = 0; offset < 8; offset++) {
    const day = new Date(now.getFullYear(), now.getMonth(), now.getDate() + offset, hour, minute);
    if (day.getTime() <= now.getTime()) continue;
    const weekday = day.getDay() === 0 ? 7 : day.getDay();
    if (rule.repeat === 'weekdays' && !rule.weekdays.includes(weekday)) continue;
    if (offset === 0) return `今天 ${clock(day)}`;
    if (offset === 1) return `明天 ${clock(day)}`;
    return monthDay(day);
  }
  return '未设置';
}
