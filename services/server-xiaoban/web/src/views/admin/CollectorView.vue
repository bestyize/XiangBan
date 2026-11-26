<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import StateBlock from '@/components/StateBlock.vue'
import Viewer from '@/components/Viewer.vue'
import AdminSelect, { type SelectOption } from '@/components/admin/AdminSelect.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import { collectorCheckStatus, collectorImport, collectorSearch, listProviders } from '@/admin/api'
import type { ProviderConfig } from '@/admin/types'
import { toast } from '@/composables/toast'
import { openViewer } from '@/composables/viewer'
import { useCatalog } from '@/stores/catalog'
import type { FeedParams, Orientation, WallpaperItem } from '@/types'
import { cardImage } from '@/utils'

const router = useRouter()
const catalog = useCatalog()

const q = ref('')
const draft = ref('')
const media = ref<'image' | 'video'>('image')
const orientation = ref<Orientation>('all')
const provider = ref('all')
const selectedCategory = ref('')

// Quick-collect preset strip
const quickMode = ref(true)
const presetCategory = ref('')
const presetStatus = ref<'published' | 'pending'>('published')

const items = ref<WallpaperItem[]>([])
const loading = ref(false)
const loadingMore = ref(false)
const error = ref<string | null>(null)
const hasMore = ref(true)
const page = ref(1)
const perPage = 24

const collectedIds = ref<Set<string>>(new Set())
const selectedIds = ref<Set<string>>(new Set())
const batchImporting = ref(false)
const providerConfigs = ref<ProviderConfig[]>([])

// Collect Modal (when quickMode is OFF)
const targetItem = ref<WallpaperItem | null>(null)
const importCategory = ref('')
const importStatus = ref<'published' | 'pending'>('published')
const importing = ref(false)

const providerOptions = computed<SelectOption[]>(() => {
  const health = (id: string): { dot: 'ok' | 'warn' | 'off'; hint?: string } => {
    const p = providerConfigs.value.find((x) => x.id === id)
    if (!p) return { dot: 'ok' }
    if (p.keys.some((k) => k.cooldown_secs)) return { dot: 'warn', hint: '冷却中' }
    if (p.keys.some((k) => k.enabled) || id === 'bing') return { dot: 'ok' }
    return { dot: 'off', hint: '未启用' }
  }
  return [
    { value: 'all', label: '全部图源', dot: 'ok' },
    { value: 'unsplash', label: 'Unsplash', ...health('unsplash') },
    { value: 'pexels', label: 'Pexels', ...health('pexels') },
    { value: 'pixabay', label: 'Pixabay', ...health('pixabay') },
    { value: 'okslop', label: 'okslop', ...health('okslop') },
    { value: 'bing', label: 'Bing 必应', ...health('bing') },
    { value: 'demo', label: 'Demo 示例', dot: 'ok' },
  ]
})

const orientationOptions: SelectOption[] = [
  { value: 'all', label: '全部画幅' },
  { value: 'landscape', label: '横屏 (PC)', icon: 'monitor' },
  { value: 'portrait', label: '竖屏 (手机)', icon: 'phone' },
  { value: 'squarish', label: '方形', icon: 'grid' },
]

const categoryOptions = computed<SelectOption[]>(() => [
  { value: '', label: '未分类' },
  ...catalog.categories.map((c) => ({ value: c.id, label: c.name })),
])

const searchCatOptions = computed<SelectOption[]>(() => [
  { value: '', label: '全部预设分类' },
  ...catalog.categories.map((c) => ({ value: c.id, label: c.name })),
])

const statusOptions: SelectOption[] = [
  { value: 'published', label: '立即上架', dot: 'ok' },
  { value: 'pending', label: '待审核', dot: 'warn' },
]

const params = computed<FeedParams>(() => ({
  q: q.value.trim() || undefined,
  category: selectedCategory.value || undefined,
  media: media.value,
  orientation: orientation.value,
  provider: provider.value,
}))

onMounted(async () => {
  if (catalog.categories.length === 0) await catalog.load()
  listProviders()
    .then((list) => { providerConfigs.value = list })
    .catch(() => {})
  void loadFirstPage()
})

