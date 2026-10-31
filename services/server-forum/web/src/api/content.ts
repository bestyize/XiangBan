// 社区内容层接口：内容包、方案、项目、周刊、分享信息。
import { request } from './forum'
import type { Page } from './types'

export interface PackAuthor {
  uid: string
  nickname: string | null
}

export interface PackItem {
  id: number
  kind: string
  slug: string
  title: string
  summary: string
  tags: string[]
  author: PackAuthor | null
  /** 0 官方，1 已审核，2 用户未审 */
  trust_level: number
  status: string
  version: number
  cover_url: string | null
  apply_count: number
  topic_id: number | null
  one_click: boolean
  pure_data: boolean
  created_at: string
  updated_at: string
}

export interface PackVersion {
  version: number
  label: string
  sha256: string
  size: number
  manifest: Record<string, unknown>
  changelog: string
  created_at: string
}

export interface PackDetail extends PackItem {
  versions: PackVersion[]
}

export interface PackListParams {
  kind?: string
  q?: string
  tag?: string
  author?: string
  sort?: 'new' | 'hot'
  page?: number
  page_size?: number
}

export interface DownloadInfo {
  pack_id: number
  kind: string
  version: number
  label: string
  sha256: string
  size: number
  url: string
  expires_at: number
  mirrors: { name: string; url: string }[]
  trust_level: number
  one_click: boolean
}

export type SchemeItem = import('./schemes').CommunityScheme

export interface ProjectLink {
  title: string
  url: string
}

export interface ProjectItem {
  slug: string
  name: string
  summary: string
  links: ProjectLink[]
  updated_at: string
}

export interface ProjectDetail extends ProjectItem {
  body_html: string
}

export interface WeeklyIssueItem {
  week: string
  title: string
  summary: string
  published_at: string | null
}

export type WeeklyEntry =
  | (PackItem & { type: 'pack'; note: string })
  | { type: 'topic'; id: number; title: string; summary: string; note: string }
  | { type: 'text'; title: string | null; note: string }

export interface WeeklyDetail extends WeeklyIssueItem {
  items: WeeklyEntry[]
}

function qs(params: Record<string, string | number | undefined>) {
  const sp = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === '') continue
    sp.set(k, String(v))
  }
  const s = sp.toString()
  return s ? `?${s}` : ''
}

export interface UploadTicket {
  ticket: string
  upload_url: string
  file_key: string
  expires_at: number
}

export interface MyPack extends PackItem {
  in_review: boolean
}

export interface PublishInput {
  kind: string
  title: string
  summary: string
  tags: string[]
  file_key: string
  cover_key?: string
  rights_ack: boolean
}

export const contentApi = {
  uploadTicket: (kind: string, size: number, sha256: string) =>
    request<UploadTicket>('POST', '/packs/upload-ticket', { kind, size, sha256 }),
  publish: (input: PublishInput) => request<{ id: number; slug: string }>('POST', '/packs', input),
  addVersion: (id: number, file_key: string, changelog: string) =>
    request<{ id: number; version: number }>('POST', `/packs/${id}/versions`, { file_key, changelog }),
  removeOwn: (id: number) => request<{ id: number }>('POST', `/packs/${id}/remove`),
  myPacks: () => request<{ items: MyPack[] }>('GET', '/me/packs'),
  packs: (p: PackListParams) => request<Page<PackItem>>('GET', `/packs${qs({ ...p })}`, undefined, false),
  pack: (id: number) => request<PackDetail>('GET', `/packs/${id}`),
  download: (id: number, version?: number) => request<DownloadInfo>('GET', `/packs/${id}/download${qs({ version })}`, undefined, false),
  schemes: (page = 1) => request<Page<SchemeItem>>('GET', `/schemes${qs({ page })}`, undefined, false),
  projects: () => request<{ items: ProjectItem[] }>('GET', '/projects', undefined, false),
  project: (slug: string) => request<ProjectDetail>('GET', `/projects/${encodeURIComponent(slug)}`, undefined, false),
  weeklyList: (page = 1) => request<Page<WeeklyIssueItem>>('GET', `/weekly${qs({ page })}`, undefined, false),
  weekly: (week: string) => request<WeeklyDetail>('GET', `/weekly/${encodeURIComponent(week)}`, undefined, false),
}
