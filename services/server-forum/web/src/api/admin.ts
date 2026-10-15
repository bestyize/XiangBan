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
  deals_active: number
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
  is_deal: boolean
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
  kind: 'deal' | 'normal'
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
  kind?: 'deal' | 'normal'
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
}
