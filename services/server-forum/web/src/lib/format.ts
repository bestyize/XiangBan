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

export function compactCount(n: number) {
  if (n < 1000) return String(n)
  if (n < 10_000) return `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k`
  return `${(n / 10_000).toFixed(1).replace(/\.0$/, '')}万`
}

export function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

/** 标签 key 保持规范化；展示时还原常用缩写的大小写。 */
export function tagLabel(tag: string) {
  return tag === '4k' ? '4K' : tag
}

export function pinScopeLabel(scope: string) {
  return ({ site: '全站置顶', parent: '一级板块置顶', category: '二级板块置顶' } as Record<string, string>)[scope] ?? '置顶'
}
