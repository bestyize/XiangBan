export type MediaType = 'image' | 'video' | 'livephoto'
export type Orientation = 'all' | 'landscape' | 'portrait' | 'squarish'

export interface Resolution {
  width: number
  height: number
  aspect_ratio: string
}

export interface VideoQuality {
  quality: string
  url: string
  width: number
  height: number
  size_bytes?: number
}

export interface VideoMeta {
  duration_secs: number
  preview_url: string
  qualities: VideoQuality[]
}

export interface Author {
  name: string
  username: string
  avatar_url?: string
  profile_url?: string
}

export interface DownloadUrls {
  original: string
  large?: string
  medium?: string
  small?: string
  thumb: string
}

export interface WallpaperItem {
  id: string
  provider: string
  source_id: string
  media_type: MediaType
  title: string
  description?: string
  tags: string[]
  resolution: Resolution
  orientation: Orientation
  dominant_color?: string
  preview_url: string
  regular_url: string
  downloads: DownloadUrls
  video?: VideoMeta
  author: Author
  stats: { views: number; likes: number; downloads: number }
  source_page_url: string
  license: string
  status?: string
  source_type?: string
  category?: string
  local_path?: string
  collected_at?: string
}

export interface SearchResult {
  items: WallpaperItem[]
  total: number
  page: number
  per_page: number
  has_more: boolean
  providers: string[]
  execution_time_ms: number
}

export interface Category {
  id: string
  name: string
  keyword: string
}

export interface ProviderInfo {
  id: string
  name: string
  description: string
  is_configured: boolean
  supported_media_types: MediaType[]
  official_url: string
  status: 'ready' | 'missing_api_key' | 'error'
}

export interface SyncReport {
  total_collected: number
  keywords_processed: string[]
  providers_used: string[]
  timestamp: string
}

export interface FeedParams {
  q?: string
  category?: string
  media?: 'image' | 'video'
  orientation?: Orientation
  provider?: string
}
