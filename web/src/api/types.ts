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
