import { request } from './admin'
export interface EditorialPack { id: number; title: string; slug: string; kind: string; status: string; author: { uid: string; nickname: string } | null }
export interface FeaturedItem { pack_id: number; sort_order: number; visible: boolean; pack: EditorialPack }
export interface FeaturedList { revision: number; items: FeaturedItem[] }
const root = '/community-editorial'
export const communityEditorial = {
  featured: () => request<FeaturedList>('GET', `${root}/featured`),
  candidates: (q: string) => request<{ items: EditorialPack[] }>('GET', `${root}/featured/candidates?q=${encodeURIComponent(q)}`),
  saveFeatured: (expected_revision: number, items: { pack_id: number; sort_order: number }[]) => request<FeaturedList>('PUT', `${root}/featured`, { expected_revision, items }),
}
export interface EditorialPage<T> { items: T[]; total: number; page: number; page_size: number }
export interface EditorialChallenge { key: string; title: string; description: string; starts_at: string; ends_at: string; submission_tag: string; topic_id: number | null; status: 'draft' | 'upcoming' | 'open' | 'ended' | 'closed'; publication_status: 'draft' | 'published' | 'closed'; revision: number; entry_count: number }
export interface ChallengeEntry { pack_id: number; pack: EditorialPack; featured: boolean; sort_order: number; revision: number; visible: boolean; vote_topic_id: number | null; likes: number | null }
export interface ChallengeDetail { challenge: EditorialChallenge; entries: EditorialPage<ChallengeEntry> }
export interface ChallengeInput { title: string; description: string; starts_at: string; ends_at: string; submission_tag: string; topic_id: number | null; expected_revision: number }
export const challengeEditorial = {
  list: (page = 1) => request<EditorialPage<EditorialChallenge>>('GET', `${root}/challenges?page=${page}`),
  detail: (key: string, page = 1) => request<ChallengeDetail>('GET', `${root}/challenges/${encodeURIComponent(key)}?page=${page}`),
  save: (key: string, input: ChallengeInput) => request<ChallengeDetail>('PUT', `${root}/challenges/${encodeURIComponent(key)}`, input),
  action: (key: string, action: 'publish' | 'unpublish' | 'close', expected_revision: number) => request<ChallengeDetail>('POST', `${root}/challenges/${encodeURIComponent(key)}/${action}`, { expected_revision }),
  feature: (key: string, entry: ChallengeEntry, featured: boolean) => request<{ ok: boolean }>('PATCH', `${root}/challenges/${encodeURIComponent(key)}/entries/${entry.pack_id}`, { expected_revision: entry.revision, featured, sort_order: entry.sort_order }),
}
