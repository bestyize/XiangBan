import { reactive } from 'vue';

/** New API 里 1 美元对应的额度点数，登录后由概览接口填入。 */
export const units = reactive({ perDollar: 500_000 });

export function toDollars(points: number): number {
  return points / units.perDollar;
}

export function toPoints(dollars: number): number {
  return Math.round(dollars * units.perDollar);
}

export interface MoneyParts {
  sign: string;
  int: string;
  dec: string;
}

/** 额度一律用美元展示：至少两位小数，不到 1 美元时最多保留三位，避免一次对话的零头显示成 $0.00。 */
export function moneyParts(points: number): MoneyParts {
  const dollars = toDollars(points);
  const abs = Math.abs(dollars);
  const min = abs >= 100 ? 0 : 2;
  const max = abs >= 100 ? 0 : abs >= 1 ? 2 : 3;
  const [int, dec] = abs.toLocaleString('en-US', { minimumFractionDigits: min, maximumFractionDigits: max }).split('.');
  return { sign: dollars < 0 ? '\u2212' : '', int, dec: dec ? `.${dec}` : '' };
}

export function money(points: number): string {
  const { sign, int, dec } = moneyParts(points);
  return `${sign}$${int}${dec}`;
}

/** 用户侧统一使用小伴点（1 美元 = 100 点，向下取整，非零不足 1 点显示 <1 点）。 */
export function xiaobanPoints(points: number): string {
  const raw = (Math.abs(points) / units.perDollar) * 100;
  if (raw === 0) return '0 点';
  if (raw < 1) return '<1 点';
  return `${Math.floor(raw).toLocaleString('en-US')} 点`;
}

