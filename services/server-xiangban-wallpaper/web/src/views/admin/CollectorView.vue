<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import StateBlock from '@/components/StateBlock.vue'
import Viewer from '@/components/Viewer.vue'
import WallpaperGrid from '@/components/WallpaperGrid.vue'
import { collectorCheckStatus, collectorImport, collectorSearch } from '@/admin/api'
import { openViewer } from '@/composables/viewer'
import { toast } from '@/composables/toast'
import { useCatalog } from '@/stores/catalog'
import type { FeedParams, Orientation, WallpaperItem } from '@/types'

const router = useRouter()
const catalog = useCatalog()

const q = ref('')
const draft = ref('')
const media = ref<'image' | 'video'>('image')
const orientation = ref<Orientation>('all')
const provider = ref('all')
const selectedCategory = ref<string>('')

const items = ref<WallpaperItem[]>([])
const loading = ref(false)
const loadingMore = ref(false)
const error = ref<string | null>(null)
const hasMore = ref(true)
const page = ref(1)
const perPage = 24

// 记录已采集的 ID 集合
const collectedIds = ref<Set<string>>(new Set())

// 采集弹窗状态
const targetItem = ref<WallpaperItem | null>(null)
const importCategory = ref<string>('')
const importStatus = ref<'published' | 'pending'>('published')
const importing = ref(false)

const params = computed<FeedParams>(() => ({
  q: q.value.trim() || undefined,
  category: selectedCategory.value || undefined,
  media: media.value,
  orientation: orientation.value,
  provider: provider.value,
}))

onMounted(async () => {
  if (catalog.categories.length === 0) {
    await catalog.load()
  }
  loadFirstPage()
})

async function checkExisting(list: WallpaperItem[]) {
  if (list.length === 0) return
  const ids = list.map((item) => item.id)
  try {
    const res = await collectorCheckStatus(ids)
    for (const id of res.collected_ids) {
      collectedIds.value.add(id)
    }
  } catch {
    // 忽略检查状态的错误
  }
}

async function loadFirstPage() {
  loading.value = true
  error.value = null
  page.value = 1
  try {
    const res = await collectorSearch(params.value, 1, perPage)
    items.value = res.items
    hasMore.value = res.has_more
    await checkExisting(res.items)
  } catch (err: any) {
    error.value = err?.message || '搜索外部壁纸失败'
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
  } catch {
    // 静默处理触底失败
  } finally {
    loadingMore.value = false
  }
}

function handleSearch() {
  q.value = draft.value
  loadFirstPage()
}

watch([media, orientation, provider, selectedCategory], () => {
  loadFirstPage()
})

function onOpen(item: WallpaperItem) {
  openViewer(router, { items, loadMore }, item)
}

function openCollectModal(item: WallpaperItem) {
  targetItem.value = item
  importCategory.value = selectedCategory.value || item.category || ''
  importStatus.value = 'published'
}

async function confirmImport() {
  if (!targetItem.value) return
  importing.value = true
  try {
    await collectorImport(targetItem.value, importStatus.value, importCategory.value || undefined)
    collectedIds.value.add(targetItem.value.id)
    toast('壁纸采集入库成功！')
    targetItem.value = null
  } catch (err: any) {
    toast(err?.message || '采集入库失败')
  } finally {
    importing.value = false
  }
}
</script>

