import { ApiError } from '@/api'
import type {
  AdminUser,
  CacheOverview,
  CacheSettings,
  ProviderConfig,
  ProxyInfo,
  ProxyProbe,
  ProxyStrategy,
} from './types'

let onUnauthorized: (() => void) | null = null

/** Called when a signed-in session turns out to have expired, so the app can return to the login page. */
export function setUnauthorizedHandler(handler: (() => void) | null) {
  onUnauthorized = handler
}

async function request<T>(path: string, method = 'GET', body?: unknown, signal?: AbortSignal): Promise<T> {
  const res = await fetch(`/api/v1/admin${path}`, {
    method,
    signal,
    credentials: 'same-origin',
    headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  })

  if (!res.ok) {
    let message = `请求失败 (${res.status})`
    try {
      const payload = await res.json()
      message = payload.error ?? message
    } catch {
      // the error body was not JSON; keep the generic message
    }
    const isSessionCheck = path.startsWith('/auth/login') || path.startsWith('/auth/me')
    if (res.status === 401 && !isSessionCheck) onUnauthorized?.()
    throw new ApiError(res.status, message)
  }
  return (await res.json()) as T
}

/* ---- session and account ---- */

export const login = (username: string, password: string) =>
  request<AdminUser>('/auth/login', 'POST', { username, password })

export const logout = () => request<{ success: boolean }>('/auth/logout', 'POST')

export const me = () => request<AdminUser>('/auth/me')

export const updateAccount = (body: { current_password: string; new_username?: string; new_password?: string }) =>
  request<AdminUser>('/account', 'PUT', body)

/* ---- provider keys ---- */

export const listProviders = async () => (await request<{ providers: ProviderConfig[] }>('/providers')).providers

export const addKey = (provider: string, label: string, key: string) =>
  request<{ id: number }>(`/providers/${provider}/keys`, 'POST', { label, key })

export const updateKey = (provider: string, id: number, change: { label?: string; enabled?: boolean }) =>
  request<unknown>(`/providers/${provider}/keys/${id}`, 'PATCH', change)

export const deleteKey = (provider: string, id: number) =>
  request<unknown>(`/providers/${provider}/keys/${id}`, 'DELETE')

export const setProviderProxy = (provider: string, useProxy: boolean) =>
  request<unknown>(`/providers/${provider}/settings`, 'PUT', { use_proxy: useProxy })

/* ---- proxy pool ---- */

export const listProxies = () => request<{ strategy: ProxyStrategy; proxies: ProxyInfo[] }>('/proxies')

export const addProxy = (name: string, url: string) => request<{ id: number }>('/proxies', 'POST', { name, url })

export const updateProxy = (id: number, change: { name?: string; enabled?: boolean }) =>
  request<unknown>(`/proxies/${id}`, 'PATCH', change)

export const deleteProxy = (id: number) => request<unknown>(`/proxies/${id}`, 'DELETE')

export const testProxy = async (id: number) =>
  (await request<{ results: ProxyProbe[] }>(`/proxies/${id}/test`, 'POST')).results

export const setProxyStrategy = (strategy: ProxyStrategy) =>
  request<unknown>('/proxies/settings', 'PUT', { strategy })

/* ---- search cache ---- */

export const getCache = () => request<CacheOverview>('/cache')

export const saveCacheSettings = (settings: CacheSettings) =>
  request<CacheOverview>('/cache/settings', 'PUT', settings)

export const clearCache = () => request<{ cleared: number }>('/cache', 'DELETE')

/* ---- collector workbench ---- */

import type { FeedParams, SearchResult, WallpaperItem } from '@/types'

export function collectorSearch(
  params: FeedParams,
  page: number,
  perPage: number,
  signal?: AbortSignal,
): Promise<SearchResult> {
  const qs = new URLSearchParams({ page: String(page), per_page: String(perPage) })
  if (params.q) qs.set('q', params.q)
  if (params.category) qs.set('category', params.category)
  if (params.media) qs.set('media_type', params.media)
  if (params.orientation && params.orientation !== 'all') qs.set('orientation', params.orientation)
  if (params.provider && params.provider !== 'all') qs.set('provider', params.provider)
  return request<SearchResult>(`/collector/search?${qs}`, 'GET', undefined, signal)
}

export function collectorImport(
  item: WallpaperItem,
  status = 'published',
  category?: string,
): Promise<{ success: boolean; message: string; item: WallpaperItem }> {
  return request('/collector/import', 'POST', { item, status, category })
}

export function collectorCheckStatus(ids: string[]): Promise<{ collected_ids: string[] }> {
  return request('/collector/check-status', 'POST', { ids })
}

/* ---- wallpaper management ---- */

export interface AdminWallpaperListResult {
  items: WallpaperItem[]
  total: number
  page: number
  per_page: number
}

export function listAdminWallpapers(params: {
  status?: string
  category?: string
  media_type?: string
  q?: string
  page: number
  per_page: number
}): Promise<AdminWallpaperListResult> {
  const qs = new URLSearchParams({
    page: String(params.page),
    per_page: String(params.per_page),
  })
  if (params.status && params.status !== 'all') qs.set('status', params.status)
  if (params.category && params.category !== 'all') qs.set('category', params.category)
  if (params.media_type && params.media_type !== 'all') qs.set('media_type', params.media_type)
  if (params.q) qs.set('q', params.q)
  return request<AdminWallpaperListResult>(`/wallpapers?${qs}`, 'GET')
}

export function updateWallpaperStatus(id: string, status: string): Promise<{ success: boolean }> {
  return request<{ success: boolean }>(`/wallpapers/${encodeURIComponent(id)}/status`, 'PUT', { status })
}

export function updateWallpaperMeta(
  id: string,
  data: { title: string; category?: string; tags: string[]; status: string },
): Promise<{ success: boolean }> {
  return request<{ success: boolean }>(`/wallpapers/${encodeURIComponent(id)}`, 'PUT', data)
}

export function deleteWallpaper(id: string): Promise<{ success: boolean }> {
  return request<{ success: boolean }>(`/wallpapers/${encodeURIComponent(id)}`, 'DELETE')
}

/* ---- custom wallpaper upload ---- */

export async function uploadFile(file: File): Promise<{ url: string; filename: string }> {
  const form = new FormData()
  form.append('file', file)
  const res = await fetch('/api/v1/admin/wallpapers/upload', {
    method: 'POST',
    credentials: 'same-origin',
    body: form,
  })
  if (!res.ok) {
    let msg = '上传失败'
    try {
      const j = await res.json()
      msg = j.error || msg
    } catch {}
    throw new ApiError(res.status, msg)
  }
  const data = await res.json()
  return data.file
}

export function createCustomWallpaper(data: {
  title: string
  description?: string
  media_type: string
  preview_url: string
  regular_url: string
  video_url?: string
  category?: string
  tags: string[]
  orientation?: string
  width?: number
  height?: number
  status: string
}): Promise<{ success: boolean; message: string; item: WallpaperItem }> {
  return request('/wallpapers/custom', 'POST', data)
}
