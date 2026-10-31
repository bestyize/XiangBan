<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import SchemeCard from '@/components/SchemeCard.vue'
import PagerNav from '@/components/PagerNav.vue'
import { schemesApi, type CommunityScheme } from '@/api/schemes'
import { useHead } from '@/lib/head'
useHead(() => ({ title: '社区方案' }))
const route = useRoute(), router = useRouter()
const items = ref<CommunityScheme[]>([]), loading = ref(false), error = ref('')
const page = ref(1), pageSize = ref(20), total = ref(0)
let sequence = 0
async function load() {
  const current = ++sequence
  loading.value = true; error.value = ''
  const requested = Number(route.query.page || 1)
  try {
    const data = await schemesApi.list(Number.isSafeInteger(requested) && requested > 0 ? requested : 1)
    if (current !== sequence) return
    items.value = data.items; page.value = data.page; pageSize.value = data.page_size; total.value = data.total
  } catch (e) { if (current === sequence) { items.value = []; error.value = e instanceof Error ? e.message : '方案加载失败' } }
  finally { if (current === sequence) loading.value = false }
}
watch(() => route.query.page, load, { immediate: true })
</script>
<template>
  <ContentShell>
    <header class="schemes-heading"><div><h1>社区方案</h1><p>把多件作品组合成一个方案，查看作者选择的版本与公开作品。</p></div><RouterLink class="btn btn-primary" to="/schemes/new">发布方案</RouterLink></header>
    <p v-if="loading" class="state" role="status">正在加载方案…</p>
    <p v-else-if="error" class="state" role="alert">{{ error }} <button class="btn" @click="load">重试</button></p>
    <p v-else-if="!items.length" class="state">这里还没有公开方案。<RouterLink to="/schemes/new">发布第一份方案</RouterLink></p>
    <template v-else><SchemeCard v-for="scheme in items" :key="scheme.id" :scheme="scheme" /><PagerNav :page="page" :page-size="pageSize" :total="total" @go="value => router.push({ query: { page: value } })" /></template>
  </ContentShell>
</template>
<style scoped>
.schemes-heading { display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; padding: 20px 16px; flex-wrap: wrap; }
h1 { margin: 0; font-size: var(--fs-xl); }
p { margin: 8px 0 0; color: var(--ink-3); }
.state { padding: 40px 16px; text-align: center; color: var(--ink-3); }
</style>
