export const homeTabs = [
  { key: 'ideas', label: '创意' },
  { key: 'recommend', label: '资讯' },
  { key: 'open-source', label: '精选' },
  { key: 'rankings', label: '排行榜' },
  { key: 'nav', label: '导航' },
] as const

export const newsHomePath = '/?tab=recommend'

export function homeTabOf(value: unknown) {
  if (value === 'weekly' || value === 'projects') return value
  return homeTabs.find(tab => tab.key === value)?.key ?? homeTabs[0].key
}
