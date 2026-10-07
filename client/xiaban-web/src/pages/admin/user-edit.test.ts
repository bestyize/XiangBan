import { describe, expect, it } from 'vitest'
import { userDraft, userPatch } from './user-edit.ts'
import type { UserDetail } from './types.ts'

const detail: UserDetail = {
  user: { id: 2, nickname: '小李', companionName: '小伴', email: 'me@example.com', emailVerifiedAt: 1, createdAt: 1,
    onboardedAt: 1, lastActiveAt: 1, deviceCount: 1, checkinCount: 1, hasNote: false },
  cityName: '北京', cityCode: '101010100', personality: 'toxic', payday: 15, timezone: 'Asia/Shanghai', workStart: '09:00', workEnd: '18:00',
  inviteCode: 'test', referralSource: null, referredBy: null, termsVersion: '0.1', termsAcceptedAt: 1, overseasConsentAt: null,
  recallOptIn: false, guardianPaused: null, note: '', noteUpdatedAt: null, noteUpdatedBy: null, devices: [], recentCheckins: [],
}
describe('administrator profile patches', () => {
  it('does not overwrite fields that the administrator did not change', () => {
    const draft = userDraft(detail)
    expect(userPatch(draft, detail)).toEqual({})
    expect(userPatch({ ...draft, companionName: '新小伴' }, detail)).toEqual({ companionName: '新小伴' })
  })
  it('sends explicit clearing values for optional profile fields', () => {
    expect(userPatch({ ...userDraft(detail), nickname: '', cityCode: '', payday: 0 }, detail)).toEqual({ nickname: '', cityCode: '', payday: 0 })
  })
  it('normalizes whitespace without changing verified email or registration fields', () => {
    const patch = userPatch({ ...userDraft(detail), nickname: ' 小李 ', timezone: ' Asia/Shanghai ' }, detail)
    expect(patch).toEqual({})
    expect(patch).not.toHaveProperty('email')
    expect(patch).not.toHaveProperty('emailVerifiedAt')
  })
})
