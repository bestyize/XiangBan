// 采集器只生成待审内容；人工终审通过后才进入公开接口。
import { request as adminRequest, type AdminPage, type DiscoverDraft } from './admin'

/** github_repos 最多 30 项；github_tools 最多 10 项，均由后端再次校验。 */
export interface CollectionSourceConfig { repositories?: string[]; feed_ids?: string[] }

export interface CollectionSource {
  id: string
  title: string
  configured: boolean
  credential_required?: boolean
  config?: CollectionSourceConfig
  hint: string | null
  enabled: boolean
  interval_minutes: number
  last_success_at: string | null
  last_error: string | null
  next_run_at: string | null
}

export interface CollectionRun {
  id: string
  source: string
  status: string
  started_at: string
  finished_at: string | null
  item_count: number
  error: string | null
}

export interface CollectionDraft {
  kind: string
  key: string
  expected_revision: number
  draft: DiscoverDraft | null
  live: { data: Record<string, unknown>; sort_order: number; published_at: string | null; updated_at: string; status: string } | null
  conflict: boolean
}

export interface CollectionBatch {
  run_id: string
  source: string
  status: string
  reviewed_at: string | null
  items: CollectionDraft[]
}

export const collectionApi = {
  sources: () => adminRequest<{ items: CollectionSource[] }>('GET', '/collection/sources'),
  credential: (source: string, key: string) => adminRequest<unknown>('PUT', `/collection/sources/${encodeURIComponent(source)}/credential`, { key }),
  configure: (source: string, body: { enabled?: boolean; interval_minutes?: number; config?: CollectionSourceConfig }) => adminRequest<unknown>('PATCH', `/collection/sources/${encodeURIComponent(source)}`, body),
  run: (source: string) => adminRequest<{ run_id: string }>('POST', `/collection/sources/${encodeURIComponent(source)}/run`),
  runs: (source = '', page = 1) => {
    const query = new URLSearchParams({ page: String(page) })
    if (source) query.set('source', source)
    return adminRequest<AdminPage<CollectionRun>>('GET', `/collection/runs?${query}`)
  },
  drafts: (run: string) => adminRequest<CollectionBatch>('GET', `/collection/runs/${encodeURIComponent(run)}/drafts`),
  review: (run: string, items: CollectionDraft[]) => adminRequest<{ status: 'published'; item_count: number }>('POST', `/collection/runs/${encodeURIComponent(run)}/review`, {
    items: items.map(({ kind, key, expected_revision }) => ({ kind, key, expected_revision })),
  }),
}
