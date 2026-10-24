<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter, type LocationQueryRaw } from 'vue-router';
import { market } from '@/lib/api';
import { KIND_NAME, SINGULAR } from '@/lib/format';
import { SUBTABS, subTabFor } from '@/lib/tabs';
import type { CategoryCount, ListResponse, Plural } from '@/lib/types';
import DetailModal from '@/components/DetailModal.vue';
import Icon from '@/components/Icon.vue';
import ItemCard from '@/components/ItemCard.vue';
import Segmented from '@/components/Segmented.vue';

const props = defineProps<{ kind: Plural; ns?: string; slug?: string }>();
const route = useRoute();
const router = useRouter();

const TABS: { kind: Plural; label: string; icon: string }[] = [
  { kind: 'experts', label: KIND_NAME.expert, icon: 'expert' },
  { kind: 'skills', label: KIND_NAME.skill, icon: 'skill' },
  { kind: 'connectors', label: KIND_NAME.connector, icon: 'connector' },
];
const SORTS = [
  { value: 'relevance', label: '综合' },
  { value: 'hot', label: '最热' },
  { value: 'new', label: '最新' },
] as const;

const q = computed(() => ({
  keyword: String(route.query.keyword ?? ''),
  category: String(route.query.category ?? ''),
  sort: (String(route.query.sort ?? 'relevance') || 'relevance') as (typeof SORTS)[number]['value'],
  tab: String(route.query.tab ?? ''),
  page: Math.max(1, Number(route.query.page ?? 1) || 1),
}));

const subTab = computed(() => subTabFor(props.kind, q.value.tab));
const data = ref<ListResponse | null>(null);
const categories = ref<CategoryCount[]>([]);
const loading = ref(true);
const error = ref('');
const keywordInput = ref(q.value.keyword);
let seq = 0;

async function loadList() {
  const id = ++seq;
  loading.value = true;
  error.value = '';
  try {
    const res = await market.list(props.kind, { keyword: q.value.keyword, category: q.value.category, sortBy: q.value.sort, page: q.value.page, variant: subTab.value?.variant, origin: subTab.value?.origin });
    if (id === seq) data.value = res;
  } catch (e) {
    if (id === seq) error.value = (e as Error).message;
  } finally {
    if (id === seq) loading.value = false;
  }
}

async function loadCategories() {
  try {
    categories.value = (await market.categories(props.kind)).results;
  } catch {
    categories.value = []; // 分类加载失败不阻塞列表
  }
}

watch(() => props.kind, () => { keywordInput.value = q.value.keyword; loadCategories(); }, { immediate: true });
watch(() => [props.kind, q.value.keyword, q.value.category, q.value.sort, q.value.page, q.value.tab], loadList, { immediate: true });
watch(() => q.value.keyword, (v) => (keywordInput.value = v));

function go(patch: Record<string, string | number | undefined>) {
  const query: LocationQueryRaw = { ...route.query, ...patch };
  for (const k of Object.keys(query)) if (query[k] === '' || query[k] === undefined) delete query[k];
  router.push({ name: 'store', params: { kind: props.kind }, query });
}

const search = () => go({ keyword: keywordInput.value.trim(), page: undefined });
const setSort = (v: string) => go({ sort: v === 'relevance' ? undefined : v, page: undefined });
const setCategory = (slug: string) => go({ category: slug, page: undefined });
const setTab = (key: string) => go({ tab: key, page: undefined, category: undefined });

const detailTo = (it: { namespace: string; slug: string }) => ({ name: 'store-detail', params: { kind: props.kind, ns: it.namespace, slug: it.slug }, query: route.query });
const closeDetail = () => router.push({ name: 'store', params: { kind: props.kind }, query: route.query });

const totalPages = computed(() => (data.value ? Math.max(1, Math.ceil(data.value.total / data.value.pageSize)) : 1));
const pages = computed(() => {
  const n = totalPages.value, cur = q.value.page;
  const set = new Set([1, n, cur - 1, cur, cur + 1].filter((p) => p >= 1 && p <= n));
  return [...set].sort((a, b) => a - b);
});
const placeholder = computed(() => `搜索${KIND_NAME[SINGULAR[props.kind]]}`);
const categoryName = computed(() => categories.value.find((c) => c.slug === q.value.category)?.name);
</script>

