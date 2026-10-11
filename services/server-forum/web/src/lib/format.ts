/** 领取条件位标志，与后端 models::CONDITIONS 一致。 */
export const CONDITIONS = [
  { key: 'card', bit: 1, label: '需要信用卡' },
  { key: 'phone_cn', bit: 2, label: '需要国内手机号' },
  { key: 'phone_intl', bit: 4, label: '需要海外手机号' },
  { key: 'edu', bit: 8, label: '需要 edu 邮箱' },
  { key: 'work_email', bit: 16, label: '需要企业邮箱' },
  { key: 'realname', bit: 32, label: '需要实名' },
] as const

export function conditionLabels(bits: number) {
  return CONDITIONS.filter(c => bits & c.bit).map(c => c.label)
}

export const TRUST_LABEL = ['新人', '成员', '老用户', '守护者']

/**
 * 发帖到某个分类需要的信任等级，和后端 `required_post_level` 一致：
 * 「仅管理员可发」的分类要守护者（后台账号发不了帖），其余看分类的最低发帖等级。
 */
export function requiredPostLevel(c: { min_post_level: number; admin_only: boolean }) {
  return c.admin_only ? 3 : Math.min(Math.max(c.min_post_level, 0), 3)
}

/** 阅读权限的选项，下标就是等级。 */
export const READ_LEVELS = [
  { level: 0, label: '所有人', short: '' },
  { level: 1, label: '成员以上', short: '仅成员以上' },
  { level: 2, label: '老用户以上', short: '仅老用户以上' },
  { level: 3, label: '守护者', short: '仅守护者' },
] as const

/** 列表和详情里标出的阅读权限；公开的帖子不标。 */
export function readLevelBadge(level: number) {
  return READ_LEVELS[Math.min(Math.max(level, 0), 3)].short
}

const MIN = 60_000
const HOUR = 60 * MIN
const DAY = 24 * HOUR

export function timeAgo(iso: string, now = Date.now()) {
  const diff = now - new Date(iso).getTime()
  if (diff < MIN) return '刚刚'
  if (diff < HOUR) return `${Math.floor(diff / MIN)} 分钟前`
  if (diff < DAY) return `${Math.floor(diff / HOUR)} 小时前`
  if (diff < 30 * DAY) return `${Math.floor(diff / DAY)} 天前`
  return formatDate(iso)
}

export function formatDate(iso: string) {
  const d = new Date(iso)
  const sameYear = d.getFullYear() === new Date().getFullYear()
  const md = `${d.getMonth() + 1} 月 ${d.getDate()} 日`
  return sameYear ? md : `${d.getFullYear()} 年 ${md}`
}

export function formatDateTime(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${formatDate(iso)} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

/** 有效期文案：长期有效 / 未知 / 还剩 N 天 / 已过期。 */
export function expiryText(expiresAt: string | null, longTerm: boolean, now = Date.now()) {
  if (longTerm) return '长期有效'
  if (!expiresAt) return '有效期未知'
  const diff = new Date(expiresAt).getTime() - now
  if (diff <= 0) return '已过期'
  if (diff < DAY) return `${Math.max(1, Math.floor(diff / HOUR))} 小时后截止`
  if (diff < 15 * DAY) return `还剩 ${Math.ceil(diff / DAY)} 天`
  return `${formatDate(expiresAt)}截止`
}

export function expiringSoon(expiresAt: string | null, longTerm: boolean, now = Date.now()) {
  if (longTerm || !expiresAt) return false
  const diff = new Date(expiresAt).getTime() - now
  return diff > 0 && diff < 3 * DAY
}

export function compactCount(n: number) {
  if (n < 1000) return String(n)
  if (n < 10_000) return `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k`
  return `${(n / 10_000).toFixed(1).replace(/\.0$/, '')}万`
}
