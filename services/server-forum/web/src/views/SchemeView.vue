<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import SchemeReferences from '@/components/SchemeReferences.vue'
import { schemesApi, type CommunityScheme } from '@/api/schemes'
import { ApiError } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
import { timeAgo } from '@/lib/format'
const route = useRoute(), router = useRouter(), session = useSession()
const scheme = ref<CommunityScheme | null>(null), loading = ref(false), missing = ref(false), error = ref(''), busy = ref(false), copied = ref(false), copyError = ref(false)
const owner = computed(() => scheme.value?.author.uid === session.user?.uid)
const link = computed(() => typeof window === 'undefined' ? '' : `${window.location.origin}/schemes/${scheme.value?.id ?? ''}`)
useHead(() => ({ title: scheme.value?.title ?? '方案详情' }))
let sequence = 0
async function load() {
  const current = ++sequence
  scheme.value = null; loading.value = true; missing.value = false; error.value = ''; copied.value = false; copyError.value = false
  try {
    const id = Number(route.params.id)
    if (!Number.isSafeInteger(id) || id <= 0) { missing.value = true; return }
    const data = await schemesApi.detail(id)
    if (current === sequence) scheme.value = data
  } catch (e) { if (current === sequence) { if (e instanceof ApiError && e.status === 404) missing.value = true; else error.value = e instanceof Error ? e.message : '方案加载失败' } }
  finally { if (current === sequence) loading.value = false }
}
async function copy() { try { await navigator.clipboard.writeText(link.value); copied.value = true; copyError.value = false } catch { copyError.value = true } }
async function remove() {
  if (!scheme.value || !window.confirm(`确定下架「${scheme.value.title}」？下架后公开链接不再可访问，引用的原作品会保留。`)) return
  busy.value = true; error.value = ''
  try { await schemesApi.remove(scheme.value.id); await router.replace('/me/works?tab=schemes') }
  catch (e) { error.value = e instanceof Error ? e.message : '下架失败，请稍后重试' }
  finally { busy.value = false }
}
watch(() => route.params.id, load, { immediate: true })
</script>
<template>
  <ContentShell>
    <nav class="scheme-nav"><RouterLink to="/schemes">全部方案</RouterLink><RouterLink to="/schemes/new">发布方案</RouterLink></nav>
    <p v-if="loading" class="state" role="status">正在读取方案…</p>
    <p v-else-if="missing" class="state">方案不存在或已下架。<RouterLink to="/schemes">查看其他方案</RouterLink></p>
    <p v-else-if="!scheme && error" class="state" role="alert">{{ error }} <button class="btn" @click="load">重试</button></p>
    <article v-else-if="scheme" class="scheme-detail">
      <h1>{{ scheme.title }}</h1><p class="meta"><RouterLink :to="`/u/${encodeURIComponent(scheme.author.uid)}`">{{ scheme.author.nickname || '社区作者' }}</RouterLink><span>{{ timeAgo(scheme.created_at) }}</span><span>{{ scheme.items.length }} 项作品</span><span v-if="scheme.apply_count > 0">{{ scheme.apply_count }} 次复刻</span></p>
      <p v-if="scheme.summary" class="summary">{{ scheme.summary }}</p>
      <h2>方案中的作品</h2><SchemeReferences :items="scheme.items" />
      <p class="hint">版本由作者发布方案时选定。当前不可用的作品会保留占位。</p>
      <div class="scheme-actions"><button class="btn" type="button" @click="copy">{{ copied ? '链接已复制' : '复制方案链接' }}</button><button v-if="owner" class="btn" :disabled="busy" @click="remove">{{ busy ? '正在下架…' : '下架自己的方案' }}</button></div>
      <p v-if="error" role="alert">{{ error }} <button class="btn" :disabled="busy" @click="load">重新读取</button></p>
      <label v-if="copyError" class="field"><span>请手动复制方案链接</span><input class="input" readonly :value="link" @focus="($event.target as HTMLInputElement).select()"></label>
    </article>
  </ContentShell>
</template>
<style scoped>
.scheme-nav { display: flex; justify-content: space-between; gap: 12px; padding: 16px; border-bottom: 1px solid var(--line); }
.scheme-detail { padding: 20px 16px; }
h1 { margin: 0; font-size: var(--fs-xl); } h2 { margin: 24px 0 14px; font-size: var(--fs-lg); }
.meta, .scheme-actions { display: flex; gap: 8px 16px; flex-wrap: wrap; margin: 12px 0; }
.meta, .hint { color: var(--ink-3); font-size: var(--fs-sm); }
.summary { white-space: pre-wrap; color: var(--ink-2); margin: 18px 0; }
.state { padding: 40px 16px; text-align: center; color: var(--ink-3); }
</style>
