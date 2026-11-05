import type { Category } from '../api/types'
import { postableCategories } from './communityStructure.ts'

export type PostingSectionKey = 'ideas' | 'discuss' | 'resources'
export interface PostingGroup { id: number; name: string; categories: Category[] }
export interface PostingSection {
  key: PostingSectionKey
  name: string
  description: string
  groups: PostingGroup[]
}
const sections: Omit<PostingSection, 'groups'>[] = [
  { key: 'ideas', name: '创意', description: '分享创作思路、作品经验与使用技巧' },
  { key: 'discuss', name: '讨论', description: '教程、生活话题与社区反馈' },
  { key: 'resources', name: '资源', description: '开源讨论、影视与求助悬赏' },
]
export function postingSections(categories: Category[], level: number, editingCategoryId?: number): PostingSection[] {
  const selectable = postableCategories(categories,level)
  const edited = categories.find(c=>c.id===editingCategoryId)
  if (edited && !selectable.some(c=>c.id===edited.id)) selectable.push(edited)
  return sections.map(section => {
    const groups: PostingGroup[] = []
    for (const category of selectable) {
      const key = category.section === 'site' ? 'discuss' : category.section
      if (key !== section.key) continue
      const parent = categories.find(c => c.id === category.parent_id) ?? category
      let group = groups.find(g => g.id === parent.id)
      if (!group) { group = { id: parent.id, name: parent.name, categories: [] }; groups.push(group) }
      group.categories.push(category)
    }
    return { ...section, groups }
  }).filter(section => section.groups.length > 0)
}

export function postingSelection(sections: PostingSection[], slug: string, categories: Category[], fallback: PostingSectionKey = 'discuss') {
  const category = categories.find(c => c.slug === slug)
  const section = sections.find(s => s.groups.some(g => g.categories.some(c => c.id === category?.id) || g.id === category?.id))
    ?? sections.find(s => s.key === fallback) ?? sections[0]
  const leaves = section?.groups.flatMap(g => g.categories) ?? []
  return { sectionKey: section?.key ?? fallback, categoryId: leaves.find(c => c.id === category?.id)?.id ?? (leaves.length === 1 ? leaves[0]!.id : 0) }
}
