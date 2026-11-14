import channels from '../../../editorial/channels.json' with { type: 'json' }
import type { Category } from '../api/types'
import { requiredPostLevel } from './format.ts'
export function canonicalCategory(slug:string) { return (channels.aliases as Record<string,string>)[slug] || slug }
export const CATEGORY_ALIASES = channels.aliases as Record<string,string>
export type CommunitySection = 'ideas' | 'discuss' | 'resources' | 'site'
export const SECTION_LABELS: Record<CommunitySection,string> = {ideas:'创意',discuss:'讨论',resources:'资源',site:'站务'}
export function activeCategories(categories: Category[]) {
  return categories.filter(c => c.status === 'active' && (!c.parent_id || categories.some(p => p.id === c.parent_id && p.status === 'active')))
}
export function sectionCategories(categories: Category[], section: CommunitySection) {
  return activeCategories(categories).filter(c => c.section === section)
}
export function sectionLeaves(categories:Category[], section:CommunitySection) {
  return sectionCategories(categories,section).filter(c=>!categories.some(child=>child.parent_id===c.id))
}
const discussionOrder = [
  'design', 'casual-games', '3d', 'learn', 'casual-travel', 'casual-books',
  'casual-screen', 'casual-feelings', 'casual-food', 'casual-work',
]

export function discussionTopicTag(tab: string): string | undefined {
  if (tab === 'design') return '设计'
  if (tab === '3d') return '3d'
}

function discussionBoards(categories: Category[]) {
  return activeCategories(categories).filter(c => navigationSection(c) === 'discuss')
}

export function discussionDestinations(categories: Category[]) {
  const byId = new Map(categories.map(c => [c.id, c]))
  const parents = new Set(categories.map(c => c.parent_id))
  const primary = discussionBoards(categories).filter(c => c.parent_id === null
    ? c.slug !== 'casual' || !parents.has(c.id)
    : byId.get(c.parent_id)?.slug === 'casual')
  const bySlug = new Map(primary.map(c => [c.slug, c]))
  const order = new Set([...discussionOrder, ...bySlug.keys()])
  return Array.from(order).flatMap(key => {
    const category = bySlug.get(key)
    const label = key === '3d' ? '3D' : discussionTopicTag(key) ?? category?.name
    return label ? [{ key, label, category: discussionTopicTag(key) ? undefined : category }] : []
  })
}
export function discussionTabs(categories: Category[]) {
  return [{key:'all',label:'全部'}, ...discussionDestinations(categories).map(({ key, label }) => ({ key, label }))]
}
export function discussionFilters(categories: Category[], tab: string) {
  const rows = discussionBoards(categories)
  const root = rows.find(c => c.slug === tab && c.parent_id === null)
  return root ? [{key:'',label:'全部'},...rows.filter(c => c.parent_id === root.id).map(c=>({key:c.slug,label:categoryLabel(c)}))] : []
}
export function learnFilters(categories: Category[]) { return discussionFilters(categories,'learn') }
export function discussionFilter(categories:Category[],tab:string,raw:string) {
  const options = discussionFilters(categories,tab)
  const direct = canonicalCategory(raw)
  const key = options.some(c=>c.key === direct) ? direct : tab === 'learn' ? learnCategory(raw) : ''
  return options.some(c=>c.key === key) ? key : ''
}
// Legacy query parameters are input aliases; current labels and availability come from the API.
export function learnCategory(filter:string) {
  return ({'claude-code':'learn-claude-code',codex:'learn-codex','提示词':'learn-prompts','工作流':'learn-workflows','本地部署':'learn-local','ai-硬件':'learn-hardware'} as Record<string,string>)[filter] || (filter.startsWith('learn-') ? filter : 'learn')
}
export function sectionScope(categories:Category[], section:CommunitySection) {
  return sectionLeaves(categories,section).map(c=>c.slug).join(',') || '__no_active_category__'
}
export function discussionCategory(tab:string, filter='', categories:Category[] = []) {
  if (tab === 'all' || discussionTopicTag(tab)) return sectionScope(categories,'discuss')
  return discussionFilter(categories,tab,filter) || canonicalCategory(tab)
}
export function discussionRoute(categories: Category[], rawTab: string, rawFilter = '', rawBoard = '') {
  const legacyTab = canonicalCategory(rawTab === 'casual' && rawFilter ? rawFilter : rawTab)
  const tab = discussionTabs(categories).some(item => item.key === legacyTab) ? legacyTab : 'all'
  const requested = canonicalCategory(rawBoard || legacyTab)
  const board = requested !== tab && discussionBoards(categories).some(category => category.slug === requested)
    ? requested
    : ''
  const filter = discussionFilter(categories, board || tab, rawFilter)
  return { tab, board, filter }
}
export function resourceTopicCategory(tab:string) {
  return ({media:'resource-movies',help:'resource-help'} as Record<string,string>)[tab] || canonicalCategory(tab)
}
export function categoryLabel(category:Pick<Category,'name'>) { return category.name }
export function resourceTabs(categories:Category[]) {
  return [{key:'software',label:'开源软件'}, ...sectionLeaves(categories, 'resources').map(c => ({key:c.slug,label:c.name}))]
}
export function categoryDestination(category:Category, categories:Category[]):string {
  if (!activeCategories(categories).some(c=>c.id===category.id)) return `/c/${category.slug}`
  const leaf=!categories.some(child=>child.parent_id===category.id)
  if(category.section==='resources') return leaf ? '/resources?tab=' + category.slug : '/resources'
  if(category.section==='ideas') return leaf ? `/ideas?category=${category.slug}` : '/ideas'
  if(category.section==='discuss') {
    const parent=categories.find(c=>c.id===category.parent_id)
    return parent && parent.slug !== 'casual'
      ? `/discuss?tab=${parent.slug}&filter=${category.slug}`
      : `/discuss?tab=${category.slug === 'casual' && !leaf ? 'all' : category.slug}`
  }
  return `/c/${category.slug}`
}

export type NavigationSectionKey = 'ideas' | 'discuss' | 'resources'
export function navigationSection(category: Pick<Category,'section'>):NavigationSectionKey {
  return category.section === 'site' ? 'discuss' : category.section
}
export function navigationSections(categories:Category[]) {
  const active = activeCategories(categories)
  return (['ideas','discuss','resources'] as const).map(key=>{
    const rows = active.filter(c=>navigationSection(c)===key)
    return {key,name:SECTION_LABELS[key],href:key==='ideas'?'/ideas':key==='resources'?'/resources':'/discuss',
      children:rows.filter(c=>!categories.some(child=>child.parent_id===c.id)),
      topic_count:rows.reduce((total,c)=>total+c.topic_count,0)}
  })
}
export function categoryHref(category:Pick<Category,'id'|'slug'>, categories:Category[]) {
  const current=categories.find(c=>c.id===category.id || c.slug===canonicalCategory(category.slug))
  return current ? categoryDestination(current,categories) : `/c/${encodeURIComponent(category.slug)}`
}
export function postableCategories(categories:Category[],level=3) {
  const active=activeCategories(categories)
  return active.filter(c=>c.posting_enabled && requiredPostLevel(c)<=level
    && !categories.some(child=>child.parent_id===c.id)
    && (!c.parent_id || categories.some(p=>p.id===c.parent_id && p.posting_enabled)))
}
