const KEY = 'chat.xiangban.invite'
const TTL = 7 * 24 * 60 * 60 * 1000

export function captureInvite() {
  const code = new URLSearchParams(location.search).get('invite')
  if (!code || !/^[a-f0-9]{32}$/i.test(code)) return
  try {
    if (!pendingInvite()) localStorage.setItem(KEY, JSON.stringify({ code, expires: Date.now() + TTL }))
  } catch { /* 私密模式下忽略 */ }
}

export function pendingInvite(): string | undefined {
  try {
    const value = JSON.parse(localStorage.getItem(KEY) ?? 'null')
    if (value?.expires > Date.now() && /^[a-f0-9]{32}$/i.test(value.code)) return value.code
  } catch { /* 忽略无效存储 */ }
  return undefined
}

export function clearInvite() {
  try { localStorage.removeItem(KEY) } catch { /* 忽略 */ }
}

export function inviteUrl(uid: string, path = '/') {
  const url = new URL(path, location.origin)
  url.searchParams.set('invite', uid.replaceAll('-', ''))
  return url.toString()
}
