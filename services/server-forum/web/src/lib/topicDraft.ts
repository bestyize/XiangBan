import type { Attachment } from '../api/types'
import type { PostingSectionKey } from './postingStructure'

export interface UploadedTopicWork {
  source: 'uploaded'
  key: string
  kind: string
  title: string
  summary: string
  originalName: string
  format: 'zip' | 'html' | 'htm'
  size: number
  rightsAccepted: boolean
}

export interface TopicDraft {
  form: {
    sectionKey: PostingSectionKey
    boardKey: string
    title: string
    body: string
    tags: string[]
    readLevel: number
  }
  tagInput: string
  attachments: Attachment[]
  work: { source: 'mine'; id: number; title: string; kind: string; version: number } | UploadedTopicWork | null
  poll: { options: string[]; days: number } | null
  requestId: string | null
}

type DraftStorage = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>

export function topicDraftKey(owner: string, editId?: number): string {
  return `xiaoban.topic-draft.v1.${encodeURIComponent(owner)}.${editId ?? 'new'}`
}

export function readTopicDraft(key: string, storage?: DraftStorage): TopicDraft | null {
  try {
    const raw = (storage ?? localStorage).getItem(key)
    const draft = raw ? JSON.parse(raw) as TopicDraft : null
    return draft && typeof draft.form?.body === 'string' && typeof draft.form.title === 'string'
      && Array.isArray(draft.form.tags) && Array.isArray(draft.attachments) ? draft : null
  } catch { return null }
}

export function writeTopicDraft(key: string, draft: TopicDraft | null, storage?: DraftStorage): void {
  try {
    const disk = storage ?? localStorage
    if (draft) disk.setItem(key, JSON.stringify(draft))
    else disk.removeItem(key)
  } catch {
    // Private browsing and full storage still allow composing and publishing in the current page.
  }
}
