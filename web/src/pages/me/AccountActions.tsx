import { useState } from 'react'
import { api, getToken, setToken } from '../../api/client.ts'
import { FieldError, QuietButton } from '../../components/ui.tsx'
import { storage } from '../../lib/storage.ts'

/** 导出数据 and 注销账号 (PRD 6, 《个人信息保护法》). */
export function AccountActions() {
  const [confirming, setConfirming] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function exportData() {
    setError(null)
    try {
      const res = await fetch('/api/account/export', { headers: { Authorization: `Bearer ${getToken()}` } })
      if (!res.ok) throw new Error('导出失败，稍后再试')
      const url = URL.createObjectURL(await res.blob())
      const a = document.createElement('a')
      a.href = url
      a.download = 'xiaban-export.json'
      a.click()
      setTimeout(() => URL.revokeObjectURL(url), 10_000)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function deleteAccount() {
    setBusy(true)
    setError(null)
    try {
      await api('/api/account', { method: 'DELETE' })
      setToken(null)
      storage.remove('xb_device')
      location.replace('/')
    } catch (e) {
      setError((e as Error).message)
      setBusy(false)
    }
  }

  return (
    <div className="space-y-1">
      <QuietButton className="px-0!" onClick={exportData}>导出我的数据（JSON）</QuietButton>
      {!confirming ? (
        <div><QuietButton className="px-0! text-blush!" onClick={() => setConfirming(true)}>注销账号</QuietButton></div>
      ) : (
        <div role="alertdialog" aria-label="确认注销" className="rounded-2xl border border-blush/60 p-4 text-sm text-window/85">
          <p>注销后，打卡记录、图鉴、平安守护等所有数据都会删除，不能恢复。</p>
          <div className="mt-3 flex gap-2">
            <button type="button" disabled={busy} onClick={deleteAccount} className="rounded-full bg-blush px-4 py-2 font-bold text-ink disabled:opacity-50">确认注销</button>
            <QuietButton onClick={() => setConfirming(false)}>再想想</QuietButton>
          </div>
        </div>
      )}
      <FieldError message={error} />
    </div>
  )
}