<template>
  <div class="page">
    <div class="site">
      <div class="site-in">
        <RouterLink to="/" class="brand">
          <svg width="22" height="22" viewBox="0 0 30 30" aria-hidden="true"><path d="M4 25V7l6 4.5h10L26 7v18a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3z" fill="#141414" /><circle cx="11" cy="18" r="1.7" fill="#fff" /><circle cx="19" cy="18" r="1.7" fill="#fff" /></svg>
          小伴市场
        </RouterLink>
        <span class="grow" />
        <a class="site-link" href="https://wallpaper.xiangban.xyz" target="_blank" rel="noopener noreferrer">下载小伴桌面</a>
      </div>
    </div>

    <main class="wrap">
      <div class="top">
        <nav class="tabs" aria-label="类型">
          <RouterLink v-for="t in TABS" :key="t.kind" :to="{ name: 'store', params: { kind: t.kind } }" class="tab" :class="{ on: t.kind === kind }" :aria-current="t.kind === kind ? 'page' : undefined">
            <Icon :name="t.icon" :size="24" :stroke="1.6" />{{ t.label }}
          </RouterLink>
        </nav>
        <form class="search" role="search" @submit.prevent="search">
          <Icon name="search" :size="18" :stroke="1.8" />
          <label class="sr-only" for="kw">{{ placeholder }}</label>
          <input id="kw" v-model="keywordInput" type="search" :placeholder="placeholder" autocomplete="off" />
          <button v-if="keywordInput" type="button" class="clear" aria-label="清除" @click="keywordInput = ''; search()"><Icon name="close" :size="14" :stroke="2" /></button>
        </form>
      </div>

      <nav v-if="SUBTABS[kind].length" class="subtabs" aria-label="子分类">
        <button v-for="t in SUBTABS[kind]" :key="t.key" type="button" class="subtab" :class="{ on: subTab?.key === t.key }" :aria-current="subTab?.key === t.key ? 'true' : undefined" @click="setTab(t.key)">{{ t.label }}</button>
      </nav>

      <div class="bar">
        <p class="count" aria-live="polite">
          <template v-if="q.keyword && data">“{{ q.keyword }}”{{ categoryName ? `在「${categoryName}」中` : `在${KIND_NAME[SINGULAR[kind]]}中` }}找到 <b>{{ data.total }}</b> 个结果</template>
        </p>
        <Segmented :model-value="q.sort" :options="[...SORTS]" label="排序" @update:model-value="setSort" />
      </div>

      <div v-if="categories.length" class="cats" role="group" aria-label="分类">
        <button type="button" class="cat" :class="{ on: !q.category }" @click="setCategory('')">全部</button>
        <button v-for="c in categories" :key="c.slug" type="button" class="cat" :class="{ on: q.category === c.slug }" @click="setCategory(c.slug)">{{ c.name }}</button>
      </div>

      <div v-if="error" class="error-box">
        {{ error }} <button class="btn btn-sm" type="button" @click="loadList">重试</button>
      </div>
      <div v-else-if="loading && !data" class="empty"><span class="spinner" /> 加载中…</div>
      <template v-else-if="data">
        <div v-if="data.results.length" class="grid" :class="{ busy: loading }">
          <ItemCard v-for="it in data.results" :key="it.id" :item="it" :to="detailTo(it)" :keyword="q.keyword" />
        </div>
        <div v-else-if="!data.near.length" class="empty">
          <template v-if="q.keyword || q.category">没有找到匹配的{{ subTab?.label ?? KIND_NAME[SINGULAR[kind]] }}，换个关键词或分类试试。</template>
          <template v-else>这里还没有{{ subTab?.label ?? KIND_NAME[SINGULAR[kind]] }}。</template>
        </div>

        <template v-if="data.near.length">
          <div class="divider"><span>{{ data.results.length ? '精确结果较少，以下是相近匹配' : '没有精确匹配，以下是相近结果' }}</span></div>
          <div class="grid">
            <ItemCard v-for="it in data.near" :key="it.id" :item="it" :to="detailTo(it)" muted />
          </div>
        </template>

        <nav v-if="totalPages > 1" class="pager" aria-label="分页">
          <button class="btn btn-sm" type="button" :disabled="q.page <= 1" @click="go({ page: q.page - 1 })">上一页</button>
          <template v-for="(p, i) in pages" :key="p">
            <span v-if="i > 0 && p - pages[i - 1] > 1" class="gap">…</span>
            <button class="pg" type="button" :class="{ on: p === q.page }" :aria-current="p === q.page ? 'page' : undefined" @click="go({ page: p })">{{ p }}</button>
          </template>
          <button class="btn btn-sm" type="button" :disabled="q.page >= totalPages" @click="go({ page: q.page + 1 })">下一页</button>
        </nav>
      </template>
    </main>

    <DetailModal v-if="ns && slug" :kind="kind" :ns="ns" :slug="slug" @close="closeDetail" />
  </div>
