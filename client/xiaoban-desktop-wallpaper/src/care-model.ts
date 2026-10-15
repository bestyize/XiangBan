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
  voice: boolean;
  quietEnabled: boolean;
  quietStart: string;
  quietEnd: string;
  weatherSource: CareSource;
}
export type CareKind = "custom" | "off-work" | "rest" | "weather";
export interface CareRule {
  id: string;
  revision: number;
  enabled: boolean;
  kind: CareKind;
  repeat: "once" | "daily" | "weekdays";
  time: string;
  onceAt: string;
  weekdays: number[];
  intervalMinutes: number;
  text: string;
  city: CareCity | null;
  notBefore: number;
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
}
export interface CareDelivery {
  id: string;
  items: CarePending[];
  sound: boolean;
  voice: boolean;
  displaySeconds: number;
}
export const careNames: Record<CareKind, string> = {
  custom: "自定义", "off-work": "下班", rest: "休息", weather: "天气",
};
export const dayNames = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
export function newCareRule(kind: CareKind = "custom"): CareRule {
  return { id: "", revision: 0, enabled: true, kind, repeat: "daily", time: kind === "off-work" ? "18:00" : kind === "weather" ? "08:00" : "15:00",
    onceAt: "", weekdays: [], intervalMinutes: 60, city: null, notBefore: 0,
    text: kind === "off-work" ? "到点啦，今天也辛苦了。" : kind === "rest" ? "起来走走，看看远处吧。" : kind === "weather" ? "" : "记得喝水。" };
}
export function describeCareRule(rule: CareRule): string {
  if (rule.kind === "rest") return `有效使用电脑 ${rule.intervalMinutes} 分钟`;
  if (rule.repeat === "once") return rule.onceAt.replace("T", " ");
  const repeat = rule.repeat === "daily" ? "每天" : [...new Set(rule.weekdays)].sort().map(day => dayNames[day - 1]).filter(Boolean).join("、");
  return `${repeat} ${rule.time}${rule.kind === "weather" && rule.city ? ` · ${rule.city.name}` : ""}`;
}
export function careRuleError(rule: CareRule): string | null {
  if (Array.from(rule.text).length > 240) return "提醒文案最多 240 个字";
  if (rule.kind !== "weather" && !rule.text.trim()) return "请填写提醒文案";
  if (rule.kind === "rest") {
    if (!Number.isInteger(rule.intervalMinutes) || rule.intervalMinutes < 5 || rule.intervalMinutes > 480) return "休息间隔应为 5–480 分钟";
  } else {
    if (!/^([01]\d|2[0-3]):[0-5]\d$/.test(rule.time)) return "请选择有效时间";
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
