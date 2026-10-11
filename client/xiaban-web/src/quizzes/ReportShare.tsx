import { useEffect, useRef, useState } from 'react'
import { Link } from 'wouter'
import { blobToBase64, inApp, send } from '../lib/bridge.ts'
import { isWeChat, publicUrl } from '../lib/env.ts'
import { POSTER_HEIGHT, POSTER_WIDTH, renderQuizPoster } from './poster.ts'
import { ShareIcon, SharePreview, ShareStatus } from '../components/SharePreview.tsx'
import { PosterThemeToggle } from '../components/PosterThemeToggle.tsx'
import { usePosterTheme } from '../poster/theme.ts'
import { shareLink } from './share.ts'
import type { QuizDefinition, QuizReport } from './types.ts'

interface PosterImage { url: string; file: File }

export function ReportShare({ quiz, report, scores, shared }: { quiz: QuizDefinition; report: QuizReport; scores: number[]; shared: boolean }) {
  const [open, setOpen] = useState(false)
  const [image, setImage] = useState<PosterImage | null>(null)
  const [message, setMessage] = useState('')
  const [posterError, setPosterError] = useState('')
  const [retry, setRetry] = useState(0)
  const [manualCopy, setManualCopy] = useState(false)
  const [busy, setBusy] = useState(false)
  const [theme, setTheme] = usePosterTheme()
  const dialog = useRef<HTMLDialogElement>(null)
  const link = shareLink(quiz, scores, publicUrl())

  useEffect(() => {
    const node = dialog.current
    if (!node) return
    if (open && !node.open) node.showModal()
    if (!open && node.open) node.close()
  }, [open])

  useEffect(() => {
    if (!open) return
    let cancelled = false
    setPosterError(''); setMessage(''); setManualCopy(false)
    setImage(null)
    renderQuizPoster(quiz, report, link, theme).then((blob) => {
      if (cancelled) return
      setImage({ url: URL.createObjectURL(blob), file: new File([blob], `${quiz.name}-${report.code.replace(/[^A-Z]/g, '')}.png`, { type: 'image/png' }) })
    }).catch(() => {
      if (!cancelled) setPosterError('海报没画好，可以重试，或先复制报告链接。')
    })
    return () => { cancelled = true }
  }, [open, quiz, report, link, retry, theme])

  useEffect(() => () => { if (image) URL.revokeObjectURL(image.url) }, [image])

  async function copy() {
    try {
      await navigator.clipboard.writeText(link)
      setMessage('报告链接已复制，可以发给朋友了。')
    } catch {
      setManualCopy(true); setMessage('可以长按或选中下方链接复制。')
    }
  }

  async function share() {
    setMessage('')
    setBusy(true)
    try {
      const title = `${quiz.name} · ${report.title}`
      const text = `我是「${report.title}」。${report.quote}`
      if (inApp()) {
        send('share', { text, url: link, ...(image ? { base64: await blobToBase64(image.file), mime: 'image/png' } : {}) })
        setMessage('已打开分享面板。')
      } else if (typeof navigator.share === 'function') {
        let files: File[] | undefined
        try { if (image && navigator.canShare?.({ files: [image.file] })) files = [image.file] } catch { /* share the link instead */ }
        await navigator.share({ title, text, url: link, ...(files ? { files } : {}) })
        setMessage('已完成系统分享。')
      } else {
        setOpen(true)
        setMessage('保存海报或复制链接，就可以分享给朋友。')
      }
    } catch (error) {
      if (!(error && typeof error === 'object' && 'name' in error && error.name === 'AbortError')) setMessage('分享暂时没有完成，也可以保存海报或复制链接。')
    } finally {
      setBusy(false)
    }
  }

  async function nativeSave() {
    if (!image) return
    setBusy(true); setMessage('')
    try {
      send('save_image', { base64: await blobToBase64(image.file), mime: 'image/png' })
      setMessage('已请求保存到相册。')
    } catch { setMessage('暂时无法保存，请长按海报保存图片。') }
    finally { setBusy(false) }
  }

  return <>
    <section className={`quiz-action-bar quiz-theme-${quiz.id}`} aria-label="分享测试结果">
      {/* The dialog shows its own status while open. */}
      <p className="quiz-action-status" role="status">{open ? '' : message}</p>
      <div className="quiz-action-row">
        {shared
          ? <Link className="quiz-button" href={`/tests/${quiz.id}`}>我也测一测</Link>
          : <button className="quiz-button" type="button" disabled={busy} onClick={() => { setMessage(''); setOpen(true) }}>分享给朋友</button>}
        <button className="quiz-button quiz-button-outline" type="button" onClick={() => setOpen(true)}>{shared ? '查看海报' : '保存海报'}</button>
      </div>
    </section>
    <dialog ref={dialog} className="quiz-poster-dialog" onClose={() => setOpen(false)} onClick={(event) => { if (event.target === event.currentTarget) setOpen(false) }} aria-labelledby="poster-title">
      <div className={`quiz-poster-panel quiz-theme-${quiz.id}`}>
        <header className="quiz-dialog-header"><h2 id="poster-title">你的分享海报</h2><button type="button" className="quiz-text-button" onClick={() => setOpen(false)} aria-label="关闭海报">关闭 ×</button></header>
        <div className="quiz-poster-content">
          <SharePreview image={image} width={POSTER_WIDTH} height={POSTER_HEIGHT}
            alt={quiz.name + ' 测试报告：' + report.code + '，' + report.title + '。' + report.quote}
            error={posterError} onRetry={() => setRetry(value => value + 1)} />
          <section className="share-settings" aria-label="海报样式">
            <h3 className="share-settings-heading">选一个海报配色</h3>
            <PosterThemeToggle value={theme} onChange={setTheme} />
          </section>
          <p className="share-note">{isWeChat() ? '长按海报保存，再发给朋友或朋友圈。' : '保存你的结果名片，邀请朋友也来测一测。'}<br />分享内容不含逐题答案和账号信息。</p>
        </div>
        <footer className="quiz-dialog-footer">
          <div className="share-actions">
            <button className="share-action" type="button" disabled={busy || (!image && !posterError)} onClick={() => void share()}><ShareIcon kind="share" />{busy ? '正在处理…' : '分享给朋友'}</button>
            {image ? (inApp() ? <button className="share-action secondary" type="button" disabled={busy} onClick={() => void nativeSave()}><ShareIcon kind="save" />保存到相册</button>
              : <a className="share-action secondary" href={image.url} download={image.file.name}><ShareIcon kind="save" />保存图片</a>)
              : <button className="share-action secondary" type="button" disabled><ShareIcon kind="save" />保存图片</button>}
          </div>
          <button className="share-copy-button" type="button" disabled={busy} onClick={() => void copy()}><ShareIcon kind="link" />复制报告链接</button>
          <ShareStatus message={open ? message : ''} />
          {manualCopy && <input className="share-manual-copy" aria-label="可复制的报告链接" value={link} readOnly onFocus={event => event.currentTarget.select()} />}
        </footer>
      </div>
    </dialog>
  </>
}
