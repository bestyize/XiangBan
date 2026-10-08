<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import ConfirmButton from '@/components/admin/ConfirmButton.vue'
import Icon from '@/components/Icon.vue'
import StateBlock from '@/components/StateBlock.vue'
import {
  deleteWallpaper,
  listAdminWallpapers,
  updateWallpaperMeta,
  updateWallpaperStatus,
} from '@/admin/api'
import { toast } from '@/composables/toast'
import { useCatalog } from '@/stores/catalog'
import type { WallpaperItem } from '@/types'

const catalog = useCatalog()

const items = ref<WallpaperItem[]>([])
const total = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)

// 过滤器
const status = ref('all')
const category = ref('all')
const mediaType = ref('all')
const q = ref('')
const page = ref(1)
const perPage = ref(20)

// 编辑模态框
const editItem = ref<WallpaperItem | null>(null)
const editTitle = ref('')
const editCategory = ref('')
const editTagsText = ref('')
const editStatus = ref('published')
const saving = ref(false)

onMounted(async () => {
  if (catalog.categories.length === 0) {
    await catalog.load()
  }
  loadList()
})

async function loadList() {
  loading.value = true
  error.value = null
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
  } catch (err: any) {
    error.value = err?.message || '加载图库列表失败'
  } finally {
    loading.value = false
  }
}

watch([status, category, mediaType], () => {
  page.value = 1
  loadList()
})

function handleSearch() {
  page.value = 1
  loadList()
}

function prevPage() {
  if (page.value > 1) {
    page.value--
    loadList()
  }
}

function nextPage() {
  if (page.value * perPage.value < total.value) {
    page.value++
    loadList()
  }
}

async function quickToggleStatus(item: WallpaperItem) {
  const nextStatus = item.status === 'published' ? 'offline' : 'published'
  try {
    await updateWallpaperStatus(item.id, nextStatus)
    item.status = nextStatus
    toast(nextStatus === 'published' ? '已上架对外展示' : '已下架')
  } catch (err: any) {
    toast(err?.message || '切换状态失败')
  }
}

function openEdit(item: WallpaperItem) {
  editItem.value = item
  editTitle.value = item.title
  editCategory.value = item.category || ''
  editTagsText.value = item.tags.join(', ')
  editStatus.value = item.status || 'published'
}

