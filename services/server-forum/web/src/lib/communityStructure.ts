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
  'design', 'casual-games', '3d', 'learn', 'casual-travel',
  'casual-screen', 'casual-feelings', 'casual-food', 'casual-work',
]
const hiddenDiscussionBoards = new Set([
  'casual-offwork', 'casual-loss', 'casual-dream', 'casual-think', 'casual-books', 'casual-fun',
])
const boardNames = new Map([
  ['learn', '教程'],
  ['casual-screen', '影视'],
  ['casual-feelings', '情感'],
  ['casual-indie', '副业'],
  ['casual-gear', '数码'],
  ['casual-chat', '闲聊'],
  ['resource-help', '求助'],
])

export function isHiddenDiscussionBoard(slug: string) {
  return hiddenDiscussionBoards.has(canonicalCategory(slug))
}

export function discussionTopicTag(tab: string): string | undefined {
  if (tab === 'design') return '设计'
  if (tab === '3d') return '3d'
}

function discussionBoards(categories: Category[]) {
  return activeCategories(categories).filter(c => navigationSection(c) === 'discuss' && !isHiddenDiscussionBoard(c.slug))
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
    const label = key === '3d' ? '3D' : discussionTopicTag(key) ?? (category && categoryLabel(category))
    return label ? [{ key, label, category: discussionTopicTag(key) ? undefined : category }] : []
  })
}
export function discussionTabs(categories: Category[]) {
  return [{key:'all',label:'全部'}, ...discussionDestinations(categories).map(({ key, label }) => ({ key, label }))]
}
export function sectionScope(categories:Category[], section:CommunitySection) {
  const boards = sectionBoards(categories, section)
  return boards.filter(board => section !== 'discuss' || !isHiddenDiscussionBoard(board.slug))
    .map(board => board.slug).join(',') || '__no_active_category__'
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
export function resourceTopicCategory(tab:string) {
  return ({media:'resource-movies',help:'resource-help'} as Record<string,string>)[tab] || canonicalCategory(tab)
}
export function categoryLabel(category: Pick<Category, 'slug' | 'name'>) {
  return boardNames.get(canonicalCategory(category.slug)) ?? category.name
}
function resourceNavigationBoards(categories: Category[]) {
  return sectionBoards(categories, 'resources').filter(board => board.kind !== 'uncategorized')
}
export function resourceTabs(categories:Category[]) {
  return [{key:'software',label:'开源软件'}, ...resourceNavigationBoards(categories).map(c => ({key:c.slug,label:categoryLabel(c)}))]
}
export function resourceTab(categories: Category[], rawTab: string) {
  const slug = resourceTopicCategory(rawTab)
  return sectionBoards(categories, 'resources').some(board => board.slug === slug) ? slug : 'software'
}
export function categoryDestination(category:Category, categories:Category[]):string {
  if (isHiddenDiscussionBoard(category.slug) || !activeCategories(categories).some(c=>c.id===category.id)) return `/c/${category.slug}`
  if (category.section === 'ideas') return '/'
  const leaf=!categories.some(child=>child.parent_id===category.id)
  if(category.section==='resources') return leaf ? '/resources?tab=' + category.slug : '/resources'
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
      : key === 'resources' ? resourceNavigationBoards(categories) : []
    return {key,name:SECTION_LABELS[key],href:key==='ideas'?'/':key==='resources'?'/resources':'/discuss',
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
