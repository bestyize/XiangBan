import { useState, type ComponentProps, type ReactNode } from 'react'
export const input = 'w-full rounded-lg border border-slate-300 bg-white px-3 py-2.5 text-sm text-slate-800 shadow-xs focus:border-slate-600 focus:outline-none disabled:bg-slate-100 disabled:opacity-60'
export const panel = 'rounded-xl border border-slate-200 bg-white p-5 shadow-xs sm:p-6'
export function Field({ title, children, hint }: { title: string; children: ReactNode; hint?: string }) {
  return <label className="block space-y-2"><span className="block text-sm font-medium text-slate-700">{title}</span>{children}
    {hint && <span className="block text-xs leading-relaxed text-slate-500">{hint}</span>}</label>
}
export function Button({ className = '', ...props }: ComponentProps<'button'>) {
  return <button type="button" className={`rounded-lg bg-slate-800 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40 ${className}`} {...props} />
}
export function TextButton({ className = '', ...props }: ComponentProps<'button'>) {
  return <button type="button" className={`rounded-lg px-3 py-2 text-sm text-slate-600 hover:bg-slate-100 disabled:opacity-40 ${className}`} {...props} />
}
export function Alerts({ error, message }: { error: string | null; message?: string | null }) {
  return <>{error && <p role="alert" className="mb-5 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{error}</p>}
    {message && <p role="status" className="mb-5 rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">{message}</p>}</>
}
export function Badge({ children, tone = 'slate' }: { children: ReactNode; tone?: 'slate' | 'green' | 'amber' | 'blue' }) {
  const colors = { slate: 'bg-slate-100 text-slate-600', green: 'bg-emerald-50 text-emerald-700', amber: 'bg-amber-50 text-amber-700', blue: 'bg-blue-50 text-blue-700' }
  return <span className={`inline-flex rounded-md px-2 py-1 text-xs font-medium ${colors[tone]}`}>{children}</span>
}
export function formatTime(value: number | null) {
  if (value === null) return '—'
  return new Intl.DateTimeFormat('zh-CN', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).format(value)
}
export function Pagination({ page, totalPages, total, size, loading, onPage, onSize }: {
  page: number; totalPages: number; total: number; size: number; loading: boolean; onPage: (page: number) => void; onSize?: (size: number) => void
}) {
  return <div className="flex flex-wrap items-center justify-between gap-3 border-t border-slate-200 px-5 py-4 text-sm text-slate-500">
    <div className="flex items-center gap-3"><span>共 {total.toLocaleString()} 条</span>{onSize && <label className="flex items-center gap-2">每页
      <select aria-label="每页条数" className="rounded-md border border-slate-300 bg-white px-2 py-1.5" value={size} disabled={loading} onChange={(e) => onSize(Number(e.target.value))}>
        {[10, 20, 50, 100].map((n) => <option key={n} value={n}>{n}</option>)}
      </select></label>}</div>
    <div className="flex items-center gap-2"><TextButton disabled={loading || page <= 1} onClick={() => onPage(1)}>首页</TextButton>
      <TextButton disabled={loading || page <= 1} onClick={() => onPage(page - 1)}>上一页</TextButton><span className="px-2">{page} / {totalPages}</span>
      <TextButton disabled={loading || page >= totalPages} onClick={() => onPage(page + 1)}>下一页</TextButton></div>
  </div>
}
export function useAdminAction() {
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  async function action(name: string, job: () => Promise<void>) {
    setBusy(name); setError(null); setMessage(null)
    try { await job() } catch (e) { setError((e as Error).message || '网络连接失败，请稍后重试') } finally { setBusy(null) }
  }
  return { busy, error, message, setMessage, setError, action }
}
