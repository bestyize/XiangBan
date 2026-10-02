import { useEffect, useRef, useState } from 'react'
import { useLocation, useRoute } from 'wouter'
import { Screen } from '../../components/ui.tsx'
import { GAME_RUNTIME_PROTOCOL, gameEntryUrl, loadGameManifest, type GamePackManifest } from '../../games/packLoader.ts'

const HOST_CHANNEL = 'xiaban.game-host'
const PACK_CHANNEL = 'xiaban.game-pack'

function createNonce(): string {
  try {
    return crypto.randomUUID()
  } catch {
    return Date.now().toString(36) + Math.random().toString(36).slice(2)
  }
}

export default function GamePlayer() {
  const [, params] = useRoute('/games/:gameId')
  const [, navigate] = useLocation()
  const gameId = params?.gameId ?? ''
  const [nonce] = useState(createNonce)
  const [pack, setPack] = useState<GamePackManifest | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [ready, setReady] = useState(false)
  const iframeRef = useRef<HTMLIFrameElement>(null)

  useEffect(() => {
    setPack(null)
    setError(null)
    setReady(false)
    if (!gameId) {
      setError('没有找到这款游戏')
      return
    }
    const controller = new AbortController()
    loadGameManifest(gameId, controller.signal)
      .then(setPack)
      .catch((cause: unknown) => {
        if (cause instanceof Error && cause.name === 'AbortError') return
        setError(cause instanceof Error ? cause.message : '游戏包暂时无法加载')
      })
    return () => controller.abort()
  }, [gameId])

  useEffect(() => {
    function receiveMessage(event: MessageEvent) {
      if (event.source !== iframeRef.current?.contentWindow) return
      const message: unknown = event.data
      if (!message || typeof message !== 'object') return
      const data = message as Record<string, unknown>
      if (
        data.channel !== PACK_CHANNEL ||
        data.protocol !== GAME_RUNTIME_PROTOCOL ||
        data.nonce !== nonce
      ) return

      if (data.type === 'game.ready') setReady(true)
      if (data.type === 'game.exit') navigate('/games')
    }

    window.addEventListener('message', receiveMessage)
    return () => window.removeEventListener('message', receiveMessage)
  }, [navigate, nonce])

  function initializeGame() {
    iframeRef.current?.contentWindow?.postMessage({
      channel: HOST_CHANNEL,
      protocol: GAME_RUNTIME_PROTOCOL,
      type: 'host.init',
      nonce,
    }, '*')
  }

  return (
    <Screen className="px-3" style={{ height: "100dvh", minHeight: 0, overflow: "hidden", paddingBottom: "var(--safe-bottom)" }}>
      <div className="flex min-h-0 flex-1 flex-col">
        <header className="flex h-10 shrink-0 items-center justify-between">
          <button type="button" onClick={() => navigate('/games')} className="text-sm text-window/70">
            ‹ 退出游戏
          </button>
          <span className="text-sm text-lamp">{pack?.name ?? '游戏'}</span>
          <span className="w-16" aria-hidden="true" />
        </header>

        {error ? (
          <div className="mt-8 rounded-2xl bg-dusk-soft/35 p-5">
            <p role="alert" className="text-sm text-blush">{error}</p>
            <button type="button" className="mt-4 text-sm text-lamp" onClick={() => navigate('/games')}>
              返回游戏广场
            </button>
          </div>
        ) : pack ? (
          <div className="relative mt-2 min-h-0 flex-1 overflow-hidden rounded-2xl border border-window/10 bg-dusk-deep">
            {!ready && (
              <div className="absolute inset-0 z-10 flex items-center justify-center bg-dusk-deep text-sm text-window/65">
                正在加载游戏…
              </div>
            )}
            <iframe
              ref={iframeRef}
              title={pack.name}
              src={gameEntryUrl(pack, nonce)}
              sandbox="allow-scripts"
              onLoad={initializeGame}
              className="absolute inset-0 block h-full w-full border-0"
            />
          </div>
        ) : (
          <p className="mt-8 text-sm text-window/55">正在读取游戏包…</p>
        )}
      </div>
    </Screen>
  )
}
