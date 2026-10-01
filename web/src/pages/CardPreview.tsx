import { useEffect, useMemo, useState } from 'react'
import { useLocation } from 'wouter'
import { api } from '../api/client.ts'
import type { Checkin, TodayResponse } from '../api/types.ts'
import { renderCard } from '../card/renderCard.ts'
import { cardStyles } from '../card/styles.ts'
import { FieldError, PrimaryButton, QuietButton, Screen } from '../components/ui.tsx'
import { isWeChat, publicUrl } from '../lib/env.ts'
import { track } from '../lib/track.ts'
import { useProfile, useSession } from '../lib/session.tsx'

interface CardImage {
  url: string
  file: File
}

export default function CardPreview() {
  const profile = useProfile()
  const { updateProfile } = useSession()
  const [, navigate] = useLocation()
  const [checkin, setCheckin] = useState<Checkin | null>(null)
  const [styleId, setStyleId] = useState('default')
  const [showCity, setShowCity] = useState(profile.showCityOnCard)
  const [image, setImage] = useState<CardImage | null>(null)
  const [error, setError] = useState<string | null>(null)
  const styles = useMemo(() => (checkin ? cardStyles(checkin) : []), [checkin])
  const canShowCity = Boolean(profile.cityName) && profile.cityCode !== 'other'

  useEffect(() => {
    api<TodayResponse>('/api/checkin/today')
      .then((t) => (t.checkin ? setCheckin(t.checkin) : navigate('/', { replace: true })))
      .catch((e: Error) => setError(e.message))
  }, [navigate])

  useEffect(() => {
    if (!checkin || styles.length === 0) return
    let cancelled = false
    const style = styles.find((s) => s.id === styleId) ?? styles[0]
    const link = `${publicUrl()}/?from=card&inv=${profile.inviteCode}`
    renderCard({ checkin, profile, style, showCity: showCity && canShowCity, link })
      .then((blob) => {
        if (cancelled) return
        const file = new File([blob], `下班卡-${checkin.workDay}.png`, { type: 'image/png' })
        setImage({ url: URL.createObjectURL(blob), file })
        track('card_generate', { style: style.id, title: checkin.title.code })
      })
      .catch((e: Error) => setError(e.message))
    return () => {
      cancelled = true
    }
  }, [checkin, styles, styleId, showCity, canShowCity, profile])

  useEffect(() => {
    return () => {
      if (image) URL.revokeObjectURL(image.url)
    }
  }, [image])

  function toggleCity() {
    const next = !showCity
    setShowCity(next)
    updateProfile({ showCityOnCard: next }).catch(() => undefined)
  }

  const canShare = image !== null && typeof navigator.canShare === 'function' && navigator.canShare({ files: [image.file] })
  const wechat = isWeChat()

  return (
    <Screen>
      <header className="flex h-10 items-center justify-between">
        <QuietButton className="px-0!" onClick={() => navigate('/')}>返回</QuietButton>
        <h1 className="text-base font-bold text-window">今日下班卡</h1>
        <span className="w-10" />
      </header>

      <div className="mt-3 flex justify-center">
        {image ? (
          <img src={image.url} alt="今日下班卡预览" className="max-h-[60vh] w-auto rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.45)]" />
        ) : (
          <div className="grid aspect-[9/16] h-[60vh] place-items-center rounded-2xl bg-dusk-soft/40 text-sm text-window/60">正在画卡片…</div>
        )}
      </div>
      {wechat && <p className="mt-3 text-center text-sm text-lamp">长按图片保存，再发朋友圈或群聊</p>}

      <div className="mt-4 flex flex-wrap justify-center gap-2" role="radiogroup" aria-label="卡片样式">
        {styles.map((s) => (
          <button key={s.id} type="button" role="radio" aria-checked={s.id === styleId} onClick={() => setStyleId(s.id)}
            className={`rounded-full border-2 px-4 py-1.5 text-sm ${s.id === styleId ? 'border-lamp bg-lamp font-bold text-ink' : 'border-dusk-soft text-window/80'}`}>
            {s.name}
          </button>
        ))}
      </div>
      {canShowCity && (
        <label className="mt-3 flex items-center justify-center gap-2 text-sm text-window/75">
          <input type="checkbox" checked={showCity} onChange={toggleCity} className="size-4 accent-[var(--color-lamp)]" />
          在卡片上显示城市（{profile.cityName}）
        </label>
      )}
      <FieldError message={error} />

      {!wechat && image && (
        <div className="mt-auto flex gap-3 pt-6">
          {canShare && (
            <PrimaryButton onClick={() => navigator.share({ files: [image.file], title: '下班了么', text: '你下班了么？' }).then(() => track('card_share', { via: 'web_share' })).catch(() => undefined)}>
              分享给朋友
            </PrimaryButton>
          )}
          <a href={image.url} download={image.file.name} onClick={() => track('card_save', { via: 'download' })}
            className={`grid w-full place-items-center rounded-full border-2 border-lamp px-6 py-3 font-bold ${canShare ? 'text-lamp' : 'bg-lamp text-ink'}`}>
            保存图片
          </a>
        </div>
      )}
    </Screen>
  )
}
