<template>
  <div class="admin-apps-page">
    <!-- 顶部操作与筛选栏 -->
    <div class="page-top-bar">
      <div class="filters-left">
        <!-- 状态快捷切换 Tab -->
        <div class="status-tabs">
          <button
            class="tab-btn"
            :class="{ active: selectedStatus === 'all' }"
            @click="setStatus('all')"
          >
            全部 ({{ apps.length }})
          </button>
          <button
            class="tab-btn tab-published"
            :class="{ active: selectedStatus === 'published' }"
            @click="setStatus('published')"
          >
            🟢 已上架 ({{ publishedCount }})
          </button>
          <button
            class="tab-btn tab-unlisted"
            :class="{ active: selectedStatus === 'unlisted' }"
            @click="setStatus('unlisted')"
          >
            🔴 已下架 ({{ unlistedCount }})
          </button>
        </div>

        <!-- 分类下拉筛选 -->
        <select v-model="selectedCategory" class="filter-select" @change="loadApps">
          <option value="all">全部分类</option>
          <option v-for="cat in categories" :key="cat.slug" :value="cat.slug">
            {{ cat.name }}
          </option>
        </select>

        <!-- 关键词搜索框 -->
        <div class="search-wrap">
          <Search :size="15" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索软件名或 slug..."
            @input="onSearchInput"
          />
        </div>
      </div>

      <!-- 右侧操作区：全量版本采集 + 新增 -->
      <div class="top-actions-right">
        <button
          class="btn-secondary"
          :disabled="syncingAll"
          @click="syncAll"
          title="自动从 GitHub Releases / Homebrew Cask 采集所有软件的最新版本与直接安装包"
        >
          <RefreshCw :size="14" :class="{ 'spin-anim': syncingAll }" />
          <span>{{ syncingAll ? '全量采集检测中...' : '一键全量采集版本' }}</span>
        </button>
        <button class="btn-primary" @click="openCreateModal">
          <Plus :size="16" />
          <span>收录新软件</span>
        </button>
      </div>
    </div>

    <!-- 提示消息 Toast -->
    <div v-if="toastMessage" class="toast-bar" :class="'toast-' + toastType">
      {{ toastMessage }}
    </div>

    <!-- 应用数据表格 -->
    <div class="table-card">
      <div v-if="loading" class="table-loading">
        <div class="spinner"></div>
        <p>正在同步应用列表...</p>
      </div>

      <table v-else-if="filteredApps.length > 0" class="apps-table">
        <thead>
          <tr>
            <th>软件信息</th>
            <th>分类</th>
            <th>最新版本 / 安装包直链</th>
            <th>平台支持</th>
            <th>资费</th>
            <th class="text-center">一键上架 / 下架</th>
            <th class="text-right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="app in filteredApps" :key="app.id" :class="{ 'row-unlisted': app.status !== 'published' }">
            <!-- 软件信息 -->
            <td class="col-app-info">
              <div class="app-info-wrap">
                <img :src="getAppIconUrl(app)" :alt="app.name" class="table-app-icon" @error="handleImgError($event, app.name, app.slug)" />
                <div class="app-texts">
                  <div class="name-line">
                    <span class="table-app-name">{{ app.name }}</span>
                  </div>
                  <span class="table-app-slug">{{ app.slug }}</span>
                </div>
              </div>
            </td>

            <!-- 分类 -->
            <td>
              <span class="table-cat-badge">{{ app.category_name || '精选' }}</span>
            </td>

            <!-- 最新版本与直接安装包下载直链 -->
            <td>
              <div class="version-direct-cell">
                <div class="version-head-row">
                  <span class="version-pill">{{ app.version ? 'v' + app.version : '待采集版本' }}</span>
                  <span v-if="app.file_size" class="size-pill">{{ app.file_size }}</span>
                </div>
                <div class="pkg-links-row">
                  <a
                    v-if="app.mac_download_url"
                    :href="app.mac_download_url"
                    target="_blank"
                    class="direct-tag tag-mac"
                    title="点击可直接下载 macOS 安装包 (.dmg/.zip)"
                  >
                    🍏 Mac直链
                  </a>
                  <a
                    v-if="app.win_download_url"
                    :href="app.win_download_url"
                    target="_blank"
                    class="direct-tag tag-win"
                    title="点击可直接下载 Windows 安装包 (.exe/.msi)"
                  >
                    🪟 Win直链
                  </a>
                  <a
                    v-if="app.linux_download_url"
                    :href="app.linux_download_url"
                    target="_blank"
                    class="direct-tag tag-linux"
                    title="点击可直接下载 Linux 安装包 (.AppImage)"
                  >
                    🐧 Linux直链
                  </a>
                  <span v-if="!app.mac_download_url && !app.win_download_url && !app.linux_download_url" class="no-direct-tip">
                    暂无直接安装包 (点击右侧采集)
                  </span>
                </div>
              </div>
            </td>

            <!-- 平台支持 -->
            <td>
              <div class="platforms-badges">
                <span v-for="plat in (app.platforms || [])" :key="plat" class="plat-badge">
                  {{ plat }}
                </span>
              </div>
            </td>

            <!-- 资费 -->
            <td>
              <span v-if="app.pricing_type === 'open_source'" class="badge badge-green">开源</span>
              <span v-else-if="app.pricing_type === 'free'" class="badge badge-blue">免费</span>
              <span v-else-if="app.pricing_type === 'freemium'" class="badge badge-gray">内购</span>
              <span v-else class="badge badge-purple">买断</span>
            </td>

            <!-- 一键上架 / 下架 Switch 开关 (核心特性) -->
            <td class="text-center">
              <div class="status-switch-cell">
                <label class="switch-container">
                  <input
                    type="checkbox"
                    :checked="app.status === 'published'"
                    :disabled="togglingId === app.id"
                    @change="toggleStatus(app)"
                  />
                  <span class="switch-slider"></span>
                </label>
                <span class="status-text" :class="app.status === 'published' ? 'text-published' : 'text-unlisted'">
                  {{ app.status === 'published' ? '已上架' : '已下架' }}
                </span>
              </div>
            </td>

            <!-- 操作 -->
            <td class="text-right">
              <div class="action-cell">
                <!-- 采集版本与直链按钮 -->
                <button
                  class="btn-icon btn-sync"
                  :title="'自动采集 ' + app.name + ' 最新版本与下载直链'"
                  :disabled="syncingId === app.id"
                  @click="syncVersion(app)"
                >
                  <RefreshCw :size="15" :class="{ 'spin-anim': syncingId === app.id }" />
                </button>
                <button class="btn-icon" title="编辑软件" @click="openEditModal(app)">
                  <Edit3 :size="16" />
                </button>
                <button class="btn-icon btn-icon-danger" title="删除软件" @click="deleteApp(app)">
                  <Trash2 :size="16" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>

      <div v-else class="table-empty">
        <p>暂无符合筛选条件的应用</p>
      </div>
    </div>

    <!-- 新增 / 编辑弹窗 -->
    <AppEditModal
      v-if="modalVisible"
      :app="editingApp"
      :categories="categories"
      @close="modalVisible = false"
      @saved="onAppSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { AppItem, Category } from '../../types';
