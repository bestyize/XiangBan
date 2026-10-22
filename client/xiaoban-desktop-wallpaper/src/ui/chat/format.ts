import type { ApprovalRequest, Risk } from './types.ts';

const TOOL_LABELS: Record<string, string> = {
  shell: '运行命令', read_file: '读取文件', write_file: '写入文件', edit_file: '修改文件', list_dir: '查看文件夹', grep: '搜索文件',
  web_fetch: '打开网页', web_search: '搜索网页', todo: '任务清单', ask_user: '向你提问', time_now: '查看时间',
  memory_read: '读取笔记', memory_write: '保存笔记', load_skill: '读取技能', skill_manage: '管理技能',
  desktop_status: '查看桌面状态', desktop_windows: '查看小伴窗口', desktop_window_set: '调整小伴窗口', wallpaper_list: '查看壁纸', wallpaper_apply: '更换壁纸', wallpaper_create: '创作壁纸', wallpaper_delete: '删除壁纸',
  effect_list: '查看特效', effect_set: '切换特效', effect_read_format: '读取特效格式', effect_create: '创作特效', effect_delete: '删除特效',
  companion_set: '设置小伴', care_list: '查看提醒', care_set: '设置提醒', care_delete: '删除提醒',
  widget_list: '查看桌面组件', widget_add: '添加桌面组件', widget_update: '调整桌面组件', widget_remove: '移除桌面组件', widget_create: '创作桌面组件', widget_delete_custom: '删除自创组件',
  pet_read_format: '读取宠物格式', pet_catalog: '查看宠物', pet_theme_save: '保存宠物主题', pet_theme_delete: '删除宠物主题',
  pet_add: '添加桌面宠物', pet_update: '调整桌面宠物', pet_remove: '移除桌面宠物',
  research: '调研子任务', process_start: '启动后台命令', process_read: '查看后台输出', process_write: '向后台命令输入', process_stop: '停止后台命令',
  game_list: '查看游戏列表', game_open: '打开游戏', game_close: '退出游戏', game_create: '创作游戏', game_delete: '删除游戏',
  community_share: '分享到论坛', community_open: '打开官网或论坛',
  request_accessibility_permission: '打开系统权限设置',
  computer_use_diagnose: '检查电脑控制权限',
  computer_use_repair_permissions: '修复电脑控制旧授权',
  list_apps: '查看应用列表', get_app_state: '获取窗口状态', click: '模拟点击',
  type_text: '模拟键盘打字', press_key: '模拟按键', scroll: '模拟滚动',
  drag: '模拟拖拽', set_value: '设置控件值', perform_secondary_action: '触发辅助动作',
};

export const toolLabel = (name: string) => TOOL_LABELS[name] ?? (name || '工具');

/** The range of the context limit slider. The host (`ai_config.rs`) holds the same range. */
export const CONTEXT_MIN = 16_000;
export const CONTEXT_MAX = 1_000_000;
export const CONTEXT_STEP = 8_000;

/** 850, 12.3K, 48K, 1M: short enough for the ring's popover. */
export function formatTokens(count: number): string {
  if (!Number.isFinite(count) || count <= 0) return '0';
  if (count < 1000) return String(Math.round(count));
  // One decimal below 100, none above, and never a trailing ".0".
  const short = (value: number) => (value >= 100 ? Math.round(value) : Math.round(value * 10) / 10);
  if (count < 1_000_000) {
    const thousands = short(count / 1000);
    if (thousands < 1000) return `${thousands}K`;
  }
  return `${short(count / 1_000_000)}M`;
}

/** How full the context is, 0 to 100. A broken or missing limit counts as empty. */
export function contextPercent(used: number, limit: number): number {
  if (!(limit > 0) || !(used > 0)) return 0;
  return Math.min(100, Math.round((used / limit) * 100));
}

