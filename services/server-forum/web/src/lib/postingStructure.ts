import type { Category } from '../api/types'
import {
  activeCategories, canonicalCategory, discussionDestinations, discussionTab, discussionTopicTag,
  navigationSection, postableCategories, resourceTopicCategory, sectionBoards, SECTION_LABELS, type NavigationSectionKey,
} from './communityStructure.ts'
import { requiredPostLevel, TRUST_LABEL } from './format.ts'

export type PostingSectionKey = NavigationSectionKey
export interface PostingCategory extends Category {
  key: string
  topicTag?: string
  unavailableReason: string
}
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
  function option(board: Category, key = board.slug, name = board.name, topicTag?: string): PostingCategory {
    const parent = board.parent_id === null ? undefined : byId.get(board.parent_id)
    let unavailableReason = ''
    if (!eligibleIds.has(board.id)) {
      unavailableReason = board.posting_enabled && parent?.posting_enabled !== false
        ? `需${TRUST_LABEL[requiredPostLevel(board)]}`
        : '已关闭发帖'
    }
    return { ...board, key, name, topicTag, unavailableReason }
  }
  const parents = new Set(categories.map(category => category.parent_id))
  const discussion = discussionDestinations(categories)
  const discussionDefault = discussion.find(({ category }) => category?.kind === 'uncategorized')?.category

  return sections.flatMap<PostingSection>(key => {
    const boards = key === 'discuss'
      ? discussion.flatMap(({ key, label, category }) => {
        const tag = discussionTopicTag(key)
        const board = category ?? (tag && discussionDefault)
        return board && !parents.has(board.id) ? [option(board, key, label, tag)] : []
      })
      : sectionBoards(categories, key)
        .filter(board => key !== 'ideas' || board.kind === 'uncategorized')
        .map(board => option(board))
    if (edited && navigationSection(edited) === key && !boards.some(board => board.key === edited.slug)) {
      boards.push(option(edited))
    }
    return boards.length ? [{ key, name: SECTION_LABELS[key], boards }] : []
  })
}

export function postingSelection(destinations: PostingSection[], slug: string, categories: Category[], fallback: PostingSectionKey = 'discuss') {
  const canonical = canonicalCategory(slug)
  const category = categories.find(c => c.slug === canonical) ?? categories.find(c => c.slug === slug)
  const requestedSection = discussionTopicTag(canonical) ? 'discuss' : category ? navigationSection(category) : fallback
  const section = destinations.find(destination => destination.key === requestedSection)
  if (!section) return { sectionKey: destinations[0]?.key ?? requestedSection, boardKey: '' }
  const selected = section.boards.find(board => board.key === canonical || board.key === category?.slug)
  const useDefault = !slug || (!selected && requestedSection === 'ideas' && category?.status === 'active')
  const board = useDefault ? section.boards.find(option => option.kind === 'uncategorized' && !option.topicTag) : selected
  return { sectionKey: section.key, boardKey: board && !board.unavailableReason ? board.key : '' }
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
    if (discussionTopicTag(current)) return { section, category: current }
    if (current !== 'all') slug = current
    else if (!categories.length) slug = read('board') || (read('tab') === 'casual' && read('filter')) || read('tab')
  } else if (page === 'category' && typeof categorySlug === 'string') {
    slug = canonicalCategory(categorySlug)
  }
  const canonical = canonicalCategory(slug)
  const category = activeCategories(categories).find(board => board.slug === canonical || board.slug === slug)
  if (category) return { section: navigationSection(category), category: category.slug }
  return !categories.length && slug && slug !== 'all' ? { section, category: canonical } : { section }
}
