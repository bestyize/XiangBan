<template>
  <div class="portal-wrapper">
    <!-- 极简顶栏 -->
    <header class="clean-navbar">
      <div class="nav-inner">
        <div class="nav-brand">
          <span class="logo-mark">❖</span>
          <div class="brand-text">
            <span class="brand-title">小伴应用精选</span>
            <span class="brand-sub">Curated Software</span>
          </div>
        </div>

        <div class="nav-links">
          <button class="nav-btn" @click="showManifesto = true">
            <ShieldCheck :size="14" />
            <span>选品准则</span>
          </button>
          <router-link to="/admin" class="nav-btn btn-admin-entry">
            <LayoutDashboard :size="14" />
            <span>管理后台</span>
          </router-link>
        </div>
      </div>
    </header>

    <!-- 极简 Hero 区域（呼吸感、大字体、搜索聚焦） -->
    <section class="clean-hero">
      <div class="hero-inner">
        <h1 class="hero-title">好在精选，乐于分享</h1>
        <p class="hero-desc">只收录真正干净、原生、好用的优质软件 · 拒绝流氓弹窗与捆绑全家桶</p>

        <!-- 居中聚合搜索框 -->
        <div class="hero-search-wrap">
          <Search :size="17" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索优质应用，如 Claude, Raycast, Typora, IINA..."
            @input="onSearchInput"
          />
          <button v-if="searchQuery" class="clear-btn" @click="clearSearch">
            <X :size="15" />
          </button>
        </div>
      </div>
    </section>

    <!-- 主内容区 -->
    <main class="clean-main">
      <!-- 单行沉浸式分类导航胶囊 (横向排列) -->
      <div class="category-pills-row">
        <button
          class="cat-pill"
          :class="{ active: selectedCategory === 'all' }"
          @click="selectCategory('all')"
        >
          <Grid :size="14" />
          <span>全部软件</span>
          <span class="cat-pill-count">{{ totalAppsCount }}</span>
        </button>
        <button
          v-for="cat in categories"
          :key="cat.slug"
          class="cat-pill"
          :class="{ active: selectedCategory === cat.slug }"
          @click="selectCategory(cat.slug)"
        >
          <component :is="getCategoryIcon(cat.icon)" :size="14" />
          <span>{{ cat.name }}</span>
          <span v-if="cat.app_count !== undefined" class="cat-pill-count">{{ cat.app_count }}</span>
        </button>
      </div>

      <!-- 搜索或筛选结果统计提示 -->
      <div v-if="searchQuery || selectedCategory !== 'all'" class="filter-status-row">
        <span class="status-tip">
          找到 <strong>{{ apps.length }}</strong> 款符合条件的精选应用
        </span>
        <button class="btn-clear-filter" @click="resetFilters">重置条件</button>
      </div>

      <!-- 应用卡片网格 -->
      <div v-if="loading" class="state-box">
        <div class="clean-spinner"></div>
        <p>正在载入精选应用库...</p>
      </div>

      <div v-else-if="apps.length > 0" class="cards-grid">
        <AppCard
          v-for="app in apps"
          :key="app.id"
          :app="app"
          @select="openAppDetail"
        />
      </div>

      <div v-else class="state-box empty-box">
        <p class="empty-title">未找到相关应用</p>
        <p class="empty-sub">我们坚持只收录最优质的软件，宁缺毋滥。</p>
        <button class="btn-secondary" @click="resetFilters">返回查看全部应用</button>
      </div>
    </main>

    <!-- 页脚 -->
    <footer class="clean-footer">
      <div class="footer-content">
        <p>小伴应用精选 · 只收录最优质的软件</p>
        <span class="copyright">© 2026 Xiaoban AppStore. 独立 · 极简 · 原生 · 开源</span>
      </div>
    </footer>

    <!-- 应用详情弹窗 -->
    <AppDetailModal
      v-if="activeApp"
      :app="activeApp"
      @close="activeApp = null"
    />

    <!-- 选品准则弹窗 -->
    <div v-if="showManifesto" class="modal-backdrop" @click="showManifesto = false">
      <div class="manifesto-card" @click.stop>
        <div class="manifesto-header">
          <div class="manifesto-title-wrap">
            <ShieldCheck :size="20" class="shield-icon" />
            <h3>小伴应用精选 · 准入与选品法则</h3>
          </div>
          <button class="btn-close" @click="showManifesto = false">
            <X :size="18" />
          </button>
        </div>
        <div class="manifesto-content">
          <div class="rule-block">
            <h4 class="title-green">四大准入法则（只收录怎样的软件）</h4>
            <ul>
              <li><strong>极简高颜值</strong>：界面干净，现代排版，视觉舒适。</li>
              <li><strong>原生极速</strong>：优先推荐 Swift、Rust、C++ 原生工具，启动秒开，不占系统后台。</li>
              <li><strong>尊重数据主权</strong>：倡导 Local-First，优先离线可用、不强制绑账号的良心软件。</li>
              <li><strong>真实解决痛点</strong>：单点极致，将一个具体功能做到行业天花板。</li>
            </ul>
          </div>
          <div class="rule-block">
            <h4 class="title-red">四大绝不原则（绝不妥协的红线）</h4>
            <ul>
              <li><strong>绝不收录捆绑全家桶</strong>：安装送全家桶的流氓软件一律封杀。</li>
              <li><strong>绝不收录开机弹窗与驻留广告</strong>：后台静默自启或右下角弹新闻的软件零容忍。</li>
              <li><strong>绝不收录诱导隐形扣费</strong>：表面免费下载、无提示锁死功能的恶意商业套路。</li>
              <li><strong>绝不过度索取隐私权限</strong>：无端要求麦克风、剪贴板、位置权限的工具坚决不收录。</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { AppItem, Category } from '../../types';
