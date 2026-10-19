import { validateNote, validateCountdown, validateFlower } from './content.ts';
export type WidgetKind = 'fortune' | 'calendar' | 'weather' | 'notes' | 'countdown' | 'flower' | 'custom' | 'clock' | 'reminders' | 'stocks' | 'woodfish';
export type WidgetTheme = 'glass' | 'paper' | 'transparent';
export interface City { name: string; country: string; admin1: string; latitude: number; longitude: number }
export interface TodoItem { id: string; text: string; done: boolean }
export interface NoteData { title: string; mode: 'note' | 'todo'; text: string; items: TodoItem[] }
export interface CountdownData { title: string; date: string; mode: 'countdown' | 'elapsed'; repeat: 'none' | 'yearly' }
/** A white flower grows one stage every few waterings; `since` is the day it was planted. */
export interface FlowerData { water: number; since: string }
export interface ReminderItem { id: string; title: string; at: string; done: boolean }
export const validStockSymbol = (symbol:string) => /^(?:1\.(?:000001|6\d{5})|0\.(?:39900[16]|[03]\d{5}))$/.test(symbol);
export interface LivingData { reminders: ReminderItem[]; count: number; symbol: string }
export interface WidgetContent { note?: NoteData; countdown?: CountdownData; flower?: FlowerData; living?: LivingData }
export interface CustomWidgetData { schemaVersion?: number; id: string; name: string; description?: string; width?: number; height?: number; theme?: string; html: string }
export interface DesktopWidget { id: string; kind: WidgetKind; theme: WidgetTheme; scale: number; x: number; y: number; enabled: boolean; city: City | null; monitor?: string; note?: NoteData; countdown?: CountdownData; flower?: FlowerData; living?: LivingData; customId?: string; custom?: CustomWidgetData }
export interface WidgetMonitor { id: string; name: string; width: number; height: number; primary: boolean }
export interface WidgetLayout { version: 1; widgets: DesktopWidget[]; editing: boolean; errors: string[]; desktop?: {width:number;height:number}; monitors?: WidgetMonitor[] }
export interface WidgetPayload { widget: DesktopWidget; editing: boolean; screenX: number; screenY: number; displayScale?: number | null }
export const widgetKinds = [
  { kind: 'clock' as const, title: '此刻时光', description: '留一片清静，让时间成为桌面的主角', label: '桌面时钟' },
  { kind: 'fortune' as const, title: '一签好意', description: '给今天收下一份温柔祝福', label: '求签' },
  { kind: 'calendar' as const, title: '日有所序', description: '时间、日期与一眼可见的月历', label: '时钟日历' },
  { kind: 'weather' as const, title: '晴雨之间', description: '关心一座城市，也关心每次出门', label: '天气' },
  { kind: 'notes' as const, title: '心事有处', description: '随手记下灵感，逐件完成今天的小事', label: '便签待办' },
  { kind: 'countdown' as const, title: '好日将至', description: '记住重要的日子，也期待下一次相逢', label: '倒数日／纪念日' },
  { kind: 'reminders' as const, title: '今日安排', description: '沿着时间线，妥帖安放今天的事', label: '提醒' },
  { kind: 'stocks' as const, title: '市场脉搏', description: '关心大盘与自选，行情时间清楚可见', label: '股票' },
  { kind: 'woodfish' as const, title: '松一口气', description: '轻敲一声，给忙碌一点留白', label: '电子木鱼' },
  { kind: 'flower' as const, title: '小白花初开', description: '每天浇一点水，陪它从嫩芽开到满庭', label: '成长小白花' },
];
export const widgetThemes = [{ value: 'glass' as const, label: '透光玻璃' }, { value: 'paper' as const, label: '温润纸张' }, { value: 'transparent' as const, label: '透明无底' }];
export const baseSize = (kind: WidgetKind, custom?: CustomWidgetData) => {
  if (kind === 'custom' && custom && Number.isFinite(custom.width) && Number.isFinite(custom.height) && custom.width! > 0 && custom.height! > 0) {
    return { width: custom.width!, height: custom.height! };
  }
  if (kind === 'custom') return { width: 280, height: 240 };
  if(kind === 'clock'||kind === 'weather') return { width: 360, height: 300 };
  if(kind === 'reminders'||kind === 'woodfish') return { width: 360, height: 320 };
  if(kind === 'stocks') return { width: 390, height: 330 };
  return kind === 'fortune' ? { width: 220, height: 330 } : kind === 'calendar' ? { width: 300, height: 400 } : kind === 'notes' ? { width: 300, height: 360 } : kind === 'flower' ? { width: 260, height: 320 } : { width: 280, height: 230 };
};
export const emptyLayout = (): WidgetLayout => ({ version: 1, widgets: [], editing: false, errors: [] });
export function validateLayout(value: unknown): WidgetLayout {
  const data = value as WidgetLayout;
  if (!data || data.version !== 1 || !Array.isArray(data.widgets) || data.widgets.length > 12) throw new Error('桌面组件记录格式不正确，原布局已保留');
  const ids = new Set<string>();
  for (const widget of data.widgets) {
    const isCustom = widget && widget.kind === 'custom';
    const kindValid = isCustom || widgetKinds.some(k => k.kind === widget.kind);
    if (!widget || !/^[a-z0-9]{1,32}$/.test(widget.id) || ids.has(widget.id) || !kindValid || !widgetThemes.some(t => t.value === widget.theme)
      || typeof widget.enabled !== 'boolean' || !Number.isFinite(widget.scale) || widget.scale < .75 || widget.scale > 1.5 || !Number.isFinite(widget.x) || widget.x < 0 || widget.x > 1 || !Number.isFinite(widget.y) || widget.y < 0 || widget.y > 1) throw new Error('组件布局无效，原记录已保留');
    if (isCustom && (!widget.customId || typeof widget.customId !== 'string' || !/^[a-z0-9_-]{1,48}$/.test(widget.customId))) throw new Error('自定义组件缺少有效的 customId');
    if (widget.city && (widget.kind !== 'weather' || typeof widget.city.name !== 'string' || !widget.city.name.trim() || widget.city.name.length > 160 || !Number.isFinite(widget.city.latitude) || Math.abs(widget.city.latitude) > 90 || !Number.isFinite(widget.city.longitude) || Math.abs(widget.city.longitude) > 180)) throw new Error('天气城市配置无效');
    if (widget.monitor != null && (typeof widget.monitor !== 'string' || !widget.monitor || new TextEncoder().encode(widget.monitor).length > 512)) throw new Error('显示器标识无效');
    if (widget.note != null) { if (widget.kind !== 'notes') throw new Error('便签配置类型不匹配'); validateNote(widget.note); }
    if (widget.countdown != null) { if (widget.kind !== 'countdown') throw new Error('纪念日配置类型不匹配'); validateCountdown(widget.countdown); }
    if(widget.living != null) {
      const d=widget.living;
      if(!['reminders','stocks','woodfish'].includes(widget.kind) || !Array.isArray(d.reminders) || d.reminders.length>12 || !Number.isSafeInteger(d.count) || d.count<0 || d.count>9999999 || !validStockSymbol(d.symbol)) throw new Error('生活组件配置无效');
      const ids=new Set<string>();
      for(const r of d.reminders){if(!r || !/^[a-z0-9]{1,32}$/.test(r.id) || ids.has(r.id) || typeof r.title!=='string' || !r.title.trim() || r.title.length>60 || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(r.at) || Number.isNaN(new Date(r.at).getTime()) || typeof r.done!=='boolean') throw new Error('提醒记录无效');ids.add(r.id);}
    }
    if (widget.flower != null) { if (widget.kind !== 'flower') throw new Error('小白花配置类型不匹配'); validateFlower(widget.flower); }
    ids.add(widget.id);
  }
  const desktop=data.desktop&&Number.isFinite(data.desktop.width)&&data.desktop.width>0&&Number.isFinite(data.desktop.height)&&data.desktop.height>0?data.desktop:undefined;
  const monitors=Array.isArray(data.monitors)?data.monitors.filter(m=>m&&typeof m.id==='string'&&!!m.id&&typeof m.name==='string'&&Number.isFinite(m.width)&&m.width>0&&Number.isFinite(m.height)&&m.height>0&&typeof m.primary==='boolean'):undefined;
  return { version: 1, widgets: data.widgets, editing: data.editing === true, errors: Array.isArray(data.errors) ? data.errors.filter(e => typeof e === 'string') : [], desktop, monitors };
}
export function calendarDays(year: number, month: number) {
  const first = new Date(year, month, 1), offset = (first.getDay() + 6) % 7;
  return Array.from({ length: 42 }, (_, index) => { const date = new Date(year, month, index - offset + 1); return { date, day: date.getDate(), current: date.getMonth() === month }; });
}

