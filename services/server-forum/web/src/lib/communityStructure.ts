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
export function discussionTabs(categories: Category[]) {
  const rows = sectionCategories(categories, 'discuss')
  const groups = rows.filter(c => c.parent_id === null && c.slug !== 'casual')
  const groupIds = new Set(groups.map(c => c.id))
  const flat = rows.filter(c => c.parent_id !== null && !groupIds.has(c.parent_id))
  return [{key:'all',label:'全部'}, ...groups.concat(flat).map(c => ({key:c.slug,label:categoryLabel(c)}))]
}
export function discussionFilters(categories: Category[], tab: string) {
  const rows = sectionCategories(categories,'discuss')
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
  if (tab === 'all') return sectionScope(categories,'discuss')
  return discussionFilter(categories,tab,filter) || canonicalCategory(tab)
}
export function resourceTopicCategory(tab:string) {
  return ({media:'resource-movies',help:'resource-help',open:'resource-open'} as Record<string,string>)[tab] || canonicalCategory(tab)
}
export function categoryLabel(category:Pick<Category,'name'>) { return category.name }
export function resourceTabs(categories:Category[]) {
  return [{key:'software',label:'开源软件'},...sectionLeaves(categories,'resources').map(c=>({key:c.slug,label:categoryLabel(c)}))]
}
export function categoryDestination(category:Category, categories:Category[]):string {
  if (!activeCategories(categories).some(c=>c.id===category.id)) return `/c/${category.slug}`
  const leaf=!categories.some(child=>child.parent_id===category.id)
  if(category.section==='resources') return leaf ? `/resources?tab=${category.slug}` : '/resources'
  if(category.section==='ideas') return leaf ? `/ideas?category=${category.slug}` : '/ideas'
  if(category.section==='discuss') {
    const parent=categories.find(c=>c.id===category.parent_id)
    return parent && parent.slug !== 'casual' ? `/discuss?tab=${parent.slug}&filter=${category.slug}` : `/discuss?tab=${category.slug === 'casual' ? 'all' : category.slug}`
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