import { api } from '../../api';
import AppCard from '../../components/AppCard.vue';
import AppDetailModal from '../../components/AppDetailModal.vue';
import {
  Search,
  X,
  ShieldCheck,
  LayoutDashboard,
  Grid,
  Bot,
  Sparkles,
  Github,
  Rocket,
  Terminal,
  Feather,
  PenTool,
  Palette,
} from 'lucide-vue-next';

const categories = ref<Category[]>([]);
const apps = ref<AppItem[]>([]);
const loading = ref(true);
const searchQuery = ref('');
const selectedCategory = ref('all');
const activeApp = ref<AppItem | null>(null);
const showManifesto = ref(false);

const totalAppsCount = computed(() => {
  return categories.value.reduce((acc, cur) => acc + (cur.app_count || 0), 0);
});

function getCategoryIcon(iconName: string) {
  const map: Record<string, any> = {
    Bot,
    Sparkles,
    Github,
    Rocket,
    Terminal,
    Feather,
    PenTool,
    Palette,
  };
  return map[iconName] || Grid;
}

async function loadData() {
  loading.value = true;
  try {
    const [catList, appRes] = await Promise.all([
      api.getCategories(),
      api.getApps({
        category: selectedCategory.value === 'all' ? undefined : selectedCategory.value,
        search: searchQuery.value || undefined,
      }),
    ]);
    categories.value = catList;
    apps.value = appRes.apps;
  } catch (err) {
    console.error('加载应用列表失败:', err);
  } finally {
    loading.value = false;
  }
}

let searchTimer: any = null;
function onSearchInput() {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    loadData();
  }, 200);
}

function clearSearch() {
  searchQuery.value = '';
  loadData();
}

function selectCategory(catSlug: string) {
  selectedCategory.value = catSlug;
  loadData();
}

function resetFilters() {
  searchQuery.value = '';
  selectedCategory.value = 'all';
  loadData();
}

function openAppDetail(app: AppItem) {
  activeApp.value = app;
}

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.portal-wrapper {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f8fafc;
}

/* 顶栏 */
.clean-navbar {
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  position: sticky;
  top: 0;
  z-index: 50;
}

