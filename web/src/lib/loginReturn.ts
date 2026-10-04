const KEY = 'xb_login_return'
export interface LoginReturn { path: string; start?: boolean }
export function rememberLoginReturn(value: LoginReturn) { try { sessionStorage.setItem(KEY, JSON.stringify(value)) } catch {} }
export function readLoginReturn(): LoginReturn | null {
  try {
    const value = JSON.parse(sessionStorage.getItem(KEY) ?? 'null')
    return value && typeof value.path === 'string' && /^\/(?:tests\/(?:mbti|mbanti|animal|afterhours)|games\/[a-z0-9-]+)$/.test(value.path) ? value : null
  } catch { return null }
}
export function clearLoginReturn() { try { sessionStorage.removeItem(KEY) } catch {} }
