import { useCallback, useEffect, useRef, useState } from 'react'
import { api, getToken } from '../../api/client.ts'
import { storage } from '../../lib/storage.ts'

export interface ChatMessage {
  id?: number
  role: 'user' | 'assistant'
  content: string
  createdAt?: number
}

const GUEST_KEY = 'xb_guest_chat'

export function useChat(active = true) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [streamingText, setStreamingText] = useState('')
  const [isStreaming, setIsStreaming] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  const loadHistory = useCallback(async () => {
    if (!getToken()) {
      try {
        const raw = storage.get(GUEST_KEY)
        setMessages(raw ? JSON.parse(raw) : [])
      } catch {
        setMessages([])
      }
      return
    }
    setLoading(true)
    setError(null)
    try {
      const res = await api<{ messages: ChatMessage[] }>('/api/chat/messages')
      setMessages(res.messages)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (active) void loadHistory()
  }, [active, loadHistory])

  const send = useCallback(async (content: string) => {
    const text = content.trim()
    if (!text || isStreaming) return

    const userMsg: ChatMessage = { role: 'user', content: text, createdAt: Date.now() }
    const nextList = [...messages, userMsg]
    setMessages(nextList)
    setStreamingText('')
    setIsStreaming(true)
    setError(null)

    abortRef.current = new AbortController()
    let accumulated = ''

    try {
      const token = getToken()
      const headers: Record<string, string> = { 'Content-Type': 'application/json' }
      if (token) headers.Authorization = `Bearer ${token}`

      const payload = {
        message: text,
        guestHistory: token ? undefined : nextList.slice(-10),
      }

      const res = await fetch('/api/chat/stream', {
        method: 'POST',
        headers,
        body: JSON.stringify(payload),
        signal: abortRef.current.signal,
      })

      if (!res.ok) {
        throw new Error('小伴没能连上网络，稍后再试试吧')
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('流式读取不可用')

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { value, done } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() ?? ''

        for (const line of lines) {
          const trimmed = line.trim()
          if (!trimmed || !trimmed.startsWith('data:')) continue
          const data = trimmed.replace(/^data:\s*/, '')
          if (data === '[DONE]') break

          try {
            const parsed = JSON.parse(data) as { delta?: string }
            if (parsed.delta) {
              accumulated += parsed.delta
              setStreamingText(accumulated)
            }
          } catch {
            // Ignore malformed chunks
          }
        }
      }

      const assistantMsg: ChatMessage = {
        role: 'assistant',
        content: accumulated || '小伴伸了个懒腰，刚才有些走神了～',
        createdAt: Date.now(),
      }
      const finalList = [...nextList, assistantMsg]
      setMessages(finalList)
      if (!token) {
        storage.set(GUEST_KEY, JSON.stringify(finalList.slice(-30)))
      }
    } catch (e) {
      if ((e as Error).name !== 'AbortError') {
        const errMsg = (e as Error).message || '发送失败，请重试'
        setError(errMsg)
      }
    } finally {
      setIsStreaming(false)
      setStreamingText('')
      abortRef.current = null
    }
  }, [messages, isStreaming])

  const clear = useCallback(async () => {
    if (abortRef.current) {
      abortRef.current.abort()
    }
    setMessages([])
    setStreamingText('')
    setIsStreaming(false)
    setError(null)

    if (!getToken()) {
      storage.remove(GUEST_KEY)
      return
    }

    try {
      await api('/api/chat/messages', { method: 'DELETE' })
    } catch (e) {
      setError((e as Error).message)
    }
  }, [])

  return {
    messages,
    streamingText,
    isStreaming,
    loading,
    error,
    send,
    clear,
    reload: loadHistory,
  }
}
