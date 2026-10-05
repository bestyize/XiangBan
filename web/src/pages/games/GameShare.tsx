import { useEffect, useMemo, useState } from 'react'
import { Link } from 'wouter'
import { blobToBase64, inApp, send } from '../../lib/bridge.ts'
import { captureReferral, isWeChat, publicUrl } from '../../lib/env.ts'
import { track } from '../../lib/track.ts'
import { gameChallengePath, gameShareLink, gameShareText, readSharedResult } from '../../games/resultShare.ts'
import { renderResultPoster, RESULT_POSTER_HEIGHT, RESULT_POSTER_WIDTH } from '../../games/resultPoster.ts'
import { shareGameToSystem } from '../../games/shareAction.ts'
import './gameShare.css'

export default function GameShare() {
  const [context, setContext] = useState(() => ({ hash: location.hash, search: location.search }))
  const result = useMemo(() => readSharedResult(context.hash), [context.hash])
  const own = new URLSearchParams(context.search).get('own') === '1'
  const inviteCode = new URLSearchParams(context.search).get('inv') || ''
  const [image, setImage] = useState<{ url: string; file: File } | null>(null)
  const [error, setError] = useState('')
  const [message, setMessage] = useState('')
  const [manualCopy, setManualCopy] = useState(false)
  const [retry, setRetry] = useState(0)
  const [busy, setBusy] = useState(false)
  const link = result ? gameShareLink(result, publicUrl(), inviteCode) : ''

  useEffect(() => {
    const update = () => setContext({ hash: location.hash, search: location.search })
    window.addEventListener('hashchange', update)
    return () => window.removeEventListener('hashchange', update)
  }, [])

  useEffect(() => {
    document.title = '贪吃蛇战绩｜下班啦'
    if (!own) captureReferral()
  }, [own, context.search])
  useEffect(() => {
    if (!result) return
    let cancelled = false
    setError(''); setImage(null); setMessage(''); setManualCopy(false)
    renderResultPoster(result, link).then(blob => {
      if (cancelled) return
      setImage({ url: URL.createObjectURL(blob), file: new File([blob], `小伴贪吃蛇-${result.score}分.png`, { type: 'image/png' }) })
    }).catch(() => { if (!cancelled) setError('战绩卡暂时没画好，可以重试或先复制链接。') })
    return () => { cancelled = true }
  }, [result, link, retry])
  useEffect(() => () => { if (image) URL.revokeObjectURL(image.url) }, [image])

  async function share() {
    if (!result) return
    setBusy(true); setMessage('')
    try {
      const text = gameShareText(result)
      if (inApp()) {
        send('share', { text, url: link, ...(image ? { base64: await blobToBase64(image.file), mime: 'image/png' } : {}) })
        setMessage('已打开分享面板。')
      } else {
        const outcome = await shareGameToSystem(navigator, { text, url: link, file: image?.file })
        if (outcome === 'cancelled') return
        if (outcome === 'unsupported') { setMessage('可以保存战绩卡发给朋友，或复制下方链接。'); return }
        setMessage('已完成系统分享。')
      }
      track('card_share', { game: 'snake-arena', kind: 'result' })
    } catch (cause) {
      if (!(cause && typeof cause === 'object' && 'name' in cause && cause.name === 'AbortError')) setMessage('系统分享暂未完成，也可以保存图片或复制链接。')
    } finally { setBusy(false) }
  }

  async function copyLink() {
    try { await navigator.clipboard.writeText(link); setMessage('战绩链接已复制，发给朋友就能一起玩。'); track('card_share', { game: 'snake-arena', kind: 'link' }) }
    catch { setManualCopy(true); setMessage('请长按或选中链接复制。') }
  }
  async function saveImage() {
    if (!image) return
    try { send('save_image', { base64: await blobToBase64(image.file), mime: 'image/png' }); setMessage('已请求保存到相册。') }
    catch { setMessage('可以长按战绩卡保存图片。') }
  }

  return <main className="game-share-page">
    <header className="game-share-header"><span>小伴贪吃蛇</span><Link href={gameChallengePath(result, inviteCode)}>去玩一局 ↗</Link></header>
    {result ? <>
      <p className="game-share-eyebrow">{own ? '我的快乐战绩' : '朋友分享的战绩'}</p>
      <h1>{own ? '把这份快乐，晒给朋友' : '这份战绩，等你来挑战'}</h1>
      <p className="game-share-subtitle">{result.roundType === 'endless' ? '无尽生存' : '三分钟竞技'} · {result.difficulty === 'lively' ? '热闹难度' : '轻松难度'} · {result.score} 分</p>
      <div className="game-share-poster">
        {image ? <img src={image.url} width={RESULT_POSTER_WIDTH} height={RESULT_POSTER_HEIGHT} alt={`贪吃蛇战绩：${result.score} 分，长度 ${result.mass}，淘汰 ${result.kills} 条 AI，存活 ${result.seconds} 秒。扫码来挑战。`} /> : <div className="game-share-placeholder" role="status">{error || '正在画你的快乐战绩…'}</div>}
      </div>
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
      <Link className="game-share-button play" href={gameChallengePath(result, inviteCode)}>{own ? '再玩一局，刷新战绩' : '我也来玩一局'}</Link>
      <p className="game-share-hint">分享内容只含本局数据，不含账号信息。战绩由分享者提供。</p>
    </> : <section className="game-share-invalid"><h1>这份战绩暂时打不开</h1><p>链接可能不完整，来玩一局，生成自己的战绩吧。</p><Link className="game-share-button" href={gameChallengePath(null)}>去玩贪吃蛇</Link></section>}
  </main>
}
