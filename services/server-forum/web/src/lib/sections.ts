export type Section = 'home' | 'resources' | 'discuss'
export type Area = 'community' | 'ai' | 'create' | 'me'
type Route = { name?: unknown; params: Record<string, unknown> }
export function sectionOf(route: Route): Section {
  if (['home', 'daily', 'daily-detail', 'model-rankings', 'weekly-issue', 'project'].includes(String(route.name))) return 'home'
  if (['resources', 'tool-detail'].includes(String(route.name))) return 'resources'
  if (route.name === 'category') {
    const slug = String(route.params.slug ?? '')
    return slug === 'share' || slug.startsWith('share-') ? 'resources' : 'discuss'
  }
  return 'discuss'
}
export function areaOf(route: Route): Area {
  const name = String(route.name)
  if (name === 'ai') return 'ai'
  if (['create', 'pack', 'schemes', 'scheme', 'scheme-publish', 'publish'].includes(name)) return 'create'
  if (['me', 'works', 'content-bookmarks', 'scheme-backups', 'settings', 'notifications', 'points', 'user'].includes(name)) return 'me'
  return 'community'
}
export function safeReturnPath(path: string): boolean {
  return path.startsWith('/') && !path.startsWith('//') && !/[\\\r\n]/.test(path)
}
