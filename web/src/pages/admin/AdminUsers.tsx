import { useEffect, useState } from 'react'
import { adminRequest } from './api.ts'
import type { UserPage } from './types.ts'
import { Alerts, Badge, Button, Field, formatTime, input, panel, Pagination, TextButton } from './ui.tsx'
import { userCsv } from './users.ts'
import UserDetail from './UserDetail.tsx'

const defaults = { q: '', type: 'all', onboarding: 'all', sort: 'newest', from: '', to: '' }
export default function AdminUsers() {
  const [draft, setDraft] = useState(defaults)
  const [filters, setFilters] = useState(defaults)
  const [page, setPage] = useState(1)
  const [size, setSize] = useState(20)
  const [data, setData] = useState<UserPage | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [revision, setRevision] = useState(0)
  const [selected, setSelected] = useState<{ id: number; action: 'view' | 'edit' | 'delete' } | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const query = new URLSearchParams({ ...filters, page: String(page), pageSize: String(size) }).toString()
  const filtered = filters.q || filters.type !== 'all' || filters.onboarding !== 'all' || filters.from || filters.to
  useEffect(() => {
    const controller = new AbortController()
    setLoading(true); setError(null)
    adminRequest<UserPage>(`/users?${query}`, 'GET', undefined, controller.signal).then(setData)
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [query, revision])
  function reset() { setDraft(defaults); setFilters(defaults); setPage(1) }
  function exportPage() {
    if (!data?.items.length) return
    const url = URL.createObjectURL(new Blob([userCsv(data.items)], { type: 'text/csv;charset=utf-8' }))
    const anchor = document.createElement('a')
    anchor.href = url; anchor.download = `users-page-${data.page}.csv`; anchor.click()
    setTimeout(() => URL.revokeObjectURL(url), 1000)
  }
  return <div className="space-y-5"><Alerts error={error} message={message} />
    <form className={panel} onSubmit={(e) => { e.preventDefault(); setFilters({ ...draft, q: draft.q.trim() }); setPage(1) }}>
      <div className="grid items-end gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <div className="sm:col-span-2"><Field title="搜索用户"><input className={input} type="search" maxLength={100} value={draft.q} onChange={(e) => setDraft({ ...draft, q: e.target.value })} placeholder="用户编号、邮箱、昵称、小伴名字或邀请码" /></Field></div>
        <Field title="账号类型"><select className={input} value={draft.type} onChange={(e) => setDraft({ ...draft, type: e.target.value })}><option value="all">全部账号</option><option value="email">邮箱账号</option><option value="guest">游客账号</option></select></Field>
        <Field title="引导状态"><select className={input} value={draft.onboarding} onChange={(e) => setDraft({ ...draft, onboarding: e.target.value })}><option value="all">全部状态</option><option value="done">已完成引导</option><option value="pending">未完成引导</option></select></Field>
        <Field title="账号创建日期 · 从"><input className={input} type="date" value={draft.from} onChange={(e) => setDraft({ ...draft, from: e.target.value })} /></Field>
        <Field title="账号创建日期 · 至"><input className={input} type="date" value={draft.to} min={draft.from || undefined} onChange={(e) => setDraft({ ...draft, to: e.target.value })} /></Field>
        <div className="flex gap-2 sm:col-span-2"><Button type="submit" disabled={loading}>查询用户</Button><TextButton disabled={loading} onClick={reset}>重置筛选</TextButton></div>
      </div>
    </form>
    <section aria-busy={loading} className="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xs">
      <div className="flex flex-wrap items-center justify-between gap-3 px-5 py-4"><div className="flex items-center gap-3"><h2 className="font-semibold">用户列表</h2>{loading && <span role="status" className="text-xs text-slate-400">正在加载…</span>}</div>
        <div className="flex flex-wrap items-center gap-2"><select aria-label="用户排序方式" className="rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm" value={filters.sort} disabled={loading} onChange={(e) => {
          const sort = e.target.value; setFilters({ ...filters, sort }); setDraft({ ...draft, sort }); setPage(1)
        }}><option value="newest">账号创建 · 最新优先</option><option value="oldest">账号创建 · 最早优先</option><option value="active">最近活跃优先</option><option value="verified">邮箱验证 · 最新优先</option></select>
          <TextButton disabled={loading} onClick={() => setRevision((n) => n + 1)}>刷新</TextButton><TextButton disabled={loading || !data?.items.length} onClick={exportPage}>导出当前页 CSV</TextButton></div>
      </div>
      <div className="overflow-x-auto"><table className="w-full min-w-[1000px] text-left text-sm"><thead className="border-y border-slate-200 bg-slate-50 text-xs text-slate-500"><tr>
        {['用户', '账号类型', '创建 / 验证时间', '最近活跃', '设备 / 打卡', '引导状态', '操作'].map((title) => <th key={title} className="px-5 py-3 font-medium">{title}</th>)}
      </tr></thead><tbody className="divide-y divide-slate-100">{data?.items.map((u) => <tr key={u.id} className="hover:bg-slate-50/70">
        <td className="px-5 py-4"><div className="flex items-center gap-2"><button className="font-medium text-slate-900 hover:underline" onClick={() => setSelected({ id: u.id, action: 'view' })}>{u.nickname || `用户 #${u.id}`}</button>{u.hasNote && <Badge tone="amber">有备注</Badge>}</div>
          <p className="mt-1 text-xs text-slate-500">{u.email ?? '尚未绑定邮箱'}</p><p className="mt-1 text-xs text-slate-400">#{u.id} · 小伴：{u.companionName}</p></td>
        <td className="px-5 py-4"><Badge tone={u.email ? 'blue' : 'slate'}>{u.email ? '邮箱账号' : '游客账号'}</Badge></td>
        <td className="whitespace-nowrap px-5 py-4 text-xs"><p className="text-slate-600">创建 {formatTime(u.createdAt)}</p><p className="mt-1 text-slate-400">{u.emailVerifiedAt ? `验证 ${formatTime(u.emailVerifiedAt)}` : '邮箱未验证'}</p></td>
        <td className="whitespace-nowrap px-5 py-4 text-xs text-slate-500">{formatTime(u.lastActiveAt)}</td>
        <td className="px-5 py-4 text-xs text-slate-500"><p>{u.deviceCount} 个设备</p><p className="mt-1">{u.checkinCount} 次打卡</p></td>
        <td className="px-5 py-4"><Badge tone={u.onboardedAt ? 'green' : 'amber'}>{u.onboardedAt ? '已完成' : '未完成'}</Badge></td>
        <td className="px-5 py-4"><div className="flex gap-3 whitespace-nowrap"><button className="font-medium text-slate-700 hover:underline" onClick={() => setSelected({ id: u.id, action: 'view' })}>详情</button>
          <button className="font-medium text-slate-700 hover:underline" onClick={() => setSelected({ id: u.id, action: 'edit' })}>编辑</button>
          <button className="font-medium text-red-600 hover:underline" onClick={() => setSelected({ id: u.id, action: 'delete' })}>删除</button></div></td>
      </tr>)}{!data?.items.length && <tr><td colSpan={7} className="px-5 py-16 text-center text-slate-400">{loading ? '正在加载用户…' : filtered ? '没有符合筛选条件的用户，请调整搜索或筛选。' : '暂无用户，用户访问网站后会显示在这里。'}</td></tr>}</tbody></table></div>
      <Pagination page={data?.page ?? page} totalPages={data?.totalPages ?? 1} total={data?.total ?? 0} size={size} loading={loading}
        onPage={setPage} onSize={(n) => { setSize(n); setPage(1) }} />
    </section>
    <p className="text-xs leading-relaxed text-slate-400">所有时间显示为北京时间。列表按页从服务器加载；“设备”是关联浏览器数量，最近活跃包含页面访问和打卡。</p>
    <UserDetail id={selected?.id ?? null} initialAction={selected?.action} onClose={() => setSelected(null)} onSaved={() => setRevision((n) => n + 1)} onDeleted={(deletedId) => {
      setSelected((current) => current?.id === deletedId ? null : current); setMessage(`用户 #${deletedId} 已删除。`); setRevision((n) => n + 1)
    }} />
  </div>
}
