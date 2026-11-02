import seed from '../../../editorial/open-source.json' with { type: 'json' }

export interface OpenSourceProject {
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

// 随版本目录用于初始展示与旧服务兼容；stores/openSource 从已发布 API 更新，原生端读取同一接口。
export const openSourceCatalog: OpenSourceCatalog = seed
export const openSourceCategories = [{ key: 'recommend', label: '推荐', description: '发现值得尝试的开源项目' }, { key: 'hot', label: '热榜', description: '查看开源项目热榜与往期' }, ...seed.categories]
export const projectPath = (name: string) => `/open-source/${name.split('/').map(encodeURIComponent).join('/')}`
export const collectionPath = (key: string) => `/open-source/collections/${encodeURIComponent(key)}`
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
