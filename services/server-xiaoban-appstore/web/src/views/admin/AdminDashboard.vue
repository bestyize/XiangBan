<template>
  <div class="dashboard-page">
    <!-- 数据概览指标卡片 -->
    <div class="kpi-grid">
      <div class="kpi-card">
        <div class="kpi-icon-wrap kpi-blue">
          <Package :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">收录软件总数</span>
          <span class="kpi-value">{{ stats.total_apps }}</span>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-icon-wrap kpi-green">
          <CheckCircle2 :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">正常上架中</span>
          <span class="kpi-value">{{ stats.published_apps }}</span>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-icon-wrap kpi-red">
          <EyeOff :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">已下架保护</span>
          <span class="kpi-value">{{ stats.unlisted_apps }}</span>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-icon-wrap kpi-amber">
          <Download :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">累计下载点击</span>
          <span class="kpi-value">{{ stats.total_downloads }}</span>
        </div>
      </div>
    </div>

    <!-- 8 特色分类与体系分布 -->
    <div class="section-card">
      <div class="section-head">
        <div class="head-title">
          <Tag :size="18" />
          <h3>8 大创新分类体系收录分布</h3>
        </div>
        <router-link to="/admin/apps" class="link-more">
          前往应用库管理
          <ArrowRight :size="14" />
        </router-link>
      </div>

      <div class="category-grid">
        <div v-for="cat in categories" :key="cat.id" class="cat-stat-card">
          <div class="cat-card-top">
            <span class="cat-card-name">{{ cat.name }}</span>
            <span class="cat-card-count">{{ cat.app_count || 0 }} 款</span>
          </div>
          <p class="cat-card-desc">{{ cat.description }}</p>
        </div>
      </div>
    </div>

    <!-- 快捷操作与说明 -->
    <div class="quick-tips-card">
      <div class="tips-icon">💡</div>
      <div class="tips-content">
        <h4>管理后台使用小提示</h4>
        <p>• 在<strong>「应用库管理」</strong>中，通过点击绿色/灰色开关可实现<strong>一键上架 / 下架</strong>，下架的应用在前台即时隐藏，保障选品灵活性与时效性。</p>
        <p>• 推荐收录时请填写详细的<strong>“编辑推荐语”</strong>，保持像腾讯柠檬精选一样真诚、有见地的推荐风格。</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { AdminStats, Category } from '../../types';
import { api } from '../../api';
import {
  Package,
  CheckCircle2,
  EyeOff,
  Download,
  Tag,
  ArrowRight,
} from 'lucide-vue-next';

const stats = ref<AdminStats>({
  total_apps: 0,
  published_apps: 0,
  unlisted_apps: 0,
  total_categories: 0,
  total_downloads: 0,
});

const categories = ref<Category[]>([]);

async function loadDashboard() {
  try {
    const [st, cats] = await Promise.all([
      api.getAdminStats(),
      api.getCategories(),
    ]);
    stats.value = st;
    categories.value = cats;
  } catch (err) {
    console.error('加载仪表盘失败:', err);
  }
}

onMounted(() => {
  loadDashboard();
});
</script>

<style scoped>
.dashboard-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.kpi-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 18px;
}

.kpi-card {
  background: #ffffff;
  padding: 22px;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.kpi-icon-wrap {
  width: 50px;
  height: 50px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.kpi-blue {
  background: #eff6ff;
  color: #2563eb;
}

.kpi-green {
  background: #ecfdf5;
  color: #059669;
}

.kpi-red {
  background: #fef2f2;
  color: #dc2626;
}

.kpi-amber {
  background: #f1f5f9;
  color: #0f172a;
}

.kpi-info {
  display: flex;
  flex-direction: column;
}

.kpi-label {
  font-size: 13px;
  color: #64748b;
  margin-bottom: 2px;
}

.kpi-value {
  font-size: 24px;
  font-weight: 700;
  color: #0f172a;
}

/* 创新分类卡片 */
.section-card {
  background: #ffffff;
  padding: 24px;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
}

.head-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #0f172a;
}

.head-title h3 {
  font-size: 16px;
  font-weight: 600;
}

.link-more {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: #2563eb;
  font-weight: 500;
}

.link-more:hover {
  text-decoration: underline;
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
}

.cat-stat-card {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 14px 16px;
  transition: all 0.15s ease;
}

.cat-stat-card:hover {
  border-color: #2563eb;
  background: #ffffff;
  transform: translateY(-2px);
}

.cat-card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.cat-card-name {
  font-weight: 600;
  color: #1e293b;
  font-size: 14px;
}

.cat-card-count {
  font-size: 12px;
  font-weight: 600;
  background: #e2e8f0;
  color: #475569;
  padding: 2px 7px;
  border-radius: 10px;
}

.cat-card-desc {
  font-size: 12px;
  color: #64748b;
  line-height: 1.45;
}

/* 贴士卡片 */
.quick-tips-card {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 18px 22px;
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.tips-icon {
  font-size: 24px;
  line-height: 1;
}

.tips-content h4 {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 6px;
}

.tips-content p {
  font-size: 13px;
  color: #475569;
  line-height: 1.6;
}
</style>
