export type WidgetKind = 'fortune' | 'calendar' | 'weather';
export type WidgetTheme = 'glass' | 'paper' | 'transparent';
export interface City { name: string; country: string; admin1: string; latitude: number; longitude: number }
export interface DesktopWidget { id: string; kind: WidgetKind; theme: WidgetTheme; scale: number; x: number; y: number; enabled: boolean; city: City | null }
export interface WidgetLayout { version: 1; widgets: DesktopWidget[]; editing: boolean; errors: string[]; desktop?: {width:number;height:number} }
export interface WidgetPayload { widget: DesktopWidget; editing: boolean; screenX: number; screenY: number }
export const widgetKinds = [
  { kind: 'fortune' as const, title: '一签好意', description: '给今天收下一份温柔祝福', label: '求签' },
  { kind: 'calendar' as const, title: '日有所序', description: '时间、日期与一眼可见的月历', label: '时钟日历' },
  { kind: 'weather' as const, title: '晴雨之间', description: '关心一座城市，也关心每次出门', label: '天气' },
];
export const widgetThemes = [{ value: 'glass' as const, label: '透光玻璃' }, { value: 'paper' as const, label: '温润纸张' }, { value: 'transparent' as const, label: '透明无底' }];
export const baseSize = (kind: WidgetKind) => kind === 'fortune' ? { width: 220, height: 330 } : kind === 'calendar' ? { width: 280, height: 330 } : { width: 280, height: 230 };
export const emptyLayout = (): WidgetLayout => ({ version: 1, widgets: [], editing: false, errors: [] });
export function validateLayout(value: unknown): WidgetLayout {
  const data = value as WidgetLayout;
  if (!data || data.version !== 1 || !Array.isArray(data.widgets) || data.widgets.length > 12) throw new Error('桌面组件记录格式不正确，原布局已保留');
  const ids = new Set<string>();
  for (const widget of data.widgets) {
    if (!widget || !/^[a-z0-9]{1,32}$/.test(widget.id) || ids.has(widget.id) || !widgetKinds.some(k => k.kind === widget.kind) || !widgetThemes.some(t => t.value === widget.theme)
      || typeof widget.enabled !== 'boolean' || !Number.isFinite(widget.scale) || widget.scale < .75 || widget.scale > 1.5 || !Number.isFinite(widget.x) || widget.x < 0 || widget.x > 1 || !Number.isFinite(widget.y) || widget.y < 0 || widget.y > 1) throw new Error('组件布局无效，原记录已保留');
    if (widget.city && (widget.kind !== 'weather' || typeof widget.city.name !== 'string' || !widget.city.name.trim() || widget.city.name.length > 160 || !Number.isFinite(widget.city.latitude) || Math.abs(widget.city.latitude) > 90 || !Number.isFinite(widget.city.longitude) || Math.abs(widget.city.longitude) > 180)) throw new Error('天气城市配置无效');
    ids.add(widget.id);
  }
  const desktop=data.desktop&&Number.isFinite(data.desktop.width)&&data.desktop.width>0&&Number.isFinite(data.desktop.height)&&data.desktop.height>0?data.desktop:undefined;
  return { version: 1, widgets: data.widgets, editing: data.editing === true, errors: Array.isArray(data.errors) ? data.errors.filter(e => typeof e === 'string') : [], desktop };
}
export function calendarDays(year: number, month: number) {
  const first = new Date(year, month, 1), offset = (first.getDay() + 6) % 7;
  return Array.from({ length: 42 }, (_, index) => { const date = new Date(year, month, index - offset + 1); return { date, day: date.getDate(), current: date.getMonth() === month }; });
}
