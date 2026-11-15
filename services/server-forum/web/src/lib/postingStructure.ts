import type { Category } from '../api/types'
import {
  activeCategories, canonicalCategory, discussionDestinations, discussionTab, discussionTopicTag,
  navigationSection, postableCategories, resourceTopicCategory, sectionBoards, SECTION_LABELS, type NavigationSectionKey,
} from './communityStructure.ts'
import { requiredPostLevel, TRUST_LABEL } from './format.ts'

export type PostingSectionKey = NavigationSectionKey
export interface PostingCategory extends Category { unavailableReason: string }
export interface PostingSection {
  key: PostingSectionKey
  name: string
  boards: PostingCategory[]
}
const sections = ['ideas', 'discuss', 'resources'] as const

export function postingSections(categories: Category[], level: number, editingCategoryId?: number): PostingSection[] {
  const eligibleIds = new Set(postableCategories(categories, level).map(category => category.id))
  const edited = categories.find(category => category.id === editingCategoryId)
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
  const parents = new Set(categories.map(category => category.parent_id))

  return sections.flatMap<PostingSection>(key => {
    const boards = key === 'discuss'
      ? discussionDestinations(categories).flatMap(({ category }) =>
        category && !parents.has(category.id) ? [category] : [])
      : sectionBoards(categories, key)
    if (edited && navigationSection(edited) === key && !parents.has(edited.id) && !boards.includes(edited)) {
      boards.push(edited)
    }
    return boards.length ? [{ key, name: SECTION_LABELS[key], boards: boards.map(toOption) }] : []
  })
}

export function postingSelection(destinations: PostingSection[], slug: string, categories: Category[], fallback: PostingSectionKey = 'discuss') {
  const canonical = canonicalCategory(slug)
  const category = categories.find(c => c.slug === canonical || c.slug === slug)
  const requestedSection = category ? navigationSection(category) : fallback
  const section = destinations.find(destination => destination.key === requestedSection)
  if (!section) return { sectionKey: destinations[0]?.key ?? requestedSection, categoryId: 0 }
  const board = section.boards.find(board => !board.unavailableReason
    && (slug ? board.id === category?.id : board.kind === 'uncategorized'))
  return { sectionKey: section.key, categoryId: board?.id ?? 0 }
}

export function postingLocation(page: unknown, query: Record<string, unknown>, categories: Category[], categorySlug?: unknown) {
  const read = (key: string) => typeof query[key] === 'string' ? query[key] : ''
  let section: PostingSectionKey = 'discuss'
  let slug = ''
  if (page === 'ideas') {
    section = 'ideas'
  } else if (page === 'resources') {
    section = 'resources'
    const tab = resourceTopicCategory(read('tab'))
    if (tab !== 'software') slug = tab
  } else if (page === 'discuss') {
    const current = discussionTab(categories, read('tab'), read('filter'), read('board'))
    if (current !== 'all' && !discussionTopicTag(current)) slug = current
  } else if (page === 'category' && typeof categorySlug === 'string') {
    slug = canonicalCategory(categorySlug)
  }
  const canonical = canonicalCategory(slug)
  const category = activeCategories(categories).find(board => board.slug === canonical || board.slug === slug)
  return category ? { section: navigationSection(category), category: category.slug } : { section }
}
