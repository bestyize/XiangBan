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
  limits: {
    daily_topics: number
    daily_posts: number
    tl1_days: number
    tl1_reads: number
    /** 单个附件的大小上限（字节），0 表示暂时不能上传 */
    upload_max_bytes: number
    /** 正文插图的大小上限（字节），0 表示暂时不能插图 */
    image_max_bytes: number
  }
}

export interface Category {
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
  /** 内置标签：发帖时按这个分类给出的可一键选用的标签 */
  preset_tags: string[]
  topic_count: number
}

export type PinScope = 'none' | 'site' | 'parent' | 'category'

export interface TopicItem {
  id: number
  title: string
  excerpt: string
  category: { id: number; slug: string; name: string }
  author: PublicUser
  tags: string[]
  status: string
  pinned: boolean
  pin_scope: PinScope
  featured: boolean
  locked: boolean
  /** 谁能读：0 所有人，1 成员以上，2 老用户以上，3 守护者 */
  min_read_level: number
  reply_count: number
  like_count: number
  view_count: number
  created_at: string
  last_reply_at: string
  edited_at: string | null
  help: HelpInfo | null
}

export interface HelpInfo {
  bounty: number
  status: 'open' | 'solved' | 'closed' | 'expired'
  accepted_post_id: number | null
  expires_at: string
}

export interface PointsWallet {
  balance: number
  visited_today: boolean
  checked_in_today: boolean
  month_check_ins: number
  invite_code: string
  invites: number
  rewarded_invites: number
  invite_income: number
  day: string
}

export interface PointsLedgerItem {
  id: number
  delta: number
  balance_after: number
  kind: string
  reference: string
  description: string
  created_at: string
}

export interface InviteItem {
  nickname: string
  invite_status: 'pending' | 'rewarded' | 'capped' | 'ineligible'
  created_at: string
}

export interface Viewer {
  liked: boolean
  bookmarked: boolean
  can_edit: boolean
  can_reply: boolean
  can_pin: boolean
}

export interface Attachment {
  id: string
  name: string
  size: number
  price: number
  purchased: boolean
}

export interface TopicDetail extends TopicItem {
  body_html: string
  body_md: string
  attachments: Attachment[]
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
  /** 你屏蔽了这个作者：正文不返回，界面折叠 */
  blocked: boolean
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
  banned: boolean
  following: boolean
  /** 你是否屏蔽了这个用户 */
  blocked: boolean
  following_count: number
  follower_count: number
}

export interface FollowState {
  following: boolean
  following_count: number
  follower_count: number
}

export interface UserReply {
  id: number
  topic_id: number
  topic_title: string
  floor: number
  excerpt: string
  created_at: string
}

export interface TopicInput {
  category_id: number
  title: string
  body: string
  tags: string[]
  /** 不传：发帖按公开，编辑保持原样 */
  min_read_level?: number
  /** 不传：发帖没有附件，编辑保持原样；传了以这个列表为准 */
  attachment_ids?: string[]
  attachment_prices?: { id: string; price: number }[]
  help?: { bounty: number; days: number }
}

export interface SiteConfig {
  site_name: string
  account_base: string
  client_id: string
}

