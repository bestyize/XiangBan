<script setup lang="ts">
import { ref, watch } from 'vue'
import ContentShell from '@/components/ContentShell.vue'
import PackCard from '@/components/PackCard.vue'
import SchemeCard from '@/components/SchemeCard.vue'
import PagerNav from '@/components/PagerNav.vue'
import { contentSocialApi, type ContentBookmark, type ContentKind } from '@/api/contentSocial'
import { useHead } from '@/lib/head'
useHead(() => ({ title: '作品收藏' }))
const kind = ref<ContentKind | ''>(''), items = ref<ContentBookmark[]>([]), page = ref(1), total = ref(0), loading = ref(false), error = ref(''), busy = ref('')
let sequence = 0
async function load(p = 1) {
  const current = ++sequence
  loading.value = true; error.value = ''; items.value = []
  try { const data = await contentSocialApi.bookmarks(p, kind.value || undefined); if (current === sequence) { items.value = data.items; total.value = data.total; page.value = data.page } }
  catch (e) { if (current === sequence) error.value = e instanceof Error ? e.message : '收藏读取失败' }
  finally { if (current === sequence) loading.value = false }
}
async function remove(item: ContentBookmark) {
  busy.value = `${item.kind}:${item.id}`; error.value = ''
  try { await contentSocialApi.bookmark(item.kind, item.id, false); await load(items.value.length === 1 && page.value > 1 ? page.value - 1 : page.value) }
  catch (e) { error.value = e instanceof Error ? e.message : '取消收藏失败' }
  finally { busy.value = '' }
}
watch(kind, () => load(), { immediate: true })
</script>
<template>
  <ContentShell>
    <header><h1>作品收藏</h1><RouterLink to="/me/works">我的作品</RouterLink></header>
    <div class="filters"><label>内容类型 <select v-model="kind" class="input" :disabled="loading"><option value="">全部</option><option value="pack">内容包</option><option value="scheme">方案</option></select></label><p>下架、封禁或不再公开的内容会自动隐藏。</p></div>
    <p v-if="error" class="state" role="alert">{{ error }} <button class="btn" :disabled="loading" @click="load(page)">重试</button></p>
    <p v-if="loading" class="state" role="status">正在读取收藏…</p>
    <p v-else-if="!items.length && !error" class="state">还没有可展示的收藏。<RouterLink to="/schemes">发现方案</RouterLink></p>
    <ul v-else class="bookmarks"><li v-for="item in items" :key="`${item.kind}:${item.id}`"><PackCard v-if="item.kind === 'pack'" :pack="item.content" wide /><SchemeCard v-else :scheme="item.content" /><button class="btn btn-sm" :disabled="!!busy" @click="remove(item)">{{ busy === `${item.kind}:${item.id}` ? '正在取消…' : '取消收藏' }}</button></li></ul>
    <PagerNav v-if="!loading && !error" :page="page" :page-size="20" :total="total" @go="load" />
  </ContentShell>
</template>
<style scoped>
header { display:flex; flex-wrap:wrap; align-items:center; justify-content:space-between; gap:12px; padding:16px; }h1 { margin:0; font-size:var(--fs-xl); }.filters { padding:0 16px; }.filters label { display:flex; align-items:center; gap:12px; }.filters select { width:auto; }.filters p { color:var(--ink-3); font-size:var(--fs-sm); }.state { padding:32px 16px; text-align:center; }.bookmarks { margin:0; padding:0 16px; list-style:none; }.bookmarks li { padding:0 0 16px; border-bottom:1px solid var(--line); }.bookmarks button { margin-top:8px; }
</style>
