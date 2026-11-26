<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import StateBlock from '@/components/StateBlock.vue'
import Viewer from '@/components/Viewer.vue'
import AdminMoreMenu, { type MoreMenuItem } from '@/components/admin/AdminMoreMenu.vue'
import AdminSelect, { type SelectOption } from '@/components/admin/AdminSelect.vue'
import TagChipEditor from '@/components/admin/TagChipEditor.vue'
import {
  deleteWallpaper,
  listAdminWallpapers,
  updateWallpaperMeta,
  updateWallpaperStatus,
} from '@/admin/api'
import { toast } from '@/composables/toast'
import { openViewer } from '@/composables/viewer'
import { useCatalog } from '@/stores/catalog'
import type { WallpaperItem } from '@/types'

const route = useRoute()
const router = useRouter()
const catalog = useCatalog()

const items = ref<WallpaperItem[]>([])
const total = ref(0)
const counts = ref({ all: 0, published: 0, pending: 0, offline: 0 })
const loading = ref(false)
const error = ref<string | null>(null)

const status = ref(typeof route.query.status === 'string' ? route.query.status : 'all')
const category = ref('all')
const mediaType = ref('all')
const q = ref('')
const viewMode = ref<'grid' | 'table'>('grid')
const page = ref(1)
const perPage = ref(24)

const selectedIds = ref<Set<string>>(new Set())
const batchBusy = ref(false)

// Slide-over Edit Drawer
const editItem = ref<WallpaperItem | null>(null)
const editTitle = ref('')
const editCategory = ref('')
const editTags = ref<string[]>([])
const editStatus = ref('published')
const saving = ref(false)

const mediaOptions: SelectOption[] = [
  { value: 'all', label: '全部类型' },
  { value: 'image', label: '静态图片', icon: 'image' },
  { value: 'video', label: '动态视频', icon: 'play' },
]

const categoryOptions = computed<SelectOption[]>(() => [
  { value: 'all', label: '全部分类' },
  ...catalog.categories.map((c) => ({ value: c.id, label: c.name })),
])

const editCategoryOptions = computed<SelectOption[]>(() => [
  { value: '', label: '未分类' },
  ...catalog.categories.map((c) => ({ value: c.id, label: c.name })),
])

const allSelected = computed(() => items.value.length > 0 && selectedIds.value.size === items.value.length)
const catName = (id?: string) => catalog.categories.find((c) => c.id === id)?.name || id || '未分类'
const statusLabel = (st?: string) => (st === 'published' ? '已上架' : st === 'pending' ? '待审核' : '已下架')
const statusDot = (st?: string) => (st === 'published' ? 'ok' : st === 'pending' ? 'warn' : 'danger')

async function loadCounts() {
  const [a, p, d, o] = await Promise.allSettled([
    listAdminWallpapers({ status: 'all', page: 1, per_page: 1 }),
    listAdminWallpapers({ status: 'published', page: 1, per_page: 1 }),
    listAdminWallpapers({ status: 'pending', page: 1, per_page: 1 }),
    listAdminWallpapers({ status: 'offline', page: 1, per_page: 1 }),
  ])
  if (a.status === 'fulfilled') counts.value.all = a.value.total
  if (p.status === 'fulfilled') counts.value.published = p.value.total
  if (d.status === 'fulfilled') counts.value.pending = d.value.total
  if (o.status === 'fulfilled') counts.value.offline = o.value.total
}

async function loadList() {
  loading.value = true
  error.value = null
  selectedIds.value = new Set()
  try {
    const res = await listAdminWallpapers({
      status: status.value,
      category: category.value,
      media_type: mediaType.value,
      q: q.value.trim() || undefined,
      page: page.value,
      per_page: perPage.value,
    })
    items.value = res.items
    total.value = res.total
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : '加载图库列表失败'
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  if (catalog.categories.length === 0) await catalog.load()
  await Promise.all([loadList(), loadCounts()])
})

