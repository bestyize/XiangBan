import { request } from './admin'
export interface OperationsColumn {
  kind: string
  published: number
  future: number
  unlisted: number
  pending: number
  scheduled: number
  sample: number
  last_live_update: string | null
}
export interface OperationsSource {
  id: string
  title: string
  enabled: boolean
  interval_minutes: number
  last_success_at: string | null
  last_error: string | null
  next_run_at: string
  active: boolean
  failed_24h: number
  succeeded_24h: number
  pending: number
  freshness: 'fresh' | 'stale' | 'never' | 'disabled'
}
export interface OperationsSummary {
  as_of: string
  columns: OperationsColumn[]
  sources: OperationsSource[]
  links: { total: number; statuses: Record<string, number> }
  freshness_rule: string
  candidate_scope: string
}
export interface LinkHealthSettings {
  enabled: boolean
  interval_hours: number
  next_run_at: string
  active_run: string | null
  lease_until: string | null
}
export interface LinkHealthRun {
  id: string
  status: string
  started_at: string
  finished_at: string | null
  total_candidates: number
  selected_count: number
  checked_count: number
  healthy_count: number
  attention_count: number
  blocked_count: number
  error: string | null
}
export interface LinkHealthItem {
  id: string
  url: string
  host: string
  refs: { kind: string; key: string; field: string; scope: string }[]
  status: string
  http_status: number | null
  final_url?: string | null
  error: string | null
  duration_ms: number | null
  last_checked_at: string | null
  last_success_at: string | null
  failures: number
}
export interface LinkHealthPage {
  items: LinkHealthItem[]
  total: number
  page: number
  page_size: number
  settings: LinkHealthSettings
  runs: LinkHealthRun[]
  max_links_per_run: number
  max_run_seconds: number
  candidate_scope: string
}
export const operationsApi = {
  summary: () => request<OperationsSummary>('GET', '/community-operations'),
  links: (page = 1, status = '') => request<LinkHealthPage>('GET', `/link-health?page=${page}&status=${encodeURIComponent(status)}`),
  run: () => request<{ run_id: string; total_candidates: number; selected_count: number }>('POST', '/link-health/run', {}),
  settings: (enabled: boolean, intervalHours: number) => request<{ ok: true }>('PATCH', '/link-health/settings', { enabled, interval_hours: intervalHours }),
}
