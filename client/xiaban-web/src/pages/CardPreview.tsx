import { useEffect, useMemo, useState } from 'react'
import { ACTIVITY_PATH } from '../lib/activity.ts'
import { useLocation } from 'wouter'
import { api, getToken } from '../api/client.ts'
import type { Checkin, TodayResponse } from '../api/types.ts'
import { browserTimezone } from '../lib/device.ts'
import { loadGuestCheckin } from '../lib/guestCheckin.ts'
import { CARD_H, CARD_W, renderCard } from '../card/renderCard.ts'
import { cardStyles } from '../card/styles.ts'
import { POSTER_THEMES, loadPosterTheme, savePosterTheme } from '../poster/theme.ts'
import { SubPage } from '../components/SubPage.tsx'
import { ShareIcon, SharePreview, ShareStatus } from '../components/SharePreview.tsx'
import { blobToBase64, inApp, send } from '../lib/bridge.ts'
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
  const [styleId, setStyleId] = useState<string>(loadPosterTheme)
  const [showCity, setShowCity] = useState(profile.showCityOnCard)
  const [image, setImage] = useState<CardImage | null>(null)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [posterError, setPosterError] = useState<string | null>(null)
  const [loadRetry, setLoadRetry] = useState(0)
  const [retry, setRetry] = useState(0)
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const [manualCopy, setManualCopy] = useState(false)
  const styles = useMemo(() => (checkin ? cardStyles(checkin) : []), [checkin])
  const canShowCity = Boolean(profile.cityName) && profile.cityCode !== 'other'

  useEffect(() => {
    let cancelled = false
    setLoadError(null)
    const tz = browserTimezone() ?? 'Asia/Shanghai'
    api<TodayResponse>(`/api/checkin/today?timezone=${encodeURIComponent(tz)}`)
      .then((t) => {
        if (cancelled) return
        // Guests have no server-side check-in; theirs lives in this browser.
        const today = t.checkin ?? (getToken() ? null : loadGuestCheckin(t.workDay))
        if (today) setCheckin(today)
        else navigate(ACTIVITY_PATH, { replace: true })
      })
      .catch((e: Error) => { if (!cancelled) setLoadError(e.message) })
    return () => { cancelled = true }
  }, [navigate, loadRetry])

  useEffect(() => {
    if (!checkin || styles.length === 0) return
    let cancelled = false
    setImage(null); setPosterError(null); setMessage(''); setManualCopy(false)
    const style = styles.find((s) => s.id === styleId) ?? styles[0]
    const link = `${publicUrl()}${ACTIVITY_PATH}?from=card${profile.inviteCode ? `&inv=${profile.inviteCode}` : ''}`
    renderCard({ checkin, profile, style, showCity: showCity && canShowCity, link })
      .then((blob) => {
        if (cancelled) return
        const file = new File([blob], `下班卡-${checkin.workDay}.png`, { type: 'image/png' })
        setImage({ url: URL.createObjectURL(blob), file })
        track('card_generate', { style: style.id, title: checkin.title.code })
      })
      .catch((e: Error) => { if (!cancelled) setPosterError(e.message) })
    return () => {
      cancelled = true
    }
  }, [checkin, styles, styleId, showCity, canShowCity, profile, retry])

  useEffect(() => {
    return () => {
      if (image) URL.revokeObjectURL(image.url)
    }
  }, [image])

  // Guests have no invite code yet; the link then just brings friends to the site.
  const shareLink = `${publicUrl()}${ACTIVITY_PATH}?from=card${profile.inviteCode ? `&inv=${profile.inviteCode}` : ''}`

  function toggleCity() {
    const next = !showCity
    setShowCity(next)
    updateProfile({ showCityOnCard: next }).catch(() => undefined)
  }

  let canShare = false
  try { canShare = image !== null && navigator.canShare?.({ files: [image.file] }) === true } catch { /* Share the link instead. */ }
  const wechat = isWeChat()
  const app = inApp()

  async function copyLink() {
    try {
      await navigator.clipboard.writeText(shareLink)
      setMessage('下班卡链接已复制，可以发给朋友了。')
      track('card_share', { via: 'link' })
    } catch { setManualCopy(true); setMessage('请长按或选中下方链接复制。') }
  }

  async function nativeSend(type: 'save_image' | 'share') {
    if (!image) return
    setBusy(true); setMessage('')
    try {
      const base64 = await blobToBase64(image.file)
      if (type === 'save_image') {
        send('save_image', { base64, mime: 'image/png' })
        track('card_save', { via: 'app' })
        setMessage('已请求保存到相册。')
      } else {
        send('share', { base64, mime: 'image/png', text: '你下班啦？', url: shareLink })
        track('card_share', { via: 'app' })
        setMessage('已打开分享面板。')
      }
    } catch { setMessage('操作暂未完成，可以长按图片保存或复制链接。') }
    finally { setBusy(false) }
  }

  async function share() {
    if (app) { await nativeSend('share'); return }
    if (typeof navigator.share !== 'function') { await copyLink(); return }
    setBusy(true); setMessage('')
    try {
      await navigator.share({ title: '下班啦', text: '你下班啦？', url: shareLink, ...(canShare && image ? { files: [image.file] } : {}) })
      track('card_share', { via: 'web_share' })
      setMessage('已完成系统分享。')
    } catch (cause) {
      if (!(cause && typeof cause === 'object' && 'name' in cause && cause.name === 'AbortError')) setMessage('分享暂未完成，可以保存图片或复制链接。')
    } finally { setBusy(false) }
  }

  return <SubPage title="今日下班卡" back="/" className="share-page">
    <section className="share-intro">
      <span className="share-eyebrow">记录今天的下班时刻</span>
      <h2>下班的快乐，分享给朋友</h2>
      <p>选一个喜欢的配色，把今天的小确幸留成一张卡。</p>
    </section>
    <SharePreview image={image} width={CARD_W} height={CARD_H} alt="今日下班卡预览" error={loadError || posterError}
      onRetry={() => loadError ? setLoadRetry(value => value + 1) : setRetry(value => value + 1)} />
    {styles.length > 0 && <section className="share-settings" aria-label="卡片设置">
      <h2 className="share-settings-heading">选一个海报配色</h2>
      <div className="share-style-picker" role="radiogroup" aria-label="卡片样式">
        {styles.map(style => <button key={style.id} type="button" role="radio" aria-checked={style.id === styleId}
          onClick={() => { setStyleId(style.id); if (style.id === 'night' || style.id === 'day') savePosterTheme(style.id) }} className="share-style-option">
          <span aria-hidden="true" className="share-style-swatch" style={{ background: 'linear-gradient(' + (style.sky ?? POSTER_THEMES[style.theme].bg).join(',') + ')' }} />{style.name}
        </button>)}
      </div>
      {canShowCity && <label className="share-city-toggle">
        <input type="checkbox" checked={showCity} onChange={toggleCity} className="size-4 accent-lamp" />
        显示城市（{profile.cityName}）
      </label>}
    </section>}
    <p className="share-note">{wechat ? '长按图片保存，再发给朋友或朋友圈。' : '保存图片发给朋友，或复制链接邀请他们一起打卡。'}</p>
    <section className="share-actions-panel" aria-label="分享下班卡">
      <div className="share-actions">
        <button type="button" className="share-action" disabled={busy || !image} onClick={() => void share()}><ShareIcon kind="share" />{busy ? '正在处理…' : '分享给朋友'}</button>
        {image ? (app ? <button type="button" className="share-action secondary" disabled={busy} onClick={() => void nativeSend('save_image')}><ShareIcon kind="save" />保存到相册</button>
          : <a className="share-action secondary" href={image.url} download={image.file.name} onClick={() => track('card_save', { via: 'download' })}><ShareIcon kind="save" />保存图片</a>)
          : <button type="button" className="share-action secondary" disabled><ShareIcon kind="save" />保存图片</button>}
      </div>
      <button type="button" className="share-copy-button" disabled={busy} onClick={() => void copyLink()}><ShareIcon kind="link" />复制分享链接</button>
      <ShareStatus message={message} />
      {manualCopy && <input className="share-manual-copy" aria-label="可复制的下班卡链接" value={shareLink} readOnly onFocus={event => event.currentTarget.select()} />}
    </section>
  </SubPage>
}