watch([status, category, mediaType], () => {
  page.value = 1
  void loadList()
})

function handleSearch() {
  page.value = 1
  void loadList()
}

function prevPage() {
  if (page.value > 1) {
    page.value--
    void loadList()
  }
}

function nextPage() {
  if (page.value * perPage.value < total.value) {
    page.value++
    void loadList()
  }
}

function toggleSelect(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function toggleAll() {
  if (allSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(items.value.map((i) => i.id))
  }
}

function previewItem(item: WallpaperItem) {
  openViewer(router, { items }, item)
}

async function quickToggleStatus(item: WallpaperItem) {
  const nextStatus = item.status === 'published' ? 'offline' : 'published'
  try {
    await updateWallpaperStatus(item.id, nextStatus)
    item.status = nextStatus
    toast(nextStatus === 'published' ? '已上架对外展示' : '已下架')
    void loadCounts()
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '切换状态失败')
  }
}

function openEdit(item: WallpaperItem) {
  editItem.value = item
  editTitle.value = item.title
  editCategory.value = item.category || ''
  editTags.value = [...item.tags]
  editStatus.value = item.status || 'published'
}

async function confirmSaveEdit() {
  if (!editItem.value) return
  saving.value = true
  try {
    const cleanTitle = editTitle.value.trim() || editItem.value.title
    await updateWallpaperMeta(editItem.value.id, {
      title: cleanTitle,
      category: editCategory.value || undefined,
      tags: editTags.value,
      status: editStatus.value,
    })
    editItem.value.title = cleanTitle
    editItem.value.category = editCategory.value || undefined
    editItem.value.tags = [...editTags.value]
    editItem.value.status = editStatus.value
    toast('壁纸信息已保存')
    editItem.value = null
    void loadCounts()
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '更新壁纸失败')
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: string) {
  try {
    await deleteWallpaper(id)
    items.value = items.value.filter((it) => it.id !== id)
    total.value = Math.max(0, total.value - 1)
    const next = new Set(selectedIds.value)
    next.delete(id)
    selectedIds.value = next
    toast('已删除壁纸')
    void loadCounts()
  } catch (err: unknown) {
    toast(err instanceof Error ? err.message : '删除失败')
  }
}

async function batchSetStatus(nextStatus: 'published' | 'offline') {
  if (batchBusy.value || !selectedIds.value.size) return
  batchBusy.value = true
  const ids = [...selectedIds.value]
  let ok = 0
  for (const id of ids) {
    try {
      await updateWallpaperStatus(id, nextStatus)
      const it = items.value.find((x) => x.id === id)
      if (it) it.status = nextStatus
      ok++
    } catch { /* continue */ }
  }
  selectedIds.value = new Set()
  batchBusy.value = false
  toast(`已批量${nextStatus === 'published' ? '上架' : '下架'} ${ok} 项`)
  void loadCounts()
}

async function batchDelete() {
  if (batchBusy.value || !selectedIds.value.size) return
  batchBusy.value = true
  const ids = [...selectedIds.value]
  let ok = 0
  for (const id of ids) {
    try {
      await deleteWallpaper(id)
      ok++
    } catch { /* continue */ }
  }
  items.value = items.value.filter((x) => !ids.includes(x.id))
  total.value = Math.max(0, total.value - ok)
  selectedIds.value = new Set()
  batchBusy.value = false
  toast(`已删除 ${ok} 项`)
  void loadCounts()
}

function menuFor(item: WallpaperItem): MoreMenuItem[] {
  return [
    {
      label: item.status === 'published' ? '下架' : '立即上架',
      icon: 'bolt',
      action: () => void quickToggleStatus(item),
    },
    { label: '编辑信息', icon: 'edit', action: () => openEdit(item) },
    { label: '查看大图', icon: 'eye', action: () => previewItem(item) },
    { sep: true },
    {
      label: '永久删除',
      icon: 'trash',
      danger: true,
      desc: '从图库中永久删除，不可恢复',
      confirmText: '确认删除？',
      action: () => void handleDelete(item.id),
    },
  ]
}
</script>

