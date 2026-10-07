import { useEffect, useState } from 'react'
import { Link, useLocation } from 'wouter'
import { AdminError, adminRequest, onAdminExpired } from './admin/api.ts'
import { Alerts, Button, Field, input, TextButton, useAdminAction } from './admin/ui.tsx'
import AdminEmail from './admin/AdminEmail.tsx'
import AdminSecurity from './admin/AdminSecurity.tsx'
import AdminOverview from './admin/AdminOverview.tsx'
import AdminMetrics from './admin/AdminMetrics.tsx'
import AdminLlm from './admin/AdminLlm.tsx'
import AdminUsers from './admin/AdminUsers.tsx'
import AdminAudit from './admin/AdminAudit.tsx'
import AdminDisplay from './admin/AdminDisplay.tsx'

const menu = [
  { href: '/admin', label: '统计概览', icon: 'overview', description: '查看账号增长、活跃情况和服务状态。' },
  { href: '/admin/metrics', label: '数据监控', icon: 'metrics', description: '每日访问、注册、打卡、分享、游戏与测试人数。' },
  { href: '/admin/users', label: '用户管理', icon: 'users', description: '搜索用户、查看注册信息与记录，管理运营备注。' },
  { href: '/admin/display', label: '展示配置', icon: 'display', description: '控制小伴底部入口，以及游戏和趣味测试项目的展示、隐藏与排序。' },
  { href: '/admin/email', label: '邮箱设置', icon: 'email', description: '配置发信服务，并确认邮件送达。' },
  { href: '/admin/llm', label: '模型配置', icon: 'llm', description: '配置大模型供应商（OpenAI 兼容协议），管理 API Key 与模型切换。' },
  { href: '/admin/audit', label: '操作日志', icon: 'audit', description: '查看用户备注、邮件配置和管理员密码的修改记录。' },
  { href: '/admin/security', label: '账号安全', icon: 'security', description: '管理管理员密码与登录会话。' },
] as const

function Icon({ name }: { name: string }) {
  const paths: Record<string, string> = {
    overview: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
    metrics: 'M3 19h18 M7 17V9 M12 17V5 M17 17v-6',
    display: 'M4 5h16M4 12h16M4 19h16M8 3v4M16 10v4M10 17v4',
    users: 'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M16 3a4 4 0 0 1 0 8 M22 21v-2a4 4 0 0 0-3-3.87 M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0',
    email: 'M3 5h18v14H3z M3 6l9 7 9-7',
    llm: 'M12 2a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2 2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z M12 16a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-2a2 2 0 0 1 2-2z M2 12a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2 2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z M16 12a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2z M8 8h8v8H8z',
    audit: 'M8 3h8v4H8z M8 5H5v16h14V5h-3 M8 12h8 M8 16h5',
    security: 'M12 3l8 4v5c0 5-8 9-8 9s-8-4-8-9V7z M8 12l3 3 5-6',
  }
  return <svg aria-hidden="true" width="19" height="19" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d={paths[name]} /></svg>
}

