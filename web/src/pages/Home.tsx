import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { Link } from 'wouter'
import { AccountBadge } from '../components/AccountBadge.tsx'
import { PageHeader } from '../components/PageHeader.tsx'
import { Robot } from '../components/Robot.tsx'
import { TAB_BAR_SPACE } from '../components/TabBar.tsx'
import { Bubble, RobotStage } from '../components/ui.tsx'
import { useDisplaySettings } from '../lib/displaySettings.tsx'
import { useProfile } from '../lib/session.tsx'
import type { RobotInput } from '../robot/expressions.ts'
import { getScenario } from './chat/ChatScenarios.ts'
import { useChat } from './chat/useChat.ts'
import { useSpeech } from './chat/useSpeech.ts'

const personalityName = {
  toxic: '毒舌心软',
  gentle: '温柔治愈',
  chatty: '元气话痨',
  tsundere: '高冷傲娇',
} as const

/** How 小伴 looks when you open the app, by time of day. */
const periodFace: Record<string, RobotInput> = {
  周末时光: 'happy',
  下班通勤: 'greeting',
  居家回血: 'happy',
  深夜树洞: { expression: 'idle', symbol: 'moon' },
  白天时光: 'idle',
}

/** Stable key for a message row, used to track which reply is being spoken. */
function messageId(message: { id?: number }, index: number): string {
  return message.id != null ? `id-${message.id}` : `idx-${index}`
}

function Activity({ href, title, note, tint, icon }: { href: string; title: string; note: string; tint: string; icon: ReactNode }) {
  return (
    <Link href={href} className="flex min-h-24 flex-col justify-between rounded-2xl border border-window/10 p-3 transition active:scale-[.97]" style={{ background: tint }}>
      <span aria-hidden="true" className="text-lamp">{icon}</span>
      <span>
        <span className="block text-[15px] font-bold text-window">{title}</span>
        <span className="mt-0.5 block text-[11px] leading-snug text-window/60">{note}</span>
      </span>
    </Link>
  )
}

const svg = (d: string) => (
  <svg viewBox="0 0 24 24" className="size-6" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d={d} /></svg>
)

