import { describe, expect, it } from 'vitest'
import { nicknameError, normalizeNickname } from './nickname.ts'
describe('nickname rules', () => {
  it('requires two characters, or four when exclusively English and numbers', () => {
    for (const name of ['阿', 'abc', 'a12']) expect(nicknameError(name)).not.toBeNull()
    for (const name of ['阿明', 'Alice', 'a123', 'User20261234']) expect(nicknameError(name)).toBeNull()
  })
  it('rejects pure numeric variants and normalizes full-width aliases', () => {
    for (const name of ['1234', '１２３４', '1 2 3']) expect(nicknameError(name)).toBe('昵称不能是纯数字')
    expect(normalizeNickname(' Ａｌｉｃｅ ')).toBe('Alice')
    expect(nicknameError('')).toBeNull()
  })
})
