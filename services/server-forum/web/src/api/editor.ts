import { adminAuth, clearAdminToken, AdminApiError, request, type AdminCategory } from './admin'
export interface Author { uid: string; nickname: string; trust_level: number; status: string }
export interface Asset { path: string; data_base64: string }
export interface DraftInput { author_uid: string; external_id: string; title: string; category: string; body: string; tags: string[]; images: Asset[]; overwrite?: boolean; expected_updated_at?: string }
export interface Draft { id: string; author_uid: string; external_id: string; title: string; category_id: number; tags: string[]; topic_id: number | null; updated_at: string }
export interface Detail { draft: Draft; category: string; body: string; images: Asset[]; html: string }
export interface Token { id: string; name: string; author_uid: string; can_publish: boolean; expires_at: string; revoked_at: string | null }
async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`/api/editor${path}`, { method, headers: { Authorization: `Bearer ${adminAuth.token}`, ...(body !== undefined ? { 'Content-Type': 'application/json' } : {}) }, body: body === undefined ? undefined : JSON.stringify(body) })
  const data = await res.json().catch(() => null)
  if (!res.ok) { if (res.status === 401) clearAdminToken(); throw new AdminApiError(res.status, data?.code || 'error', data?.message || `请求失败（${res.status}）`) }
  return data as T
}
export const editorApi = {
  authors: () => request<{ items: Author[]; provision_enabled: boolean }>('GET', '/editor/authors'),
  createAuthor: (body: { request_id: string; username: string; password: string; nickname: string; avatar_url: string; trust_level: number }) => request<{ uid: string }>('POST', '/editor/authors', body),
  bindAuthor: (uid: string) => request<{ uid: string }>('POST', '/editor/authors/bind', { uid }),
  tokens: () => request<{ items: Token[] }>('GET', '/editor/tokens'),
  createToken: (body: { name: string; author_uid: string; can_publish: boolean; days: number }) => request<{ token: string }>('POST', '/editor/tokens', body),
  revoke: (id: string) => request('DELETE', `/editor/tokens/${id}`),
  context: () => call<{ categories: AdminCategory[] }>('GET', '/context'),
  drafts: (page = 1, published = false) => call<{ items: Draft[]; total: number; page_size: number }>('GET', `/drafts?page=${page}&published=${published}`),
  detail: (id: string) => call<Detail>('GET', `/drafts/${id}`),
  save: (body: DraftInput) => call<{ id: string }>('POST', '/drafts', body),
  remove: (id: string) => call('DELETE', `/drafts/${id}`),
  publish: (id: string) => call<{ id: number; url: string; already_published: boolean }>('POST', `/drafts/${id}/publish`),
}
