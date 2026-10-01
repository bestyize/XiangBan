import { useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { BindResponse } from '../../api/types.ts'
import { FieldError, PrimaryButton } from '../../components/ui.tsx'
import { useSession } from '../../lib/session.tsx'

const input =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'

/** Binding an email keeps the account across devices; an email that already has one brings it back. */
export function EmailBind({ onDone }: { onDone?: (merged: boolean) => void }) {
  const { adopt } = useSession()
  const [email, setEmail] = useState('')
  const [code, setCode] = useState('')
  const [sent, setSent] = useState(false)
  const [cooldown, setCooldown] = useState(0)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (cooldown <= 0) return
    const id = setTimeout(() => setCooldown((s) => s - 1), 1000)
    return () => clearTimeout(id)
  }, [cooldown])

  async function sendCode() {
    setBusy(true)
    setError(null)
    try {
      await api('/api/account/bind/code', { method: 'POST', body: { email } })
      setSent(true)
      setCooldown(60)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  async function bind() {
    setBusy(true)
    setError(null)
    try {
      const res = await api<BindResponse>('/api/account/bind', { method: 'POST', body: { email, code } })
      adopt(res.token, res.profile)
      onDone?.(res.merged)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="space-y-3">
      <label className="block">
        <span className="mb-1.5 block text-sm text-window/70">邮箱</span>
        <input className={input} type="email" inputMode="email" autoComplete="email" value={email}
          onChange={(e) => setEmail(e.target.value)} placeholder="you@example.com" />
      </label>
      {sent && (
        <label className="block">
          <span className="mb-1.5 block text-sm text-window/70">邮件里的 6 位验证码</span>
          <input className={input} inputMode="numeric" autoComplete="one-time-code" maxLength={6} value={code}
            onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))} />
        </label>
      )}
      <FieldError message={error} />
      <div className="flex gap-3">
        <button type="button" disabled={busy || cooldown > 0 || !email.includes('@')} onClick={sendCode}
          className="shrink-0 rounded-full border-2 border-lamp px-4 py-3 text-sm font-bold text-lamp disabled:opacity-50">
          {cooldown > 0 ? `${cooldown} 秒后重发` : sent ? '重发验证码' : '发送验证码'}
        </button>
        {sent && <PrimaryButton disabled={busy || code.length !== 6} onClick={bind}>绑定</PrimaryButton>}
      </div>
      <p className="text-xs leading-relaxed text-window/50">换手机或清了浏览器数据，用这个邮箱就能找回记录。这个邮箱已经绑定过账号的话，会把这台设备上的记录合并过去。</p>
    </div>
  )
}
