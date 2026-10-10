import { useEffect, useRef, useState } from 'react'
import { api } from '../../api/client.ts'
import { dropAccountSession, loginWithCode, sendLoginCode, type AccountCenterLogin } from '../../api/accountCenter.ts'
import type { BindResponse } from '../../api/types.ts'
import { FieldError, PrimaryButton } from '../../components/ui.tsx'
import { useProfile, useSession } from '../../lib/session.tsx'
import { normalizeNickname } from '../../lib/nickname.ts'
import { readLoginReturn } from '../../lib/loginReturn.ts'
import { importGuestAttempts } from '../../quizzes/progress.ts'
import { quizCatalog } from '../../quizzes/catalog.ts'
import { getDeviceId } from '../../lib/device.ts'

const input =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'

/** Passwordless login through the unified account center; an email that already has an account brings it back. */
export function EmailBind({ onDone, disabled = false, termsVersion, beforeSend }: {
  onDone?: (merged: boolean) => void
  disabled?: boolean
  termsVersion?: string
  beforeSend?: () => Promise<void>
}) {
  const { adopt } = useSession()
  const profile = useProfile()
  const [email, setEmail] = useState('')
  const [code, setCode] = useState('')
  const [sent, setSent] = useState(false)
  const [cooldown, setCooldown] = useState(0)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [mergeGuest, setMergeGuest] = useState(false)
  // The code is single-use, so keep the account center's answer until this site accepts it; a retry must not ask for a new code.
  const verified = useRef<AccountCenterLogin | null>(null)

  useEffect(() => {
    if (cooldown <= 0) return
    const id = setTimeout(() => setCooldown((s) => s - 1), 1000)
    return () => clearTimeout(id)
  }, [cooldown])

  async function sendCode() {
    setBusy(true)
    setError(null)
    try {
      await beforeSend?.()
      await sendLoginCode(email.trim())
      setSent(true)
      setCooldown(60)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  async function login() {
    setBusy(true)
    setError(null)
    try {
      verified.current ??= await loginWithCode(email.trim(), code)
      const accountCenter = verified.current
      const preferredUsername = profile.nickname ? normalizeNickname(profile.nickname) : undefined
      const res = await api<BindResponse>('/api/account/center/login', {
        method: 'POST',
        body: { accountToken: accountCenter.access_token, deviceId: getDeviceId(), mergeGuest, username: preferredUsername, termsVersion },
      })
      dropAccountSession(accountCenter.refresh_token)
      verified.current = null
      adopt(res.token, res.profile)
      const selected = readLoginReturn()?.path.split('/')[2]
      if (mergeGuest || selected) importGuestAttempts(quizCatalog.filter((quiz) => mergeGuest || quiz.id === selected), res.profile.userId)
      onDone?.(res.merged)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  return (
    <form className="space-y-3" onSubmit={(e) => { e.preventDefault(); if (sent && code.length === 6 && !busy && !disabled) void login() }}>
      <label className="block">
        <span className="mb-1.5 block text-sm text-window/70">邮箱</span>
        <input className={input} type="email" inputMode="email" autoComplete="email" value={email}
          disabled={busy} onChange={(e) => { setEmail(e.target.value); setSent(false); setCode(''); setError(null); verified.current = null }} placeholder="you@example.com" />
      </label>
      {sent && (
        <label className="block">
          <span className="mb-1.5 block text-sm text-window/70">邮件里的 6 位验证码</span>
          <input className={input} inputMode="numeric" autoComplete="one-time-code" maxLength={6} value={code}
            onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))} />
        </label>
      )}
      {sent && <p role="status" className="text-sm text-lamp">验证码已发出，5 分钟内有效。没收到时请检查垃圾邮件。</p>}
      <label className="flex gap-2 text-sm text-window/70">
        <input type="checkbox" checked={mergeGuest} disabled={busy} onChange={(e) => setMergeGuest(e.target.checked)} />
        同时将当前游客的打卡、游戏和安排记录合并到该账号
      </label>
      <FieldError message={error} />
      <div className="flex gap-3">
        <button type="button" disabled={busy || disabled || cooldown > 0 || !email.includes('@')} onClick={sendCode}
          className="shrink-0 rounded-full border-2 border-lamp px-4 py-3 text-sm font-bold text-lamp disabled:opacity-50">
          {cooldown > 0 ? `${cooldown} 秒后重发` : sent ? '重发验证码' : '发送验证码'}
        </button>
        {sent && <PrimaryButton type="submit" disabled={busy || disabled || code.length !== 6}>{busy ? '正在验证…' : '验证并进入'}</PrimaryButton>}
      </div>
    </form>
  )
}
