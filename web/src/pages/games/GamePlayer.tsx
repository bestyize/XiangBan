import { useEffect, useRef, useState } from 'react'
import { useLocation, useRoute } from 'wouter'
import { Screen } from '../../components/ui.tsx'
import { GAME_RUNTIME_PROTOCOL, gameEntryUrl, loadGameManifest, type GamePackManifest } from '../../games/packLoader.ts'
import { api, getToken } from '../../api/client.ts'
import { storage } from '../../lib/storage.ts'

interface Round { id: string | null; score: number; lines: number; maxTile: number; finished: boolean; timer: ReturnType<typeof setTimeout> | null }

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
  const roundRef = useRef<Round | null>(null)
  const [recordError, setRecordError] = useState<string | null>(null)

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
    async function persist(round: Round, keepalive = false) {
      if (!round.id) return
      if (!getToken()) {
        try {
          const raw = storage.get('xb_guest_games') || '{}'
          const records = JSON.parse(raw) as Record<string, { sessions: number; bestScore: number; bestTile: number }>
          const current = records[gameId] || { sessions: 0, bestScore: 0, bestTile: 0 }
          if (round.finished) current.sessions += 1
          if (round.score > current.bestScore) current.bestScore = round.score
          if (round.maxTile > current.bestTile) current.bestTile = round.maxTile
          records[gameId] = current
          storage.set('xb_guest_games', JSON.stringify(records))
        } catch {}
        return
      }
      try { await api(`/api/games/sessions/${round.id}`, { method: 'PUT', keepalive, body: { score: round.score, lines: round.lines, maxTile: round.maxTile, finished: round.finished } }) }
      catch { if (round === roundRef.current) setRecordError('游戏记录暂未保存，网络恢复后可继续重试。') }
    }
    function finish(round: Round) {
      if (round.finished) return
      if (round.timer) clearTimeout(round.timer)
      round.timer = null; round.finished = true
      void persist(round, true)
    }
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
      if (data.type === 'game.start') {
        if (roundRef.current) finish(roundRef.current)
        const round: Round = { id: null, score: 0, lines: 0, maxTile: 0, finished: false, timer: null }
        roundRef.current = round; setRecordError(null)
        if (!getToken()) {
          round.id = `guest_${Date.now()}`
        } else {
          void api<{ id: string }>('/api/games/sessions', { method: 'POST', body: { gameId } }).then((session) => {
            round.id = session.id; void persist(round)
          }).catch(() => { if (round === roundRef.current) setRecordError('对局记录暂未建立，请检查网络。') })
        }
      }
      if (data.type === 'game.progress' || data.type === 'game.result') {
        const round = roundRef.current
        if (!round) return
        const score = Number(data.score); const lines = Number(data.lines ?? 0); const tile = Number(data.maxTile ?? 0)
        if (!Number.isInteger(score) || score < 0 || score > 1e9 || !Number.isInteger(lines) || lines < 0 || lines > 1e6 || !Number.isInteger(tile) || tile < 0 || tile > 1073741824) return
        round.score = score; round.lines = lines; round.maxTile = tile
        if (data.type === 'game.result') finish(round)
        else if (!round.timer) round.timer = setTimeout(() => { round.timer = null; void persist(round) }, 1200)
      }
      if (data.type === 'game.exit') {
        if (roundRef.current) finish(roundRef.current)
        navigate('/games')
      }
    }

    window.addEventListener('message', receiveMessage)
    const hide = () => { if (document.visibilityState === 'hidden' && roundRef.current) void persist(roundRef.current, true) }
    document.addEventListener('visibilitychange', hide)
    return () => { window.removeEventListener('message', receiveMessage); document.removeEventListener('visibilitychange', hide); if (roundRef.current) finish(roundRef.current) }
  }, [navigate, nonce, gameId])

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
        {recordError && <p role="status" className="text-[11px] text-window/50">{recordError}</p>}

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
