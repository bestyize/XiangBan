import { useEffect, useMemo, useState } from 'react'
import { Link } from 'wouter'
import { blobToBase64, inApp, send } from '../../lib/bridge.ts'
import { captureReferral, isWeChat, publicUrl } from '../../lib/env.ts'
import { track } from '../../lib/track.ts'
import { renderGamePoster, RESULT_POSTER_HEIGHT, RESULT_POSTER_WIDTH } from '../../games/common/resultPoster.ts'
import { challengePath as gameChallengePath, resultShareLink } from '../../games/common/share.ts'
import { shareGameToSystem } from '../../games/common/shareAction.ts'
import { resultGame } from '../../games/index.ts'
import { PosterThemeToggle } from '../../components/PosterThemeToggle.tsx'
import { usePosterTheme } from '../../poster/theme.ts'
import './gameShare.css'

export default function GameShare({ gameId = 'snake-arena' }: { gameId?: string }) {
  const [context, setContext] = useState(() => ({ hash: location.hash, search: location.search }))
  const game = resultGame(gameId)
  const result = useMemo(() => game?.read(context.hash) ?? null, [game, context.hash])
  const name = game?.name ?? '小游戏'
  const own = new URLSearchParams(context.search).get('own') === '1'
  const inviteCode = new URLSearchParams(context.search).get('inv') || ''
  const [image, setImage] = useState<{ url: string; file: File } | null>(null)
  const [error, setError] = useState('')
  const [message, setMessage] = useState('')
  const [manualCopy, setManualCopy] = useState(false)
  const [retry, setRetry] = useState(0)
  const [busy, setBusy] = useState(false)
  const [theme, setTheme] = usePosterTheme()
  const link = game && result ? resultShareLink(game, result, publicUrl(), inviteCode) : ''
  const challengePath = game ? gameChallengePath(game, result, inviteCode) : '/games'
  const shareText = game && result ? game.shareText(result) : ''

  useEffect(() => {
    const update = () => setContext({ hash: location.hash, search: location.search })
    window.addEventListener('hashchange', update)
    return () => window.removeEventListener('hashchange', update)
  }, [])

  useEffect(() => {
    document.title = `${name}战绩｜相伴`
    if (!own) captureReferral()
  }, [own, context.search, name])
  useEffect(() => {
    if (!game || !result) return
    let cancelled = false
    setError(''); setImage(null); setMessage(''); setManualCopy(false)
    renderGamePoster(game.poster(result), link, theme).then(blob => {
      if (cancelled) return
      setImage({ url: URL.createObjectURL(blob), file: new File([blob], `${name}-${result.score}分.png`, { type: 'image/png' }) })
    }).catch(() => { if (!cancelled) setError('战绩卡暂时没画好，可以重试或先复制链接。') })
    return () => { cancelled = true }
  }, [game, result, link, retry, name, theme])
  useEffect(() => () => { if (image) URL.revokeObjectURL(image.url) }, [image])

  async function share() {
    if (!result) return
    setBusy(true); setMessage('')
    try {
      const text = shareText
      if (inApp()) {
        send('share', { text, url: link, ...(image ? { base64: await blobToBase64(image.file), mime: 'image/png' } : {}) })
        setMessage('已打开分享面板。')
      } else {
        const outcome = await shareGameToSystem(navigator, { text, url: link, file: image?.file, title: `${name} · 快乐战绩` })
        if (outcome === 'cancelled') return
        if (outcome === 'unsupported') { setMessage('可以保存战绩卡发给朋友，或复制下方链接。'); return }
        setMessage('已完成系统分享。')
      }
      track('card_share', { game: gameId, kind: 'result' })
    } catch (cause) {
      if (!(cause && typeof cause === 'object' && 'name' in cause && cause.name === 'AbortError')) setMessage('系统分享暂未完成，也可以保存图片或复制链接。')
    } finally { setBusy(false) }
  }

  async function copyLink() {
    try { await navigator.clipboard.writeText(link); setMessage('战绩链接已复制，发给朋友就能一起玩。'); track('card_share', { game: gameId, kind: 'link' }) }
    catch { setManualCopy(true); setMessage('请长按或选中链接复制。') }
  }
  async function saveImage() {
    if (!image) return
    try { send('save_image', { base64: await blobToBase64(image.file), mime: 'image/png' }); setMessage('已请求保存到相册。') }
    catch { setMessage('可以长按战绩卡保存图片。') }
  }

  return <main className="game-share-page">
    <header className="game-share-header"><span>{name}</span><Link href={challengePath}>去玩一局 ↗</Link></header>
    {game && result ? <>
      <p className="game-share-eyebrow">{own ? '我的快乐战绩' : '朋友分享的战绩'}</p>
      <h1>{own ? '把这份快乐，晒给朋友' : '这份战绩，等你来挑战'}</h1>
      <p className="game-share-subtitle">{game.subtitle(result)}</p>
      <div className="game-share-poster">
        {image ? <img src={image.url} width={RESULT_POSTER_WIDTH} height={RESULT_POSTER_HEIGHT} alt={`${shareText} 扫码来挑战。`} /> : <div className="game-share-placeholder" role="status">{error || '正在画你的快乐战绩…'}</div>}
      </div>
      <PosterThemeToggle value={theme} onChange={setTheme} tone="light" className="mb-4" />
      {error && <button className="game-share-link-button" onClick={() => setRetry(value => value + 1)}>重新生成战绩卡</button>}
      <div className="game-share-actions">
        <button className="game-share-button" disabled={busy} onClick={() => void share()}>{busy ? '正在分享…' : '分享给朋友'}</button>
        {image && (inApp() ? <button className="game-share-button outline" onClick={() => void saveImage()}>保存到相册</button> : <a className="game-share-button outline" href={image.url} download={image.file.name}>保存战绩卡</a>)}
      </div>
      {image && <p className="game-share-hint">长按战绩卡也可以保存图片，发给朋友或朋友圈。</p>}
      <button className="game-share-link-button" onClick={() => void copyLink()}>复制战绩链接</button>
      <p className="game-share-status" role="status">{message}</p>
      {manualCopy && <input className="game-share-copy" aria-label="可复制的战绩链接" readOnly value={link} onFocus={event => event.currentTarget.select()} />}
      {isWeChat() && !image && <p className="game-share-hint">战绩卡生成后，可以长按保存，再发给朋友或朋友圈。</p>}
      <Link className="game-share-button play" href={challengePath}>{own ? '再玩一局，刷新战绩' : '我也来玩一局'}</Link>
      <p className="game-share-hint">分享内容只含本局数据，不含账号信息。战绩由分享者提供。</p>
    </> : <section className="game-share-invalid"><h1>这份战绩暂时打不开</h1><p>链接可能不完整，来玩一局，生成自己的战绩吧。</p><Link className="game-share-button" href={challengePath}>去玩{name}</Link></section>}
  </main>
}
