import { expect, it } from 'vitest'
import { passwordError, usernameError } from './username.ts'

it('accepts letters, digits and underscores that start with a letter', () => {
  for (const ok of ['abcd', 'Alice_01', 'a2345678901234567890']) expect(usernameError(ok)).toBeNull()
})

it('says what is wrong with a bad username, and stays quiet while empty', () => {
  expect(usernameError('')).toBeNull()
  expect(usernameError('  ')).toBeNull()
  expect(usernameError('1abc')).toContain('字母开头')
  expect(usernameError('ab-cd')).toContain('字母、数字和下划线')
  expect(usernameError('用户名abcd')).toContain('字母开头')
  expect(usernameError('abc')).toContain('4-20')
  expect(usernameError('a23456789012345678901')).toContain('4-20')
})

it('checks password length and rejects the username as a password', () => {
  expect(passwordError('')).toBeNull()
  expect(passwordError('short')).toContain('8')
  expect(passwordError('Alice_01', 'alice_01')).toContain('相同')
  expect(passwordError('long-enough-1', 'alice_01')).toBeNull()
})