import { api } from '../../api';
import AppEditModal from './AppEditModal.vue';
import { Search, Plus, Edit3, Trash2, RefreshCw } from 'lucide-vue-next';
import { handleImageFallback, getAppIconUrl } from '../../utils/icon';

const apps = ref<AppItem[]>([]);
const categories = ref<Category[]>([]);
const loading = ref(true);
const selectedStatus = ref('all');
const selectedCategory = ref('all');
const searchQuery = ref('');

const modalVisible = ref(false);
const editingApp = ref<AppItem | null>(null);
const togglingId = ref<number | null>(null);
const syncingId = ref<number | null>(null);
const syncingAll = ref(false);

const toastMessage = ref('');
const toastType = ref<'success' | 'error'>('success');

const publishedCount = computed(() => apps.value.filter((a) => a.status === 'published').length);
const unlistedCount = computed(() => apps.value.filter((a) => a.status === 'unlisted').length);

const filteredApps = computed(() => {
  return apps.value.filter((app) => {
    // 状态过滤
    if (selectedStatus.value !== 'all' && app.status !== selectedStatus.value) {
      return false;
    }
    // 分类过滤
    if (selectedCategory.value !== 'all' && app.category_slug !== selectedCategory.value) {
      return false;
    }
    // 搜索过滤
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      const matchName = app.name.toLowerCase().includes(q);
      const matchSlug = app.slug.toLowerCase().includes(q);
      const matchSummary = app.summary?.toLowerCase().includes(q);
      if (!matchName && !matchSlug && !matchSummary) return false;
    }
    return true;
  });
});

