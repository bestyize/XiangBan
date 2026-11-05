import type { Category } from '../api/types'
import { canonicalCategory } from './communityStructure.ts'
export type Section = 'home' | 'resources' | 'discuss' | 'ideas'
export type Area = 'community' | 'ai' | 'create' | 'me'
type Route = { name?: unknown; params: Record<string, unknown> }
export function sectionOf(route: Route, categories: Pick<Category,'slug'|'section'>[] = []): Section {
  if (['home', 'frontier', 'frontier-detail', 'daily', 'daily-detail', 'model-rankings', 'weekly-issue', 'project', 'open-source', 'open-source-project', 'open-source-collection'].includes(String(route.name))) return 'home'
  if (route.name === 'ideas') return 'ideas'
  if (['resources', 'tool-detail'].includes(String(route.name))) return 'resources'
  if (route.name === 'category') {
    const slug = String(route.params.slug ?? '')
    const category = categories.find(c=>c.slug===canonicalCategory(slug))
    return category?.section === 'ideas' ? 'ideas' : category?.section === 'resources' ? 'resources' : 'discuss'
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
