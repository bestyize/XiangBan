<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { schemesApi, type OwnScheme } from '@/api/schemes'
import { timeAgo } from '@/lib/format'
const items = ref<OwnScheme[]>([]), loading = ref(false), error = ref(''), removing = ref<number | null>(null)
async function load() {
  loading.value = true; error.value = ''
  try { items.value = (await schemesApi.mine()).items }
  catch (e) { error.value = e instanceof Error ? e.message : '读取自己的方案失败' }
  finally { loading.value = false }
}
async function remove(scheme: OwnScheme) {
  if (removing.value !== null || !window.confirm(`确定下架「${scheme.title}」？方案公开链接将不可访问，原作品会保留。`)) return
  removing.value = scheme.id; error.value = ''
  try { await schemesApi.remove(scheme.id); await load() }
  catch (e) { error.value = e instanceof Error ? e.message : '下架失败，请稍后重试' }
  finally { removing.value = null }
}
onMounted(load)
</script>
<template>
  <section id="my-schemes" class="my-schemes"><div class="scheme-heading"><h2>我的方案</h2><RouterLink class="btn btn-primary" to="/schemes/new">发布方案</RouterLink><button class="btn" :disabled="loading || removing !== null" @click="load">刷新</button></div>
    <p v-if="loading" class="scheme-state" role="status">正在读取方案…</p><p v-else-if="error" class="scheme-state" role="alert">{{ error }} <button class="btn" @click="load">重试</button></p><p v-else-if="!items.length" class="scheme-state">还没有发布过方案，可以用自己的已上架作品组装。</p>
    <ul v-else class="own-schemes"><li v-for="scheme in items" :key="scheme.id"><div><RouterLink v-if="scheme.status === 'published'" :to="`/schemes/${scheme.id}`"><strong>{{ scheme.title }}</strong></RouterLink><strong v-else>{{ scheme.title }}</strong><p>{{ scheme.status === 'published' ? '已发布' : '已下架' }} · {{ scheme.items.length }} 项作品 · {{ timeAgo(scheme.created_at) }}</p></div><button v-if="scheme.status === 'published'" class="btn btn-sm" :disabled="removing !== null" @click="remove(scheme)">{{ removing === scheme.id ? '正在下架…' : '下架' }}</button></li></ul>
  </section>
</template>
<style scoped>
.my-schemes { margin-top: 24px; padding: 16px; border-top: 1px solid var(--line); }
.scheme-heading { display: flex; gap: 12px; align-items: center; flex-wrap: wrap; }
h2 { margin: 0 auto 0 0; font-size: var(--fs-lg); }
.scheme-state { color: var(--ink-3); padding: 16px 0; }
.own-schemes { list-style: none; margin: 12px 0 0; padding: 0; }
.own-schemes li { display: flex; justify-content: space-between; gap: 12px; align-items: center; padding: 12px 0; border-bottom: 1px solid var(--line); }
.own-schemes p { margin: 6px 0 0; color: var(--ink-3); font-size: var(--fs-xs); }
</style>
