export interface CareCity {
  name: string;
  country: string;
  admin1: string;
  latitude: number;
  longitude: number;
}
export interface CareSource {
  provider: "disabled" | "open-meteo" | "custom";
  forecastUrl: string;
  geocodingUrl: string;
}
export interface CareConfig {
  enabled: boolean;
  sound: boolean;
  /** One-time reminders sound even when `sound` is off. */
  onceSound: boolean;
  voice: boolean;
  /** One-time reminders are read aloud even when `voice` is off. */
  onceVoice: boolean;
  quietEnabled: boolean;
  quietStart: string;
  quietEnd: string;
  weatherSource: CareSource;
}
export type CareKind = "custom" | "off-work" | "rest" | "weather" | "agent";
export type CareRepeat = "once" | "daily" | "weekdays" | "interval";
export interface CareRule {
  id: string;
  revision: number;
  enabled: boolean;
  kind: CareKind;
  repeat: CareRepeat;
  time: string;
  onceAt: string;
  weekdays: number[];
  intervalMinutes: number;
  /** For `repeat === "interval"`: fire every N minutes inside `[windowStart, windowEnd]`. */
  intervalEveryMinutes?: number;
  windowStart?: string;
  windowEnd?: string;
  text: string;
  city: CareCity | null;
  notBefore: number;
  /** Per-rule delivery overrides; `null`/`undefined` falls back to global config. */
  sound?: boolean | null;
  voice?: boolean | null;
  sticky?: boolean | null;
}
export interface CareRuleNotify {
  sound: boolean;
  voice: boolean;
  sticky: boolean;
}
export interface CareWeather {
  city: CareCity;
  text: string;
  updatedAt: number;
  modelAt: number;
  source: string;
}
export interface CareStatus {
  config: CareConfig;
  rules: CareRule[];
  pendingCount: number;
  weather: CareWeather | null;
  weatherError: string | null;
  error: string | null;
  systemNotificationsAvailable: boolean;
  activityAvailable: boolean;
  /** The last run of each scheduled task, by rule id. */
  agentRuns?: Record<string, CareAgentRun>;
}
export interface CareAgentRun { at: number; ok: boolean; summary: string; session: string | null; inputTokens: number; outputTokens: number; count: number }
const tokens = (n: number) => n >= 10_000 ? `${Math.round(n / 1000)}k` : n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n);
/** One line for the care page: when it last ran, whether it finished, and what it cost. */
export function describeAgentRun(run: CareAgentRun, now = new Date()): string {
  const at = new Date(run.at * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  const clock = `${pad(at.getHours())}:${pad(at.getMinutes())}`;
  const day = at.toDateString() === now.toDateString() ? `今天 ${clock}` : `${at.getMonth() + 1}月${at.getDate()}日 ${clock}`;
  const cost = run.inputTokens || run.outputTokens ? ` · 输入 ${tokens(run.inputTokens)} / 输出 ${tokens(run.outputTokens)} token` : '';
  return `上次运行：${day} · ${run.ok ? '已完成' : '没有完成'}${cost} · 共 ${run.count} 次`;
}
export interface CarePending {
  key: string;
  ruleId: string;
  revision: number;
  kind: CareKind;
  text: string;
  dueAt: number;
  availableAt: number;
  expiresAt: number;
  preview: boolean;
  weatherReady: boolean;
  sticky?: boolean;
  sound?: boolean | null;
  voice?: boolean | null;
  session?: string | null;
}
export interface CareDelivery {
  id: string;
  items: CarePending[];
  sound: boolean;
  voice: boolean;
  displaySeconds: number;
}
export const careNames: Record<CareKind, string> = {
  custom: "自定义", "off-work": "下班", rest: "休息", weather: "天气", agent: "定时任务",
};
export const dayNames = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

/** Resolves effective sound / voice / sticky for a single rule, falling back to global config. */
export function resolveRuleNotify(rule: CareRule, config: CareConfig): CareRuleNotify {
  const defaultSticky = rule.repeat === "once" && rule.kind === "custom";
  const sticky = typeof rule.sticky === "boolean" ? rule.sticky : defaultSticky;
  const sound = typeof rule.sound === "boolean" ? rule.sound : (config.sound || (sticky && config.onceSound));
  const voice = typeof rule.voice === "boolean" ? rule.voice : (config.voice || (sticky && config.onceVoice));
  return { sound, voice, sticky };
}

export function describeRuleNotify(notify: CareRuleNotify): string {
  const parts: string[] = [];
  if (notify.sound) parts.push("提示音");
  if (notify.voice) parts.push("语音朗读");
  if (notify.sticky) parts.push("常驻气泡");
  return parts.length ? parts.join(" + ") : "静音轻气泡";
}

export function newCareRule(kind: CareKind = "custom"): CareRule {
  return {
    id: "", revision: 0, enabled: true, kind, repeat: "daily",
    time: kind === "off-work" ? "18:00" : kind === "weather" || kind === "agent" ? "08:00" : "15:00",
    onceAt: "", weekdays: [], intervalMinutes: 60,
    intervalEveryMinutes: 90, windowStart: "09:00", windowEnd: "18:00",
    city: null, notBefore: 0, sound: null, voice: null, sticky: null,
    text: kind === "off-work" ? "到点啦，今天也辛苦了。" : kind === "rest" ? "起来走走，看看远处吧。" : kind === "weather" ? ""
      : kind === "agent" ? "看看工作目录里昨天新增或修改了哪些文件，用三句话总结。" : "记得喝水。",
  };
}

function formatWeekdays(weekdays: number[]): string {
  const unique = [...new Set(weekdays)].sort((a, b) => a - b);
  if (unique.length === 0 || unique.length === 7) return "每天";
  if (unique.join(",") === "1,2,3,4,5") return "工作日";
  if (unique.join(",") === "6,7") return "周末";
  return unique.map(day => dayNames[day - 1]).filter(Boolean).join("、");
}

export function describeCareRule(rule: CareRule): string {
  if (rule.kind === "rest") return `有效使用电脑 ${rule.intervalMinutes} 分钟`;
  if (rule.repeat === "once") return rule.onceAt.replace("T", " ");
  if (rule.repeat === "interval") {
    const every = rule.intervalEveryMinutes ?? 90;
    const start = rule.windowStart || "09:00";
    const end = rule.windowEnd || "18:00";
    return `${formatWeekdays(rule.weekdays)} ${start}–${end} · 每 ${every} 分钟`;
  }
  const repeat = rule.repeat === "daily" ? "每天" : [...new Set(rule.weekdays)].sort((a, b) => a - b).map(day => dayNames[day - 1]).filter(Boolean).join("、");
  return `${repeat} ${rule.time}${rule.kind === "weather" && rule.city ? ` · ${rule.city.name}` : ""}`;
}
/** How long the text of a rule may be: an agent task's instructions may say more than a reminder. */
export const careTextLimit = (kind: CareKind) => kind === "agent" ? 1000 : 240;
const CLOCK_RE = /^([01]\d|2[0-3]):[0-5]\d$/;
export function careRuleError(rule: CareRule): string | null {
  if (rule.kind === "agent" && !rule.text.trim()) return "请写下要小伴定时做的事";
  if (Array.from(rule.text).length > careTextLimit(rule.kind)) return rule.kind === "agent" ? "任务说明最多 1000 个字" : "提醒文案最多 240 个字";
  if (rule.kind !== "weather" && !rule.text.trim()) return "请填写提醒文案";
  if (rule.kind === "rest") {
    if (!Number.isInteger(rule.intervalMinutes) || rule.intervalMinutes < 5 || rule.intervalMinutes > 480) return "休息间隔应为 5–480 分钟";
  } else if (rule.repeat === "interval") {
    const every = rule.intervalEveryMinutes ?? 90;
    const start = rule.windowStart ?? "";
    const end = rule.windowEnd ?? "";
    if (!Number.isInteger(every) || every < 15 || every > 360) return "时段间隔应为 15–360 分钟";
    if (!CLOCK_RE.test(start) || !CLOCK_RE.test(end)) return "请选择有效的生效时段";
    if (start >= end) return "时段结束时间应晚于开始时间";
    if (rule.weekdays.some(day => !Number.isInteger(day) || day < 1 || day > 7)) return "请选择有效的提醒星期";
  } else {
    if (!CLOCK_RE.test(rule.time)) return "请选择有效时间";
    if (rule.repeat === "once" && (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(rule.onceAt) || !Number.isFinite(new Date(rule.onceAt).getTime()))) return "请选择一次提醒的日期和时间";
    if (rule.repeat === "weekdays" && (!rule.weekdays.length || rule.weekdays.some(day => !Number.isInteger(day) || day < 1 || day > 7))) return "请选择提醒的星期";
  }
  if (rule.kind === "weather" && (!rule.city || !Number.isFinite(rule.city.latitude) || !Number.isFinite(rule.city.longitude) || Math.abs(rule.city.latitude) > 90 || Math.abs(rule.city.longitude) > 180)) return "请手动选择天气城市";
  return null;
}
export function weatherIsFresh(weather: CareWeather, now = Date.now() / 1000): boolean {
  return now >= weather.updatedAt && now - weather.updatedAt < 1800 && now - weather.modelAt < 7200 && weather.modelAt - now <= 1800;
}
export function deliverySummary(delivery: CareDelivery): string {
  const items = delivery.items.filter(item => item.text.trim());
  if (items.length === 1) return items[0].text;
  return `${items.slice(0, 2).map(item => item.text).join("\n")}${items.length > 2 ? `\n还有 ${items.length - 2} 条到期提醒。` : ""}`;
}
