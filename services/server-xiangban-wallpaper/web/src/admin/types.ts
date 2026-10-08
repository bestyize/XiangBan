export interface AdminUser {
  username: string
  is_default_password: boolean
}

export interface KeyInfo {
  id: number
  label: string
  /** First and last characters only; the full key is never sent to the browser. */
  preview: string
  enabled: boolean
  created_at: string
  requests: number
  failures: number
  last_status: number | null
  cooldown_secs: number | null
}

export type ProviderId = 'unsplash' | 'pixabay' | 'pexels'

export interface ProviderConfig {
  id: ProviderId
  name: string
  use_proxy: boolean
  keys: KeyInfo[]
}

export type ProxyStrategy = 'round_robin' | 'random'

export interface ProxyInfo {
  id: number
  name: string
  /** Address without username and password. */
  url: string
  has_auth: boolean
  enabled: boolean
  created_at: string
  requests: number
  failures: number
  last_error: string | null
  cooldown_secs: number | null
}

export interface ProxyProbe {
  target: string
  ok: boolean
  status: number | null
  latency_ms: number
  error: string | null
}

export interface CacheSettings {
  enabled: boolean
  ttl_secs: number
  max_entries: number
}

export interface CacheStats {
  available: boolean
  entries: number
  bytes: number
  hits: number
  misses: number
  oldest_age_secs: number | null
}

export interface CacheOverview {
  settings: CacheSettings
  stats: CacheStats
  hit_rate: number | null
}
