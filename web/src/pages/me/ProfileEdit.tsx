import { useState } from 'react'
import { useLocation } from 'wouter'
import type { Personality, ProfileUpdate } from '../../api/types.ts'
import { CityPicker } from '../../components/CityPicker.tsx'
import { FieldError, PrimaryButton, QuietButton, Screen } from '../../components/ui.tsx'
import { useMeta } from '../../lib/meta.ts'
import { useProfile, useSession } from '../../lib/session.tsx'

const input =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'

/** 小伴档案 and the settings the titles and tips depend on. */
export default function ProfileEdit() {
  const profile = useProfile()
  const { updateProfile } = useSession()
  const meta = useMeta()
  const [, navigate] = useLocation()
  const [draft, setDraft] = useState({
    companionName: profile.companionName,
    personality: profile.personality as Personality,
    nickname: profile.nickname ?? '',
    cityCode: profile.cityCode,
    workStart: profile.workStart,
    workEnd: profile.workEnd,
    payday: profile.payday ?? 0,
  })
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const set = (patch: Partial<typeof draft>) => setDraft((d) => ({ ...d, ...patch }))

  async function save() {
    setBusy(true)
    setError(null)
    const patch: ProfileUpdate = {
      companionName: draft.companionName.trim(), personality: draft.personality, nickname: draft.nickname.trim(),
      workStart: draft.workStart, workEnd: draft.workEnd, payday: draft.payday,
      ...(draft.cityCode ? { cityCode: draft.cityCode } : {}),
    }
    try {
      await updateProfile(patch)
      navigate('/me')
    } catch (e) {
      setError((e as Error).message)
      setBusy(false)
    }
  }

  return (
    <Screen>
      <header className="flex h-10 items-center">
        <QuietButton className="px-0!" onClick={() => navigate('/me')}>返回</QuietButton>
      </header>
      <h1 className="mt-2 text-2xl font-bold text-window">小伴档案和作息</h1>
      <div className="mt-5 space-y-5 pb-8">
        <div className="grid grid-cols-2 gap-3">
          <label className="block"><span className="mb-1.5 block text-sm text-window/70">小伴的名字</span>
            <input className={input} value={draft.companionName} maxLength={8} onChange={(e) => set({ companionName: e.target.value })} /></label>
          <label className="block"><span className="mb-1.5 block text-sm text-window/70">小伴怎么叫你</span>
            <input className={input} value={draft.nickname} maxLength={24} placeholder="昵称至少 2 字，英文至少 4 字" onChange={(e) => set({ nickname: e.target.value })} /></label>
        </div>
        <fieldset>
          <legend className="mb-1.5 text-sm text-window/70">性格</legend>
          <div className="grid grid-cols-2 gap-2">
            {(meta?.personalities ?? []).map((p) => (
              <button key={p.code} type="button" aria-pressed={draft.personality === p.code} onClick={() => set({ personality: p.code })}
                className={`rounded-2xl border-2 px-3 py-2.5 text-left ${draft.personality === p.code ? 'border-lamp bg-lamp/15' : 'border-dusk-soft'}`}>
                <span className="block font-bold text-window">{p.name}</span>
                <span className="block text-xs text-window/60">{p.desc}</span>
              </button>
            ))}
          </div>
        </fieldset>
        <div className="grid grid-cols-3 gap-3">
          <label className="block"><span className="mb-1.5 block text-sm text-window/70">上班</span>
            <input type="time" className={input} value={draft.workStart} onChange={(e) => set({ workStart: e.target.value })} /></label>
          <label className="block"><span className="mb-1.5 block text-sm text-window/70">正常下班</span>
            <input type="time" className={input} value={draft.workEnd} onChange={(e) => set({ workEnd: e.target.value })} /></label>
          <label className="block"><span className="mb-1.5 block text-sm text-window/70">发薪日</span>
            <select className={input} value={draft.payday} onChange={(e) => set({ payday: Number(e.target.value) })}>
              <option value={0}>不设置</option>
              {Array.from({ length: 31 }, (_, i) => <option key={i + 1} value={i + 1}>每月 {i + 1} 号</option>)}
            </select></label>
        </div>
        {meta && (
          <div>
            <span className="mb-1.5 block text-sm text-window/70">城市</span>
            <CityPicker cities={meta.cities} value={draft.cityCode} onChange={(cityCode) => set({ cityCode })} />
          </div>
        )}
        <FieldError message={error} />
        <PrimaryButton disabled={busy} onClick={save}>保存</PrimaryButton>
      </div>
    </Screen>
  )
}
