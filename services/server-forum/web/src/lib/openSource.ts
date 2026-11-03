import seed from '../../../editorial/open-source.json' with { type: 'json' }

export interface OpenSourceProject {
  stars?: number
  last_pushed_at?: string
  full_name: string
  name: string
  title: string
  summary: string
  category: string
  related_categories: string[]
  tags: string[]
  effort: string
  cover_url: string
  cover_caption: string
  icon_url?: string
  icon_source_url?: string
  cover_style?: string
  cover_layout?: string
  cover_accent?: string
  cover_lines?: string[]
  cover_template?: string
  cover_variant?: string
  cover_focus_x?: number
  cover_focus_y?: number
  icon_fit?: string
  homepage_url: string
  highlights: string[]
  audience: string
  getting_started: string
  notes: string[]
  source_url: string
  featured: boolean
  order: number
}
export type OpenSourceProjectSummary = Omit<OpenSourceProject,
  'effort' | 'cover_caption' | 'icon_source_url' | 'homepage_url' | 'highlights' |
  'audience' | 'getting_started' | 'notes' | 'source_url' | 'featured' | 'order'>

export interface OpenSourceCollection {
  key: string
  title: string
  summary: string
  description: string
  projects: string[]
  notes: string[]
}
export interface OpenSourceCatalog {
  version: number
  reviewed_at: string
  notice: string
  categories: { key: string; label: string; description: string }[]
  projects: OpenSourceProject[]
  collections: OpenSourceCollection[]
}

// 随版本目录用于校验采编数据。公开页面通过分页接口读取已发布目录。
export const openSourceCatalog: OpenSourceCatalog = seed
export const openSourceCategories = [{ key: 'recommend', label: '推荐', description: '发现值得尝试的开源项目' }, { key: 'hot', label: '热榜', description: '查看开源项目热榜与往期' }, ...seed.categories]
export const categoryLabel = (key: string) => seed.categories.find(item => item.key === key)?.label || key
export function findProject(name: string, catalog: OpenSourceCatalog = openSourceCatalog) {
  return catalog.projects.find(item => item.full_name.toLowerCase() === name.toLowerCase())
}
export function collectionProjects(collection: OpenSourceCollection, catalog: OpenSourceCatalog = openSourceCatalog) {
  return collection.projects.flatMap(name => { const project = findProject(name, catalog); return project ? [project] : [] })
}
export function filterProjects(projects: OpenSourceProject[], category: string, query = '', tag = '') {
  const text = query.trim().toLocaleLowerCase()
  return projects.filter(project =>
    (category === 'all' || (category === 'recommend' ? project.featured : project.category === category || project.related_categories.includes(category))) &&
    (!tag || project.tags.includes(tag)) &&
    (!text || [project.name, project.full_name, project.title, project.summary, ...project.tags].join(' ').toLocaleLowerCase().includes(text)),
  ).sort((a, b) => a.order - b.order)
}
