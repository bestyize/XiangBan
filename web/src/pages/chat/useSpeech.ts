import { useCallback, useEffect, useRef, useState } from 'react'
import { getToken } from '../../api/client.ts'

/**
 * Plays 小伴's replies aloud.
 *
 * The audio comes from our backend (which calls the Edge read-aloud service), not from the
 * browser's own speech synthesis: Web Speech is unavailable or silent in most Android
 * WebViews and WeChat, while a plain <audio> element plays MP3 everywhere.
 */
export function useSpeech() {
  const [speakingId, setSpeakingId] = useState<string | null>(null)
  const [loadingId, setLoadingId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const audioRef = useRef<HTMLAudioElement | null>(null)
  const urlRef = useRef<string | null>(null)

  const stop = useCallback(() => {
    const audio = audioRef.current
    if (audio) {
      audio.pause()
      audioRef.current = null
    }
    if (urlRef.current) {
      URL.revokeObjectURL(urlRef.current)
      urlRef.current = null
    }
    setSpeakingId(null)
    setLoadingId(null)
  }, [])

  // Release the object URL and stop playback when the page goes away.
  useEffect(() => stop, [stop])

  const speak = useCallback(async (id: string, text: string) => {
    // Tapping the line that is already playing stops it.
    if (speakingId === id) {
      stop()
      return
    }
    stop()
    setError(null)
    setLoadingId(id)
    try {
      const token = getToken()
      const headers: Record<string, string> = { 'Content-Type': 'application/json' }
      if (token) headers.Authorization = `Bearer ${token}`

      const response = await fetch('/api/chat/speak', {
        method: 'POST',
        headers,
        body: JSON.stringify({ text }),
      })
      if (!response.ok) throw new Error('语音暂时发不出来，稍后再试')

      const blob = await response.blob()
      const url = URL.createObjectURL(blob)
      urlRef.current = url

      const audio = new Audio(url)
      audioRef.current = audio
      // A blocked or undecodable source surfaces here; without this the button would
      // just silently do nothing.
      audio.onended = () => stop()
      audio.onerror = () => {
        stop()
        setError('这段语音播放不了，请检查浏览器的媒体权限或网络')
      }
      setLoadingId(null)
      setSpeakingId(id)
      await audio.play()
    } catch (e) {
      stop()
      setError(e instanceof Error ? e.message : '语音暂时发不出来，稍后再试')
    }
  }, [speakingId, stop])

  return { speak, stop, speakingId, loadingId, error }
}
