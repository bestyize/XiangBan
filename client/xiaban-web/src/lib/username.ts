export const USERNAME_MIN = 4
export const USERNAME_MAX = 20
export const PASSWORD_MIN = 8

/** 与账号中心的规则一致，只用来在输入时少打几次请求；最终以服务端为准。 */
export function usernameError(raw: string): string | null {
  const value = raw.trim()
  if (!value) return null
  if (!/^[A-Za-z]/.test(value)) return '用户名需以字母开头'
  if (!/^[A-Za-z0-9_]+$/.test(value)) return '只能包含字母、数字和下划线'
  if (value.length < USERNAME_MIN || value.length > USERNAME_MAX) return `需为 ${USERNAME_MIN}-${USERNAME_MAX} 个字符`
  return null
}

export function passwordError(raw: string, username = ''): string | null {
  if (!raw) return null
  if (raw.length < PASSWORD_MIN) return `密码至少 ${PASSWORD_MIN} 位`
  if (raw.length > 128) return '密码最多 128 位'
  if (username && raw.toLowerCase() === username.trim().toLowerCase()) return '密码不能和用户名相同'
  return null
}
