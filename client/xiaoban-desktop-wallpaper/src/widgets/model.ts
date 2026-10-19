import { validateNote, validateCountdown, validateReminder } from './content.ts';
export type WidgetKind = 'clock' | 'weather' | 'notes' | 'reminders' | 'stocks' | 'woodfish' | 'calendar' | 'countdown' | 'custom';
export type WidgetTheme = 'ivory';
export interface City { name: string; country: string; admin1: string; latitude: number; longitude: number }
export type ItemRepeat = 'none' | 'daily' | 'weekdays' | 'weekly';
export interface TodoItem { id: string; text: string; done: boolean; time?:string; date?:string; notify?:boolean; repeat?:ItemRepeat; completedOn?:string }
export interface NoteData { title: string; mode: 'note' | 'todo'; text: string; items: TodoItem[] }
export interface CountdownData { title: string; date: string; mode: 'countdown' | 'elapsed'; repeat: 'none' | 'yearly' }
export interface ReminderItem { id: string; title: string; at: string; done: boolean; end?:string; location?:string; notify?:boolean; repeat?:ItemRepeat; completedOn?:string }
export const validStockSymbol = (symbol:string) => /^(?:116\.\d{5}|1\.(?:000001|6\d{5})|0\.(?:39900[16]|[03]\d{5}))$/.test(symbol);
export interface LivingData { reminders: ReminderItem[]; count: number; symbol: string; symbols?:string[]; sound?:boolean }
export interface WidgetContent { note?: NoteData; countdown?: CountdownData; living?: LivingData }
export interface CustomWidgetPack { schemaVersion: 1; id: string; name: string; description?: string; width?: number; height?: number; theme?: string; html: string; }
export interface DesktopWidget { id: string; kind: WidgetKind; theme: WidgetTheme | string; scale: number; x: number; y: number; enabled: boolean; city: City | null; layer?: 'desktop' | 'topmost'; monitor?: string; note?: NoteData; countdown?: CountdownData; living?: LivingData; customId?: string; custom?: CustomWidgetPack; }
export interface WidgetMonitor { id: string; name: string; width: number; height: number; primary: boolean }
export interface WidgetLayout { version: 2; widgets: DesktopWidget[]; editing: boolean; errors: string[]; desktop?: {width:number;height:number}; monitors?: WidgetMonitor[] }
export interface WidgetPayload { widget: DesktopWidget; editing: boolean; screenX: number; screenY: number; displayScale?: number | null }
export const widgetKinds = [
 {kind:'clock' as const,title:'桌面时钟',label:'时钟',description:'新的一天，也要好好生活'},
 {kind:'weather' as const,title:'城市天气',label:'天气',description:'关心每一次出门'},
 {kind:'notes' as const,title:'待办事项',label:'待办',description:'把想做的小事留在眼前'},
 {kind:'reminders' as const,title:'今日日程',label:'日程',description:'从容安排今天的时间'},
 {kind:'stocks' as const,title:'股票行情',label:'股票',description:'关注自选与市场走势'},
 {kind:'woodfish' as const,title:'电子木鱼',label:'木鱼',description:'心平气和，万事顺遂'},
 {kind:'calendar' as const,title:'月历',label:'月历',description:'翻过日常，回到今天'},
 {kind:'countdown' as const,title:'纪念日',label:'纪念日',description:'每一天，都值得期待'},
];
export const baseSize = (kind:WidgetKind, custom?: CustomWidgetPack) => {
 if (kind === 'custom') return { width: custom?.width ?? 280, height: custom?.height ?? 240 };
 return ({clock:{width:596,height:306},weather:{width:326,height:306},notes:{width:352,height:306},reminders:{width:480,height:332},stocks:{width:404,height:338},woodfish:{width:390,height:332},calendar:{width:500,height:286},countdown:{width:790,height:286}} as Record<string,{width:number;height:number}>)[kind] ?? {width:280,height:240};
};
export const emptyLayout = (): WidgetLayout => ({ version: 2, widgets: [], editing: false, errors: [] });
export function validateLayout(value: unknown): WidgetLayout {
  const data = value as WidgetLayout;
  if (!data || data.version !== 2 || !Array.isArray(data.widgets) || data.widgets.length > 12) throw new Error('桌面组件记录格式不正确，原布局已保留');
  const ids = new Set<string>();
  for (const widget of data.widgets) {
    const isCustom = widget.kind === 'custom';
    const kindValid = isCustom || widgetKinds.some(k => k.kind === widget.kind);
    if (!widget || !/^[a-z0-9]{1,32}$/.test(widget.id) || ids.has(widget.id) || !kindValid
      || typeof widget.enabled !== 'boolean' || !Number.isFinite(widget.scale) || widget.scale < .5 || widget.scale > 1.5 || !Number.isFinite(widget.x) || widget.x < 0 || widget.x > 1 || !Number.isFinite(widget.y) || widget.y < 0 || widget.y > 1) throw new Error('组件布局无效，原记录已保留');
    if (isCustom && !widget.customId) throw new Error('自定义组件缺少模板标识');
    if (!isCustom && widget.theme !== 'ivory') throw new Error('内置组件材质必须为 ivory');
    if (widget.city && (widget.kind !== 'weather' || typeof widget.city.name !== 'string' || !widget.city.name.trim() || widget.city.name.length > 160 || !Number.isFinite(widget.city.latitude) || Math.abs(widget.city.latitude) > 90 || !Number.isFinite(widget.city.longitude) || Math.abs(widget.city.longitude) > 180)) throw new Error('天气城市配置无效');
    if(widget.layer!==undefined&&!['desktop','topmost'].includes(widget.layer))throw Error('组件层级无效');
    if (widget.monitor != null && (typeof widget.monitor !== 'string' || !widget.monitor || new TextEncoder().encode(widget.monitor).length > 512)) throw new Error('显示器标识无效');
    if (widget.note != null) { if (widget.kind !== 'notes') throw new Error('便签配置类型不匹配'); validateNote(widget.note); }
    if (widget.countdown != null) { if (widget.kind !== 'countdown') throw new Error('纪念日配置类型不匹配'); validateCountdown(widget.countdown); }
    if(widget.living != null) {
      const d=widget.living;
      if(!['reminders','stocks','woodfish'].includes(widget.kind) || !Array.isArray(d.reminders) || d.reminders.length>12 || !Number.isSafeInteger(d.count) || d.count<0 || d.count>9999999 || !validStockSymbol(d.symbol)) throw new Error('生活组件配置无效');
      const ids=new Set<string>();
      for(const r of d.reminders){validateReminder(r);if(ids.has(r.id))throw Error('日程标识重复');ids.add(r.id);}
      if(d.symbols!==undefined&&(!Array.isArray(d.symbols)||d.symbols.length>8||new Set(d.symbols).size!==d.symbols.length||d.symbols.some(s=>typeof s!=='string'||!validStockSymbol(s))||d.symbols.length>0&&!d.symbols.includes(d.symbol)))throw Error('自选列表应有 1–8 个不重复的有效代码，并包含当前股票');
      if(d.sound!==undefined&&typeof d.sound!=='boolean')throw Error('声音开关无效');
    }
    ids.add(widget.id);
  }
  const desktop=data.desktop&&Number.isFinite(data.desktop.width)&&data.desktop.width>0&&Number.isFinite(data.desktop.height)&&data.desktop.height>0?data.desktop:undefined;
  const monitors=Array.isArray(data.monitors)?data.monitors.filter(m=>m&&typeof m.id==='string'&&!!m.id&&typeof m.name==='string'&&Number.isFinite(m.width)&&m.width>0&&Number.isFinite(m.height)&&m.height>0&&typeof m.primary==='boolean'):undefined;
  return { version: 2, widgets: data.widgets, editing: data.editing === true, errors: Array.isArray(data.errors) ? data.errors.filter(e => typeof e === 'string') : [], desktop, monitors };
}
export function calendarDays(year: number, month: number) {
  const first = new Date(year, month, 1), offset = first.getDay();
  return Array.from({ length: 42 }, (_, index) => { const date = new Date(year, month, index - offset + 1); return { date, day: date.getDate(), current: date.getMonth() === month }; });
}

