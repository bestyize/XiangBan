import { request } from './forum'
import { request as adminRequest } from './admin'
import type { PackItem } from './content'
import type { CommunityScheme } from './schemes'
import type { Page } from './types'
export type ContentKind = 'pack' | 'scheme'
export type ContentBookmark = { kind: 'pack'; id: number; created_at: string; content: PackItem } | { kind: 'scheme'; id: number; created_at: string; content: CommunityScheme }
export interface WorkDiscussion { topic_id: number | null; like_count: number; reply_count: number; liked: boolean; can_reply: boolean; locked: boolean }
export interface ContentReport {
  id: number; kind: ContentKind; target_id: number; reason: string; detail: string
  status: 'open' | 'resolved' | 'dismissed'; revision: number; created_at: string
  handled_at: string | null; handled_by: string | null; result: string
  reporter: { uid: string; nickname: string | null }
  target: { exists: boolean; title: string | null; status: string | null }
}
const path = (kind: ContentKind, id: number) => `/content-social/${kind}/${id}`
export const contentSocialApi = {
  discussion: (kind: ContentKind, id: number) => request<WorkDiscussion>('GET', `${path(kind, id)}/discussion`),
  ensureDiscussion: (kind: ContentKind, id: number) => request<WorkDiscussion>('POST', `${path(kind, id)}/discussion`, {}),
  state: (kind: ContentKind, id: number) => request<{ bookmarked: boolean }>('GET', path(kind, id)),
  bookmark: (kind: ContentKind, id: number, on: boolean) => request<{ bookmarked: boolean }>(on ? 'PUT' : 'DELETE', `${path(kind, id)}/bookmark`),
  report: (kind: ContentKind, id: number, reason: string, detail: string) => request<{ ok: boolean }>('POST', `${path(kind, id)}/report`, { reason, detail }),
  bookmarks: (page = 1, kind?: ContentKind) => request<Page<ContentBookmark>>('GET', `/me/content-bookmarks?page=${page}${kind ? `&kind=${kind}` : ''}`),
}
export const contentReportsApi = {
  list: (status: string, page: number) => adminRequest<Page<ContentReport>>('GET', `/content-social/reports?status=${status}&page=${page}`),
  handle: (row: ContentReport, action: 'dismiss' | 'remove', note: string) => adminRequest<{ ok: boolean; target_removed: boolean; reports_handled: number }>('POST', `/content-social/reports/${row.id}/handle`, { action, note, expected_revision: row.revision }),
}
export const CONTENT_REPORT_REASONS = [
  ['spam', '垃圾广告或私发推广'], ['account_trade', '分享或交易 API Key、账号'],
  ['abuse', '人身攻击或开盒'], ['politics', '引战或时政'], ['copyright', '侵犯版权'], ['illegal', '违法违规'], ['other', '其他'],
] as const
