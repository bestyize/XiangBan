import { useEffect } from 'react'
import type { Checkin } from '../../api/types.ts'
import { Bubble } from '../../components/ui.tsx'
import { track } from '../../lib/track.ts'

export function ReplyBubble({ checkin }: { checkin: Checkin }) {
  // One event per line actually shown, so each line's share rate can be measured (PRD 10).
  useEffect(() => {
    if (!checkin.reply.pending) track('reply_shown', { checkin: checkin.id, source: checkin.reply.source })
  }, [checkin.id, checkin.reply.source, checkin.reply.pending])

  return (
    <div className="mt-5">
      <Bubble align="left">
        {/* Keyed by text so a model rewrite fades in over the library line. */}
        <span key={checkin.reply.text} className="block animate-fade-in">{checkin.reply.text}</span>
        {checkin.reply.source === 'llm' && <span className="mt-1 block text-right text-xs text-ink/45">AI 润色</span>}
      </Bubble>
    </div>
  )
}