<template>
  <div class="collector-view">
    <header class="head">
      <div class="title-wrap">
        <h1 class="a-h1">壁纸采集工作台</h1>
        <p class="desc">检索外部供应商海量壁纸，人工挑选并一键采集入库，审核上架后对外展示。</p>
      </div>

      <!-- 搜索栏与过滤面板 -->
      <div class="filters-panel">
        <form class="search-box" @submit.prevent="handleSearch">
          <Icon name="search" :size="18" class="search-icon" />
          <input
            v-model="draft"
            type="search"
            placeholder="搜索外部壁纸关键词（如 4K, Cyberpunk, Nature...）"
            class="search-input"
          />
          <button type="submit" class="a-btn primary search-btn">搜索</button>
        </form>

        <div class="controls-row">
          <!-- 媒体类型 -->
          <div class="tabs-group">
            <button
              type="button"
              class="tab-btn"
              :class="{ active: media === 'image' }"
              @click="media = 'image'"
            >
              <Icon name="image" :size="16" />
              图片壁纸
            </button>
            <button
              type="button"
              class="tab-btn"
              :class="{ active: media === 'video' }"
              @click="media = 'video'"
            >
              <Icon name="play" :size="16" />
              动态视频
            </button>
          </div>

          <!-- 方向 -->
          <div class="select-group">
            <label>方向：</label>
            <select v-model="orientation" class="a-select">
              <option value="all">全部方向</option>
              <option value="landscape">横屏 (PC)</option>
              <option value="portrait">竖屏 (手机)</option>
              <option value="squarish">方形</option>
            </select>
          </div>

          <!-- 供应商 -->
          <div class="select-group">
            <label>供应商：</label>
            <select v-model="provider" class="a-select">
              <option value="all">全部已配置供应商</option>
              <option value="unsplash">Unsplash</option>
              <option value="pixabay">Pixabay</option>
              <option value="pexels">Pexels</option>
              <option value="demo">Demo 示例</option>
            </select>
          </div>

          <!-- 预置分类 -->
          <div class="select-group">
            <label>分类：</label>
            <select v-model="selectedCategory" class="a-select">
              <option value="">全部预设分类</option>
              <option v-for="cat in catalog.categories" :key="cat.id" :value="cat.id">
                {{ cat.name }}
              </option>
            </select>
          </div>
        </div>
      </div>
    </header>

    <!-- 列表展示区 -->
    <div class="body">
      <WallpaperGrid v-if="loading && items.length === 0" :skeleton="true" />
      <StateBlock
        v-else-if="error && items.length === 0"
        kind="error"
        :title="error"
        action="重新尝试"
        @action="loadFirstPage"
      />
      <StateBlock
        v-else-if="items.length === 0"
        kind="empty"
        title="未检索到相关壁纸"
        hint="请尝试调整搜索关键词或更换供应商"
      />
      <div v-else class="grid-wrap">
        <WallpaperGrid
          :items="items"
          :collector-mode="true"
          :collected-ids="collectedIds"
          @open="onOpen"
          @collect="openCollectModal"
        />

        <div v-if="hasMore" class="load-more-bar">
          <button
            type="button"
            class="a-btn load-more-btn"
            :disabled="loadingMore"
            @click="loadMore"
          >
            {{ loadingMore ? '正在加载更多...' : '加载下一页' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 采集入库弹窗 -->
    <div v-if="targetItem" class="modal-backdrop" @click="targetItem = null">
      <div class="modal-card" @click.stop>
        <div class="modal-header">
          <h3>采集入库</h3>
          <button type="button" class="close-btn" @click="targetItem = null">
            <Icon name="close" :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <div class="preview-mini">
            <img :src="targetItem.preview_url" :alt="targetItem.title" />
            <div class="mini-info">
              <div class="mini-title">{{ targetItem.title }}</div>
              <div class="mini-meta">
                来源: {{ targetItem.provider }} · {{ targetItem.resolution.width }}x{{ targetItem.resolution.height }}
              </div>
            </div>
          </div>

          <div class="form-group">
            <label class="form-label">指定分类：</label>
            <select v-model="importCategory" class="a-select full">
              <option value="">未分类 (保留默认)</option>
              <option v-for="cat in catalog.categories" :key="cat.id" :value="cat.id">
                {{ cat.name }}
              </option>
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">入库状态：</label>
            <div class="radio-row">
              <label class="radio-item">
                <input v-model="importStatus" type="radio" value="published" />
                <span>立即对外上架 (Published)</span>
              </label>
              <label class="radio-item">
                <input v-model="importStatus" type="radio" value="pending" />
                <span>仅入库待审核 (Pending)</span>
              </label>
            </div>
          </div>
        </div>

        <div class="modal-foot">
          <button type="button" class="a-btn" @click="targetItem = null">取消</button>
          <button
            type="button"
            class="a-btn primary"
            :disabled="importing"
            @click="confirmImport"
          >
            {{ importing ? '正在入库...' : '确认采集入库' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 全屏预览 -->
    <Viewer />
  </div>
</template>

<style scoped>
.collector-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 32px 36px 60px;
  max-width: 1600px;
}

.head {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.desc {
  font-size: 14px;
  color: var(--muted);
  margin-top: 4px;
}

.filters-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 18px 22px;
  border-radius: var(--r-card);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--line);
}

.search-box {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--line);
  border-radius: var(--r-input);
  padding: 4px 6px 4px 14px;
}

.search-icon {
  color: var(--muted);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--ink);
  font-size: 14px;
}

.controls-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 16px;
}

.tabs-group {
  display: inline-flex;
  padding: 3px;
  border-radius: var(--r-input);
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--line);
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  font-size: 13px;
  border-radius: calc(var(--r-input) - 2px);
  color: var(--muted);
  cursor: pointer;
  transition: all 0.2s;
}

.tab-btn.active {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
  font-weight: 500;
}

.select-group {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--muted);
}

.select-group .a-select {
  padding: 6px 12px;
  font-size: 13px;
}

.load-more-bar {
  display: flex;
  justify-content: center;
  margin-top: 36px;
}

.load-more-btn {
  min-width: 180px;
  padding: 10px 24px;
}

/* Modal */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.65);
  -webkit-backdrop-filter: blur(8px);
  backdrop-filter: blur(8px);
}

.modal-card {
  width: 90%;
  max-width: 480px;
  background: #18181c;
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-header h3 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.preview-mini {
  display: flex;
  gap: 14px;
  padding: 10px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
}

.preview-mini img {
  width: 90px;
  height: 60px;
  object-fit: cover;
  border-radius: 6px;
}

.mini-title {
  font-size: 13px;
  font-weight: 500;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.mini-meta {
  font-size: 12px;
  color: var(--muted);
  margin-top: 4px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
}

.radio-row {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.radio-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}

.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.full {
  width: 100%;
}
</style>