async function loadApps() {
  loading.value = true;
  try {
    const [cats, appsRes] = await Promise.all([
      api.getCategories(),
      api.getAdminApps(),
    ]);
    categories.value = cats;
    apps.value = appsRes.apps;
  } catch (err: any) {
    showToast(err.message || '加载应用列表失败', 'error');
  } finally {
    loading.value = false;
  }
}

function setStatus(status: string) {
  selectedStatus.value = status;
}

let searchTimer: any = null;
function onSearchInput() {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    // filteredApps computed property automatically updates
  }, 100);
}

// 核心特性：一键上架 / 下架切换
async function toggleStatus(app: AppItem) {
  togglingId.value = app.id;
  const newStatus = app.status === 'published' ? 'unlisted' : 'published';
  try {
    await api.updateAppStatus(app.id, newStatus);
    app.status = newStatus;
    showToast(
      `应用《${app.name}》已成功${newStatus === 'published' ? '上架并在前台显示' : '下架（前台已隐藏）'}！`,
      'success'
    );
  } catch (err: any) {
    showToast(`切换状态失败: ${err.message}`, 'error');
  } finally {
    togglingId.value = null;
  }
}

// 单个应用一键采集最新版本和安装包直链
async function syncVersion(app: AppItem) {
  syncingId.value = app.id;
  try {
    const res = await api.syncAppVersion(app.id);
    if (res.version) app.version = res.version;
    if (res.mac_download_url) app.mac_download_url = res.mac_download_url;
    if (res.win_download_url) app.win_download_url = res.win_download_url;
    if (res.linux_download_url) app.linux_download_url = res.linux_download_url;
    if (res.file_size) app.file_size = res.file_size;
    showToast(`成功采集《${app.name}》最新版本 v${res.version || '最新'} 及直接安装包！`, 'success');
  } catch (err: any) {
    showToast(`采集失败: ${err.message || '源未找到可用直链，请手动配置'}`, 'error');
  } finally {
    syncingId.value = null;
  }
}

// 批量全量采集检测所有软件
async function syncAll() {
  syncingAll.value = true;
  try {
    const res = await api.syncAllVersions();
    showToast(`全量检测完成！共检查 ${res.total} 款软件，成功采集更新 ${res.updated} 款的最新安装包直链。`, 'success');
    loadApps();
  } catch (err: any) {
    showToast(`全量采集失败: ${err.message}`, 'error');
  } finally {
    syncingAll.value = false;
  }
}

function openCreateModal() {
  editingApp.value = null;
  modalVisible.value = true;
}

function openEditModal(app: AppItem) {
  editingApp.value = app;
  modalVisible.value = true;
}

async function deleteApp(app: AppItem) {
  if (!confirm(`确定要彻底删除软件《${app.name}》吗？此操作不可恢复。`)) return;
  try {
    await api.deleteApp(app.id);
    showToast(`已成功删除《${app.name}》`, 'success');
    loadApps();
  } catch (err: any) {
    showToast(`删除失败: ${err.message}`, 'error');
  }
}

function onAppSaved() {
  showToast('保存成功！', 'success');
  loadApps();
}

function showToast(msg: string, type: 'success' | 'error' = 'success') {
  toastMessage.value = msg;
  toastType.value = type;
  setTimeout(() => {
    toastMessage.value = '';
  }, 3000);
}

function handleImgError(e: Event, name: string, slug?: string) {
  handleImageFallback(e, name, slug);
}

onMounted(() => {
  loadApps();
});
</script>

<style scoped>
.admin-apps-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}

.filters-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.top-actions-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-tabs {
  display: flex;
  background: #e2e8f0;
  padding: 3px;
  border-radius: 8px;
}

.tab-btn {
  font-size: 13px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: 6px;
  color: #64748b;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  color: #1e293b;
}

.tab-btn.active {
  background: #ffffff;
  color: #0f172a;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  font-weight: 600;
}

.filter-select {
  padding: 7px 12px;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  border-radius: 8px;
  font-size: 13px;
  color: #334155;
  outline: none;
}

