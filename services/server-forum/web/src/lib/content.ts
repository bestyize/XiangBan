import type { PackItem } from '@/api/content'

/** 内容类型的中文名。顺序就是资源页二级 Tab 的顺序。 */
export const PACK_KINDS = [
  { key: 'effect', label: '特效' },
  { key: 'wallpaper', label: '壁纸' },
  { key: 'widget', label: '组件' },
  { key: 'game', label: '游戏' },
  { key: 'reminder', label: '提醒' },
  { key: 'routine', label: '例行' },
  { key: 'task', label: '任务' },
  { key: 'goal', label: '目标' },
  { key: 'skill', label: '技能' },
  { key: 'expert', label: '专家' },
  { key: 'workflow', label: '工作流' },
  { key: 'scheme', label: '方案文件' },
] as const

export function kindLabel(kind: string): string {
  return PACK_KINDS.find(k => k.key === kind)?.label ?? kind
}

/** 视觉类内容用封面图卡片，文字为主的能力类用单列。 */
export const VISUAL_KINDS = ['effect', 'wallpaper', 'widget', 'game']

export function trustLabel(level: number): string {
  return level === 0 ? '官方' : level === 1 ? '已审核' : '未审核'
}

/** 不同类型的封面占位渐变，没有封面图时使用，保持黑白灰为主的克制配色。 */
const GRADIENTS = [
  ['#2b2b2e', '#5a5a60'],
  ['#3a3a3f', '#8d8d93'],
  ['#1f2933', '#52606d'],
  ['#44403c', '#a8a29e'],
  ['#262626', '#737373'],
]

export function coverStyle(pack: Pick<PackItem, 'id' | 'cover_url'>): Record<string, string> {
  if (pack.cover_url) return { backgroundImage: `url("${pack.cover_url}")` }
  const [a, b] = GRADIENTS[pack.id % GRADIENTS.length]!
  return { backgroundImage: `linear-gradient(135deg, ${a}, ${b})` }
}

/** 唤起小伴桌面端的一键应用链接（协议见 PRD §6.9）。 */
export function applyLink(pack: Pick<PackItem, 'kind' | 'id'>, version: number, sha256?: string): string {
  const params = new URLSearchParams({ version: String(version) })
  if (sha256) params.set('sha256', sha256)
  return `xiaoban://community/apply/${pack.kind}/${pack.id}?${params}`
}

export function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

/** 当前周的标识，形如 2026-W44（ISO 周）。 */
export function isoWeek(date: Date): string {
  const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()))
  const day = d.getUTCDay() || 7
  d.setUTCDate(d.getUTCDate() + 4 - day)
  const yearStart = Date.UTC(d.getUTCFullYear(), 0, 1)
  const week = Math.ceil(((d.getTime() - yearStart) / 86_400_000 + 1) / 7)
  return `${d.getUTCFullYear()}-W${String(week).padStart(2, '0')}`
}
