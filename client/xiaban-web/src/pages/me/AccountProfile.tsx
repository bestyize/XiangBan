import { useEffect, useState } from 'react'
import { api, ApiError, getToken } from '../../api/client.ts'
import type { Profile } from '../../api/types.ts'
import { SubPage } from '../../components/SubPage.tsx'
import { FieldError, PrimaryButton, QuietButton } from '../../components/ui.tsx'
import { compressAvatar, uploadAvatar } from '../../lib/avatar.ts'
import { useProfile, useSession } from '../../lib/session.tsx'

export default function AccountProfile() {
  const profile = useProfile()
  const { setProfile, refresh } = useSession()
  const [avatar, setAvatar] = useState<Blob | null>(null)
  const [preview, setPreview] = useState<string | null>(null)
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
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
  return <SubPage title="头像和用户名" back="/me">
    <div className="mt-2 space-y-6">
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
      <section className="rounded-2xl bg-dusk-soft/35 p-5"><h2 className="font-bold">我的用户名</h2>
        <p className="mt-3 text-lg">{profile.nickname ?? '登录后显示'}</p>
        <p className="mt-3 text-xs leading-relaxed text-window/55">用户名是你的唯一账号，注册时设置，不能修改，也不能与他人重复。</p>
      </section>
    </div>
  </SubPage>
}
