export interface PublicUser {
  uid: string
  nickname: string
  avatar_url: string | null
  trust_level: number
}

export interface ForumUser extends PublicUser {
  points: number
  status: 'active' | 'muted' | 'banned'
  muted_until: string | null
  read_count: number
  visit_days: number
  created_at: string
  last_seen_at: string
}

export interface SessionInfo {
  user: ForumUser
  limits: { daily_topics: number; daily_posts: number; tl1_days: number; tl1_reads: number }
}

export interface Category {
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
  topic_count: number
}

export interface Product {
  id: number
  slug: string
  name: string
  vendor: string
  website: string
  description: string
  deal_count: number
  active_deal_count: number
}

export type DealStatus = 'valid' | 'suspect' | 'expired'
export type DealType =
  | 'free_credit'
  | 'signup_bonus'
  | 'trial'
  | 'free_limited'
  | 'invite'
  | 'student'
  | 'discount'
  | 'public_site'
export type Region = 'any' | 'cn' | 'intl'

export interface DealInfo {
  product: { id: number; slug: string; name: string }
  deal_type: DealType
  value_text: string
  conditions: number
  region: Region
  link_url: string
  expires_at: string | null
  long_term: boolean
  deal_status: DealStatus
  valid_votes: number
  invalid_votes: number
  last_verified_at: string
}

export interface TopicItem {
  id: number
  title: string
  excerpt: string
  category: { id: number; slug: string; name: string }
  author: PublicUser
  tags: string[]
  status: string
  pinned: boolean
  featured: boolean
  locked: boolean
  reply_count: number
  like_count: number
  view_count: number
  created_at: string
  last_reply_at: string
  edited_at: string | null
  deal: DealInfo | null
}

export interface Viewer {
  liked: boolean
  bookmarked: boolean
  vote: -1 | 0 | 1
  can_edit: boolean
  can_reply: boolean
  can_vote: boolean
  can_moderate: boolean
}

export interface TopicDetail extends TopicItem {
  body_html: string
  body_md: string
  viewer: Viewer
}

export interface PostItem {
  id: number
  floor: number
  reply_to: number | null
  reply_to_floor: number | null
  author: PublicUser
  body_html: string
  status: string
  like_count: number
  created_at: string
  edited_at: string | null
  liked: boolean
  mine: boolean
}

export interface Page<T> {
  items: T[]
  page: number
  page_size: number
  total: number
}

export interface Profile extends PublicUser {
  points: number
  created_at: string
  last_seen_at: string
  topic_count: number
  post_count: number
  deal_count: number
  banned: boolean
}

export interface UserReply {
  id: number
  topic_id: number
  topic_title: string
  floor: number
  excerpt: string
  created_at: string
}

export interface DealInput {
  product_id: number
  deal_type: DealType
  value_text: string
  conditions: number
  region: Region
  link_url: string
  expires_at: string | null
  long_term: boolean
}

export interface TopicInput {
  category_id: number
  title: string
  body: string
  tags: string[]
  deal?: DealInput
}

export interface SiteConfig {
  site_name: string
  account_base: string
  client_id: string
}
