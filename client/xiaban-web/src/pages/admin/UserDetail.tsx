import { useEffect, useRef, useState, type ReactNode } from 'react'
import { adminRequest } from './api.ts'
import type { UserDetail as Detail } from './types.ts'
import { Alerts, Badge, Button, Field, formatTime, input, TextButton } from './ui.tsx'
import UserEditor from './UserEditor.tsx'
import type { UserDraft } from './user-edit.ts'

function Value({ label, children }: { label: string; children: ReactNode }) {
  return <div><dt className="text-xs text-slate-400">{label}</dt><dd className="mt-1.5 break-words text-sm text-slate-700">{children ?? '—'}</dd></div>
}
export default function UserDetail({ id, initialAction = 'view', onClose, onSaved, onDeleted }: {
  id: number | null; initialAction?: 'view' | 'edit' | 'delete'; onClose: () => void; onSaved: () => void; onDeleted: (id: number) => void
}) {
  const dialog = useRef<HTMLDialogElement>(null)
  const activeId = useRef<number | null>(id)
  const [detail, setDetail] = useState<Detail | null>(null)
  const [note, setNote] = useState('')
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const [editing, setEditing] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [deleteInput, setDeleteInput] = useState('')
  useEffect(() => {
    activeId.current = id
    const element = dialog.current
    if (id === null) { element?.close(); return }
    if (element && !element.open) element.showModal()
    const controller = new AbortController()
    setDetail(null); setLoading(true); setError(null); setMessage(null); setNote('')
    setEditing(initialAction === 'edit'); setConfirmDelete(initialAction === 'delete'); setDeleteInput('')
    adminRequest<Detail>(`/users/${id}`, 'GET', undefined, controller.signal).then((result) => { setDetail(result); setNote(result.note) })
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [id, initialAction])
  async function save() {
    if (id === null) return
    const selected = id
    setSaving(true); setError(null); setMessage(null)
    try {
      await adminRequest(`/users/${selected}/note`, 'PUT', { note })
      const updated = await adminRequest<Detail>(`/users/${selected}`)
      if (activeId.current === selected) { setDetail(updated); setNote(updated.note); setMessage('运营备注已保存。') }
      onSaved()
    } catch (e) { if (activeId.current === selected) setError((e as Error).message) } finally { setSaving(false) }
  }
  async function saveProfile(patch: Partial<UserDraft>) {
    if (id === null) return
    const selected = id
    setSaving(true); setError(null); setMessage(null)
    try {
      const updated = await adminRequest<Detail>(`/users/${selected}`, 'PUT', patch)
      if (activeId.current === selected) { setDetail(updated); setEditing(false); setMessage('用户资料已保存。') }
      onSaved()
    } catch (e) { if (activeId.current === selected) setError((e as Error).message) } finally { setSaving(false) }
  }
  async function deleteUser() {
    if (id === null || deleteInput.trim() !== String(id) || saving) return
    const selected = id
    setSaving(true); setError(null); setMessage(null)
    try {
      await adminRequest(`/users/${selected}`, 'DELETE', { confirmUserId: selected })
      onDeleted(selected)
    } catch (e) { if (activeId.current === selected) setError((e as Error).message) } finally { setSaving(false) }
  }
  return <dialog ref={dialog} aria-labelledby="admin-user-detail-title" style={{ colorScheme: 'light' }}
    className="fixed inset-y-0 right-0 left-auto m-0 h-dvh max-h-dvh w-full max-w-xl overflow-y-auto border-0 bg-white p-0 text-slate-800 shadow-2xl backdrop:bg-slate-950/35"
    onCancel={(e) => { if (saving) e.preventDefault(); else onClose() }} onClick={(e) => {
      if (saving) return
      if (e.target !== e.currentTarget) return
      const rect = e.currentTarget.getBoundingClientRect()
      if (e.clientX < rect.left || e.clientX > rect.right || e.clientY < rect.top || e.clientY > rect.bottom) onClose()
    }}>
    <header className="sticky top-0 z-10 flex items-center justify-between border-b border-slate-200 bg-white px-6 py-5"><div><h2 id="admin-user-detail-title" className="text-lg font-semibold">用户详情</h2><p className="mt-1 text-xs text-slate-400">用户 #{id}</p></div><TextButton aria-label="关闭用户详情" disabled={saving} onClick={onClose}>关闭 ✕</TextButton></header>
    <div className="space-y-7 px-6 py-6"><Alerts error={error} message={message} />{loading && <p className="py-12 text-center text-slate-400">正在加载用户信息…</p>}
      {detail && <><div><div className="flex flex-wrap items-center gap-3"><h3 className="text-xl font-semibold">{detail.user.nickname || `用户 #${detail.user.id}`}</h3><Badge tone={detail.user.email ? 'blue' : 'slate'}>{detail.user.email ? '邮箱账号' : '游客账号'}</Badge></div><p className="mt-2 break-all text-sm text-slate-500">{detail.user.email ?? '尚未绑定邮箱'}</p></div>
        {!editing && <Button disabled={saving} onClick={() => { setEditing(true); setError(null); setMessage(null) }}>编辑用户资料</Button>}
        {editing && <UserEditor detail={detail} busy={saving} onSave={saveProfile} onCancel={() => setEditing(false)} />}
        <section className="rounded-xl border border-red-200 bg-red-50 p-4"><h3 className="font-semibold text-red-800">删除用户</h3>
          <p className="mt-2 text-xs leading-relaxed text-red-700">删除后，该用户的打卡、图鉴、记忆、浏览器关联和守护设置会一并删除，操作不可恢复。</p>
          {!confirmDelete ? <button type="button" disabled={saving} className="mt-4 rounded-lg border border-red-300 px-3 py-2 text-sm text-red-700 disabled:opacity-40" onClick={() => { setConfirmDelete(true); setDeleteInput('') }}>删除此用户…</button> :
            <form className="mt-4 space-y-3" onSubmit={(e) => { e.preventDefault(); void deleteUser() }}>
              <Field title={`请输入用户编号 ${id} 确认删除`}><input className={input} autoComplete="off" inputMode="numeric" value={deleteInput} disabled={saving} onChange={(e) => setDeleteInput(e.target.value)} /></Field>
              <div className="flex gap-2"><TextButton disabled={saving} onClick={() => { setConfirmDelete(false); setDeleteInput('') }}>取消删除</TextButton>
                <button type="submit" className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white disabled:opacity-40" disabled={saving || deleteInput.trim() !== String(id)}>{saving ? '正在处理…' : '确认永久删除'}</button></div>
            </form>}
        </section>
        <section><h3 className="mb-4 font-semibold">注册与登录</h3><dl className="grid grid-cols-2 gap-x-5 gap-y-5">
          <Value label="账号创建时间">{formatTime(detail.user.createdAt)}</Value><Value label="邮箱验证时间">{formatTime(detail.user.emailVerifiedAt)}</Value>
          <Value label="最近活跃">{formatTime(detail.user.lastActiveAt)}</Value><Value label="引导完成时间">{formatTime(detail.user.onboardedAt)}</Value>
          <Value label="邀请码">{detail.inviteCode}</Value><Value label="注册来源">{detail.referralSource ?? '直接访问 / 未记录'}</Value><Value label="邀请人">{detail.referredBy ? `用户 #${detail.referredBy}` : '—'}</Value>
          <Value label="有效打卡 / 关联浏览器">{detail.user.checkinCount} 次 / {detail.user.deviceCount} 个</Value>
        </dl></section>
        <section className="border-t border-slate-100 pt-6"><h3 className="mb-4 font-semibold">用户设置</h3><dl className="grid grid-cols-2 gap-x-5 gap-y-5">
          <Value label="小伴名字">{detail.user.companionName}</Value><Value label="城市">{detail.cityName ?? '未设置'}</Value><Value label="时区">{detail.timezone}</Value><Value label="上下班时间">{detail.workStart}–{detail.workEnd}</Value>
          <Value label="小伴性格">{{ toxic: '毒舌心软', gentle: '温柔治愈', chatty: '元气话痨', tsundere: '高冷傲娇' }[detail.personality]}</Value><Value label="发薪日">{detail.payday ? `每月 ${detail.payday} 号` : '未设置'}</Value>
          <Value label="提醒邮件">{detail.recallOptIn ? '已订阅' : '未订阅'}</Value><Value label="平安守护">{detail.guardianPaused === null ? '未开启' : detail.guardianPaused ? '已暂停' : '已开启'}</Value>
          <Value label="协议版本">{detail.termsVersion ?? '未同意'}</Value><Value label="协议同意时间">{formatTime(detail.termsAcceptedAt)}</Value>
        </dl></section>
        <section className="border-t border-slate-100 pt-6"><h3 className="mb-4 font-semibold">关联浏览器</h3>{detail.devices.length ? <div className="space-y-2">{detail.devices.map((device, index) => <div key={`${device.createdAt}-${index}`} className="flex items-center justify-between gap-3 rounded-lg bg-slate-50 px-3 py-3 text-xs"><span className="text-slate-500">浏览器 {index + 1}<span className="mt-1 block text-slate-400">首次关联 {formatTime(device.createdAt)}</span></span><span className="text-right text-slate-500">最近打开<br />{formatTime(device.lastSeenAt)}</span></div>)}{detail.user.deviceCount > 20 && <p className="text-xs text-slate-400">展示最近使用的 20 个浏览器。</p>}</div> : <p className="text-sm text-slate-400">当前没有关联浏览器。</p>}</section>
        <section className="border-t border-slate-100 pt-6"><h3 className="mb-4 font-semibold">最近 10 次有效打卡</h3>{detail.recentCheckins.length ? <div className="space-y-3">{detail.recentCheckins.map((item) => <div key={item.workDay} className="flex items-center justify-between gap-3 text-sm"><span className="text-slate-500">{item.workDay}</span><span>{item.title}</span><span className="text-xs text-slate-400">{formatTime(item.checkedAt)}</span></div>)}</div> : <p className="text-sm text-slate-400">暂无有效打卡记录。</p>}</section>
        <section className="border-t border-slate-100 pt-6"><Field title="运营备注" hint="用于记录客服跟进与运营事项，不展示给用户。"><textarea className={`${input} min-h-32 resize-y`} maxLength={2000} value={note} disabled={saving} onChange={(e) => { setNote(e.target.value); setMessage(null) }} placeholder="例如：用户反馈收不到验证码，待确认垃圾邮件与邮箱服务商。" /></Field>
          <div className="mt-3 flex items-center justify-between gap-3"><span className="text-xs text-slate-400">{note.length} / 2000{note !== detail.note ? ' · 未保存' : ''}</span><Button disabled={saving || note === detail.note} onClick={() => void save()}>{saving ? '正在保存…' : '保存备注'}</Button></div>
          {detail.noteUpdatedAt && <p className="mt-3 text-xs text-slate-400">最近修改：{detail.noteUpdatedBy} · {formatTime(detail.noteUpdatedAt)}</p>}
        </section>
      </>}
    </div>
  </dialog>
}
