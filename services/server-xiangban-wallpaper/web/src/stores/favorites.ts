import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { addFavorite, listFavorites, removeFavorite } from '@/api'
import { toast } from '@/composables/toast'
import type { WallpaperItem } from '@/types'

export const useFavorites = defineStore('favorites', () => {
  const items = ref<WallpaperItem[]>([])
  const loaded = ref(false)
  const ids = computed(() => new Set(items.value.map((i) => i.id)))

  async function load() {
    try {
      items.value = await listFavorites()
      loaded.value = true
    } catch {
      toast('收藏列表加载失败')
    }
  }

  const has = (id: string) => ids.value.has(id)

  /** Updates the UI first and rolls back if the server rejects the change. */
  async function toggle(item: WallpaperItem) {
    const before = items.value
    if (has(item.id)) {
      items.value = before.filter((i) => i.id !== item.id)
      try {
        await removeFavorite(item.id)
        toast('已取消收藏')
      } catch {
        items.value = before
        toast('取消收藏失败')
      }
    } else {
      items.value = [item, ...before]
      try {
        await addFavorite(item)
        toast('已加入收藏')
      } catch {
        items.value = before
        toast('收藏失败')
      }
    }
  }

  return { items, loaded, has, load, toggle }
})
