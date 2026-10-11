import { useEffect, useState } from 'react'
import type { Meta } from '../../api/types.ts'
import type { UserDetail } from './types.ts'
import { userDraft, userPatch, type UserDraft } from './user-edit.ts'
import { Alerts, Button, Field, input, TextButton } from './ui.tsx'

export default function UserEditor({ detail, busy, onSave, onCancel }: {
  detail: UserDetail; busy: boolean; onSave: (patch: Partial<UserDraft>) => Promise<void>; onCancel: () => void
}) {
  const [draft, setDraft] = useState(() => userDraft(detail))
  const [meta, setMeta] = useState<Meta | null>(null)
  const [error, setError] = useState<string | null>(null)
  const patch = userPatch(draft, detail)
  useEffect(() => {
    const controller = new AbortController()
    fetch('/api/meta', { signal: controller.signal }).then(async (response) => {
      if (!response.ok) throw new Error('城市选项加载失败，请稍后重试')
      setMeta(await response.json() as Meta)
    }).catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
    return () => controller.abort()
  }, [])
  function change<K extends keyof UserDraft>(key: K, value: UserDraft[K]) { setDraft({ ...draft, [key]: value }) }
  return <form className="rounded-xl border border-slate-200 bg-slate-50 p-4" onSubmit={(e) => {
    e.preventDefault(); if (Object.keys(patch).length && !busy) void onSave(patch)
  }}>
    <h3 className="mb-4 font-semibold">编辑用户资料</h3><Alerts error={error} />
    <fieldset className="grid grid-cols-2 gap-4" disabled={busy}>
      <Field title="小伴名字（2–8 字）"><input className={input} required minLength={2} maxLength={8} value={draft.companionName} onChange={(e) => change('companionName', e.target.value)} /></Field>
      <Field title="小伴性格"><select className={input} value={draft.personality} onChange={(e) => change('personality', e.target.value as UserDraft['personality'])}>
        <option value="toxic">毒舌心软</option><option value="gentle">温柔治愈</option><option value="chatty">元气话痨</option><option value="tsundere">高冷傲娇</option>
      </select></Field>
      <Field title="城市"><select className={input} value={draft.cityCode} disabled={!meta || busy} onChange={(e) => change('cityCode', e.target.value)}>
        <option value="">未设置</option>{!meta && draft.cityCode && <option value={draft.cityCode}>{detail.cityName ?? draft.cityCode}</option>}
        {meta?.cities.map((city) => <option key={city.code} value={city.code}>{city.province} · {city.name}</option>)}
      </select></Field>
      <Field title="上班时间"><input className={input} type="time" required value={draft.workStart} onChange={(e) => change('workStart', e.target.value)} /></Field>
      <Field title="下班时间"><input className={input} type="time" required value={draft.workEnd} onChange={(e) => change('workEnd', e.target.value)} /></Field>
      <Field title="时区"><input className={input} required maxLength={64} value={draft.timezone} onChange={(e) => change('timezone', e.target.value)} placeholder="Asia/Shanghai" /></Field>
      <Field title="发薪日"><select className={input} value={draft.payday} onChange={(e) => change('payday', Number(e.target.value))}><option value={0}>未设置</option>
        {Array.from({ length: 31 }, (_, i) => <option key={i + 1} value={i + 1}>每月 {i + 1} 号</option>)}
      </select></Field>
    </fieldset><div className="mt-5 flex items-center justify-end gap-2"><TextButton disabled={busy} onClick={onCancel}>取消编辑</TextButton><Button type="submit" disabled={busy || !Object.keys(patch).length}>{busy ? '正在保存…' : '保存资料'}</Button></div>
  </form>
}
