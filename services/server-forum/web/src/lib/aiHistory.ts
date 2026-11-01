export interface AiMessage { role: 'user' | 'assistant'; content: string }
export interface AiConversation { id: string; title: string; messages: AiMessage[] }
export const AI_HISTORY_LIMITS = { conversations: 30, messages: 100, content: 64_000, serialized: 1_000_000 } as const
const prefix = 'xiangban.ai.history.v1.'
type HistoryStorage = Pick<Storage, 'getItem' | 'setItem'>

/** Accessing the localStorage property itself can throw in restricted browsers. */
export function aiHistoryStorage(access: () => HistoryStorage = () => globalThis.localStorage): HistoryStorage | null {
  try { return access() } catch { return null }
}
export function boundedAiHistory(value: unknown): AiConversation[] {
  if (!Array.isArray(value)) return []
  const result: AiConversation[] = []
  for (const raw of value.slice(0, AI_HISTORY_LIMITS.conversations)) {
    if (!raw || typeof raw.id !== 'string' || typeof raw.title !== 'string' || !Array.isArray(raw.messages)) continue
    if (!raw.messages.every((m: unknown) => m && typeof m === 'object' && ('role' in m) && (m.role === 'user' || m.role === 'assistant') && 'content' in m && typeof m.content === 'string')) continue
    const messages: AiMessage[] = raw.messages.slice(-AI_HISTORY_LIMITS.messages).map((m: AiMessage) => ({ role: m.role, content: m.content.slice(0, AI_HISTORY_LIMITS.content) }))
    if (messages[0]?.role === 'assistant') messages.shift()
    const conversation = { id:raw.id.slice(0,100), title:raw.title.slice(0,80), messages }
    result.push(conversation)
    // Preserve complete recent messages. Older conversations are dropped before newer ones.
    if (JSON.stringify(result).length > AI_HISTORY_LIMITS.serialized) {
      if (result.length > 1) { result.pop(); break }
      while (conversation.messages.length && JSON.stringify(result).length > AI_HISTORY_LIMITS.serialized) {
        conversation.messages.shift()
        if (conversation.messages[0]?.role === 'assistant') conversation.messages.shift()
      }
    }
  }
  return result
}
export function readAiHistory(storage: HistoryStorage | null, uid: string): AiConversation[] {
  if (!storage || !uid) return []
  try {
    const raw = storage.getItem(prefix + uid) || '[]'
    if (raw.length > AI_HISTORY_LIMITS.serialized) return []
    return boundedAiHistory(JSON.parse(raw))
  } catch { return [] }
}
export function writeAiHistory(storage: HistoryStorage | null, uid: string, conversations: AiConversation[]): boolean {
  if (!storage || !uid) return false
  try { storage.setItem(prefix + uid, JSON.stringify(boundedAiHistory(conversations))); return true } catch { return false }
}