/** `warn` from 70%, `full` from 90%: the point where compacting starts to pay off, and where old messages are about to be left out. */
export function contextLevel(used: number, limit: number): 'ok' | 'warn' | 'full' {
  const ratio = limit > 0 ? used / limit : 0;
  return ratio >= 0.9 ? 'full' : ratio >= 0.7 ? 'warn' : 'ok';
}

const RISK_VERBS: Record<Risk, string> = { read: '读取', write: '修改', network: '联网', exec: '运行命令' };
export const riskVerb = (risk: Risk) => RISK_VERBS[risk] ?? '操作';

const cut = (text: string, max: number) => (text.length > max ? `${text.slice(0, max)}…` : text);
const str = (value: unknown) => (typeof value === 'string' ? value : '');

/** What the confirmation card shows: the thing itself (a command, a path and its new text), not a blob of JSON. */
export function approvalPreview(request: Pick<ApprovalRequest, 'tool' | 'arguments'>): string {
  const args = request.arguments && typeof request.arguments === 'object' ? request.arguments as Record<string, unknown> : {};
  switch (request.tool) {
    case 'shell':
      return str(args.command) || '（没有命令内容）';
    case 'write_file':
      return `${str(args.path)}\n\n${cut(str(args.content), 1500)}`;
    case 'game_open':
      return `进入游戏空间${args.id ? `并启动游戏：${str(args.id)}` : '页面'}`;
    case 'game_close':
      return '退出游戏空间，返回桌面';
    case 'click':
      return `点击：${str(args.app || '')} ${str(args.target || args.element || '')}`;
    case 'type_text':
      return `在 ${str(args.app || '')} 中输入：${str(args.text)}`;
    case 'press_key':
      return `按键：${str(args.app || '')} ${str(args.key)}`;
    case 'widget_add':
      return `在桌面添加组件：${str(args.kind)}${args.theme ? `（${str(args.theme)}风格）` : ''}`;
    case 'widget_remove':
      return `从桌面移除组件：${str(args.id)}`;
    case 'widget_update':
      return `调整桌面组件 ${str(args.id)}`;
    case 'edit_file':
      return `${str(args.path)}\n\n- ${cut(str(args.old_string), 600).replace(/\n/g, '\n  ')}\n+ ${cut(str(args.new_string), 600).replace(/\n/g, '\n  ')}`;
    default:
      try { return cut(JSON.stringify(request.arguments ?? {}, null, 2), 2000); } catch { return String(request.arguments); }
  }
}

/** Enter that confirms an input-method candidate is not "send". WebKit reports it as keyCode 229 after the composition ended. */
export function isImeEnter(event: Pick<KeyboardEvent, 'isComposing' | 'keyCode'>, composing: boolean): boolean {
  return composing || event.isComposing || event.keyCode === 229;
}

/** Close enough to the end that new text should keep the view following it. */
export function isNearBottom(el: Pick<HTMLElement, 'scrollTop' | 'clientHeight' | 'scrollHeight'>, slack = 80): boolean {
  return el.scrollHeight - el.scrollTop - el.clientHeight <= slack;
}

export function formatAgo(seconds: number, nowSeconds: number): string {
  const diff = Math.max(0, nowSeconds - seconds);
  if (diff < 60) return '刚刚';
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 172800) return '昨天';
  if (diff < 7 * 86400) return `${Math.floor(diff / 86400)} 天前`;
  const date = new Date(seconds * 1000);
  return `${date.getMonth() + 1} 月 ${date.getDate()} 日`;
}

/** Long tool input and output stay readable in the page. */
export const clip = (text: string, max = 6000) => (text.length > max ? `${text.slice(0, max)}\n… 还有 ${text.length - max} 个字符没有显示` : text);

export function prettyArguments(argumentsJson: string): string {
  try { return clip(JSON.stringify(JSON.parse(argumentsJson), null, 2), 4000); } catch { return clip(argumentsJson, 4000); }
}

