import { expect, it } from 'vitest'
import { passwordError, usernameError } from './username.ts'

it('accepts letters, digits, underscores and Chinese characters, counting each Chinese character as 2', () => {
  for (const ok of ['abc', 'Alice_01', 'abc45678901234567890', '张三', '张a', '用户名abcd', '用户名用户名用户名用']) expect(usernameError(ok)).toBeNull()
})

it('says what is wrong with a bad username, and stays quiet while empty', () => {
  expect(usernameError('')).toBeNull()
  expect(usernameError('  ')).toBeNull()
  expect(usernameError('1abc')).toContain('字母或汉字开头')
  expect(usernameError('ab-cd')).toContain('字母、数字、下划线和汉字')
  expect(usernameError('张三！')).toContain('字母、数字、下划线和汉字')
  expect(usernameError('用')).toContain('2 个汉字或 3 个英文字母')
  expect(usernameError('ab')).toContain('2 个汉字或 3 个英文字母')
  expect(usernameError('ab1')).toContain('2 个汉字或 3 个英文字母')
  expect(usernameError('用户名用户名用户名用户')).toContain('最长 20')
  expect(usernameError('abc456789012345678901')).toContain('最长 20')
})

it('checks password length and rejects the username as a password', () => {
  expect(passwordError('')).toBeNull()
  expect(passwordError('short')).toContain('8')
  expect(passwordError('Alice_01', 'alice_01')).toContain('相同')
  expect(passwordError('long-enough-1', 'alice_01')).toBeNull()
})
