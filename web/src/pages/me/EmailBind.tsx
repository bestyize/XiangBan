import { useEffect, useState } from 'react'
import { api, ApiError } from '../../api/client.ts'
import type { BindResponse } from '../../api/types.ts'
import { FieldError, PrimaryButton } from '../../components/ui.tsx'
import { useProfile, useSession } from '../../lib/session.tsx'
import { nicknameError, normalizeNickname } from '../../lib/nickname.ts'
import { getDeviceId } from '../../lib/device.ts'

const input =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'

/** Binding an email keeps the account across devices; an email that already has one brings it back. */
export function EmailBind({ onDone, mode = 'bind', disabled = false, beforeSend }: {
  onDone?: (merged: boolean) => void
  mode?: 'bind' | 'login'
  disabled?: boolean
  beforeSend?: () => Promise<void>
}) {
  const { adopt } = useSession()
  const profile = useProfile()
  const [registering, setRegistering] = useState(mode === 'bind')
  const [username, setUsername] = useState(profile.nickname ?? '')
  const usernameError = registering ? normalizeNickname(username) ? nicknameError(username) : '注册时用户名必填' : null
  const [email, setEmail] = useState('')
  const [code, setCode] = useState('')
  const [sent, setSent] = useState(false)
  const [cooldown, setCooldown] = useState(0)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [mergeGuest, setMergeGuest] = useState(false)
  const [available, setAvailable] = useState<boolean | null>(null)

  useEffect(() => {
    let alive = true
    const refresh = () => {
      api<{ enabled: boolean; configured: boolean }>('/api/account/email/status')
        .then((s) => { if (alive) setAvailable(s.enabled && s.configured) })
        .catch(() => { if (alive) setAvailable(null) })
    }
    refresh()
    window.addEventListener('focus', refresh)
    return () => { alive = false; window.removeEventListener('focus', refresh) }
  }, [])

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
      await api(mode === 'login' ? '/api/account/login/code' : '/api/account/bind/code', { method: 'POST', body: { email } })
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
      const res = await api<BindResponse>(mode === 'login' ? '/api/account/login' : '/api/account/bind', {
        method: 'POST', body: mode === 'login' ? { email, code, deviceId: getDeviceId(), mergeGuest, username: registering ? normalizeNickname(username) : undefined } : { email, code, username: normalizeNickname(username) },
      })
      adopt(res.token, res.profile)
      onDone?.(res.merged)
    } catch (e) {
      if (e instanceof ApiError && e.code === 'username_required') setRegistering(true)
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  return (
    <form className="space-y-3" onSubmit={(e) => { e.preventDefault(); if (sent && code.length === 6 && !busy && !disabled && !usernameError) void bind() }}>
      {mode === 'login' && <div className="flex gap-2" aria-label="登录或注册"><button type="button" disabled={busy} className={`flex-1 rounded-full px-4 py-2 text-sm ${!registering ? 'bg-lamp/20 text-lamp' : 'text-window/60'}`} onClick={() => { setRegistering(false); setError(null) }}>已有账号登录</button>
        <button type="button" disabled={busy} className={`flex-1 rounded-full px-4 py-2 text-sm ${registering ? 'bg-lamp/20 text-lamp' : 'text-window/60'}`} onClick={() => { setRegistering(true); setError(null) }}>新用户注册</button></div>}
      {registering && <label className="block"><span className="mb-1.5 block text-sm text-window/70">用户名 <span className="text-blush">*</span></span>
        <input className={input} required maxLength={24} autoComplete="nickname" value={username} disabled={busy} onChange={(e) => setUsername(e.target.value)} placeholder="注册必填，请给自己取个名字" />
        <span className="mt-2 block text-xs leading-relaxed text-window/50">用户名就是你的昵称，注册后可修改。至少 2 个字符；英文或英文加数字至少 4 个；不能是纯数字或与他人重复。</span>
        {username && <FieldError message={usernameError} />}</label>}
      <label className="block">
        <span className="mb-1.5 block text-sm text-window/70">邮箱</span>
        <input className={input} type="email" inputMode="email" autoComplete="email" value={email}
          disabled={busy} onChange={(e) => { setEmail(e.target.value); setSent(false); setCode(''); setError(null) }} placeholder="you@example.com" />
      </label>
      {sent && (
        <label className="block">
          <span className="mb-1.5 block text-sm text-window/70">邮件里的 6 位验证码</span>
          <input className={input} inputMode="numeric" autoComplete="one-time-code" maxLength={6} value={code}
            onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))} />
        </label>
      )}
      {sent && <p role="status" className="text-sm text-lamp">验证码已发出，10 分钟内有效。没收到时请检查垃圾邮件。</p>}
      {mode === 'login' && <label className="flex gap-2 text-sm text-window/70">
        <input type="checkbox" checked={mergeGuest} disabled={busy} onChange={(e) => setMergeGuest(e.target.checked)} />
        同时将当前游客的打卡、游戏和安排记录合并到该邮箱账号
      </label>}
      {available === false && <p role="status" className="text-sm text-window/60">邮箱服务暂未开通，你可以先以游客身份体验。</p>}
      <FieldError message={error} />
      <div className="flex gap-3">
        <button type="button" disabled={busy || disabled || available === false || cooldown > 0 || !email.includes('@')} onClick={sendCode}
          className="shrink-0 rounded-full border-2 border-lamp px-4 py-3 text-sm font-bold text-lamp disabled:opacity-50">
          {cooldown > 0 ? `${cooldown} 秒后重发` : sent ? '重发验证码' : '发送验证码'}
        </button>
        {sent && <PrimaryButton type="submit" disabled={busy || disabled || code.length !== 6 || !!usernameError}>{busy ? '正在验证…' : registering ? '验证并注册' : '验证并登录'}</PrimaryButton>}
      </div>
      <p className="text-xs leading-relaxed text-window/50">{mode === 'login'
        ? '无需密码。新邮箱验证后注册，已有邮箱验证后找回账号。合并记录后将保留邮箱账号的原有设置。'
        : '绑定邮箱后，换设备也能找回记录。若邮箱已有账号，验证后会将本机游客记录合并过去，保留原账号设置。'}</p>
    </form>
  )
}
