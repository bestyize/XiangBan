import { useEffect, useState } from 'react'
import { Link } from 'wouter'
import { api, ApiError, getToken } from '../../api/client.ts'
import type { Profile } from '../../api/types.ts'
import { FieldError, PrimaryButton, QuietButton, Screen } from '../../components/ui.tsx'
import { compressAvatar, uploadAvatar } from '../../lib/avatar.ts'
import { nicknameError, normalizeNickname } from '../../lib/nickname.ts'
import { useProfile, useSession } from '../../lib/session.tsx'

const input = 'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window focus:border-lamp focus:outline-none'
export default function AccountProfile() {
  const profile = useProfile()
  const { updateProfile, setProfile, refresh, isLoggedIn } = useSession()
  const [nickname, setNickname] = useState(profile.nickname ?? '')
  const [avatar, setAvatar] = useState<Blob | null>(null)
  const [preview, setPreview] = useState<string | null>(null)
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const invalidNickname = nicknameError(nickname)
  useEffect(() => {
    if (!avatar) { setPreview(null); return }
    const url = URL.createObjectURL(avatar); setPreview(url)
    return () => URL.revokeObjectURL(url)
  }, [avatar])
  async function action(name: string, job: () => Promise<void>) {
    setBusy(name); setError(null); setMessage(null)
    try {
      if (name === 'avatar' && !getToken()) {
        setError('请先登录或注册账号，再保存云端头像。')
        return
      }
      await job()
    } catch (e) {
      setError((e as Error).message)
      if (e instanceof ApiError && e.status === 401) void refresh().catch(() => undefined)
    } finally { setBusy(null) }
  }
  return <Screen><Link href="/me" className="mb-6 text-sm text-window/60">← 返回我的</Link>
    <h1 className="text-2xl font-bold text-window">昵称和头像</h1><p className="mt-2 text-xs text-window/55">用户 ID：{profile.userId} · {isLoggedIn ? '已登录' : '游客模式'}</p>
    <div className="mt-6 space-y-6">
      <FieldError message={error} />{message && <p role="status" className="text-sm text-lamp">{message}</p>}
      <section className="rounded-2xl bg-dusk-soft/35 p-5"><h2 className="font-bold">我的头像</h2><div className="mt-4 flex items-center gap-5">
        <div className="flex size-24 shrink-0 items-center justify-center overflow-hidden rounded-full bg-lamp/15 text-3xl text-lamp">{preview || profile.avatarUrl ? <img src={preview ?? profile.avatarUrl!} alt="头像预览" className="size-full object-cover" /> : '🐾'}</div>
        <label className="cursor-pointer rounded-full border border-lamp px-4 py-2 text-sm text-lamp">{busy === 'compress' ? '正在压缩…' : '选择图片'}<input type="file" accept="image/jpeg,image/png,image/webp,image/gif" className="sr-only" disabled={!!busy} onChange={(e) => {
          const file = e.target.files?.[0]; e.target.value = ''
          if (file) void action('compress', async () => { setAvatar(await compressAvatar(file)) })
        }} /></label>
      </div><p className="mt-4 text-xs leading-relaxed text-window/55">图片会在上传前裁成方形并压缩到 64KB 以内，只上传压缩后的头像。</p>
        {avatar && <p className="mt-2 text-xs text-lamp">压缩后 {(avatar.size / 1024).toFixed(1)}KB / 64KB</p>}
        <div className="mt-4 flex items-center gap-3"><PrimaryButton className="max-w-44" disabled={!!busy || !avatar} onClick={() => void action('avatar', async () => {
          if (!avatar) return; setProfile(await uploadAvatar(avatar)); setAvatar(null); setMessage('头像已保存，小伴认得你啦。')
        })}>{busy === 'avatar' ? '正在上传…' : '上传头像'}</PrimaryButton>
          {profile.avatarUrl && <QuietButton disabled={!!busy} onClick={() => void action('remove-avatar', async () => { setProfile(await api<Profile>('/api/account/avatar', { method: 'DELETE' })); setAvatar(null); setMessage('已恢复默认头像。') })}>恢复默认</QuietButton>}
        </div>
      </section>
      <form className="rounded-2xl bg-dusk-soft/35 p-5" onSubmit={(e) => { e.preventDefault(); if (!invalidNickname && !busy) void action('nickname', async () => {
        const saved = await updateProfile({ nickname: normalizeNickname(nickname) }); setNickname(saved.nickname ?? ''); setMessage('昵称已保存。')
      }) }}><label className="block"><span className="mb-3 block font-bold">我的昵称</span><input className={input} value={nickname} maxLength={24} disabled={!!busy} onChange={(e) => { setNickname(e.target.value); setMessage(null) }} placeholder={`留空则显示${isLoggedIn ? '用户' : '游客'} #${profile.userId}`} /></label>
        <p className="mt-3 text-xs leading-relaxed text-window/55">至少 2 个字符；纯英文或英文加数字至少 4 个字符；不能为纯数字，最多 24 个字符。昵称不可重复，大小写与全角写法也会判重。</p>
        <FieldError message={invalidNickname} /><PrimaryButton className="mt-5" type="submit" disabled={!!busy || !!invalidNickname || normalizeNickname(nickname) === (profile.nickname ?? '')}>{busy === 'nickname' ? '正在保存…' : '保存昵称'}</PrimaryButton>
      </form>
    </div>
  </Screen>
}
