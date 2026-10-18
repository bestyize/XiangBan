import { useEffect, useRef, useState, type FormEvent } from 'react'
import { api, ApiError } from '../api/client.ts'
import * as account from '../api/accountCenter.ts'
import type { AccountSession } from '../api/accountCenter.ts'
import type { BindResponse } from '../api/types.ts'
import { getDeviceId } from '../lib/device.ts'
import { readLoginReturn } from '../lib/loginReturn.ts'
import { useSession } from '../lib/session.tsx'
import { passwordError, usernameError } from '../lib/username.ts'
import { importGuestAttempts } from '../quizzes/progress.ts'
import { quizCatalog } from '../quizzes/catalog.ts'
import { FieldError, PrimaryButton } from './ui.tsx'

const input =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'
const secondary =
  'shrink-0 rounded-full border-2 border-lamp px-4 py-3 text-sm font-bold text-lamp disabled:opacity-50'
const link = 'text-sm text-lamp underline-offset-2 hover:underline disabled:opacity-50'

type Mode = 'password' | 'code' | 'reset'
type Step = 'identify' | 'verify' | 'setup'
type Availability = { kind: 'idle' } | { kind: 'checking' } | { kind: 'ok' } | { kind: 'taken' | 'invalid'; message: string }

/**
 * 登录 / 注册：已有账号用「用户名 / 邮箱 / 手机号 + 密码」；新用户用邮箱验证码验证后设置用户名和密码。
 * 忘记密码走验证码重置。账号中心的会话只用来换取本站令牌，换完即作废。
 */
