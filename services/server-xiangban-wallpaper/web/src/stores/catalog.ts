import { ref } from 'vue'
import { defineStore } from 'pinia'
import { listCategories, listProviders, searchWallpapers } from '@/api'
import type { Category, ProviderInfo, WallpaperItem } from '@/types'

const COVER_KEY = 'wc:category-covers:v1'
const COVER_TTL_MS = 24 * 60 * 60 * 1000

interface CoverEntry {
  at: number
  item: WallpaperItem
}

function readCovers(): Record<string, CoverEntry> {
  try {
    return JSON.parse(localStorage.getItem(COVER_KEY) ?? '{}')
  } catch {
    return {}
  }
}

export const useCatalog = defineStore('catalog', () => {
  const categories = ref<Category[]>([])
  const providers = ref<ProviderInfo[]>([])
  const covers = ref<Record<string, WallpaperItem>>({})
  let coversStarted = false

  async function load() {
    const [cats, provs] = await Promise.allSettled([listCategories(), listProviders()])
    if (cats.status === 'fulfilled') categories.value = cats.value
    if (provs.status === 'fulfilled') providers.value = provs.value
  }

  /**
   * One image per category. Fetched from a single provider and cached for a day, because
   * free API tiers (Unsplash allows ~50 requests/hour) cannot afford a dozen lookups per visit.
   */
  async function loadCovers() {
    if (coversStarted || categories.value.length === 0) return
    coversStarted = true

    const cached = readCovers()
    const now = Date.now()
    const fresh: Record<string, WallpaperItem> = {}
    const missing: Category[] = []
    for (const c of categories.value) {
      const hit = cached[c.id]
      if (hit && now - hit.at < COVER_TTL_MS) fresh[c.id] = hit.item
      else missing.push(c)
    }
    covers.value = fresh
    if (missing.length === 0) return

    const configured = providers.value.filter((p) => p.is_configured && p.supported_media_types.includes('image'))
    const preferred = ['pixabay', 'pexels', 'unsplash', 'demo']
    const source = preferred.find((id) => configured.some((p) => p.id === id))
    if (!source) return

    const store = { ...cached }
    for (const c of missing) {
      try {
        const res = await searchWallpapers(
          { category: c.id, media: 'image', orientation: 'landscape', provider: source },
          1,
          1,
        )
        const item = res.items[0]
        if (!item) continue
        covers.value = { ...covers.value, [c.id]: item }
        store[c.id] = { at: Date.now(), item }
      } catch {
        // a missing cover just leaves that tile as a plain color block
      }
    }
    try {
      localStorage.setItem(COVER_KEY, JSON.stringify(store))
    } catch {
      // storage full or unavailable; covers will be refetched next visit
    }
  }

  return { categories, providers, covers, load, loadCovers }
})
