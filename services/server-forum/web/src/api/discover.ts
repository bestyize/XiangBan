import { request } from './forum'
import type { Page } from './types'
import type { PackItem } from './content'
import { discoverPaths as path } from '@/components/community/contract'

export interface NewsItem { key: string; title: string; source?: string; url?: string; summary?: string; body_html?: string; cover_url?: string; published_at?: string; source_published_at?: string | null; time_basis?: string; sample?: boolean }
export interface HomeBundle { banners: { key: string; title: string; tag?: string; byline?: string; link?: string; sample?: boolean }[]; news: { date: string | null; total: number; items: NewsItem[] }; featured: PackItem[]; repos: RepoIssue | null; sample?: boolean }
export interface RepoRank { rank: number; full_name: string; description?: string; description_zh?: string; language?: string; star_total: number; star_gain?: number | null; suspected_growth?: boolean; complete?: boolean; observed_days?: number | null }
export interface RepoIssue { key: string; issue_no: number; week_start: string; week_end: string; source: string; metric: string; as_of?: string | null; total_gain?: number | null; wow_pct?: number | null; observed_days?: number | null; complete?: boolean; sample?: boolean; ranks: RepoRank[] }
export interface RepoIssueRef { key: string; issue_no: number; week_start: string; week_end: string }
export interface Tracker { now: string; sample?: boolean; quota?: { progress_pct: number; next_reset_at: string; published_at: string; sample?: boolean } | null; launches: { key: string; title: string; tags?: string[]; published_at?: string; summary?: string; source_url?: string | null; sample?: boolean }[]; promos: { key: string; text: string; eligibility?: string; ends_at?: string | null; days_left?: number | null; source_url?: string | null; sample?: boolean }[] }
export interface NavItem { sample?: boolean; key: string; name: string; url: string; vendor?: string }
export interface DocItem { sample?: boolean; key: string; title: string; url: string; product?: string }
export interface Group<T> { name: string; items: T[] }
export interface ToolSummary { key: string; name: string; developer?: string; type_label?: string; license?: string; platforms?: string[]; icon_url?: string; latest_version?: string | null; latest_date?: string | null; sample?: boolean }
export interface ToolArtifact { os: string; arch?: string; filename?: string; asset_id?: number | null; desc?: string; size?: number; sha256?: string; mirror_url?: string; github_url?: string; hash_source?: string }
export interface ToolDetail extends ToolSummary { description?: string; homepage_url?: string; source?: string; as_of?: string; release_url?: string; license_spdx?: string | null; license_source_url?: string; collection_notes?: string[]; discussion_tag?: string; discussion_count: number; artifacts?: ToolArtifact[]; releases?: { version: string; date?: string; url?: string; items?: { kind: string; text: string }[] }[] }
export interface NewsComment { id: number; reply_to?: number | null; body: string; created_at: string; author: { uid: string; nickname: string; avatar_url?: string | null }; liked: boolean; like_count: number; mine: boolean }
export type ReadmeBlock =
  | { t: 'h'; l: number; x: string } | { t: 'p'; x: string } | { t: 'quote'; x: string } | { t: 'hr' }
  | { t: 'li'; x: string; d: number; n: number | null } | { t: 'code'; x: string; lang: string } | { t: 'table'; rows: string[][] }
export interface RepoDetail {
  full_name: string; owner: string; repo: string; url: string; description: string; description_zh: string
  language: string; language_color: string; star_total: number; star_gain: number | null
  rank: number; is_latest: boolean; issue_key: string; week_end?: string; as_of?: string | null; metric?: string; source?: string
  readme: { status: 'ok' | 'none' | 'unavailable'; blocks: ReadmeBlock[]; zh_blocks: ReadmeBlock[] | null; zh_truncated: boolean; fetched_at?: string | null }
}
import type { Board as ModelBoard, Usage as UsageBoard } from '@/lib/modelRankings'
export type { Board as ModelBoard, Usage as UsageBoard } from '@/lib/modelRankings'
const get = <T>(url: string, auth = false) => request<T>('GET', url, undefined, auth)
export const discoverApi = {
  home: () => get<HomeBundle>(path.home),
  daily: (page = 1) => get<Page<NewsItem>>(`${path.daily}?page=${page}`),
  article: (key: string) => get<NewsItem>(path.article(key)),
  repo: (owner: string, repo: string) => get<RepoDetail>(path.repo(owner, repo)),
  repoComments: (owner: string, repo: string, page = 1) => get<Page<NewsComment>>(`${path.repoComments(owner, repo)}?page=${page}`, true),
  repoComment: (owner: string, repo: string, body: string, reply_to?: number) => request<{ id: number }>('POST', path.repoComments(owner, repo), { body, reply_to }),
  likeRepoComment: (owner: string, repo: string, id: number, on: boolean) => request(on ? 'PUT' : 'DELETE', `${path.repoComments(owner, repo)}/${id}/like`),
  deleteRepoComment: (owner: string, repo: string, id: number) => request('DELETE', `${path.repoComments(owner, repo)}/${id}`),
  repos: (issue = '') => get<RepoIssue>(`${path.repos}${issue ? `?issue=${encodeURIComponent(issue)}` : ''}`),
  issues: () => get<{ items: RepoIssueRef[] }>(path.issues),
  tracker: () => get<Tracker>(path.tracker),
  nav: () => get<{ groups: Group<NavItem>[] }>(path.nav),
  docs: () => get<{ groups: Group<DocItem>[] }>(path.docs),
  tools: () => get<{ items: ToolSummary[] }>(path.tools),
  helpStats: () => get<{ open:number; solved_this_week:number; bounty_points:number }>('/help/stats'),
  tool: (key: string) => get<ToolDetail>(path.tool(key)),
  models: () => get<ModelBoard>(path.models),
  usage: () => get<UsageBoard>(path.usage),
  comments: (key: string, page = 1) => get<Page<NewsComment>>(`${path.comments(key)}?page=${page}`, true),
  comment: (key: string, body: string, reply_to?: number) => request<{ id: number }>('POST', path.comments(key), { body, reply_to }),
  likeComment: (key: string, id: number, on: boolean) => request(on ? 'PUT' : 'DELETE', `${path.comments(key)}/${id}/like`),
  deleteComment: (key: string, id: number) => request('DELETE', `${path.comments(key)}/${id}`),
}
export function licenseLabel(raw?: string) { return raw === 'open' ? '开源' : raw === 'closed' ? '闭源' : '许可证待确认' }
