// 管理后台接口：独立的管理员令牌（与前台用户登录无关），所有请求走 /api/admin。
import { reactive } from 'vue'

const TOKEN_KEY = 'chat.xiangban.admin'

/** 管理员登录状态。令牌为空表示未登录；401 时会被自动清空，界面随之回到登录页。 */
export const adminAuth = reactive({
  token: readToken(),
  username: '',
})

function readToken(): string {
  try {
    return localStorage.getItem(TOKEN_KEY) || ''
  } catch {
    return ''
  }
}

export function setAdminToken(token: string) {
  adminAuth.token = token
  try {
    localStorage.setItem(TOKEN_KEY, token)
  } catch {
    // 隐私模式下写入失败时只在本次页面内有效
  }
}

export function clearAdminToken() {
  adminAuth.token = ''
  adminAuth.username = ''
  try {
    localStorage.removeItem(TOKEN_KEY)
  } catch {
    // 忽略
  }
}

export class AdminApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message)
  }
}

type Query = Record<string, string | number | boolean | null | undefined>

function qs(params?: Query) {
  if (!params) return ''
  const sp = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === null || v === '') continue
    sp.set(k, String(v))
  }
  const s = sp.toString()
  return s ? `?${s}` : ''
}

export async function request<T>(method: string, path: string, body?: unknown, auth = true): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  if (auth && adminAuth.token) headers.Authorization = `Bearer ${adminAuth.token}`

  let res: Response
  try {
    res = await fetch(`/api/admin${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  } catch {
    throw new AdminApiError(0, 'network', '网络连接失败，请检查网络后重试')
  }

  let data: unknown = null
  try {
    data = await res.json()
  } catch {
    // 响应不是 JSON 时按状态码给通用提示
  }
  if (!res.ok) {
    const err = (data ?? {}) as { code?: string; message?: string }
    // 登录接口自己的 401 是密码错误，不当作令牌过期
    if (res.status === 401 && auth) clearAdminToken()
    throw new AdminApiError(res.status, err.code || 'error', err.message || `请求失败（${res.status}）`)
  }
  return data as T
}

// ───────────────────────── 类型 ─────────────────────────

export interface AdminPage<T> {
  items: T[]
  page: number
  page_size: number
  total: number
}

export interface LoginResult {
  token: string
  expires_in: number
  admin: { id: string; username: string }
}

export interface AdminStats {
  users_total: number
  users_new_24h: number
  users_new_7d: number
  topics_total: number
  topics_24h: number
  posts_total: number
  posts_24h: number
  reports_open: number
  dau: number
}

export type ReportStatus = 'open' | 'resolved' | 'dismissed'
export type ContentStatus = 'normal' | 'hidden' | 'deleted'
export type UserStatus = 'active' | 'muted' | 'banned'

export interface ReportTarget {
  type: 'topic' | 'post'
  id: number
  exists: boolean
  title?: string | null
  topic_id?: number | null
  topic_title?: string | null
  floor?: number | null
  excerpt?: string | null
  status: string | null
}

export interface AdminReport {
  id: number
  target_type: 'topic' | 'post'
  target_id: number
  target: ReportTarget
  reporter: { uid: string; nickname: string | null }
  reason: string
  detail: string
  status: ReportStatus
  handled_by: string | null
  handled_at: string | null
  result: string | null
  created_at: string
  report_count: number
}

export type PinScope = 'none' | 'site' | 'parent' | 'category'

export interface AdminTopic {
  id: number
  title: string
  status: ContentStatus
  category_id: number
  category_name: string
  author_uid: string
  author_nickname: string
  pinned: boolean
  pin_scope: PinScope
  featured: boolean
  locked: boolean
  reply_count: number
  created_at: string
  /** 阅读权限：0 所有人，1 成员以上，2 老用户以上，3 守护者 */
  min_read_level: number
  open_reports: number
}

export interface AdminUser {
  uid: string
  nickname: string
  trust_level: number
  points: number
  status: UserStatus
  muted_until: string | null
  read_count: number
  visit_days: number
  created_at: string
  last_seen_at: string
  topic_count: number
  post_count: number
}

export interface AdminCategory {
  id: number
  parent_id: number | null
  slug: string
  name: string
  description: string
  kind: 'normal'
  sort_order: number
  min_post_level: number
  on_home: boolean
  award_points: boolean
  admin_only: boolean
  preset_tags: string[]
  topic_count: number
}

export interface CategoryInput {
  parent_id?: number | null
  slug?: string
  kind?: 'normal'
  name: string
  description: string
  sort_order: number
  min_post_level: number
  on_home: boolean
  award_points: boolean
  admin_only: boolean
  preset_tags: string[]
}

export interface AuditRow {
  id: number
  admin: string
  action: string
  target_type: string
  target_id: string
  detail: string
  created_at: string
}

export type TopicAction = 'hide' | 'restore' | 'delete' | 'pin' | 'unpin' | 'feature' | 'unfeature' | 'lock' | 'unlock'

export interface TopicState {
  id: number
  status: ContentStatus
  pinned: boolean
  pin_scope: PinScope
  featured: boolean
  locked: boolean
}

// ───────────────────────── 接口 ─────────────────────────

export const adminApi = {
  login: (username: string, password: string) =>
    request<LoginResult>('POST', '/login', { username, password }, false),
  me: () => request<{ id: string; username: string }>('GET', '/me'),
  changePassword: (current_password: string, new_password: string) =>
    request<{ ok: boolean; token: string; expires_in: number }>('POST', '/password', { current_password, new_password }),
  stats: () => request<AdminStats>('GET', '/stats'),

  reports: (params: { status?: string; page?: number; page_size?: number }) =>
    request<AdminPage<AdminReport>>('GET', `/reports${qs(params)}`),
  handleReport: (id: number, body: { action: 'resolve' | 'dismiss'; result?: string; hide_target?: boolean }) =>
    request<{ ok: boolean; status: string; reports_handled: number; target_hidden: boolean }>(
      'POST',
      `/reports/${id}`,
      body,
    ),

  topics: (params: { q?: string; status?: string; category?: string; page?: number; page_size?: number }) =>
    request<AdminPage<AdminTopic>>('GET', `/topics${qs(params)}`),
  moderateTopic: (id: number, action: TopicAction, pin_scope?: PinScope) =>
    request<TopicState>('POST', `/topics/${id}/moderate`, { action, ...(pin_scope ? { pin_scope } : {}) }),
  moveTopic: (id: number, category_id: number) =>
    request<{ id: number; category_id: number }>('POST', `/topics/${id}/move`, { category_id }),
  moderatePost: (id: number, action: 'hide' | 'restore') =>
    request<{ id: number; topic_id: number; status: string }>('POST', `/posts/${id}/moderate`, { action }),

  users: (params: { q?: string; status?: string; page?: number; page_size?: number }) =>
    request<AdminPage<AdminUser>>('GET', `/users${qs(params)}`),
  setTrust: (uid: string, level: number) =>
    request<{ uid: string; trust_level: number }>('POST', `/users/${uid}/trust`, { level }),
  setUserStatus: (uid: string, status: UserStatus, muted_hours?: number) =>
    request<{ uid: string; status: UserStatus; muted_until: string | null }>('POST', `/users/${uid}/status`, {
      status,
      ...(muted_hours ? { muted_hours } : {}),
    }),

  categories: () => request<AdminCategory[]>('GET', '/categories'),
  createCategory: (body: CategoryInput) => request<AdminCategory>('POST', '/categories', body),
  updateCategory: (id: number, body: Partial<CategoryInput>) =>
    request<AdminCategory>('PATCH', `/categories/${id}`, body),


  audit: (limit = 50) => request<AuditRow[]>('GET', `/audit${qs({ limit })}`),

  packs: (params: { status?: string; kind?: string; page?: number; page_size?: number }) =>
    request<AdminPage<AdminPack>>('GET', `/packs${qs(params)}`),
  createPack: (body: OfficialPackInput) => request<{ id: number; slug: string }>('POST', '/packs', body),
  reviewPack: (id: number, action: PackReviewAction, note = '') =>
    request<{ id: number; trust_level: number; status: string }>('POST', `/packs/${id}/review`, { action, note }),
  metrics: (days = 14) => request<Metrics>('GET', `/metrics${qs({ days })}`),
  moderationQueue: () => request<{ items: ModerationRow[] }>('GET', '/moderation'),
  resolveModeration: (id: number, action: 'approve' | 'reject', note = '') =>
    request<{ id: number; status: string }>('POST', `/moderation/${id}/resolve`, { action, note }),
  words: () => request<{ items: SensitiveWord[] }>('GET', '/words'),
  addWords: (words: string, action: 'block' | 'review') => request<{ added: number; submitted: number }>('POST', '/words', { words, action }),
  deleteWord: (id: number) => request<{ ok: boolean }>('DELETE', `/words/${id}`),
  projects: () => request<{ items: (ProjectInput & { slug: string })[] }>('GET', '/projects'),
  weeklyIssues: () => request<{ items: (WeeklyInput & { week: string })[] }>('GET', '/weekly'),
  putProject: (slug: string, body: ProjectInput) => request<{ slug: string }>('PUT', `/projects/${encodeURIComponent(slug)}`, body),
  putWeekly: (week: string, body: WeeklyInput) => request<{ week: string }>('PUT', `/weekly/${encodeURIComponent(week)}`, body),
  discoverItems: (kind: string) => request<{ items: DiscoverRow[] }>('GET', `/discover/${encodeURIComponent(kind)}`),
  putDiscover: (kind: string, key: string, body: DiscoverInput) =>
    request<{ kind: string; key: string }>('PUT', `/discover/${encodeURIComponent(kind)}/${encodeURIComponent(key)}`, body),
  deleteDiscover: (kind: string, key: string) =>
    request<{ deleted: boolean }>('DELETE', `/discover/${encodeURIComponent(kind)}/${encodeURIComponent(key)}`),
  discoverDrafts: (params: { kind?: string; status?: string; page?: number }) =>
    request<AdminPage<DiscoverDraft>>('GET', `/discover-workflow/drafts${qs(params)}`),
  discoverDraft: (kind: string, key: string) =>
    request<{ draft: DiscoverDraft; live: { data: Record<string, unknown>; updated_at: string } | null }>('GET', `/discover-workflow/drafts/${encodeURIComponent(kind)}/${encodeURIComponent(key)}`),
  saveDiscoverDraft: (kind: string, key: string, body: DiscoverDraftInput) =>
    request<DiscoverDraft>('PUT', `/discover-workflow/drafts/${encodeURIComponent(kind)}/${encodeURIComponent(key)}`, body),
  previewDiscover: (kind: string, body: { data: Record<string, unknown>; sample: boolean }) =>
    request<{ kind: string; data: Record<string, unknown> }>('PUT', `/discover-workflow/preview/${encodeURIComponent(kind)}`, body),
  importDiscoverDrafts: (items: (DiscoverDraftInput & { kind: string; key: string })[]) =>
    request<{ items: DiscoverDraft[] }>('POST', '/discover-workflow/drafts/import', { items }),
  reviewDiscover: (kind: string, key: string, expected_revision: number, publish_after?: string) =>
    request<{ status: string }>('POST', `/discover-workflow/drafts/${encodeURIComponent(kind)}/${encodeURIComponent(key)}/review`, { expected_revision, ...(publish_after ? { publish_after } : {}) }),
  cancelDiscover: (kind: string, key: string, expected_revision: number) =>
    request<{ ok: boolean }>('POST', `/discover-workflow/drafts/${encodeURIComponent(kind)}/${encodeURIComponent(key)}/cancel`, { expected_revision }),
  discardDiscover: (kind: string, key: string, expected_revision: number) =>
    request<{ ok: boolean }>('POST', `/discover-workflow/drafts/${encodeURIComponent(kind)}/${encodeURIComponent(key)}/discard`, { expected_revision }),
  discoverTokens: () => request<{ items: DiscoverToken[] }>('GET', '/discover-workflow/tokens'),
  createDiscoverToken: (body: { name: string; kinds: string[]; days: number }) =>
    request<{ id: string; token: string; expires_at: string }>('POST', '/discover-workflow/tokens', body),
  revokeDiscoverToken: (id: string) => request<{ ok: boolean }>('DELETE', `/discover-workflow/tokens/${encodeURIComponent(id)}`),
}

export interface DiscoverInput {
  data: Record<string, unknown>
  sort_order: number
  status: 'draft' | 'published'
  published_at?: string
  sample: boolean
}
export interface DiscoverRow {
  key: string
  sort_order: number
  status: 'draft' | 'published'
  published_at: string
  data: Record<string, unknown> & { sample?: boolean }
}

export interface DiscoverDraftInput {
  data: Record<string, unknown>
  sort_order?: number
  published_at?: string
  sample?: boolean
  source_name?: string
  source_url?: string
  expected_revision: number
}

export interface DiscoverDraft extends Omit<DiscoverRow, 'status' | 'published_at'> {
  kind: string
  status: 'draft' | 'scheduled' | 'published'
  published_at: string | null
  source_name: string
  source_url: string
  revision: number
  base_updated_at: string | null
  publish_after: string | null
  reviewed_by: string | null
  reviewed_at: string | null
  last_error: string | null
  updated_by: string
  updated_at: string
}

export interface DiscoverToken {
  id: string
  name: string
  kinds: string[]
  expires_at: string
  revoked_at: string | null
}

export interface Metrics {
  daily_devices: { day: string; devices: number }[]
  events: { day: string; name: string; dim: string; count: number; devices: number }[]
  retention: { day: string; devices: number; returned: number }[]
}

export type PackReviewAction = 'approve' | 'reject' | 'remove' | 'restore'

export interface AdminPack {
  id: number
  kind: string
  slug: string
  title: string
  author: { uid: string; nickname: string | null } | null
  trust_level: number
  status: 'pending' | 'published' | 'rejected' | 'removed'
  version: number
  apply_count: number
  updated_at: string
}

export interface ModerationRow {
  id: number
  target_type: 'pack' | 'topic' | 'post' | 'scheme' | string
  target_id: number
  note: string
  created_at: string
  title: string | null
  kind: string | null
  author: string | null
  /** 回复和帖子所在的帖子编号，后台用它跳到帖子页 */
  topic_id: number | null
}

export interface SensitiveWord {
  id: number
  word: string
  action: 'block' | 'review'
  created_at: string
}

export interface OfficialPackInput {
  kind: string
  title: string
  summary: string
  tags: string[]
  manifest: Record<string, unknown>
  file_key: string
  sha256: string
  size: number
  cover_key?: string
  mirrors: { name: string; url: string }[]
  trust_level: 0 | 1
}

export interface ProjectInput {
  name: string
  summary: string
  body_md: string
  links: { title: string; url: string }[]
  status: 'draft' | 'published'
  sort_order: number
}

export interface WeeklyInput {
  title: string
  summary: string
  items: { type: 'pack' | 'topic' | 'text'; ref?: number; title?: string; note: string }[]
  status: 'draft' | 'published'
}
