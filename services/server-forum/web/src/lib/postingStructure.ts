import type { Category } from '../api/types'
import {
  activeCategories, canonicalCategory, discussionDestinations, discussionRoute, discussionTopicTag, navigationSection, postableCategories,
  resourceTopicCategory, SECTION_LABELS, type NavigationSectionKey,
} from './communityStructure.ts'
import { requiredPostLevel, TRUST_LABEL } from './format.ts'

export type PostingSectionKey = NavigationSectionKey
export interface PostingCategory extends Category { unavailableReason: string }
export interface PostingGroup { id: number; name: string; categories: PostingCategory[]; expandable?: boolean }
export interface PostingSection {
  key: PostingSectionKey
  name: string
  groups: PostingGroup[]
}
const sections = ['ideas', 'discuss', 'resources'] as const

export function postingSections(categories: Category[], level: number, editingCategoryId?: number): PostingSection[] {
  const parents = new Set(categories.map(category => category.parent_id))
  const boards = activeCategories(categories).filter(category => !parents.has(category.id))
  const eligibleIds = new Set(postableCategories(categories, level).map(category => category.id))
  const edited = categories.find(category => category.id === editingCategoryId)
  if (edited && !parents.has(edited.id) && !boards.includes(edited)) boards.push(edited)
  if (edited) eligibleIds.add(edited.id)

  const byId = new Map(categories.map(category => [category.id, category]))
  const toOption = (board: Category): PostingCategory => {
    const parent = board.parent_id === null ? undefined : byId.get(board.parent_id)
    let unavailableReason = ''
    if (!eligibleIds.has(board.id)) {
      unavailableReason = board.posting_enabled && parent?.posting_enabled !== false
        ? `需${TRUST_LABEL[requiredPostLevel(board)]}`
        : '已关闭发帖'
    }
    return { ...board, unavailableReason }
  }
  const discussion = discussionDestinations(categories).flatMap(({ category }) => {
    if (!category) return []
    const options = boards.filter(board => board.id === category.id || board.parent_id === category.id)
    return options.length ? [{
      id: category.id,
      name: category.name,
      categories: options.map(toOption),
      expandable: !options.some(board => board.id === category.id),
    }] : []
  })
  if (edited && navigationSection(edited) === 'discuss' && !discussion.some(group =>
    group.categories.some(board => board.id === edited.id),
  ) && boards.includes(edited)) {
    discussion.push({ id: edited.id, name: edited.name, categories: [toOption(edited)], expandable: false })
  }

  return sections.flatMap<PostingSection>(key => {
    if (key === 'discuss') return discussion.length ? [{ key, name: SECTION_LABELS[key], groups: discussion }] : []
    const groups = new Map<number, PostingGroup>()
    for (const board of boards) {
      if (navigationSection(board) !== key) continue
      const parent = (board.parent_id === null ? undefined : byId.get(board.parent_id)) ?? board
      let group = groups.get(parent.id)
      if (!group) {
        group = { id: parent.id, name: parent.name, categories: [] }
        groups.set(parent.id, group)
      }
      group.categories.push(toOption(board))
    }
    return groups.size ? [{ key, name: SECTION_LABELS[key], groups: [...groups.values()] }] : []
  })
}

export function postingSelection(destinations: PostingSection[], slug: string, categories: Category[], fallback: PostingSectionKey = 'discuss') {
  const canonical = canonicalCategory(slug)
  const category = categories.find(c => c.slug === canonical || c.slug === slug)
  const matchingSection = category && destinations.find(destination => destination.key === navigationSection(category))
  const section = matchingSection ?? destinations.find(destination => destination.key === fallback) ?? destinations[0]
  const available = section?.groups.flatMap(group => group.categories).filter(board => !board.unavailableReason) ?? []
  const choices = slug ? available.filter(board => board.id === category?.id || board.parent_id === category?.id) : available
  return { sectionKey: section?.key ?? fallback, categoryId: choices[0]?.id ?? 0 }
}

export function postingLocation(page: unknown, query: Record<string, unknown>, categories: Category[], categorySlug?: unknown) {
  const read = (key: string) => typeof query[key] === 'string' ? query[key] : ''
  let section: PostingSectionKey = 'discuss'
  let slug = ''
  if (page === 'ideas') {
    section = 'ideas'
    slug = read('category')
  } else if (page === 'resources') {
    section = 'resources'
    const tab = resourceTopicCategory(read('tab'))
    if (tab !== 'software') slug = tab
  } else if (page === 'discuss') {
    const { tab, board, filter } = discussionRoute(categories, read('tab'), read('filter'), read('board'))
    const current = board || tab
    if (board || (current !== 'all' && !discussionTopicTag(current))) slug = filter || current
  } else if (page === 'category' && typeof categorySlug === 'string') {
    slug = canonicalCategory(categorySlug)
  }
  const canonical = canonicalCategory(slug)
  const category = activeCategories(categories).find(board => board.slug === canonical || board.slug === slug)
  return category ? { section: navigationSection(category), category: category.slug } : { section }
}
