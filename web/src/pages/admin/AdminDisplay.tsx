import { useEffect, useState } from 'react'
import { adminRequest } from './api.ts'
import { Alerts, Badge, Button, panel, TextButton, useAdminAction } from './ui.tsx'
import { defaultDisplaySettings, moveDisplayItem, type DisplayItem, type DisplaySettings } from '../../lib/display.ts'
import { publishDisplaySettings } from '../../lib/displaySettings.tsx'

function CatalogEditor({ title, items, disabled, onChange }: { title: string; items: DisplayItem[]; disabled: boolean; onChange: (items: DisplayItem[]) => void }) {
  return <section className={panel}>
    <div className="mb-5 flex items-center justify-between gap-3"><h2 className="text-lg font-semibold text-slate-900">{title}</h2><Badge tone="green">展示 {items.filter(item => item.visible).length} / {items.length}</Badge></div>
    <p className="mb-4 text-xs text-slate-500">列表按下方顺序展示。可上移、下移或置顶，隐藏后记录仍保留。</p>
    <ol className="space-y-2">{items.map((item, index) => <li key={item.id} className={`flex flex-wrap items-center gap-3 rounded-xl border p-3 sm:flex-nowrap ${item.visible ? 'border-slate-200 bg-white' : 'border-dashed border-slate-200 bg-slate-50'}`}>
      <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-slate-100 text-xs font-semibold tabular-nums text-slate-500">{index + 1}</span>
      <div className="min-w-0 flex-1"><p className="text-sm font-medium text-slate-800">{item.name}</p><p title={item.id} className="mt-1 truncate text-[11px] text-slate-400">{item.id}</p></div>
      <label className="flex min-h-11 cursor-pointer items-center gap-2 text-sm text-slate-600"><input className="size-4 accent-emerald-600" type="checkbox" aria-label={`${item.name}展示`} disabled={disabled} checked={item.visible} onChange={event => onChange(items.map(entry => entry.id === item.id ? { ...entry, visible: event.target.checked } : entry))} />{item.visible ? '展示' : '隐藏'}</label>
      <div className="flex shrink-0 gap-1">
        <TextButton className="min-h-11" aria-label={`${item.name}上移`} disabled={disabled || index === 0} onClick={() => onChange(moveDisplayItem(items, index, index - 1))}>↑</TextButton>
        <TextButton className="min-h-11" aria-label={`${item.name}下移`} disabled={disabled || index === items.length - 1} onClick={() => onChange(moveDisplayItem(items, index, index + 1))}>↓</TextButton>
        <TextButton className="min-h-11 text-xs" aria-label={`${item.name}置顶`} disabled={disabled || index === 0} onClick={() => onChange(moveDisplayItem(items, index, 0))}>置顶</TextButton>
      </div>
    </li>)}</ol>
  </section>
}

export default function AdminDisplay() {
  const [draft, setDraft] = useState<DisplaySettings | null>(null)
  const [saved, setSaved] = useState<DisplaySettings | null>(null)
  const [loading, setLoading] = useState(true)
  const { busy, error, message, setError, setMessage, action } = useAdminAction()
  const dirty = draft && JSON.stringify(draft) !== JSON.stringify(saved)
  useEffect(() => {
    const controller = new AbortController()
    adminRequest<DisplaySettings>('/display', 'GET', undefined, controller.signal).then(value => { setDraft(value); setSaved(value) })
      .catch((cause: Error) => { if (cause.name !== 'AbortError') setError(cause.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [])
  if (loading) return <p className="py-8 text-slate-500">正在读取展示配置…</p>
  if (!draft) return <Alerts error={error} />
  function change(next: DisplaySettings) { setDraft(next); setMessage(null) }
  return <div className="space-y-5">
    <Alerts error={error} message={message} />
    <section className={panel}>
      <div className="flex flex-wrap items-center justify-between gap-4"><div><h2 className="text-lg font-semibold text-slate-900">底部导航 · 小伴</h2><p className="mt-2 text-sm text-slate-500">控制底部 TAB 中的小伴入口。</p></div>
        <label className="flex min-h-11 cursor-pointer items-center gap-3 rounded-full border border-slate-200 bg-slate-50 px-4 text-sm font-medium"><input className="size-4 accent-emerald-600" aria-label="底部小伴入口展示" type="checkbox" disabled={!!busy} checked={draft.chatTabVisible} onChange={event => change({ ...draft, chatTabVisible: event.target.checked })} />{draft.chatTabVisible ? '展示小伴' : '隐藏小伴'}</label>
      </div>
    </section>
    <div className="grid gap-5 xl:grid-cols-2">
      <CatalogEditor title="玩一局 · 游戏项目" items={draft.games} disabled={!!busy} onChange={games => change({ ...draft, games })} />
      <CatalogEditor title="趣味测试 · 测试项目" items={draft.quizzes} disabled={!!busy} onChange={quizzes => change({ ...draft, quizzes })} />
    </div>
    <div className="sticky bottom-4 z-10 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 bg-white/95 p-4 shadow-lg backdrop-blur">
      <p className="text-xs text-slate-500">{dirty ? '有尚未保存的修改' : '展示配置已同步'} · 用户刷新页面后生效</p>
      <div className="flex gap-2"><TextButton disabled={!!busy} onClick={() => { change(defaultDisplaySettings(draft)); setMessage('默认展示已恢复到草稿，保存后生效。') }}>恢复默认</TextButton>
        <Button disabled={!!busy || !dirty} onClick={() => void action('save', async () => {
          const next = await adminRequest<DisplaySettings>('/display', 'PUT', draft)
          setDraft(next); setSaved(next); publishDisplaySettings(next); setMessage('展示配置已保存。用户刷新页面后即可看到新顺序和展示状态。')
        })}>{busy ? '正在保存…' : '保存展示配置'}</Button>
      </div>
    </div>
  </div>
}
