import { useState } from 'react'
import { adminRequest as request } from './api.ts'
import { Alerts, Field, input, panel, useAdminAction } from './ui.tsx'

export default function AdminSecurity({ onChanged }: { onChanged: () => void }) {
  const [oldPassword, setOldPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmation, setConfirmation] = useState('')
  const { busy, error, message, action } = useAdminAction()
  return <><Alerts error={error} message={message} />
<section className={panel}>
          <h2 className="text-lg font-semibold text-slate-900">管理员密码</h2>
          <form className="mt-5 space-y-5" onSubmit={(e) => { e.preventDefault(); void action('password', async () => {
            if (newPassword !== confirmation) throw new Error('两次输入的新密码不一致')
            await request('/password', 'PUT', { oldPassword, newPassword }); setOldPassword(''); setNewPassword(''); setConfirmation('')
            onChanged()
          }) }}>
            <fieldset disabled={!!busy} className="grid gap-5 sm:grid-cols-3">
              <Field title="当前密码"><input className={input} type="password" autoComplete="current-password" required maxLength={256} value={oldPassword} onChange={(e) => setOldPassword(e.target.value)} /></Field>
              <Field title="新密码（至少 6 位）"><input className={input} type="password" autoComplete="new-password" required minLength={6} maxLength={256} value={newPassword} onChange={(e) => setNewPassword(e.target.value)} /></Field>
              <Field title="确认新密码"><input className={input} type="password" autoComplete="new-password" required minLength={6} maxLength={256} value={confirmation} onChange={(e) => setConfirmation(e.target.value)} /></Field>
            </fieldset>
            <button type="submit" disabled={!!busy} className="rounded-xl border border-slate-300 px-5 py-3 text-sm disabled:opacity-40">{busy === 'password' ? '正在修改…' : '修改密码并重新登录'}</button>
          </form>
        </section></>
}