</template>

<style scoped>
.site { border-bottom: 1px solid var(--line); }
.site-in { max-width: 1080px; margin: 0 auto; padding: 0 32px; min-height: 56px; display: flex; align-items: center; gap: 28px; }
.brand { display: flex; align-items: center; gap: 9px; font-weight: 600; letter-spacing: 0.02em; }
.grow { flex: 1; }
.site-link { color: var(--ink-2); font-size: 14px; min-height: 44px; display: flex; align-items: center; }
.wrap { max-width: 1080px; margin: 0 auto; padding: 36px 32px 72px; display: flex; flex-direction: column; gap: 28px; }
.top { display: flex; flex-wrap: wrap; align-items: center; gap: 14px 32px; }
.tabs { display: flex; flex-wrap: wrap; gap: 0 30px; flex: 1 1 340px; }
.tab { display: flex; align-items: center; gap: 9px; min-height: 44px; font-size: 22px; letter-spacing: -0.01em; font-weight: 500; color: var(--ink-3); }
.tab.on { color: var(--ink); font-weight: 600; }
.tab:hover { color: var(--ink); }
.search { flex: 1 1 260px; max-width: 360px; display: flex; align-items: center; gap: 8px; min-height: 44px; padding: 0 8px 0 14px; background: var(--soft); border: 1px solid #ececec; border-radius: 12px; color: var(--ink-3); }
.search:focus-within { background: #fff; border-color: var(--ink); box-shadow: 0 0 0 3px rgba(20, 20, 20, 0.06); color: var(--ink); }
.search input { flex: 1; min-width: 0; border: 0; outline: 0; background: transparent; font-size: 15px; }
.clear { width: 32px; height: 32px; border: 0; border-radius: 999px; background: var(--wash); color: var(--ink-2); display: flex; align-items: center; justify-content: center; cursor: pointer; }
.subtabs { display: flex; gap: 24px; margin-bottom: -8px; }
.subtab { min-height: 44px; border: 0; background: transparent; padding: 0; font-size: 18px; font-weight: 500; letter-spacing: -0.005em; color: var(--ink-3); cursor: pointer; }
.subtab.on { color: var(--ink); font-weight: 600; }
.subtab:hover { color: var(--ink); }
.bar { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px; min-height: 44px; }
.count { font-size: 15px; color: var(--ink-2); }
.count b { color: var(--ink); font-weight: 600; }
.cats { display: flex; gap: 4px; overflow-x: auto; padding: 2px 0; margin-top: -10px; scrollbar-width: none; }
.cat { flex: 0 0 auto; min-height: 44px; padding: 0 16px; border: 0; border-radius: 12px; background: transparent; color: var(--ink-2); font-size: 15px; cursor: pointer; }
.cat:hover { background: var(--soft); }
.cat.on { background: #f2f2f0; color: var(--ink); font-weight: 600; }
.grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px; transition: opacity 0.15s; }
.grid.busy { opacity: 0.55; }
.divider { display: flex; align-items: center; gap: 14px; color: var(--ink-3); font-size: 13px; }
.divider::before, .divider::after { content: ''; flex: 1; height: 1px; background: var(--line); }
.pager { display: flex; justify-content: center; align-items: center; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
.pg { min-width: 44px; min-height: 44px; border: 1px solid var(--line); border-radius: 10px; background: #fff; cursor: pointer; font-size: 14px; }
.pg.on { background: var(--ink); border-color: var(--ink); color: #fff; }
.gap { color: var(--ink-3); padding: 0 4px; }
@media (max-width: 760px) {
  .site-in, .wrap { padding-left: 16px; padding-right: 16px; }
  .grid { grid-template-columns: minmax(0, 1fr); }
  .tab { font-size: 19px; }
  .tabs { gap: 0 20px; }
  .search { max-width: none; }
}
</style>
