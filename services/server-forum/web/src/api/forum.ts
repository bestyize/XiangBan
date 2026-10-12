import type {
  Attachment,
  Category,
  Page,
  PostItem,
  Profile,
  SessionInfo,
  SiteConfig,
  TopicDetail,
  TopicInput,
  TopicItem,
  UserReply,
} from './types'

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message)
  }
}

let configPromise: Promise<SiteConfig> | null = null

export function siteConfig(): Promise<SiteConfig> {
  configPromise ??= fetch('/api/config')
    .then(r => (r.ok ? r.json() : Promise.reject(new Error('config'))))
    .then((c: SiteConfig) => ({ ...c, account_base: c.account_base.replace(/\/+$/, '') }))
    .catch(() => {
      configPromise = null
      return { site_name: '超级小伴', account_base: 'https://account.xiangban.xyz', client_id: 'forum-web' }
    })
  return configPromise
}

/** 由会话 store 注入：返回当前可用的访问令牌。 */
let tokenProvider: () => Promise<string | null> = async () => null
let onUnauthorized: () => void = () => {}

export function configureAuth(provider: () => Promise<string | null>, unauthorized: () => void) {
  tokenProvider = provider
  onUnauthorized = unauthorized
}

type Query = Record<string, string | number | boolean | null | undefined>

function qs(params?: Query) {
  if (!params) return ''
  const sp = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === null || v === '' || v === false) continue
    sp.set(k, String(v))
  }
  const s = sp.toString()
  return s ? `?${s}` : ''
}

async function request<T>(method: string, path: string, body?: unknown, auth = true): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  if (auth) {
    const token = await tokenProvider()
    if (token) headers.Authorization = `Bearer ${token}`
  }
  let res: Response
  try {
    res = await fetch(`/api${path}`, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) })
  } catch {
    throw new ApiError(0, '网络连接失败，请稍后再试')
  }
  const data = await res.json().catch(() => null)
  if (!res.ok) {
    if (res.status === 401 && headers.Authorization) onUnauthorized()
    throw new ApiError(res.status, data?.message ?? `请求失败（${res.status}）`)
  }
  return data as T
}

export interface ListParams extends Query {
  category?: string
  tag?: string
  q?: string
  sort?: 'latest' | 'new' | 'hot' | 'expiring'
  home?: boolean
  kind?: 'deal' | 'normal'
  /** 默认不返回已到期的羊毛 */
  include_expired?: boolean
  featured?: boolean
  page?: number
  page_size?: number
}

export const api = {
  categories: () => request<Category[]>('GET', '/categories', undefined, false),
  tags: () => request<{ name: string; count: number }[]>('GET', '/tags', undefined, false),

  session: () => request<SessionInfo>('POST', '/session'),
  bookmarks: (page = 1) => request<Page<TopicItem>>('GET', `/me/bookmarks${qs({ page })}`),

  topics: (p: ListParams) => request<Page<TopicItem>>('GET', `/topics${qs(p)}`),
  deals: (p: ListParams) => request<Page<TopicItem>>('GET', `/deals${qs(p)}`),
  search: (p: ListParams) => request<Page<TopicItem>>('GET', `/search${qs(p)}`),
  topic: (id: number) => request<TopicDetail>('GET', `/topics/${id}`),
  createTopic: (input: TopicInput) => request<{ id: number }>('POST', '/topics', input),
  updateTopic: (id: number, input: TopicInput) => request<{ id: number }>('PATCH', `/topics/${id}`, input),
  deleteTopic: (id: number) => request<unknown>('DELETE', `/topics/${id}`),
  pinTopic: (id: number, on: boolean) => request<{ pinned: boolean }>(on ? 'PUT' : 'DELETE', `/topics/${id}/pin`),
  preview: (body: string) => request<{ html: string }>('POST', '/markdown/preview', { body }),

  posts: (topicId: number, page = 1) => request<Page<PostItem>>('GET', `/topics/${topicId}/posts${qs({ page })}`),
  reply: (topicId: number, body: string, replyTo?: number | null) =>
    request<{ id: number; floor: number }>('POST', `/topics/${topicId}/posts`, { body, reply_to: replyTo ?? null }),
  updatePost: (id: number, body: string) => request<{ body_html: string }>('PATCH', `/posts/${id}`, { body }),
  deletePost: (id: number) => request<unknown>('DELETE', `/posts/${id}`),
  postSource: (id: number) => request<{ body: string }>('GET', `/posts/${id}/source`),

  likeTopic: (id: number, on: boolean) =>
    request<{ liked: boolean; like_count: number }>(on ? 'PUT' : 'DELETE', `/topics/${id}/like`),
  likePost: (id: number, on: boolean) =>
    request<{ liked: boolean; like_count: number }>(on ? 'PUT' : 'DELETE', `/posts/${id}/like`),
  bookmark: (id: number, on: boolean) => request<{ bookmarked: boolean }>(on ? 'PUT' : 'DELETE', `/topics/${id}/bookmark`),
  report: (target_type: 'topic' | 'post', target_id: number, reason: string, detail: string) =>
    request<unknown>('POST', '/reports', { target_type, target_id, reason, detail }),

  profile: (uid: string) => request<Profile>('GET', `/users/${uid}`, undefined, false),
  userTopics: (uid: string, page = 1) => request<Page<TopicItem>>('GET', `/users/${uid}/topics${qs({ page })}`, undefined, false),
  userPosts: (uid: string, page = 1) =>
    request<{ items: UserReply[] }>('GET', `/users/${uid}/posts${qs({ page })}`, undefined, false),
}

