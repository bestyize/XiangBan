export interface PluginEntry {
  slug: string
  namespace: string
  name: string
  description: string
  version: string | null
  iconUrl: string | null
  category: string | null
  source: string | null
  downloads: number
  stars: number
  labels: Record<string, string> | null
}
export interface PluginPage { results: PluginEntry[]; page: number; total: number; hasMore: boolean }
export interface PluginDetail { entry: PluginEntry; changelog: string | null; sourceUrl: string; publisher: string }
export interface Filters { keyword: string; category: string; source: string; sortBy: string; page: number }
export const sorts = [
  { value: 'score', label: '综合推荐' }, { value: 'downloads', label: '最多下载' },
  { value: 'stars', label: '最多收藏' }, { value: 'updated_at', label: '最近更新' },
]
type Query = Record<string, unknown>
const string = (value: unknown) => typeof value === 'string' ? value : ''
export function filtersFromQuery(query: Query): Filters {
  const page = Number(string(query.page) || '1')
  const category = string(query.category)
  const sort = string(query.sort)
  return {
    keyword: string(query.q).slice(0, 80),
    category: /^[a-zA-Z0-9_-]{1,80}$/.test(category) ? category : '',
    source: ['official', 'community', 'enterprise'].includes(string(query.source)) ? string(query.source) : '',
    sortBy: sorts.some(s => s.value === sort) ? sort : 'score',
    page: Number.isInteger(page) && page >= 1 && page <= 10000 ? page : 1,
  }
}
export function queryFromFilters(filters: Filters): Record<string, string> {
  return Object.fromEntries(Object.entries({ q: filters.keyword.trim(), category: filters.category,
    source: filters.source, sort: filters.sortBy === 'score' ? '' : filters.sortBy,
    page: filters.page === 1 ? '' : String(filters.page) }).filter(([, v]) => v))
}
const root = '/api/v1/ai/plugins'
export function pluginPath(slug: string, namespace: string): string {
  const params = new URLSearchParams()
  if (namespace) params.set('namespace', namespace)
  return `${root}/${encodeURIComponent(slug)}${params.size ? '?' + params : ''}`
}
export function releasePath(entry: PluginEntry, action: 'readme' | 'download'): string {
  if (!entry.version) throw new Error('此技能暂未提供可用版本')
  const params = new URLSearchParams({ version: entry.version })
  if (entry.namespace) params.set('namespace', entry.namespace)
  return `${root}/${encodeURIComponent(entry.slug)}/${action}?${params}`
}
export function installPrompt(entry: PluginEntry): string {
  const coordinate = entry.namespace ? `${entry.namespace}/${entry.slug}` : entry.slug
  return `请从 SkillHub 查找并安装技能「${coordinate}」${entry.version ? `，固定版本 ${entry.version}` : ''}。安装前先展示文件和脚本审查结果，等待我的确认；不要自动执行包内脚本。`
}
async function responseError(response: Response): Promise<Error> {
  const body = await response.json().catch(() => ({}))
  return new Error(typeof body.error === 'string' ? body.error : `插件服务暂不可用（${response.status}）`)
}
export async function fetchJson<T>(url: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(url, { signal: signal ?? AbortSignal.timeout(20000) })
  if (!response.ok) throw await responseError(response)
  return response.json() as Promise<T>
}
export function fetchPlugins(filters: Filters, signal: AbortSignal): Promise<PluginPage> {
  const params = new URLSearchParams({ ...filters, page: String(filters.page) })
  return fetchJson(`${root}?${params}`, signal)
}
export async function downloadPlugin(entry: PluginEntry): Promise<void> {
  const response = await fetch(releasePath(entry, 'download'), { signal: AbortSignal.timeout(45000) })
  if (!response.ok) throw await responseError(response)
  const blob = await response.blob()
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${entry.slug}-${entry.version}.zip`
  document.body.append(link)
  link.click()
  link.remove()
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}
