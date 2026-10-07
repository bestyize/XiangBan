import type { UserDetail } from './types.ts'

export interface UserDraft {
  nickname: string; companionName: string; personality: UserDetail['personality']; cityCode: string;
  timezone: string; workStart: string; workEnd: string; payday: number
}
export function userDraft(detail: UserDetail): UserDraft {
  return { nickname: detail.user.nickname ?? '', companionName: detail.user.companionName, personality: detail.personality,
    cityCode: detail.cityCode ?? '', timezone: detail.timezone, workStart: detail.workStart, workEnd: detail.workEnd, payday: detail.payday ?? 0 }
}
export function userPatch(draft: UserDraft, detail: UserDetail): Partial<UserDraft> {
  const previous = userDraft(detail)
  const cleaned = { ...draft, nickname: draft.nickname.trim(), companionName: draft.companionName.trim(), timezone: draft.timezone.trim() }
  return Object.fromEntries(Object.entries(cleaned).filter(([key, value]) => value !== previous[key as keyof UserDraft])) as Partial<UserDraft>
}
