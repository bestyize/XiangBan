export type Personality = 'toxic' | 'gentle' | 'chatty' | 'tsundere'

export interface Profile {
  companionName: string
  personality: Personality
  nickname: string | null
  cityCode: string | null
  cityName: string | null
  timezone: string
  workStart: string
  workEnd: string
  payday: number | null
  showCityOnCard: boolean
  inviteCode: string
  /** Masked; null while anonymous */
  email: string | null
  recallOptIn: boolean
  termsVersion: string | null
  onboarded: boolean
  createdAt: number
}

export interface BindResponse {
  token: string
  merged: boolean
  profile: Profile
}

export interface Guardian {
  contactName: string
  contactEmailMasked: string
  selfName: string
  thresholdDays: number
  paused: boolean
  nextRemindAt: number | null
  nextNotifyAt: number | null
}

export interface GuardianUpdate {
  contactName: string
  contactEmail: string
  selfName: string
  thresholdDays: number
  consent: boolean
  informedContact: boolean
}

export interface ProfileUpdate {
  companionName?: string
  personality?: Personality
  /** Empty string clears it. */
  nickname?: string
  cityCode?: string
  workStart?: string
  workEnd?: string
  /** 0 clears it. */
  payday?: number
  showCityOnCard?: boolean
  onboarded?: boolean
}

export interface SessionResponse {
  token: string
  deviceId: string
  profile: Profile
}

export interface City {
  code: string
  name: string
  province: string
}

export interface PersonalityInfo {
  code: Personality
  name: string
  desc: string
  sample: string
}

export interface Meta {
  cities: City[]
  personalities: PersonalityInfo[]
}

export type Mood =
  | 'idle' | 'smug' | 'happy' | 'welcome' | 'heartache' | 'sleepy'
  | 'worried' | 'sparkle' | 'shocked' | 'aggrieved' | 'excited' | 'care'

export type TitleCode =
  | 'exact_on_time' | 'weekend_work' | 'borrowed_weekend' | 'holiday_countdown'
  | 'early_leaver' | 'on_time' | 'normal_human' | 'overtime_hero' | 'night_owl' | 'immortal'

export interface TitleView {
  code: TitleCode
  name: string
  rare: boolean
  mood: Mood
}

export interface ReplyView {
  text: string
  source: 'library' | 'llm'
  pending: boolean
}

export interface Checkin {
  id: string
  workDay: string
  checkedAt: number
  localTime: string
  title: TitleView
  mood: Mood
  reply: ReplyView
  careMode: boolean
  undoUntil: number | null
  stats: CheckinStats
  holiday: { code: string; name: string } | null
  newUnlocks: Unlock[]
}

export interface Unlock {
  id: string
  name: string
  hint: string
  unlockedAt: number | null
}

export interface UnlocksResponse {
  appTotal: number
  webTotal: number
  unlockedCount: number
  items: Unlock[]
}

export interface Percentile {
  percent: number
  scope: 'city' | 'all'
  cityName: string | null
}

export interface CheckinStats {
  streak: number
  totalDays: number
  percentile: Percentile | null
}

export type TipKind = 'rain' | 'cold' | 'traffic' | 'holiday' | 'makeup' | 'morning' | 'duty' | 'week' | 'month' | 'payday'

export interface Tip {
  kind: TipKind
  text: string
}

export interface Stats {
  totalDays: number
  currentStreak: number
  lastStreak: number | null
  bestStreak: number
  todayOnDutyMinutes: number | null
  weekAvgOffTime: string | null
  weekCount: number
  monthOvertimeMinutes: number
  monthCount: number
  titleCounts: Partial<Record<TitleCode, number>>
}

export interface Greeting {
  text: string
  mood: Mood
  kind: 'idle' | 'return'
}

export interface TodayResponse {
  workDay: string
  checkin: Checkin | null
  greeting: Greeting | null
  tips: Tip[]
}
