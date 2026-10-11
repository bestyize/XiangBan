import './share.css'

export function ShareIcon({ kind }: { kind: 'share' | 'save' | 'link' }) {
  const path = {
    share: 'M12 16V3m-5 5 5-5 5 5M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6',
    save: 'M12 3v12m-5-5 5 5 5-5M5 17v3h14v-3',
    link: 'M10 13a5 5 0 0 0 7 .3l3-3a5 5 0 0 0-7-7l-2 2M14 11a5 5 0 0 0-7-.3l-3 3a5 5 0 0 0 7 7l2-2',
  }[kind]
  return <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"><path d={path} /></svg>
}

/** Shared preview states for check-in cards, game results and quiz posters. */
export function SharePreview({ image, alt, width, height, error, onRetry }: {
  image: { url: string } | null
  alt: string
  width: number
  height: number
  error?: string | null
  onRetry: () => void
}) {
  return <section className="share-preview" aria-label="分享图片预览" aria-busy={!image && !error}>
    <div className="share-preview-label"><span>海报预览</span><span>{image ? '长按图片可保存' : error ? '生成失败' : '正在生成'}</span></div>
    <div className="share-preview-stage">
      {image ? <img src={image.url} width={width} height={height} alt={alt} /> : <div className="share-preview-placeholder" role="status">
        {!error && <span className="share-preview-spinner" aria-hidden="true" />}
        <p>{error || '正在准备你的分享海报…'}</p>
        {error && <button type="button" className="share-copy-button" onClick={onRetry}>重新生成</button>}
      </div>}
    </div>
  </section>
}

export function ShareStatus({ message }: { message: string }) {
  return <p className="share-status" role="status" aria-live="polite">{message}</p>
}
