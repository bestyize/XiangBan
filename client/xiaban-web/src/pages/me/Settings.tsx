import { useState } from 'react'
import { api, getToken, setToken } from '../../api/client.ts'
import { ButtonRow, InfoRow, LinkRow, ListGroup } from '../../components/List.tsx'
import { Sheet } from '../../components/Sheet.tsx'
import { SubPage } from '../../components/SubPage.tsx'
import { FieldError, QuietButton } from '../../components/ui.tsx'
import { useLoginModal } from '../../lib/loginModal.ts'
import { logout } from '../../lib/logout.ts'
import { useProfile, useSession } from '../../lib/session.tsx'
import { storage } from '../../lib/storage.ts'
import { THEME_OPTIONS, useTheme } from '../../lib/theme.tsx'

/** Account, privacy and data controls (PRD 6, 《个人信息保护法》), kept off the main 我的 screen. */
export default function Settings() {
  const profile = useProfile()
  const { isLoggedIn } = useSession()
  const { openLogin } = useLoginModal()
  const { preference, setPreference } = useTheme()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)

  async function run(job: () => Promise<void>) {
    setBusy(true)
    setError(null)
    try {
      await job()
    } catch (e) {
      setError((e as Error).message)
      setBusy(false)
    }
  }

  const exportData = () => run(async () => {
    const res = await fetch('/api/account/export', { headers: { Authorization: `Bearer ${getToken()}` } })
    if (!res.ok) throw new Error('导出失败，稍后再试')
    const url = URL.createObjectURL(await res.blob())
    const a = document.createElement('a')
    a.href = url
    a.download = 'xiaban-export.json'
    a.click()
    setTimeout(() => URL.revokeObjectURL(url), 10_000)
    setBusy(false)
  })

  const deleteAccount = () => run(async () => {
    await api('/api/account', { method: 'DELETE' })
    setToken(null)
    storage.remove('xb_device')
    location.replace('/')
  })

  return (
    <SubPage title="设置" back="/me">
      <ListGroup title="外观" footer="外观偏好保存在当前设备。跟随系统会自动切换深浅色。">
        <div role="radiogroup" aria-label="外观模式">
          {THEME_OPTIONS.map(option => <label key={option.value} className="app-row cursor-pointer">
            <span className="app-row-label">{option.label}</span>
            <input type="radio" name="appearance" value={option.value} checked={preference === option.value}
              className="size-4 accent-lamp" onChange={() => setPreference(option.value)} />
          </label>)}
        </div>
      </ListGroup>

      <ListGroup title="账号">
        {isLoggedIn ? (
          <InfoRow label="登录邮箱" detail={profile.email ?? '未绑定'} />
        ) : (
          <ButtonRow onClick={openLogin} label="登录 / 注册" tone="accent" />
        )}
        <LinkRow href="/me/account" label="昵称和头像" />
        {profile.userId > 0 && <InfoRow label="用户 ID" detail={<span className="selectable">{profile.userId}</span>} />}
      </ListGroup>

      <ListGroup title="隐私">
        <LinkRow href="/privacy" label="隐私政策" />
        <LinkRow href="/terms" label="用户协议" />
        {isLoggedIn && <ButtonRow label="导出我的数据" detail="JSON" onClick={exportData} disabled={busy} />}
      </ListGroup>

      {isLoggedIn && (
        <ListGroup>
          <ButtonRow label="退出登录" tone="accent" disabled={busy} onClick={() => run(() => logout('/me'))} />
        </ListGroup>
      )}
      {isLoggedIn && (
        <ListGroup>
          <ButtonRow label="注销账号" tone="danger" disabled={busy} onClick={() => setConfirmDelete(true)} />
        </ListGroup>
      )}
      <div className="mt-3"><FieldError message={error} /></div>

      <Sheet open={confirmDelete} onClose={() => setConfirmDelete(false)} title="注销账号">
        <p className="text-[15px] leading-relaxed text-window/85">注销后，打卡记录、图鉴、平安守护等所有数据都会删除，不能恢复。</p>
        <button type="button" disabled={busy} onClick={deleteAccount}
          className="mt-6 w-full rounded-full bg-action px-6 py-3.5 font-bold text-white transition active:scale-[.98] disabled:opacity-50">
          {busy ? '正在注销…' : '确认注销'}
        </button>
        <QuietButton className="mt-2 w-full" onClick={() => setConfirmDelete(false)}>再想想</QuietButton>
      </Sheet>
    </SubPage>
  )
}
