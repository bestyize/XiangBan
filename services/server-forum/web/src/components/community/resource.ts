import { onScopeDispose, ref, shallowRef } from 'vue'
import { ApiError } from '@/api/forum'
import { createRequestGate } from './contract'

/** Every load clears the previous result; stale responses cannot replace a new route/filter. */
export function useCommunityResource<T>(fetcher: () => Promise<T>) {
  const data = shallowRef<T | null>(null)
  const loading = ref(false)
  const error = ref('')
  const gate = createRequestGate()
  async function load() {
    const id = gate.next()
    data.value = null; error.value = ''; loading.value = true
    try {
      const result = await fetcher()
      if (gate.current(id)) data.value = result
    } catch (e) {
      if (gate.current(id)) error.value = e instanceof ApiError && e.status === 404 ? '内容尚未发布或已不可用' : e instanceof Error ? e.message : '加载失败，请重试'
    } finally { if (gate.current(id)) loading.value = false }
  }
  onScopeDispose(() => gate.cancel())
  return { data, loading, error, load }
}
