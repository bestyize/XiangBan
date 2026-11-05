import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/api/forum'
import type { Category } from '@/api/types'
import { CATEGORY_ALIASES, navigationSections } from '@/lib/communityStructure'

/** 帖子数达到这个值的二级分类才在导航里单独露出，其余合并进一级分类。 */
export const SHOW_CHILD_THRESHOLD = 20

export const useMeta = defineStore('meta', () => {
  const categories = ref<Category[]>([])
  let loading: Promise<void> | null = null

  const navigation = computed(()=>navigationSections(categories.value))

  const bySlug = computed(() => {
    const map = new Map(categories.value.map(c => [c.slug, c]))
    for (const [old, name] of Object.entries(CATEGORY_ALIASES)) { const category=map.get(name); if(category) map.set(old,category) }
    return map
  })
  const byId = computed(() => new Map(categories.value.map(c => [c.id, c])))

  function load(force = false) {
    if (loading && !force) return loading
    loading = api
      .categories()
      .then(c => {
        categories.value = c
      })
      .catch(() => {
        loading = null
      })
    return loading
  }

  return { categories, navigation, bySlug, byId, load }
})
