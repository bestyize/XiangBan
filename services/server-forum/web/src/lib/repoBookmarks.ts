import { computed, ref } from 'vue'

const key = 'xiaoban.web.repo-bookmarks'
function read(): string[] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(key) || '[]')
    return Array.isArray(value) ? [...new Set(value.filter((item): item is string => typeof item === 'string').map(item => item.toLowerCase()))].slice(0, 500) : []
  } catch { return [] }
}
const names = ref(read())
const error = ref('')
if (typeof window !== 'undefined') window.addEventListener('storage', event => {
  if (event.key === key || event.key === null) names.value = read()
})
export function useRepoBookmarks() {
  function has(name: string) { return names.value.includes(name.toLowerCase()) }
  function toggle(name: string) {
    const normalized = name.toLowerCase()
    const old = read()
    const next = old.includes(normalized) ? old.filter(item => item !== normalized) : [normalized, ...old].slice(0, 500)
    try { localStorage.setItem(key, JSON.stringify(next)); names.value = next; error.value = '' }
    catch { error.value = '浏览器无法保存收藏，请检查存储设置。' }
  }
  return { names: computed(() => names.value), has, toggle, error }
}
