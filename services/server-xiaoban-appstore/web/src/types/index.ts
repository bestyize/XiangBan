export interface Category {
  id: number;
  slug: string;
  name: string;
  icon: string;
  description: string;
  sort_order: number;
  app_count?: number;
}

export interface AppItem {
  id: number;
  slug: string;
  name: string;
  icon_url: string;
  category_id: number;
  category_name?: string;
  category_slug?: string;
  summary: string;
  comment: string;
  referrer: string;
  referrer_avatar?: string;
  pricing_type: 'free' | 'freemium' | 'paid' | 'open_source';
  platforms: string[]; // ["mac", "windows", "linux", "web", "android", "ios"]
  badges: string[]; // ["safe", "native", "ai", "clean", "open_source", "local_first", "recommended"]
  status: 'published' | 'unlisted' | 'draft';
  is_featured: boolean;
  sort_order: number;
  download_url: string;
  download_alt_url?: string;
  official_url?: string;
  appstore_url?: string;
  github_url?: string;
  github_stars?: number;
  brew_cask?: string;
  winget_id?: string;
  mac_download_url?: string;
  win_download_url?: string;
  linux_download_url?: string;
  file_size?: string;
  release_notes?: string;
  auto_sync_source?: string;
  last_synced_at?: string;
  pros: string[];
  cons: string[];
  alternatives: string[];
  version?: string;
  downloads_count: number;
  created_at: string;
  updated_at: string;
}

export interface AdminStats {
  total_apps: number;
  published_apps: number;
  unlisted_apps: number;
  total_categories: number;
  total_downloads: number;
}
