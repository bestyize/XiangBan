// 匿名埋点：随机设备标识、只报白名单事件名和一个内容类型/来源维度，不带账号、内容、搜索词。
// 可在「设置」里一键关闭；浏览器开了“不跟踪”也不上报。事件攒一小会儿成批发送，失败就丢掉，不重试。

const DEVICE_KEY = 'chat.xiangban.device'
const OFF_KEY = 'chat.xiangban.analytics'

function read(key: string): string | null {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}
function write(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key)
    else localStorage.setItem(key, value)
  } catch {
    // 隐私模式下存不了，本次页面内有效即可
  }
}

export function analyticsEnabled(): boolean {
  if (typeof navigator !== 'undefined' && (navigator.doNotTrack === '1' || (window as { doNotTrack?: string }).doNotTrack === '1')) return false
  return read(OFF_KEY) !== 'off'
}

export function setAnalyticsEnabled(on: boolean) {
  write(OFF_KEY, on ? null : 'off')
  if (!on) {
    queue.clear()
    // 关闭后连设备标识一起忘掉；再打开会是一个全新的匿名设备。
    write(DEVICE_KEY, null)
  }
}

function device(): string {
  let id = read(DEVICE_KEY)
  if (!id || !/^[A-Za-z0-9_-]{16,64}$/.test(id)) {
    id = crypto.randomUUID()
    write(DEVICE_KEY, id)
  }
  return id
}

const queue = new Map<string, { name: string; dim?: string; count: number }>()
let timer: number | undefined

export function track(name: string, dim?: string) {
  if (!analyticsEnabled()) return
  const key = `${name}|${dim ?? ''}`
  const item = queue.get(key)
  if (item) item.count += 1
  else queue.set(key, { name, dim, count: 1 })
  timer ??= window.setTimeout(flush, 3000)
}

export function flush() {
  if (timer !== undefined) window.clearTimeout(timer)
  timer = undefined
  if (!queue.size || !analyticsEnabled()) {
    queue.clear()
    return
  }
  const body = JSON.stringify({ device: device(), events: [...queue.values()] })
  queue.clear()
  try {
    // 页面关闭时 sendBeacon 比 fetch 可靠；两者都是尽力而为。
    if (!navigator.sendBeacon?.('/api/events', new Blob([body], { type: 'application/json' }))) {
      void fetch('/api/events', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body, keepalive: true }).catch(() => {})
    }
  } catch {
    // 忽略
  }
}

if (typeof document !== 'undefined') {
  document.addEventListener('visibilitychange', () => document.visibilityState === 'hidden' && flush())
  // 关闭或整页跳走时 visibilitychange 不一定触发，pagehide 兜底。
  window.addEventListener('pagehide', flush)
}

/** 路由名 → 事件。只看“到了哪类页面”，不带路径参数。 */
export function trackRoute(name: unknown, query: Record<string, unknown>) {
  switch (name) {
    case 'home':
      track(query.tab === 'weekly' ? 'view_weekly' : 'view_home')
      break
    case 'weekly-issue': track('view_weekly_issue'); break
    case 'project': track('view_project'); break
    case 'discuss': track('view_discuss'); break
    case 'search': track('search'); break
    case 'publish': track('publish_start'); break
  }
}