async function checkExisting(list: WallpaperItem[]) {
  if (!list.length) return
  try {
    const res = await collectorCheckStatus(list.map((i) => i.id))
    const next = new Set(collectedIds.value)
    for (const id of res.collected_ids) next.add(id)
    collectedIds.value = next
  } catch { /* ignore */ }
}

async function loadFirstPage() {
  loading.value = true
  error.value = null
  page.value = 1
  selectedIds.value = new Set()
  try {
    const res = await collectorSearch(params.value, 1, perPage)
    items.value = res.items
    hasMore.value = res.has_more
    await checkExisting(res.items)
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : '搜索外部壁纸失败'
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (loading.value || loadingMore.value || !hasMore.value) return
  loadingMore.value = true
  const nextPage = page.value + 1
  try {
    const res = await collectorSearch(params.value, nextPage, perPage)
    items.value = [...items.value, ...res.items]
    hasMore.value = res.has_more
    page.value = nextPage
    await checkExisting(res.items)
  } catch { /* ignore */ } finally {
    loadingMore.value = false
  }
}

function handleSearch() {
  q.value = draft.value
  void loadFirstPage()
}

watch([media, orientation, provider, selectedCategory], () => {
  void loadFirstPage()
})

function onOpen(item: WallpaperItem) {
  openViewer(router, { items, loadMore }, item)
}

function toggleSelect(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

async function importOne(item: WallpaperItem, statusVal: 'published' | 'pending', categoryVal: string) {
  await collectorImport(item, statusVal, categoryVal || undefined)
  const next = new Set(collectedIds.value)
  next.add(item.id)
  collectedIds.value = next
  const nextSel = new Set(selectedIds.value)
  nextSel.delete(item.id)
  selectedIds.value = nextSel
}

async function onCollectClick(item: WallpaperItem) {
  if (quickMode.value) {
    try {
      await importOne(item, presetStatus.value, presetCategory.value || selectedCategory.value || item.category || '')
      toast(`已采入「${item.title}」`)
    } catch (err: unknown) {
      toast(err instanceof Error ? err.message : '采集入库失败')
    }
    return
  }
  targetItem.value = item
  importCategory.value = presetCategory.value || selectedCategory.value || item.category || ''
  importStatus.value = presetStatus.value
}

async function confirmImport() {
  if (!targetItem.value || importing.value) return
  importing.value = true
  try {
    await importOne(targetItem.value, importStatus.value, importCategory.value)
    toast('壁纸采集入库成功')
    targetItem.value = null
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '采集入库失败')
  } finally {
    importing.value = false
  }
}

async function batchImport() {
  if (batchImporting.value || !selectedIds.value.size) return
  batchImporting.value = true
  const ids = [...selectedIds.value]
  let ok = 0
  for (const id of ids) {
    const it = items.value.find((x) => x.id === id)
    if (!it || collectedIds.value.has(id)) continue
    try {
      await importOne(it, presetStatus.value, presetCategory.value || selectedCategory.value || it.category || '')
      ok++
    } catch { /* continue */ }
  }
  batchImporting.value = false
  toast(`已批量入库 ${ok} 项`)
}
</script>

<template>
  <div class="xb-page">
    <header class="xb-page-head">
      <div class="xb-tabs-lg">
        <button type="button" :class="{ 'is-active': media === 'image' }" @click="media = 'image'">图片壁纸</button>
        <button type="button" :class="{ 'is-active': media === 'video' }" @click="media = 'video'">动态视频</button>
      </div>
      <div class="xb-page-actions">
        <AdminSelect v-model="provider" :options="providerOptions" />
        <AdminSelect v-model="orientation" :options="orientationOptions" />
        <AdminSelect v-model="selectedCategory" :options="searchCatOptions" />
      </div>
    </header>

    <form class="search-row" @submit.prevent="handleSearch">
      <div class="xb-affix has-l" :class="{ 'has-r': !!draft }" style="flex: 1; min-width: 220px">
        <span class="af-l"><Icon name="search" :size="14" /></span>
        <input
          v-model="draft"
          type="search"
          class="xb-input"
          placeholder="搜索外部供应商素材（如 极光、Cyberpunk、4K…）"
        />
        <button v-if="draft" type="button" class="af-r" aria-label="清空" @click="draft = ''; handleSearch()">
          <Icon name="close" :size="13" />
        </button>
      </div>
      <button type="submit" class="xb-btn-primary">检索</button>
    </form>

    <div class="preset-strip" title="开启一键快采后，点击卡片直接按此处分类与状态入库，跳过确认弹窗">
      <div class="preset-l">
        <strong class="preset-tag"><Icon name="bolt" :size="13" /><span>采集预设</span></strong>
        <AdminSelect v-model="presetCategory" :options="categoryOptions" prefix="分类：" sm />
        <AdminSelect v-model="presetStatus" :options="statusOptions" prefix="状态：" sm />
      </div>
      <div class="preset-r">
        <span>一键快采</span>
        <ToggleSwitch v-model="quickMode" label="一键快采模式" />
      </div>
    </div>

    <div v-if="loading && !items.length" class="xb-group xb-empty-box">正在检索外部图源…</div>
    <StateBlock
      v-else-if="error && !items.length"
      kind="error"
      :title="error"
      action="重新尝试"
      @action="loadFirstPage"
    />
    <div v-else-if="!items.length" class="xb-group xb-empty-box">
      <Icon name="compass" :size="28" />
      <strong>未检索到相关素材</strong>
    </div>

    <template v-else>
      <div class="wp-grid">
        <article
          v-for="item in items"
          :key="item.id"
          class="wp-card"
          :class="{ 'is-selected': selectedIds.has(item.id) }"
        >
          <div class="wp-thumb" @click="onOpen(item)">
            <video
              v-if="item.media_type === 'video' && item.video"
              :src="`${item.video.preview_url}#t=0.001`"
              :poster="item.preview_url || undefined"
              muted
              loop
              playsinline
              preload="metadata"
            />
            <img v-else :src="cardImage(item)" :alt="item.title" loading="lazy" />

            <button
              v-if="!collectedIds.has(item.id)"
              type="button"
              class="wp-sel"
              aria-label="勾选批量入库"
              @click.stop="toggleSelect(item.id)"
            >
              <Icon v-if="selectedIds.has(item.id)" name="check" :size="13" />
            </button>

            <div class="wp-top-r">
              <span v-if="collectedIds.has(item.id)" class="wp-glass is-ok">
                <Icon name="check" :size="11" /><span>已入库</span>
              </span>
              <button v-else type="button" class="wp-glass is-action" @click.stop="onCollectClick(item)">
                <Icon :name="quickMode ? 'bolt' : 'download'" :size="11" />
                <span>{{ quickMode ? '快采' : '采集' }}</span>
              </button>
            </div>

            <div class="wp-bot-l">
              <span v-if="item.media_type === 'video'" class="wp-glass">
                <Icon name="play" :size="10" /><span>视频</span>
              </span>
              <span class="wp-glass xb-num">{{ item.resolution.width }}×{{ item.resolution.height }}</span>
            </div>
          </div>

          <div class="wp-info">
            <div class="wp-title" :title="item.title">{{ item.title }}</div>
            <div class="wp-sub">
              <span>{{ item.provider }}</span>
              <span>·</span>
              <span>{{ item.orientation === 'landscape' ? '横屏' : item.orientation === 'portrait' ? '竖屏' : '方形' }}</span>
            </div>
          </div>
        </article>
      </div>

      <div v-if="hasMore" class="load-more">
        <button type="button" class="xb-btn-soft" :disabled="loadingMore" @click="loadMore">
          {{ loadingMore ? '加载中…' : '加载下一页' }}
        </button>
      </div>
    </template>

    <div v-if="selectedIds.size > 0" class="xb-batch-bar">
      <span>已选 <strong class="xb-num">{{ selectedIds.size }}</strong> 项</span>
      <button type="button" class="xb-batch-btn" :disabled="batchImporting" @click="batchImport">
        {{ batchImporting ? '入库中…' : '批量入库' }}
      </button>
      <button type="button" class="xb-batch-btn" @click="selectedIds = new Set()">取消</button>
    </div>

    <!-- 24px Collect Modal -->
    <Teleport to="body">
      <div v-if="targetItem" class="xb-backdrop is-center" @click.self="targetItem = null">
        <form class="xb-dialog is-sm" @submit.prevent="confirmImport">
          <header class="xb-ov-head">
            <h2>采集入库</h2>
            <button type="button" class="xb-icon-btn" aria-label="关闭" @click="targetItem = null">
              <Icon name="close" :size="18" />
            </button>
          </header>
          <div class="xb-ov-body">
            <div class="mini-card">
              <img :src="targetItem.preview_url" :alt="targetItem.title" />
              <div class="mini-text">
                <div class="wp-title">{{ targetItem.title }}</div>
                <div class="wp-sub xb-num">{{ targetItem.provider }} · {{ targetItem.resolution.width }}×{{ targetItem.resolution.height }}</div>
              </div>
            </div>
            <div class="xb-field">
              <span class="xb-field-label">分类</span>
              <AdminSelect v-model="importCategory" :options="categoryOptions" full />
            </div>
            <div class="xb-field">
              <span class="xb-field-label">入库状态</span>
              <div class="xb-seg" style="align-self: flex-start">
                <button type="button" :class="{ 'is-active': importStatus === 'published' }" @click="importStatus = 'published'">
                  立即上架
                </button>
                <button type="button" :class="{ 'is-active': importStatus === 'pending' }" @click="importStatus = 'pending'">
                  待审核
                </button>
              </div>
            </div>
          </div>
          <footer class="xb-ov-foot">
            <button type="button" class="xb-btn-ghost" @click="targetItem = null">取消</button>
            <button type="submit" class="xb-btn-primary" :disabled="importing">
              {{ importing ? '入库中…' : '确认入库' }}
            </button>
          </footer>
        </form>
      </div>
    </Teleport>

    <Viewer />
  </div>
</template>

<style scoped>
.search-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.preset-strip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
  padding: 9px 14px;
  border-radius: 10px;
  background: var(--ui-soft);
  color: var(--ui-ink);
  font-size: 12.5px;
}
.preset-l,
.preset-r {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.preset-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.wp-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 14px;
}
.wp-card {
  border-radius: 12px;
  background: var(--ui-surface-sunken);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: box-shadow 0.14s;
}
.wp-card.is-selected { box-shadow: 0 0 0 2px var(--ui-btn); }
.wp-thumb {
  position: relative;
  aspect-ratio: 16 / 10;
  overflow: hidden;
  cursor: pointer;
  background: var(--ui-control);
}
.wp-thumb img,
.wp-thumb video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s var(--ui-ease);
}
.wp-card:hover .wp-thumb img { transform: scale(1.03); }
.wp-sel {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 4;
  width: 22px;
  height: 22px;
  border-radius: 6px;
  background: rgba(18, 18, 22, 0.54);
  backdrop-filter: blur(8px);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  opacity: 0;
  transition: opacity 0.14s, background 0.14s;
}
.wp-card:hover .wp-sel,
.wp-card.is-selected .wp-sel { opacity: 1; }
.wp-card.is-selected .wp-sel { background: var(--ui-btn); }
.wp-top-r {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 4;
  display: flex;
  gap: 5px;
}
.wp-bot-l {
  position: absolute;
  bottom: 8px;
  left: 8px;
  z-index: 3;
  display: flex;
  gap: 5px;
}
.wp-glass {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 22px;
  padding: 0 8px;
  border-radius: 999px;
  line-height: 1;
  background: rgba(16, 16, 20, 0.62);
  backdrop-filter: blur(10px);
  color: #fff;
  font-size: 11px;
  font-weight: 550;
}
.wp-glass.is-ok { background: rgba(47, 163, 107, 0.92); }
.wp-glass.is-action {
  background: rgba(255, 255, 255, 0.94);
  color: #1D1D1F;
  font-weight: 600;
}
.wp-glass.is-action:hover { background: #fff; }
.wp-info {
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.wp-title {
  font-size: 13px;
  font-weight: 550;
  line-height: 1.25;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.wp-sub {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
  color: var(--ui-text-tertiary);
  line-height: 1;
}
.load-more {
  display: flex;
  justify-content: center;
  padding-top: 8px;
}
.mini-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
}
.mini-card img {
  width: 76px;
  height: 48px;
  border-radius: 7px;
  object-fit: cover;
  flex: none;
}
.mini-text { min-width: 0; display: flex; flex-direction: column; gap: 4px; }

@media (max-width: 1100px) { .wp-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
@media (max-width: 760px) { .wp-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; } }
</style>