/** 相伴 home: 小伴 at home, ready to chat, with a few things to do together. */
export default function Home({ active = true }: { active?: boolean }) {
  const profile = useProfile()
  const { settings } = useDisplaySettings()
  const chatOn = settings.chatTabVisible
  const { messages, streamingText, isStreaming, error, send, clear } = useChat(active && chatOn)
  const { speak, speakingId, loadingId, error: speechError } = useSpeech()
  const [input, setInput] = useState('')
  const scrollRef = useRef<HTMLDivElement>(null)
  const sentRef = useRef(false)
  const scenario = useMemo(() => getScenario(), [])

  const companionName = profile.companionName || '小伴'
  const personalityLabel = personalityName[profile.personality] || '温柔治愈'

  // Follow the conversation once the user starts talking; opening the page keeps 小伴 in view.
  useEffect(() => {
    const el = scrollRef.current
    if (el && sentRef.current) el.scrollTop = el.scrollHeight
  }, [messages, streamingText])

  function handleSend(textToSend?: string) {
    const text = textToSend ?? input
    if (!text.trim() || isStreaming) return
    sentRef.current = true
    void send(text)
    if (!textToSend) setInput('')
  }

  return (
    <main
      className="mx-auto flex h-dvh w-full max-w-[480px] flex-col overflow-hidden bg-dusk-deep text-window"
      style={{ paddingTop: 'var(--safe-top)', paddingBottom: TAB_BAR_SPACE }}
    >
      <div className="shrink-0 px-5">
        <PageHeader title="相伴">
          {chatOn && messages.length > 0 && (
            <button type="button" onClick={() => { if (window.confirm(`要清空和${companionName}的对话记录吗？`)) void clear() }}
              className="min-h-11 rounded-full px-3 text-xs text-window/50 hover:bg-window/10 hover:text-window">
              清空对话
            </button>
          )}
          <AccountBadge compact />
        </PageHeader>
      </div>

      <div ref={scrollRef} className="flex-1 overflow-y-auto overscroll-contain px-5 pb-4">
        <section aria-label={companionName} className="pt-6 text-center">
          <RobotStage spec={periodFace[scenario.periodName] ?? 'idle'} label={`${companionName}在家等你`} size={212}
            interactive autoSleep thinking={isStreaming && !streamingText}
            speaking={(isStreaming && Boolean(streamingText)) || speakingId !== null} />
          <p className="mt-3 text-[13px] text-window/55">
            <b className="font-bold text-window/85">{companionName}</b> · {personalityLabel} · {scenario.periodName}
          </p>
          <Bubble className="mt-4">{scenario.greeting}</Bubble>
          {chatOn && (
            <div className="mt-4 flex flex-wrap justify-center gap-2">
              {scenario.chips.map((chip) => (
                <button key={chip} type="button" disabled={isStreaming} onClick={() => handleSend(chip)}
                  className="min-h-9 rounded-full border border-window/15 bg-window/5 px-3 text-[13px] text-window/80 transition active:scale-95 active:bg-lamp/20 disabled:opacity-50">
                  {chip}
                </button>
              ))}
            </div>
          )}
        </section>

        <section aria-labelledby="together" className="mt-7">
          <h2 id="together" className="text-[15px] font-bold text-window/85">今天一起做点什么</h2>
          <div className="mt-3 grid grid-cols-3 gap-2.5">
            <Activity href="/xiaban" title="下班打卡" note="领今日称号，晒下班卡" tint="linear-gradient(160deg,#4a3a26,#2b3b52)"
              icon={svg('M12 7v5l3 2M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z')} />
            <Activity href="/games" title="玩一局" note="几分钟的小游戏" tint="linear-gradient(160deg,#263f4a,#2b3b52)"
              icon={svg('M7 7h10c2 0 3 2 3.5 4l1 6c.4 2-1.6 3-3 1.5L16 16H8l-2.5 2.5C4 20 2 19 2.5 17l1-6C4 9 5 7 7 7ZM8 10v5m-2.5-2.5h5')} />
            <Activity href="/tests" title="测一测" note="看看今天的你" tint="linear-gradient(160deg,#41304d,#2b3b52)"
              icon={svg('M12 3 14.8 9.2 21.5 10l-5 4.5 1.4 6.5-5.9-3.4-5.9 3.4 1.4-6.5-5-4.5 6.7-.8L12 3Z')} />
          </div>
        </section>

        {chatOn && messages.length > 0 && (
          <section aria-label="聊天记录" className="mt-7 space-y-3">
            {messages.map((m, i) => (
              <div key={m.id ?? i} className={`flex gap-2.5 ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                {m.role === 'assistant' && (
                  <div className="mt-0.5 grid size-8 shrink-0 place-items-center overflow-hidden rounded-full bg-lamp/15">
                    <Robot spec="idle" alive={false} size={30} label={companionName} />
                  </div>
                )}
                <div className={`max-w-[78%] rounded-2xl px-3.5 py-2.5 text-sm leading-relaxed ${m.role === 'user'
                  ? 'rounded-tr-xs bg-lamp font-medium text-ink'
                  : 'rounded-tl-xs border border-window/10 bg-dusk-soft/60 text-window'}`}>
                  {m.content}
                  {m.role === 'assistant' && (
                    <button type="button" onClick={() => void speak(messageId(m, i), m.content)}
                      aria-label={speakingId === messageId(m, i) ? '停止朗读' : '朗读这条回复'}
                      className="mt-2 flex min-h-8 items-center gap-1.5 rounded-full px-2 text-xs text-window/55 transition active:scale-95 hover:bg-window/10 hover:text-window">
                      {loadingId === messageId(m, i) ? (
                        <><span className="inline-block size-3 animate-spin rounded-full border border-current border-t-transparent" aria-hidden="true" />正在生成…</>
                      ) : speakingId === messageId(m, i) ? (
                        <><svg viewBox="0 0 24 24" className="size-3.5" fill="currentColor" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1" /><rect x="14" y="5" width="4" height="14" rx="1" /></svg>停止</>
                      ) : (
                        <><svg viewBox="0 0 24 24" className="size-3.5" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M11 5 6 9H3v6h3l5 4V5Z" /><path d="M16 9a4 4 0 0 1 0 6" /></svg>听{companionName}念</>
                      )}
                    </button>
                  )}
                </div>
              </div>
            ))}
            {isStreaming && (
              <div className="flex justify-start gap-2.5">
                <div className="mt-0.5 grid size-8 shrink-0 place-items-center overflow-hidden rounded-full bg-lamp/15">
                  <Robot spec="speaking" alive={false} size={30} label={companionName} />
                </div>
                <div className="max-w-[78%] rounded-2xl rounded-tl-xs border border-lamp/30 bg-dusk-soft/80 px-3.5 py-2.5 text-sm leading-relaxed text-window">
                  {streamingText || `${companionName}正在想…`}
                  <span className="ml-1 inline-block animate-pulse text-lamp">▍</span>
                </div>
              </div>
            )}
          </section>
        )}
        {(error || speechError) && <p role="alert" className="mt-3 text-center text-xs text-blush">{error || speechError}</p>}
      </div>

      {chatOn && (
        <form className="flex shrink-0 items-center gap-2 border-t border-window/10 bg-dusk-deep/95 px-3 py-2 backdrop-blur"
          onSubmit={(e) => { e.preventDefault(); handleSend() }}>
          <input type="text" value={input} disabled={isStreaming} onChange={(e) => setInput(e.target.value)}
            placeholder={`跟${companionName}说说话…`} aria-label={`跟${companionName}说话`}
            className="h-11 flex-1 rounded-full border border-window/15 bg-dusk-soft/50 px-4 text-[15px] text-window placeholder:text-window/40 focus:border-lamp focus:outline-none disabled:opacity-60" />
          <button type="submit" disabled={!input.trim() || isStreaming}
            className="flex h-11 items-center justify-center rounded-full bg-lamp px-4 text-sm font-bold text-ink transition active:scale-95 disabled:cursor-not-allowed disabled:opacity-40">
            发送
          </button>
        </form>
      )}
    </main>
  )
}
