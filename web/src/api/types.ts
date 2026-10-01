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
  onboarded: boolean
  createdAt: number
}

export interface ProfileUpdate {
  companionName?: string
  personality?: Personality
  /** Empty string clears it. */
  nickname?: string
  cityCode?: string
  workStart?: string
  workEnd?: string
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
  undoUntil: number | null
}

export interface TodayResponse {
  workDay: string
  checkin: Checkin | null
}
