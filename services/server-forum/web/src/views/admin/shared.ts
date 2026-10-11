// 管理后台共用的文案映射、时间格式化、提示消息与列表加载逻辑。
import { ref, type Ref } from 'vue'
import { AdminApiError, type AdminPage } from '@/api/admin'

// ───────────────────────── 文案 ─────────────────────────

export const TOPIC_STATUS: Record<string, string> = { normal: '正常', hidden: '已隐藏', deleted: '已删除' }
export const DEAL_STATUS: Record<string, string> = { valid: '有效', suspect: '疑似失效', expired: '已失效' }
export const USER_STATUS: Record<string, string> = { active: '正常', muted: '禁言中', banned: '已封禁' }
export const TRUST_LEVELS = [
  { level: 0, name: '新人' },
  { level: 1, name: '成员' },
  { level: 2, name: '老用户' },
  { level: 3, name: '守护者' },
]
export const REPORT_REASONS: Record<string, string> = {
  spam: '垃圾广告',
  abuse: '人身攻击',
  illegal: '违法违规',
  copyright: '侵权',
  account_trade: '账号或 Key 交易',
  politics: '引战或时政',
  other: '其他',
}
export const REPORT_STATUS: Record<string, string> = { open: '待处理', resolved: '已处理', dismissed: '已驳回' }
export const CATEGORY_KIND: Record<string, string> = { deal: '羊毛', normal: '普通' }

/** 操作日志里的动作码对应的中文说明，未收录的原样显示。 */
export const AUDIT_ACTIONS: Record<string, string> = {
  'admin.login': '登录后台',
  'admin.password_change': '修改密码',
  'report.resolve': '处理举报',
  'report.dismiss': '驳回举报',
  'topic.hide': '隐藏帖子',
  'topic.restore': '恢复帖子',
  'topic.delete': '删除帖子',
  'topic.pin': '置顶帖子',
  'topic.unpin': '取消置顶',
  'topic.feature': '加精帖子',
  'topic.unfeature': '取消加精',
  'topic.lock': '锁定帖子',
  'topic.unlock': '解除锁定',
  'topic.move': '移动帖子',
  'deal.status': '修改羊毛状态',
  'post.hide': '隐藏回复',
  'post.restore': '恢复回复',
  'user.trust': '修改信任等级',
  'user.status': '修改用户状态',
  'category.create': '新建分类',
  'category.update': '修改分类',
  'product.create': '新建产品',
  'product.update': '修改产品',
}

export const AUDIT_TARGETS: Record<string, string> = {
  admin: '管理员',
  report: '举报',
  topic: '帖子',
  post: '回复',
  user: '用户',
  category: '分类',
  product: '产品',
}

export function trustName(level: number): string {
  return TRUST_LEVELS.find(t => t.level === level)?.name ?? `等级 ${level}`
}

/** 状态对应的标签色调，样式见 admin.css。 */
export function topicTone(status: string): string {
  return status === 'normal' ? 'valid' : status === 'hidden' ? 'suspect' : 'expired'
}
export function dealTone(status: string): string {
  return status // valid / suspect / expired 与样式类同名
}
export function userTone(status: string): string {
  return status === 'active' ? 'valid' : status === 'muted' ? 'suspect' : 'danger'
}

// ───────────────────────── 时间 ─────────────────────────

const pad = (n: number) => String(n).padStart(2, '0')

/** 2026-10-11 14:30，用本地时区。 */
export function fmtTime(iso: string | null | undefined): string {
  if (!iso) return '-'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '-'
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function fmtDate(iso: string | null | undefined): string {
  return fmtTime(iso).slice(0, 10)
}

/** 相对时间，最近用「3 分钟前」，超过 30 天退回到日期。 */
export function fromNow(iso: string | null | undefined): string {
  if (!iso) return '-'
  const diff = Date.now() - new Date(iso).getTime()
  if (Number.isNaN(diff)) return '-'
  const min = Math.floor(diff / 60000)
  if (min < 1) return '刚刚'
  if (min < 60) return `${min} 分钟前`
  const hr = Math.floor(min / 60)
  if (hr < 24) return `${hr} 小时前`
  const day = Math.floor(hr / 24)
  if (day < 30) return `${day} 天前`
  return fmtDate(iso)
}

// ───────────────────────── 提示消息 ─────────────────────────

export interface Toast {
  id: number
  kind: 'ok' | 'error'
  text: string
}

export const toasts = ref<Toast[]>([])
let toastSeq = 0

export function notify(text: string, kind: Toast['kind'] = 'ok') {
  const id = ++toastSeq
  toasts.value.push({ id, kind, text })
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, kind === 'error' ? 6000 : 2800)
}

export function errorText(e: unknown): string {
  return e instanceof AdminApiError || e instanceof Error ? e.message : '操作失败，请稍后重试'
}

/** 执行一次写操作：成功弹提示，失败弹错误；返回是否成功。 */
export async function perform(fn: () => Promise<unknown>, okText?: string): Promise<boolean> {
  try {
    await fn()
    if (okText) notify(okText)
    return true
  } catch (e) {
    // 401 时登录态已被清空，界面会回到登录页，不再重复提示
    if (e instanceof AdminApiError && e.status === 401) return false
    notify(errorText(e), 'error')
    return false
  }
}

// ───────────────────────── 分页列表 ─────────────────────────

/** 通用的分页列表加载：自带加载中、错误、过期响应丢弃。 */
export function usePagedList<T>(fetcher: (page: number, pageSize: number) => Promise<AdminPage<T>>, pageSize = 30) {
  const items = ref([]) as Ref<T[]>
  const total = ref(0)
  const page = ref(1)
  const loading = ref(false)
  const error = ref('')
  let seq = 0

  async function load() {
    const mine = ++seq
    loading.value = true
    error.value = ''
    try {
      const res = await fetcher(page.value, pageSize)
      if (mine !== seq) return
      items.value = res.items
      total.value = res.total
    } catch (e) {
      if (mine !== seq) return
      if (!(e instanceof AdminApiError && e.status === 401)) error.value = errorText(e)
    } finally {
      if (mine === seq) loading.value = false
    }
  }

  /** 筛选条件变化后回到第一页重新加载。 */
  function search() {
    page.value = 1
    return load()
  }

  function goto(p: number) {
    page.value = p
    return load()
  }

  return { items, total, page, pageSize, loading, error, load, search, goto }
}

/** 页面顶部的未处理举报数，侧栏用来显示角标。 */
export const openReports = ref(0)
