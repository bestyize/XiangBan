import { request } from './forum'
import type { Page } from './types'
export interface SchemeReference { pack_id: number; version: number | null; available?: boolean; title?: string; kind?: string; slug?: string }
export interface CommunityScheme { id: number; title: string; summary?: string; author: { uid: string; nickname: string | null }; items: SchemeReference[]; apply_count: number; created_at: string; status?: string }
export interface OwnScheme extends CommunityScheme { status: string }
export interface SchemeInput { title: string; summary: string; items: { pack_id: number; version: number }[] }
export const schemesApi = {
  list: (page = 1) => request<Page<CommunityScheme>>('GET', `/schemes?page=${page}`, undefined, false),
  detail: (id: number) => request<CommunityScheme>('GET', `/schemes/${id}`, undefined, false),
  mine: () => request<{ items: OwnScheme[] }>('GET', '/me/schemes'),
  publish: (input: SchemeInput) => request<{ id: number }>('POST', '/schemes', input),
  remove: (id: number) => request<{ id: number; status: string }>('POST', `/schemes/${id}/remove`),
}
