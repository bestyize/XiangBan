import { reactive } from 'vue'
import { openSourceCatalog as initial, type OpenSourceCatalog } from '@/lib/openSource'
import { request, ApiError } from '@/api/forum'

export const openSourceCatalog = reactive<OpenSourceCatalog>(structuredClone(initial))
let pending: Promise<void> | null = null
let loaded = false
/** The bundled directory keeps older deployments readable; a published snapshot is authoritative. */
export function loadOpenSourceCatalog(force = false): Promise<void> {
  if (pending) return pending
  if (loaded && !force) return Promise.resolve()
  pending = request<OpenSourceCatalog>('GET', '/open-source', undefined, false).then(value => {
    if (!Array.isArray(value.projects) || !Array.isArray(value.categories) || !Array.isArray(value.collections)) throw new Error('精选目录格式无效')
    Object.assign(openSourceCatalog, value); loaded = true
  }).catch(error => {
    if (error instanceof ApiError && error.status === 404) return
    // Keep the last readable snapshot on transient failures, never replace it with partial data.
  }).finally(() => { pending = null })
  return pending
}
