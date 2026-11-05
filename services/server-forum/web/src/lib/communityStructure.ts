import channels from '../../../editorial/channels.json' with { type: 'json' }
import type { Category } from '../api/types'
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
export function discussionTabs(categories: Category[]) {
  const rows = sectionCategories(categories, 'discuss')
  return [{key:'all',label:'全部'}, ...rows.filter(c => c.slug === 'learn' || c.parent_id !== null && !rows.some(p => p.id === c.parent_id && p.slug === 'learn')).map(c => ({key:c.slug,label:c.name}))]
}
export function learnFilters(categories: Category[]) {
  const root = categories.find(c => c.slug === 'learn')
  return [{key:'',label:'全部'},...sectionCategories(categories,'discuss').filter(c => root && c.parent_id === root.id).map(c => ({key:c.slug,label:c.name}))]
}
// Legacy query parameters are input aliases; current labels and availability come from the API.
export function learnCategory(filter:string) {
  return ({'claude-code':'learn-claude-code',codex:'learn-codex','提示词':'learn-prompts','工作流':'learn-workflows','本地部署':'learn-local','ai-硬件':'learn-hardware'} as Record<string,string>)[filter] || (filter.startsWith('learn-') ? filter : 'learn')
}
export function sectionScope(categories:Category[], section:CommunitySection) {
  return sectionCategories(categories,section).filter(c => c.parent_id !== null).map(c=>c.slug).join(',') || '__no_active_category__'
}
export function discussionCategory(tab:string, filter='', categories:Category[] = []) {
  return tab === 'all' ? sectionScope(categories,'discuss') : tab === 'learn' ? learnCategory(filter) : canonicalCategory(tab)
}
export function resourceTopicCategory(tab:string) {
  return ({media:'resource-movies',help:'resource-help',open:'resource-open'} as Record<string,string>)[tab] || canonicalCategory(tab)
}
export function resourceTabs(categories:Category[]) {
  return [{key:'software',label:'开源软件'},...sectionCategories(categories,'resources').filter(c=>c.parent_id !== null).map(c=>({key:c.slug,label:c.slug==='resource-open' ? '开源讨论' : c.name}))]
}
export function categoryDestination(category:Category, categories:Category[]):string {
  if (!activeCategories(categories).some(c=>c.id===category.id)) return `/c/${category.slug}`
  if(category.section==='resources') return category.parent_id ? `/resources?tab=${category.slug}` : '/resources'
  if(category.section==='ideas') return category.parent_id ? `/ideas?category=${category.slug}` : '/ideas'
  if(category.section==='discuss') {
    const parent=categories.find(c=>c.id===category.parent_id)
    return parent?.slug==='learn' ? `/discuss?tab=learn&filter=${category.slug}` : `/discuss?tab=${category.parent_id ? category.slug : category.slug==='learn' ? 'learn' : 'all'}`
  }
  return `/c/${category.slug}`
}
