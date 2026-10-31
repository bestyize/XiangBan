import { request } from './forum'
import type { PackItem } from './content'
import type { Page } from './types'
export interface CommunityChallenge { key: string; title: string; description: string; starts_at: string; ends_at: string; submission_tag: string; topic_id: number | null; status: 'upcoming' | 'open' | 'ended' | 'closed'; entry_count: number; created_at: string; updated_at: string }
export interface CommunityChallengeEntry { pack_id: number; pack: PackItem; featured: boolean; sort_order: number; vote_method: 'topic_like'; vote_topic_id: number | null; likes: number | null }
export interface CommunityChallengeDetail { challenge: CommunityChallenge; entries: Page<CommunityChallengeEntry> }
export const challengesApi = {
  list: (page = 1) => request<Page<CommunityChallenge>>('GET', `/challenges?page=${page}`, undefined, false),
  detail: (key: string, page = 1) => request<CommunityChallengeDetail>('GET', `/challenges/${encodeURIComponent(key)}?page=${page}`, undefined, false),
  submit: (key: string, pack_id: number) => request<{ status: string; pack_id: number }>('POST', `/challenges/${encodeURIComponent(key)}/entries`, { pack_id }),
}
