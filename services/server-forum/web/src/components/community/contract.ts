/** Paths shared by the public discover views. Keys are always one URL segment. */
export const discoverPaths = {
  home: '/discover/home/v2', daily: '/discover/daily', repos: '/discover/repos/v2',
  issues: '/discover/repos/issues/v2', tracker: '/discover/tracker/v2',
  nav: '/discover/nav', docs: '/discover/docs', tools: '/tools',
  models: '/discover/models/v2', usage: '/discover/model-usage',
  article: (key: string) => `/discover/daily/${encodeURIComponent(key)}`,
  tool: (key: string) => `/tools/${encodeURIComponent(key)}`,
  comments: (key: string) => `/discover/daily/${encodeURIComponent(key)}/comments`,
}

/** Only web links or same-origin paths can become links/images in editorial data. */
export function webLink(raw: string | null | undefined): string | undefined {
  if (!raw || /[\u0000-\u0020\u007f\\]/.test(raw)) return undefined
  if (raw.startsWith('/') && !raw.startsWith('//')) return raw
  try {
    const u = new URL(raw)
    return (u.protocol === 'https:' || u.protocol === 'http:') && !u.username && !u.password ? u.href : undefined
  } catch { return undefined }
}

export function officialSource(raw: string | null | undefined): string | undefined {
  const link = webLink(raw)
  if (!link || !link.startsWith('https://')) return undefined
  const u = new URL(link)
  return !u.port || u.port === '443' ? link : undefined
}

export function repoMetric(issue: { source?: string; metric?: string; complete?: boolean }) {
  return issue.source === 'GitHub' && issue.metric === 'star total baseline'
    ? 'Star 总量基线' : issue.complete ? '周期 Star 增量' : '观察期 Star 增量'
}
export function repoGrowth(value: number | null | undefined): string {
  return value == null ? '尚无可比增量' : `${value >= 0 ? '+' : ''}${value.toLocaleString('zh-CN')}`
}
export function displayDate(raw?: string | null) {
  if (!raw) return ''
  const date = new Date(raw)
  return Number.isNaN(date.getTime()) ? raw : date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}
export function createRequestGate() {
  let sequence = 0
  return { next: () => ++sequence, current: (id: number) => id === sequence, cancel: () => { ++sequence } }
}
