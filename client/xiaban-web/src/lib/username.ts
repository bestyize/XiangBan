export const USERNAME_MAX = 20
export const PASSWORD_MIN = 8

/** 与账号中心的规则一致（汉字占 2 个宽度），只用来在输入时少打几次请求；最终以服务端为准。 */
export function usernameError(raw: string): string | null {
  const value = raw.trim()
  if (!value) return null
  if (!/^[A-Za-z一-鿿]/.test(value)) return '用户名需以字母或汉字开头'
  if (!/^[A-Za-z0-9_一-鿿]+$/.test(value)) return '只能包含字母、数字、下划线和汉字'
  const han = (value.match(/[一-鿿]/g) ?? []).length
  const letters = (value.match(/[A-Za-z]/g) ?? []).length
  if (han > 0 ? han + letters < 2 : letters < 3) return '至少要有 2 个汉字或 3 个英文字母'
  const width = [...value].reduce((n, c) => n + (/[一-鿿]/.test(c) ? 2 : 1), 0)
  if (width > USERNAME_MAX) return `最长 ${USERNAME_MAX} 个字符，汉字算 2 个`
  return null
}

export function passwordError(raw: string, username = ''): string | null {
  if (!raw) return null
  if (raw.length < PASSWORD_MIN) return `密码至少 ${PASSWORD_MIN} 位`
  if (raw.length > 128) return '密码最多 128 位'
  if (username && raw.toLowerCase() === username.trim().toLowerCase()) return '密码不能和用户名相同'
  return null
}
