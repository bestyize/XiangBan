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
  desktop_download?: string
  pack_slug?: string
}

const bundledCovers: Record<string, string> = {
  'whale-girl': 'whale-girl.webp',
  'jiangshan-map': 'jiangshan-map.webp',
  'micro-gallery': 'micro-gallery.webp',
  'plush-rabbit': 'plush-rabbit.webp',
  'starry-nightrider': 'starry-nightrider.webp',
  'little-prince': 'little-prince.webp',
  'saltline': 'saltline.webp',
  'oil-paint': 'oil-paint.webp',
}

export function ideaCoverFallback(id: string): string | undefined {
  if (Object.hasOwn(bundledCovers, id)) return `${import.meta.env.BASE_URL}ideas-covers/${bundledCovers[id]}`
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
