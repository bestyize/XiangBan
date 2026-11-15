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
export function sectionBoards(categories:Category[], section:CommunitySection) {
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
  const defaultBoards = primary.filter(category => category.kind === 'uncategorized')
  const order = new Set([...defaultBoards.map(category => category.slug), ...discussionOrder, ...bySlug.keys()])
  return Array.from(order).flatMap(key => {
    const category = bySlug.get(key)
    const label = key === '3d' ? '3D' : discussionTopicTag(key) ?? category?.name
    return label ? [{ key, label, category: discussionTopicTag(key) ? undefined : category }] : []
  })
}
export function discussionTabs(categories: Category[]) {
  return [{key:'all',label:'全部'}, ...discussionDestinations(categories).map(({ key, label }) => ({ key, label }))]
}
export function sectionScope(categories:Category[], section:CommunitySection) {
  return sectionBoards(categories,section).map(c=>c.slug).join(',') || '__no_active_category__'
}
export function discussionCategory(tab:string, categories:Category[]) {
  if (tab === 'all' || discussionTopicTag(tab)) return sectionScope(categories,'discuss')
  return tab
}
export function discussionTab(categories: Category[], rawTab: string, rawFilter = '', rawBoard = '') {
  const tabs = discussionTabs(categories)
  for (const raw of [rawBoard, rawTab === 'casual' ? rawFilter : '', rawTab]) {
    if (!raw) continue
    const slug = canonicalCategory(raw)
    if (tabs.some(tab => tab.key === slug)) return slug
    const board = categories.find(category => category.slug === raw)
    const parent = categories.find(category => category.id === board?.parent_id)
    if (parent && tabs.some(tab => tab.key === parent.slug)) return parent.slug
  }
  return 'all'
}
export function ideaBoard(categories: Category[], raw: string) {
  const slug = canonicalCategory(raw)
  return sectionBoards(categories, 'ideas').find(category => category.slug === slug)?.slug ?? ''
}
export function resourceTopicCategory(tab:string) {
  return ({media:'resource-movies',help:'resource-help'} as Record<string,string>)[tab] || canonicalCategory(tab)
}
export function categoryLabel(category:Pick<Category,'name'>) { return category.name }
export function resourceTabs(categories:Category[]) {
  return [{key:'software',label:'开源软件'}, ...sectionBoards(categories, 'resources').map(c => ({key:c.slug,label:c.name}))]
}
export function categoryDestination(category:Category, categories:Category[]):string {
  if (!activeCategories(categories).some(c=>c.id===category.id)) return `/c/${category.slug}`
  const leaf=!categories.some(child=>child.parent_id===category.id)
  if(category.section==='resources') return leaf ? '/resources?tab=' + category.slug : '/resources'
  if(category.section==='ideas') return leaf ? `/ideas?category=${category.slug}` : '/ideas'
  const tab = discussionTab(categories, category.slug)
  return tab === 'all' ? '/discuss' : `/discuss?tab=${tab}`
}

export type NavigationSectionKey = 'ideas' | 'discuss' | 'resources'
export function navigationSection(category: Pick<Category,'section'>):NavigationSectionKey {
  return category.section === 'site' ? 'discuss' : category.section
}
export function navigationSections(categories:Category[]) {
  const active = activeCategories(categories)
  return (['ideas','discuss','resources'] as const).map(key=>{
    const rows = active.filter(c=>navigationSection(c)===key)
    const children = key === 'discuss'
      ? discussionDestinations(categories).flatMap(({ category }) => category ? [category] : [])
      : sectionBoards(categories, key)
    return {key,name:SECTION_LABELS[key],href:key==='ideas'?'/ideas':key==='resources'?'/resources':'/discuss',
      children,
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
