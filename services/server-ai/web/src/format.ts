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
  'product.save': '保存产品',
  'tier.save': '保存档位',
  'tier.delete': '删除档位',
  'media.save': '保存媒体能力',
  'media.delete': '关闭媒体能力',
  'binding.grant': '补发额度',
  'bucket.void': '扣回额度',
  'anomaly.resolve': '处理异常',
  'batch.create': '生成兑换码批次',
  'batch.void': '作废批次剩余的码',
  'code.void': '作废兑换码',
  'settings.newapi_token': '更新网关令牌',
};

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
  return uid.slice(0, 8);
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
