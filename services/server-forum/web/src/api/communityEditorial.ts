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
