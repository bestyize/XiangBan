import { useEffect, useState } from 'react'
import { adminRequest } from './api.ts'
import type { AuditPage } from './types.ts'
import { Alerts, Badge, formatTime, Pagination, TextButton } from './ui.tsx'

const actionNames: Record<string, string> = { user_note: '用户备注', user_edit: '编辑用户', user_delete: '删除用户', email_settings: '邮件配置', admin_password: '管理员密码' }
export default function AdminAudit() {
  const [data, setData] = useState<AuditPage | null>(null)
  const [page, setPage] = useState(1)
  const [revision, setRevision] = useState(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  useEffect(() => {
    const controller = new AbortController()
    setLoading(true); setError(null)
    adminRequest<AuditPage>(`/audit?page=${page}&pageSize=20`, 'GET', undefined, controller.signal).then(setData)
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [page, revision])
  return <><Alerts error={error} /><section aria-busy={loading} className="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xs">
    <div className="flex items-center justify-between px-5 py-4"><h2 className="font-semibold">后台操作记录</h2><TextButton disabled={loading} onClick={() => setRevision((n) => n + 1)}>刷新</TextButton></div>
    <div className="overflow-x-auto"><table className="w-full min-w-[760px] text-left text-sm"><thead className="border-y border-slate-200 bg-slate-50 text-xs text-slate-500"><tr>{['时间（北京时间）', '操作人', '操作类型', '关联用户', '操作说明'].map((title) => <th key={title} className="px-5 py-3 font-medium">{title}</th>)}</tr></thead>
      <tbody className="divide-y divide-slate-100">{data?.items.map((item) => <tr key={item.id}><td className="whitespace-nowrap px-5 py-4 text-slate-500">{formatTime(item.createdAt)}</td><td className="px-5 py-4">{item.actor}</td><td className="px-5 py-4"><Badge>{actionNames[item.action] ?? item.action}</Badge></td><td className="px-5 py-4">{item.targetUserId ? `#${item.targetUserId}` : '—'}</td><td className="px-5 py-4">{item.summary}</td></tr>)}
        {!data?.items.length && <tr><td colSpan={5} className="px-5 py-16 text-center text-slate-400">{loading ? '正在加载操作记录…' : '暂无操作记录。后续修改会记录在这里。'}</td></tr>}</tbody></table></div>
    <Pagination page={data?.page ?? page} totalPages={data?.totalPages ?? 1} total={data?.total ?? 0} size={20} loading={loading} onPage={setPage} />
  </section></>
}