.search-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: #94a3b8;
}

.search-wrap input {
  padding: 7px 12px 7px 32px;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  border-radius: 8px;
  font-size: 13px;
  color: #1e293b;
  outline: none;
  width: 200px;
  transition: width 0.2s ease, border-color 0.2s ease;
}

.search-wrap input:focus {
  width: 240px;
  border-color: #2563eb;
}

/* Toast */
.toast-bar {
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.toast-success {
  background: #ecfdf5;
  color: #047857;
  border: 1px solid #a7f3d0;
}

.toast-error {
  background: #fef2f2;
  color: #b91c1c;
  border: 1px solid #fecaca;
}

/* 表格容器 */
.table-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.apps-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  text-align: left;
}

.apps-table th {
  background: #f8fafc;
  padding: 12px 18px;
  font-weight: 600;
  color: #475569;
  border-bottom: 1px solid #e2e8f0;
  white-space: nowrap;
}

.apps-table td {
  padding: 14px 18px;
  border-bottom: 1px solid #f1f5f9;
  vertical-align: middle;
}

.apps-table tr:hover {
  background: #f8fafc;
}

.row-unlisted {
  background: #fafafa;
  opacity: 0.85;
}

/* 软件基本信息单元格 */
.app-info-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
}

.table-app-icon {
  width: 40px;
  height: 40px;
  min-width: 40px;
  border-radius: 10px;
  object-fit: cover;
  border: 1px solid #e2e8f0;
  background: #f8fafc;
}

.name-line {
  display: flex;
  align-items: center;
  gap: 6px;
}

.table-app-name {
  font-weight: 600;
  color: #0f172a;
}

.table-app-slug {
  font-size: 11px;
  color: #94a3b8;
  font-family: monospace;
}

.table-cat-badge {
  font-size: 12px;
  color: #475569;
  background: #f1f5f9;
  padding: 3px 8px;
  border-radius: 4px;
}

/* 版本与直接安装包单元格 */
.version-direct-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.version-head-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.version-pill {
  font-size: 11px;
  font-weight: 600;
  color: #0f172a;
  background: #f1f5f9;
  padding: 1px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.size-pill {
  font-size: 11px;
  color: #64748b;
}

.pkg-links-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.direct-tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.tag-mac {
  background: #eff6ff;
  color: #1d4ed8;
}

.tag-mac:hover {
  background: #dbeafe;
}

.tag-win {
  background: #f0fdf4;
  color: #15803d;
}

.tag-win:hover {
  background: #dcfce7;
}

.tag-linux {
  background: #faf5ff;
  color: #7e22ce;
}

.tag-linux:hover {
  background: #f3e8ff;
}

.no-direct-tip {
  font-size: 11px;
  color: #94a3b8;
}

.platforms-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.plat-badge {
  font-size: 11px;
  color: #2563eb;
  background: #eff6ff;
  padding: 1px 5px;
  border-radius: 3px;
}

/* 一键上下架 Switch 开关样式 */
.status-switch-cell {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.switch-container {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  cursor: pointer;
}

.switch-container input {
  opacity: 0;
  width: 0;
  height: 0;
}

.switch-slider {
  position: absolute;
  inset: 0;
  background-color: #cbd5e1;
  border-radius: 24px;
  transition: 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.switch-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  border-radius: 50%;
  transition: 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

input:checked + .switch-slider {
  background-color: #10b981;
}

input:checked + .switch-slider:before {
  transform: translateX(20px);
}

.status-text {
  font-size: 11px;
  font-weight: 600;
}

.text-published {
  color: #059669;
}

.text-unlisted {
  color: #dc2626;
}

.action-cell {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.btn-icon {
  color: #64748b;
  padding: 6px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-icon:hover {
  background: #e2e8f0;
  color: #0f172a;
}

.btn-sync:hover {
  background: #eff6ff;
  color: #2563eb;
}

.btn-icon-danger:hover {
  background: #fee2e2;
  color: #dc2626;
}

.spin-anim {
  animation: spin 0.8s linear infinite;
}

.text-center {
  text-align: center;
}

.text-right {
  text-align: right;
}

.table-loading,
.table-empty {
  padding: 48px;
  text-align: center;
  color: #64748b;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e2e8f0;
  border-top-color: #2563eb;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 12px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