async function confirmSaveEdit() {
  if (!editItem.value) return
  saving.value = true
  const tags = editTagsText.value
    .split(/[,，]/)
    .map((s) => s.trim())
    .filter(Boolean)

  try {
    await updateWallpaperMeta(editItem.value.id, {
      title: editTitle.value.trim() || editItem.value.title,
      category: editCategory.value || undefined,
      tags,
      status: editStatus.value,
    })
    editItem.value.title = editTitle.value.trim()
    editItem.value.category = editCategory.value || undefined
    editItem.value.tags = tags
    editItem.value.status = editStatus.value
    toast('壁纸信息更新成功')
    editItem.value = null
  } catch (err: any) {
    toast(err?.message || '更新壁纸失败')
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: string) {
  try {
    await deleteWallpaper(id)
    items.value = items.value.filter((it) => it.id !== id)
    total.value = Math.max(0, total.value - 1)
    toast('已删除壁纸')
  } catch (err: any) {
    toast(err?.message || '删除失败')
  }
}
</script>

<template>
  <div class="wallpapers-view">
    <header class="head">
      <div class="title-wrap">
        <h1 class="a-h1">壁纸图库管理</h1>
        <p class="desc">管理数据库中已采集与自主上架的壁纸及视频资源（共 {{ total }} 条）。</p>
      </div>

      <!-- 筛选栏 -->
      <div class="filter-bar">
        <form class="search-form" @submit.prevent="handleSearch">
          <input
            v-model="q"
            type="search"
            placeholder="搜索标题或标签..."
            class="a-input search-input"
          />
          <button type="submit" class="a-btn">搜索</button>
        </form>

        <div class="filters-row">
          <select v-model="status" class="a-select">
            <option value="all">全部状态</option>
            <option value="published">已上架 (公开)</option>
            <option value="pending">待审核 (未公开)</option>
            <option value="offline">已下架</option>
          </select>

          <select v-model="mediaType" class="a-select">
            <option value="all">全部媒体类型</option>
            <option value="image">静态图片</option>
            <option value="video">动态视频</option>
          </select>

          <select v-model="category" class="a-select">
            <option value="all">全部分类</option>
            <option v-for="cat in catalog.categories" :key="cat.id" :value="cat.id">
              {{ cat.name }}
            </option>
          </select>

          <RouterLink to="/admin/upload" class="a-btn primary upload-link">
            <Icon name="upload" :size="16" />
            自定义上架
          </RouterLink>
        </div>
      </div>
    </header>

    <div class="table-wrap">
      <div v-if="loading && items.length === 0" class="loading-state">
        正在加载图库数据...
      </div>
      <StateBlock
        v-else-if="error && items.length === 0"
        kind="error"
        :title="error"
        action="重新加载"
        @action="loadList"
      />
      <StateBlock
        v-else-if="items.length === 0"
        kind="empty"
        title="暂无壁纸记录"
        hint="可以前往「采集工作台」挑选外部壁纸入库，或点击右上角「自定义上架」上传本地资源。"
      />
      <table v-else class="data-table">
        <thead>
          <tr>
            <th style="width: 80px">预览</th>
            <th>标题与标签</th>
            <th style="width: 100px">分类</th>
            <th style="width: 100px">类型/规格</th>
            <th style="width: 90px">来源</th>
            <th style="width: 110px">状态</th>
            <th style="width: 160px; text-align: right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in items" :key="item.id">
            <td>
              <div class="thumb-box">
                <img :src="item.preview_url" :alt="item.title" loading="lazy" />
                <span v-if="item.media_type === 'video'" class="video-tag">
                  <Icon name="play" :size="11" />
                </span>
              </div>
            </td>
            <td>
              <div class="item-title">{{ item.title }}</div>
              <div v-if="item.tags.length" class="tags-list">
                <span v-for="t in item.tags.slice(0, 4)" :key="t" class="tag">{{ t }}</span>
                <span v-if="item.tags.length > 4" class="tag-more">+{{ item.tags.length - 4 }}</span>
              </div>
            </td>
            <td>
              <span class="category-badge">{{ item.category || '未分类' }}</span>
            </td>
            <td>
              <div class="spec-text">
                {{ item.media_type === 'video' ? '视频' : '图片' }}
              </div>
              <div class="res-text">{{ item.resolution.width }}x{{ item.resolution.height }}</div>
            </td>
            <td>
              <span class="source-badge" :class="item.source_type">
                {{ item.source_type === 'custom' ? '自主上架' : item.provider }}
              </span>
            </td>
            <td>
              <span
                class="status-pill"
                :class="item.status"
                :title="item.status === 'published' ? '对外展示中' : '未对外展示'"
              >
                {{
                  item.status === 'published'
                    ? '● 已上架'
                    : item.status === 'pending'
                      ? '○ 待审核'
                      : '✕ 已下架'
                }}
              </span>
            </td>
            <td style="text-align: right">
              <div class="actions">
                <button
                  type="button"
                  class="action-btn"
                  :class="{ active: item.status === 'published' }"
                  :title="item.status === 'published' ? '点击下架' : '点击上架'"
                  @click="quickToggleStatus(item)"
                >
                  {{ item.status === 'published' ? '下架' : '上架' }}
                </button>
                <button type="button" class="action-btn" title="编辑信息" @click="openEdit(item)">
                  编辑
                </button>
                <ConfirmButton
                  label="删除"
                  confirm-label="确定删除？"
                  class="delete-btn"
                  @confirm="handleDelete(item.id)"
                />
              </div>
            </td>
          </tr>
        </tbody>
      </table>

      <!-- 底部翻页 -->
      <div v-if="total > perPage" class="pagination">
        <span class="page-info">
          第 {{ page }} 页 / 共 {{ Math.ceil(total / perPage) }} 页（{{ total }} 条）
        </span>
        <div class="page-btns">
          <button type="button" class="a-btn" :disabled="page <= 1" @click="prevPage">
            上一页
          </button>
          <button
            type="button"
            class="a-btn"
            :disabled="page * perPage >= total"
            @click="nextPage"
          >
            下一页
          </button>
        </div>
      </div>
    </div>

    <!-- 编辑模态框 -->
    <div v-if="editItem" class="modal-backdrop" @click="editItem = null">
      <div class="modal-card" @click.stop>
        <div class="modal-header">
          <h3>编辑壁纸信息</h3>
          <button type="button" class="close-btn" @click="editItem = null">
            <Icon name="close" :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <div class="form-group">
            <label class="form-label">标题：</label>
            <input v-model="editTitle" type="text" class="a-input full" />
          </div>

          <div class="form-group">
            <label class="form-label">所属分类：</label>
            <select v-model="editCategory" class="a-select full">
              <option value="">未分类</option>
              <option v-for="cat in catalog.categories" :key="cat.id" :value="cat.id">
                {{ cat.name }}
              </option>
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">标签（逗号分隔）：</label>
            <input
              v-model="editTagsText"
              type="text"
              placeholder="例如：风景, 治愈, 4K"
              class="a-input full"
            />
          </div>

          <div class="form-group">
            <label class="form-label">上架状态：</label>
            <select v-model="editStatus" class="a-select full">
              <option value="published">已上架 (Published 对外公开)</option>
              <option value="pending">待审核 (Pending 仅库内保存)</option>
              <option value="offline">已下架 (Offline 隐藏)</option>
            </select>
          </div>
        </div>

        <div class="modal-foot">
          <button type="button" class="a-btn" @click="editItem = null">取消</button>
          <button type="button" class="a-btn primary" :disabled="saving" @click="confirmSaveEdit">
            {{ saving ? '保存中...' : '保存修改' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wallpapers-view {
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

.filter-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 16px 20px;
  border-radius: var(--r-card);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--line);
}

.search-form {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-input {
  width: 260px;
}

.filters-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.upload-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.loading-state {
  padding: 60px 0;
  text-align: center;
  color: var(--muted);
  font-size: 14px;
}

.table-wrap {
  display: flex;
  flex-direction: column;
  gap: 16px;
  border-radius: var(--r-card);
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--line);
  padding: 8px 16px 20px;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.data-table th,
.data-table td {
  padding: 14px 12px;
  text-align: left;
  border-bottom: 1px solid var(--line);
}

.data-table th {
  color: var(--muted);
  font-weight: 500;
  font-size: 12px;
}

.thumb-box {
  position: relative;
  width: 72px;
  height: 48px;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.3);
}

.thumb-box img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.video-tag {
  position: absolute;
  right: 4px;
  bottom: 4px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 4px;
  padding: 2px 4px;
  color: #fff;
  display: flex;
  align-items: center;
}

.item-title {
  font-weight: 500;
  color: var(--ink);
  margin-bottom: 4px;
  max-width: 320px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag {
  font-size: 11px;
  background: rgba(255, 255, 255, 0.06);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--muted);
}

.tag-more {
  font-size: 11px;
  color: var(--muted);
}

.category-badge {
  font-size: 12px;
  color: var(--ink);
  background: rgba(255, 255, 255, 0.05);
  padding: 4px 8px;
  border-radius: 4px;
}

.spec-text {
  font-size: 12px;
  font-weight: 500;
}

.res-text {
  font-size: 11px;
  color: var(--muted);
}

.source-badge {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  color: var(--muted);
}

.source-badge.custom {
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
  font-weight: 500;
}

.status-pill {
  font-size: 12px;
  font-weight: 500;
}

.status-pill.published {
  color: #34d399;
}

.status-pill.pending {
  color: #fbbf24;
}

.status-pill.offline {
  color: #f87171;
}

.actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.action-btn {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--line);
  color: var(--ink);
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.action-btn.active {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border-color: rgba(239, 68, 68, 0.3);
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 16px;
}

.page-info {
  font-size: 13px;
  color: var(--muted);
}

.page-btns {
  display: flex;
  gap: 10px;
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

.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.full {
  width: 100%;
}
</style>
