import { useEffect, useMemo, useRef, useState } from 'react'
import { Robot } from '../../components/Robot.tsx'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { useProfile } from '../../lib/session.tsx'
import { getScenario } from './ChatScenarios.ts'
import { useChat } from './useChat.ts'
import { useSpeech } from './useSpeech.ts'

const personalityName = {
  toxic: '毒舌心软',
  gentle: '温柔治愈',
  chatty: '元气话痨',
  tsundere: '高冷傲娇',
} as const

/** Stable key for a message row, used to track which reply is being spoken. */
function messageId(message: { id?: number }, index: number): string {
  return message.id != null ? `id-${message.id}` : `idx-${index}`
}

export default function CompanionRoom({ active = true }: { active?: boolean }) {
  const profile = useProfile()
  const { messages, streamingText, isStreaming, error, send, clear } = useChat(active)
  const { speak, speakingId, loadingId, error: speechError } = useSpeech()
  const [input, setInput] = useState('')
  const [toast, setToast] = useState<string | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)
  const scenario = useMemo(() => getScenario(), [])

  const companionName = profile.companionName || '小伴'
  const personalityLabel = personalityName[profile.personality] || '温柔治愈'

  // Auto-scroll to bottom on new messages or streaming chunk
  useEffect(() => {
    const el = scrollRef.current
    if (el) {
      el.scrollTop = el.scrollHeight
    }
  }, [messages, streamingText])

  function handleSend(textToSend?: string) {
    const text = textToSend ?? input
    if (!text.trim() || isStreaming) return
    void send(text)
    if (!textToSend) setInput('')
  }

  function showToast(msg: string) {
    setToast(msg)
    window.setTimeout(() => setToast(null), 2500)
  }

  return (
    <main
      className="mx-auto flex h-dvh w-full max-w-[480px] flex-col overflow-hidden bg-dusk-deep text-window"
      style={{
        paddingTop: 'calc(var(--safe-top) + 12px)',
        paddingBottom: TAB_BAR_SPACE,
      }}
    >
      {/* Top Bar */}
      <header className="flex shrink-0 items-center justify-between border-b border-window/10 px-5 pb-3">
        <div className="flex items-center gap-2.5">
          <span className="size-2 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]" aria-hidden="true" />
          <div>
            <h1 className="text-base font-bold leading-none text-window">{companionName}</h1>
            <span className="mt-1 block text-[11px] text-window/50">{personalityLabel} · 在家等你</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {messages.length > 0 && (
            <button
              type="button"
              onClick={() => {
                if (window.confirm('要清空和小伴的对话记录吗？')) void clear()
              }}
              className="rounded-full px-2.5 py-1 text-xs text-window/50 hover:bg-window/10 hover:text-window"
            >
              清空
            </button>
          )}
        </div>
      </header>

      {/* Chat scroll body */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto overscroll-contain px-4 py-3">
        {/* Companion Hero & Scenario Greeting */}
        <div className="my-2 rounded-2xl border border-lamp/20 bg-gradient-to-b from-dusk to-dusk-deep p-4 text-center">
          <div className="flex justify-center">
            <Robot spec="greeting" size={96} label={companionName} interactive autoSleep
              thinking={isStreaming && !streamingText} speaking={(isStreaming && Boolean(streamingText)) || speakingId !== null} />
          </div>
          <div className="mt-2 inline-flex items-center gap-1.5 rounded-full bg-lamp/15 px-2.5 py-0.5 text-[11px] font-medium text-lamp">
            <span>🌙 {scenario.periodName}</span>
          </div>
          <p className="mt-2 text-xs leading-relaxed text-window/80">{scenario.greeting}</p>

          {/* Quick Scenario Chips */}
          <div className="mt-3 flex flex-wrap justify-center gap-1.5">
            {scenario.chips.map((chip) => (
              <button
                key={chip}
                type="button"
                disabled={isStreaming}
                onClick={() => handleSend(chip)}
                className="rounded-full border border-window/15 bg-window/5 px-2.5 py-1 text-xs text-window/75 transition active:scale-95 active:bg-lamp/20 hover:border-lamp/40 hover:text-window disabled:opacity-50"
              >
                {chip}
              </button>
            ))}
          </div>
        </div>

        {/* Message history */}
        <div className="space-y-3 pt-2">
          {messages.map((m, i) => (
            <div
              key={m.id ?? i}
              className={`flex gap-2.5 ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              {m.role === 'assistant' && (
                <div className="mt-0.5 size-7 shrink-0 overflow-hidden rounded-full bg-lamp/20">
                  <Robot spec="idle" alive={false} size={28} label={companionName} />
                </div>
              )}
              <div
                className={`max-w-[78%] rounded-2xl px-3.5 py-2.5 text-sm leading-relaxed ${
                  m.role === 'user'
                    ? 'rounded-tr-xs bg-lamp font-medium text-ink shadow-xs'
                    : 'rounded-tl-xs border border-window/10 bg-dusk-soft/60 text-window shadow-xs'
                }`}
              >
                {m.content}
                {m.role === 'assistant' && (
                  <button
                    type="button"
                    onClick={() => void speak(messageId(m, i), m.content)}
                    aria-label={speakingId === messageId(m, i) ? '停止朗读' : '朗读这条回复'}
                    className="mt-2 flex min-h-8 items-center gap-1.5 rounded-full px-2 text-xs text-window/55 transition active:scale-95 hover:bg-window/10 hover:text-window"
                  >
                    {loadingId === messageId(m, i) ? (
                      <>
                        <span className="inline-block size-3 animate-spin rounded-full border border-current border-t-transparent" aria-hidden="true" />
                        正在生成…
                      </>
                    ) : speakingId === messageId(m, i) ? (
                      <>
                        <svg viewBox="0 0 24 24" className="size-3.5" fill="currentColor" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1" /><rect x="14" y="5" width="4" height="14" rx="1" /></svg>
                        停止
                      </>
                    ) : (
                      <>
                        <svg viewBox="0 0 24 24" className="size-3.5" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                          <path d="M11 5 6 9H3v6h3l5 4V5Z" /><path d="M16 9a4 4 0 0 1 0 6" />
                        </svg>
                        听小伴念
                      </>
                    )}
                  </button>
                )}
              </div>
            </div>
          ))}

          {/* Live streaming bubble */}
          {isStreaming && (
            <div className="flex justify-start gap-2.5">
              <div className="mt-0.5 size-7 shrink-0 overflow-hidden rounded-full bg-lamp/20">
                <Robot spec="speaking" alive={false} size={28} label={companionName} />
              </div>
              <div className="max-w-[78%] rounded-2xl rounded-tl-xs border border-lamp/30 bg-dusk-soft/80 px-3.5 py-2.5 text-sm leading-relaxed text-window shadow-xs">
                {streamingText || '小伴正在想…'}
                <span className="ml-1 inline-block animate-pulse text-lamp">▍</span>
              </div>
            </div>
          )}
        </div>

        {error && (
          <p role="alert" className="mt-3 text-center text-xs text-blush">
            {error}
          </p>
        )}
        {speechError && (
          <p role="alert" className="mt-3 text-center text-xs text-blush">
            {speechError}
          </p>
        )}
      </div>

      {/* Input bar */}
      <div className="shrink-0 border-t border-window/10 bg-dusk-deep/95 px-3 py-2 backdrop-blur">
        {toast && (
          <div className="mb-2 text-center text-xs text-lamp transition-all">
            {toast}
          </div>
        )}
        <form
          className="flex items-center gap-2"
          onSubmit={(e) => {
            e.preventDefault()
            handleSend()
          }}
        >
          {/* Voice mode toggle (prepared for Phase 2 TTS/ASR) */}
          <button
            type="button"
            onClick={() => showToast('🎙️ 语音对讲功能正在准备中，先跟小伴打字聊聊吧～')}
            className="flex size-10 shrink-0 items-center justify-center rounded-full border border-window/20 text-window/70 transition active:scale-95 hover:text-window"
            aria-label="语音模式"
          >
            <svg viewBox="0 0 24 24" className="size-5" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="22" />
            </svg>
          </button>

          <input
            type="text"
            value={input}
            disabled={isStreaming}
            onChange={(e) => setInput(e.target.value)}
            placeholder={`跟${companionName}说说话…`}
            className="h-10 flex-1 rounded-full border border-window/15 bg-dusk-soft/50 px-4 text-sm text-window placeholder:text-window/40 focus:border-lamp focus:outline-none disabled:opacity-60"
          />

          <button
            type="submit"
            disabled={!input.trim() || isStreaming}
            className="flex h-10 items-center justify-center rounded-full bg-lamp px-4 text-sm font-bold text-ink transition active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
          >
            发送
          </button>
        </form>
      </div>
    </main>
  )
}
