<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import PackCard from '@/components/PackCard.vue'
import SchemeCard from '@/components/SchemeCard.vue'
import PagerNav from '@/components/PagerNav.vue'
import WeeklyFeed from '@/components/WeeklyFeed.vue'
import ProjectFeed from '@/components/ProjectFeed.vue'
import { contentApi, type PackItem, type SchemeItem } from '@/api/content'
import { PACK_KINDS, VISUAL_KINDS } from '@/lib/content'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '' }))

const route = useRoute()
const router = useRouter()
const toast = useToast()

/** 二级 Tab：类型、周刊、项目。周刊与项目是整页列表，其余是内容包卡片流。 */
const TABS = [{ key: '', label: '全部' }, { key: 'weekly', label: '周刊' }, { key: 'projects', label: '项目' }, ...PACK_KINDS, { key: 'schemes', label: '方案' }]
const tab = computed(() => {
  const k = String(route.query.tab ?? route.query.kind ?? '')
  return TABS.some(t => t.key === k) ? k : ''
})
const sort = computed<'new' | 'hot'>(() => (route.query.sort === 'hot' ? 'hot' : 'new'))
const feedTab = computed(() => tab.value === 'weekly' || tab.value === 'projects')
const wide = computed(() => tab.value !== '' && !VISUAL_KINDS.includes(tab.value))

const packs = ref<PackItem[]>([])
const schemes = ref<SchemeItem[]>([])
const page = ref(1)
const total = ref(0)
const pageSize = ref(20)
const loading = ref(false)
const schemeError = ref('')
let seq = 0

async function load(p = 1) {
  if (feedTab.value) {
    packs.value = []
    schemes.value = []
    total.value = 0
    return
  }
  const mine = ++seq
  loading.value = true
  schemeError.value = ''
  if (tab.value === 'schemes') packs.value = []
  try {
    if (tab.value === 'schemes') {
      const res = await contentApi.schemes(p)
      if (mine !== seq) return
      schemes.value = res.items
      packs.value = []
      ;({ page: page.value, total: total.value, page_size: pageSize.value } = res)
    } else {
      const res = await contentApi.packs({ kind: tab.value || undefined, sort: sort.value, page: p })
      if (mine !== seq) return
      packs.value = res.items
      schemes.value = []
      ;({ page: page.value, total: total.value, page_size: pageSize.value } = res)
    }
  } catch (e) {
    if (mine === seq) {
      if (tab.value === 'schemes') { schemeError.value = e instanceof Error ? e.message : '方案加载失败'; schemes.value = []; total.value = 0 }
      else toast.error(e)
    }
  } finally {
    if (mine === seq) loading.value = false
  }
}

watch([tab, sort], () => load(1), { immediate: true })

function setTab(key: string) {
  router.replace({ query: { ...(key ? { tab: key } : {}), ...(sort.value === 'hot' && key !== 'weekly' && key !== 'projects' ? { sort: 'hot' } : {}) } })
}
function setSort(s: 'new' | 'hot') {
  router.replace({ query: { ...(tab.value ? { tab: tab.value } : {}), ...(s === 'hot' ? { sort: 'hot' } : {}) } })
}
function go(p: number) {
  load(p)
  window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>

<template>
  <ContentShell>
    <h1 class="sr-only">超级小伴</h1>
    <p class="model-rank-entry"><RouterLink to="/models">模型能力榜</RouterLink> · <RouterLink to="/models?board=usage">模型调用量榜</RouterLink></p>
    <div class="subtabs" role="tablist" aria-label="内容类型">
      <button v-for="t in TABS" :key="t.key" type="button" role="tab" :aria-selected="tab === t.key" :class="{ on: tab === t.key }" @click="setTab(t.key)">
        {{ t.label }}
      </button>
    </div>
    <div class="community-entry"><RouterLink to="/challenges">参与主题挑战</RouterLink></div>
    <div v-if="tab !== 'schemes' && !feedTab" class="sortbar">
      <button type="button" :class="{ on: sort === 'new' }" @click="setSort('new')">最新</button>
      <button type="button" :class="{ on: sort === 'hot' }" @click="setSort('hot')">最多使用</button>
    </div>

    <WeeklyFeed v-if="tab === 'weekly'" />
    <ProjectFeed v-else-if="tab === 'projects'" />

    <p v-else-if="loading && !packs.length && !schemes.length" class="state">加载中</p>
    <p v-else-if="tab === 'schemes' && schemeError" class="state" role="alert">{{ schemeError }} <button class="btn" @click="load(page)">重试</button></p>
    <p v-else-if="!loading && !packs.length && !schemes.length" class="state">这里还没有内容。</p>

    <div v-if="packs.length" :class="wide ? 'list' : 'grid'">
      <PackCard v-for="p in packs" :key="p.id" :pack="p" :wide="wide" />
    </div>

    <div v-if="tab === 'schemes'" class="scheme-toolbar"><RouterLink class="btn btn-primary" to="/schemes/new">发布方案</RouterLink><RouterLink to="/schemes">查看全部方案</RouterLink></div>
    <div v-if="schemes.length"><SchemeCard v-for="scheme in schemes" :key="scheme.id" :scheme="scheme" /></div>

    <PagerNav v-if="!feedTab" :page="page" :page-size="pageSize" :total="total" @go="go" />
  </ContentShell>
</template>

<style scoped>
.community-entry { padding: 12px 16px 0; font-size: var(--fs-sm); }
.scheme-toolbar { display: flex; gap: 16px; align-items: center; padding: 16px; }
.subtabs {
  display: flex;
  gap: 18px;
  padding: 10px 16px 0;
  overflow-x: auto;
  scrollbar-width: none;
}
.subtabs::-webkit-scrollbar {
  display: none;
}
.subtabs button {
  flex: none;
  border: 0;
  background: none;
  padding: 4px 0 8px;
  font-size: var(--fs-sm);
  color: var(--ink-3);
  border-bottom: 2px solid transparent;
}
.subtabs button.on {
  color: var(--ink);
  font-weight: 600;
  border-bottom-color: var(--brand);
}
.sortbar {
  display: flex;
  gap: 14px;
  padding: 10px 16px 4px;
  font-size: var(--fs-xs);
}
.sortbar button {
  border: 0;
  background: none;
  padding: 0;
  color: var(--ink-3);
}
.sortbar button.on {
  color: var(--ink);
  font-weight: 600;
}
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px 12px;
  padding: 8px 16px;
}
.list {
  padding: 0 16px;
}
@media (min-width: 768px) {
  .grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}
</style>