<template>
  <div class="xb-page">
    <header class="xb-page-head">
      <div class="xb-tabs-lg" role="tablist">
        <button
          v-for="tab in [
            { k: 'all', l: '全部', n: counts.all },
            { k: 'published', l: '已上架', n: counts.published },
            { k: 'pending', l: '待审核', n: counts.pending },
            { k: 'offline', l: '已下架', n: counts.offline },
          ]"
          :key="tab.k"
          type="button"
          :class="{ 'is-active': status === tab.k }"
          @click="status = tab.k"
        >
          <span>{{ tab.l }}</span>
          <span class="tab-n">{{ tab.n }}</span>
        </button>
      </div>
      <div class="xb-page-actions">
        <RouterLink to="/admin/collector" class="xb-btn-ghost">
          <Icon name="compass" :size="14" /><span>采集</span>
        </RouterLink>
        <RouterLink to="/admin/upload" class="xb-btn-primary">
          <Icon name="plus" :size="14" /><span>自定义上架</span>
        </RouterLink>
      </div>
    </header>

    <div class="filter-bar">
      <form class="filter-l" @submit.prevent="handleSearch">
        <div class="xb-affix has-l" :class="{ 'has-r': !!q }" style="width: 220px">
          <span class="af-l"><Icon name="search" :size="14" /></span>
          <input v-model="q" type="search" class="xb-input" placeholder="搜索标题或标签…" />
          <button v-if="q" type="button" class="af-r" aria-label="清空" @click="q = ''; handleSearch()">
            <Icon name="close" :size="13" />
          </button>
        </div>
        <AdminSelect v-model="mediaType" :options="mediaOptions" />
        <AdminSelect v-model="category" :options="categoryOptions" />
        <button type="submit" class="xb-btn-soft">搜索</button>
      </form>

      <div class="filter-r">
        <button v-if="items.length" type="button" class="xb-btn-ghost is-sm" @click="toggleAll">
          {{ allSelected ? '取消全选' : '全选' }}
        </button>
        <div class="xb-seg" role="group" aria-label="视图切换">
          <button type="button" :class="{ 'is-active': viewMode === 'grid' }" title="画廊视图" @click="viewMode = 'grid'">
            <Icon name="grid" :size="14" />
          </button>
          <button type="button" :class="{ 'is-active': viewMode === 'table' }" title="列表视图" @click="viewMode = 'table'">
            <Icon name="list" :size="14" />
          </button>
        </div>
      </div>
    </div>

    <div v-if="loading && !items.length" class="xb-group xb-empty-box">正在加载图库…</div>
    <StateBlock
      v-else-if="error && !items.length"
      kind="error"
      :title="error"
      action="重试"
      @action="loadList"
    />
    <div v-else-if="!items.length" class="xb-group xb-empty-box">
      <Icon name="image" :size="28" />
      <strong>暂无匹配壁纸</strong>
      <button
        type="button"
        class="xb-btn-soft is-sm"
        @click="status = 'all'; mediaType = 'all'; category = 'all'; q = ''; handleSearch()"
      >
        重置筛选
      </button>
    </div>

    <div v-else-if="viewMode === 'grid'" class="wp-grid">
      <article
        v-for="item in items"
        :key="item.id"
        class="wp-card"
        :class="{ 'is-selected': selectedIds.has(item.id) }"
      >
        <div class="wp-thumb" @click="previewItem(item)">
          <img :src="item.preview_url" :alt="item.title" loading="lazy" />
          <button
            type="button"
            class="wp-sel"
            aria-label="选择壁纸"
            @click.stop="toggleSelect(item.id)"
          >
            <Icon v-if="selectedIds.has(item.id)" name="check" :size="13" />
          </button>
          <div class="wp-top-r">
            <span v-if="item.media_type === 'video'" class="wp-glass">
              <Icon name="play" :size="10" /><span>视频</span>
            </span>
            <span class="wp-glass xb-num">{{ item.resolution.width }}×{{ item.resolution.height }}</span>
          </div>
          <div class="wp-bot-l">
            <span class="wp-glass">{{ item.source_type === 'custom' ? '自主上架' : item.provider }}</span>
          </div>
        </div>
        <div class="wp-info">
          <div class="wp-info-l">
            <div class="wp-title" :title="item.title">{{ item.title }}</div>
            <div class="wp-sub">
              <i class="xb-dot-status" :class="statusDot(item.status)" />
              <span>{{ statusLabel(item.status) }}</span>
              <span>·</span>
              <span>{{ catName(item.category) }}</span>
            </div>
          </div>
          <AdminMoreMenu :items="menuFor(item)" />
        </div>
      </article>
    </div>

    <div v-else class="tbl-wrap">
      <table class="tbl">
        <thead>
          <tr>
            <th style="width: 40px">
              <button type="button" class="xb-check" :class="{ 'is-checked': allSelected }" @click="toggleAll">
                <Icon v-if="allSelected" name="check" :size="12" />
              </button>
            </th>
            <th style="width: 92px">预览</th>
            <th>标题</th>
            <th>分类</th>
            <th>规格</th>
            <th>来源</th>
            <th>状态</th>
            <th style="text-align: right; width: 100px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in items" :key="item.id" :class="{ 'is-selected': selectedIds.has(item.id) }">
            <td>
              <button
                type="button"
                class="xb-check"
                :class="{ 'is-checked': selectedIds.has(item.id) }"
                @click="toggleSelect(item.id)"
              >
                <Icon v-if="selectedIds.has(item.id)" name="check" :size="12" />
              </button>
            </td>
            <td>
              <div class="mini-thumb" @click="previewItem(item)">
                <img :src="item.preview_url" :alt="item.title" loading="lazy" />
              </div>
            </td>
            <td>
              <div class="row-title" :title="item.tags.join(' / ')">{{ item.title }}</div>
            </td>
            <td>{{ catName(item.category) }}</td>
            <td class="xb-num sub-col">{{ item.resolution.width }}×{{ item.resolution.height }}</td>
            <td><span class="xb-pill">{{ item.source_type === 'custom' ? '自主上架' : item.provider }}</span></td>
            <td>
              <span class="xb-pill" :class="`is-${statusDot(item.status)}`">
                <i class="dot" />{{ statusLabel(item.status) }}
              </span>
            </td>
            <td style="text-align: right">
              <div class="tbl-acts">
                <button type="button" class="xb-btn-ghost is-sm" @click="openEdit(item)">编辑</button>
                <AdminMoreMenu :items="menuFor(item)" />
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="total > perPage" class="pager">
      <span class="sub-col xb-num">第 {{ page }} / {{ Math.ceil(total / perPage) }} 页（共 {{ total }} 条）</span>
      <div class="pager-btns">
        <button type="button" class="xb-btn-soft is-sm" :disabled="page <= 1" @click="prevPage">上一页</button>
        <button type="button" class="xb-btn-soft is-sm" :disabled="page * perPage >= total" @click="nextPage">
          下一页
        </button>
      </div>
    </div>

    <div v-if="selectedIds.size > 0" class="xb-batch-bar">
      <span>已选 <strong class="xb-num">{{ selectedIds.size }}</strong> 项</span>
      <button type="button" class="xb-batch-btn" :disabled="batchBusy" @click="batchSetStatus('published')">上架</button>
      <button type="button" class="xb-batch-btn" :disabled="batchBusy" @click="batchSetStatus('offline')">下架</button>
      <button type="button" class="xb-batch-btn is-danger-fill" :disabled="batchBusy" @click="batchDelete">删除</button>
      <button type="button" class="xb-batch-btn" @click="selectedIds = new Set()">取消</button>
    </div>

    <!-- 20px Slide-over Edit Drawer -->
    <Teleport to="body">
      <div v-if="editItem" class="xb-backdrop" @click.self="editItem = null">
        <form class="xb-drawer" @submit.prevent="confirmSaveEdit">
          <header class="xb-ov-head">
            <h2>编辑壁纸</h2>
            <button type="button" class="xb-icon-btn" aria-label="关闭" @click="editItem = null">
              <Icon name="close" :size="18" />
            </button>
          </header>

          <div class="xb-ov-body">
            <div class="drawer-preview">
              <img :src="editItem.preview_url" :alt="editItem.title" />
            </div>

            <label class="xb-field">
              <span class="xb-field-label">标题</span>
              <input v-model="editTitle" type="text" class="xb-input" required />
            </label>

            <div class="xb-field">
              <span class="xb-field-label">所属分类</span>
              <AdminSelect v-model="editCategory" :options="editCategoryOptions" full />
            </div>

            <div class="xb-field">
              <span class="xb-field-label">状态</span>
              <div class="xb-seg" style="align-self: flex-start">
                <button
                  v-for="s in [
                    { v: 'published', l: '已上架' },
                    { v: 'pending', l: '待审核' },
                    { v: 'offline', l: '已下架' },
                  ]"
                  :key="s.v"
                  type="button"
                  :class="{ 'is-active': editStatus === s.v }"
                  @click="editStatus = s.v"
                >
                  {{ s.l }}
                </button>
              </div>
            </div>

            <div class="xb-field">
              <span class="xb-field-label">标签</span>
              <TagChipEditor v-model="editTags" />
            </div>
          </div>

          <footer class="xb-ov-foot">
            <button type="button" class="xb-btn-ghost" @click="editItem = null">取消</button>
            <button type="submit" class="xb-btn-primary" :disabled="saving">
              {{ saving ? '保存中…' : '保存' }}
            </button>
          </footer>
        </form>
      </div>
    </Teleport>

    <Viewer />
  </div>
