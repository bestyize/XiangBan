import { useEffect, useState } from 'react'
import { adminRequest as request } from './api.ts'
import type { EmailSettings } from './types.ts'
import { Alerts, Button as PrimaryButton, Field, input, panel, useAdminAction } from './ui.tsx'

export default function AdminEmail() {
  const [settings, setSettings] = useState<EmailSettings | null>(null)
  const [dirty, setDirty] = useState(false)
  const [testEmail, setTestEmail] = useState('')
  const { busy, error, message, setMessage, setError, action } = useAdminAction()
  useEffect(() => {
    const controller = new AbortController()
    request<EmailSettings>('/email', 'GET', undefined, controller.signal).then(setSettings)
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
    return () => controller.abort()
  }, [])
  function change<K extends keyof EmailSettings>(key: K, value: EmailSettings[K]) {
    if (!settings) return
    setSettings({ ...settings, [key]: value }); setDirty(true); setMessage(null)
  }
  return <><Alerts error={error} message={message} />{settings ?
<section className={panel}>
          <div className="flex flex-wrap items-center justify-between gap-3"><h2 className="text-lg font-semibold text-slate-900">邮箱与 SMTP 设置</h2>
            <span className="rounded-full bg-slate-100 px-3 py-1 text-xs text-slate-500">{dirty ? '有未保存修改' : settings.enabled && settings.passwordConfigured && settings.host ? '已保存配置' : '尚未启用'}</span></div>
          <p className="mt-2 text-sm text-slate-500">配置发信服务后，用户即可通过邮箱验证码注册、登录和保存记录。</p>
          <form className="mt-6 space-y-5" onSubmit={(e) => { e.preventDefault(); void action('save', async () => {
            setSettings(await request<EmailSettings>('/email', 'PUT', settings)); setDirty(false)
            setMessage('邮箱配置已保存，立即生效。请发送测试邮件并确认收件。')
          }) }}>
            <label className="flex items-center gap-3"><input type="checkbox" checked={settings.enabled} onChange={(e) => change('enabled', e.target.checked)} disabled={!!busy} />启用邮件服务与邮箱登录</label>
            <fieldset disabled={!!busy} className="grid gap-5 sm:grid-cols-2">
              <Field title="SMTP 主机"><input className={input} required={settings.enabled} value={settings.host} placeholder="smtp.example.com" onChange={(e) => change('host', e.target.value)} /></Field>
              <Field title="端口"><input className={input} required type="number" min={1} max={65535} value={settings.port} onChange={(e) => change('port', Number(e.target.value))} /></Field>
              <Field title="连接加密"><select className={input} value={settings.ssl ? 'ssl' : 'starttls'} onChange={(e) => change('ssl', e.target.value === 'ssl')}><option value="ssl">TLS（通常为 465）</option><option value="starttls">STARTTLS（通常为 587）</option></select></Field>
              <Field title="SMTP 登录用户名"><input className={input} required={settings.enabled} autoComplete="off" value={settings.user} onChange={(e) => change('user', e.target.value)} /></Field>
              <Field title="SMTP 授权码 / 密码" hint={settings.passwordConfigured ? '已有授权码，留空会保留。输入新值即可替换。' : '使用邮件服务提供的 SMTP 授权码。'}>
                <input className={input} type="password" autoComplete="new-password" required={settings.enabled && !settings.passwordConfigured} value={settings.password} onChange={(e) => change('password', e.target.value)} /></Field>
              <Field title="发件邮箱"><input className={input} type="email" required={settings.enabled} value={settings.from} onChange={(e) => change('from', e.target.value)} /></Field>
              <Field title="发件人名称"><input className={input} maxLength={80} value={settings.fromName} onChange={(e) => change('fromName', e.target.value)} /></Field>
            </fieldset>
            <div className="max-w-xs"><PrimaryButton type="submit" disabled={!!busy}>{busy === 'save' ? '正在保存…' : '保存邮箱配置'}</PrimaryButton></div>
          </form>
          <form className="mt-8 border-t border-slate-200 pt-6" onSubmit={(e) => { e.preventDefault(); void action('test', async () => {
            await request('/email/test', 'POST', { email: testEmail }); setMessage('SMTP 已接受测试邮件，请到测试邮箱确认收件；也请检查垃圾邮件。')
          }) }}>
            <h3 className="mb-3 font-bold">测试发信</h3>
            <div className="flex flex-col gap-3 sm:flex-row"><input aria-label="测试收件邮箱" className={input} type="email" required value={testEmail} onChange={(e) => setTestEmail(e.target.value)} placeholder="填写你自己的收件邮箱" />
              <button className="shrink-0 rounded-xl border border-amber-400 px-5 py-3 text-amber-700 disabled:opacity-40" type="submit" disabled={!!busy || dirty || !settings.enabled || !settings.passwordConfigured}>{busy === 'test' ? '正在发送…' : '发送测试邮件'}</button></div>
            {dirty && <p className="mt-2 text-xs text-slate-500">请先保存修改，再发送测试邮件。</p>}
          </form>
        </section>: <p className="py-8 text-slate-500">正在加载邮箱配置…</p>}</>
}
