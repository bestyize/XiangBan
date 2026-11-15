export interface FeaturedIdea {
  id: string
  kind: 'gallery' | 'exhibit' | 'work'
  title: string
  description: string
  introduction?: string
  controls?: { desktop?: string; mobile?: string }
  cover: string
  images?: string[]
  label?: string
  author: string
  date: string
  href: string
  player?: string
  site?: string
  repository?: string
  pack_slug?: string
}

export async function loadFeaturedIdeas(): Promise<FeaturedIdea[]> {
  try {
    const response = await fetch('/ideas-catalog.json', { cache: 'no-cache', credentials: 'omit' })
    if (!response.ok) throw new Error()
    const catalog: { items: FeaturedIdea[] } = await response.json()
    if (!Array.isArray(catalog.items)) throw new Error()
    return catalog.items
  } catch {
    throw new Error('精选作品加载失败，请重试')
  }
}
