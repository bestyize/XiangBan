export class AdminError extends Error {
  readonly status: number
  constructor(message: string, status: number) { super(message); this.status = status }
}
let expired: (() => void) | null = null
export function onAdminExpired(handler: (() => void) | null) { expired = handler }
export async function adminRequest<T>(path: string, method = 'GET', body?: unknown, signal?: AbortSignal): Promise<T> {
  const options: RequestInit = { method, credentials: 'same-origin', signal }
  if (body !== undefined) { options.headers = { 'Content-Type': 'application/json' }; options.body = JSON.stringify(body) }
  const response = await fetch(`/api/admin${path}`, options)
  const result = await response.json().catch(() => null)
  if (!response.ok) {
    if (response.status === 401 && path !== '/login' && path !== '/session') expired?.()
    throw new AdminError(result?.message ?? '请求失败，请稍后重试', response.status)
  }
  return result as T
}