export function LoginForm({ onDone, disabled = false, termsVersion, beforeSend }: {
  onDone?: (merged: boolean) => void
  disabled?: boolean
  termsVersion?: string
  beforeSend?: () => Promise<void>
}) {
  const { adopt } = useSession()
  const [mode, setMode] = useState<Mode>('password')
  const [step, setStep] = useState<Step>('identify')
  const [identifier, setIdentifier] = useState('')
  const [password, setPassword] = useState('')
  const [code, setCode] = useState('')
  const [username, setUsername] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [mergeGuest, setMergeGuest] = useState(false)
  const [cooldown, setCooldown] = useState(0)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [availability, setAvailability] = useState<Availability>({ kind: 'idle' })
  const [methods, setMethods] = useState<account.SignupMethods>({ email: true, phone: false })
  const registrationToken = useRef<string | null>(null)
  // 账号中心已经认可、但本站还没接受的登录结果：验证码和注册凭证只能用一次，重试换票时不能再要新的。
  const verified = useRef<AccountSession | null>(null)

  useEffect(() => {
    void account.getSignupMethods().then(setMethods)
  }, [])

  useEffect(() => {
    if (cooldown <= 0) return
    const id = setTimeout(() => setCooldown((s) => s - 1), 1000)
    return () => clearTimeout(id)
  }, [cooldown])

  // 设置账号时边输入边查重，只是提示；真正的唯一性由服务端保证。
  useEffect(() => {
    if (step !== 'setup') return
    const name = username.trim()
    if (!name) { setAvailability({ kind: 'idle' }); return }
    const local = usernameError(name)
    if (local) { setAvailability({ kind: 'invalid', message: local }); return }
    setAvailability({ kind: 'checking' })
    let alive = true
    const timer = setTimeout(() => {
      account.checkUsername(name)
        .then((r) => { if (alive) setAvailability(r.available ? { kind: 'ok' } : { kind: 'taken', message: r.message ?? '这个用户名已被占用' }) })
        .catch(() => { if (alive) setAvailability({ kind: 'idle' }) })
    }, 350)
    return () => { alive = false; clearTimeout(timer) }
  }, [username, step])

  function changeIdentifier() {
    setStep('identify')
    setCode('')
    setPassword('')
    setError(null)
    setNotice(null)
    setCooldown(0)
    verified.current = null
  }

  function switchMode(next: Mode) {
    setMode(next)
    setStep('identify')
    setCode('')
    setPassword('')
    setError(null)
    setNotice(null)
    registrationToken.current = null
    verified.current = null
  }

  async function run(action: () => Promise<void>) {
    setBusy(true)
    setError(null)
    try {
      await action()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  /** 账号中心认可了身份：换成本站令牌并进入登录态。 */
  async function exchange(session: AccountSession) {
    verified.current = session
    const res = await api<BindResponse>('/api/account/center/login', {
      method: 'POST',
      body: { accountToken: session.access_token, deviceId: getDeviceId(), mergeGuest, termsVersion },
    })
    account.dropAccountSession(session.refresh_token)
    verified.current = null
    adopt(res.token, res.profile)
    const selected = readLoginReturn()?.path.split('/')[2]
    if (mergeGuest || selected) importGuestAttempts(quizCatalog.filter((quiz) => mergeGuest || quiz.id === selected), res.profile.userId)
    onDone?.(res.merged)
  }

  const sendCode = () => run(async () => {
    if (mode === 'code' && !methods.phone && !identifier.includes('@')) throw new Error('暂未开放手机号注册，请使用邮箱')
    await beforeSend?.()
    await (mode === 'reset' ? account.sendResetCode(identifier.trim()) : account.sendLoginCode(identifier.trim()))
    setStep('verify')
    setCooldown(60)
  })

  const submitPassword = () => run(async () => {
    await exchange(verified.current ?? await account.loginWithPassword(identifier.trim(), password))
  })

  const submitCode = () => run(async () => {
    if (verified.current) return exchange(verified.current)
    const result = await account.loginWithCode(identifier.trim(), code)
    if (account.needsRegistration(result)) {
      registrationToken.current = result.registration_token
      setStep('setup')
      return
    }
    await exchange(result)
  })

  const submitSetup = () => run(async () => {
    if (verified.current) return exchange(verified.current)
    const token = registrationToken.current
    if (!token) { setStep('identify'); throw new Error('验证已过期，请重新获取验证码') }
    try {
      await exchange(await account.register(token, username.trim(), password))
    } catch (e) {
      if (e instanceof ApiError && e.code === 'username_taken') setAvailability({ kind: 'taken', message: e.message })
      if (e instanceof ApiError && (e.code === 'registration_expired' || e.code === 'identity_exists')) {
        registrationToken.current = null
        setStep('identify')
        setCode('')
      }
      throw e
    }
  })

  const submitReset = () => run(async () => {
    await account.resetPassword(identifier.trim(), code, password)
    switchMode('password')
    setNotice('密码已重置，请用新密码登录。')
  })

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    if (busy || disabled || !canSubmit) return
    if (mode === 'password') void submitPassword()
    else if (mode === 'reset') void submitReset()
    else if (step === 'setup') void submitSetup()
    else void submitCode()
  }

  const setupInvalid = !!usernameError(username) || availability.kind === 'taken' || availability.kind === 'checking' || !username.trim()
  const canSubmit =
    mode === 'password' ? !!identifier.trim() && !!password
      : mode === 'reset' ? step === 'verify' && code.length === 6 && !passwordError(password) && !!password
        : step === 'setup' ? !setupInvalid && !!password && !passwordError(password, username)
          : step === 'verify' && code.length === 6

  const codeSent = step === 'verify'
  const canSend = !busy && !disabled && cooldown <= 0 && !!identifier.trim()

  return (
    <form className="space-y-3" onSubmit={onSubmit}>
      {step !== 'setup' && (
        <div className="flex gap-2 rounded-full bg-dusk-soft/40 p-1" role="tablist">
          <button type="button" role="tab" aria-selected={mode !== 'code'} onClick={() => switchMode('password')}
            className={`flex-1 rounded-full py-2 text-sm font-bold ${mode !== 'code' ? 'bg-lamp text-white' : 'text-window/70'}`}>密码登录</button>
          <button type="button" role="tab" aria-selected={mode === 'code'} onClick={() => switchMode('code')}
            className={`flex-1 rounded-full py-2 text-sm font-bold ${mode === 'code' ? 'bg-lamp text-white' : 'text-window/70'}`}>验证码登录 / 注册</button>
        </div>
      )}

      {mode === 'reset' && step !== 'setup' && (
        <p className="text-sm text-window/70">重置密码：先验证邮箱或手机号，再设置新密码。</p>
      )}

      {step === 'setup' ? (
        <>
          <p role="status" className="text-sm text-window/70">验证通过，设置你的账号。用户名是你的唯一账号，设置后不能修改。</p>
          <label className="block">
            <span className="mb-1.5 block text-sm text-window/70">用户名</span>
            <input className={input} autoComplete="username" autoCapitalize="none" spellCheck={false} maxLength={20}
              value={username} disabled={busy} placeholder="中英文、数字、下划线，至少 2 个汉字或 3 个字母"
              onChange={(e) => { setUsername(e.target.value); setError(null) }} />
            <UsernameHint availability={availability} />
          </label>
        </>
      ) : (
        <label className="block">
          <span className="mb-1.5 block text-sm text-window/70">{mode === 'password' ? '用户名 / 邮箱 / 手机号' : mode === 'code' && !methods.phone ? '邮箱' : '邮箱或手机号'}</span>
          <input className={input} autoComplete="username" autoCapitalize="none" spellCheck={false} value={identifier} disabled={busy || codeSent}
            placeholder={mode === 'password' ? '用户名、邮箱或手机号' : 'you@example.com'}
            onChange={(e) => { setIdentifier(e.target.value); setError(null) }} />
          {codeSent && (
            <button type="button" className={`${link} mt-1.5`} disabled={busy} onClick={changeIdentifier}>填错了？换一个{mode === 'code' && !methods.phone ? '邮箱' : '邮箱 / 手机号'}</button>
          )}
        </label>
      )}

      {mode === 'password' && (
        <PasswordField label="密码" value={password} onChange={setPassword} show={showPassword} onToggle={setShowPassword} busy={busy} autoComplete="current-password" />
      )}

      {codeSent && mode !== 'password' && (
        <label className="block">
          <span className="mb-1.5 block text-sm text-window/70">收到的 6 位验证码</span>
          <input className={input} inputMode="numeric" autoComplete="one-time-code" maxLength={6} value={code} disabled={busy}
            onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))} />
        </label>
      )}

      {(mode === 'reset' ? codeSent : step === 'setup') && (
        <PasswordField label={mode === 'reset' ? '新密码' : '设置密码'} value={password} onChange={setPassword} show={showPassword} onToggle={setShowPassword}
          busy={busy} autoComplete="new-password" error={passwordError(password, mode === 'reset' ? '' : username)} />
      )}

      {codeSent && <p role="status" className="text-sm text-window/70">验证码已发出，5 分钟内有效。没收到时请检查垃圾邮件。</p>}
      {notice && <p role="status" className="text-sm text-window/70">{notice}</p>}

      {step !== 'setup' && mode !== 'reset' && (
        <label className="flex gap-2 text-sm text-window/70">
          <input type="checkbox" checked={mergeGuest} disabled={busy} onChange={(e) => setMergeGuest(e.target.checked)} />
          同时将当前游客的打卡、游戏和安排记录合并到该账号
        </label>
      )}

      <FieldError message={error} />

      <div className="flex flex-wrap items-center gap-3">
        {mode !== 'password' && step !== 'setup' && (
          <button type="button" className={secondary} disabled={!canSend} onClick={() => void sendCode()}>
            {cooldown > 0 ? `${cooldown} 秒后重发` : codeSent ? '重发验证码' : '发送验证码'}
          </button>
        )}
        {(mode === 'password' || codeSent || step === 'setup') && (
          <PrimaryButton type="submit" disabled={busy || disabled || !canSubmit}>
            {busy ? '请稍候…' : mode === 'password' ? '登录' : mode === 'reset' ? '重置密码' : step === 'setup' ? '完成注册' : '验证并继续'}
          </PrimaryButton>
        )}
      </div>

      {mode === 'password' && (
        <button type="button" className={link} onClick={() => switchMode('reset')}>忘记密码？</button>
      )}
      {mode === 'reset' && (
        <button type="button" className={link} onClick={() => switchMode('password')}>返回密码登录</button>
      )}
    </form>
  )
}