</template>

<style scoped>
.filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}
.filter-l,
.filter-r {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
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
.wp-thumb img {
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
  align-items: center;
  gap: 5px;
}
.wp-bot-l {
  position: absolute;
  bottom: 8px;
  left: 8px;
  z-index: 3;
  display: flex;
  align-items: center;
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
.wp-info {
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.wp-info-l {
  min-width: 0;
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

.tbl-wrap {
  background: var(--ui-surface-sunken);
  border-radius: 12px;
  overflow-x: auto;
}
.tbl {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.tbl th,
.tbl td {
  padding: 10px 14px;
  text-align: left;
  border-top: 1px solid var(--ui-line);
  vertical-align: middle;
}
.tbl thead th {
  border-top: 0;
  font-size: 11.5px;
  font-weight: 550;
  color: var(--ui-text-tertiary);
  white-space: nowrap;
}
.tbl tbody tr:hover { background: var(--ui-surface-hover); }
.tbl tbody tr.is-selected { background: var(--ui-soft); }
.mini-thumb {
  width: 76px;
  height: 48px;
  border-radius: 7px;
  overflow: hidden;
  cursor: pointer;
  background: var(--ui-control);
}
.mini-thumb img { width: 100%; height: 100%; object-fit: cover; }
.row-title {
  font-weight: 550;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sub-col { color: var(--ui-text-secondary); font-size: 12.5px; }
.tbl-acts { display: inline-flex; align-items: center; gap: 4px; }

.pager {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.pager-btns { display: flex; gap: 8px; }

.drawer-preview {
  border-radius: 10px;
  overflow: hidden;
  aspect-ratio: 16 / 9;
  background: var(--ui-control);
}
.drawer-preview img { width: 100%; height: 100%; object-fit: cover; }

@media (max-width: 1100px) { .wp-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
@media (max-width: 760px) { .wp-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; } }
</style>
