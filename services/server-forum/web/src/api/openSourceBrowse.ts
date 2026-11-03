import { request } from './forum'
import type { OpenSourceCatalog, OpenSourceCollection, OpenSourceProject, OpenSourceProjectSummary } from '@/lib/openSource'

type Metadata = Pick<OpenSourceCatalog, 'categories' | 'reviewed_at' | 'notice'> & { revision: number }
export type ProjectPage = Metadata & {
  projects: OpenSourceProjectSummary[]
  collections: (OpenSourceCollection & { project_names: string[] })[]
  total: number
  next_page: number | null
  tags: string[]
}

export const openSourceBrowse = {
  page(category: string, tag: string, page: number, q = '') {
    const query = new URLSearchParams({ category, tag, page: String(page), q })
    return request<ProjectPage>('GET', `/open-source/projects?${query}`, undefined, false)
  },
  project(owner: string, repo: string) {
    return request<Metadata & { projects: OpenSourceProject[]; related: OpenSourceProjectSummary[] }>(
      'GET', `/open-source/projects/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`, undefined, false,
    )
  },
  collection(key: string) {
    return request<Metadata & { projects: OpenSourceProjectSummary[]; collections: OpenSourceCollection[] }>(
      'GET', `/open-source/collections/${encodeURIComponent(key)}`, undefined, false,
    )
  },
}
