export function normalizeNickname(value: string): string {
  return value.normalize('NFKC').replace(/[\p{Cc}\p{Cf}]/gu, '').trim().replace(/\s+/g, ' ')
}
export function nicknameError(raw: string): string | null {
  const value = normalizeNickname(raw)
  if (!value) return null
  const compact = value.replace(/\s/g, '')
  if (/^\p{N}+$/u.test(compact)) return '昵称不能是纯数字'
  const min = /^[A-Za-z0-9]+$/.test(compact) ? 4 : 2
  if ([...compact].length < min) return `昵称至少 ${min} 个字符`
  if ([...value].length > 24) return '昵称最多 24 个字符'
  if (/(https?|www\.|\.com|\.cn|\.xyz|@|微信|vx|qq|加我)/i.test(value)) return '昵称中不能包含联系方式或链接'
  return null
}