async function uploadFile<T>(path: string, file: File, onProgress?: (ratio: number) => void): Promise<T> {
  const token = await tokenProvider()
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('POST', `/api${path}`)
    if (token) xhr.setRequestHeader('Authorization', `Bearer ${token}`)
    xhr.upload.onprogress = e => {
      if (e.lengthComputable) onProgress?.(e.loaded / e.total)
    }
    xhr.onerror = () => reject(new ApiError(0, '网络连接失败，请稍后再试'))
    xhr.onload = () => {
      let data: { message?: string } | null = null
      try {
        data = JSON.parse(xhr.responseText)
      } catch {
        data = null
      }
      if (xhr.status >= 200 && xhr.status < 300) return resolve(data as unknown as T)
      if (xhr.status === 401 && token) onUnauthorized()
      const fallback = xhr.status === 413 ? '文件太大，被服务器拒绝了' : `上传失败（${xhr.status}）`
      reject(new ApiError(xhr.status, data?.message ?? fallback))
    }
    const form = new FormData()
    form.append('file', file)
    xhr.send(form)
  })
}

/** 上传一个附件，进度用 0-1 回调。返回的 id 在发帖或编辑时通过 attachment_ids 挂到帖子上。 */
export const uploadAttachment = (file: File, onProgress?: (ratio: number) => void) =>
  uploadFile<Attachment>('/attachments', file, onProgress)

/** 上传一张正文插图，返回可直接写进 Markdown 的站内地址。 */
export const uploadImage = (file: File) => uploadFile<{ id: string; url: string; size: number }>('/images', file)

export const attachmentUrl = (id: string) => `/api/attachments/${id}`

export const deleteAttachment = (id: string) => request<unknown>('DELETE', `/attachments/${id}`)

/** 需要登录才能读的帖子，下载链接带不上令牌，先带令牌取回再交给浏览器保存。 */
export async function downloadAttachment(att: Attachment): Promise<void> {
  const token = await tokenProvider()
  let res: Response
  try {
    res = await fetch(attachmentUrl(att.id), { headers: token ? { Authorization: `Bearer ${token}` } : {} })
  } catch {
    throw new ApiError(0, '网络连接失败，请稍后再试')
  }
  if (!res.ok) {
    const data = await res.json().catch(() => null)
    throw new ApiError(res.status, data?.message ?? `下载失败（${res.status}）`)
  }
  const url = URL.createObjectURL(await res.blob())
  const a = document.createElement('a')
  a.href = url
  a.download = att.name
  document.body.appendChild(a)
  a.click()
  a.remove()
  setTimeout(() => URL.revokeObjectURL(url), 10_000)
}