function PasswordField({ label, value, onChange, show, onToggle, busy, autoComplete, error }: {
  label: string
  value: string
  onChange: (value: string) => void
  show: boolean
  onToggle: (show: boolean) => void
  busy: boolean
  autoComplete: string
  error?: string | null
}) {
  return (
    <label className="block">
      <span className="mb-1.5 block text-sm text-window/70">{label}</span>
      <div className="relative">
        <input className={`${input} pr-16`} type={show ? 'text' : 'password'} autoComplete={autoComplete} value={value} disabled={busy}
          placeholder="至少 8 位" onChange={(e) => onChange(e.target.value)} />
        <button type="button" className="absolute inset-y-0 right-4 text-sm text-lamp" onClick={() => onToggle(!show)}>{show ? '隐藏' : '显示'}</button>
      </div>
      {error && <span role="alert" className="mt-1 block text-sm text-lamp">{error}</span>}
    </label>
  )
}

function UsernameHint({ availability }: { availability: Availability }) {
  if (availability.kind === 'idle') return null
  const text = availability.kind === 'checking' ? '正在检查…' : availability.kind === 'ok' ? '可以使用' : availability.message
  const problem = availability.kind === 'taken' || availability.kind === 'invalid'
  return <span role="status" className={`mt-1 block text-sm ${problem ? 'text-lamp' : 'text-window/70'}`}>{text}</span>
}
