import { useEffect, useRef, useState } from 'react'
import { blobToBase64, inApp, send } from '../lib/bridge.ts'
import { isWeChat, publicUrl } from '../lib/env.ts'
import { renderQuizPoster } from './poster.ts'
import { shareLink } from './share.ts'
import type { QuizDefinition, QuizReport } from './types.ts'

interface PosterImage { url: string; file: File }

export function ReportShare({ quiz, report, scores }: { quiz: QuizDefinition; report: QuizReport; scores: number[] }) {
  const [open, setOpen] = useState(false)
  const [image, setImage] = useState<PosterImage | null>(null)
  const [message, setMessage] = useState('')
  const [posterError, setPosterError] = useState('')
  const [retry, setRetry] = useState(0)
  const [busy, setBusy] = useState(false)
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
    setPosterError('')
    setImage(null)
    renderQuizPoster(quiz, report, link).then((blob) => {
      if (cancelled) return
      setImage({ url: URL.createObjectURL(blob), file: new File([blob], `${quiz.name}-${report.code.replace(/[^A-Z]/g, '')}.png`, { type: 'image/png' }) })
    }).catch(() => {
      if (!cancelled) setPosterError('海报没画好，可以重试，或先复制报告链接。')
    })
    return () => { cancelled = true }
  }, [open, quiz, report, link, retry])

  useEffect(() => () => { if (image) URL.revokeObjectURL(image.url) }, [image])

  async function copy() {
    try {
      await navigator.clipboard.writeText(link)
      setMessage('报告链接已复制，可以发给朋友了。')
    } catch {
      setMessage('可以长按或选中下方链接复制。')
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
      } else if (typeof navigator.share === 'function') {
        let files: File[] | undefined
        try { if (image && navigator.canShare?.({ files: [image.file] })) files = [image.file] } catch { /* share the link instead */ }
        await navigator.share({ title, text, url: link, ...(files ? { files } : {}) })
      } else {
        setOpen(true)
        setMessage('保存海报或复制链接，就可以分享给朋友。')
      }
    } catch (error) {
      if (!(error instanceof Error && error.name === 'AbortError')) setMessage('分享暂时没有完成，也可以保存海报或复制链接。')
    } finally {
      setBusy(false)
    }
  }

  async function nativeSave() {
    if (!image) return
    try {
      send('save_image', { base64: await blobToBase64(image.file), mime: 'image/png' })
    } catch { setMessage('暂时无法保存，请长按海报保存图片。') }
  }

  return <section className="quiz-share" aria-label="分享测试结果">
    <h2>这张名片，想送给谁看？</h2>
    <p>分享内容不包含逐题答案或账号信息。</p>
    <div className="quiz-share-actions">
      <button className="quiz-button" type="button" disabled={busy} onClick={() => void share()}>{busy ? '正在分享…' : '分享给朋友'} <span aria-hidden="true">↗</span></button>
      <button className="quiz-button quiz-button-outline" type="button" onClick={() => setOpen(true)}>保存分享海报</button>
    </div>
    <button className="quiz-text-button" type="button" onClick={() => void copy()}>复制报告链接</button>
    <p className="quiz-message" role="status">{message}</p>
    {message.includes('下方链接') && <input className="quiz-copy-link" aria-label="可复制的报告链接" value={link} readOnly onFocus={(event) => event.currentTarget.select()} />}
    <dialog ref={dialog} className="quiz-poster-dialog" onClose={() => setOpen(false)} aria-labelledby="poster-title">
      <div className={`quiz-theme-${quiz.id}`}>
        <header className="quiz-dialog-header"><h2 id="poster-title">你的分享海报</h2><button type="button" className="quiz-text-button" onClick={() => setOpen(false)} aria-label="关闭海报">关闭 ×</button></header>
        {image ? <img className="quiz-poster-preview" src={image.url} alt={`${quiz.name} 测试报告：${report.code}，${report.title}。${report.quote}`} /> : <div className="quiz-poster-placeholder" role="status">{posterError || '正在画你的结果名片…'}</div>}
        {posterError && <button className="quiz-button" type="button" onClick={() => setRetry((value) => value + 1)}>重新生成海报</button>}
        {isWeChat() && <p className="quiz-dialog-hint">长按海报保存，再发给朋友或朋友圈。</p>}
        {image && <div className="quiz-share-actions">
          {inApp() ? <button className="quiz-button" type="button" onClick={() => void nativeSave()}>保存到相册</button> : <a className="quiz-button" href={image.url} download={image.file.name}>保存图片</a>}
          <button className="quiz-button quiz-button-outline" type="button" disabled={busy} onClick={() => void share()}>分享</button>
        </div>}
        <p className="quiz-dialog-hint">如浏览器未直接保存，可长按海报保存图片。</p>
        <button className="quiz-text-button" type="button" onClick={() => void copy()}>复制报告链接</button>
        <p className="quiz-message" role="status">{message}</p>
        {message.includes('下方链接') && <input className="quiz-copy-link" aria-label="可复制的报告链接" value={link} readOnly onFocus={(event) => event.currentTarget.select()} />}
      </div>
    </dialog>
  </section>
}