export default function AdminPage() {
  const [location] = useLocation()
  const [checking, setChecking] = useState(true)
  const [username, setUsername] = useState<string | null>(null)
  const [loginName, setLoginName] = useState('admin')
  const [password, setPassword] = useState('')
  const [mobileMenu, setMobileMenu] = useState(false)
  const { busy, error, message, setError, setMessage, action } = useAdminAction()
  const current = menu.find((item) => item.href === location) ?? menu[0]

  useEffect(() => {
    onAdminExpired(() => { setUsername(null); setMessage('登录已过期，请重新登录管理员账号。') })
    const controller = new AbortController()
    adminRequest<{ username: string }>('/session', 'GET', undefined, controller.signal)
      .then((s) => setUsername(s.username))
      .catch((e: Error) => { if (e.name !== 'AbortError' && !(e instanceof AdminError && e.status === 401)) setError(e.message) })
      .finally(() => setChecking(false))
    return () => { controller.abort(); onAdminExpired(null) }
  }, [])
  useEffect(() => { setMobileMenu(false) }, [location])
  useEffect(() => {
    const close = (e: KeyboardEvent) => { if (e.key === 'Escape') setMobileMenu(false) }
    window.addEventListener('keydown', close)
    return () => window.removeEventListener('keydown', close)
  }, [])

  if (checking) return <main style={{ colorScheme: 'light' }} className="flex min-h-dvh items-center justify-center bg-[#0f1726] text-slate-400">正在打开管理后台…</main>
  if (!username) return <main style={{ colorScheme: 'light' }} className="relative flex min-h-dvh items-center justify-center overflow-hidden bg-[#0f1726] px-5 py-12 text-slate-800">
    <div aria-hidden="true" className="pointer-events-none absolute -top-32 left-1/2 size-96 -translate-x-1/2 rounded-full bg-[#edbd76] opacity-15 blur-3xl" />
    <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/40 to-transparent" />
    <div className="login-modal-in relative w-full max-w-sm">
      <div className="mb-7 text-center"><span className="inline-flex size-12 items-center justify-center rounded-2xl bg-amber-200 text-xl font-bold text-slate-900 shadow-lg shadow-black/20">伴</span>
        <p className="mt-4 text-xs font-semibold tracking-widest text-white/50">相伴 · 管理控制台</p></div>
      <div className="rounded-[22px] bg-white p-7 shadow-[0_24px_60px_-12px_rgba(0,0,0,0.5)] sm:p-8">
        <h1 className="mb-6 text-center text-xl font-semibold text-slate-900">管理员登录</h1>
        <Alerts error={error} message={message} />
        <form className="space-y-5" onSubmit={(e) => { e.preventDefault(); void action('login', async () => {
          const session = await adminRequest<{ username: string }>('/login', 'POST', { username: loginName, password })
          setPassword(''); setUsername(session.username)
        }) }}>
          <Field title="管理员账号"><input className={input} autoComplete="username" required maxLength={40} value={loginName} onChange={(e) => setLoginName(e.target.value)} /></Field>
          <Field title="密码"><input className={input} type="password" autoComplete="current-password" required maxLength={256} value={password} onChange={(e) => setPassword(e.target.value)} /></Field>
          <Button className="w-full" type="submit" disabled={!!busy}>{busy ? '正在登录…' : '登录管理后台'}</Button>
        </form>
      </div>
      <Link href="/" className="mt-6 block text-center text-sm text-white/50 hover:text-white/80">返回网站</Link>
    </div>
  </main>

  return <div style={{ colorScheme: 'light' }} className="min-h-dvh bg-slate-50 text-slate-800">
    {mobileMenu && <button aria-label="关闭导航" className="fixed inset-0 z-30 bg-slate-950/40 md:hidden" onClick={() => setMobileMenu(false)} />}
    <aside id="admin-sidebar" className={`fixed inset-y-0 left-0 z-40 flex w-60 flex-col bg-[#182338] text-slate-300 transition-transform md:translate-x-0 ${mobileMenu ? 'translate-x-0' : '-translate-x-full'}`}>
      <div className="border-b border-white/10 px-6 py-7"><div className="flex items-center gap-3"><span className="flex size-9 items-center justify-center rounded-xl bg-amber-200 text-lg font-bold text-slate-900">伴</span>
        <div><p className="font-semibold text-white">相伴</p><p className="mt-0.5 text-xs text-slate-400">网站管理后台</p></div></div></div>
      <p className="px-6 pt-7 pb-3 text-[11px] tracking-widest text-slate-500">工作台</p>
      <nav aria-label="后台导航" className="space-y-1 px-3">{menu.map((item) => <Link key={item.href} href={item.href} aria-current={item.href === current.href ? 'page' : undefined}
        className={`flex items-center gap-3 rounded-lg px-4 py-3 text-sm transition ${item.href === current.href ? 'bg-amber-200/10 font-medium text-amber-200' : 'hover:bg-white/5 hover:text-white'}`}>
        <Icon name={item.icon} />{item.label}</Link>)}</nav>
      <div className="mt-auto border-t border-white/10 px-6 py-5"><Link href="/" className="text-sm text-slate-400 hover:text-white">← 返回网站</Link><p className="mt-3 text-xs text-slate-500">相伴 · 运营控制台</p></div>
    </aside>
    <div className="min-w-0 md:ml-60">
      <header className="sticky top-0 z-20 flex h-18 items-center justify-between gap-4 border-b border-slate-200 bg-white/95 px-5 backdrop-blur sm:px-8">
        <div className="flex items-center gap-3"><button aria-label="打开导航" aria-controls="admin-sidebar" aria-expanded={mobileMenu} className="rounded-lg p-2 text-slate-600 md:hidden" onClick={() => setMobileMenu(!mobileMenu)}>☰</button>
          <p className="text-sm"><span className="hidden text-slate-400 sm:inline">管理后台 <span className="mx-3">/</span></span><span className="font-medium">{current.label}</span></p></div>
        <div className="flex items-center gap-3"><span className="flex size-8 items-center justify-center rounded-full bg-slate-100 text-sm font-medium">{username.slice(0, 1).toUpperCase()}</span>
          <span className="hidden text-sm sm:block">{username}</span><TextButton disabled={!!busy} onClick={() => void action('logout', async () => {
            await adminRequest('/logout', 'POST'); setUsername(null); setPassword('')
          })}>退出</TextButton></div>
      </header>
      <main className="mx-auto max-w-[1500px] px-5 py-7 sm:px-8 sm:py-8"><div className="mb-6"><h1 className="text-2xl font-semibold tracking-tight text-slate-900">{current.label}</h1><p className="mt-2 text-sm text-slate-500">{current.description}</p></div>
        <Alerts error={error} message={message} />
        {current.href === '/admin' && <AdminOverview />}
        {current.href === '/admin/users' && <AdminUsers />}
        {current.href === '/admin/display' && <AdminDisplay />}
        {current.href === '/admin/metrics' && <AdminMetrics />}
        {current.href === '/admin/email' && <AdminEmail />}
        {current.href === '/admin/llm' && <AdminLlm />}
        {current.href === '/admin/audit' && <AdminAudit />}
        {current.href === '/admin/security' && <AdminSecurity onChanged={() => { setUsername(null); setMessage('密码已更新，请使用新密码重新登录。') }} />}
      </main>
    </div>
  </div>
}
