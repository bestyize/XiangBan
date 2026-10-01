import { useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { Checkin } from '../../api/types.ts'
import { Bubble, QuietButton } from '../../components/ui.tsx'
import { track } from '../../lib/track.ts'

type ReportState = 'idle' | 'confirm' | 'sending' | 'done' | 'error'

export function ReplyBubble({ checkin }: { checkin: Checkin }) {
  const [report, setReport] = useState<ReportState>('idle')
  const [message, setMessage] = useState<string | null>(null)

  // One event per line actually shown, so each line's share rate can be measured (PRD 10).
  useEffect(() => {
    if (!checkin.reply.pending) track('reply_shown', { checkin: checkin.id, source: checkin.reply.source })
  }, [checkin.id, checkin.reply.source, checkin.reply.pending])

  async function send() {
    setReport('sending')
    try {
      await api('/api/reports', { method: 'POST', body: { checkinId: checkin.id } })
      setReport('done')
    } catch (e) {
      setMessage((e as Error).message)
      setReport('error')
    }
  }

  return (
    <div className="mt-5">
      <Bubble align="left">
        {/* Keyed by text so a model rewrite fades in over the library line. */}
        <span key={checkin.reply.text} className="block animate-fade-in">{checkin.reply.text}</span>
        {checkin.reply.source === 'llm' && <span className="mt-1 block text-right text-xs text-ink/45">AI 润色</span>}
      </Bubble>
      <div className="mt-1 flex min-h-9 items-center justify-end gap-1 text-sm">
        {report === 'idle' && <QuietButton className="px-2! text-xs" onClick={() => setReport('confirm')}>这句话不合适？</QuietButton>}
        {report === 'confirm' && (
          <>
            <span className="text-xs text-window/70">举报后这句话会下架</span>
            <QuietButton className="px-2! text-xs text-blush!" onClick={send}>举报</QuietButton>
            <QuietButton className="px-2! text-xs" onClick={() => setReport('idle')}>算了</QuietButton>
          </>
        )}
        {report === 'sending' && <span className="text-xs text-window/60">正在提交…</span>}
        {report === 'done' && <span className="text-xs text-window/70">收到了，这句话以后不会再出现。</span>}
        {report === 'error' && <span role="alert" className="text-xs text-blush">{message}</span>}
      </div>
    </div>
  )
}