export function formatTime(value: string | null): string {
  if (!value) return '—';
  return new Date(value).toLocaleString('zh-CN', { hour12: false, month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
}

export function formatDate(value: string | null): string {
  if (!value) return '长期有效';
  return new Date(value).toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

/** 距离到期还剩多久，用人话说；已过期返回「已过期」。 */
export function timeLeft(value: string | null): string {
  if (!value) return '长期有效';
  const ms = new Date(value).getTime() - Date.now();
  if (ms <= 0) return '已过期';
  const hours = ms / 3_600_000;
  if (hours < 1) return '不到 1 小时';
  if (hours < 48) return `还剩 ${Math.floor(hours)} 小时`;
  return `还剩 ${Math.floor(hours / 24)} 天`;
}

export function isSoon(value: string | null): boolean {
  if (!value) return false;
  const ms = new Date(value).getTime() - Date.now();
  return ms > 0 && ms < 3 * 86_400_000;
}

export const kindLabels: Record<string, string> = {
  trial: '试用',
  monthly: '月卡',
  topup: '加油包',
};

export const ledgerLabels: Record<string, string> = {
  grant: '入账',
  consume: '消耗',
  consume_overdraft: '透支',
  expire: '过期作废',
  void: '管理员扣回',
};

export const closedLabels: Record<string, string> = {
  spent: '已用完',
  expired: '已过期',
  admin_void: '已扣回',
};

export const auditLabels: Record<string, string> = {
  'admin.password_change': '修改管理员密码',
  'product.save': '保存产品配置',
  'tier.save': '更新档位配置',
  'tier.delete': '删除档位',
  'media.save': '更新媒体能力',
  'media.delete': '关闭媒体能力',
  'binding.grant': '手动补发额度',
  'bucket.void': '扣回额度桶',
  'anomaly.resolve': '确认对账异常',
  'batch.create': '生成兑换码批次',
  'batch.void': '作废批次余码',
  'code.void': '作废单个兑换码',
  'settings.newapi_token': '更新网关令牌',
};

export type AuditCategory = 'product' | 'quota' | 'code' | 'system';

export function auditCategory(action: string): AuditCategory {
  if (action.startsWith('product.') || action.startsWith('tier.') || action.startsWith('media.')) return 'product';
  if (action.startsWith('binding.') || action.startsWith('bucket.')) return 'quota';
  if (action.startsWith('batch.') || action.startsWith('code.')) return 'code';
  return 'system';
}

/** 把后台审计日志里的 key=value 结构化解析为易读标签胶囊。 */
export function parseAuditDetail(raw: string | null): string[] {
  if (!raw) return [];
  if (!raw.includes('=')) return [raw];
  const labels: Record<string, string> = {
    model: '模型',
    paid: '付费要求',
    default: '默认档',
    enabled: '状态',
    kind: '类型',
    quota: '额度',
    remaining: '扣回',
    trial: '试用额度',
    days: '有效期',
    note: '备注',
    product: '产品',
    count: '数量',
    binding: '绑定',
    voided: '作废数量',
    clients: '客户端',
  };
  // 匹配 key=[...] 或 key=value（兼容 note= 后含空格的尾段）
  const tokens: string[] = [];
  let rest = raw.trim();
  const noteIdx = rest.search(/\bnote=/);
  let notePart = '';
  if (noteIdx >= 0) {
    notePart = rest.slice(noteIdx + 5).trim();
    rest = rest.slice(0, noteIdx).trim();
  }
  const matches = rest.match(/[\w.-]+=(?:\[[^\]]*\]|\S+)/g) ?? [];
  for (const part of matches) {
    const eq = part.indexOf('=');
    const k = part.slice(0, eq);
    const v = part.slice(eq + 1);
    if (!v) continue;
    if ((k === 'quota' || k === 'remaining' || k === 'trial') && /^-?\d+$/.test(v)) {
      tokens.push(`${labels[k] ?? k}: ${money(Number(v))}`);
    } else if (k === 'days') {
      tokens.push(`有效期: ${v} 天`);
    } else if (k === 'paid') {
      tokens.push(v === 'true' ? '需付费额度' : '试用可用');
    } else if (k === 'default') {
      tokens.push(v === 'true' ? '默认档' : '非默认');
    } else if (k === 'enabled') {
      tokens.push(v === 'true' ? '已启用' : '已停用');
    } else if (k === 'kind') {
      tokens.push(`类型: ${kindLabels[v] ?? v}`);
    } else if (k === 'binding') {
      tokens.push(`绑定 #${v}`);
    } else {
      tokens.push(`${labels[k] ?? k}: ${v}`);
    }
  }
  if (notePart) tokens.push(`备注: ${notePart}`);
  return tokens.length ? tokens : [raw];
}

export const codeStatusLabels: Record<string, string> = {
  unused: '未兑换',
  redeemed: '已兑换',
  void: '已作废',
};

export function relativeTime(value: string | null): string {
  if (!value) return '从未';
  const seconds = Math.floor((Date.now() - new Date(value).getTime()) / 1000);
  if (seconds < 45) return '刚刚';
  if (seconds < 3600) return `${Math.max(1, Math.round(seconds / 60))} 分钟前`;
  if (seconds < 86400) return `${Math.round(seconds / 3600)} 小时前`;
  if (seconds < 86400 * 30) return `${Math.round(seconds / 86400)} 天前`;
  return formatDate(value);
}

export function shortUid(uid: string): string {
  if (uid.length <= 14) return uid;
  return `${uid.slice(0, 8)}…${uid.slice(-4)}`;
}

function localKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

/** 按本地日期分组用的键，如 2026-10-17。 */
export function dayKey(value: string): string {
  return localKey(new Date(value));
}

export function dayTitle(key: string): string {
  const today = new Date();
  const yesterday = new Date(today.getTime() - 86_400_000);
  if (key === localKey(today)) return '今天';
  if (key === localKey(yesterday)) return '昨天';
  const [y, m, d] = key.split('-').map(Number);
  const weekday = ['日', '一', '二', '三', '四', '五', '六'][new Date(y, m - 1, d).getDay()];
  return `${m} 月 ${d} 日 周${weekday}`;
}

export function clock(value: string): string {
  return new Date(value).toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit' });
}

export function formatTokens(n?: number | null): string {
  if (!n) return '';
  if (n >= 1_000_000) return `${+(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${Math.round(n / 1000)}K`;
  return String(n);
}
