import type {
  Category,
  FeedParams,
  ProviderInfo,
  SearchResult,
  SyncReport,
  WallpaperItem,
} from './types'

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message)
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api/v1${path}`, init)
  if (!res.ok) {
    let message = `请求失败 (${res.status})`
    try {
      const body = await res.json()
      message = body.error ?? body.message ?? message
    } catch {
      // error body was not JSON; keep the generic message
    }
    throw new ApiError(res.status, message)
  }
  return (await res.json()) as T
}

export function searchWallpapers(
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
  return request(`/wallpapers/search?${qs}`, { signal })
}

/** Item ids look like `pexels_v_123`: the provider is everything before the first underscore. */
export function getWallpaper(itemId: string): Promise<WallpaperItem> {
  const cut = itemId.indexOf('_')
  if (cut < 1) throw new ApiError(400, '无效的壁纸 ID')
  const provider = itemId.slice(0, cut)
  const sourceId = itemId.slice(cut + 1)
  return request(`/wallpapers/${encodeURIComponent(provider)}/${encodeURIComponent(sourceId)}`)
}

export async function listCategories(): Promise<Category[]> {
  const body = await request<{ categories: Category[] }>('/categories')
  return body.categories
}

export async function listProviders(): Promise<ProviderInfo[]> {
  const body = await request<{ providers: ProviderInfo[] }>('/providers')
  return body.providers
}

export async function runCollectorSync(): Promise<SyncReport> {
  const body = await request<{ report: SyncReport }>('/collector/sync', { method: 'POST' })
  return body.report
}
