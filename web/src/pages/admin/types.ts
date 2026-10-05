export interface EmailSettings {
  enabled: boolean; host: string; port: number; user: string; password: string; from: string;
  fromName: string; ssl: boolean; passwordConfigured: boolean
}
export interface Health { status: string; db: boolean; redis: boolean }
export interface AdminUserRow {
  id: number; email: string | null; emailVerifiedAt: number | null; nickname: string | null; companionName: string;
  createdAt: number; onboardedAt: number | null; lastActiveAt: number; deviceCount: number; checkinCount: number; hasNote: boolean
}
export interface UserPage { items: AdminUserRow[]; total: number; page: number; pageSize: number; totalPages: number }
export interface UserDetail {
  user: AdminUserRow; cityName: string | null; timezone: string; workStart: string; workEnd: string;
  inviteCode: string; referralSource: string | null; referredBy: number | null; termsVersion: string | null;
  termsAcceptedAt: number | null; overseasConsentAt: number | null; recallOptIn: boolean; guardianPaused: boolean | null;
  note: string; noteUpdatedAt: number | null; noteUpdatedBy: string | null;
  devices: { createdAt: number; lastSeenAt: number }[];
  recentCheckins: { workDay: string; checkedAt: number; title: string }[]
  cityCode: string | null; personality: 'toxic' | 'gentle' | 'chatty' | 'tsundere'; payday: number | null
}
export interface Overview {
  totalUsers: number; emailUsers: number; guestUsers: number; newUsers7d: number; newEmailUsers7d: number;
  activeUsers7d: number; totalCheckins: number; sentMail24h: number; failedMail24h: number
}
export interface AuditRow { id: number; actor: string; action: string; targetUserId: number | null; summary: string; createdAt: number }
export interface AuditPage { items: AuditRow[]; total: number; page: number; pageSize: number; totalPages: number }

/** People counted on one statistics day, split by account type. */
export interface MetricPoint { day: string; total: number; registered: number; guest: number }
export interface MetricSeries { key: string; label: string; points: MetricPoint[] }
export interface MetricsResponse {
  days: string[]; dayNote: string; isLatest: boolean
  visits: MetricSeries[]; growth: MetricSeries[]; product: MetricSeries[]
  games: MetricSeries[]; quizzes: MetricSeries[]
}

export interface LlmProviderConfig {
  id: string
  name: string
  baseUrl: string
  apiKey: string
  model: string
  apiKeyConfigured: boolean
}

export interface LlmSettings {
  enabled: boolean
  activeProviderId: string
  providers: LlmProviderConfig[]
  webSearchEnabled: boolean
  webSearchBaseUrl: string
  voiceEnabled: boolean
  /** Empty means the voice follows the user's companion personality. */
  voice: string
}