.nav-inner {
  max-width: 1200px;
  margin: 0 auto;
  height: 60px;
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.nav-brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo-mark {
  color: #2563eb;
  font-size: 20px;
}

.brand-title {
  font-size: 15px;
  font-weight: 600;
  color: #0f172a;
}

.brand-sub {
  font-size: 11px;
  color: #94a3b8;
  margin-left: 6px;
}

.nav-links {
  display: flex;
  align-items: center;
  gap: 10px;
}

.nav-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  color: #475569;
  padding: 6px 12px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.nav-btn:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.btn-admin-entry {
  background: #f1f5f9;
  color: #0f172a;
  font-weight: 500;
}

.btn-admin-entry:hover {
  background: #e2e8f0;
}

/* 极简 Hero */
.clean-hero {
  padding: 56px 24px 36px;
  text-align: center;
  background: linear-gradient(180deg, #f1f5f9 0%, #f8fafc 100%);
}

.hero-inner {
  max-width: 680px;
  margin: 0 auto;
}

.hero-title {
  font-size: 28px;
  font-weight: 700;
  color: #0f172a;
  letter-spacing: -0.3px;
  margin-bottom: 8px;
}

.hero-desc {
  font-size: 14px;
  color: #64748b;
  margin-bottom: 28px;
}

.hero-search-wrap {
  position: relative;
  display: flex;
  align-items: center;
  max-width: 540px;
  margin: 0 auto;
}

.search-icon {
  position: absolute;
  left: 16px;
  color: #94a3b8;
}

.hero-search-wrap input {
  width: 100%;
  height: 46px;
  padding: 0 40px 0 44px;
  background: #ffffff;
  border: 1px solid #cbd5e1;
  border-radius: 24px;
  font-size: 14px;
  color: #0f172a;
  outline: none;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.04);
  transition: all 0.2s ease;
}

.hero-search-wrap input:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
}

.clear-btn {
  position: absolute;
  right: 14px;
  color: #94a3b8;
  padding: 4px;
  border-radius: 50%;
}

.clear-btn:hover {
  color: #0f172a;
}

/* 主内容 */
.clean-main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 10px 24px 64px;
  width: 100%;
  flex: 1;
}

/* 单行胶囊分类导航 */
.category-pills-row {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow-x: auto;
  padding: 6px 2px 18px;
  scrollbar-width: none;
}

.category-pills-row::-webkit-scrollbar {
  display: none;
}

.cat-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  font-size: 13px;
  padding: 7px 14px;
  border-radius: 20px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  color: #475569;
  font-weight: 500;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.cat-pill:hover {
  border-color: #94a3b8;
  color: #0f172a;
}

.cat-pill.active {
  background: #0f172a;
  border-color: #0f172a;
  color: #ffffff;
}

.cat-pill-count {
  font-size: 11px;
  opacity: 0.7;
}

.cat-pill.active .cat-pill-count {
  opacity: 0.9;
}

/* 过滤统计 */
.filter-status-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  font-size: 13px;
  color: #64748b;
  padding: 0 4px;
}

.status-tip strong {
  color: #2563eb;
}

.btn-clear-filter {
  color: #2563eb;
  font-size: 13px;
}

.btn-clear-filter:hover {
  text-decoration: underline;
}

/* 卡片网格 */
.cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
}

/* 状态 */
.state-box {
  text-align: center;
  padding: 72px 20px;
}

.clean-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e2e8f0;
  border-top-color: #2563eb;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 14px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  margin-bottom: 6px;
}

.empty-sub {
  font-size: 13px;
  color: #64748b;
  margin-bottom: 18px;
}

/* 页脚 */
.clean-footer {
  border-top: 1px solid #e2e8f0;
  background: #ffffff;
  padding: 32px 24px;
  text-align: center;
  font-size: 13px;
  color: #64748b;
}

.copyright {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 6px;
  display: block;
}

/* 准则模态框 */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.45);
  backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.manifesto-card {
  background: #ffffff;
  width: 100%;
  max-width: 580px;
  border-radius: 14px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.12);
  overflow: hidden;
}

.manifesto-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  border-bottom: 1px solid #e2e8f0;
}

.manifesto-title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shield-icon {
  color: #2563eb;
}

.manifesto-title-wrap h3 {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
}

.btn-close {
  color: #94a3b8;
  padding: 4px;
  border-radius: 6px;
}

.btn-close:hover {
  color: #0f172a;
}

.manifesto-content {
  padding: 22px 24px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.rule-block h4 {
  font-size: 13px;
  margin-bottom: 8px;
  font-weight: 600;
}

.title-green {
  color: #059669;
}

.title-red {
  color: #dc2626;
}

.rule-block ul {
  list-style: none;
  font-size: 13px;
  color: #475569;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rule-block li strong {
  color: #0f172a;
}
</style>
