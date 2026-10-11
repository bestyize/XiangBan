import type { DealStatus, DealType, Region } from '@/api/types'

export const DEAL_TYPE_LABEL: Record<DealType, string> = {
  free_credit: '免费额度',
  signup_bonus: '新用户赠金',
  trial: '会员试用',
  free_limited: '限时免费',
  invite: '邀请码',
  student: '学生优惠',
  discount: '促销折扣',
  public_site: '公益站',
}

export const DEAL_STATUS_LABEL: Record<DealStatus, string> = {
  valid: '有效',
  suspect: '疑似失效',
  expired: '已失效',
}

export const REGION_LABEL: Record<Region, string> = {
  any: '不限地区',
  cn: '国内可用',
  intl: '仅海外',
}

/** 领取条件位标志，与后端 models::CONDITIONS 一致。 */
export const CONDITIONS = [
  { key: 'card', bit: 1, label: '需要信用卡', without: '不要信用卡' },
  { key: 'phone_cn', bit: 2, label: '需要国内手机号', without: '不要国内手机号' },
  { key: 'phone_intl', bit: 4, label: '需要海外手机号', without: '不要海外手机号' },
  { key: 'edu', bit: 8, label: '需要 edu 邮箱', without: '不要 edu 邮箱' },
  { key: 'work_email', bit: 16, label: '需要企业邮箱', without: '不要企业邮箱' },
  { key: 'realname', bit: 32, label: '需要实名', without: '不要实名' },
] as const

export function conditionLabels(bits: number) {
  return CONDITIONS.filter(c => bits & c.bit).map(c => c.label)
}

export const TRUST_LABEL = ['新人', '成员', '老用户', '守护者']

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
