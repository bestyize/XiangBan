<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import PagerNav from '@/components/PagerNav.vue'
import { challengesApi, type CommunityChallenge } from '@/api/challenges'
import { useHead } from '@/lib/head'
useHead(() => ({ title: '主题挑战' }))
const route = useRoute(), router = useRouter()
const labels = { upcoming: '即将开始', open: '投稿中', ended: '已截止', closed: '已关闭' }
const items = ref<CommunityChallenge[]>([]), loading = ref(false), error = ref('')
const page = ref(1), pageSize = ref(25), total = ref(0)
function date(value: string) { return new Date(value).toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' }) }
let sequence = 0
async function load() {
  const current = ++sequence; loading.value = true; error.value = ''
  const requested = Number(route.query.page || 1)
  try {
    const data = await challengesApi.list(Number.isSafeInteger(requested) && requested > 0 ? requested : 1)
    if (current !== sequence) return
    items.value = data.items; page.value = data.page; pageSize.value = data.page_size; total.value = data.total
  } catch (e) { if (current === sequence) { items.value = []; error.value = e instanceof Error ? e.message : '挑战加载失败' } }
  finally { if (current === sequence) loading.value = false }
}
watch(() => route.query.page, load, { immediate: true })
</script>
<template>
  <ContentShell><header class="challenge-heading"><h1>主题挑战</h1><p>围绕一个主题分享自己的作品，在作品讨论中交流。</p><RouterLink to="/">返回社区首页</RouterLink></header>
    <p v-if="loading" class="state" role="status">正在加载挑战…</p><p v-else-if="error" class="state" role="alert">{{ error }} <button class="btn" @click="load">重试</button></p><p v-else-if="!items.length" class="state">还没有公开挑战，稍后再来看看。</p>
    <template v-else><article v-for="challenge in items" :key="challenge.key" class="challenge-card"><div class="challenge-top"><span class="phase" :class="{ open: challenge.status === 'open' }">{{ labels[challenge.status] }}</span><span class="meta">{{ challenge.entry_count }} 件公开作品</span></div><h2><RouterLink :to="`/challenges/${encodeURIComponent(challenge.key)}`">{{ challenge.title }}</RouterLink></h2><p class="description">{{ challenge.description }}</p><p class="meta">{{ date(challenge.starts_at) }} — {{ date(challenge.ends_at) }} · #{{ challenge.submission_tag }}</p><RouterLink class="btn btn-sm" :to="`/challenges/${encodeURIComponent(challenge.key)}`">{{ challenge.status === 'open' ? '查看挑战并投稿' : '查看挑战与作品' }}</RouterLink></article><PagerNav :page="page" :page-size="pageSize" :total="total" @go="value => router.push({ query: { page: value } })" /></template>
  </ContentShell>
</template>
<style scoped>
.challenge-heading { padding: 24px 20px; border-bottom: 1px solid var(--line); }
h1 { margin: 0; font-size: var(--fs-xl); } .challenge-heading p { color: var(--ink-3); margin: 10px 0; }
.challenge-card { padding: 24px 20px; border-bottom: 1px solid var(--line); }
.challenge-top { display: flex; gap: 12px; align-items: center; }.phase { padding: 3px 10px; background: var(--surface-2); color: var(--ink-2); border-radius: 20px; font-size: var(--fs-xs); }.phase.open { color: var(--brand); border: 1px solid currentColor; }
h2 { margin: 12px 0; font-size: var(--fs-lg); }.meta { color: var(--ink-3); font-size: var(--fs-sm); }.description { color: var(--ink-2); overflow: hidden; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; white-space: pre-wrap; line-height: 1.6; }
.state { padding: 40px 16px; color: var(--ink-3); text-align: center; }
</style>
