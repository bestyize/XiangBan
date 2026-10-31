import type { Attachment } from './types'
import type { PackItem } from './content'
import type { CommunityScheme } from './schemes'
export interface ReplyWorkRef { kind: 'pack' | 'scheme'; id: number; version?: number | null }
export type ReplyWorkCard = ReplyWorkRef & ({ available: false; content?: never } | { available: true; content: PackItem | CommunityScheme })
export interface ReplyExtras { work_refs?: ReplyWorkRef[]; attachment_ids?: string[]; attachment_prices?: { id: string; price: number }[] }
export interface ReplyContent { work_refs: ReplyWorkCard[]; attachments: Attachment[] }
export interface ReplySource { body: string; work_refs: ReplyWorkRef[]; attachments: Attachment[] }
export function replyExtras(refs: ReplyWorkRef[], files: Attachment[]): ReplyExtras {
  return { work_refs: refs, attachment_ids: files.map(a => a.id), attachment_prices: files.map(a => ({ id: a.id, price: a.price })) }
}
