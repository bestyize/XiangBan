import type { AdminUserRow } from './types.ts'
import { formatTime } from './ui.tsx'

export function csvCell(value: string | number | null): string {
  let text = value === null ? '' : String(value)
  if (/^[\t\r\n ]*[=+\-@]|^[\t\r\n]/.test(text)) text = `'${text}`
  return `"${text.replace(/"/g, '""')}"`
}
export function userCsv(items: AdminUserRow[]): string {
  const header = ['用户编号', '邮箱', '昵称', '小伴名', '账号类型', '账号创建时间（北京时间）', '邮箱验证时间（北京时间）', '最近活跃（北京时间）', '关联设备数', '有效打卡数', '引导状态']
  const rows = items.map((u) => [u.id, u.email, u.nickname, u.companionName, u.email ? '邮箱账号' : '游客账号',
    formatTime(u.createdAt), formatTime(u.emailVerifiedAt), formatTime(u.lastActiveAt), u.deviceCount, u.checkinCount, u.onboardedAt ? '已完成' : '未完成'])
  return '\uFEFF' + [header, ...rows].map((row) => row.map(csvCell).join(',')).join('\r\n')
}
