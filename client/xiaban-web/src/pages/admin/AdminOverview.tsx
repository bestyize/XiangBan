import { useEffect, useState } from 'react'
import { Link } from 'wouter'
import { adminRequest } from './api.ts'
import type { Overview, Health, EmailSettings } from './types.ts'
import { Alerts, Badge, panel, TextButton } from './ui.tsx'

export default function AdminOverview() {
  const [stats, setStats] = useState<Overview | null>(null)
  const [health, setHealth] = useState<Health | null>(null)
  const [mail, setMail] = useState<EmailSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [revision, setRevision] = useState(0)
  useEffect(() => {
    const controller = new AbortController()
    setLoading(true); setError(null)
    Promise.all([
      adminRequest<Overview>('/overview', 'GET', undefined, controller.signal),
      fetch('/api/health', { signal: controller.signal }).then(async (r) => { if (!r.ok) throw new Error('服务健康检查失败'); return await r.json() as Health }),
      adminRequest<EmailSettings>('/email', 'GET', undefined, controller.signal),
    ]).then(([counts, status, settings]) => { setStats(counts); setHealth(status); setMail(settings) })
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [revision])
  const cards = stats ? [
    { label: '全部账号', value: stats.totalUsers, description: `${stats.guestUsers.toLocaleString()} 个游客账号` },
    { label: '邮箱账号', value: stats.emailUsers, description: '已验证邮箱，可跨设备登录' },
    { label: '近 7 天新账号', value: stats.newUsers7d, description: `同期 ${stats.newEmailUsers7d.toLocaleString()} 个账号完成邮箱验证` },
    { label: '近 7 天活跃账号', value: stats.activeUsers7d, description: '按打开页面、访问事件或打卡计算' },
  ] : []
  return <div className="space-y-6"><Alerts error={error} />
    <div className="flex items-center justify-between"><p className="text-xs text-slate-500">游客会自动创建；邮箱账号数代表已验证邮箱的用户。</p><TextButton disabled={loading} onClick={() => setRevision((n) => n + 1)}>{loading ? '正在刷新…' : '刷新数据'}</TextButton></div>
    <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">{stats ? cards.map((card) => <section key={card.label} className={panel}>
      <p className="text-sm text-slate-500">{card.label}</p><p className="mt-4 text-3xl font-semibold text-slate-900">{card.value.toLocaleString()}</p><p className="mt-3 text-xs leading-relaxed text-slate-500">{card.description}</p>
    </section>) : <p className="text-slate-500">正在加载统计…</p>}</div>
    <div className="grid gap-6 lg:grid-cols-2"><section className={panel}><div className="flex items-center justify-between"><h2 className="font-semibold">服务状态</h2><Badge tone={health?.db && health.redis ? 'green' : 'amber'}>{health?.db && health.redis ? '运行正常' : '待检查'}</Badge></div>
      <div className="mt-5 space-y-4">{[['网站服务', health?.status === 'ok'], ['数据库', health?.db], ['Redis', health?.redis]].map(([label, ok]) => <div key={String(label)} className="flex items-center justify-between border-b border-slate-100 pb-3 text-sm"><span className="text-slate-600">{label}</span><span className={ok ? 'text-emerald-600' : 'text-slate-400'}>{health ? ok ? '正常' : '异常' : '检测中'}</span></div>)}</div>
    </section><section className={panel}><h2 className="font-semibold">邮件与打卡</h2><dl className="mt-5 grid grid-cols-3 gap-4">
      {[['有效打卡', stats?.totalCheckins], ['24 小时投递成功', stats?.sentMail24h], ['24 小时失败尝试', stats?.failedMail24h]].map(([label, value]) => <div key={String(label)}><dt className="text-xs text-slate-500">{label}</dt><dd className="mt-2 text-2xl font-semibold">{value?.toLocaleString() ?? '—'}</dd></div>)}</dl>
      <div className="mt-6 flex flex-wrap items-center justify-between gap-3 rounded-lg bg-slate-50 p-4 text-sm"><span className="text-slate-600">邮件服务：{mail ? !mail.enabled ? '已关闭' : mail.passwordConfigured && mail.host ? '已配置' : '待配置' : '检测中'}</span><Link href="/admin/email" className="font-medium text-slate-800 underline">管理发信设置 →</Link></div>
      <p className="mt-3 text-xs text-slate-400">邮件数字来自后台通知任务的投递日志，包含失败重试。</p>
    </section></div>
    <section className={panel}><h2 className="font-semibold">常用操作</h2><div className="mt-4 grid gap-3 sm:grid-cols-3">{[
      ['/admin/users', '查找用户', '查看邮箱验证、设备和打卡情况'], ['/admin/email', '配置邮箱服务', '保存 SMTP 配置并发送测试邮件'], ['/admin/audit', '检查操作记录', '追踪用户备注和后台设置变更'],
    ].map(([href, title, description]) => <Link key={href} href={href} className="rounded-lg border border-slate-200 p-4 transition hover:border-slate-400 hover:bg-slate-50"><p className="text-sm font-medium">{title} →</p><p className="mt-2 text-xs text-slate-500">{description}</p></Link>)}</div></section>
  </div>
}
